//! Schema-one consumer, not a second producer. Required facts fail closed;
//! optional measurements retain explicit null and are never zero-filled.
use super::{Case, Fixture, PROFILE, Route, fixture};
use diff_dirs_perf_runner::Result;
use serde_json::{Value, json};
use std::{os::unix::ffi::OsStrExt, path::Path};

const NATIVE_PROFILE: &str = "local-tree-state-33d3a595";
const NATIVE_VERSION: &str = "jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b";
const EFFECTS: [&str; 10] = [
    "files_copied",
    "files_reflinked",
    "bytes_copied",
    "bytes_reflinked",
    "symlinks_created",
    "directories_created",
    "files_removed",
    "symlinks_removed",
    "directories_removed",
    "directory_permissions_set",
];

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value> {
    value.get(key).ok_or_else(|| format!("public report missing {key}").into())
}
fn require(actual: &Value, expected: Value, label: &str) -> Result<()> {
    if *actual != expected {
        return Err(format!("public report {label}: expected {expected}, got {actual}").into());
    }
    Ok(())
}
fn eq(value: &Value, key: &str, expected: impl Into<Value>) -> Result<()> {
    require(field(value, key)?, expected.into(), key)
}
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    field(value, key)?
        .as_str()
        .ok_or_else(|| format!("public report {key} must be a string").into())
}
fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    field(value, key)?
        .as_array()
        .ok_or_else(|| format!("public report {key} must be an array").into())
}

/// Check syntax before conversion: Rust's integer parser also accepts a '+' sign.
fn decimal(value: &Value) -> Result<u128> {
    let s = value.as_str().ok_or("public report counter must be a canonical decimal string")?;
    if s.is_empty() || (s.len() > 1 && s.starts_with('0')) || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err("public report counter must be a canonical unsigned decimal string".into());
    }
    s.parse::<u128>().map_err(|_| "public report counter overflows u128".into())
}
fn count(value: &Value, key: &str) -> Result<u128> {
    decimal(field(value, key)?)
}
fn counters(value: &Value, keys: &[&str]) -> Result<()> {
    for key in keys {
        count(value, key)?;
    }
    Ok(())
}
fn optional_ms(value: &Value, key: &str) -> Result<()> {
    let value = field(value, key)?;
    if !value.is_null() {
        decimal(value)?;
    }
    Ok(())
}
fn wire_path(value: &Value, expected: &Path) -> Result<()> {
    text(value, "display")?; // Diagnostic only; identity is the exact Unix bytes.
    let hex = text(value, "unix_bytes_hex")?;
    if !hex.len().is_multiple_of(2) || !hex.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
        return Err("public report path must have lowercase even-length Unix hex".into());
    }
    let bytes = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).expect("ASCII checked"), 16))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if bytes != expected.as_os_str().as_bytes() {
        return Err(format!("public report root differs from bound fixture: {}", expected.display()).into());
    }
    Ok(())
}
fn diagnostics(value: &Value) -> Result<()> {
    if value.is_null() {
        return Ok(());
    }
    for kind in ["paired", "opaque", "declared"] {
        counters(
            field(value, kind)?,
            &[
                "listings",
                "metadata_acquisitions",
                "link_acquisitions",
                "fact_hits",
                "cache_hits",
                "cache_misses",
            ],
        )?;
    }
    counters(value, &["retained_batches", "retained_charged_capacity", "active_batches", "evictions"])
}

