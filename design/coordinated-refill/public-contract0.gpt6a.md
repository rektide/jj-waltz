---
type: ImplementationDesign
title: Public coordinated-refill wire contract and CLI cutover
description: Application-owned schema 1, truthful source and receiver projections, structured failure output, old-driver retirement and the bounded renewal evidence binding.
resource: /design/coordinated-refill/public-contract0.gpt6a.md
status: design-for-public-cutover-after-native-review
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e78f2e0eaffejn7gBDnYKJ2RhA"
    title: Defines the missing public DTO/schema, status and unknown-value rules, committed caller retirement map, native reporting handoff and explicit live-asserted measurement profile
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions design only, reserves native checkpoint review and authorizes reading completed designs
  - resource: /design/coordinated-refill/execution0.gpt6a.md
    title: Accepted sections 7–10 reporting, nonmutating preview, exact native profile and coherent public cutover
  - resource: /design/coordinated-refill/execution0.sol61x.md
    title: Implemented shared command, source-only producer and preview boundary
  - resource: /design/coordinated-refill/execution1.sol61x.md
    title: Implemented JW 0.9 contextual N1/N2 cohort and distinct provider report adoption
  - resource: /design/coordinated-refill/context-validation1.sol61x.md
    title: Committed restored-context correction and receiving verification limits
  - resource: "file:///home/rektide/src/diff-dirs/design/architecture/execution3.sol61x.md"
    title: Provider 0.6 source diagnostics, deviation versus retention and V3 reporting
  - resource: "file:///home/rektide/src/diff-dirs/design/architecture/execution4.sol61x.md"
    title: Provider 0.7 actual listed coverage and returned effects
  - resource: /app/cli/src/cli.rs
    revision: a11befca
    title: Actual command arguments, legacy dispatch, output and READY-era rendering
  - resource: /app/cli/src/main.rs
    revision: a11befca
    title: Existing stderr error-chain and nonzero-exit boundary
  - resource: /native/jj/src/context.rs
    revision: a11befca
    title: Generic error hint currently parses strings and rediscovers context after failure; refill outcome handling must bypass it
  - resource: /refill/engine/src/lib.rs
    revision: a11befca
    title: Actual requests, unversioned legacy report and two old public wrappers
  - resource: /refill/engine/src/engine.rs
    revision: a11befca
    title: Old orchestration and shared configuration, identity, lock and snapshot dependencies
  - resource: /refill/engine/src/engine/command.rs
    revision: a11befca
    title: Actual command plan, source/receiver execution and phase timing boundaries
  - resource: /refill/engine/src/engine/command/report.rs
    revision: a11befca
    title: Actual native, ordinary, finish, history and failure structures
  - resource: /refill/engine/src/engine/command/source.rs
    revision: a11befca
    title: Actual S versus upstream goal, source plans and preparation partial reports
  - resource: /refill/engine/src/engine/prepare.rs
    revision: a11befca
    title: Legacy goal-returning producer and still-used argv settings
  - resource: /transfer/workspace/src/streaming.rs
    revision: a11befca
    title: Actual single SourceSession cohort and returned failure reports; diagnostics not yet exported
  - resource: /workspace/lifecycle/src/ignored.rs
    revision: a11befca
    title: Creation-owned IgnoreRules, selection, validation and transfer adapter callers that survive retirement
  - resource: /tools/performance/src/renewal/mod.rs
    revision: a11befca
    title: Pending report_evidence, current source-at argv, artifact binding and bounded acceptance/sample separation
  - resource: /docs/reseed.md
    revision: a11befca
    title: Public promises that must migrate together with dispatch
  - resource: /CONTEXT.md
    revision: a11befca
    title: Existing prepared-commit, snapshot and preview vocabulary needing deliberate migration
verification:
  method: Read committed JW a11befca and provider 86c6bdb8 interfaces plus completed design accounts; explicit Markdown/OKF check
  scope: Design and caller mapping only; no production changes, unfinished native inspection, protocol rerun or benchmark
---

# One public contract, after both native routes

**Cut over both commands together to application-owned JSON schema 1.** Reseed
uses the shared command and `ContextualOrdinary` for N1 and N2; prepare uses only
`ResolvedSource`/`prepare_source`. The parent reviews the native two-route
checkpoint before this cutover. This document fixes the wire names, not the Rust
member names being implemented by the native owner.

