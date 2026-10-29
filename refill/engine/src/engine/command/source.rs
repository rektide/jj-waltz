use super::*;
use crate::freshen::source::SourceSelection;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceEvidence {
    AssumedPrepared,
    LiveAssertion,
    Produced,
}
#[derive(Debug, Clone)]
pub struct SourceContext {
    pub name: String,
    pub root: PathBuf,
    pub operation: String,
    pub recorded_commit: String,
    pub assertion: Option<SourceAssertion>,
    pub upstream: Option<SourceSelection>,
    pub elapsed: Option<std::time::Duration>,
    pub preparation_argv: Option<Vec<Vec<String>>>,
}
#[derive(Debug, Clone)]
pub struct SourceAssertion {
    pub revset: String,
    pub commit: Option<String>,
    pub status: AssertionStatus,
}
#[derive(Debug, Clone, Copy)]
pub enum AssertionStatus {
    Pending,
    Succeeded,
    Failed,
}
#[derive(Debug, Clone)]
pub struct SourceReport {
    pub context: SourceContext,
    pub profile: NativeProfile,
    pub root: PathBuf,
    pub operation: String,
    /// Actual resulting donor WC S, never the upstream goal in its place.
    pub commit: String,
    pub change: String,
    pub upstream_goal: Option<String>,
    pub evidence: SourceEvidence,
    pub preparation: Option<PreparationReport>,
}
#[derive(Debug)]
pub struct PreparedSource {
    pub binding: Arc<BoundWorkspace>,
    pub report: SourceReport,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Selected,
    Started,
    Succeeded,
}
#[derive(Debug, Clone)]
pub struct PreparationStep {
    pub argv: Vec<String>,
    pub status: StepStatus,
    pub elapsed: Option<std::time::Duration>,
}
#[derive(Debug, Clone, Default)]
pub struct PreparationReport {
    pub elapsed_ms: u128,
    pub saved_commit: Option<String>,
    pub upstream_goal: Option<String>,
    pub source_commit: Option<String>,
    pub steps: Vec<PreparationStep>,
    pub final_conformance_started: bool,
    pub final_conformance_succeeded: bool,
    pub failed_step: Option<usize>,
}
#[derive(Debug)]
pub struct PreparationFailure {
    pub context: Box<SourceContext>,
    pub completed: Box<PreparationReport>,
    pub source: anyhow::Error,
}
impl std::fmt::Display for PreparationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "source preparation stopped; progress remains, no rollback: {}", self.source)
    }
}
impl std::error::Error for PreparationFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// Resolves only source/primary registrations and declarations. No receiver
/// inventory, links, ordinary visitors, payload selection or filesystem census.
pub struct ResolvedSource {
    pub binding: Arc<BoundWorkspace>,
    pub context: SourceContext,
    pub facts: ResolutionFacts,
    pub assertion: Option<String>,
    settings: Option<prepare::Settings>,
    preview_only: bool,
}
impl ResolvedSource {
    pub fn resolve(cwd: &Path, request: crate::PrepareRequest) -> std::result::Result<Self, ResolutionFailure> {
        let request = ReseedRequest {
            source: request.source,
            source_at: request.source_at,
            prepare: true,
            dry_run: request.dry_run,
            ..Default::default()
        };
        let mut facts = ResolutionFacts::default();
        let result = (|| {
            let resolution = Resolution::capture(cwd, &request, true, &mut facts)?;
            if resolution.config.legacy_prepare {
                eprintln!("warning: [prepare] is a deprecated local alias for [freshen]; rename it to [freshen]");
            }
            let registrations = jj_waltz_workspace_state::ownership::refill_registrations_at(&resolution.caller, &resolution.registry.operation)?;
            resolution.check_ownership(&resolution.source, &registrations)?;
            Topology::capture()?.workspace(&resolution.source.root, true)?;
            Ok(Self::from_resolution(&resolution, &request, facts.source.clone().expect("captured source")))
        })();
        result.map_err(|source| ResolutionFailure {
            facts: Box::new(facts),
            location: FailureLocation {
                phase: Phase::Resolve,
                receiver: None,
            },
            source,
        })
    }
    pub(super) fn from_resolution(r: &Resolution, request: &ReseedRequest, context: SourceContext) -> Self {
        Self {
            facts: ResolutionFacts {
                profile: Some(r.source.repository.profile.clone()),
                source: Some(context.clone()),
                ..Default::default()
            },
            context,
            binding: Arc::clone(&r.source),
            assertion: request.source_at.clone().or_else(|| r.config.source_at.clone()),
            settings: r.config.freshen.clone(),
            preview_only: request.dry_run,
        }
    }

