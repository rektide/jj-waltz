use diff_dirs_perf_runner::{Result, artifacts::sha256};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{Read, Write},
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Call {
    pub tool: String,
    pub argv: Vec<String>,
    pub cwd: PathBuf,
    pub pid: u32,
    pub unix_ns: u128,
}

/// Rust symlink/exec wrapper; no shell or Python startup per observed invocation.
pub fn wrapper() -> Result<bool> {
    let name = std::env::args_os()
        .next()
        .and_then(|p| Path::new(&p).file_name().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_default();
    if name != "jj" && name != "git" {
        return Ok(false);
    }
    let native = std::env::var_os(if name == "jj" { "JW_PERF_REAL_JJ" } else { "JW_PERF_REAL_GIT" }).ok_or("wrapper missing native executable identity")?;
    let argv: Vec<_> = std::env::args().skip(1).collect();
    if let Some(trace) = std::env::var_os("JW_PERF_TRACE") {
        let call = Call {
            tool: name,
            argv: argv.clone(),
            cwd: std::env::current_dir()?,
            pid: std::process::id(),
            unix_ns: SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        };
        let mut row = serde_json::to_vec(&call)?;
        row.push(b'\n');
        let mut file = OpenOptions::new().append(true).open(trace)?;
        file.write_all(&row)?;
    }
    Err(Command::new(native).args(argv).exec().into())
}

pub fn native(path: &Path) -> Result<PathBuf> {
    let path = fs::canonicalize(path)?;
    let mut magic = [0; 4];
    fs::File::open(&path)?.read_exact(&mut magic)?;
    if magic != *b"\x7fELF" {
        return Err(format!("use a native ELF executable, not a tool-manager script: {}", path.display()).into());
    }
    Ok(path)
}

pub fn identity(path: &Path) -> Result<serde_json::Value> {
    Ok(serde_json::json!({"path":path,"sha256":sha256(path)?}))
}

pub fn classify(tool: &str, args: &[String]) -> String {
    let valued = if tool == "jj" {
        vec!["--repository", "-R", "--at-operation", "--color", "--config", "--config-file", "--config-toml"]
    } else {
        vec!["-C", "-c", "--git-dir", "--work-tree", "--namespace"]
    };
    let mut i = 0;
    while i < args.len() && args[i].starts_with('-') {
        i += if valued.contains(&args[i].as_str()) { 2 } else { 1 };
    }
    let Some(first) = args.get(i) else {
        return "(global)".into();
    };
    if tool == "jj" && first == "diff" && args[i + 1..].iter().any(|a| a == "--git") {
        return "diff --git".into();
    }
    if tool == "jj"
        && ["file", "workspace", "git", "bookmark", "config", "operation", "op", "debug"].contains(&first.as_str())
        && let Some(next) = args.get(i + 1).filter(|s| !s.starts_with('-'))
    {
        return format!("{first} {next}");
    }
    first.clone()
}

pub fn counts(path: &Path) -> Result<serde_json::Value> {
    let mut jj = BTreeMap::<String, u64>::new();
    let mut git = BTreeMap::<String, u64>::new();
    for line in fs::read_to_string(path)?.lines() {
        let call: Call = serde_json::from_str(line)?;
        let map = match call.tool.as_str() {
            "jj" => &mut jj,
            "git" => &mut git,
            _ => return Err("unknown traced tool".into()),
        };
        *map.entry(classify(&call.tool, &call.argv)).or_default() += 1;
    }
    Ok(
        serde_json::json!({"jj_calls":jj.values().sum::<u64>(),"file_show_calls":jj.get("file show").copied().unwrap_or(0),
        "git_calls":git.values().sum::<u64>(),"jj_by_command":jj,"git_by_command":git}),
    )
}

#[test]
fn classify_respects_global_values_and_argv_boundaries() {
    let args = |xs: &[&str]| xs.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(
        classify("jj", &args(&["--config", "x=space value", "file", "show", "path with space"])),
        "file show"
    );
    assert_eq!(classify("jj", &args(&["--ignore-working-copy", "diff", "--git"])), "diff --git");
    assert_eq!(classify("git", &args(&["-C", "/space path", "check-ignore", "-z"])), "check-ignore");
}
