use super::*;
use crate::renewal::tests::{control, scratch};

fn path(path: &Path) -> Value {
    json!({"display":path.display().to_string(),"unix_bytes_hex":path.as_os_str().as_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>()})
}
fn complete(f: &Fixture, case: Case, route: Route) -> Value {
    // Start from the application's actual schema-one fixture, not a guessed
    // legacy wire shape. Replace its illustrative coordinates with our fixture.
    let mut raw: Value = serde_json::from_str(include_str!("../../../../../app/cli/tests/fixtures/refill/reseed-complete.json")).unwrap();
    raw["renewal_route"] = json!(route);
    raw["source"]["root"] = path(&f.source());
    raw["source"]["recorded_commit_id"] = json!(f.s);
    raw["source"]["source_commit_id"] = json!(f.s);
    raw["source"]["evidence"] = json!("live_assertion");
    raw["source"]["assertion"] = json!({"revset":fixture::exact(&f.s).unwrap(),"commit_id":f.s,"status":"succeeded"});
    raw["desired"] = json!({"kind":"revset","revset":fixture::exact(f.desired(case)).unwrap(),"commit_id":f.desired(case),"state":"resolved"});
    let mut receiver = raw["receivers"][0].clone();
    receiver["renewal_route"] = json!(route);
    receiver["desired_commit_id"] = json!(f.desired(case));
    receiver["saved"]["commit_id"] = json!("c".repeat(40));
    receiver["child"]["commit_id"] = json!("d".repeat(40));
    receiver["child"]["parent_commit_ids"] = json!([f.desired(case)]);
    receiver["native"]["manifest"]["desired"] = json!("8");
    receiver["native"]["manifest"]["installed"] = json!("8");
    receiver["native"]["renewal_effects"]["files_reflinked"] = json!("7");
    receiver["native"]["renewal_effects"]["symlinks_created"] = json!("1");
    let mut phases = vec!["capture"];
    if route == Route::ClearThenPopulate {
        phases.push("native_clear");
    }
    phases.extend(["native_child", "install", "admit_desired", "adopt"]);
    if case == Case::DesiredN2 {
        phases.push("restore");
    }
    if route == Route::ReplaceAuthorized {
        phases.push("reparent");
    }
    phases.push("native_verify");
    receiver["native"]["steps"] = json!(
        phases
            .into_iter()
            .map(|phase| json!({"phase":phase,"status":"succeeded","elapsed_ms":"1"}))
            .collect::<Vec<_>>()
    );
    let mut ordinary: Value = serde_json::from_str(include_str!("../../../../../app/cli/tests/fixtures/refill/ordinary-wide.json")).unwrap();
    for key in [
        "deviated",
        "retained",
        "applications",
        "planned",
        "opaque_deviated",
        "additional_retained",
        "excluded",
    ] {
        ordinary[key] = json!("0");
    }
    ordinary["satisfied"] = json!(true);
    ordinary["end"] = Value::Null;
    ordinary["pruned"] = json!(false);
    ordinary["observed"]["state"] = json!("complete");
    for key in EFFECTS {
        ordinary["completed"][key] = json!("0");
    }
    receiver["ordinary"] = json!({"state":"satisfied","report":ordinary,"application_ms":null});
    raw["receivers"] = json!(
        (0..case.receivers())
            .map(|index| {
                let mut row = receiver.clone();
                row["name"] = json!(["worker-a", "worker-b"][index]);
                row["root"] = path(&f.worker(index));
                row
            })
            .collect::<Vec<_>>()
    );
    raw["selection"]["names"] = json!(["worker-a", "worker-b"][..case.receivers()]);
    raw["selection"]["resolved"] = json!(
        (0..case.receivers())
            .map(|index| json!({"name":(["worker-a","worker-b"][index]),"root":path(&f.worker(index)),"origins":["positional"]}))
            .collect::<Vec<_>>()
    );
    raw
}