    pub fn preview(&self) -> Result<SourcePlan> {
        let client = self.binding.client();
        let operation = client.operation_id()?;
        self.binding.verify_at(&operation)?;
        let selection = SourceSelection::select(
            &client,
            &operation,
            self.assertion.as_deref(),
            self.settings.as_ref().and_then(|s| s.remote.as_deref()),
        )?;
        Ok(SourcePlan {
            context: SourceContext {
                upstream: Some(selection.clone()),
                ..self.context.clone()
            },
            recorded_commit: selection.source.commit_id.clone(),
            prospective: Some(selection),
            assertion_pending: false,
            argv: self.argv()?,
            resulting_commit: None,
            evidence: None,
        })
    }
    fn argv(&self) -> Result<Vec<Vec<String>>> {
        self.settings.as_ref().map(prepare::Settings::argv).transpose().map(Option::unwrap_or_default)
    }

    pub(super) fn existing(&self, context: &mut SourceContext) -> Result<PreparedSource> {
        let started = Instant::now();
        let result = (|| {
            let client = self.binding.client();
            let operation = client.operation_id()?;
            self.binding.verify_at(&operation)?;
            let evidence = if let Some(assertion) = &self.assertion {
                let assertion = client.resolve_one_at(&operation, assertion)?.commit_id;
                context.assertion.as_mut().expect("effective assertion").commit = Some(assertion.clone());
                native_diff(&client, &assertion).context("explicit source_at live assertion (may have recorded donor edits)")?;
                context.assertion.as_mut().unwrap().status = AssertionStatus::Succeeded;
                SourceEvidence::LiveAssertion
            } else {
                SourceEvidence::AssumedPrepared
            };
            self.handoff(None, evidence, None, context.clone())
        })();
        context.elapsed = Some(started.elapsed());
        if result.is_err()
            && let Some(assertion) = &mut context.assertion
            && !matches!(assertion.status, AssertionStatus::Succeeded)
        {
            assertion.status = AssertionStatus::Failed;
        }
        result.map(|mut ready| {
            ready.report.context = context.clone();
            ready
        })
    }
    fn handoff(
        &self,
        upstream_goal: Option<String>,
        evidence: SourceEvidence,
        preparation: Option<PreparationReport>,
        context: SourceContext,
    ) -> Result<PreparedSource> {
        let client = self.binding.client();
        let operation = client.operation_id()?;
        self.binding.verify_at(&operation)?;
        let facts = client.commit_facts_at(&operation, "@")?;
        ensure!(!facts.conflicted && !facts.divergent, "unsupported source native revision");
        Ok(PreparedSource {
            binding: Arc::clone(&self.binding),
            report: SourceReport {
                context,
                profile: self.binding.repository.profile.clone(),
                root: self.binding.root.clone(),
                operation,
                commit: facts.commit_id,
                change: facts.change_id,
                upstream_goal,
                evidence,
                preparation,
            },
        })
    }
}

#[derive(Debug)]
pub struct SourcePlan {
    pub context: SourceContext,
    pub recorded_commit: String,
    pub prospective: Option<SourceSelection>,
    pub assertion_pending: bool,
    pub argv: Vec<Vec<String>>,
    pub resulting_commit: Option<String>,
    pub evidence: Option<SourceEvidence>,
}

