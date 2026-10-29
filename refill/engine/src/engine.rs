//! Shared identity, configuration and preservation primitives. Orchestration
//! lives exclusively in command; creation owns its separate lifecycle helpers.
use self::selection::Group;
use anyhow::{Context, Result, bail};
use fs2::FileExt;
use jj_waltz_native_jj::jj::JjClient;
use jj_waltz_transfer_workspace::tracked::directory_identity;
use jj_waltz_workspace_state::metadata::WorkspaceMetadataStore;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

mod bookmarks;
pub mod command;
mod prepare;
mod selection;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalConfig {
    source: Option<PathBuf>,
    source_at: Option<String>,
    copy_ignored: Option<Vec<String>>,
    #[serde(default)]
    groups: BTreeMap<String, Group>,
    freshen: Option<prepare::Settings>,
    prepare: Option<prepare::Settings>,
    #[serde(skip)]
    legacy_prepare: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Identity {
    device: u64,
    inode: u64,
}
impl Identity {
    fn read(path: &Path) -> Result<Self> {
        let (device, inode) = directory_identity(path)?;
        Ok(Self { device, inode })
    }
    fn verify(&self, path: &Path) -> Result<()> {
        if &Self::read(path)? != self {
            bail!("directory identity changed: {}", path.display());
        }
        Ok(())
    }
}

struct PreparationLock {
    _lock: File,
}
impl PreparationLock {
    fn lock(client: &JjClient) -> Result<Self> {
        let config = client.repo_config_path()?;
        WorkspaceMetadataStore::from_repo_config_path(&config)?.ensure_initialized()?;
        let parent = config.parent().context("repository config has no parent")?.canonicalize()?;
        let root = parent.join("jj-waltz/reseed");
        fs::create_dir_all(&root)?;
        if root.canonicalize()? != root {
            bail!("reseed storage contains a symlink");
        }
        let lock_path = root.join("operation.lock");
        if fs::symlink_metadata(&lock_path).is_ok_and(|m| !m.is_file()) {
            bail!("invalid reseed lock");
        }
        let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false).open(lock_path)?;
        lock.try_lock_exclusive()
            .context("another jw preparation is running; pause other writers before retrying")?;
        Ok(Self { _lock: lock })
    }
}

fn repository(client: &JjClient) -> Result<PathBuf> {
    jj_waltz_native_jj::context::repository_path(client.cwd())
        .map_err(anyhow::Error::msg)?
        .canonicalize()
        .context("cannot resolve JJ repository identity")
}

fn load_config(default: &Path) -> Result<LocalConfig> {
    let path = default.join(".jwreseed.local.toml");
    let result = (|| match fs::read(&path) {
        Ok(bytes) => {
            let mut config: LocalConfig = toml::from_str(std::str::from_utf8(&bytes)?).with_context(|| format!("invalid {}", path.display()))?;
            selection::validate_groups(&config.groups)?;
            if config.freshen.is_some() && config.prepare.is_some() {
                bail!("[freshen] and legacy [prepare] cannot both be present; keep only [freshen]");
            }
            if let Some(legacy) = config.prepare.take() {
                config.freshen = Some(legacy);
                config.legacy_prepare = true;
            }
            if let Some(settings) = &config.freshen {
                settings.argv()?;
            }
            Ok(config)
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(LocalConfig::default()),
        Err(e) => Err(e).with_context(|| format!("cannot read {}", path.display())),
    })();
    result.with_context(|| command::ErrorPath(path))
}

fn snapshot(client: &JjClient) -> Result<()> {
    let output = client.run(["--config=fsmonitor.backend=none", "--config=snapshot.auto-update-stale=false", "status"])?;
    if !output.stderr().is_empty() {
        eprintln!("{}", output.stderr());
    }
    Ok(())
}
