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
    let text = fs::read_to_string(&trace).unwrap();
    let rows: Vec<_> = text.lines().collect();
    assert_eq!(rows.len(), 1);
    let row: serde_json::Value = serde_json::from_str(rows[0]).unwrap();
    assert_eq!(row["tool"], "jj");
    assert_eq!(row["argv"][1], "space=value with spaces");
    assert_eq!(row["argv"][4], "a 'quoted' path");

    let output = Command::new(env!("CARGO_BIN_EXE_jw-perf")).arg("count").arg(&trace).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let counts: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(counts["jj_calls"], 1);
    assert_eq!(counts["file_show_calls"], 1);
    assert_eq!(counts["git_calls"], 0);
    assert_eq!(counts["jj_by_command"]["file show"], 1);
}

#[test]
fn standalone_count_distinguishes_native_clear_from_sparse_inspection() {
    let own = tempfile::tempdir().unwrap();
    let trace = own.path().join("calls.jsonl");
    let rows = [
        serde_json::json!({"tool":"jj","argv":["sparse","list"],"cwd":"/fixture","pid":1,"unix_ns":1}),
        serde_json::json!({"tool":"jj","argv":["sparse","set","--clear"],"cwd":"/fixture","pid":2,"unix_ns":2}),
        serde_json::json!({"tool":"jj","argv":["new","commit_id(deadbeef)"],"cwd":"/fixture","pid":3,"unix_ns":3}),
        serde_json::json!({"tool":"git","argv":["-C","/fixture","check-ignore","-z"],"cwd":"/fixture","pid":4,"unix_ns":4}),
    ];
    fs::write(&trace, rows.iter().map(|row| format!("{row}\n")).collect::<String>()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jw-perf")).arg("count").arg(&trace).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let counts: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(counts["jj_calls"], 3);
    assert_eq!(counts["file_show_calls"], 0);
    assert_eq!(counts["git_calls"], 1);
    assert_eq!(counts["jj_by_command"]["sparse list"], 1);
    assert_eq!(counts["jj_by_command"]["sparse set"], 1);
    assert_eq!(counts["jj_by_command"]["new"], 1);
    assert_eq!(counts["git_by_command"]["check-ignore"], 1);
}
