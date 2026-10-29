---
type: TechnicalPlan
title: Forward trajectory — from callable refill to an everyday command
description: Refined delivery sequence, command-owned interfaces, product decision gates, bounded evidence and conditional optimization work for completing the new reseed.
resource: /design/coordinated-refill/trajectory1.gpt6ax.md
status: proposed-forward-plan
generated: { by: "model:openai/gpt-6-astra#xhigh", at: "2026-10-29" }
sources:
  - resource: "opencode://ses_e7c8f24ebffelGRFC0RXsQvf61"
    title: Human requests a forward trajectory in two writing stages; parent preserves a broad brain dump, then refines it into a concrete delivery plan with explicit product choices and evidence gates
  - resource: /design/coordinated-refill/trajectory0.gpt6ax.md
    revision: 0fcebdfc
    title: Preserved broad first pass, narrowed by this technical plan
  - resource: /design/coordinated-refill/reseed-checkin0.gpt6ax.md
    revision: a4df2bf2
    title: Integrated cross-project review, native outcome fork, current live-cursor costs and bounded measurement recommendations
  - resource: /design/coordinated-refill/reseed-checkin0.gpt61sxh.md
    title: Consumer-side pass ledger, sharing limits and whole-command completion requirements
  - resource: "file:///home/rektide/src/diff-dirs/design/fit/jw-checkin0.gpt61sxh.md"
    title: Library cost attribution and alternatives to the current live execution shape
  - resource: "file:///home/rektide/src/diff-dirs/doc/consumer/jj-waltz.md"
    title: Provider-owned connection for returning consumer evidence and improvement requests
  - resource: /design/coordinated-refill/review0.gpt6ax.md
    title: Repeated root discovery, redundant native indexes and independent product decisions
  - resource: /design/coordinated-refill/implementation0.gpt6ax.md
    title: Detailed native protocol and streaming authority mechanics to retain where this plan chooses the renewal route
  - resource: /design/coordinated-refill/implementation2.sol61x.md
    title: Completed conditional-route capability and remaining cohort/CLI handoff
  - resource: /design/coordinated-refill/corrections0.sol61x.md
    title: Completed path corrections and regression boundaries to preserve
  - resource: /refill/engine/src/lib.rs
    title: Current public request/report APIs and old-driver dispatch beside the temporary N1 export
  - resource: /refill/engine/src/engine/replacement.rs
    title: Current callable flow and phase reports to decompose into shared command inputs and per-receiver execution
  - resource: /app/cli/src/cli.rs
    title: Actual reseed/prepare dispatch and result rendering integration points
  - resource: /tools/performance/src/build.rs
    title: Repaired source-input and artifact provenance selection for controlled comparisons
---

# Forward trajectory — from callable refill to an everyday command

## 1. Recommendation and decision boundary

**Deliver one real replacement command around a shared source/batch context, then
let measured residual cost choose the next native or library optimization.** The
first whole-command milestone includes N1, a heterogeneous N2 batch, explicit donor
preparation, honest preview, useful partial-failure reporting and removal of the
superseded reseed driver.

This refines the [brain dump](trajectory0.gpt6ax.md), rather than replacing the
native/authority mechanics already documented in the
[implementation plan](implementation0.gpt6ax.md). It incorporates the newer
[cross-project check-in](reseed-checkin0.gpt6ax.md). It proposes the trajectory;
writing it is not renewed authorization to cut over the CLI or change defaults.

### Proposed first-release contract

Use the following as the working baseline for detailed planning:

| Concern | Recommended first-release outcome |
| --- | --- |
| Reseed's native job | Renew tracked donor sharing and establish a receiving child of D; normal D=S. |
| Explicit D≠S | Use the already-proven full-S/adopt/restore route initially, with its extra work and local-obstruction restrictions stated. Optimize a different upfront route only if actual use warrants it. |
| Ordinary data | Retain implemented source-qualified selection. Empty/disabled selection copies nothing; vanished donor roots retain the corresponding target root; stale descendants inside admitted mirrors are removed. |
| Storage claims | Newly installed leaves use required private reflinks. Retained stat-equal ordinary leaves are retained as supplied, not certified/repaired for inode privacy or extent sharing. |
| Failure | Stop on the first surfaced unexpected error, report its phase/cause and known effects, and retain completed work. |
| Donor | Consume a prepared, idle source without a default live resnapshot. An explicit assertion or producer has its own named native work. |
| History | Retain the current coverage gate/bypass for this first integration, unless the human selects its removal. Save/report old tracked work; no automatic bookmark creation. |
| Preview | Nonmutating root/native plan, clearly not a promise that payload/path obstructions have been inspected. |

