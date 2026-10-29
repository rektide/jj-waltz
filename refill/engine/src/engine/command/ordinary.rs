//! Downward adapter: command-owned bindings become transfer-owned policy inputs.
//! Transfer does not depend on the engine's command/report types.
use super::*;
use cow::streaming::{CohortLane, OrdinarySource, SourceQualified, sync_cohort};

#[derive(Default)]
pub struct ContextualOrdinary {
    pub resources: diff_dirs::ObservationResources,
}
impl OrdinaryTransfer for ContextualOrdinary {
    fn admit(&self, receivers: usize, _enabled: bool) -> Result<()> {
        ensure!(receivers != 0, "ordinary cohort requires receivers");
        Ok(())
    }
    fn transfer(&mut self, cohort: &OrdinaryCohort, outcomes: &mut [OrdinaryOutcome]) -> std::result::Result<(), OrdinaryFailure> {
        if outcomes.len() != cohort.lanes.len() {
            return Err(OrdinaryFailure {
                lane: None,
                source: anyhow::anyhow!("ordinary outcome count differs from cohort"),
            });
        }
        let source = Arc::new(OrdinarySource::new(Arc::clone(&cohort.patterns), &cohort.native.paths));
        let lanes = cohort
            .lanes
            .iter()
            .map(|lane| CohortLane {
                name: lane.binding.name.clone(),
                target: lane.binding.root.clone(),
                policy: SourceQualified::new(
                    Arc::clone(&source),
                    Arc::clone(&lane.source_ignore),
                    Arc::clone(&lane.desired_ignore),
                    &lane.active_links,
                    (cohort.source.device, lane.binding.device),
                ),
            })
            .collect();
        let result = sync_cohort(&cohort.source.root, lanes, self.resources.clone());
        let reports = match &result {
            Ok(reports) => reports,
            Err(failure) => &failure.reports,
        };
        for (lane, outcome) in cohort.lanes.iter().zip(outcomes) {
            let report = reports.iter().find(|r| r.target.0 == lane.binding.name);
            outcome.report = report.cloned();
            outcome.state = if report.is_some_and(|r| r.satisfied) {
                OrdinaryState::Satisfied
            } else {
                OrdinaryState::Partial
            };
        }
        result.map(|_| ()).map_err(|failure| OrdinaryFailure {
            lane: failure.lane,
            source: failure.source,
        })
    }
}
