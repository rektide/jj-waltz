use super::*;
use clap::Parser;

#[test]
fn cli_requires_explicit_case_acceptance_and_fixed_bounded_dimensions() {
    let parsed = crate::Cli::try_parse_from([
        "jw-perf",
        "renewal",
        "sample",
        "owned",
        "output",
        "--acceptance",
        "receipt",
        "--case",
        "mixed-n2",
    ])
    .unwrap();
    assert!(matches!(
        parsed.action,
        crate::Action::Renewal {
            action: Action::Sample {
                case: Case::MixedN2,
                timing: Timing::Native,
                ..
            }
        }
    ));
    assert!(crate::Cli::try_parse_from(["jw-perf", "renewal", "sample", "owned", "output", "--case", "mixed-n2"]).is_err());
    assert!(
        crate::Cli::try_parse_from([
            "jw-perf",
            "renewal",
            "sample",
            "owned",
            "output",
            "--acceptance",
            "receipt",
            "--case",
            "unbounded"
        ])
        .is_err()
    );
    assert!(
        crate::Cli::try_parse_from([
            "jw-perf",
            "renewal",
            "sample",
            "owned",
            "output",
            "--acceptance",
            "receipt",
            "--case",
            "mixed-n2",
            "--destinations",
            "3"
        ])
        .is_err()
    );
}

pub(super) fn control(root: &Path, binary: &Path) -> Control {
    let value = json!({"source_commit":"a".repeat(40),"library_commit":"b".repeat(40),"rustc":"fake test compiler (not build evidence)",
        "build_command":"fake fixture test only","profile":"fake test profile","sha256":artifacts::sha256(binary).unwrap(),
        "source_inputs":{},"library_inputs":{},"build_environment":{},"features":[]});
    let path = root.join("fake-provenance.json");
    fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    Control {
        label: "candidate".into(),
        binary: binary.into(),
        sha256: artifacts::sha256(binary).unwrap(),
        mode: "none".into(),
        provenance_path: path.clone(),
        provenance: value,
        provenance_sha256: artifacts::sha256(&path).unwrap(),
    }
}

pub(super) fn scratch() -> tempfile::TempDir {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/annealing-evidence");
    fs::create_dir_all(&root).unwrap();
    tempfile::tempdir_in(root.canonicalize().unwrap()).unwrap()
}

#[test]
fn literal_argv_keeps_route_source_and_independent_desired_pins() {
    let own = scratch();
    let f = Fixture::fake(&own.path().join("root with 'literal quotes'"), control(own.path(), Path::new("/usr/bin/true")));
    for case in Case::ALL {
        for route in Route::ALL {
            let args = argv(&f, case, route).unwrap();
            assert_eq!(args.iter().filter(|s| s.starts_with("--renewal-route=")).count(), 1);
            assert!(args.contains(&format!("--renewal-route={}", route.label())));
            assert_eq!(args[1..=case.receivers()], ["worker-a", "worker-b"][..case.receivers()]);
            let value = |flag: &str| &args[args.iter().position(|s| s == flag).unwrap() + 1];
            assert_eq!(value("--from"), &f.source().display().to_string());
            assert_eq!(value("--source-at"), &fixture::exact(&f.s).unwrap());
            assert_eq!(value("--at"), &fixture::exact(f.desired(case)).unwrap());
            assert_eq!(value("--source-at") != value("--at"), case == Case::DesiredN2);
            assert!(!args.iter().any(|s| s == "--replace-content" || s == "edit"));
        }
    }
}

#[test]
fn legacy_help_and_missing_report_are_never_route_acceptance() {
    assert!(capability("reseed --replace-content --dry-run").is_err());
    assert!(capability("--renewal-route clear-then-populate --format json").is_err());
    assert!(capability("--renewal-route clear-then-populate replace-authorized --format json").is_ok());
    let own = scratch();
    let f = Fixture::fake(own.path(), control(own.path(), Path::new("/usr/bin/true")));
    for raw in [json!({}), json!({"native_ms":0,"status":"success"}), json!({"fake":true})] {
        let error = report_evidence(&raw, &f, Case::CleanN1, Route::ReplaceAuthorized).unwrap_err().to_string();
        assert!(error.contains("schema_version"));
    }
}

#[test]
fn mirrored_route_order_is_four_commands_on_one_binary_not_build_variants() {
    let order = diff_dirs_perf_runner::scheduling::schedule(2, 2, 1701)
        .unwrap()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    assert_eq!(order.len(), 4);
    assert_eq!(order[0], order[3]);
    assert_eq!(order[1], order[2]);
    assert_ne!(order[0], order[1]);
    assert_eq!(Route::ALL[0].label(), "clear-then-populate");
    assert_eq!(Route::ALL[1].label(), "replace-authorized");
}

