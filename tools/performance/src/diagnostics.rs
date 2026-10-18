use diff_dirs_perf_runner::{Result, fixture, process, write_json};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path, process::Command, time::Duration};

fn successful(root: &Path) -> Result<(Value, Vec<Value>)> {
    let checked = diff_dirs_perf_runner::report::retained(root)?;
    if checked["campaign_passed"] != true {
        return Err("refuse failed/incomplete raw chronology".into());
    }
    let summary: Value = serde_json::from_slice(&fs::read(root.join("summary.json"))?)?;
    let campaign: Value = serde_json::from_slice(&fs::read(root.join("campaign.json"))?)?;
    if summary["passed"] != true {
        return Err("refuse failed/incomplete campaign".into());
    }
    let mut samples = Vec::new();
    for line in fs::read_to_string(root.join("samples.jsonl"))?.lines() {
        let sample: Value = serde_json::from_str(line)?;
        if !sample["error"].is_null() {
            return Err("refuse failed sample".into());
        }
        samples.push(sample);
    }
    if samples.is_empty() {
        return Err("empty campaign".into());
    }
    Ok((campaign, samples))
}

fn metrics(samples: &[Value]) -> Result<BTreeMap<String, u64>> {
    let mut result = BTreeMap::new();
    for sample in samples {
        let label = sample["label"].as_str().ok_or("missing label")?;
        let phases = sample["details"]["phases"].as_object().ok_or("missing measured phase counts")?;
        let mut totals = BTreeMap::<&str, u64>::new();
        for (phase, row) in phases {
            for metric in ["jj_calls", "file_show_calls", "git_calls"] {
                let count = row["counts"][metric].as_u64().ok_or("count gates require instrumented mode")?;
                *totals.entry(metric).or_default() += count;
                let key = format!("{label}/{phase}/{metric}");
                let old = result.entry(key).or_insert(0);
                *old = (*old).max(count);
            }
        }
        for (metric, count) in totals {
            let key = format!("{label}/total/{metric}");
            let old = result.entry(key).or_insert(0);
            *old = (*old).max(count);
        }
    }
    Ok(result)
}

