use crate::instrumentation;
use diff_dirs_perf_runner::{Result, process};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

pub struct Context {
    pub root: PathBuf,
    pub jj: PathBuf,
    pub git: PathBuf,
    pub own: PathBuf,
    pub env: BTreeMap<String, String>,
    sequence: usize,
}

impl Context {
    pub fn new(root: &Path, jj: &Path, git: &Path, own: &Path) -> Result<Self> {
        fs::create_dir(root.join("home"))?;
        fs::create_dir(root.join("commands"))?;
        let native_bin = root.join("native-bin");
        let count_bin = root.join("count-bin");
        fs::create_dir(&native_bin)?;
        fs::create_dir(&count_bin)?;
        for (name, path) in [("jj", jj), ("git", git)] {
            std::os::unix::fs::symlink(path, native_bin.join(name))?;
            std::os::unix::fs::symlink(own, count_bin.join(name))?;
        }
        let home = root.join("home");
        let mut env = BTreeMap::new();
        for (key, value) in [
            ("HOME", home.display().to_string()),
            ("XDG_CONFIG_HOME", home.join("config").display().to_string()),
            ("XDG_CACHE_HOME", home.join("cache").display().to_string()),
            ("XDG_STATE_HOME", home.join("state").display().to_string()),
            ("XDG_DATA_HOME", home.join("data").display().to_string()),
            ("PATH", format!("{}:/usr/bin:/bin", native_bin.display())),
            ("JJ_CONFIG", String::new()),
            ("JJ_USER", "Performance Fixture".into()),
            ("JJ_EMAIL", "performance@example.invalid".into()),
            ("JJ_EDITOR", "true".into()),
            ("GIT_CONFIG_NOSYSTEM", "1".into()),
            ("GIT_CONFIG_GLOBAL", "/dev/null".into()),
            ("GIT_AUTHOR_NAME", "Performance Fixture".into()),
            ("GIT_AUTHOR_EMAIL", "performance@example.invalid".into()),
            ("GIT_COMMITTER_NAME", "Performance Fixture".into()),
            ("GIT_COMMITTER_EMAIL", "performance@example.invalid".into()),
            ("GIT_TERMINAL_PROMPT", "0".into()),
            ("LC_ALL", "C".into()),
            ("TERM", "dumb".into()),
            ("NO_COLOR", "1".into()),
            ("JW_PERF_REAL_JJ", jj.display().to_string()),
            ("JW_PERF_REAL_GIT", git.display().to_string()),
        ] {
            env.insert(key.into(), value);
        }
        Ok(Self {
            root: root.into(),
            jj: jj.into(),
            git: git.into(),
            own: own.into(),
            env,
            sequence: 0,
        })
    }
    pub fn run(&mut self, binary: &Path, args: &[String], cwd: &Path, trace: Option<&Path>) -> Result<(String, process::Outcome)> {
        let stem = self.root.join("commands").join(format!("{:04}", self.sequence));
        self.sequence += 1;
        let mut command = Command::new(binary);
        command.args(args).current_dir(cwd).env_clear().envs(&self.env);
        if let Some(trace) = trace {
            command
                .env("JW_PERF_TRACE", trace)
                .env("PATH", format!("{}:/usr/bin:/bin", self.root.join("count-bin").display()));
        }
        let result = process::run(&mut command, &stem, Duration::from_secs(180))?;
        if result.code != Some(0) || result.timed_out {
            return Err(format!("native command failed; see {}.stderr", stem.display()).into());
        }
        Ok((fs::read_to_string(stem.with_extension("stdout"))?.trim().into(), result))
    }
    pub fn jj(&mut self, cwd: &Path, args: &[&str]) -> Result<String> {
        let binary = self.jj.clone();
        Ok(self.run(&binary, &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(), cwd, None)?.0)
    }
    pub fn root_check(&mut self, root: &Path) -> Result<()> {
        if !root.join(".jj").is_dir() || self.jj(root, &["--ignore-working-copy", "root"])? != fs::canonicalize(root)?.display().to_string() {
            return Err("JJ root escaped the explicit owned repository/workspace".into());
        }
        Ok(())
    }
    pub fn identities(&self) -> Result<serde_json::Value> {
        Ok(
            serde_json::json!({"jj":instrumentation::identity(&self.jj)?,"git":instrumentation::identity(&self.git)?,"adapter":instrumentation::identity(&self.own)?}),
        )
    }
}
