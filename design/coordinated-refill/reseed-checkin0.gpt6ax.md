---
type: ArchitectureAssessment
title: Reseed performance check-in across jj-waltz and diff-dirs
description: Product and architecture assessment of the shipped reseed, callable replacement and library, with a cost model and evidence-gated next decisions.
resource: /design/coordinated-refill/reseed-checkin0.gpt6ax.md
status: parent-assessment-awaiting-independent-returns
generated: { by: "model:openai/gpt-6-astra#xhigh", at: 2026-10-28 }
sources:
  - resource: "opencode://ses_e7eb31c84ffev4GUeYjdmZO7mY"
    title: Human commissions an architectural and performance check-in of both projects after the coordinated rebuild; parent reconstructs current adoption, runtime identity and historical measurements
  - resource: /design/coordinated-refill/README.md
    revision: 76fb0656
    title: Actual delivered slices and the held CLI/product checkpoint
  - resource: /design/coordinated-refill/architecture0.gpt6a.md
    revision: 76fb0656
    title: Native/ordinary phase separation, real traversal costs and proposed cohort loop
  - resource: /design/coordinated-refill/assessment1.gpt6a.md
    title: Recorded human direction to reconsider lifecycle promises and prioritize frequent reseeding
  - resource: /design/coordinated-refill/implementation1.sol61x.md
    title: Callable N1 sequence, native snapshot ledger and attributed required-CoW evidence
  - resource: /design/coordinated-refill/implementation2.sol61x.md
    title: Conditional routing, diff-dirs 0.2 adoption and explicit opaque-recursion sharing limit
  - resource: /design/coordinated-refill/review0.gpt6ax.md
    title: Earlier pinned native review and consequential product questions; its two path defects were subsequently corrected
  - resource: /refill/engine/src/engine/replacement.rs
    revision: 76fb0656
    title: Strict full-S installation, native convergence, ordinary cursor and finish ordering
  - resource: /refill/engine/src/engine.rs
    revision: 76fb0656
    title: Current legacy command pipeline and preparation-only lock
  - resource: /app/cli/src/cli.rs
    revision: 76fb0656
    title: Public reseed dispatch remains on run_with_diagnostics
  - resource: /design/up-performance/primary0.sol61m.md
    title: Cancelled historical large-workload campaign and retained successful samples
  - resource: /design/up-performance/integrated-controls0.sol61m.md
    title: Historical source/build identities and bounded synthetic controls
  - resource: /tools/performance/README.md
    title: Existing native measurement and count tooling, including fixture costs and attribution limits
  - resource: /.test-agent/primary-reseed/clean-n2-serial/summary.json
    title: Rechecked retained two-pair native timings and completion status
  - resource: /.test-agent/primary-reseed/clean-n2-serial/samples.jsonl
    title: Rechecked sample timings, CPU, inode preservation and setup/verification costs
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/README.md"
    revision: 10361cf0
    title: Current production effects, continuation and conditional-container contract
---

# Reseed performance check-in

## Judgment

**Keep the native/transfer separation and the production diff-dirs core. Reassess
what frequent reseed promises before treating the current full-S native route as
the universal fast path.** The architecture is becoming much more explicit and
useful, but its main new source-to-many benefit is not yet delivered through the
public command. The immediate leverage is command-wide reuse and a deliberate
native materialization contract, not another general framework.

This is an assessment, not selection of new product defaults. The parent recorded
this position before reading the two independent check-in returns. Their eventual
integration belongs below, with any revisions to this judgment made explicit.

## Basis: three different things currently exist

| Layer | Current basis | What it establishes |
| --- | --- | --- |
| Public `jw reseed` | JW code `76fb0656`; `app/cli/src/cli.rs::run_reseed` calls `run_with_diagnostics` | The older orchestration remains the actual command path. It already uses diff-dirs effects, but is not the new coordinated command. |
| Callable replacement | Same JW code, `refill_one_from` / `engine/replacement.rs` | Executable strict N1 native refill plus source-qualified ordinary transfer through the coordinated library. The current seam requires one receiver. |
| Library capability | diff-dirs `10361cf0`, exact `=0.2.0` path dependencies | Multiple lanes, partial-frontier continuation, production effects and deferred source-only routing exist. This does not prove JW cohort adoption or whole-command speed. |
| Installed tools | `jw 0.8.0`; JJ `0.45.1-33d3a595…`; Rust `1.101.0-nightly 822182db4` | Read-only version checks. No workspace operation was run for this assessment. |

