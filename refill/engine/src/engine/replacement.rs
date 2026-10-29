//! Compatibility projection for existing first-slice callers. All command/native
//! orchestration now lives in `command`; this is only the narrow old N1 ordinary
//! adapter and report projection, to be removed at public cutover.
use super::command::*;
use crate::ReseedRequest;
use anyhow::{Result, ensure};
use jj_waltz_transfer_workspace::streaming::{SourceQualified, sync_one};
use jj_waltz_workspace_state::links;
use std::{path::Path, sync::Arc};

#[derive(Debug)]
pub struct RefillOneReport {
    pub name: String,
    pub source_commit: String,
    pub desired_commit: String,
    pub old_commit: String,
    pub final_commit: String,
    pub change_id: String,
    pub native_install: diff_dirs_sync::Progress,
    pub explicit_clear: diff_dirs_sync::Progress,
    pub ordinary: Option<diff_dirs_sync::coordinated::LaneReport>,
    pub links: links::LinkApplyReport,
    pub native_ms: u128,
    pub ordinary_ms: u128,
    pub finish_ms: u128,
    pub elapsed_ms: u128,
}

pub fn refill_one_from(cwd: &Path, request: ReseedRequest, mut diagnostic: impl FnMut(&str)) -> Result<RefillOneReport> {
    let command = ResolvedCommand::resolve(cwd, request, RenewalRoute::ClearThenPopulate, &mut diagnostic)?;
    ensure!(
        command.receivers.len() == 1,
        "old integration projection requires one receiver; use the shared command/cohort boundary"
    );
    let report = command.execute(&mut LegacyOrdinary, &mut diagnostic)?;
    let source = report.source.expect("successful command has source");
    let receiver = report.receivers.into_iter().next().expect("one receiver");
    let child = receiver.child.expect("successful receiver has child");
    Ok(RefillOneReport {
        name: receiver.name,
        source_commit: source.commit,
        desired_commit: receiver.desired.unwrap(),
        old_commit: receiver.old.unwrap().commit,
        final_commit: child.commit_id,
        change_id: child.change_id,
        native_install: receiver.native.installed,
        explicit_clear: receiver.native.explicit_clear,
        ordinary: receiver.ordinary.report,
        links: receiver.finish.links,
        native_ms: report.setup_ms + receiver.native.elapsed_ms,
        ordinary_ms: report.ordinary_ms,
        finish_ms: receiver.finish.elapsed_ms,
        elapsed_ms: report.elapsed_ms,
    })
}

struct LegacyOrdinary;
impl OrdinaryTransfer for LegacyOrdinary {
    fn admit(&self, receivers: usize, enabled: bool) -> Result<()> {
        ensure!(!enabled || receivers == 1, "contextual cohort adapter has not landed");
        Ok(())
    }
    fn transfer(&mut self, cohort: &OrdinaryCohort, outcomes: &mut [OrdinaryOutcome]) -> std::result::Result<(), OrdinaryFailure> {
        let lane = &cohort.lanes[0];
        outcomes[0].state = OrdinaryState::Partial;
        let visitor = SourceQualified {
            patterns: Arc::clone(&cohort.patterns),
            source_ignore: Arc::clone(&lane.source_ignore),
            desired_ignore: Arc::clone(&lane.desired_ignore),
            native: Arc::new(cohort.native.paths.clone()),
            links: lane.active_links.clone(),
            devices: (cohort.source.device, lane.binding.device),
        };
        match sync_one(&cohort.source.root, &lane.binding.root, &lane.binding.name, visitor) {
            Ok(done) => {
                outcomes[0].report = done;
                outcomes[0].state = OrdinaryState::Satisfied;
                Ok(())
            }
            Err(source) => Err(OrdinaryFailure { lane: Some(0), source }),
        }
    }
}
