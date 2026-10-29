//! Refusal only: schema 1 has Unix path semantics. There is no non-Unix refill
//! engine, path transcoding or ordinary-copy fallback.
use jj_waltz_refill_engine::{PrepareRequest, RenewalRoute, ReseedRequest};
use serde::Serialize;
use serde_json::{Value, json};
use std::time::Instant;

#[derive(Serialize)]
#[serde(transparent)]
pub struct Envelope(Value);
impl Envelope {
    pub fn plain(&self) -> String {
        "refill requires Unix filesystem identity and CoW support\n".into()
    }
}
pub struct Outcome {
    pub envelope: Envelope,
    pub error: Option<anyhow::Error>,
}
fn unsupported(command: &'static str, preview: bool, request: Option<&ReseedRequest>) -> Outcome {
    let started = Instant::now();
    let error = anyhow::anyhow!("refill requires Unix filesystem identity and CoW support");
    let route = request.map(|r| match r.renewal_route {
        RenewalRoute::ClearThenPopulate => "clear-then-populate",
        RenewalRoute::ReplaceAuthorized => "replace-authorized",
    });
    let value = json!({
        "schema_version": 1, "command": command, "mode": if preview { "preview" } else { "execute" },
        "status": if preview { "blocked" } else { "failed" }, "renewal_route": route, "native_profile": null,
        "selection": {"names": request.map_or_else(Vec::new, |r| r.names.clone()), "groups": request.map_or_else(Vec::new, |r| r.groups.clone()), "resolved": [], "skipped": []},
        "source": null, "desired": request.map(|r| json!({"kind": if r.at.is_some() { "revset" } else { "source" }, "revset": r.at, "commit_id": null, "state": "unresolved"})),
        "preview": null, "receivers": [], "capture": null, "source_diagnostics": {"native": null, "ordinary": null},
        "timings": {"command_ms": started.elapsed().as_millis().to_string(), "engine_ms": null, "pre_native_ms": null, "ordinary_ms": null},
        "error": {"kind": "resolution", "phase": "resolve", "receiver": null, "path": null, "argv_step": null, "message": error.to_string(), "causes": [], "secondary": []}
    });
    Outcome {
        envelope: Envelope(value),
        error: Some(error),
    }
}
pub fn reseed(request: ReseedRequest) -> Outcome {
    unsupported("reseed", request.dry_run, Some(&request))
}
pub fn prepare(request: PrepareRequest) -> Outcome {
    unsupported("prepare", request.dry_run, None)
}