#[test]
fn all_case_route_reports_bind_exact_cohort_pins_and_profile() {
    let own = scratch();
    let f = Fixture::fake(own.path(), control(own.path(), Path::new("/usr/bin/true")));
    for case in Case::ALL {
        for route in Route::ALL {
            let raw = complete(&f, case, route);
            let checked = evidence(&raw, &f, case, route).unwrap();
            assert_eq!(checked["measurement_profile"], PROFILE);
            assert_eq!(checked["outcome"], raw);
            let mut mutants = vec![];
            for (pointer, value) in [
                ("/schema_version", json!("1")),
                ("/schema_version", json!(3)),
                ("/command", json!("prepare")),
                ("/mode", json!("preview")),
                ("/status", json!("failed")),
                ("/error", json!({"message":"original failure"})),
                ("/renewal_route", json!("wrong-route")),
                ("/native_profile/id", json!("other-profile")),
                ("/native_profile/raw_version", json!("jj other-build")),
                ("/source/evidence", json!("assumed_prepared")),
                ("/source/assertion/status", json!("pending")),
                ("/source/assertion/commit_id", json!("b".repeat(40))),
                ("/source/source_commit_id", json!("b".repeat(40))),
                ("/source/root/unix_bytes_hex", json!("2f77726f6e67")),
                ("/desired/commit_id", json!("wrong-D")),
                ("/receivers/0/name", json!("worker-b")),
                ("/receivers/0/status", json!("partial")),
                ("/receivers/0/desired_commit_id", json!("wrong-D")),
                ("/receivers/0/child/parent_commit_ids", json!(["wrong-parent"])),
                ("/receivers/0/native/state", json!("in_progress")),
                ("/receivers/0/native/source_state_adopted", json!(false)),
                ("/receivers/0/native/manifest/complete", json!(false)),
                ("/receivers/0/native/manifest/installed", json!("7")),
                ("/receivers/0/native/manifest/owned_obsolete", json!("1")),
                ("/receivers/0/native/renewal_effects/files_reflinked", json!("6")),
                ("/receivers/0/native/renewal_effects/symlinks_created", json!("0")),
                ("/receivers/0/native/renewal_effects/bytes_copied", json!("1")),
                ("/receivers/0/ordinary/report/satisfied", json!(false)),
                ("/receivers/0/finish/state", json!("partial")),
                ("/receivers/0/native/steps", json!([])),
            ] {
                let mut bad = raw.clone();
                *bad.pointer_mut(pointer).unwrap() = value;
                mutants.push((pointer, bad));
            }
            for (label, bad) in mutants {
                assert!(evidence(&bad, &f, case, route).is_err(), "accepted invalid {label}: {case:?}/{route:?}");
            }
            let mut reordered = raw.clone();
            reordered["receivers"].as_array_mut().unwrap().reverse();
            if case.receivers() == 2 {
                assert!(evidence(&reordered, &f, case, route).is_err());
            }
            reordered["receivers"].as_array_mut().unwrap().pop();
            assert!(evidence(&reordered, &f, case, route).is_err());
        }
    }
}

#[test]
fn schema_two_unconditional_profiles_require_new_options_retention_and_metadata_effects() {
    let own = scratch();
    let f = Fixture::fake(own.path(), control(own.path(), Path::new("/usr/bin/true")));
    for route in Route::ALL {
        let mut raw = complete(&f, Case::CleanN1, route);
        raw["schema_version"] = json!(2);
        raw["refresh_check"] = json!("mtime,size");
        raw["postcheck"] = json!("none");
        raw["receivers"][0]["native"]["manifest"]["retained"] = json!("0");
        raw["receivers"][0]["native"]["retained_permissions_set"] = json!("0");
        assert!(evidence(&raw, &f, Case::CleanN1, route).is_ok());
        for pointer in [
            "/refresh_check",
            "/postcheck",
            "/receivers/0/native/manifest/retained",
            "/receivers/0/native/retained_permissions_set",
        ] {
            let mut bad = raw.clone();
            *bad.pointer_mut(pointer).unwrap() = Value::Null;
            assert!(evidence(&bad, &f, Case::CleanN1, route).is_err(), "accepted absent {pointer}");
        }
        for (pointer, value) in [
            ("/refresh_check", json!("size")),
            ("/postcheck", json!("mtime")),
            ("/renewal_route", json!("refresh")),
            ("/receivers/0/native/manifest/retained", json!("1")),
            ("/receivers/0/native/retained_permissions_set", json!("1")),
        ] {
            let mut bad = raw.clone();
            *bad.pointer_mut(pointer).unwrap() = value;
            assert!(evidence(&bad, &f, Case::CleanN1, route).is_err(), "accepted changed profile {pointer}");
        }
    }
}

#[test]
fn required_objects_counters_and_optional_keys_cannot_disappear() {
    let own = scratch();
    let f = Fixture::fake(own.path(), control(own.path(), Path::new("/usr/bin/true")));
    let raw = complete(&f, Case::CleanN1, Route::ReplaceAuthorized);
    for (parent, key) in [
        ("", "error"),
        ("", "capture"),
        ("/source_diagnostics", "ordinary"),
        ("/timings", "command_ms"),
        ("/receivers/0/native", "manifest"),
        ("/receivers/0/native", "renewal_effects"),
        ("/receivers/0/native", "tracked_install_ms"),
        ("/receivers/0/native/steps/0", "elapsed_ms"),
        ("/receivers/0/ordinary/report", "completed"),
        ("/receivers/0/ordinary/report/completed", "files_copied"),
        ("/receivers/0/ordinary/report", "retained"),
        ("/receivers/0/finish", "links"),
    ] {
        let mut bad = raw.clone();
        bad.pointer_mut(parent).unwrap().as_object_mut().unwrap().remove(key);
        assert!(evidence(&bad, &f, Case::CleanN1, Route::ReplaceAuthorized).is_err(), "missing {parent}/{key}");
    }
    for pointer in [
        "/receivers/0/native/manifest",
        "/receivers/0/native/renewal_effects",
        "/receivers/0/ordinary/report/completed",
        "/receivers/0/finish/links",
    ] {
        let mut bad = raw.clone();
        *bad.pointer_mut(pointer).unwrap() = Value::Null;
        assert!(evidence(&bad, &f, Case::CleanN1, Route::ReplaceAuthorized).is_err());
    }
    for pointer in [
        "/receivers/0/native/manifest/desired",
        "/receivers/0/native/renewal_effects/bytes_reflinked",
        "/receivers/0/ordinary/report/retained",
        "/receivers/0/finish/links/satisfied",
        "/source_diagnostics/native/declared/metadata_acquisitions",
        "/capture/desired_manifests",
        "/timings/engine_ms",
    ] {
        for invalid in [
            json!(0),
            json!("00"),
            json!("+1"),
            json!("-1"),
            json!("1.0"),
            json!(" 1"),
            json!("340282366920938463463374607431768211456"),
        ] {
            let mut bad = raw.clone();
            *bad.pointer_mut(pointer).unwrap() = invalid;
            assert!(evidence(&bad, &f, Case::CleanN1, Route::ReplaceAuthorized).is_err(), "numeric {pointer}");
        }
    }
}

