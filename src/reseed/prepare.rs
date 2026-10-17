//! Source-only upstream preparation, with no durable progress or recovery state.
use super::*;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Settings {
    pub remote: Option<String>,
    pub command: Option<Vec<String>>,
    pub commands: Option<Vec<Vec<String>>>,
}

impl Settings {
    pub(super) fn argv(&self) -> Result<Vec<Vec<String>>> {
        if self.command.is_some() && self.commands.is_some() {
            bail!("freshen.command and freshen.commands cannot both be present, even when empty");
        }
        let argv = match (&self.command, &self.commands) {
            (Some(argv), None) => vec![argv.clone()],
            (None, Some(argv)) => argv.clone(),
            _ => Vec::new(),
        };
        for (index, command) in argv.iter().enumerate() {
            if command.is_empty() || command[0].is_empty() || command.iter().any(|arg| arg.contains('\0')) {
                bail!("freshen command {} must be a nonempty argv with a nonempty program and no NUL bytes", index + 1);
            }
        }
        Ok(argv)
    }
}

fn configured_argv(settings: Option<&Settings>) -> Result<Vec<Vec<String>>> {
    settings.map(Settings::argv).transpose().map(Option::unwrap_or_default)
}

pub(super) fn preflight(metadata: &WorkspaceMetadataStore, inventory: &Inventory, source: &Path, settings: Option<&Settings>) -> Result<()> {
    let context = checked_context(&JjClient::new(source))?;
    let name = context.jj.workspace_name.as_deref().context("source workspace name missing")?;
    if inventory.root(name)? != source {
        bail!("source registration changed");
    }
    validate_ownership(source, name, metadata)?;
    configured_argv(settings)?;
    Ok(())
}

fn validate_ownership(source: &Path, name: &str, metadata: &WorkspaceMetadataStore) -> Result<()> {
    let record = metadata.get(name)?;
    if record.as_ref().is_some_and(|m| m.external_owner.is_some() || m.owned_git_worktree.is_some()) || crate::ownership::detect(source)?.is_some() {
        bail!("freshen does not mutate external/app-owned or linked Git donors");
    }
    crate::ownership::ensure_removal_allowed(&JjClient::new(source), name, Some(source), record.as_ref())?;
    Ok(())
}

pub(super) fn describe(source: &Path, selected: &SourceSelection, settings: Option<&Settings>, ignored: &[String]) -> Result<String> {
    Ok(format!(
        "Prepare source {:?}; pinned {:?} bookmark {:?}@{:?}; prospective post-fetch goal {:?} (currently {:?}); preparation argv sequence {:?}, cwd {:?}; ignored selection {:?}; {}",
        source,
        selected.kind,
        selected.name,
        selected.remote,
        selected.source_at,
        selected.old_goal,
        configured_argv(settings)?,
        source,
        ignored,
        completion_summary(settings)?,
    ))
}

pub(super) fn completion_summary(settings: Option<&Settings>) -> Result<String> {
    Ok(if configured_argv(settings)?.is_empty() {
        "tracked-source preparation only; no configured commands; selected ignored payload is existing data, not freshly built"
    } else {
        "trusted configured programs, sequentially (no implicit shell); commands must tolerate retry when prepare is run again"
    }
    .into())
}

struct SourceIdentity {
    root: Identity,
    jj: Identity,
    repository: PathBuf,
    name: String,
}

impl SourceIdentity {
    fn capture(source: &Path) -> Result<Self> {
        Ok(Self {
            root: Identity::read(source)?,
            jj: Identity::read(&source.join(".jj"))?,
            repository: repository(&JjClient::new(source))?,
            name: checked_context(&JjClient::new(source))?
                .jj
                .workspace_name
                .context("source workspace name missing")?,
        })
    }

    fn verify(&self, client: &JjClient) -> Result<()> {
        self.root.verify(client.cwd())?;
        self.jj.verify(&client.cwd().join(".jj"))?;
        let context = checked_context(client)?;
        if repository(client)? != self.repository || context.jj.workspace_name.as_ref() != Some(&self.name) {
            bail!("preparation source/repository registration changed");
        }
        let metadata = WorkspaceMetadataStore::from_repo_config_path(client.repo_config_path()?)?;
        validate_ownership(client.cwd(), &self.name, &metadata)?;
        ensure_quiescence(client)?;
        require_full(client)?;
        validate_contents(client.cwd(), true)
    }
}

