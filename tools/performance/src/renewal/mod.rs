//! Deliberately small two-route adapter. This is not the fresh-per-sample campaign.
mod fixture;
#[cfg(test)]
mod tests;

use crate::{context::Context, instrumentation, workload::Timing};
use clap::{Subcommand, ValueEnum};
use diff_dirs_perf_runner::{
    Result,
    artifacts::{self, Control},
    execution::{self, Sample, Workload},
    fixture::fresh_path,
    write_json,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    fs::OpenOptions,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Instant,
};

use fixture::{Fixture, Start};

#[derive(Subcommand)]
pub enum Action {
    /// Create ONE owned donor and TWO receivers; no renewal or timing.
    Init {
        root: PathBuf,
        #[arg(long)]
        candidate: PathBuf,
        #[arg(long)]
        candidate_provenance: PathBuf,
        #[arg(long, default_value = "/usr/local/bin/jj")]
        jj: PathBuf,
        #[arg(long, default_value = "/usr/bin/git")]
        git: PathBuf,
    },
    /// Gate the real public command on four tiny shapes, both routes, sequentially.
    Accept { root: PathBuf, output: PathBuf },
    /// Requires successful bound acceptance. Four native samples OR two count probes.
    Sample {
        root: PathBuf,
        output: PathBuf,
        #[arg(long)]
        acceptance: PathBuf,
        #[arg(long, value_enum)]
        case: Case,
        #[arg(long, value_enum, default_value = "native")]
        timing: Timing,
        #[arg(long, default_value_t = 1701)]
        seed: u64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Case {
    CleanN1,
    PairedN2,
    MixedN2,
    DesiredN2,
}
impl Case {
    const ALL: [Self; 4] = [Self::CleanN1, Self::PairedN2, Self::MixedN2, Self::DesiredN2];
    fn receivers(self) -> usize {
        if self == Self::CleanN1 { 1 } else { 2 }
    }
    fn label(self) -> &'static str {
        match self {
            Self::CleanN1 => "clean-n1",
            Self::PairedN2 => "paired-n2",
            Self::MixedN2 => "mixed-n2",
            Self::DesiredN2 => "desired-n2",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Route {
    ClearThenPopulate,
    ReplaceAuthorized,
}
impl Route {
    const ALL: [Self; 2] = [Self::ClearThenPopulate, Self::ReplaceAuthorized];
    fn label(self) -> &'static str {
        match self {
            Self::ClearThenPopulate => "clear-then-populate",
            Self::ReplaceAuthorized => "replace-authorized",
        }
    }
    fn from_label(label: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|r| r.label() == label)
            .ok_or_else(|| "unknown renewal route label".into())
    }
}

pub fn run(action: Action) -> Result<()> {
    match action {
        Action::Init {
            root,
            candidate,
            candidate_provenance,
            jj,
            git,
        } => {
            let _lock = diff_dirs_perf_runner::host_lock()?;
            let control = crate::import("candidate", candidate, candidate_provenance)?;
            build_provenance(&control.provenance)?;
            let root = fresh_path(&root)?;
            fs::create_dir(&root)?;
            let controls = artifacts::preserve(&[control], &root)?;
            let mut control = controls.into_iter().next().ok_or("missing candidate")?;
            let provenance = root.join("artifacts/provenance.json");
            fs::copy(&control.provenance_path, &provenance)?;
            control.provenance_path = provenance;
            check_control(&control)?;
            let logs = root.join("setup");
            fs::create_dir(&logs)?;
            let mut ctx = context(&logs, &instrumentation::native(&jj)?, &instrumentation::native(&git)?)?;
            let f = Fixture::create(&root, control, &mut ctx)?;
            write_json(&root.join("fixture.json"), &f)?;
            println!("{}", root.join("fixture.json").display());
        }
        Action::Accept { root, output } => accept(&root, &output)?,
        Action::Sample {
            root,
            output,
            acceptance,
            case,
            timing,
            seed,
        } => {
            // Check BEFORE output creation, reset, instrumentation or performance.
            let fixture = Fixture::open(&root)?;
            check_acceptance(&fixture, &acceptance)?;
            outside_fixture(&fixture, &fresh_path(&output)?)?;
            let controls = Route::ALL.map(|route| {
                let mut c = fixture.control.clone();
                c.label = route.label().into();
                c
            });
            let mut work = Renewal {
                fixture,
                output: output.clone(),
                case,
                timing,
                ctx: None,
                matched: None,
                halted: false,
            };
            // The shared engine supplies exactly A/B/B/A (or its seeded mirror).
            // Count probes are deliberately a different, non-latency invocation.
            if timing == Timing::Native {
                if !execution::balanced(&output, &controls, 2, seed, &mut work)? {
                    return Err("renewal samples failed; raw attempts retained".into());
                }
            } else {
                let _lock = diff_dirs_perf_runner::host_lock()?;
                let output = fresh_path(&output)?;
                outside_fixture(&work.fixture, &output)?;
                fs::create_dir(&output)?;
                work.fixture.verify_bindings()?;
                let initialized = work.initialize_at(&output)?;
                write_json(
                    &output.join("probe.json"),
                    &json!({"workload":initialized,"timing":"instrumented counts only; not native latency"}),
                )?;
                for (index, control) in controls.iter().enumerate() {
                    let mut sample = empty_sample(index, &control.label);
                    let result = work.sample(control, &output.join(format!("probe-{index}")), &mut sample);
                    if let Err(error) = &result {
                        sample.error = Some(error.to_string());
                    }
                    write_json(&output.join(format!("probe-{index}.json")), &sample)?;
                    result?;
                }
                work.finish()?;
            }
        }
    }
    Ok(())
}

fn build_provenance(value: &Value) -> Result<()> {
    for key in ["source_commit", "library_commit", "rustc", "build_command", "profile", "sha256"] {
        if value[key].as_str().is_none_or(str::is_empty) {
            return Err(format!("renewal requires jw-perf build provenance: missing {key}").into());
        }
    }
    for key in ["source_commit", "library_commit"] {
        fixture::exact(value[key].as_str().ok_or("missing source pin")?)?;
    }
    for key in ["source_inputs", "library_inputs", "build_environment"] {
        if !value[key].is_object() {
            return Err(format!("build provenance missing {key}").into());
        }
    }
    if !value["features"].is_array() {
        return Err("build provenance missing exact features".into());
    }
    Ok(())
}
fn check_control(control: &Control) -> Result<()> {
    artifacts::verify(control)?;
    if artifacts::sha256(&control.provenance_path)? != control.provenance_sha256 {
        return Err("candidate build sidecar changed".into());
    }
    let value: Value = serde_json::from_slice(&fs::read(&control.provenance_path)?)?;
    if value != control.provenance || value["sha256"] != control.sha256 {
        return Err("candidate build provenance identity changed".into());
    }
    build_provenance(&value)
}

fn context(output: &Path, jj: &Path, git: &Path) -> Result<Context> {
    let own = output.join("adapter");
    fs::copy(std::env::current_exe()?, &own)?;
    fs::set_permissions(&own, fs::Permissions::from_mode(0o555))?;
    let ctx = Context::new(output, jj, git, &own)?;
    write_json(
        &output.join("environment.json"),
        &json!({"tools":ctx.identities()?,"env":ctx.env,"runner":diff_dirs_perf_runner::source_identity()}),
    )?;
    Ok(ctx)
}
fn outside_fixture(fixture: &Fixture, output: &Path) -> Result<()> {
    if output.starts_with(fixture.root.join("trees")) {
        return Err("artifacts/logs must be outside the ordinary fixture".into());
    }
    Ok(())
}

fn argv(fixture: &Fixture, case: Case, route: Route) -> Result<Vec<String>> {
    let mut args = vec!["reseed".into()];
    args.extend(["worker-a", "worker-b"].into_iter().take(case.receivers()).map(str::to_owned));
    args.extend([
        "--from".into(),
        fixture.source().to_str().ok_or("non-UTF8 fixture path")?.into(),
        "--source-at".into(),
        fixture::exact(&fixture.s)?,
        "--at".into(),
        fixture::exact(fixture.desired(case))?,
        format!("--renewal-route={}", route.label()),
        "--format=json".into(),
    ]);
    Ok(args)
}

fn capability(help: &str) -> Result<()> {
    if !["--renewal-route", "clear-then-populate", "replace-authorized", "--format", "json"]
        .into_iter()
        .all(|s| help.contains(s))
    {
        return Err("candidate lacks the reviewed public renewal-route/JSON interface; legacy dispatch is NOT a route result".into());
    }
    Ok(())
}

/// Single deliberate integration seam: the public report is not settled yet.
/// Do NOT infer a route from argv/help or replace unavailable fields with zero.
/// Implement against the reviewed CLI envelope, checking actual route, S/D,
/// complete rows/coverage, required-clone successes/zero copies, clean ordinary
/// zero writes, paired acquisition vs opaque work, and retaining optional phases.
/// Failure process status is handled first, independently of JSON parsing.
fn report_evidence(_raw: &Value, _fixture: &Fixture, _case: Case, _route: Route) -> Result<Value> {
    Err("public command report extraction pending reviewed CLI fields; raw output retained; semantic acceptance and timing are blocked".into())
}

fn invoke(ctx: &mut Context, control: &Control, fixture: &Fixture, case: Case, route: Route, timing: Timing, stem: &Path) -> Result<Value> {
    let guard = Instant::now();
    check_control(control)?;
    let guard_ns = guard.elapsed().as_nanos();
    let trace = stem.with_extension("processes.jsonl");
    if timing == Timing::Instrumented {
        OpenOptions::new().write(true).create_new(true).open(&trace)?;
    }
    let args = argv(fixture, case, route)?;
    let (stdout, outcome) = ctx.attempt(
        &control.binary,
        &args,
        &fixture.source(),
        if timing == Timing::Instrumented { Some(&trace) } else { None },
    )?;
    // Preserve status and raw paths BEFORE parsing. Nonzero stdout may not be JSON.
    let observation = json!({"argv":args,"process":outcome,"stdout":stdout,"stderr":stdout.with_extension("stderr"),"artifact_guard_ns":guard_ns,
        "timing":timing,"trace":if timing == Timing::Instrumented { Some(&trace) } else { None },
        "reported_fields":"unavailable until reviewed public report extraction"});
    write_json(&stem.with_extension("command.json"), &observation)?;
    if outcome.code != Some(0) || outcome.timed_out {
        return Err(format!(
            "public renewal failed (exit {:?}, signal {:?}, timeout {}); original stderr: {}",
            outcome.code,
            outcome.signal,
            outcome.timed_out,
            stdout.with_extension("stderr").display()
        )
        .into());
    }
    Ok(observation)
}

fn accept(root: &Path, output: &Path) -> Result<()> {
    let _lock = diff_dirs_perf_runner::host_lock()?;
    let fixture = Fixture::open(root)?;
    let output = fresh_path(output)?;
    outside_fixture(&fixture, &output)?;
    fs::create_dir(&output)?;
    let mut ctx = context(&output, &fixture.jj, &fixture.git)?;
    let result = (|| -> Result<Vec<Value>> {
        fixture.verify(&mut ctx)?;
        let (help, _) = ctx.run(&fixture.control.binary, &["reseed".into(), "--help".into()], &fixture.source(), None)?;
        capability(&help)?;
        let mut rows = Vec::new();
        for case in Case::ALL {
            let mut matched = None;
            for route in Route::ALL {
                let stem = output.join(format!("{}-{}", case.label(), route.label()));
                let start = fixture.reset(&mut ctx, case)?;
                match_start(&mut matched, &start)?;
                write_json(&stem.with_extension("start.json"), &start)?;
                let observation = invoke(&mut ctx, &fixture.control, &fixture, case, route, Timing::Native, &stem)?;
                let physical = fixture.verify_result(&mut ctx, case, &start)?;
                write_json(&stem.with_extension("physical.json"), &physical)?;
                let raw: Value = serde_json::from_slice(&fs::read(observation["stdout"].as_str().ok_or("missing raw stdout path")?)?)?;
                let evidence = report_evidence(&raw, &fixture, case, route)?;
                rows.push(json!({"case":case,"route":route,"command":observation,"physical":physical,"report":evidence}));
            }
        }
        Ok(rows)
    })();
    write_json(
        &output.join("acceptance.json"),
        &json!({"passed":result.is_ok(),"binding":binding(&fixture)?,
        "cases":result.as_ref().ok(),"error":result.as_ref().err().map(ToString::to_string),
        "scope":"bounded public-command comparison gate, not comprehensive protocol approval or a timing result"}),
    )?;
    result?;
    Ok(())
}

fn binding(fixture: &Fixture) -> Result<Value> {
    Ok(json!({"fixture_sha256":artifacts::sha256(&fixture.root.join("fixture.json"))?,
        "candidate_sha256":fixture.control.sha256,"provenance_sha256":fixture.control.provenance_sha256,
        "adapter_sha256":artifacts::sha256(&std::env::current_exe()?)?,"jj":instrumentation::identity(&fixture.jj)?,"git":instrumentation::identity(&fixture.git)?}))
}
fn check_acceptance(fixture: &Fixture, path: &Path) -> Result<()> {
    let value: Value = serde_json::from_slice(&fs::read(path)?)?;
    if value["passed"] != true || value["binding"] != binding(fixture)? {
        return Err("successful acceptance for this exact fixture/candidate/compiler/features/native tools/adapter is required before performance".into());
    }
    let rows = value["cases"].as_array().ok_or("missing semantic acceptance rows")?;
    if rows.len() != 8 {
        return Err("both routes on all four fixture shapes must pass before timing".into());
    }
    for case in Case::ALL {
        for route in Route::ALL {
            let matching = rows
                .iter()
                .filter(|row| row["case"] == json!(case) && row["route"] == json!(route))
                .collect::<Vec<_>>();
            if matching.len() != 1 {
                return Err("missing or duplicated acceptance case/route".into());
            }
            let row = matching[0];
            let raw: Value = serde_json::from_slice(&fs::read(
                row["command"]["stdout"].as_str().ok_or("acceptance must retain original public stdout")?,
            )?)?;
            if report_evidence(&raw, fixture, case, route)? != row["report"] {
                return Err("accepted public report evidence changed".into());
            }
        }
    }
    Ok(())
}
fn match_start(matched: &mut Option<Value>, start: &Start) -> Result<()> {
    if matched.as_ref().is_some_and(|old| old != &start.signature) {
        return Err("unmatched reused fixture starting bytes/kinds/modes/mtimes".into());
    }
    *matched = Some(start.signature.clone());
    Ok(())
}

struct Renewal {
    fixture: Fixture,
    output: PathBuf,
    case: Case,
    timing: Timing,
    ctx: Option<Context>,
    matched: Option<Value>,
    halted: bool,
}
impl Renewal {
    fn initialize_at(&mut self, output: &Path) -> Result<Value> {
        outside_fixture(&self.fixture, output)?;
        let mut ctx = context(output, &self.fixture.jj, &self.fixture.git)?;
        self.fixture.verify(&mut ctx)?;
        let value = json!({"binding":binding(&self.fixture)?,"case":self.case,"timing":self.timing,"S":self.fixture.s,
            "D":self.fixture.desired(self.case),"native_residency":"one donor, two reused receiver roots; no import, remote or administration copy",
            "scope":"whole public command; reset, hashes and external verification excluded; reported phases optional, never zero-filled"});
        self.ctx = Some(ctx);
        Ok(value)
    }
    fn sample_once(&mut self, control: &Control, stem: &Path, sample: &mut Sample) -> Result<()> {
        let ctx = self.ctx.as_mut().ok_or("uninitialized renewal workload")?;
        let setup = Instant::now();
        self.fixture.verify(ctx)?;
        let start = self.fixture.reset(ctx, self.case)?;
        match_start(&mut self.matched, &start)?;
        write_json(&stem.with_extension("start.json"), &start)?;
        sample.preparation_ns += setup.elapsed().as_nanos();
        let route = Route::from_label(&control.label)?;
        sample.argv = argv(&self.fixture, self.case, route)?;
        if control.sha256 != self.fixture.control.sha256 || control.provenance_sha256 != self.fixture.control.provenance_sha256 {
            return Err("renewal route labels must use the identical preserved candidate and feature/build sidecar".into());
        }
        // Shared execution archives per-label copies. Both routes actually execute
        // the ONE original preserved candidate path, not separate build variants.
        let invoked = invoke(ctx, &self.fixture.control, &self.fixture, self.case, route, self.timing, stem);
        // Even failed invocations already have a raw process record on disk.
        let command_path = stem.with_extension("command.json");
        if command_path.exists() {
            let observation: Value = serde_json::from_slice(&fs::read(command_path)?)?;
            sample.process_ns = observation["process"]["elapsed_ns"].as_u64().ok_or("missing external process elapsed time")? as u128;
            sample.preparation_ns += observation["artifact_guard_ns"].as_u64().ok_or("missing artifact guard duration")? as u128;
            sample.exit_code = observation["process"]["code"].as_i64().map(|v| v as i32);
            sample.details["command"] = observation;
        }
        let observation = invoked?;
        let verify = Instant::now();
        let checked = (|| -> Result<Value> {
            let physical = self.fixture.verify_result(ctx, self.case, &start)?;
            let raw: Value = serde_json::from_slice(&fs::read(observation["stdout"].as_str().ok_or("missing stdout path")?)?)?;
            let report = report_evidence(&raw, &self.fixture, self.case, route)?;
            let counts = if self.timing == Timing::Instrumented {
                instrumentation::counts(&stem.with_extension("processes.jsonl"))?
            } else {
                Value::Null
            };
            Ok(json!({"physical":physical,"report":report,"counts":counts}))
        })();
        sample.verification_ns = verify.elapsed().as_nanos();
        sample.details["verification"] = checked?;
        sample.native_ns = if self.timing == Timing::Native { Some(sample.process_ns) } else { None };
        Ok(())
    }
}
impl Workload for Renewal {
    fn initialize(&mut self) -> Result<Value> {
        let output = fs::canonicalize(&self.output)?;
        self.initialize_at(&output)
    }
    fn sample(&mut self, control: &Control, stem: &Path, sample: &mut Sample) -> Result<()> {
        if self.halted {
            return Err("earlier renewal sample failed; no reset, continuation or route fallback attempted".into());
        }
        let result = self.sample_once(control, stem, sample);
        if result.is_err() {
            self.halted = true;
        }
        result
    }
    fn finish(&mut self) -> Result<Value> {
        self.fixture.verify(self.ctx.as_mut().ok_or("uninitialized renewal workload")?)?;
        Ok(json!({"source_and_bindings_unchanged":true,"budget":self.fixture.budget()?}))
    }
}

fn empty_sample(sequence: usize, label: &str) -> Sample {
    Sample {
        sequence,
        block: 0,
        position: sequence,
        label: label.into(),
        argv: Vec::new(),
        preparation_ns: 0,
        process_ns: 0,
        verification_ns: 0,
        api_ns: None,
        native_ns: None,
        exit_code: None,
        error: None,
        started_unix_ns: 0,
        details: json!({}),
    }
}