#[test]
fn optional_nulls_stay_null_and_missing_work_cannot_be_compared_as_zero() {
    let own = scratch();
    let f = Fixture::fake(own.path(), control(own.path(), Path::new("/usr/bin/true")));
    let mut raw = complete(&f, Case::MixedN2, Route::ReplaceAuthorized);
    for key in ["native", "ordinary"] {
        raw["source_diagnostics"][key] = Value::Null;
    }
    for key in ["command_ms", "engine_ms", "pre_native_ms", "ordinary_ms"] {
        raw["timings"][key] = Value::Null;
    }
    raw["source"]["elapsed_ms"] = Value::Null;
    for receiver in raw["receivers"].as_array_mut().unwrap() {
        for key in ["elapsed_ms", "tracked_install_ms"] {
            receiver["native"][key] = Value::Null;
        }
        receiver["finish"]["elapsed_ms"] = Value::Null;
        for step in receiver["native"]["steps"].as_array_mut().unwrap() {
            step["elapsed_ms"] = Value::Null;
        }
    }
    let checked = evidence(&raw, &f, Case::MixedN2, Route::ReplaceAuthorized).unwrap();
    assert_eq!(checked["outcome"], raw);
    assert!(source_work(&checked, "ordinary", "paired", "listings").is_err());
    assert!(source_work(&checked, "native", "declared", "metadata_acquisitions").is_err());
}

#[test]
fn wide_exact_counts_and_unequal_deviation_retention_survive_without_arithmetic() {
    let own = scratch();
    let f = Fixture::fake(own.path(), control(own.path(), Path::new("/usr/bin/true")));
    let mut raw = complete(&f, Case::PairedN2, Route::ReplaceAuthorized);
    raw["receivers"][0]["ordinary"]["report"]["deviated"] = json!(u128::MAX.to_string());
    raw["receivers"][0]["ordinary"]["report"]["retained"] = json!("9007199254740993");
    raw["source_diagnostics"]["native"]["declared"]["metadata_acquisitions"] = json!("18446744073709551616");
    let checked = evidence(&raw, &f, Case::PairedN2, Route::ReplaceAuthorized).unwrap();
    assert_eq!(checked["outcome"], raw);
    assert_eq!(
        source_work(&checked, "native", "declared", "metadata_acquisitions").unwrap(),
        u64::MAX as u128 + 1
    );
    assert_eq!(decimal(&json!(u128::MAX.to_string())).unwrap(), u128::MAX);
    assert_eq!(decimal(&json!("0")).unwrap(), 0);
    raw["receivers"][0]["ordinary"]["report"]["completed"]["files_removed"] = json!("1");
    assert!(evidence(&raw, &f, Case::PairedN2, Route::ReplaceAuthorized).is_ok());
    let mut clean = complete(&f, Case::CleanN1, Route::ClearThenPopulate);
    clean["receivers"][0]["ordinary"]["report"]["completed"]["files_removed"] = json!("1");
    assert!(
        evidence(&clean, &f, Case::CleanN1, Route::ClearThenPopulate).is_err(),
        "zero different observations cannot substitute for zero effects"
    );
}

#[test]
fn lossless_unix_path_identity_ignores_display_but_rejects_bad_hex() {
    use std::os::unix::ffi::OsStrExt;
    let expected = Path::new(std::ffi::OsStr::from_bytes(b"/owned/byte-\xff"));
    let mut value = path(expected);
    value["display"] = json!("diagnostic text, not the path identity");
    wire_path(&value, expected).unwrap();
    for bad in ["f", "FF", "gg", "2f77726f6e67"] {
        value["unix_bytes_hex"] = json!(bad);
        assert!(wire_path(&value, expected).is_err());
    }
}
