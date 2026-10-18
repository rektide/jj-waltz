use std::{fs, path::Path, process::Command};

#[test]
fn rust_exec_wrapper_logs_literal_argv_then_executes_native_binary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-perf-unit");
    fs::create_dir_all(&root).unwrap();
    let own = tempfile::tempdir_in(root).unwrap();
    let wrapper = own.path().join("jj");
    let trace = own.path().join("calls.jsonl");
    fs::write(&trace, "").unwrap();
    std::os::unix::fs::symlink(env!("CARGO_BIN_EXE_jw-perf"), &wrapper).unwrap();
    let status = Command::new(wrapper)
        .args(["--config", "space=value with spaces", "file", "show", "a 'quoted' path"])
        .env("JW_PERF_TRACE", &trace)
        .env("JW_PERF_REAL_JJ", "/usr/bin/true")
        .status()
        .unwrap();
    assert!(status.success());
    let text = fs::read_to_string(trace).unwrap();
    let rows: Vec<_> = text.lines().collect();
    assert_eq!(rows.len(), 1);
    let row: serde_json::Value = serde_json::from_str(rows[0]).unwrap();
    assert_eq!(row["tool"], "jj");
    assert_eq!(row["argv"][1], "space=value with spaces");
    assert_eq!(row["argv"][4], "a 'quoted' path");
}
