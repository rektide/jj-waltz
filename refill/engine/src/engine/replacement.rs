//! Compatibility projection for existing first-slice callers. All command/native
//! orchestration now lives in `command`; this report projection delegates N1 to
//! the same actual cohort as N2, and can be removed at public cutover.
use super::command::*;
use crate::ReseedRequest;
use anyhow::{Result, ensure};
use jj_waltz_workspace_state::links;
use std::path::Path;

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
    let report = command.execute(&mut ContextualOrdinary::default(), &mut diagnostic)?;
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
        native_install: receiver.native.completed,
        explicit_clear: receiver.native.explicit_clear,
        ordinary: receiver.ordinary.report,
        links: receiver.finish.links,
        native_ms: (report.setup + receiver.native.elapsed.unwrap_or_default()).as_millis(),
        ordinary_ms: report.ordinary_elapsed.unwrap_or_default().as_millis(),
        finish_ms: receiver.finish.elapsed.unwrap_or_default().as_millis(),
        elapsed_ms: report.elapsed.as_millis(),
    })
}