pub(super) fn execute(
    source: &Path,
    settings: Option<&Settings>,
    ignored_patterns: &[String],
    selected: SourceSelection,
    diagnostic: &mut dyn FnMut(&str),
) -> Result<String> {
    let identity = SourceIdentity::capture(source)?;
    let client = JjClient::new(source);
    let commands = configured_argv(settings)?;
    let result = (|| {
        identity.verify(&client)?;
        selected.verify_source_at(&client, &client.operation_id()?)?;
        let old_entries = entries(&client, &selected.operation, &selected.source.commit_id)?;
        verify_disk(&client, &selected.operation, &selected.source.commit_id, source, &old_entries)?;
        diagnostic(&format!(
            "Prepare: fetching remote {:?} bookmark {:?} in {:?}; previous work preserved at {}",
            selected.remote, selected.name, source, selected.source.commit_id,
        ));
        let output = client.fetch_bookmark(&selected.remote, &selected.name)?;
        diagnostics(output.stderr());
        let operation = client.operation_id()?;
        let goal = selected.resolve_fetched(&client, &operation)?;
        require_unconflicted(&client, &operation, &goal)?;
        let goal_entries = entries(&client, &operation, &goal)?;
        let rules = IgnoreRules::new(&client, &operation, &goal, &goal_entries)?;
        select_ignored(source, ignored_patterns, &rules, &goal_entries)?;
        verify_symlink_support(&client, &operation, &goal, &goal_entries)?;
        identity.verify(&client)?;
        verify_disk(&client, &selected.operation, &selected.source.commit_id, source, &old_entries)?;
        selected.verify_source_at(&client, &client.operation_id()?)?;
        let (operation, current) = current_commit(&client)?;
        if !exact_empty_child(&client, &operation, &current, &goal)? {
            let output = client.run([
                "--config=fsmonitor.backend=none",
                "--config=snapshot.auto-track=none()",
                "--config=snapshot.auto-update-stale=false",
                "new",
                &crate::jj::commit_id_revset(&goal)?,
            ])?;
            diagnostics(output.stderr());
        }
        let (operation, current) = current_commit(&client)?;
        if !exact_empty_child(&client, &operation, &current, &goal)? {
            bail!("source JJ new did not select expected empty child");
        }
        let change_id = client.resolve_one_at(&operation, &current)?.change_id;
        verify_command_result(&identity, &client, &goal, &goal_entries, &change_id)?;
        for (index, argv) in commands.iter().enumerate() {
            diagnostic(&format!(
                "Prepare: command {}/{} argv {:?}, cwd {:?}; sequential execution; commands must tolerate retry",
                index + 1,
                commands.len(),
                argv,
                source,
            ));
            let status = Command::new(&argv[0])
                .args(&argv[1..])
                .current_dir(source)
                .stdin(Stdio::inherit())
                .stdout(std::io::stderr())
                .stderr(Stdio::inherit())
                .status()
                .with_context(|| format!("could not run configured preparation argv {}", index + 1))?;
            if !status.success() {
                bail!(
                    "configured preparation command failed with {status} (step {}); ignored installation may be incomplete",
                    index + 1
                );
            }
            verify_command_result(&identity, &client, &goal, &goal_entries, &change_id)?;
        }
        let operation = client.operation_id()?;
        let rules = IgnoreRules::new(&client, &operation, &goal, &goal_entries)?;
        let (ignored, _) = select_ignored(source, ignored_patterns, &rules, &goal_entries)?;
        for path in ignored {
            cow::validate_tree(&source.join(path))?;
        }
        verify_command_result(&identity, &client, &goal, &goal_entries, &change_id)?;
        diagnostic(&format!(
            "Prepare: donor verified at exact goal {goal}; source-only completion; no targets selected"
        ));
        Ok(goal)
    })();
    result.context(
        "source preparation stopped; source progress was not rolled back; fix the source and run prepare again (all configured commands may run again)",
    )
}

fn verify_command_result(identity: &SourceIdentity, client: &JjClient, goal: &str, goal_entries: &[Entry], change_id: &str) -> Result<()> {
    identity.verify(client)?;
    verify_disk(client, &client.operation_id()?, goal, client.cwd(), goal_entries)
        .context("preparer changed tracked tree (including lockfiles); restore the selected source tree before running prepare again")?;
    let (operation, current) = current_commit(client)?;
    if client.resolve_one_at(&operation, &current)?.change_id != change_id || !exact_empty_child(client, &operation, &current, goal)? {
        bail!("source selected revision changed during preparation");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_command_settings_preserve_presence_and_order() {
        for text in [
            "command = []\ncommands = []",
            "command = ['cargo']\ncommands = []",
            "command = []",
            "commands = [[]]",
            "commands = [['']]",
        ] {
            assert!(toml::from_str::<Settings>(text).unwrap().argv().is_err(), "{text}");
        }
        assert_eq!(
            toml::from_str::<Settings>("commands = [['bun', 'install'], ['bun', 'run', 'build']]")
                .unwrap()
                .argv()
                .unwrap(),
            [vec!["bun", "install"], vec!["bun", "run", "build"]]
        );
        assert!(toml::from_str::<Settings>("commands = []").unwrap().argv().unwrap().is_empty());
        assert_eq!(
            toml::from_str::<Settings>("command = ['cargo', 'build']").unwrap().argv().unwrap(),
            [vec!["cargo", "build"]]
        );
    }
}