**The material unanswered question is the first row.** If everyday use is primarily
refresh rather than renewal, decide that before committing the public native default.
The shared context, source-qualified policy, cohort and reporting work below remain
useful either way. The refresh branch substitutes a separately proved native
transition; it does not silently skip required renewal work or build every possible
mode at once. These documents still contain a recommendation, not the operator's
answer to that question.

The other rows constrain the proposed first delivery. Broader deletion authority,
automatic repair of retained aliases and a permanent donor cache service are
independent product choices. They need not block the recommended source-qualified
renewal milestone.

## 2. Starting point and end condition

The six-crate foundation, native N1 seam, immutable ignore matching, conditional
routing and reviewed path fixes are delivered. Public `run_with_diagnostics` still
dispatches to the old engine. `refill_one_from` accepts one receiver and rejects
prepare/dry-run; `sync_one` binds one lane and runs sequentially.

The end condition is **one production path** serving both N1 and N2 through the
actual CLI, with preparation/preview semantics and documents matching the code.
A library bump, a second internal helper or an adapter-only benchmark does not
satisfy that condition.

## 3. Command shape and data lifetimes

Proposed responsibility names below are design coordinates, not mandatory public
types. Keep them as concrete structures/functions within the approved owners.

```text
request + configuration
    │
    ▼
resolve bindings and declarations ── preview → render PLANNED and return
    │
    ├── explicit producer → PreparedSource
    └── consume existing prepared source
    │
    ▼
select/pin S, D and supported native route
    │
    ▼
save receiving work and capture required native inputs once
    │
    ▼
serial native transitions → native-ready receivers
    │
    ▼
one ordinary transfer cohort → policy-satisfied receivers
    │
    ▼
loaded links + relevant native finish checks → complete report
```

Any phase can return an error. The diagram is a forward function, not a persistent
workflow system. The preview branch describes prospective preparation when it was
requested; it never invokes that producer or claims its resulting S is known.

### Root resolution

`ResolvedBindings` owns canonical selected roots, repository/primary relationships,
workspace identities, source choice, relevant topology/native profile and loaded
configuration/link declarations. It does no payload census, `file list`, recursive
audit or implicit snapshot to prove readiness. Some native metadata reads deserialize
tracked state; account that cost instead of calling it constant-time.

Refactor `Inventory::load`, `checked_context`, `validate_target` and the finish queries
around these supplied facts. A caller/donor/primary that is the same root should not
trigger three independent primary-discovery walks through the registry. With R
registrations and N selected receivers, avoid N repetitions of the R-root discovery.
Target-specific binding/ownership checks remain target-specific.

A useful validity distinction is **root/registration identity versus revision state**.
Most root facts survive the command under the quiescence premise. Revision facts
must be reread when native commands deliberately change them. Do not cache the latter
under the former's lifetime or rediscover the whole root context to obtain a new
commit ID.

### Producer handoff

`PreparedSource` carries the source binding, actual resulting S, upstream goal
separately, supported native profile and already-established preparation result.
Membership/cache inputs can be acquired lazily when refill needs them. Standalone
prepare returns the producer result; combined reseed consumes it directly.

For plain reseed, resolve recorded S without live proof. For a requested live
assertion A, perform that named diff once and pin the resulting S. Resolve explicit
D against the intended post-preparation repository view; without `--at`, D=S.
After receiver preservation, check the donor coordinate once: native snapshots can
rewrite related repository history. A moved S fails before clearing rather than
silently adopting a new goal.

### Shared native inputs

`NativeInputs` owns the captured donor cache, membership vectors and useful indexes
for each distinct S/D/O tree, frozen control-file bytes and lane-specific effective
exclude contexts. All receivers in today's invocation have the same selected D.
Their old revisions and effective exclude contexts can differ.

- Obtain required membership once per distinct tree/equivalent native coordinate.
- Build JW's reusable S membership, S/D ownership union and D-addition indexes once,
  rather than cloning path sets in each receiver helper.
- Reuse source/desired ignore compilation only when both native rule inputs and
  effective excludes agree; sharing S alone does not establish that equivalence.