pub(super) fn evidence(raw: &Value, f: &Fixture, case: Case, route: Route) -> Result<Value> {
    f.check_profile()?;
    // schema_version is the sole JSON numeric value in this wire vocabulary.
    if field(raw, "schema_version")?.as_u64() != Some(1) {
        return Err("unsupported public refill schema_version (requires numeric 1)".into());
    }
    eq(raw, "command", "reseed")?;
    eq(raw, "mode", "execute")?;
    eq(raw, "status", "complete")?;
    eq(raw, "error", Value::Null)?;
    eq(raw, "preview", Value::Null)?;
    eq(raw, "renewal_route", route.label())?;
    let profile = field(raw, "native_profile")?;
    eq(profile, "id", NATIVE_PROFILE)?;
    eq(profile, "raw_version", NATIVE_VERSION)?;
    let source = field(raw, "source")?;
    eq(source, "name", "default")?;
    wire_path(field(source, "root")?, &f.source())?;
    eq(source, "state", "ready")?;
    eq(source, "recorded_commit_id", f.s.as_str())?;
    eq(source, "source_commit_id", f.s.as_str())?;
    for key in ["operation_id", "source_change_id"] {
        if text(source, key)?.is_empty() {
            return Err(format!("public report {key} must be known").into());
        }
    }
    eq(source, "evidence", "live_assertion")?;
    let assertion = field(source, "assertion")?;
    eq(assertion, "revset", fixture::exact(&f.s)?)?;
    eq(assertion, "commit_id", f.s.as_str())?;
    eq(assertion, "status", "succeeded")?;
    for key in ["upstream_goal_commit_id", "upstream", "preparation"] {
        eq(source, key, Value::Null)?;
    }
    optional_ms(source, "elapsed_ms")?;
    let desired = field(raw, "desired")?;
    eq(desired, "kind", "revset")?;
    eq(desired, "revset", fixture::exact(f.desired(case))?)?;
    eq(desired, "commit_id", f.desired(case))?;
    eq(desired, "state", "resolved")?;

    let names = ["worker-a", "worker-b"][..case.receivers()].to_vec();
    let selection = field(raw, "selection")?;
    eq(selection, "names", json!(names))?;
    eq(selection, "groups", json!([]))?;
    eq(selection, "skipped", json!([]))?;
    let selected = array(selection, "resolved")?;
    let receivers = array(raw, "receivers")?;
    if selected.len() != names.len() || receivers.len() != names.len() {
        return Err("public report ordered receiver cohort has wrong length".into());
    }
    for (index, (selected, receiver)) in selected.iter().zip(receivers).enumerate() {
        eq(selected, "name", names[index])?;
        wire_path(field(selected, "root")?, &f.worker(index))?;
        // Origins are labels owned by selection, not inferred from argv.
        if array(selected, "origins")?.is_empty() || array(selected, "origins")?.iter().any(|v| !v.is_string()) {
            return Err("public report selection origins must be known labels".into());
        }
        eq(receiver, "name", names[index])?;
        wire_path(field(receiver, "root")?, &f.worker(index))?;
        eq(receiver, "renewal_route", route.label())?;
        eq(receiver, "status", "complete")?;
        eq(receiver, "phase", "complete")?;
        eq(receiver, "desired_commit_id", f.desired(case))?;
        let saved = field(receiver, "saved")?;
        fixture::exact(text(saved, "commit_id")?)?;
        let history = field(saved, "history")?;
        eq(history, "status", "covered")?;
        for key in ["bookmark", "bookmark_target_commit_id", "work_tip_commit_id"] {
            if text(history, key)?.is_empty() {
                return Err("public report saved history coverage is unavailable".into());
            }
        }
        let child = field(receiver, "child")?;
        fixture::exact(text(child, "commit_id")?)?;
        if text(child, "change_id")?.is_empty() || child["change_id"] == source["source_change_id"] {
            return Err("public report child change identity is missing or aliases source".into());
        }
        eq(child, "parent_commit_ids", json!([f.desired(case)]))?;
        let native = field(receiver, "native")?;
        eq(native, "state", "established")?;
        eq(native, "source_state_adopted", true)?;
        eq(native, "explicit_clear_effects", Value::Null)?;
        let manifest = field(native, "manifest")?;
        eq(manifest, "complete", true)?;
        if count(manifest, "desired")? != 8 || count(manifest, "installed")? != 8 {
            return Err("public report manifest must completely install all eight S leaves".into());
        }
        if count(manifest, "owned_obsolete")? != count(manifest, "owned_retired")? {
            return Err("public report manifest has incomplete owned retirement".into());
        }
        let effects = field(native, "renewal_effects")?;
        counters(effects, &EFFECTS)?;
        if count(effects, "files_reflinked")? != 7 || count(effects, "symlinks_created")? != 1 {
            return Err("public report requires seven actual regular clones and one literal-link install per receiver".into());
        }
        if count(effects, "files_copied")? != 0 || count(effects, "bytes_copied")? != 0 {
            return Err("public report renewal byte-copy fallback is forbidden".into());
        }
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
        let steps = array(native, "steps")?;
        if steps.len() != phases.len() {
            return Err("public report native phase sequence is incomplete".into());
        }
        for (step, phase) in steps.iter().zip(phases) {
            eq(step, "phase", phase)?;
            eq(step, "status", "succeeded")?;
            optional_ms(step, "elapsed_ms")?;
        }
        for key in ["elapsed_ms", "tracked_install_ms"] {
            optional_ms(native, key)?;
        }
        let ordinary = field(receiver, "ordinary")?;
        eq(ordinary, "state", "satisfied")?;
        optional_ms(ordinary, "application_ms")?;
        let report = field(ordinary, "report")?;
        eq(report, "satisfied", true)?;
        eq(report, "end", Value::Null)?;
        eq(report, "pruned", false)?;
        counters(
            report,
            &[
                "deviated",
                "retained",
                "applications",
                "planned",
                "opaque_deviated",
                "additional_retained",
                "excluded",
            ],
        )?;
        let completed = field(report, "completed")?;
        counters(completed, &EFFECTS)?;
        if count(completed, "files_copied")? != 0 || count(completed, "bytes_copied")? != 0 {
            return Err("public report ordinary byte-copy fallback is forbidden".into());
        }
        if case == Case::CleanN1 && EFFECTS.iter().any(|key| count(completed, key).expect("validated counters") != 0) {
            return Err("clean ordinary work must return actual zero effects".into());
        }
        let observed = field(report, "observed")?;
        eq(observed, "state", "complete")?;
        if !field(observed, "scoped")?.is_boolean() || !(field(observed, "comparator")?.is_null() || observed["comparator"].is_string()) {
            return Err("public report invalid ordinary observation attributes".into());
        }
        counters(
            observed,
            &[
                "excluded",
                "planned",
                "deviated",
                "retained",
                "equal",
                "different",
                "not_assessed",
                "unsupported",
                "unresolved",
                "stopped_paired",
                "examined_directories",
            ],
        )?;
        let finish = field(receiver, "finish")?;
        eq(finish, "state", "complete")?;
        eq(finish, "failed_path", Value::Null)?;
        counters(
            field(finish, "links")?,
            &["linked", "directories_created", "satisfied", "skipped_missing_target"],
        )?;
        optional_ms(finish, "elapsed_ms")?;
    }
    counters(
        field(raw, "capture")?,
        &[
            "distinct_trees",
            "membership_queries",
            "indexes",
            "donor_cache_reads",
            "desired_manifests",
            "owned_manifests",
        ],
    )?;
    let source_diagnostics = field(raw, "source_diagnostics")?;
    for kind in ["native", "ordinary"] {
        diagnostics(field(source_diagnostics, kind)?)?;
    }
    for key in ["command_ms", "engine_ms", "pre_native_ms", "ordinary_ms"] {
        optional_ms(field(raw, "timings")?, key)?;
    }
    // Keep every actual phase, counter (including separate deviation/retention),
    // optional null and lossless path, without numeric JSON/u64 conversion.
    Ok(json!({"measurement_profile":PROFILE,"outcome":raw}))
}

/// Comparisons must explicitly request available work; absent instrumentation
/// is never interpreted as a measured zero or as proof of source sharing.
#[cfg(test)]
fn source_work(evidence: &Value, phase: &str, kind: &str, counter: &str) -> Result<u128> {
    count(field(field(field(field(evidence, "outcome")?, "source_diagnostics")?, phase)?, kind)?, counter)
}

#[cfg(test)]
mod tests;