The installed `jw` resolves through mise to `/home/rektide/.cargo/bin/jw`. Its SHA-256
is `00ae8cb62630f1ccdfd9a27ee36f35708367daed44ced305908a31d2ec5a5652`, identical
to the current `target/release/jw`. Cargo records a release installation from
`app/cli`, with no optional features, but no source revision. That is a real
artifact identity, **not proof that the installed executable was built from the
latest checkout**. JJ resolves through `/usr/local/bin/jj` to the archived build,
SHA `0b7462db466ffb0394cb30b0bb56480da3bce10a91f98b80b109302b0ec610d8`.

The two starting uncommitted JW drafts were preserved. In particular, the older
lock-contention account is historical: current `engine.rs:340–343` takes
`PreparationLock` only around explicit preparation. Its former whole-reseed
global-lock diagnosis must not be carried into this assessment as a current defect.

## What diff-dirs is doing well

1. **It owns substantial, reusable filesystem behavior.** Staging, required CoW,
   literal links, timestamps, permissions, exact removals and returned partial
   effects stay in production kernels rather than being reimplemented by JW.
2. **The consumer still owns meaning.** JJ membership, receiving identity,
   snapshot/cache semantics, source-qualified authority and link ownership remain
   JW responsibilities. Pure policy consumes observations instead of running native
   commands from the traversal callbacks.
3. **The frontier represents real partial traversal.** Per-lane memberships and
   pending application decisions support different destination heads without a
   source-tree catalog. JW can use the same machine as an in-process iterator;
   persistence is a capability, not a requirement to add command recovery.
4. **The latest missing-route work is a good library extension.**
   `CreateIfNeeded` expresses a generic effect: traverse a source-only routing
   directory, materializing it only when selected descendants require it. JW's
   wildcard/ignore rule remains outside the library. Avoiding an empty skeleton
   does not require a prewalk or create-then-prune behavior.

The principal caveat is granularity. Source sharing covers participating **paired
directory groups**. A large subtree absent in several targets is copied through
separate opaque calls. Deferred routing preserves that boundary; it does not make
absent interiors source-once or independently pausable. This is a sensible first
implementation, with a clear future pressure if missing-subtree work dominates.

## The native promise is the largest architectural decision

The new sequence is not an incremental tracked update. `replacement.rs:225–259`
clears tracked content, establishes the new receiving child, strictly installs
every S leaf, adopts the donor's tree state, and establishes D. With D≠S, it also
performs native S→D restore. That deliberately renews tracked CoW sharing.

The implementation account records three full receiver snapshot boundaries for
D=S—preserve, sparse clear, final diff—plus the empty-sparse `new` snapshot. D≠S
adds restore's adopted-S snapshot and desired-tree materialization. These are
meaningful native costs even when JJ's stat cache makes individual checks cheap;
they are not automatically full byte rehashes.

Two useful products should be distinguished before choosing their default:

| Outcome | Plausible upfront route | Cost/contract consequence |
| --- | --- | --- |
| Refresh to D and reconcile selected ordinary data | Native update/conformance plus incremental ordinary transfer | Could preserve already-correct tracked files, but does not promise to renew their donor extent sharing. Needs an actual protocol/proof. |
| Renew donor sharing and establish D | Current strict full-S route, especially coherent at D=S | Intentionally pays per-tracked-leaf reinstalls; CoW renewal is part of success rather than an incidental optimization. |
| Establish distant D efficiently | Compare current full-S adoption with the existing equal-leaf/native route | Full-S can install S-only files only to remove them, or changed files before native replacement. Any alternative must retain its own authority and sharing contract. |

My recommendation is to keep strict renewal as a first-class supported operation
and evaluate a less disruptive frequent-refresh route against it. **Do not silently
skip equal tracked files while continuing to promise renewed sharing.** Likewise,
an S-file reflink count is not a count of sharing retained in the final D tree.

There is also a larger possible investment: a JJ-native materialization operation
that can combine receiving identity, donor tree-state knowledge and tracked update
more directly. That belongs behind `native/jj`, with explicit version/support and
cache obligations. It should earn its cost through evidence that stock-command
snapshots and reinstall work dominate; moving those details into diff-dirs would
give the filesystem library the wrong owner responsibility.

## Make the unit of work a prepared source and a selected batch

The next command integration should build one command-owned context: source S,
distinct desired trees D, frozen source/desired membership and ignore indexes,
resolved workspace inventory, loaded configuration/links and chosen native routes.
Run serial native prefixes initially, then form one ordinary-data cohort from
native-ready destinations. Do not implement N2 by invoking the entire N1 helper
twice: that repeats both native input capture and ordinary source discovery.

This also matters for N1. Generic context discovery currently re-queries repository
and workspace facts; native verification occurs before and after ordinary/link
finish. Preserve the checks that establish successful identity, but carry the facts
whose validity interval spans those stages. Share membership **indexes**, not merely
the underlying vectors, and release native-only inputs when no longer needed.

