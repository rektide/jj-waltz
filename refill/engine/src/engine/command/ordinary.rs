//! Downward adapter: command-owned bindings become transfer-owned policy inputs.
//! Transfer does not depend on the engine's command/report types.
use super::*;
use cow::streaming::{CohortLane, OrdinarySource, SourceQualified, sync_cohort_with_execution};

#[derive(Default)]
pub struct ContextualOrdinary {
    pub resources: diff_dirs::ObservationResources,
    execution: Option<cow::transfer::Execution>,
}

impl OrdinaryTransfer for ContextualOrdinary {
    fn admit(&mut self, receivers: usize, enabled: bool) -> Result<()> {
        ensure!(receivers != 0, "ordinary cohort requires receivers");
        self.execution = Some(cow::transfer::Execution::new(enabled)?);
        Ok(())
    }
    fn transfer(
        &mut self,
        cohort: &OrdinaryCohort,
        outcomes: &mut [OrdinaryOutcome],
    ) -> std::result::Result<Option<diff_dirs_inspect::SourceDiagnostics>, OrdinaryFailure> {
        if outcomes.len() != cohort.lanes.len() {
            return Err(OrdinaryFailure {
                lane: None,
                diagnostics: None,
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
        let execution = self.execution.as_ref().ok_or_else(|| OrdinaryFailure {
            lane: None,
            diagnostics: None,
            source: anyhow::anyhow!("ordinary execution resources were not admitted before native work"),
        })?;
        let result = sync_cohort_with_execution(&cohort.source.root, lanes, self.resources.clone(), execution);
        let reports = match &result {
            Ok(report) => &report.reports,
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
        result.map(|report| report.diagnostics).map_err(|failure| OrdinaryFailure {
            lane: failure.lane,
            diagnostics: failure.diagnostics,
            source: failure.source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutable_admission_owns_execution_before_native_work_and_disabled_is_serial() {
        let mut ordinary = ContextualOrdinary::default();
        assert!(ordinary.execution.is_none());
        ordinary.admit(2, true).unwrap();
        ordinary.execution.as_ref().unwrap().scope(|mode| {
            #[cfg(feature = "payload-parallel")]
            {
                assert_eq!(mode, diff_dirs::Execution::Parallel);
                assert!(std::thread::current().name().unwrap().starts_with("jw-payload-"));
            }
            #[cfg(not(feature = "payload-parallel"))]
            assert_eq!(mode, diff_dirs::Execution::Sequential);
        });
        ordinary.admit(2, false).unwrap();
        ordinary
            .execution
            .as_ref()
            .unwrap()
            .scope(|mode| assert_eq!(mode, diff_dirs::Execution::Sequential));
    }
}