- Retain saved O coordinates for reports, but release O membership and donor cache
  bytes after the native phase. Keep only ordinary-policy indexes that transfer needs.
- The existing library listed installer still collects/sorts its supplied inputs
  per call. Record that remaining cost; a reusable seed-list library interface is
  a later targeted improvement if it matters, not a reason to invent a new copier.

These are native repository enumerations and indexes, not a filesystem payload
inventory. They are real work and belong in the native pass ledger.

### Per-receiver execution

Extract the current N1 native body into an operation accepting shared inputs and
one receiver's saved native state. It returns the established child identity,
tracked-install/clear effects and phase timing. It does not resolve configuration,
recapture S or construct an ordinary runtime.

Initially keep native operations serial. Complete a receiver's native transition
before starting the next; do not sparse-clear all receivers together. Once all
selected receivers are native-ready, create one ordinary cohort. If a native step
fails, stop: already changed receivers remain visible as partial/native-ready,
not completed and not automatically unwound.

The selected full-S route keeps its existing three full receiver snapshot
boundaries for D=S, four for D≠S, plus empty-sparse `new`. Merely reorganizing these
calls does not remove their work. A different native route requires its own result
and ownership proof, not an error-triggered fallback.

## 4. Cohort transfer and failure reporting

Replace `sync_one`'s fixed vector with a batch API taking one source and a vector
of receiving bindings plus frozen policies. Build **one `SyncRuntime` and one
`SyncCursor`**, even for N1, using the existing explicit fail-fast loop:

```text
next_group
→ surface a lane inspection error, if any
→ apply every pending lane using that group's token
→ settle
→ repeat to exhaustion
```

N2 is not a loop around N1. Paired source listings/facts are shared by the actual
cohort; target observations, policies and effects remain per receiver. Source-only
copy and conditional-route recursion remain opaque per-lane work. Apply pending
groups even when immediate work is zero: that is not a recursive no-effect proof.

Use one concrete visitor type with per-lane values and shared immutable source
inputs. Preserve the two path corrections and the source-qualified reachability
argument through opaque copy/removal/replacement. No mutable discovered-root map
or prewalk is introduced while generalizing to several destinations.

### Result design

The current `RefillOneReport` and public `ReseedReport` should converge on a compact
command report, with a failure carrying that report plus the original cause.
An ordinary `Result<Report, Failure>` is sufficient; there is no reason for a
durable effects ledger or a new event framework.

Per receiver retain:

- identity, saved O and desired D, final child coordinate when established;
- reached phase and completion/failure/not-started status;
- actual tracked install and explicit-clear work;
- ordinary coverage/satisfaction and actual recursive effects;
- link result and useful timing.

Overall wall time is recorded once. Do not sum lane durations as if they were
independent command durations. Partial reports are read from already-returned work,
not reconstructed through another filesystem scan. Render the failure's workspace,
phase, path and cause, plus concise completed-work counts; avoid a default dump of
every retained directory/decision.

Keep four concepts distinguishable: immediate potential obligations, recursively
resolved plans, completed filesystem effects and native correctness. Most normal
output needs the latter two plus phase. A completed ordinary lane is not a completed
workspace until its requested finish succeeds.

## 5. Preparation, preview and CLI cutover

Extract source preparation from `run(request, source_only)` into a source-only
producer, retaining upstream selection and its existing preparation-only lock.
It does not enumerate target inventories or realize selected payload roots.

Recommended preparation contract: configured argv steps run in order, and the
successful sequence must leave the intended final tracked source result. Use one
final tracked conformance check rather than a full proof after each argv step.
That allows an intermediate tracked change restored by a later step: document and
test the semantic change, rather than presenting it only as a speed optimization.
Keep ordinary command failure and relevant native identity checks.

The cheap `--dry-run` branch reports roots, selected routes, recorded revisions and
declared transfer scope, with explicit unknowns. It performs no snapshots, writes,
clone probes or recursive payload planning. Label it `PLANNED`, with payload/path
obstructions uninspected. An assertion is pending live validation; a prospective
prepare result is unknown. Detailed recursive preview, if wanted, is a separately
requested inspection; it is not a dependency of ordinary execution.

Wire `app/cli::run_reseed`, `run_prepare` and their renderers to the new operations.
On failure, render known partial outcomes before returning nonzero; JSON stdout
must remain a complete documented result, with diagnostics on stderr. Update any
public report schema and pre-1.0 version marker deliberately with changed behavior.