#[test]
fn candidate_provenance_and_semantic_acceptance_bind_the_exact_artifact() {
    let own = scratch();
    let binary = own.path().join("candidate");
    fs::copy("/usr/bin/true", &binary).unwrap();
    let c = control(own.path(), &binary);
    check_control(&c).unwrap();
    fs::write(&c.provenance_path, "{}").unwrap();
    assert!(check_control(&c).unwrap_err().to_string().contains("sidecar changed"));
    let c = control(own.path(), &binary);
    fs::write(&binary, "changed").unwrap();
    assert!(check_control(&c).unwrap_err().to_string().contains("SHA-256 mismatch"));
    assert!(build_provenance(&json!({"sha256":"known"})).is_err());

    let f = Fixture::fake(own.path(), c);
    fs::write(own.path().join("fixture.json"), "fake fixture test only").unwrap();
    let acceptance = own.path().join("acceptance.json");
    write_json(&acceptance, &json!({"passed":true,"binding":{},"cases":[]})).unwrap();
    assert!(
        check_acceptance(&f, &acceptance).is_err(),
        "stale or unrelated acceptance must not start performance"
    );

    let raw = own.path().join("fake-public-stdout");
    fs::write(&raw, "{\"fake\":true}").unwrap();
    let rows = Case::ALL
        .into_iter()
        .flat_map(|case| Route::ALL.map(|route| json!({"case":case,"route":route,"command":{"stdout":raw},"report":{}})))
        .collect::<Vec<_>>();
    fs::write(
        &acceptance,
        serde_json::to_vec(&json!({"passed":true,"binding":binding(&f).unwrap(),"cases":rows})).unwrap(),
    )
    .unwrap();
    assert!(
        check_acceptance(&f, &acceptance).unwrap_err().to_string().contains("schema_version"),
        "a flag in a receipt cannot bypass real public report extraction"
    );
}

#[test]
fn fake_command_preserves_literal_argv_raw_non_json_failure_and_exit_status() {
    let own = scratch();
    let root = own.path().join("owned 'root' with spaces");
    fs::create_dir_all(root.join("trees/source")).unwrap();
    let logs = own.path().join("logs");
    fs::create_dir(&logs).unwrap();
    let binary = own.path().join("fake-command");
    // Test-only fake, never imported as a real ELF candidate or native route.
    fs::write(
        &binary,
        "#!/bin/sh\nprintf 'raw not-json failure\\n'\nprintf 'original fake cause\\n' >&2\nexit 73\n",
    )
    .unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    let c = control(own.path(), &binary);
    let f = Fixture::fake(&root, c.clone());
    let mut ctx = Context::new(&logs, Path::new("/usr/bin/true"), Path::new("/usr/bin/true"), Path::new("/usr/bin/true")).unwrap();
    let stem = logs.join("attempt");
    let error = invoke(&mut ctx, &c, &f, Case::DesiredN2, Route::ReplaceAuthorized, Timing::Native, &stem)
        .unwrap_err()
        .to_string();
    assert!(error.contains("73"));
    assert!(!error.contains("JSON"), "parse errors must not mask nonzero command failure");
    let row: Value = serde_json::from_slice(&fs::read(stem.with_extension("command.json")).unwrap()).unwrap();
    let args = argv(&f, Case::DesiredN2, Route::ReplaceAuthorized).unwrap();
    assert_eq!(
        row["process"]["argv"],
        json!(std::iter::once(binary.display().to_string()).chain(args).collect::<Vec<_>>())
    );
    assert_eq!(row["process"]["cwd"], root.join("trees/source").display().to_string());
    assert_eq!(row["process"]["code"], 73);
    let stdout = row["stdout"].as_str().unwrap();
    assert_eq!(fs::read_to_string(stdout).unwrap(), "raw not-json failure\n");
    assert_eq!(fs::read_to_string(Path::new(stdout).with_extension("stderr")).unwrap(), "original fake cause\n");
    assert!(!row["reported_fields"].is_object(), "no invented phase schema or zero fields");

    fs::write(&binary, "#!/bin/sh\nprintf '{\"future_wire\":\"raw\"}\\n'\n").unwrap();
    let c = control(own.path(), &binary);
    let row = invoke(&mut ctx, &c, &f, Case::CleanN1, Route::ClearThenPopulate, Timing::Native, &logs.join("success")).unwrap();
    let raw: Value = serde_json::from_slice(&fs::read(row["stdout"].as_str().unwrap()).unwrap()).unwrap();
    assert_eq!(raw, json!({"future_wire":"raw"}));
    assert!(
        report_evidence(&raw, &f, Case::CleanN1, Route::ClearThenPopulate).is_err(),
        "successful process exit is not semantic acceptance"
    );
}

#[test]
fn first_failure_halts_later_scheduled_resets_and_route_attempts() {
    let own = scratch();
    let c = control(own.path(), Path::new("/usr/bin/true"));
    let fixture = Fixture::fake(own.path(), c.clone());
    let mut workload = Renewal {
        fixture,
        output: own.path().into(),
        case: Case::CleanN1,
        timing: Timing::Native,
        ctx: None,
        matched: None,
        halted: false,
    };
    let mut sample = empty_sample(0, "clear-then-populate");
    assert!(workload.sample(&c, &own.path().join("first"), &mut sample).is_err());
    assert!(workload.halted);
    let error = workload.sample(&c, &own.path().join("second"), &mut sample).unwrap_err().to_string();
    assert!(error.contains("no reset, continuation or route fallback"));
    assert!(!own.path().join("trees").exists());
}
