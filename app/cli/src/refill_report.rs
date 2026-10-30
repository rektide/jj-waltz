//! Application-owned schema 2. Projection is pure: never discover or refresh
//! state while reporting an error. Native child facts are last observed facts.
use anyhow::Result;
use jj_waltz_refill_engine::{PrepareRequest, ReseedRequest, command::*};
use serde::{Serialize, Serializer};
use std::{
    fmt::Write,
    os::unix::ffi::OsStrExt,
    path::Path,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy)]
pub struct Decimal(pub u128);
impl Serialize for Decimal {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}
fn ms(d: Duration) -> Decimal {
    Decimal(d.as_millis())
}
#[derive(Serialize)]
pub struct WirePath {
    display: String,
    unix_bytes_hex: String,
}
impl From<&Path> for WirePath {
    fn from(path: &Path) -> Self {
        let mut hex = String::with_capacity(path.as_os_str().as_bytes().len() * 2);
        for b in path.as_os_str().as_bytes() {
            write!(hex, "{b:02x}").expect("string write");
        }
        Self {
            display: path.display().to_string(),
            unix_bytes_hex: hex,
        }
    }
}
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Complete,
    Planned,
    Failed,
    Blocked,
}
#[derive(Serialize)]
pub struct Envelope {
    schema_version: u8,
    command: &'static str,
    mode: &'static str,
    status: Status,
    renewal_route: Option<&'static str>,
    refresh_check: Option<String>,
    postcheck: Option<&'static str>,
    native_profile: Option<Profile>,
    selection: Selection,
    source: Option<Source>,
    desired: Option<Desired>,
    preview: Option<Preview>,
    receivers: Vec<Receiver>,
    capture: Option<Capture>,
    source_diagnostics: Diagnostics,
    timings: Timings,
    error: Option<Error>,
}
#[derive(Serialize)]
struct Profile {
    id: String,
    raw_version: String,
}
#[derive(Serialize)]
struct Selection {
    names: Vec<String>,
    groups: Vec<String>,
    resolved: Vec<Selected>,
    skipped: Vec<Skipped>,
}
#[derive(Serialize)]
struct Selected {
    name: String,
    root: WirePath,
    origins: Vec<String>,
}
#[derive(Serialize)]
struct Skipped {
    candidate: String,
    reason: String,
}
#[derive(Serialize)]
struct Desired {
    kind: &'static str,
    revset: Option<String>,
    commit_id: Option<String>,
    state: &'static str,
}
#[derive(Serialize)]
struct Preview {
    label: &'static str,
    clone_capability_probed: bool,
    preservation_pending: bool,
    obstructions_inspected: bool,
    ordinary_patterns: Option<Vec<String>>,
}
#[derive(Serialize)]
struct Source {
    name: String,
    root: WirePath,
    operation_id: Option<String>,
    recorded_commit_id: Option<String>,
    source_commit_id: Option<String>,
    source_change_id: Option<String>,
    upstream_goal_commit_id: Option<String>,
    state: &'static str,
    evidence: Option<&'static str>,
    assertion: Option<Assertion>,
    upstream: Option<Upstream>,
    preparation: Option<Preparation>,
    elapsed_ms: Option<Decimal>,
}
#[derive(Serialize)]
struct Assertion {
    revset: String,
    commit_id: Option<String>,
    status: &'static str,
}
#[derive(Serialize)]
struct Upstream {
    remote: String,
    bookmark: String,
    prospective_goal_commit_id: Option<String>,
}
#[derive(Serialize)]
struct Preparation {
    status: &'static str,
    saved_commit_id: Option<String>,
    steps: Vec<Step>,
    final_conformance: &'static str,
    elapsed_ms: Option<Decimal>,
}
#[derive(Serialize)]
struct Step {
    index: Decimal,
    argv: Vec<String>,
    status: &'static str,
    elapsed_ms: Option<Decimal>,
}
#[derive(Serialize)]
struct Receiver {
    name: String,
    root: WirePath,
    renewal_route: &'static str,
    status: &'static str,
    phase: &'static str,
    saved: Option<Saved>,
    desired_commit_id: Option<String>,
    child: Option<Child>,
    native: Native,
    ordinary: Ordinary,
    finish: Finish,
}
#[derive(Serialize)]
struct Saved {
    commit_id: String,
    history: History,
}
#[derive(Serialize)]
struct History {
    status: &'static str,
    work_tip_commit_id: Option<String>,
    bookmark: Option<String>,
    remote: Option<String>,
    bookmark_target_commit_id: Option<String>,
}
#[derive(Serialize)]
struct Child {
    commit_id: String,
    change_id: String,
    parent_commit_ids: Vec<String>,
}
#[derive(Serialize)]
struct Native {
    state: &'static str,
    manifest: Option<Manifest>,
    renewal_effects: Option<Effects>,
    retained_permissions_set: Option<Decimal>,
    explicit_clear_effects: Option<Effects>,
    source_state_adopted: Option<bool>,
    steps: Vec<NativeStep>,
    elapsed_ms: Option<Decimal>,
    tracked_install_ms: Option<Decimal>,
}
#[derive(Serialize)]
struct Manifest {
    desired: Decimal,
    installed: Decimal,
    retained: Decimal,
    owned_obsolete: Decimal,
    owned_retired: Decimal,
    complete: bool,
}
#[derive(Serialize)]
struct NativeStep {
    phase: &'static str,
    status: &'static str,
    elapsed_ms: Option<Decimal>,
}
#[derive(Serialize)]
struct Ordinary {
    state: &'static str,
    report: Option<OrdinaryReport>,
    application_ms: Option<Decimal>,
}
#[derive(Serialize)]
struct OrdinaryReport {
    satisfied: bool,
    deviated: Decimal,
    retained: Decimal,
    applications: Decimal,
    planned: Decimal,
    completed: Effects,
    opaque_deviated: Decimal,
    additional_retained: Decimal,
    excluded: Decimal,
    end: Option<&'static str>,
    pruned: bool,
    observed: Observed,
}
#[derive(Serialize)]
struct Observed {
    state: &'static str,
    scoped: bool,
    comparator: Option<String>,
    excluded: Decimal,
    planned: Decimal,
    deviated: Decimal,
    retained: Decimal,
    equal: Decimal,
    different: Decimal,
    not_assessed: Decimal,
    unsupported: Decimal,
    unresolved: Decimal,
    stopped_paired: Decimal,
    examined_directories: Decimal,
}
#[derive(Serialize)]
struct Finish {
    state: &'static str,
    links: Option<Links>,
    failed_path: Option<WirePath>,
    elapsed_ms: Option<Decimal>,
}
#[derive(Serialize)]
struct Links {
    linked: Decimal,
    directories_created: Decimal,
    satisfied: Decimal,
    skipped_missing_target: Decimal,
}
#[derive(Serialize)]
struct Capture {
    distinct_trees: Decimal,
    membership_queries: Decimal,
    indexes: Decimal,
    donor_cache_reads: Decimal,
    desired_manifests: Decimal,
    owned_manifests: Decimal,
}
#[derive(Serialize)]
struct Diagnostics {
    native: Option<SourceDiagnostics>,
    ordinary: Option<SourceDiagnostics>,
}
#[derive(Serialize)]
struct SourceDiagnostics {
    paired: SourceWork,
    opaque: SourceWork,
    declared: SourceWork,
    retained_batches: Decimal,
    retained_charged_capacity: Decimal,
    active_batches: Decimal,
    evictions: Decimal,
}
#[derive(Serialize)]
struct SourceWork {
    listings: Decimal,
    metadata_acquisitions: Decimal,
    link_acquisitions: Decimal,
    fact_hits: Decimal,
    cache_hits: Decimal,
    cache_misses: Decimal,
}
#[derive(Serialize)]
struct Timings {
    command_ms: Option<Decimal>,
    engine_ms: Option<Decimal>,
    pre_native_ms: Option<Decimal>,
    ordinary_ms: Option<Decimal>,
}
#[derive(Serialize)]
struct Error {
    kind: &'static str,
    phase: &'static str,
    receiver: Option<String>,
    path: Option<WirePath>,
    argv_step: Option<Decimal>,
    message: String,
    causes: Vec<String>,
    secondary: Vec<Secondary>,
}
#[derive(Serialize)]
struct Secondary {
    operation: Option<String>,
    path: Option<WirePath>,
    message: String,
    causes: Vec<String>,
}

