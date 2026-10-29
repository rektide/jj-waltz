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
pub struct SourceReport {
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
}
#[derive(Debug, Clone, Default)]
pub struct PreparationReport {
    pub saved_commit: Option<String>,
    pub upstream_goal: Option<String>,
    pub source_commit: Option<String>,
    pub steps: Vec<PreparationStep>,
    pub final_conformance_started: bool,
    pub final_conformance_succeeded: bool,
}
#[derive(Debug)]
pub struct PreparationFailure {
    pub completed: PreparationReport,
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
    pub assertion: Option<String>,
    settings: Option<prepare::Settings>,
}
impl ResolvedSource {
    pub fn resolve(cwd: &Path, request: crate::PrepareRequest) -> Result<Self> {
        let request = ReseedRequest {
            source: request.source,
            source_at: request.source_at,
            ..Default::default()
        };
        let resolution = Resolution::capture(cwd, &request, true)?;
        let registrations = jj_waltz_workspace_state::ownership::refill_registrations_at(&resolution.caller, &resolution.registry.operation)?;
        resolution.check_ownership(&resolution.source, &registrations)?;
        Ok(Self::from_resolution(&resolution, &request))
    }
    pub(super) fn from_resolution(r: &Resolution, request: &ReseedRequest) -> Self {
        Self {
            binding: Arc::clone(&r.source),
            assertion: request.source_at.clone().or_else(|| r.config.source_at.clone()),
            settings: r.config.freshen.clone(),
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

    pub(super) fn existing(&self) -> Result<PreparedSource> {
        let client = self.binding.client();
        let operation = client.operation_id()?;
        let evidence = if let Some(assertion) = &self.assertion {
            let assertion = client.resolve_one_at(&operation, assertion)?.commit_id;
            native_diff(&client, &assertion).context("explicit source_at live assertion (may have recorded donor edits)")?;
            SourceEvidence::LiveAssertion
        } else {
            SourceEvidence::AssumedPrepared
        };
        self.handoff(None, evidence, None)
    }
    fn handoff(&self, upstream_goal: Option<String>, evidence: SourceEvidence, preparation: Option<PreparationReport>) -> Result<PreparedSource> {
        let client = self.binding.client();
        let operation = client.operation_id()?;
        self.binding.verify_at(&operation)?;
        let facts = client.commit_facts_at(&operation, "@")?;
        ensure!(!facts.conflicted && !facts.divergent, "unsupported source native revision");
        Ok(PreparedSource {
            binding: Arc::clone(&self.binding),
            report: SourceReport {
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
    let mut completed = PreparationReport::default();
    let result = (|| {
        let client = source.binding.client();
        let _lock = PreparationLock::lock(&client)?;
        let operation = client.operation_id()?;
        source.binding.verify_at(&operation)?;
        binding::validate_local(&client, &operation)?;
        snapshot(&client).context("save source tracked work")?;
        let operation = client.operation_id()?;
        let selected = SourceSelection::select(
            &client,
            &operation,
            source.assertion.as_deref(),
            source.settings.as_ref().and_then(|s| s.remote.as_deref()),
        )?;
        completed.saved_commit = Some(selected.source.commit_id.clone());
        completed.steps = source
            .argv()?
            .into_iter()
            .map(|argv| PreparationStep {
                argv,
                status: StepStatus::Selected,
            })
            .collect();
        selected.verify_source_at(&client, &client.operation_id()?)?;
        diagnostic(&format!(
            "Prepare: fetching {:?}@{:?}; saved source {}",
            selected.name, selected.remote, selected.source.commit_id
        ));
        client.fetch_bookmark(&selected.remote, &selected.name)?;
        let operation = client.operation_id()?;
        source.binding.verify_at(&operation)?;
        let goal = selected.resolve_fetched(&client, &operation)?;
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
            let status = Command::new(&step.argv[0])
                .args(&step.argv[1..])
                .current_dir(&source.binding.root)
                .stdin(Stdio::inherit())
                .stdout(std::io::stderr())
                .stderr(Stdio::inherit())
                .status()
                .with_context(|| format!("could not run configured preparation argv {}", index + 1))?;
            ensure!(
                status.success(),
                "configured preparation command failed with {status} (step {}); ignored installation may be incomplete",
                index + 1
            );
            step.status = StepStatus::Succeeded;
            // No snapshot/live diff here: later argv may repair tracked edits.
            source_identity(source, &goal, &child.change_id)?;
        }
        completed.final_conformance_started = true;
        native_diff(&client, &goal).context("preparer changed tracked tree; final conformance failed")?;
        source_identity(source, &goal, &child.change_id)?;
        let operation = client.operation_id()?;
        let facts = client.commit_facts_at(&operation, "@")?;
        ensure!(facts.ordinary_empty_child(), "prepared source is not an empty child");
        completed.final_conformance_succeeded = true;
        completed.source_commit = Some(facts.commit_id);
        source.handoff(Some(goal), SourceEvidence::Produced, Some(completed.clone()))
    })();
    result.map_err(|source| PreparationFailure { completed, source })
}

fn source_identity(source: &ResolvedSource, goal: &str, change: &str) -> Result<()> {
    let client = source.binding.client();
    let operation = client.operation_id()?;
    source.binding.verify_at(&operation)?;
    binding::validate_local(&client, &operation)?;
    let facts = client.commit_facts_at(&operation, "@")?;
    ensure!(
        facts.change_id == change && facts.parents == [goal] && !facts.conflicted && !facts.divergent,
        "source selected revision changed during preparation"
    );
    Ok(())
}
