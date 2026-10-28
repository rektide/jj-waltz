use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = "jj-waltz";
const CONFIG_FILE: &str = "config.toml";
const DEFAULT_BOOKMARK_TEMPLATE: &str = "{workspace}";
const DEFAULT_TRUNK_REVSET: &str = "trunk()";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub default_command: DefaultCommand,
    /// Root-level ignored-copy defaults; omission and explicit [] stay distinct.
    pub copy_ignored: Option<Vec<String>>,
    pub workspace: WorkspaceConfig,
    pub trunk: TrunkConfig,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DefaultCommand {
    #[default]
    Ui,
    List,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceConfig {
    pub create_bookmark: bool,
    /// None uses JJ-only creation; Some(true) explicitly requests colocation.
    pub colocate: Option<bool>,
    pub bookmark_template: String,
    /// None defaults to CoW unless colocation is explicitly selected.
    pub copy_on_write: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrunkConfig {
    pub revset: String,
}

#[derive(Debug, Deserialize, Default)]
struct RawConfig {
    default_command: Option<DefaultCommand>,
    copy_ignored: Option<Vec<String>>,
    workspace: Option<RawWorkspaceConfig>,
    trunk: Option<RawTrunkConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct RawWorkspaceConfig {
    #[serde(default)]
    create_bookmark: bool,
    colocate: Option<bool>,
    bookmark_template: Option<String>,
    copy_on_write: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
struct RawTrunkConfig {
    revset: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_command: DefaultCommand::Ui,
            copy_ignored: None,
            workspace: WorkspaceConfig {
                create_bookmark: false,
                colocate: None,
                bookmark_template: DEFAULT_BOOKMARK_TEMPLATE.to_owned(),
                copy_on_write: None,
            },
            trunk: TrunkConfig {
                revset: DEFAULT_TRUNK_REVSET.to_owned(),
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let Some(path) = config_path()? else {
            return Ok(Self::default());
        };
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
        let raw: RawConfig = toml::from_str(&contents).with_context(|| format!("failed to parse {}", path.display()))?;

        Ok(raw.into())
    }

    /// Global `copy_ignored` layer, read only when a caller would actually
    /// inherit it: a disabled policy, an explicit CLI list, or an explicit
    /// local list each bypass the global configuration file entirely, so a
    /// broken or unreadable global config cannot fail those routes. Returns
    /// the layer to hand to the transfer crate's supplied-value resolver;
    /// `None` means no global layer participates.
    pub fn load_inherited_copy_ignored(cli: Option<&[String]>, local: Option<&[String]>, disabled: bool) -> Result<Option<Vec<String>>> {
        Self::load_inherited_copy_ignored_with(cli, local, disabled, Config::load)
    }

    /// Testable core: the loader is injected so bypass behavior can be proven
    /// without touching the real configuration path.
    pub fn load_inherited_copy_ignored_with(
        cli: Option<&[String]>,
        local: Option<&[String]>,
        disabled: bool,
        loader: impl FnOnce() -> Result<Self>,
    ) -> Result<Option<Vec<String>>> {
        if disabled || cli.is_some() || local.is_some() {
            return Ok(None);
        }
        Ok(loader()?.copy_ignored)
    }
}

impl From<RawConfig> for Config {
    fn from(raw: RawConfig) -> Self {
        let defaults = Config::default();
        let workspace = raw.workspace.unwrap_or_default();
        let trunk = raw.trunk.unwrap_or_default();

        Self {
            default_command: raw.default_command.unwrap_or_default(),
            copy_ignored: raw.copy_ignored,
            workspace: WorkspaceConfig {
                create_bookmark: workspace.create_bookmark,
                colocate: workspace.colocate,
                bookmark_template: workspace.bookmark_template.unwrap_or(defaults.workspace.bookmark_template),
                copy_on_write: workspace.copy_on_write,
            },
            trunk: TrunkConfig {
                revset: trunk.revset.unwrap_or(defaults.trunk.revset),
            },
        }
    }
}

pub fn bookmark_from_template(template: &str, workspace: &str) -> String {
    template.replace("{workspace}", workspace)
}

fn config_path() -> Result<Option<PathBuf>> {
    let config_home = if let Some(value) = env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(value)
    } else if let Some(value) = env::var_os("HOME") {
        PathBuf::from(value).join(".config")
    } else {
        return Ok(None);
    };

    Ok(Some(config_home.join(CONFIG_DIR).join(CONFIG_FILE)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::sync::Mutex;

    /// Serializes tests that temporarily repoint XDG_CONFIG_HOME.
    static CONFIG_ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn defaults_do_not_create_bookmarks() {
        let config = Config::default();
        assert_eq!(config.default_command, DefaultCommand::Ui);
        assert_eq!(config.copy_ignored, None);
        assert!(!config.workspace.create_bookmark);
        assert_eq!(config.workspace.bookmark_template, "{workspace}");
        assert_eq!(config.workspace.copy_on_write, None);
        assert_eq!(config.trunk.revset, "trunk()");
    }

    #[test]
    fn root_ignored_copy_policy_distinguishes_missing_empty_and_selected() {
        for (text, expected) in [
            ("", None),
            ("copy_ignored = []", Some(Vec::new())),
            (
                "copy_ignored = ['**/target', '**/node_modules', '**/dist/', '**/build/']",
                Some(vec!["**/target".into(), "**/node_modules".into(), "**/dist/".into(), "**/build/".into()]),
            ),
        ] {
            assert_eq!(Config::from(toml::from_str::<RawConfig>(text).unwrap()).copy_ignored, expected);
        }
    }

    #[test]
    fn bare_command_is_explicitly_configurable() {
        for (value, expected) in [("ui", DefaultCommand::Ui), ("list", DefaultCommand::List), ("help", DefaultCommand::Help)] {
            let raw: RawConfig = toml::from_str(&format!("default_command = {value:?}")).unwrap();
            assert_eq!(Config::from(raw).default_command, expected);
        }
        assert!(toml::from_str::<RawConfig>("default_command = 'remove'").is_err());
    }

    #[test]
    fn preserves_copy_on_write_configuration_intent() {
        let raw: RawConfig = toml::from_str(
            r#"
                [workspace]
                copy_on_write = true
            "#,
        )
        .unwrap();

        let config = Config::from(raw);
        assert_eq!(config.workspace.copy_on_write, Some(true));
        assert!(!config.workspace.create_bookmark);
        let raw: RawConfig = toml::from_str("[workspace]\ncopy_on_write = false").unwrap();
        assert_eq!(Config::from(raw).workspace.copy_on_write, Some(false));
    }

    #[test]
    fn colocation_preserves_unset_and_explicit_false() {
        for (text, expected) in [
            ("", None),
            ("[workspace]", None),
            ("[workspace]\ncolocate = false", Some(false)),
            ("[workspace]\ncolocate = true", Some(true)),
        ] {
            let raw: RawConfig = toml::from_str(text).unwrap();
            assert_eq!(Config::from(raw).workspace.colocate, expected);
        }
    }

    #[test]
    fn parses_trunk_revset_without_workspace_section() {
        let raw: RawConfig = toml::from_str(
            r#"
                [trunk]
                revset = "main@origin"
            "#,
        )
        .unwrap();

        let config = Config::from(raw);
        assert_eq!(config.trunk.revset, "main@origin");
        assert!(!config.workspace.create_bookmark);
        assert_eq!(config.workspace.bookmark_template, "{workspace}");
    }

    #[test]
    fn empty_trunk_section_uses_default_revset() {
        let raw: RawConfig = toml::from_str("[trunk]").unwrap();

        assert_eq!(Config::from(raw).trunk.revset, "trunk()");
    }

    #[test]
    fn template_replaces_workspace_token() {
        assert_eq!(bookmark_from_template("wip/{workspace}", "feature-a"), "wip/feature-a");
    }

    /// An injected loader proves the bypass routes never read global config:
    /// reaching the loader on those routes would return the poisoned error.
    #[test]
    fn inherited_copy_ignored_bypasses_global_config_for_explicit_or_disabled_policy() {
        fn broken() -> Result<Config> {
            Err(anyhow!("broken global configuration must not be read"))
        }
        let cli = vec!["cli".to_owned()];
        let local = vec!["local".to_owned()];
        for (cli, local, disabled) in [
            (Some(cli.as_slice()), None, false),
            (Some(&[]), None, false),
            (None, Some(local.as_slice()), false),
            (None, Some(&[]), false),
            (None, None, true),
        ] {
            let inherited = Config::load_inherited_copy_ignored_with(cli, local, disabled, broken).expect("bypass routes never read the global layer");
            assert_eq!(inherited, None);
        }
        assert!(
            Config::load_inherited_copy_ignored_with(None, None, false, broken).is_err(),
            "inheritance must actually read the global layer"
        );
    }

    /// The real reader must show the same laziness against the filesystem:
    /// a broken global file only fails the route that inherits from it.
    #[test]
    fn broken_global_config_file_fails_only_the_inheriting_route() {
        let guard = CONFIG_ENV_LOCK.lock();
        let previous = env::var_os("XDG_CONFIG_HOME");
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("jj-waltz")).unwrap();
        fs::write(temp.path().join("jj-waltz/config.toml"), "not valid toml [[").unwrap();
        // Safety: the only environment mutation in this test, serialized by the
        // guard and restored before returning; config_path reads it directly.
        unsafe { env::set_var("XDG_CONFIG_HOME", temp.path()) };

        let cli = vec!["cli".to_owned()];
        let bypassed = Config::load_inherited_copy_ignored(Some(&cli), None, false).expect("explicit CLI list bypasses the broken global file");
        let inherited = Config::load_inherited_copy_ignored(None, None, false);

        unsafe {
            match previous {
                Some(value) => env::set_var("XDG_CONFIG_HOME", value),
                None => env::remove_var("XDG_CONFIG_HOME"),
            }
        }
        drop(guard);

        assert_eq!(bypassed, None);
        assert!(inherited.is_err(), "inheriting route must surface the broken global file");
    }
}