The harder workload question is whether real usage provides batches. Several
independent agents each issuing N1 commands cannot obtain in-process source sharing
just because the library supports N2. Prefer explicit batching at the orchestration
level when available. If repeated N1 invocation is the dominant pattern, consider
an explicit prepared-source generation supplying reusable native/ignore inputs.
Tracked S alone cannot identify an installed ignored payload generation. Such a
producer contract would be JW-owned; it should not become an implicit freshness
cache or recreate the discarded per-file recovery journal.

## Performance model: source-once is one term

For N independent target operations, separate:

```text
root/config/native-query work
+ native preservation/materialization/conformance for each destination
+ common source acquisition
+ per-destination listing, policy and effects
+ finish
```

For the same paired region, coordination replaces roughly `N × source acquisition`
with one acquisition, while target I/O, lane decisions and effects remain. Its net
saving must exceed coordination, decision-storage and extra routing costs. In the
idealized balanced source/target-observation-only case, the change is `2N` equal
costs to `N+1`, not an N-fold whole-command speedup. Native work and opaque effects
further reduce the fraction available to that optimization.

Measure matched existing payloads, missing payloads and mixed destination shapes
separately. The existing steady-state workload is the strongest case for paired
sharing; initial fills and missing wildcard routes exercise opaque recursion.
Policy is pure but still repeated per lane: selector prefix replay, frozen-ignore
ancestor searches, native membership and lane-slot allocation deserve CPU/allocation
evidence alongside syscall counts. More filesystem parallelism will not necessarily
fix those costs.

## What the evidence does—and does not—say

| Claim | Evidence inspected | Standing |
| --- | --- | --- |
| Incremental payload preservation helps this workload | Historical native samples retain all 171,567 regular payload inodes per target while the old full-copy baseline rewrites them | Strong mechanism evidence; old implementations, not current coordinated timing. |
| Large-workload latency improved in those samples | Retained serial N2 pairs: 108.31→21.15 seconds and 99.11→17.64 seconds | Two matched warm/cache-uncontrolled pairs, rechecked in raw JSON; no extrapolation to the new route. |
| New N1 native protocol establishes its promised result | Required-CoW integration account: actual JJ chronology, FIEMAP, private-write and native identity checks | Attributed existing test evidence; this assessment did not rerun it. |
| Conditional routing works without pre-created skeletons | 0.2 integration account and focused required-CoW cases | Implemented and tested at the named checkpoint; not a new cohort performance result. |
| Current new whole-command N2 is faster | No matching evidence; public cutover/cohort still pending | Open. |

The old large campaign's fixture verification cost 312 seconds before repeated
sample work; later sample setup cost hundreds of seconds and the operator cancelled
the campaign under disk pressure. Do not repeat that experiment design merely to
obtain a more impressive fixture. The retained samples are useful, but current
decisions need a cheaper, bounded method.

## Recommended next check-in gates

1. **Agree the native outcome:** routine refresh versus mandatory sharing renewal,
   with D=S and D≠S treated explicitly. Keep source-qualified auxiliary selection
   unless product intent selects different source-absence behavior.
2. **Build the actual command context/cohort boundary**, including N1, without
   copying the entire old planner or making N calls to the N1 prototype. Demonstrate
   two differently shaped receivers and a native/ordinary/finish failure trace.
3. **Account before tuning:** counts of native commands and snapshot-triggering
   operations, repository enumeration, shared source observations, route checks,
   opaque work and actual installs. Existing phase timers are useful but not a
   substitute for this attribution. Native timings and instrumented counts are
   separate measurements.
4. **Use a small discriminating workload matrix**, then a bounded representative
   sample: N1/N2; D=S/D≠S; clean/sparse/missing selected roots; narrow/broad selectors.
   Reuse owned fixtures, verify setup separately, and predeclare a fixture/storage
   budget. Compare like-for-like authority and native outcomes.
5. **Let residual cost select library work:** optimize route checking or decision
   storage if dominant; consider source-shared opaque traversal only if it matters
   materially. Generalized pause/recovery, a service daemon and broad policy
   frameworks are not prerequisites for this decision.

## Connections

- [Current integration index](README.md) owns the delivered/held slice boundaries.
- [Earlier native review](review0.gpt6ax.md) anticipated the full-renewal and
  source-authority choices; its two reproduced path bugs have since been corrected.
- [Technical architecture](architecture0.gpt6a.md) supplies the proposed command
  context and cohort shape. This check-in challenges the universal-route assumption
  and establishes how to evaluate its payoff, rather than automatically selecting
  the next implementation slice.