Retire the temporary `refill_one_from` export and old reseed driver in the coherent
cutover. Delete superseded root realization, repeated donor proof and future-content
prediction code once unused. Keep helpers genuinely used by creation with their
owners; their existence does not justify preserving a second public reseed engine.

Before rollout choose a usable native support profile. At minimum, exercise the
operator's actual JJ build and state the scope honestly. The current semantic
0.45.1 equality check is neither permanent policy nor exact-build certification.
Widen support with focused protocol evidence as needed; do not claim all advertised
lifecycle versions automatically support the new private-state route. Known
unsupported setup fails up front; later I/O errors still fail normally.

## 6. Build sequence and acceptance gates

These are coherent delivery units, with no calendar estimates. The remaining
implementation starts only when commissioned after the product discussion.

| Unit | Implemented result | Gate |
| --- | --- | --- |
| **A. Fix the first-release job** | A short selected outcome table based on section 1, especially renewal versus refresh; a supported native profile and honest storage claims. | The implementation can name the promised result without implying a stronger sharing/deletion guarantee. If refresh is selected, substitute a native protocol proof rather than building both products by default. |
| **B. Share the command inputs** | Root resolution and revision/native capture split; shared memberships/indexes/ignore inputs; native receiver operation extracted from N1. | Instrumented N1/N2 fixtures show one command-owned registry/config resolution and no duplicate S/D extraction; actual target-specific work remains visible. Existing native result/obstruction behavior holds. |
| **C. Complete one real cohort** | New callable N1/N2 flow, one transfer runtime, loaded forward finish and compact success/partial-failure result. | Two differently shaped receivers converge; a surfaced error leaves honest earlier effects and stops further admission. The test actually observes shared source inspection, not just matching effects. Parent reviews the complete execution boundary. |
| **D. Replace the command** | Source-only producer/handoff, nonmutating preview, public dispatch/rendering, docs, supported-profile behavior and old-driver retirement. | Real CLI tests establish prepare-once handoff, preview without mutation, default/explicit D, scope and forward failure. Public command and documentation agree. |
| **E. Establish useful cost and release** | Matched artifact/phase evidence, demonstrated corrections, release candidate and operator guide. | Evidence identifies native versus transfer cost, clean-repeat writes and N1/N2 behavior; supported ordinary use is workable. Install/deploy is deliberate, not an incidental build test. |

Units B/C have shared value independent of the renewal/refresh choice. Their input
reuse can be developed without inventing a new public default. The public native
route and its acceptance tests depend on A. D consumes C; E measures the actual
integrated route rather than extrapolating from an adapter.

The next implementation commission should encompass B–E under the selected A,
with review at the working cohort and public-command boundaries. Do not repeatedly
return after a small helper while leaving the everyday command undelivered.

## 7. Bounded evidence, not another campaign

Use existing native/transfer tests and the repaired Rust tooling. Earlier passing
checks are evidence; rerun only affected tests and an appropriate integrated set.
The known baseline colocation/removal failure is already attributed and should not
be disguised as either a new regression or a new-route success.

### Semantic and cost discriminators

| Case | What it must discriminate |
| --- | --- |
| N1 D=S, clean repeat | Exact native result, required tracked renewal, zero clean ordinary effects; native snapshot/command ledger. |
| N1 D≠S | Final D and identity, temporary S installs versus final sharing, local-obstruction restrictions and extra native work. |
| N2 both paired, one clean and one changed | Real source-fact sharing, independent effects and one set of shared native inputs. |
| N2 one missing selected subtree, one paired | Correct conditional routes and per-lane opaque cost; no claim that missing interiors are source-once. |
| Explicit preparation and preview | One source producer with consumed handoff; no donor reproof in refill; preview has no writes/snapshots or false READY result. |
| Failure after visible work | Known native/ordinary/link effects retained, failing phase/cause preserved, no silent fallback or whole-workspace undo. |
| Stat-equal ordinary copy and source hardlink | The difference between retained equality, inode privacy and extent sharing. Accurately describe the result; do not automatically add repair authority. |

For initial semantic cases cap the total fixture at 256 ordinary entries and 1 MiB
of ordinary contents including staging. Use the existing small native-file fixtures
and real required reflinks. `/usr/local/bin` precedes mise shims; discovery tests do
not receive a blanket project-local TMPDIR. A few discriminating cases are better
than multiplying every option into a combinatorial matrix.