pub fn check(root: &Path, budgets: &[(String, String)]) -> Result<()> {
    let (_, samples) = successful(root)?;
    let counts = metrics(&samples)?;
    for (metric, spec) in budgets {
        let (phase, limit) = spec.split_once('=').ok_or("budget syntax PHASE=N")?;
        let limit: u64 = limit.parse()?;
        if !["prepare", "preview", "reseed", "total"].contains(&phase) {
            return Err("unknown budget phase".into());
        }
        let suffix = format!("/{phase}/{metric}");
        let selected: Vec<_> = counts.iter().filter(|(key, _)| key.ends_with(&suffix)).collect();
        if selected.is_empty() {
            return Err("budget phase was not measured".into());
        }
        for (key, count) in selected {
            if *count > limit {
                return Err(format!("budget failed: {key}={count} > {limit}").into());
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&json!({"max_observed_counts":counts,"status":"passed"}))?);
    Ok(())
}

pub fn compare(before: &Path, after: &Path, scaling: bool, growth: &[(String, u64)]) -> Result<()> {
    let (a, sa) = successful(before)?;
    let (b, sb) = successful(after)?;
    let mut ca = a["workload"]["config"].clone();
    let mut cb = b["workload"]["config"].clone();
    if scaling {
        if cb["files"].as_u64().ok_or("missing tracked count")? <= ca["files"].as_u64().ok_or("missing tracked count")? {
            return Err("scaling requires increased regular-file count".into());
        }
        ca.as_object_mut().ok_or("missing config")?.remove("files");
        cb.as_object_mut().ok_or("missing config")?.remove("files");
        let identities = |campaign: &Value| {
            campaign["controls"]
                .as_array()
                .map(|controls| controls.iter().map(|c| (c["label"].clone(), c["sha256"].clone())).collect::<Vec<_>>())
        };
        if identities(&a) != identities(&b) {
            return Err("scaling requires identical native JW artifacts".into());
        }
    }
    if ca != cb {
        return Err("incompatible fixture/phase/timing configuration".into());
    }
    for tool in ["jj", "git", "adapter"] {
        if a["workload"]["environment"][tool]["sha256"] != b["workload"]["environment"][tool]["sha256"] {
            return Err(format!("incompatible native {tool} executable").into());
        }
    }
    if a["runner"] != b["runner"] {
        return Err("incompatible Rust engine source/compiler/profile".into());
    }
    if !growth.is_empty() {
        let old = metrics(&sa)?;
        let new = metrics(&sb)?;
        if old.keys().collect::<Vec<_>>() != new.keys().collect::<Vec<_>>() {
            return Err("incompatible count groups".into());
        }
        for (metric, max) in growth {
            for (key, value) in &new {
                if key.ends_with(&format!("/{metric}")) {
                    let delta = *value as i128 - old[key] as i128;
                    if delta > *max as i128 {
                        return Err(format!("count growth failed: {key}={delta} > {max}").into());
                    }
                }
            }
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({"compatible":true,"scaling":scaling,
        "before":serde_json::from_slice::<Value>(&fs::read(before.join("summary.json"))?)?,
        "after":serde_json::from_slice::<Value>(&fs::read(after.join("summary.json"))?)?,
        "limits":"counts gate maximum observed phase counts; independent campaigns are not silently pooled or causal latency evidence"}))?
    );
    Ok(())
}

pub fn copy_baseline(source: &Path, output: &Path) -> Result<()> {
    let _lock = diff_dirs_perf_runner::host_lock()?;
    let source = fs::canonicalize(source)?;
    if fs::read_to_string(source.join(".perf-owned-native"))? != source.display().to_string() {
        return Err("copy baseline requires its original owned native source".into());
    }
    if fixture::fresh_path(output)?.starts_with(&source) {
        return Err("copy output must be outside its read-only source".into());
    }
    fs::create_dir(output)?;
    let output = fs::canonicalize(output)?;
    let target = output.join("copy");
    fs::create_dir(&target)?;
    let mut entries = fs::read_dir(&source)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|e| e.file_name());
    entries.retain(|e| !matches!(e.file_name().to_str(), Some(".jj" | ".git" | ".perf-owned-native" | ".jwreseed.local.toml")));
    let expected: Vec<_> = entries
        .iter()
        .map(|e| Ok((e.file_name(), fixture::payload(&e.path())?)))
        .collect::<Result<_>>()?;
    let mut command = Command::new("/usr/bin/cp");
    command.args(["-a", "--reflink=always", "--target-directory"]).arg(&target).arg("--");
    for entry in &entries {
        command.arg(entry.path());
    }
    let result = process::run(&mut command, &output.join("copy-process"), Duration::from_secs(1800))?;
    if result.code != Some(0) || result.timed_out {
        return Err("raw copying failed".into());
    }
    for (name, hash) in expected {
        if fixture::payload(&target.join(name))? != hash {
            return Err("raw copy payload mismatch".into());
        }
    }
    write_json(
        &output.join("result.json"),
        &json!({"passed":true,"process":result,"source":source,"target":target,
        "scope":"native GNU cp of owned tracked/payload selections, excluding JJ admin and fixture configuration; not whole reseed"}),
    )?;
    println!("{}", output.join("result.json").display());
    Ok(())
}

#[test]
fn budgets_require_real_instrumentation_and_keep_argument_counts() {
    let samples = vec![json!({"label":"baseline","details":{"phases":{"reseed":{"counts":{"jj_calls":7,"file_show_calls":2,"git_calls":3}}}}})];
    let m = metrics(&samples).unwrap();
    assert_eq!(m["baseline/total/jj_calls"], 7);
    assert_eq!(m["baseline/reseed/file_show_calls"], 2);
    assert!(metrics(&[json!({"label":"baseline","details":{"phases":{"reseed":{"counts":null}}}})]).is_err());
}