macro_rules! effects {
    ($($field:ident),* $(,)?) => {
        #[derive(Serialize)]
        struct Effects { $($field: Decimal,)* }
        impl From<&diff_dirs_sync::Progress> for Effects {
            fn from(p: &diff_dirs_sync::Progress) -> Self { Self { $($field: Decimal(p.$field as u128),)* } }
        }
        impl From<&diff_dirs_sync::coordinated::WorkTotals> for Effects {
            fn from(p: &diff_dirs_sync::coordinated::WorkTotals) -> Self { Self { $($field: Decimal(p.$field),)* } }
        }
    }
}
effects!(
    files_copied,
    files_reflinked,
    bytes_copied,
    bytes_reflinked,
    symlinks_created,
    directories_created,
    files_removed,
    symlinks_removed,
    directories_removed,
    directory_permissions_set
);

pub fn route(r: RenewalRoute) -> &'static str {
    match r {
        RenewalRoute::Refresh => "refresh",
        RenewalRoute::ClearThenPopulate => "clear-then-populate",
        RenewalRoute::ReplaceAuthorized => "replace-authorized",
    }
}
fn phase(p: Phase) -> &'static str {
    match p {
        Phase::Resolve => "resolve",
        Phase::Source => "source",
        Phase::Preserve => "preserve",
        Phase::Capture => "capture",
        Phase::NativeClear => "native_clear",
        Phase::NativeChild => "native_child",
        Phase::ExplicitClear => "explicit_clear",
        Phase::Install => "install",
        Phase::AdmitDesired => "admit_desired",
        Phase::Adopt => "adopt",
        Phase::Restore => "restore",
        Phase::Reparent => "reparent",
        Phase::NativeVerify => "native_verify",
        Phase::Ordinary => "ordinary",
        Phase::Links => "links",
        Phase::Finish => "finish",
        Phase::Complete => "complete",
    }
}
impl Envelope {
    fn new(command: &'static str, preview: bool, request: Option<&ReseedRequest>) -> Self {
        Self {
            schema_version: 2,
            command,
            mode: if preview { "preview" } else { "execute" },
            status: Status::Failed,
            renewal_route: request.map(|r| route(r.renewal_route)),
            refresh_check: request.map(|r| r.refresh_check.to_string()),
            postcheck: request.map(|r| match r.postcheck {
                jj_waltz_refill_engine::Postcheck::None => "none",
                jj_waltz_refill_engine::Postcheck::Mtime => "mtime",
            }),
            native_profile: None,
            selection: Selection {
                names: request.map_or_else(Vec::new, |r| r.names.clone()),
                groups: request.map_or_else(Vec::new, |r| r.groups.clone()),
                resolved: vec![],
                skipped: vec![],
            },
            source: None,
            desired: request.map(|r| Desired {
                kind: if r.at.is_some() { "revset" } else { "source" },
                revset: r.at.clone(),
                commit_id: None,
                state: "unresolved",
            }),
            preview: None,
            receivers: vec![],
            capture: None,
            source_diagnostics: Diagnostics { native: None, ordinary: None },
            timings: Timings {
                command_ms: None,
                engine_ms: None,
                pre_native_ms: None,
                ordinary_ms: None,
            },
            error: None,
        }
    }
    fn facts(&mut self, facts: &ResolutionFacts, preparing: bool) {
        self.native_profile = facts.profile.as_ref().map(|p| Profile {
            id: p.id.into(),
            raw_version: p.raw_version.clone(),
        });
        self.selection.resolved = facts
            .selections
            .iter()
            .map(|s| Selected {
                name: s.name.clone(),
                root: s.root.as_path().into(),
                origins: s.origins.clone(),
            })
            .collect();
        self.selection.skipped = facts
            .skipped
            .iter()
            .map(|s| Skipped {
                candidate: s.candidate.clone(),
                reason: s.reason.clone(),
            })
            .collect();
        self.source = facts
            .source
            .as_ref()
            .map(|s| Source::context(s, if self.mode == "preview" { "planned" } else { "not_started" }, preparing));
        self.receivers = facts
            .selections
            .iter()
            .map(|s| Receiver::unstarted(&s.name, &s.root, self.renewal_route.expect("reseed route"), self.mode == "preview"))
            .collect();
    }
    fn fail(&mut self, error: &anyhow::Error, kind: &'static str, location: &FailureLocation) {
        self.status = if self.mode == "preview" { Status::Blocked } else { Status::Failed };
        if self.mode == "preview"
            && let Some(row) = self.receivers.iter_mut().find(|r| Some(&r.name) == location.receiver.as_ref())
        {
            row.status = "blocked";
        }
        self.error = Some(Error::project(error, kind, location));
    }
    fn report(&mut self, report: &CommandReport, failed: bool, preparing: bool) {
        let source_failed = failed && report.source_context.elapsed.is_some() && report.source.is_none();
        self.source = Some(Source::context(
            &report.source_context,
            if source_failed {
                "failed"
            } else if report.source.is_some() {
                "ready"
            } else {
                "not_started"
            },
            preparing,
        ));
        let source = self.source.as_mut().unwrap();
        if let Some(ready) = &report.source {
            source.ready(ready);
        }
        if let Some(preparation) = &report.preparation {
            source.preparation = Some(Preparation::report(preparation, source_failed));
            source.upstream_goal_commit_id = preparation.upstream_goal.clone();
        }
        if let Some(desired) = &mut self.desired {
            desired.commit_id = report.desired.clone();
            if desired.commit_id.is_some() {
                desired.state = "resolved";
            }
        }
        self.receivers = report.receivers.iter().map(Receiver::report).collect();
        self.capture = report.capture.as_ref().map(|c| Capture {
            distinct_trees: Decimal(c.distinct_trees as u128),
            membership_queries: Decimal(c.membership_queries as u128),
            indexes: Decimal(c.indexes as u128),
            donor_cache_reads: Decimal(c.donor_cache_reads as u128),
            desired_manifests: Decimal(c.desired_manifests as u128),
            owned_manifests: Decimal(c.owned_manifests as u128),
        });
        self.source_diagnostics = Diagnostics {
            native: report.tracked_source.map(SourceDiagnostics::from),
            ordinary: report.ordinary_source.map(SourceDiagnostics::from),
        };
        self.timings.engine_ms = Some(ms(report.elapsed));
        self.timings.pre_native_ms = Some(ms(report.setup));
        self.timings.ordinary_ms = report.ordinary_elapsed.map(ms);
    }
    fn preview(&mut self, plan: SourcePlan, patterns: Option<Vec<String>>, desired: Option<DesiredPlan>) {
        self.status = Status::Planned;
        let preparing = plan.prospective.is_some();
        self.source = Some(Source::context(&plan.context, "planned", preparing));
        let source = self.source.as_mut().unwrap();
        source.recorded_commit_id = Some(plan.recorded_commit);
        source.evidence = plan.evidence.map(evidence);
        if preparing {
            source.preparation = Some(Preparation::declarations(plan.argv, "planned"));
        }
        if let (Some(desired), Some(state)) = (&mut self.desired, desired) {
            match state {
                DesiredPlan::Recorded(id) => {
                    desired.state = "recorded";
                    desired.commit_id = Some(id.clone());
                    for r in &mut self.receivers {
                        r.desired_commit_id = Some(id.clone());
                    }
                }
                DesiredPlan::AfterLiveSourceAssertion => desired.state = "after_live_assertion",
                DesiredPlan::FuturePreparedSource => desired.state = "future_prepared_source",
                DesiredPlan::PostPreparationRevset(_) => desired.state = "post_preparation_revset",
            }
        }
        self.preview = Some(Preview {
            label: "PLANNED — payload/path obstructions not inspected",
            clone_capability_probed: false,
            preservation_pending: self.command == "reseed",
            obstructions_inspected: false,
            ordinary_patterns: patterns,
        });
    }
    pub fn plain(&self) -> String {
        let mut text = String::new();
        if let Some(preview) = &self.preview {
            writeln!(text, "{}", preview.label).unwrap();
        }
        if self.mode == "preview" {
            for s in &self.selection.resolved {
                writeln!(text, "SELECT {:?} {} via {}", s.name, s.root.display, s.origins.join(", ")).unwrap();
            }
        }
        if let Some(d) = &self.desired {
            writeln!(text, "desired: {} ({})", d.commit_id.as_deref().unwrap_or("unknown"), d.state).unwrap();
        }
        if let Some(s) = &self.source {
            writeln!(
                text,
                "source: {}; {} {} {} S={} goal={}",
                s.root.display,
                s.name,
                s.state,
                s.evidence.unwrap_or("pending"),
                s.source_commit_id.as_deref().unwrap_or("unknown"),
                s.upstream_goal_commit_id.as_deref().unwrap_or("unknown")
            )
            .unwrap();
            if let Some(p) = &s.preparation {
                writeln!(
                    text,
                    "preparation: {}; {} configured commands; {}",
                    p.status,
                    p.steps.len(),
                    if p.steps.is_empty() {
                        "existing data, not freshly built or verified"
                    } else {
                        "trusted sequential argv; one final tracked conformance check"
                    }
                )
                .unwrap();
            }
        }
        if let Some(checks) = &self.refresh_check {
            writeln!(text, "refresh-check={checks} postcheck={}", self.postcheck.expect("reseed postcheck")).unwrap();
        }
        for r in &self.receivers {
            let count = |n: Option<Decimal>| n.map(|n| n.0.to_string()).unwrap_or_else(|| "unknown".into());
            write!(
                text,
                "{} {}: route={} phase={} renewal={} retained={} retained-chmod={} ordinary={} links={}; root={} O={} history={}",
                r.status,
                r.name,
                r.renewal_route,
                r.phase,
                count(r.native.renewal_effects.as_ref().map(|e| e.files_reflinked)),
                count(r.native.manifest.as_ref().map(|m| m.retained)),
                count(r.native.retained_permissions_set),
                count(r.ordinary.report.as_ref().map(|o| o.completed.files_reflinked)),
                count(r.finish.links.as_ref().map(|l| l.linked)),
                r.root.display,
                r.saved.as_ref().map(|s| s.commit_id.as_str()).unwrap_or("unknown"),
                r.saved.as_ref().map(|s| s.history.status).unwrap_or("unknown")
            )
            .unwrap();
            if let Some(saved) = &r.saved
                && let Some(bookmark) = &saved.history.bookmark
            {
                write!(
                    text,
                    " bookmark={bookmark:?} remote={:?} work_tip={} target={}",
                    saved.history.remote,
                    saved.history.work_tip_commit_id.as_deref().unwrap_or("unknown"),
                    saved.history.bookmark_target_commit_id.as_deref().unwrap_or("unknown")
                )
                .unwrap();
            }
            text.push('\n');
        }
        if let Some(error) = &self.error {
            writeln!(
                text,
                "{} at {}: {}; partial effects may remain; no rollback or automatic recovery",
                error.kind, error.phase, error.message
            )
            .unwrap();
        }
        text
    }
}
fn evidence(e: SourceEvidence) -> &'static str {
    match e {
        SourceEvidence::AssumedPrepared => "assumed_prepared",
        SourceEvidence::LiveAssertion => "live_assertion",
        SourceEvidence::Produced => "produced",
    }
}
impl Source {
    fn context(c: &SourceContext, state: &'static str, preparing: bool) -> Self {
        Self {
            name: c.name.clone(),
            root: c.root.as_path().into(),
            operation_id: Some(c.operation.clone()),
            recorded_commit_id: Some(c.recorded_commit.clone()),
            source_commit_id: None,
            source_change_id: None,
            upstream_goal_commit_id: None,
            state,
            evidence: None,
            assertion: c.assertion.as_ref().map(|a| Assertion {
                revset: a.revset.clone(),
                commit_id: a.commit.clone(),
                status: match a.status {
                    AssertionStatus::Pending => "pending",
                    AssertionStatus::Succeeded => "succeeded",
                    AssertionStatus::Failed => "failed",
                },
            }),
            upstream: c.upstream.as_ref().map(|u| Upstream {
                remote: u.remote.clone(),
                bookmark: u.name.clone(),
                prospective_goal_commit_id: u.old_goal.clone(),
            }),
            preparation: if preparing {
                c.preparation_argv
                    .clone()
                    .map(|argv| Preparation::declarations(argv, if state == "planned" { "planned" } else { "not_started" }))
            } else {
                None
            },
            elapsed_ms: c.elapsed.map(ms),
        }
    }
    fn ready(&mut self, ready: &SourceReport) {
        self.operation_id = Some(ready.operation.clone());
        self.source_commit_id = Some(ready.commit.clone());
        self.source_change_id = Some(ready.change.clone());
        self.upstream_goal_commit_id = ready.upstream_goal.clone();
        self.evidence = Some(evidence(ready.evidence));
        if let Some(p) = &ready.preparation {
            self.preparation = Some(Preparation::report(p, false));
        }
    }
}
impl Preparation {
    fn declarations(argv: Vec<Vec<String>>, status: &'static str) -> Self {
        Self {
            status,
            saved_commit_id: None,
            steps: argv
                .into_iter()
                .enumerate()
                .map(|(i, argv)| Step {
                    index: Decimal(i as u128),
                    argv,
                    status: "selected",
                    elapsed_ms: None,
                })
                .collect(),
            final_conformance: "pending",
            elapsed_ms: None,
        }
    }
    fn report(p: &PreparationReport, failed: bool) -> Self {
        Self {
            status: if failed { "failed" } else { "succeeded" },
            saved_commit_id: p.saved_commit.clone(),
            steps: p
                .steps
                .iter()
                .enumerate()
                .map(|(i, s)| Step {
                    index: Decimal(i as u128),
                    argv: s.argv.clone(),
                    status: match s.status {
                        StepStatus::Selected => "selected",
                        StepStatus::Started => "started",
                        StepStatus::Succeeded => "succeeded",
                    },
                    elapsed_ms: s.elapsed.map(ms),
                })
                .collect(),
            final_conformance: if p.final_conformance_succeeded {
                "succeeded"
            } else if p.final_conformance_started {
                "failed"
            } else {
                "pending"
            },
            elapsed_ms: Some(Decimal(p.elapsed_ms)),
        }
    }
}
impl Receiver {
    fn unstarted(name: &str, root: &Path, route: &'static str, preview: bool) -> Self {
        Self {
            name: name.into(),
            root: root.into(),
            renewal_route: route,
            status: if preview { "planned" } else { "not_started" },
            phase: "resolve",
            saved: None,
            desired_commit_id: None,
            child: None,
            native: Native {
                state: "unstarted",
                manifest: None,
                renewal_effects: None,
                retained_permissions_set: None,
                explicit_clear_effects: None,
                source_state_adopted: None,
                steps: vec![],
                elapsed_ms: None,
                tracked_install_ms: None,
            },
            ordinary: Ordinary {
                state: "unstarted",
                report: None,
                application_ms: None,
            },
            finish: Finish {
                state: "unstarted",
                links: None,
                failed_path: None,
                elapsed_ms: None,
            },
        }
    }
    fn report(r: &ReceiverReport) -> Self {
        let mut row = Self::unstarted(&r.name, &r.root, route(r.route), false);
        row.status = match r.status {
            ReceiverStatus::NotStarted => "not_started",
            ReceiverStatus::Partial => "partial",
            ReceiverStatus::Complete => "complete",
            ReceiverStatus::Failed => "failed",
        };
        row.phase = phase(r.reached);
        row.desired_commit_id = r.desired.clone();
        row.saved = r.old.as_ref().map(|s| Saved {
            commit_id: s.commit.clone(),
            history: History::from(&s.history),
        });
        row.child = r.child.as_ref().map(|c| Child {
            commit_id: c.commit_id.clone(),
            change_id: c.change_id.clone(),
            parent_commit_ids: c.parents.clone(),
        });
        row.native = Native {
            state: match r.native.state {
                Establishment::Unstarted => "unstarted",
                Establishment::InProgress => "in_progress",
                Establishment::Established => "established",
            },
            manifest: r.native.manifest.map(|m| Manifest {
                desired: Decimal(m.desired as u128),
                installed: Decimal(m.installed as u128),
                retained: Decimal(m.retained as u128),
                owned_obsolete: Decimal(m.owned_obsolete as u128),
                owned_retired: Decimal(m.owned_retired as u128),
                complete: m.complete,
            }),
            renewal_effects: r.native.completed.as_ref().map(Effects::from),
            retained_permissions_set: r.native.retained_permissions_set.map(|n| Decimal(n as u128)),
            explicit_clear_effects: r.native.explicit_clear.as_ref().map(Effects::from),
            source_state_adopted: r.native.source_state_adopted,
            steps: r
                .native
                .timings
                .iter()
                .map(|s| NativeStep {
                    phase: phase(s.phase),
                    status: if s.succeeded { "succeeded" } else { "failed" },
                    elapsed_ms: Some(ms(s.elapsed)),
                })
                .collect(),
            elapsed_ms: r.native.elapsed.map(ms),
            tracked_install_ms: r.native.tracked_install_elapsed().map(ms),
        };
        row.ordinary = Ordinary {
            state: match r.ordinary.state {
                OrdinaryState::Disabled => "disabled",
                OrdinaryState::Unstarted => "unstarted",
                OrdinaryState::Partial => "partial",
                OrdinaryState::Satisfied => "satisfied",
            },
            report: r.ordinary.report.as_ref().map(OrdinaryReport::from),
            application_ms: None,
        };
        row.finish = Finish {
            state: if r.finish.complete {
                "complete"
            } else if r.finish.elapsed.is_some() {
                "partial"
            } else {
                "unstarted"
            },
            links: r.finish.links.as_ref().map(|l| Links {
                linked: Decimal(l.linked as u128),
                directories_created: Decimal(l.directories_created as u128),
                satisfied: Decimal(l.satisfied as u128),
                skipped_missing_target: Decimal(l.skipped_missing_target as u128),
            }),
            failed_path: r.finish.failed_path.as_deref().map(WirePath::from),
            elapsed_ms: r.finish.elapsed.map(ms),
        };
        row
    }
}
impl From<&HistoryCoverage> for History {
    fn from(h: &HistoryCoverage) -> Self {
        let mut out = Self {
            status: "pending",
            work_tip_commit_id: None,
            bookmark: None,
            remote: None,
            bookmark_target_commit_id: None,
        };
        match h {
            HistoryCoverage::Pending => {}
            HistoryCoverage::Bypassed => out.status = "bypassed",
            HistoryCoverage::Covered {
                work_tip,
                name,
                remote,
                target,
            } => {
                out.status = "covered";
                out.work_tip_commit_id = Some(work_tip.clone());
                out.bookmark = Some(name.clone());
                out.remote = remote.clone();
                out.bookmark_target_commit_id = Some(target.clone());
            }
        }
        out
    }
}
impl From<&diff_dirs_sync::coordinated::LaneReport> for OrdinaryReport {
    fn from(r: &diff_dirs_sync::coordinated::LaneReport) -> Self {
        use diff_dirs_sync::coordinated::End;
        let t = &r.totals;
        Self {
            satisfied: r.satisfied,
            deviated: Decimal(r.deviated),
            retained: Decimal(r.retained),
            applications: Decimal(t.applications as u128),
            planned: Decimal(t.planned),
            completed: (&t.completed).into(),
            opaque_deviated: Decimal(t.opaque_deviated),
            additional_retained: Decimal(t.additional_retained),
            excluded: Decimal(t.excluded),
            end: t.end.map(|e| match e {
                End::ComparisonFailed => "comparison_failed",
                End::ApplicationFailed => "application_failed",
                End::RefreshFailed => "refresh_failed",
                End::DecisionChanged => "decision_changed",
                End::Abandoned => "abandoned",
            }),
            pruned: t.pruned,
            observed: (&r.observed).into(),
        }
    }
}
impl From<&diff_dirs::Summary> for Observed {
    fn from(o: &diff_dirs::Summary) -> Self {
        Self {
            state: match o.state {
                diff_dirs::ScanState::Incomplete => "incomplete",
                diff_dirs::ScanState::Complete => "complete",
                diff_dirs::ScanState::Failed => "failed",
            },
            scoped: o.scoped,
            comparator: o.comparator.as_ref().map(|id| {
                let mut text = format!("{}@{}:", id.family, id.version);
                for byte in &id.parameters {
                    write!(text, "{byte:02x}").unwrap();
                }
                text
            }),
            excluded: Decimal(o.excluded as u128),
            planned: Decimal(o.planned as u128),
            deviated: Decimal(o.deviated as u128),
            retained: Decimal(o.retained as u128),
            equal: Decimal(o.equal as u128),
            different: Decimal(o.different as u128),
            not_assessed: Decimal(o.not_assessed as u128),
            unsupported: Decimal(o.unsupported as u128),
            unresolved: Decimal(o.unresolved as u128),
            stopped_paired: Decimal(o.stopped_paired as u128),
            examined_directories: Decimal(o.examined_directories as u128),
        }
    }
}
impl From<diff_dirs_inspect::SourceDiagnostics> for SourceDiagnostics {
    fn from(d: diff_dirs_inspect::SourceDiagnostics) -> Self {
        Self {
            paired: d.paired.into(),
            opaque: d.opaque.into(),
            declared: d.declared.into(),
            retained_batches: Decimal(d.retained_batches as u128),
            retained_charged_capacity: Decimal(d.retained_charged_capacity as u128),
            active_batches: Decimal(d.active_batches as u128),
            evictions: Decimal(d.evictions as u128),
        }
    }
}
impl From<diff_dirs_inspect::SourceWorkDiagnostics> for SourceWork {
    fn from(d: diff_dirs_inspect::SourceWorkDiagnostics) -> Self {
        Self {
            listings: Decimal(d.listings as u128),
            metadata_acquisitions: Decimal(d.metadata_acquisitions as u128),
            link_acquisitions: Decimal(d.link_acquisitions as u128),
            fact_hits: Decimal(d.fact_hits as u128),
            cache_hits: Decimal(d.cache_hits as u128),
            cache_misses: Decimal(d.cache_misses as u128),
        }
    }
}
impl Error {
    fn project(error: &anyhow::Error, kind: &'static str, location: &FailureLocation) -> Self {
        let mut out = Self {
            kind,
            phase: phase(location.phase),
            receiver: location.receiver.clone(),
            path: error.downcast_ref::<ErrorPath>().map(|p| p.0.as_path().into()),
            argv_step: None,
            message: error.to_string(),
            causes: error.chain().skip(1).map(ToString::to_string).collect(),
            secondary: vec![],
        };
        if let Some(p) = error.downcast_ref::<PreparationFailure>() {
            out.argv_step = p.completed.failed_step.map(|i| Decimal(i as u128));
        }
        for cause in error.chain() {
            if let Some(f) = cause.downcast_ref::<diff_dirs_sync::Failure>() {
                out.path = Some(f.primary.path.as_path().into());
                out.secondary.extend(f.secondary.iter().map(|i| Secondary {
                    operation: Some(i.operation.into()),
                    path: Some(i.path.as_path().into()),
                    message: i.cause.to_string(),
                    causes: chain(i.cause.source()),
                }));
                break;
            }
            if let Some(i) = cause.downcast_ref::<diff_dirs_inspect::Error>() {
                out.path = Some(i.path().into());
            }
            if let Some(l) = cause.downcast_ref::<jj_waltz_workspace_state::links::LinkFinishFailure>() {
                out.path = l.path.as_deref().map(WirePath::from);
            }
        }
        out
    }
}
fn chain(mut error: Option<&(dyn std::error::Error + 'static)>) -> Vec<String> {
    let mut causes = vec![];
    while let Some(e) = error {
        causes.push(e.to_string());
        error = e.source();
    }
    causes
}

/// The error is retained, not replaced by the serialized projection. CLI writes
/// the envelope once, then returns this original cause to its stderr/exit boundary.
pub struct Outcome {
    pub envelope: Envelope,
    pub error: Option<anyhow::Error>,
}
pub fn reseed(request: ReseedRequest) -> Outcome {
    let started = Instant::now();
    let mut e = Envelope::new("reseed", request.dry_run, Some(&request));
    let preparing = request.prepare;
    let mut diagnostic = |s: &str| eprintln!("{s}");
    let result = (|| -> Result<()> {
        let cwd = std::env::current_dir()?;
        let command = match ResolvedCommand::resolve(&cwd, request, &mut diagnostic) {
            Ok(c) => c,
            Err(f) => {
                e.facts(&f.facts, preparing);
                e.fail(&f.source, "resolution", &f.location);
                return Err(f.into());
            }
        };
        e.facts(&command.facts, preparing);
        if e.mode == "preview" {
            let plan = command.preview()?;
            e.preview(plan.source, Some(plan.patterns), Some(plan.desired));
        } else {
            match command.execute(&mut ContextualOrdinary::default(), &mut diagnostic) {
                Ok(r) => {
                    e.report(&r, false, preparing);
                    e.status = Status::Complete;
                }
                Err(f) => {
                    e.report(&f.report, true, preparing);
                    e.fail(
                        &f.source,
                        if f.source.downcast_ref::<PreparationFailure>().is_some() {
                            "preparation"
                        } else {
                            "execution"
                        },
                        &f.location,
                    );
                    return Err(f.into());
                }
            }
        }
        Ok(())
    })();
    if let Err(error) = &result
        && e.error.is_none()
    {
        e.fail(
            error,
            "resolution",
            &FailureLocation {
                phase: Phase::Resolve,
                receiver: None,
            },
        );
    }
    e.timings.command_ms = Some(ms(started.elapsed()));
    Outcome {
        envelope: e,
        error: result.err(),
    }
}
pub fn prepare(request: PrepareRequest) -> Outcome {
    let started = Instant::now();
    let mut e = Envelope::new("prepare", request.dry_run, None);
    let result = (|| -> Result<()> {
        let cwd = std::env::current_dir()?;
        let source = match ResolvedSource::resolve(&cwd, request) {
            Ok(s) => s,
            Err(f) => {
                e.facts(&f.facts, true);
                e.fail(&f.source, "resolution", &f.location);
                return Err(f.into());
            }
        };
        e.facts(&source.facts, true);
        if e.mode == "preview" {
            e.preview(source.preview()?, None, None);
        } else {
            match prepare_source(&source, &mut |s| eprintln!("{s}")) {
                Ok(s) => {
                    let mut projected = Source::context(&s.report.context, "ready", true);
                    projected.ready(&s.report);
                    e.source = Some(projected);
                    e.status = Status::Complete;
                }
                Err(f) => {
                    let mut projected = Source::context(&f.context, "failed", true);
                    projected.preparation = Some(Preparation::report(&f.completed, true));
                    projected.upstream_goal_commit_id = f.completed.upstream_goal.clone();
                    e.source = Some(projected);
                    let error = anyhow::Error::new(f);
                    e.fail(
                        &error,
                        "preparation",
                        &FailureLocation {
                            phase: Phase::Source,
                            receiver: None,
                        },
                    );
                    return Err(error);
                }
            }
        }
        Ok(())
    })();
    if let Err(error) = &result
        && e.error.is_none()
    {
        e.fail(
            error,
            "resolution",
            &FailureLocation {
                phase: Phase::Resolve,
                receiver: None,
            },
        );
    }
    e.timings.command_ms = Some(ms(started.elapsed()));
    Outcome {
        envelope: e,
        error: result.err(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    #[test]
    fn plain_row_identifies_saved_history_without_dumping_reports() {
        let mut envelope = Envelope::new("reseed", false, Some(&ReseedRequest::default()));
        let mut row = Receiver::unstarted("worker", Path::new("/receiver"), "clear-then-populate", false);
        row.saved = Some(Saved {
            commit_id: "saved-O".into(),
            history: History::from(&HistoryCoverage::Covered {
                work_tip: "work-tip".into(),
                name: "saved-work".into(),
                remote: None,
                target: "bookmark-target".into(),
            }),
        });
        envelope.receivers.push(row);
        let plain = envelope.plain();
        assert!(plain.contains("root=/receiver O=saved-O history=covered bookmark=\"saved-work\""));
        assert!(plain.contains("work_tip=work-tip target=bookmark-target"));
        assert_eq!(plain.lines().count(), 3);
        assert!(!plain.contains("NativeOutcome"));
    }
    #[test]
    fn schema_and_explicit_fixtures_accept_nulls_and_reject_numeric_noncanonical_and_overflow_counts() {
        let schema: Value = serde_json::from_str(include_str!("../../../docs/schema/refill-v1.schema.json")).unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        for fixture in [
            include_str!("../tests/fixtures/refill/resolution-failed.json"),
            include_str!("../tests/fixtures/refill/prepare-planned.json"),
            include_str!("../tests/fixtures/refill/prepare-complete.json"),
            include_str!("../tests/fixtures/refill/reseed-complete.json"),
        ] {
            let value: Value = serde_json::from_str(fixture).unwrap();
            assert!(
                validator.is_valid(&value),
                "{:?}",
                validator.iter_errors(&value).map(|e| e.to_string()).collect::<Vec<_>>()
            );
        }
        let complete: Value = serde_json::from_str(include_str!("../tests/fixtures/refill/reseed-complete.json")).unwrap();
        for field in ["renewal_effects", "manifest"] {
            let mut invalid = complete.clone();
            invalid["receivers"][0]["native"][field] = Value::Null;
            assert!(!validator.is_valid(&invalid));
            invalid["receivers"][0]["native"].as_object_mut().unwrap().remove(field);
            assert!(!validator.is_valid(&invalid));
        }
        let mut incomplete = complete;
        incomplete["receivers"][0]["native"]["manifest"]["complete"] = json!(false);
        assert!(!validator.is_valid(&incomplete));
        let counts =
            jsonschema::validator_for(&json!({"$schema": "https://json-schema.org/draft/2020-12/schema", "$defs": schema["$defs"], "$ref": "#/$defs/N"}))
                .unwrap();
        for good in ["0", "1", "9007199254740993", "18446744073709551616", "340282366920938463463374607431768211455"] {
            assert!(counts.is_valid(&json!(good)));
        }
        for bad in [
            json!(0),
            json!(null),
            json!("01"),
            json!("-1"),
            json!("340282366920938463463374607431768211456"),
            json!("999999999999999999999999999999999999999"),
        ] {
            assert!(!counts.is_valid(&bad), "{bad}");
        }
        let ordinary: Value = serde_json::from_str(include_str!("../tests/fixtures/refill/ordinary-wide.json")).unwrap();
        let lane = jsonschema::validator_for(
            &json!({"$schema": "https://json-schema.org/draft/2020-12/schema", "$defs": schema["$defs"], "$ref": "#/$defs/ordinaryReport"}),
        )
        .unwrap();
        assert!(lane.is_valid(&ordinary));
        let mut missing = ordinary.clone();
        missing.as_object_mut().unwrap().remove("completed");
        assert!(!lane.is_valid(&missing));
        let mut null = ordinary;
        null["completed"] = Value::Null;
        assert!(!lane.is_valid(&null));
    }
    #[test]
    fn wide_counts_are_decimal_and_retention_is_not_deviation() {
        let mut totals = diff_dirs_sync::coordinated::LaneTotals {
            applications: 1,
            planned: u128::MAX,
            end: Some(diff_dirs_sync::coordinated::End::ApplicationFailed),
            ..Default::default()
        };
        totals.completed.files_reflinked = u64::MAX as u128 + 1;
        totals.completed.bytes_reflinked = u128::MAX;
        let report = diff_dirs_sync::coordinated::LaneReport {
            target: diff_dirs::coordinated::TargetId("target".into()),
            totals,
            observed: Default::default(),
            deviated: u64::MAX as u128 + 2,
            retained: 9_007_199_254_740_993,
            satisfied: false,
        };
        let actual = serde_json::to_value(OrdinaryReport::from(&report)).unwrap();
        let expected: Value = serde_json::from_str(include_str!("../tests/fixtures/refill/ordinary-wide.json")).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(serde_json::to_value(Decimal(u128::MAX)).unwrap(), json!(u128::MAX.to_string()));
    }

    #[test]
    fn schema_two_requires_options_retained_coverage_and_separate_actual_metadata_availability() {
        let schema: Value = serde_json::from_str(include_str!("../../../docs/schema/refill-v2.schema.json")).unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        let mut value: Value = serde_json::from_str(include_str!("../tests/fixtures/refill/reseed-complete.json")).unwrap();
        value["schema_version"] = json!(2);
        value["renewal_route"] = json!("refresh");
        value["refresh_check"] = json!("mtime,size");
        value["postcheck"] = json!("none");
        value["receivers"][0]["renewal_route"] = json!("refresh");
        value["receivers"][0]["native"]["manifest"]["retained"] = json!("0");
        value["receivers"][0]["native"]["retained_permissions_set"] = json!("0");
        assert!(validator.is_valid(&value));
        for pointer in [
            "/refresh_check",
            "/postcheck",
            "/receivers/0/native/manifest/retained",
            "/receivers/0/native/retained_permissions_set",
        ] {
            let mut invalid = value.clone();
            *invalid.pointer_mut(pointer).unwrap() = Value::Null;
            assert!(!validator.is_valid(&invalid), "accepted unavailable complete field {pointer}");
        }
        for invalid in ["size,size", "size,mtime", "", "hash", "mtime,"] {
            value["refresh_check"] = json!(invalid);
            assert!(!validator.is_valid(&value));
        }
        for fixture in [
            include_str!("../tests/fixtures/refill/prepare-planned.json"),
            include_str!("../tests/fixtures/refill/prepare-complete.json"),
        ] {
            let mut prepare: Value = serde_json::from_str(fixture).unwrap();
            prepare["schema_version"] = json!(2);
            prepare["refresh_check"] = Value::Null;
            prepare["postcheck"] = Value::Null;
            assert!(validator.is_valid(&prepare));
        }
    }
    #[test]
    fn provider_primary_secondary_and_original_io_causes_survive_projection() {
        use std::os::unix::ffi::OsStringExt;
        let path = std::path::PathBuf::from(std::ffi::OsString::from_vec(b"/root/bad-\xff".to_vec()));
        let issue = |operation| diff_dirs_sync::Issue {
            operation,
            path: path.clone(),
            cause: Box::new(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "typed sentinel")),
        };
        let failure = diff_dirs_sync::Failure {
            planned: 1,
            completed: Default::default(),
            policy: Box::default(),
            primary: Box::new(issue("install")),
            secondary: Box::new(vec![issue("cleanup")]),
            observed: None,
        };
        let error = anyhow::Error::new(failure).context("native boundary");
        let out = Error::project(
            &error,
            "execution",
            &FailureLocation {
                phase: Phase::Install,
                receiver: Some("target".into()),
            },
        );
        let wire = serde_json::to_value(out).unwrap();
        assert_eq!(wire["path"]["unix_bytes_hex"], "2f726f6f742f6261642dff");
        assert_eq!(wire["secondary"][0]["operation"], "cleanup");
        assert_eq!(
            error.downcast_ref::<diff_dirs_sync::Failure>().unwrap().secondary[0]
                .cause
                .downcast_ref::<std::io::Error>()
                .unwrap()
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert!(error.chain().any(|c| c.downcast_ref::<std::io::Error>().is_some()));
    }
    #[test]
    fn early_failure_fixture_has_explicit_unavailable_fields() {
        let mut e = Envelope::new(
            "reseed",
            false,
            Some(&ReseedRequest {
                names: vec!["worker-a".into()],
                ..Default::default()
            }),
        );
        let error = anyhow::Error::new(std::io::Error::from(std::io::ErrorKind::NotFound)).context("cannot resolve configured source");
        e.fail(
            &error,
            "resolution",
            &FailureLocation {
                phase: Phase::Resolve,
                receiver: None,
            },
        );
        let mut actual = serde_json::to_value(e).unwrap();
        let mut expected: Value = serde_json::from_str(include_str!("../tests/fixtures/refill/resolution-failed.json")).unwrap();
        expected["schema_version"] = json!(2);
        expected["renewal_route"] = json!("refresh");
        expected["refresh_check"] = json!("mtime,size");
        expected["postcheck"] = json!("none");
        actual["timings"]["command_ms"] = json!("2");
        actual["error"]["causes"] = expected["error"]["causes"].clone();
        assert_eq!(actual, expected);
    }
}