S is the actual donor WC commit consumed; D is the exact desired receiver parent;
O is the saved receiver WC commit. Upstream goal, S and tree equality are distinct.
Native coverage proves full S installation before adoption; final D conformance
and loaded finish establish completion. Neither ordinary satisfaction nor a clone
counter proves final-D donor sharing. Accepted semantics remain in
[execution0](execution0.gpt6a.md#7-structured-phase-reports-and-forward-errors).

## 1. CLI and application boundary

In [`app/cli/src/cli.rs`](/app/cli/src/cli.rs):

- Add `reseed --format=plain|json` (plain default) and
  `--renewal-route=clear-then-populate|replace-authorized` (clear-first default).
  The shared `RenewalRoute` is lowered explicitly; do not derive wire spelling
  from Rust Debug/Display. `--replace-content` remains independent broad-clear
  authority. Both routes are public only after their actual implementations pass.
- `run_reseed` resolves once, then calls `preview()` or
  `execute(&mut ContextualOrdinary, diagnostic)`. `--prepare` stays inside that
  same command and passes actual S directly, with no extra donor assertion.
- `run_prepare` resolves the source-only request and calls its preview/producer.
  Remove prepare's `--copy-ignored`/`--no-copy-ignored` and the corresponding
  `PrepareRequest` fields: preparation no longer selects or audits payload.
  These flags remain meaningful on reseed/creation. Do not silently accept them
  as no-ops. Configured builders still run normally; no builder means existing
  ignored data, not newly built or verified payload.
- Add `renewal_route` with the clear-first default to `ReseedRequest`; remove the
  redundant separate route argument from `ResolvedCommand::resolve` when updating
  its callers. No second route setting or alternate CLI driver is retained.
- Replace `run_refill`/`print_refill_report` with one outcome renderer. Start its
  envelope before cwd/config/source resolution, so those errors also get JSON.
  Keep successful resolution facts as data, not strings scraped from diagnostics.

Put DTOs and projections in a new **`app/cli/src/refill_report.rs`** and register
the module in [`main.rs`](/app/cli/src/main.rs). Engine/provider reports keep
native types and original errors; serde belongs to this application boundary.
Use `#[derive(Serialize)]`, explicit enum renames, and `Decimal(u128)` with a
`serialize_str` implementation. No `#[serde(default)]` or `unwrap_or_default()`
may turn an absent measurement/report into a completed zero. Serialize to a byte
buffer before writing, as today's `write_json` already does.

Add **`docs/schema/refill-v1.schema.json`** (JSON Schema 2020-12) and JSON fixtures
under **`app/cli/tests/fixtures/refill/`** during implementation. No reseed/prepare
JSON schema file exists at the inspected revision. The unrelated snapshot/context/
doctor schemas retain their own versions. Consumers dispatch on `command` and
`schema_version`, reject unsupported versions, and may ignore additive fields.

## 2. Schema 1 vocabulary and envelope

All listed keys are emitted, including explicit `null`; arrays retain command
selection order. `N` below means canonical unsigned decimal **string** matching
`^(0|[1-9][0-9]*)$`, decoded with checked `u128` conversion. Every count, duration
and step index uses N, even when its current Rust source is `u64`/`usize`.
`schema_version` is the JSON **number** `1`. IDs/revsets are strings, never numbers.
Durations use integer milliseconds throughout; a genuinely measured sub-ms span
may be `"0"`. Unknown, unstarted, disabled or unavailable spans are `null`.

`Path` is `{display: string, unix_bytes_hex: string}`: lowercase, even-length hex
of the exact Unix `OsStr` bytes, plus human display text. Decode the hex for exact
identity; display text is not reversible. This avoids losing byte-named provider
failure coordinates or failing JSON serialization after effects. Initially only
Unix is supported. Do not JSON-serialize `PathBuf` blindly or parse Debug paths.

The following type notation defines the complete object keys. `?` means nullable,
not omitted; `T[]` means an array. Nested definitions follow.

```text
Envelope {
  schema_version: 1,
  command: "reseed" | "prepare",
  mode: "execute" | "preview",
  status: "complete" | "planned" | "failed" | "blocked",
  renewal_route: Route?,
  native_profile: {id: string, raw_version: string}?,
  selection: Selection,
  source: Source?,
  desired: Desired?,
  preview: Preview?,
  receivers: Receiver[],
  capture: {distinct_trees: N, membership_queries: N,
            indexes: N, donor_cache_reads: N}?,
  source_diagnostics: {native: SourceDiagnostics?, ordinary: SourceDiagnostics?},
  timings: {command_ms: N?, engine_ms: N?, pre_native_ms: N?, ordinary_ms: N?},
  error: Error?
}
Route = "clear-then-populate" | "replace-authorized"
Selection {
  names: string[], groups: string[],
  resolved: {name: string, root: Path, origins: string[]}[],
  skipped: {candidate: string, reason: string}[]
}
Desired {
  kind: "source" | "revset", revset: string?, commit_id: string?,
  state: "unresolved" | "recorded" | "after_live_assertion" |
         "future_prepared_source" | "post_preparation_revset" | "resolved"
}
Preview {
  label: "PLANNED — payload/path obstructions not inspected",
  clone_capability_probed: false,
  preservation_pending: boolean,
  obstructions_inspected: false,
  ordinary_patterns: string[]?
}
```

Standalone prepare has `renewal_route`, `desired`, `capture` null, no receivers,
and empty selection arrays. Its preview has `preservation_pending:false` (no
receivers) and `ordinary_patterns:null`. Reseed preview exposes declarations only.
If resolution fails before a plan exists, `preview` is null even in preview mode.
The profile is the actual `NativeProfile`: initially ID
`local-tree-state-33d3a595`, exact build
`0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`; retain the raw `jj …` text.
Classification stays in [`native/jj/src/refill.rs`](/native/jj/src/refill.rs).
An unsupported/unresolved profile is null, never the requested profile disguised
as an observed one; its original diagnostic remains in `error`.

`selection.names/groups` preserve user input (empty for implicit current selection).
Resolved origins preserve the existing selection resolver's labels, including
`current`. Keep structured skips as well as its stderr warning. Current
`ResolvedCommand::resolve` discards origins/skips: retain them in its resolved
data and carry available facts through a typed resolution failure. An error
before a valid selection is available has `resolved:[]`, `receivers:[]`; unresolved
requested names are not synthetic failed receiver rows. Partial resolution is
allowed to return known bound rows, but never requires more discovery after error.

### Source and preparation

```text
Source {
  name: string, root: Path, operation_id: string?,
  recorded_commit_id: string?, source_commit_id: string?, source_change_id: string?,
  upstream_goal_commit_id: string?,
  state: "planned" | "not_started" | "ready" | "failed",
  evidence: "assumed_prepared" | "live_assertion" | "produced" | null,
  assertion: {revset: string, commit_id: string?,
              status: "pending" | "succeeded" | "failed"}?,
  upstream: {remote: string, bookmark: string,
             prospective_goal_commit_id: string?}?,
  preparation: Preparation?,
  elapsed_ms: N?
}
Preparation {
  status: "planned" | "not_started" | "succeeded" | "failed",
  saved_commit_id: string?,
  steps: {index: N, argv: string[],
          status: "selected" | "started" | "succeeded", elapsed_ms: N?}[],
  final_conformance: "pending" | "succeeded" | "failed",
  elapsed_ms: N?
}
```

`recorded_commit_id` is the source revision read before requested work; it is not
a verified live payload coordinate. `source_commit_id` is **only accepted actual
S from a successful handoff**, null in every preview and failed production/assertion.
Even plain preview's current `SourcePlan.resulting_commit` goes into recorded
intent, not this execution-result field. Plain preview says `evidence:assumed_prepared`;
assertion/producer previews have null evidence and pending work. A successful plain
execution says `ready/assumed_prepared` without an implicit donor snapshot.

`operation_id` binds accepted S when present; otherwise it is the captured recorded
view. `source_change_id` is null until the handoff. Resolve effective assertions
from CLI **or local config**. Keep their exact resolved commit independently of S;
equal trees are not equal IDs. `upstream` records pinned preparation selection;
its prospective goal may predate fetch. Only the fetched goal goes in
`upstream_goal_commit_id`. No upstream goal is invented for ordinary assertions.

Map producer `Selected/Started/Succeeded` exactly. On process failure the attempted
step stays `started`, preparation is `failed`, and `error.argv_step` locates it;
later steps stay `selected`. A successful process followed by an identity-check
failure remains `succeeded` as a process, while preparation fails. Final conformance
is failed only when started and unsuccessful; an earlier failure leaves it pending.
A handoff/identity error after successful conformance does not rewrite that check
as failed. No successful prepared-source claim follows an incomplete producer.
Step `index` and `error.argv_step` are zero-based; human labels may display index+1.

### Receiver, native and ordinary results

```text
Receiver {
  name: string, root: Path, renewal_route: Route,
  status: "planned" | "blocked" | "not_started" | "partial" | "failed" | "complete",
  phase: Phase,
  saved: {commit_id: string, history: History}?,
  desired_commit_id: string?,
  child: {commit_id: string, change_id: string, parent_commit_ids: string[]}?,
  native: Native,
  ordinary: Ordinary,
  finish: Finish
}
History {
  status: "pending" | "bypassed" | "covered",
  work_tip_commit_id: string?, bookmark: string?, remote: string?,
  bookmark_target_commit_id: string?
}
Native {
  state: "unstarted" | "in_progress" | "established",
  manifest: {desired: N, installed: N, owned_obsolete: N,
             owned_retired: N, complete: boolean}?,
  renewal_effects: Effects?, explicit_clear_effects: Effects?,
  source_state_adopted: boolean?,
  steps: {phase: Phase, status: "succeeded" | "failed", elapsed_ms: N?}[],
  elapsed_ms: N?, tracked_install_ms: N?
}
Ordinary {
  state: "disabled" | "unstarted" | "partial" | "satisfied",
  report: {
    satisfied: boolean, deviated: N, retained: N,
    applications: N, planned: N, completed: Effects,
    opaque_deviated: N, additional_retained: N, excluded: N,
    end: "comparison_failed" | "application_failed" | "refresh_failed" |
         "decision_changed" | "abandoned" | null,
    pruned: boolean, observed: Observed
  }?,
  application_ms: N?
}
Finish {
  state: "unstarted" | "partial" | "complete",
  links: {linked: N, directories_created: N, satisfied: N,
          skipped_missing_target: N}?,
  failed_path: Path?, elapsed_ms: N?
}
```

`Phase` values are `resolve`, `source`, `preserve`, `capture`, `native_clear`,
`native_child`, `explicit_clear`, `install`, `admit_desired`, `adopt`, `restore`,
`reparent`, `native_verify`, `ordinary`, `links`, `finish`, `complete`.
These are explicit stable mappings, not today's prose `Phase::Display` strings.
`reparent` is the required new native phase. `phase` means last entered/attempted,
not last succeeded. Preview rows are `planned/resolve`, with unstarted native,
ordinary and finish, empty native steps, null effects/coverage/timings/saved/child.
Recorded D may populate `desired_commit_id` only with `desired.state:recorded`.

`Effects` has exactly these N fields (same names for `Progress` and `WorkTotals`):
`files_copied`, `files_reflinked`, `bytes_copied`, `bytes_reflinked`,
`symlinks_created`, `directories_created`, `files_removed`, `symlinks_removed`,
`directories_removed`, `directory_permissions_set`.
It contains actual returned visible kernel work, including returned failure
progress; private discarded preparations are excluded. Replacements are not a
claim about unique names, all syscalls, or predicate evaluations. Do not combine
native removal counts with K(O).len(), or relabel retired-absence credits as deletes.
Native command effects such as sparse clear/restore/rebase are **unenumerated**;
their attempts appear in steps, never as fabricated provider Effects.

`Observed` is a compact projection of the actual committed comparison prefix:
`{state: "incomplete"|"complete"|"failed", scoped: boolean, comparator: string?,
excluded: N, planned: N, deviated: N, retained: N, equal: N, different: N,
not_assessed: N, unsupported: N, unresolved: N, stopped_paired: N,
examined_directories: N}`. It intentionally omits path vectors and difference
kind inventories. It is not a post-effect census. Ordinary top-level `deviated`
and `retained` map independently from `LaneReport`, not from one another or by
subtraction. `opaque_deviated`/`additional_retained` are diagnostic components
already included in those totals; do not add them again. Keep application
exclusions separate from observed exclusions. Satisfaction is scoped-policy
completion, not full-root equality or storage-sharing proof.

`source_state_adopted` is null until native work starts, false before successful
cache adoption, true afterward; true is not final conformance. Coverage requires
the actual returned manifest and successful listed outcome before adoption.
Even installed=desired can coexist with complete=false after cleanup failure.
`renewal_effects` includes listed owned removals as well as S installations;
`explicit_clear_effects` is the separate broad-clear call. No synthetic total
mixes those with native JJ effects. D≠S may remove S installs and materialize D
from the store; the schema deliberately has no `final_shared_files` field.

Finish links are actual `LinkApplyReport`/`LinkFinishFailure.completed`, not
`LinkPlan` counts. A pre-link inspection failure has links=null; a returned
zero-effect link application has an all-zero object. A successful link application
followed by native identity failure retains its link counts with finish partial.

### Terminal status and failure rules

- Successful execution: command `complete`; every reseed row `complete`, native
  established, ordinary disabled or satisfied, finish complete. Prepare completes
  only with an accepted produced source. Zero receivers is normal for prepare.
- Successful preview: command/rows `planned`, exit 0. Static preview refusal:
  command `blocked`, nonzero; mark a bound offending row blocked only if identified.
  Other known rows remain planned. Never produce READY or claim payload readiness.
- Execution error: command `failed`, nonzero. The attributed receiver is failed;
  earlier fully finished rows remain complete. Work started in another unfinished
  row makes it partial, even if only preservation ran. Truly untouched rows remain
  not_started. Shared errors need not be assigned to a receiver. In particular,
  final donor-barrier failure can coexist with every receiver complete.
- A native-ready or ordinary-satisfied row whose finish never ran is partial,
  not complete. A missing provider report is null, not an empty successful report.
  Disabled ordinary work has report=null and application_ms=null.

## 3. Failure transport, timing and shared-source diagnostics

```text
Error {
  kind: "resolution" | "preparation" | "execution",
  phase: Phase, receiver: string?, path: Path?, argv_step: N?,
  message: string, causes: string[],
  secondary: {operation: string?, path: Path?, message: string, causes: string[]}[]
}
SourceDiagnostics {
  paired: SourceWork, opaque: SourceWork, declared: SourceWork,
  retained_batches: N, retained_charged_capacity: N, active_batches: N, evictions: N
}
SourceWork {
  listings: N, metadata_acquisitions: N, link_acquisitions: N,
  fact_hits: N, cache_hits: N, cache_misses: N
}
```

Use an application result like `Result<Envelope, ReportedFailure>` where
`ReportedFailure { envelope, source: anyhow::Error }` retains the actual error.
Project a borrowed `CommandFailure`/`PreparationFailure` before moving it into the
outer error. Match typed provider `Failure`/`Issue`, link failure and typed
location data to obtain path/step/secondary details; walk `Error::source()` for
human cause strings. `message` is the primary contextual message, `causes` its
ordered inner chain. It is explicitly diagnostic text, never a machine classifier.
Unknown path/step is null. Do not parse error strings, invent OS error codes or
serialize arbitrary anyhow/provider internals. Keep original causes downcastable
in Rust, including secondary provider causes; the wire is a projection.

For every normally returned success/failure of a **parsed** JSON command, emit one
complete stdout document, then exit 0 or nonzero. Child builder stdout and all
diagnostics go to stderr. Serialize failures before returning their original
error to main's existing stderr/exit handling; no second report or full Debug
dump. Clap syntax/help/version and broken stdout/process termination are outside
this command-outcome guarantee. Do not try to print a replacement JSON document
after a partial stdout write.
Bypass `cli::run`'s final `context::add_workspace_hint` for these already handled
outcomes: at the inspected revision it classifies text and rediscovers context
after errors. Reporting must not launch fresh native discovery, especially on a
blocked preview or partial refill. Any applicable hint comes from captured facts.

Human output: one source line with evidence and separate S/goal, one compact row
per receiver with route/status/phase and known renewal/ordinary/link counts,
then an error summary identifying preserved O/history/root and partial state.
Use `unknown` for unavailable counts. Preview prints the exact PLANNED label.
Do not print observed path vectors, full report Debug, or universal recovery
instructions inferred only from sparse state: a failed replacement can be full
sparse but incomplete. Retain deliberate out-of-band recovery guidance.

Timing scopes (all nullable N milliseconds):

| Wire field | Meaning and current mapping |
| --- | --- |
| `timings.command_ms` | CLI handler entry before cwd/resolution through returned outcome, excluding rendering/stdout; measured also for early failures and preview. |
| `timings.engine_ms` | Existing `CommandReport.elapsed_ms`; resolve through final barrier. Null for preview/standalone prepare. |
| `timings.pre_native_ms` | Existing `setup_ms`: resolution, probes, source work, preservation and shared capture before first native receiver; elapsed prefix on early failure. Not pure configuration time. |
| `source.elapsed_ms` | Source phase (plain handoff, named live assertion or producer) once; new timer, null before it starts. |
| `source.preparation.elapsed_ms` | Existing `PreparationReport.elapsed_ms`; producer scope only, nested in source time. |
| `source.preparation.steps[].elapsed_ms` | Optional actual process span; not presently supplied. |
| `receivers[].native.elapsed_ms` | Existing native establishment span, excludes receiver preservation. |
| `receivers[].native.tracked_install_ms` | Listed renewal call, including its inspection/guards/cleanup, excluding native clear/new/adopt/restore/reparent/verification. Native owner is adding this separation. |
| `receivers[].native.steps[].elapsed_ms` | Optional timed phase span; failed attempts retain elapsed prefix. Install includes the same call as tracked_install_ms, not additional time. |
| `timings.ordinary_ms` | Existing one-cohort wall span, once across all receivers; null if disabled/unstarted. |
| `receivers[].ordinary.application_ms` | Optional sum of actual lane apply-call spans only; excludes shared comparison/group acquisition. Currently unavailable, null. |
| `receivers[].finish.elapsed_ms` | Existing loaded-link plus snapshot-free native identity finish span. |

Do not multiply cohort time by lane count, sum overlapping spans, subtract rounded
totals to manufacture phases, or use internal milliseconds as external process
latency. Existing default `0` fields need start/availability evidence before
projection; ideally engine timing members become `Option<u128>`.

Diagnostics live **once at command/source-phase scope**, never copied into rows.
Map the provider `SourceSession::diagnostics()` fields verbatim. Native and ordinary
are separate physical-source phases/generations. If native uses separate sessions
per serial receiver, sum work/eviction counters once per disjoint session; sum
final gauge snapshots only with the documented meaning “retained/active ownership
at each session's return,” never peak RSS. Prefer one stable native source session
when already supported, without changing serial fail-fast execution for reporting.
Sample before releasing owners, on success and failure; before construction use
null. Source diagnostics count acquisitions/reuse attempts, **not raw syscalls or
policy-predicate proofs**. Route checks, target work and actual Effects are separate.

## 4. Existing-field mapping and exact implementation gaps

| Committed owner/type at `a11befca` | Required projection/change |
| --- | --- |
| `command/report.rs`: `CommandReport.profile/source/preparation/receivers/capture` | Map profile and rows; place preparation only under source, avoiding its current duplicate storage in report/source. Capture is null until capture actually starts; retain only actual partial counts. |
| `command/source.rs`: `SourceReport.commit/change/operation/upstream_goal/evidence` | Map to source result IDs, operation, goal and explicit evidence enum; bind name/root from resolved source. Preserve recorded-before-work coordinate separately. |
| `SourcePlan.recorded_commit/prospective/assertion_pending/argv` | Recorded/prospective/pending plan; never pretend resulting S has been produced. Preserve effective assertion revset/exact result and upstream selection in execution reports too. |
| `DesiredPlan` | Recorded → recorded; AfterLiveSourceAssertion → after_live_assertion; FuturePreparedSource → future_prepared_source; PostPreparationRevset → post_preparation_revset. Execution records resolved D once, even before first receiver snapshot. |
| `ReceiverReport.old/desired/child/reached/status` | Saved O/history, D, actual known child facts, explicit phase/status mapping. Current NotStarted survives preservation in some failure paths: correct typed status at phase entry or normalize from explicit entered-phase facts, never from elapsed time/counts. |
| `NativeOutcome.installed/explicit_clear/manifest_complete/attempted` | Effects only where a provider result exists; replace boolean-only coverage with actual manifest. Record adoption completion and typed step outcomes, including reparent. Attempts alone cannot prove success. |
| `OrdinaryOutcome.report` / provider `LaneReport` | Actual totals/observed/satisfaction; preserve null before valid report. No local reconstruction of deviation or retention. |
| `FinishOutcome.links/failed_path/complete` | Preserve actual returned links and failure path; add entered/returned availability so default zeros are not success. |
| `FailureLocation {phase, receiver}` | Extend typed location with optional path/argv index where owned; keep original chain and provider secondary issues. Resolution failure carries available resolved facts rather than bypassing renderer. |
| `streaming.rs`: `sync_cohort` / `CohortFailure` | Return source diagnostics alongside lane reports on success/failure; retain the existing SourceSession long enough to sample it. Do not build another runtime/walk for metrics. |

**Native handoff:** the native worker is adding actual `ManifestCoverage` and
separate tracked-install timing. Required semantics are all five coverage fields,
complete plus successful lane before adoption, visible partial effects on failure,
install-call elapsed prefix, and no fake zeros before calls. Additional public
projection needs are phase attempts with outcomes, cache-adoption completion,
effect/timing availability, and command-scoped native source diagnostics. These
are requirements for the next integration slice, **not assertions about names or
fields already delivered by that worker**. Read its committed checkpoint before
binding the DTO. No native protocol rerun is commissioned here.

## 5. Complete early-failure example

Illustrative values, not a runtime receipt. An invalid configured donor fails
before resolving a source or any receivers; the requested target remains visible.

```json
{
  "schema_version": 1,
  "command": "reseed",
  "mode": "execute",
  "status": "failed",
  "renewal_route": "clear-then-populate",
  "native_profile": null,
  "selection": {"names": ["worker-a"], "groups": [], "resolved": [], "skipped": []},
  "source": null,
  "desired": {"kind": "source", "revset": null, "commit_id": null, "state": "unresolved"},
  "preview": null,
  "receivers": [],
  "capture": null,
  "source_diagnostics": {"native": null, "ordinary": null},
  "timings": {"command_ms": "2", "engine_ms": null, "pre_native_ms": null, "ordinary_ms": null},
  "error": {
    "kind": "resolution", "phase": "resolve", "receiver": null,
    "path": null, "argv_step": null,
    "message": "cannot resolve configured source",
    "causes": ["No such file or directory (os error 2)"],
    "secondary": []
  }
}
```

For a partial native failure, the same envelope instead retains actual bound rows,
saved O, resolved D, known child, manifest prefix and returned renewal effects;
the offending row is failed at its actual phase. Ordinary/finish remain unstarted
with null reports if their phases never ran. A prepare result uses this same
envelope with command=prepare, no desired/route/receivers, and its source report.

## 6. Retirement dependency and caller map

The committed production chain is
`cli::run_reseed → run_refill → lib::run_with_diagnostics → run_internal →
engine::run/run_inner → Plan/execute`; prepare reaches that same driver through
`prepare_with_diagnostics` with `source_only=true`. The new shared command is
already callable but not this public path. Retire the old chain completely once
dispatch/tests move; do not retain it as a fallback or adapter implementation.

| Owner | Delete or preserve on actual caller evidence |
| --- | --- |
| [`refill/engine/src/lib.rs`](/refill/engine/src/lib.rs) | Delete old `ReseedReport`, `ReseedOutcome`, `ReseedStatus`, `run_internal` and source_only routing. Old `run`/diagnostic/prepare convenience entrypoints may be removed or become thin wrappers returning the new engine outcomes, never execute old Plan. Keep request/selection data still used by the resolver. |
| [`engine.rs`](/refill/engine/src/engine.rs) | Delete `Plan`, `Readiness`, old `run/run_inner/execute`, old Inventory/context rediscovery, payload realization/audits, repeated `verify_disk`, preflight obstruction/link simulation, `FutureContent/FutureNode`, descriptions and old per-target source validation once unreferenced. Retain/extract `Identity`, `LocalConfig/load_config`, `PreparationLock`, `snapshot`: the new command imports these directly. Do not delete the parent module wholesale. |
| [`engine/prepare.rs`](/refill/engine/src/engine/prepare.rs) | Keep `Settings` and argv normalization used by new source/config. Delete old `execute`, SourceIdentity, per-step conformance and payload checks; new producer owns execution. Keep native upstream/publication logic in `freshen/source.rs`. |
| [`engine/payload.rs`](/refill/engine/src/engine/payload.rs) | Old-engine-only audit/selection owner: delete when old callers are gone, including its superseded tests. Shared command uses provider guards, not this prewalk. |
| [`engine/replacement.rs`](/refill/engine/src/engine/replacement.rs) | Delete temporary `refill_one_from`/`RefillOneReport` export and one-target restrictions. Migrate `app/cli/tests/refill_one.rs` to shared command/public invocation while retaining meaningful native assertions. |
| `engine/bookmarks.rs`, `engine/selection.rs`, `command/*` | Keep actual shared-command dependencies. Preserve coverage/history and strict/warn selection semantics; carry their data into the DTO. |
| [`workspace/lifecycle/src/ignored.rs`](/workspace/lifecycle/src/ignored.rs) | Keep creation's `IgnoredPlan`, `IgnoreRules`, tracked membership/identity/confinement helpers, `Patterns::select`/`validate_selected`, `transfer::Execution`, `Auxiliary::legacy(...).new_targets_only()` and rollback-owned links. These are real add/s callers, not reasons to keep old reseed orchestration. |
| `transfer/workspace/src/frozen.rs`, `tracked.rs`, `transfer.rs` | Keep shared `FrozenIgnores`, creation's `IgnoreRules` and actually used transfer/tracked helpers. `FrozenRules`' old reseed cache can retire after caller search. Do not delete a whole transfer module because one reseed driver disappeared. `streaming::sync_one` is a thin cohort projection used in tests, not a second public reseed path. |

Inspect the **settled committed** caller graph again at cutover. This map is based
on `a11befca`, not concurrent native edits. Update engine callable tests and
`app/cli/tests/{reseed,prepare,reseed_bookmarks,refill_one}.rs`; keep creation tests
as the dependency-retention guard. Route/default/JSON and failure tests belong at
the actual application subprocess boundary, not only DTO unit serialization.

Migrate help, [`docs/reseed.md`](/docs/reseed.md), README, CONTEXT, architecture,
skill exposure and schema/fixtures together. Explicitly replace these old promises:

- Unversioned prepare `.source_commit_id` was **goal** (old `prepare::execute`
  returns goal); schema 1 uses `.source.source_commit_id` for actual S and
  `.source.upstream_goal_commit_id` for goal. No ambiguous compatibility alias.
  Scripts selecting the old goal use the latter; scripts intending actual S use
  the former. `reseed --at` remains independent, defaulting to actual S.
- Per-argv tracked conformance → one final check after successful ordered argv;
  payload verification → no preparer payload census.
- Snapshotting READY/BLOCKED exhaustive readiness preview → read-only PLANNED,
  pending live assertion/preservation, unprobed cloning and uninspected payload.
  Remove tests requiring READY, peer-wide payload preflight or preview snapshots.
  Static blockers still fail; read-only planning is not an all-target readiness promise.
- Old sparse-reset/no-cache-adoption description → the selected actual two routes,
  complete-S adoption gate, D≠S limits and potentially full-sparse interrupted state.

Coordinate the next pre-alpha JW minor/API pins and receiving locks under native
policy, after inspecting the native owner's version marker. Provider 0.7 adoption
belongs to that checkpoint. Neither this design nor cutover implicitly authorizes
build installation, deployment, publication or push. Parent owns indexes/tickets.

## 7. Performance adapter binding and acceptance

[`tools/performance/src/renewal/mod.rs`](/tools/performance/src/renewal/mod.rs)
already preserves raw argv/stdout/stderr/process outcome before parsing and gates
samples behind four shapes × two routes. Implement its `report_evidence` against
this schema, not inferred argv/help capability:

1. Require numeric schema 1, reseed/execute/complete, error=null, actual selected
   route and exact supported profile; require expected ordered receiver names.
2. Require source state ready, actual S=`fixture.s`, resolved desired D equal to
   the fixture case, per-row D and child parent exactly D. Require every row
   complete/native established, adopted=true, manifest complete and
   installed=desired, owned_retired=owned_obsolete. Match desired coverage to the
   fixture's expected tracked leaf count; do not infer it from clone count alone.
3. Parse every required count as canonical decimal/u128. Require actual required
   clone results for tracked regulars, zero file/byte copies, expected link installs;
   reject null required coverage/effects. Clean ordinary requires satisfied plus
   actual zero Effects, not just no different observations. Keep deviated and
   retained distinct and preserve optional null timings/diagnostics.
4. Extract timing keys exactly from §3. Retain source diagnostics once with native
   declared versus ordinary paired/opaque attribution. Missing diagnostics are
   unavailable evidence, not zeros or proof of sharing; any source-work comparison
   requiring them must refuse that comparison. Physical fixture verification stays
   independent: JSON alone cannot prove inode privacy, history, bytes or final state.
5. Preserve process failure first; optionally decode its valid failure envelope for
   evidence without treating parse success as command success. Semantic acceptance
   and timing remain blocked on any failed required condition.

**Measurement profile decision:** retain this adapter's existing `--source-at
exact(S)` and name its profile **`live-asserted-source`** in initialized workload,
invocation evidence and acceptance binding. Check
`source.evidence == "live_assertion"`, assertion succeeded and its exact ID=S.
Its command time includes the named live donor diff/snapshot. This is a deliberate
asserted-source route comparison, **not plain prepared-source latency**. Do not
subtract it to manufacture plain latency or silently remove it from existing
measurements. A later separately named `assumed-prepared-source` profile would
omit CLI **and configured** source_at and require assumed_prepared; it would need
its own acceptance binding. No such additional campaign is commissioned here.
Public plain reseed remains free of implicit donor snapshots.

After native review and public cutover, the existing bounded recipe is: preserve
one candidate/build-feature provenance, initialize one donor/two reused receivers,
run `renewal accept` for clean N1, paired N2, mixed N2 and D≠S N2 on both routes,
bind the successful receipt to the exact candidate/tools/adapter/profile, and
only then run the existing balanced native samples or separate instrumented count
probes when commissioned. Reset/hashes/external verification stay outside process
latency; phase spans are supplementary. This design runs none of those commands.

Meaningful cutover acceptance cases:

- Actual public N1 and N2 dispatch reaches the same cohort; both route values and
  omitted/default route work. Complete coverage gates adoption even on zero-leaf
  manifests; a cleanup failure after all installs still fails without adoption.
- JSON success, source failure, native second-lane partial failure, ordinary
  failure after writes, link failure after creation, and command-global final
  barrier failure each retain their real prefixes/causes and emit exactly one
  parseable document before nonzero exit. Unstarted work remains null.
- Early cwd/config/source/profile resolution errors produce zero resolved rows
  without losing requested names; byte-named failure paths remain lossless.
  Test counts beyond JavaScript's safe-integer range and u64 without numeric coercion.
- Standalone prepare has no receiver discovery/payload work; S≠goal is reported
  correctly. Two successful commands may edit then repair tracked content; a
  failing command stops successors; final divergence fails with actual step state.
- Plain/asserted/preparing previews leave repository operation/WC/config/lock/
  fixture bytes unchanged and invoke none of snapshot/live diff/fetch/builder/
  membership/clone probe/transfer. Source and D pending states differ correctly.
- Ordinary deviation and retention have discriminating unequal values; clean
  required tracked renewal still reports installs while clean ordinary Effects
  are zero. D≠S reports installations in S without claiming every final D leaf shares.
- Parser fixtures reject missing/null required effects, wrong S/D/route/profile,
  incomplete coverage and numeric rather than decimal counters. Optional timing/
  diagnostics null survives extraction. No remaining CLI caller reaches old Plan;
  creation behavior remains exercised through its retained helpers.
