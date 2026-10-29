use super::*;
use jj_waltz_native_jj::refill::config_guard;
use jj_waltz_workspace_state::ownership;

#[derive(Debug)]
pub struct RepositoryBinding {
    pub path: PathBuf,
    pub profile: NativeProfile,
    identity: Identity,
}

/// Only stable facts live here. Native commands change revision coordinates,
/// never refresh the entire config/root/registration discovery graph.
#[derive(Debug)]
pub struct BoundWorkspace {
    pub name: String,
    pub root: PathBuf,
    pub repository: Arc<RepositoryBinding>,
    pub device: u64,
    root_identity: Identity,
    jj_identity: Identity,
    repo_pointer: Vec<u8>,
    receiver: bool,
}
impl BoundWorkspace {
    pub fn client(&self) -> JjClient {
        JjClient::new(&self.root)
    }

    pub(super) fn capture(root: &Path, registry: &RegistrySnapshot, repository: Arc<RepositoryBinding>, receiver: bool) -> Result<Self> {
        ensure!(root == root.canonicalize()?, "workspace must name its canonical root");
        config_guard(root, &repository.path)?;
        let client = JjClient::new(root);
        let mut matching = Vec::new();
        for name in client.current_workspace_names_at(&registry.operation)? {
            let actual = client.run_at(&registry.operation, ["workspace", "root", "--name", &name])?;
            if PathBuf::from(actual.trimmed_stdout()?).canonicalize()? == root {
                matching.push(name);
            }
        }
        let [name] = matching.as_slice() else {
            bail!("workspace root registration missing or ambiguous")
        };
        ensure!(super::super::repository(&client)? == repository.path, "workspace repository differs");
        let root_identity = Identity::read(root)?;
        let jj_identity = Identity::read(&root.join(".jj"))?;
        let repo_pointer = pointer_bytes(root)?;
        validate_local(&client, &registry.operation)?;
        if receiver {
            refuse_git(root)?;
        }
        Ok(Self {
            name: name.clone(),
            root: root.to_path_buf(),
            device: root_identity.device,
            root_identity,
            jj_identity,
            repository,
            repo_pointer,
            receiver,
        })
    }

    pub fn verify_at(&self, operation: &str) -> Result<()> {
        self.root_identity.verify(&self.root)?;
        self.jj_identity.verify(&self.root.join(".jj"))?;
        self.repository.identity.verify(&self.repository.path)?;
        ensure!(
            pointer_bytes(&self.root)? == self.repo_pointer && super::super::repository(&self.client())? == self.repository.path,
            "native repository pointer changed"
        );
        let client = self.client();
        ensure!(client.current_workspace_names_at(operation)?.contains(&self.name), "workspace name changed");
        let actual = client.run_at(operation, ["workspace", "root", "--name", &self.name])?;
        ensure!(PathBuf::from(actual.trimmed_stdout()?).canonicalize()? == self.root, "registered root changed");
        if self.receiver {
            refuse_git(&self.root)?;
        }
        Ok(())
    }
}

fn pointer_bytes(root: &Path) -> Result<Vec<u8>> {
    let path = root.join(".jj/repo");
    let m = fs::symlink_metadata(&path)?;
    if m.is_dir() {
        Ok(Vec::new())
    } else {
        ensure!(m.is_file(), "unsupported repository pointer");
        Ok(fs::read(path)?)
    }
}
fn refuse_git(root: &Path) -> Result<()> {
    match fs::symlink_metadata(root.join(".git")) {
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
        Ok(_) => bail!("receiving workspace is colocated; requires JJ-only targets"),
    }
}

pub(super) fn validate_local(client: &JjClient, operation: &str) -> Result<()> {
    let backend = client.run_at_unchecked(operation, ["config", "get", "fsmonitor.backend"])?;
    ensure!(
        !backend.success() || matches!(backend.trimmed_stdout()?.as_str(), "none" | ""),
        "unsupported filesystem monitor; pause background snapshots"
    );
    let trigger = client.run_at_unchecked(operation, ["config", "get", "fsmonitor.watchman.register-snapshot-trigger"])?;
    ensure!(
        !trigger.success() || trigger.trimmed_stdout()? != "true",
        "unsupported background snapshot trigger"
    );
    require_full_at(client, operation)?;
    Identity::read(&client.cwd().join(".jj/working_copy"))?;
    ensure!(
        fs::read(client.cwd().join(".jj/working_copy/type"))? == b"local",
        "requires native local working copies"
    );
    for file in ["type", "checkout", "tree_state"] {
        ensure!(
            fs::symlink_metadata(client.cwd().join(".jj/working_copy").join(file))?.is_file(),
            "unsupported native state layout"
        );
    }
    Ok(())
}
pub(super) fn require_full_at(client: &JjClient, operation: &str) -> Result<()> {
    ensure!(
        client.run_at(operation, ["sparse", "list"])?.stdout()?.trim_end_matches(['\r', '\n']) == ".",
        "workspace is sparse or mid-refill; requires full sparse patterns ('.'); recover deliberately out of band"
    );
    Ok(())
}

pub(super) struct Resolution {
    pub caller: JjClient,
    pub registry: Arc<RegistrySnapshot>,
    pub source: Arc<BoundWorkspace>,
    pub config: LocalConfig,
    pub metadata: WorkspaceMetadataStore,
}
impl Resolution {
    pub fn capture(cwd: &Path, request: &ReseedRequest, source_only: bool) -> Result<Self> {
        ensure!(
            std::env::var_os("GIT_CONFIG_PARAMETERS").is_none_or(|v| v.is_empty())
                && std::env::var_os("GIT_CONFIG_COUNT").is_none_or(|v| v.is_empty() || v == "0"),
            "unset active Git config environment overrides before reseed"
        );
        let profile = NativeProfile::capture()?;
        let (caller, registry) = if source_only {
            RegistrySnapshot::capture_source(cwd)?
        } else {
            RegistrySnapshot::capture(cwd)?
        };
        let config = load_config(&registry.primary_root)?;
        let raw = request.source.as_ref().or(config.source.as_ref()).unwrap_or(&registry.primary_root);
        let source = if raw.is_absolute() {
            raw.clone()
        } else if request.source.is_some() {
            cwd.join(raw)
        } else {
            registry.primary_root.join(raw)
        }
        .canonicalize()?;
        let repo = Arc::new(RepositoryBinding {
            identity: Identity::read(&registry.repository)?,
            path: registry.repository.clone(),
            profile,
        });
        let source = Arc::new(BoundWorkspace::capture(&source, &registry, repo, false)?);
        let metadata = WorkspaceMetadataStore::from_repo_config_path(&registry.repo_config_path)?;
        Ok(Self {
            caller,
            registry: Arc::new(registry),
            source,
            config,
            metadata,
        })
    }

    pub fn check_ownership(&self, binding: &BoundWorkspace, registrations: &[PathBuf]) -> Result<()> {
        ownership::ensure_refill_allowed(&binding.root, self.metadata.get(&binding.name)?.as_ref(), registrations)
    }
}