/// One final tracked conformance proof after all successful argv. The lock is
/// preparation-only and is dropped before this in-process handoff is consumed.
pub fn prepare_source(source: &ResolvedSource, diagnostic: &mut dyn FnMut(&str)) -> std::result::Result<PreparedSource, PreparationFailure> {
    let started = Instant::now();
    let mut completed = PreparationReport {
        steps: source
            .context
            .preparation_argv
            .as_ref()
            .into_iter()
            .flatten()
            .map(|argv| PreparationStep {
                argv: argv.clone(),
                status: StepStatus::Selected,
                elapsed: None,
            })
            .collect(),
        ..Default::default()
    };
    let mut context = source.context.clone();
    let result = (|| {
        ensure!(!source.preview_only, "use preview() for a read-only preparation plan");
        let client = source.binding.client();
        let operation = client.operation_id()?;
        source.binding.verify_at(&operation)?;
        binding::validate_local(&client, &operation)?;
        let _lock = PreparationLock::lock(&client)?;
        snapshot(&client).context("save source tracked work")?;
        let operation = client.operation_id()?;
        completed.saved_commit = Some(client.resolve_one_at(&operation, "@")?.commit_id);
        let selected = SourceSelection::select(
            &client,
            &operation,
            source.assertion.as_deref(),
            source.settings.as_ref().and_then(|s| s.remote.as_deref()),
        )?;
        context.upstream = Some(selected.clone());
        if let Some(assertion) = &mut context.assertion {
            assertion.commit = selected.old_goal.clone();
        }
        selected.verify_source_at(&client, &client.operation_id()?)?;
        diagnostic(&format!(
            "Prepare: fetching {:?}@{:?}; saved source {}",
            selected.name, selected.remote, selected.source.commit_id
        ));
        client.fetch_bookmark(&selected.remote, &selected.name)?;
        let operation = client.operation_id()?;
        source.binding.verify_at(&operation)?;
        let goal = selected.resolve_fetched(&client, &operation)?;
        if let Some(assertion) = &mut context.assertion {
            assertion.commit = Some(goal.clone());
            assertion.status = AssertionStatus::Succeeded;
        }
        completed.upstream_goal = Some(goal.clone());
        let current = client.commit_facts_at(&operation, "@")?;
        if !current.ordinary_empty_child() || current.parents != [goal.clone()] {
            client.run_refill(["new", &commit_id_revset(&goal)?])?;
        }
        let operation = client.operation_id()?;
        let child = client.commit_facts_at(&operation, "@")?;
        ensure!(
            child.ordinary_empty_child() && child.parents == [goal.clone()],
            "source new did not establish exact empty goal child"
        );
        source_identity(source, &goal, &child.change_id)?;
        for (index, step) in completed.steps.iter_mut().enumerate() {
            diagnostic(&format!("Prepare: argv {} {:?}, cwd {:?}", index + 1, step.argv, source.binding.root));
            step.status = StepStatus::Started;
            completed.failed_step = Some(index);
            let step_started = Instant::now();
            let status = Command::new(&step.argv[0])
                .args(&step.argv[1..])
                .current_dir(&source.binding.root)
                .stdin(Stdio::inherit())
                .stdout(std::io::stderr())
                .stderr(Stdio::inherit())
                .status()
                .with_context(|| format!("could not run configured preparation argv {}", index + 1));
            step.elapsed = Some(step_started.elapsed());
            let status = status?;
            ensure!(
                status.success(),
                "configured preparation command failed with {status} (step {}); ignored installation may be incomplete",
                index + 1
            );
            step.status = StepStatus::Succeeded;
            // No snapshot/live diff here: later argv may repair tracked edits.
            source_identity(source, &goal, &child.change_id)?;
            completed.failed_step = None;
        }
        completed.final_conformance_started = true;
        native_diff(&client, &goal).context("preparer changed tracked tree; final conformance failed")?;
        completed.final_conformance_succeeded = true;
        source_identity(source, &goal, &child.change_id)?;
        let operation = client.operation_id()?;
        let facts = client.commit_facts_at(&operation, "@")?;
        ensure!(facts.ordinary_empty_child(), "prepared source is not an empty child");
        completed.source_commit = Some(facts.commit_id);
        completed.elapsed_ms = started.elapsed().as_millis();
        source.handoff(Some(goal), SourceEvidence::Produced, Some(completed.clone()), context.clone())
    })();
    completed.elapsed_ms = started.elapsed().as_millis();
    context.elapsed = Some(started.elapsed());
    result
        .map(|mut ready| {
            ready.report.context = context.clone();
            ready
        })
        .map_err(|source| PreparationFailure {
            context: Box::new(context),
            completed: Box::new(completed),
            source,
        })
}

fn source_identity(source: &ResolvedSource, goal: &str, change: &str) -> Result<()> {
    let client = source.binding.client();
    let operation = client.operation_id()?;
    source.binding.verify_at(&operation)?;
    ensure!(
        jj_waltz_workspace_state::ownership::detect(&source.binding.root)?.is_none(),
        "preparation source acquired external Git ownership"
    );
    binding::validate_local(&client, &operation)?;
    let facts = client.commit_facts_at(&operation, "@")?;
    ensure!(
        facts.change_id == change && facts.parents == [goal] && !facts.conflicted && !facts.divergent,
        "source selected revision changed during preparation"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn preparation_failure_keeps_progress_and_original_cause() {
        let failure = PreparationFailure {
            context: Box::new(SourceContext {
                name: "source".into(),
                root: "/source".into(),
                operation: "op".into(),
                recorded_commit: "recorded".into(),
                assertion: None,
                upstream: None,
                elapsed: None,
                preparation_argv: None,
            }),
            completed: Box::new(PreparationReport {
                saved_commit: Some("saved source".into()),
                steps: vec![PreparationStep {
                    argv: vec!["builder".into()],
                    status: StepStatus::Started,
                    elapsed: None,
                }],
                ..Default::default()
            }),
            source: anyhow::Error::new(std::io::Error::new(ErrorKind::PermissionDenied, "builder refused")).context("start builder"),
        };
        assert!(std::mem::size_of::<PreparationFailure>() < 128);
        assert_eq!(failure.completed.saved_commit.as_deref(), Some("saved source"));
        assert_eq!(failure.completed.steps[0].status, StepStatus::Started);
        assert_eq!(failure.source.downcast_ref::<std::io::Error>().unwrap().kind(), ErrorKind::PermissionDenied);
        assert_eq!(failure.source().unwrap().to_string(), "start builder");
        let error = anyhow::Error::new(failure);
        assert_eq!(
            error.chain().find_map(|cause| cause.downcast_ref::<std::io::Error>()).unwrap().kind(),
            ErrorKind::PermissionDenied
        );
        assert!(format!("{error:#}").contains("start builder: builder refused"));
    }
}
