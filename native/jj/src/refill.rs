//! The private local-state protocol has a narrower support boundary than the
//! lifecycle CLI. Keep the raw executable identity and read-only startup guards.
use crate::{
    context::repository_path,
    jj::{JjClient, JjVersion, WorkspaceBinding},
};
use anyhow::{Context, Result, bail, ensure};
use std::{
    collections::BTreeMap,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub const TESTED_BUILD: &str = "0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeProfile {
    pub id: &'static str,
    pub raw_version: String,
    pub version: JjVersion,
}

impl NativeProfile {
    pub fn classify(raw_version: String) -> Result<Self> {
        let version = raw_version.parse()?;
        ensure!(
            raw_version.trim().strip_prefix("jj ") == Some(TESTED_BUILD),
            "source-state refill supports tested JJ build {TESTED_BUILD} only; found {raw_version}"
        );
        Ok(Self {
            id: "local-tree-state-33d3a595",
            raw_version,
            version,
        })
    }

    pub fn capture(client: &JjClient) -> Result<Self> {
        Self::classify(client.run(["--version"])?.trimmed_stdout()?)
    }
}

/// Stable registration/root view, without tree diffs, metadata initialization or
/// repository reconciliation. Every repository query uses the captured operation.
#[derive(Debug)]
pub struct RegistrySnapshot {
    pub operation: String,
    pub repository: PathBuf,
    pub bindings: BTreeMap<String, WorkspaceBinding>,
    pub roots: BTreeMap<String, PathBuf>,
    pub registered: BTreeMap<String, Option<PathBuf>>,
    pub primary_name: String,
    pub primary_root: PathBuf,
    pub current_name: String,
    pub repo_config_path: PathBuf,
}

impl RegistrySnapshot {
    pub fn capture(cwd: &Path) -> Result<(JjClient, Self)> {
        Self::capture_view(cwd, true)
    }

    /// Source-only preparation needs the primary and caller, not receiver roots.
    pub fn capture_source(cwd: &Path) -> Result<(JjClient, Self)> {
        Self::capture_view(cwd, false)
    }

    fn capture_view(cwd: &Path, all_roots: bool) -> Result<(JjClient, Self)> {
        let cwd = cwd.canonicalize()?;
        let root = cwd.ancestors().find(|p| p.join(".jj").is_dir()).context("no JJ workspace root")?.to_path_buf();
        let repository = repository_path(&root).map_err(anyhow::Error::msg)?.canonicalize()?;
        // Even read-only JJ commands load secure config. Refuse conditions which
        // would migrate, repair or copy it before launching the first process.
        let repo_config_path = config_guard(&root, &repository)?;
        let client = JjClient::new(&root);
        let operation = client.operation_id()?;
        let bindings = client.workspace_bindings_at(&operation)?;
        let primary = repository
            .parent()
            .filter(|p| p.file_name().is_some_and(|n| n == ".jj"))
            .and_then(Path::parent)
            .context("unsupported detached repository owner layout")?
            .to_path_buf();
        config_guard(&primary, &repository)?;
        let primary_names = JjClient::new(&primary).current_workspace_names_at(&operation)?;
        let caller_names = client.current_workspace_names_at(&operation)?;
        let mut roots = BTreeMap::new();
        let mut registered = BTreeMap::new();
        for name in bindings.keys() {
            if !all_roots && !primary_names.contains(name) && !caller_names.contains(name) {
                continue;
            }
            let output = client.run_at_unchecked(&operation, ["workspace", "root", "--name", name])?;
            let path = if output.success() {
                Some(PathBuf::from(output.trimmed_stdout()?))
            } else {
                None
            };
            if let Some(path) = &path
                && let Ok(path) = path.canonicalize()
            {
                roots.insert(name.clone(), path);
            }
            registered.insert(name.clone(), path);
        }
        let owners = roots
            .iter()
            .filter(|(_, root)| {
                let pointer = root.join(".jj/repo");
                fs::symlink_metadata(&pointer).is_ok_and(|m| m.is_dir()) && pointer.canonicalize().ok().as_ref() == Some(&repository)
            })
            .collect::<Vec<_>>();
        let [(primary_name, primary_root)] = owners.as_slice() else {
            bail!("repository owner registration missing or ambiguous")
        };
        let names = roots.iter().filter(|(_, p)| *p == &root).map(|(n, _)| n.clone()).collect::<Vec<_>>();
        let [current_name] = names.as_slice() else {
            bail!("caller root registration missing or ambiguous")
        };
        ensure!(
            client.current_workspace_names_at(&operation)?.contains(current_name),
            "caller workspace identity disagrees with registration"
        );
        Ok((
            client,
            Self {
                operation,
                repository,
                bindings,
                primary_name: (*primary_name).clone(),
                primary_root: (*primary_root).clone(),
                current_name: current_name.clone(),
                roots,
                registered,
                repo_config_path,
            },
        ))
    }
}