For timing, freeze baseline/candidate source and binary identities with the existing
build tool. Keep transfer policy, native result and supported input shape matched.
Use one owned donor and at most two receivers at a time. Reuse them for clean runs;
reset only named small deltas for changed cases. Predeclare a short balanced run
sequence. Record setup/checking separately. Native timings and instrumented counts
are separate observations; neither wrapper overhead nor fixture creation is silently
called reseed latency.

Start with N1 clean, N2 paired-clean, and N2 mixed missing/paired as separate questions,
not a large all-shapes campaign. Tiny timing is a smoke/control result. Before a
representative claim, select one representative workload question with an explicit
resident-data/run budget; avoid repeated full-workload imports or hashing several
copies for each sample. A result showing no improvement is useful evidence to change
the implementation choice.

Count/attribute: native process calls and snapshot boundaries; repository membership
and control-file extraction; source versus target acquisition; comparison-route
versus application-route checks; policy/decision CPU or allocations where measured;
actual installs/removals; wall/CPU time and high-water memory. Existing counters
cover only parts of this budget—name their scope rather than treating one source-stat
count as all source work. Add small test/benchmark instruments only where needed.

## 8. Conditional follow-on work

The following are ranked candidates, not mandatory prerequisites to the command.

| Measured residual cost / actual need | Next coherent work | Owner |
| --- | --- | --- |
| Root queries or repeated native index construction | Reuse command-owned bindings and immutable indexes further; batch actual queries. | JW |
| N1 clean live cursor overhead | Compare the existing paired driver under the same visitor/effects; investigate lazy checkpoint projection, lighter group views and bounded live-route reuse in the same library machinery. Select execution upfront, never as an error fallback. | JW experiment, diff-dirs mechanism |
| Repeated policy prefix/ignore work | Improve immutable query/index representation and allocations; any cached result must depend only on frozen inputs, not callback order. | JW first |
| Both-missing or mixed opaque source work dominates | Share bounded source observation batches within effect execution while retaining per-destination staging/permissions/failure and opaque ownership. | diff-dirs |
| Native renewal work dominates the real daily job | Revisit refresh versus renewal; compare one explicit native route at equal stated intent, or a JJ-supported materialization seam if worth the integration cost. | JW/native JJ |
| Retained aliases/unshared leaves must be repaired | Select the stronger storage contract, then add explicit install authority rather than manufacture a comparison difference. | Human/JW policy, library effect |
| Real heterogeneous receivers suffer barrier latency | Consider bounded opportunistic cohorts only after measuring waiting and retained memory. Stopped lanes are abandoned, not suspended. | diff-dirs |
| Independent N1 commands dominate usage | Improve explicit batching first; only then assess a prepared-generation handoff with a real ignored-payload lifetime contract. | JW/orchestrating caller |

Checkpoint export/restoration must retain its existing exact-decision semantics if
live projection is optimized. `planned == 0` is not a valid effect-free shortcut.
Source-only sharing should not give read-only paired comparison a second recursion
owner. These are concrete constraints on the next experiment, not an invitation to
build a universal scheduler or persistent donor catalog.

Feed actual outcomes and selected follow-on requests back to
[diff-dirs' consumer account](file:///home/rektide/src/diff-dirs/doc/consumer/jj-waltz.md).
Keep source-side wishes distinct from accepted upstream work.

## 9. What this refinement keeps and postpones

| Brain-dump direction | Disposition in this plan |
| --- | --- |
| Finish something useful | Critical path is the real CLI, including producer/preview/failure behavior and old-driver removal. |
| Make the source/batch the work unit | Immediate B/C design, benefiting N1 as well as N2. |
| Reconsider tracked renewal | Explicit A decision; working recommendation is renewal, not an assumed human ruling. |
| Improve live library cost | Evidence-gated next investment, with existing pair execution as a control and shared kernels retained. |
| Rework ordinary authority | Keep the implemented source-qualified contract for this milestone; broaden only after explicit selection. |
| Persistent generations, independent scheduling, seed reuse | Later work driven by use; none is a prerequisite to completing frequent reseed. |

The trajectory is therefore **select the job → finish the shared command → establish
its real cost → improve the dominant remaining term**. This advances the product
without either canonizing the prototype or restarting architecture from scratch.
