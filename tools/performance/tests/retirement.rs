use std::{fs, path::Path, process::Command};

fn active(root: &Path) {
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() {
            if !matches!(entry.file_name().to_str(), Some("target" | "__pycache__")) {
                active(&path);
            }
        } else {
            assert_ne!(
                path.extension().and_then(|s| s.to_str()),
                Some("py"),
                "active Python survived: {}",
                path.display()
            );
        }
    }
}

#[test]
fn supported_guide_and_all_native_entrypoints_have_no_python_launcher() {
    let tool = Path::new(env!("CARGO_MANIFEST_DIR"));
    active(tool);
    for path in [tool.join("README.md"), tool.join("../../CONTRIBUTING.md")] {
        let text = fs::read_to_string(path).unwrap();
        assert!(!text.contains("python3 "));
    }
    for entry in fs::read_dir(tool.join("src")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let text = fs::read_to_string(path).unwrap();
            assert!(!text.contains("python3"));
            assert!(!text.contains("from experiments"));
        }
    }
    let output = Command::new(env!("CARGO_BIN_EXE_jw-perf")).arg("--help").output().unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for name in ["snapshot", "build", "run", "check", "compare", "copy-baseline"] {
        assert!(help.contains(name));
    }
}