/// Supported-build secure config inspection. Does not call `config path`: that
/// command creates IDs/directories. Startup itself can migrate or repair config,
/// so a stale/moved/copied/missing secure binding is a static refusal, not a repair.
pub fn config_guard(root: &Path, repository: &Path) -> Result<PathBuf> {
    let config_root = user_config_root().context("cannot locate JJ user configuration for read-only discovery")?;
    let repo_config = secure_config(repository, "config-id", "config.toml", &config_root.join("repos"))?;
    secure_config(
        &root.join(".jj"),
        "workspace-config-id",
        "workspace-config.toml",
        &config_root.join("workspaces"),
    )?;
    Ok(repo_config.unwrap_or_else(|| repository.join("config.toml")))
}

fn user_config_root() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support/jj"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .map(|p| p.join("jj"))
    }
}

fn secure_config(owner: &Path, id_name: &str, legacy: &str, configs: &Path) -> Result<Option<PathBuf>> {
    let id = match fs::read_to_string(owner.join(id_name)) {
        Ok(id) => id,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            ensure!(
                matches!(fs::symlink_metadata(owner.join(legacy)), Err(e) if e.kind() == ErrorKind::NotFound),
                "JJ legacy config would require migration; run a deliberate native command outside preview first"
            );
            return Ok(None);
        }
        Err(e) => return Err(e.into()),
    };
    ensure!(id.len() == 20 && id.bytes().all(|b| b.is_ascii_hexdigit()), "invalid JJ config ID");
    let directory = configs.join(id);
    let metadata = fs::read(directory.join("metadata.binpb")).context("JJ secure config metadata missing; read-only discovery refuses native repair")?;
    // This profile's ConfigMetadata is exactly optional bytes path = 1. Require
    // its canonical encoding rather than interpreting a broader future schema.
    let path = owner.as_os_str().as_encoded_bytes();
    let mut expected = vec![0x0a];
    let mut length = path.len();
    while length >= 128 {
        expected.push((length as u8 & 0x7f) | 0x80);
        length >>= 7;
    }
    expected.push(length as u8);
    expected.extend_from_slice(path);
    ensure!(
        metadata == expected,
        "JJ secure config owner differs; read-only discovery refuses native migration/copy"
    );
    Ok(Some(directory.join("config.toml")))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build_identity_does_not_collapse_to_semver() {
        let good = NativeProfile::classify(format!("jj {TESTED_BUILD}")).unwrap();
        assert_eq!(good.version, JjVersion::new(0, 45, 1));
        for raw in ["jj 0.45.1", "jj 0.45.1-0000000000000000000000000000000000000000", "jj 0.46.0"] {
            assert!(NativeProfile::classify(raw.into()).is_err());
        }
    }
    #[test]
    fn config_guard_never_initializes_or_repairs() {
        let temp = tempfile::tempdir().unwrap();
        let owner = temp.path().join("owner");
        let configs = temp.path().join("configs");
        fs::create_dir(&owner).unwrap();
        assert!(secure_config(&owner, "config-id", "config.toml", &configs).unwrap().is_none());
        fs::write(owner.join("config.toml"), "legacy").unwrap();
        assert!(secure_config(&owner, "config-id", "config.toml", &configs).is_err());
        assert!(!configs.exists());
        assert!(!owner.join("config-id").exists());
        fs::write(owner.join("config-id"), "a".repeat(20)).unwrap();
        assert!(secure_config(&owner, "config-id", "config.toml", &configs).is_err());
        assert!(!configs.exists());
    }
}
