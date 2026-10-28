---
type: ArchitectureAssessment
title: Reseed check-in — choose the job before completing the cohort
description: Independent source-derived assessment of shipped versus callable reseeding, native and transfer costs, sharing promises, architectural alternatives and bounded discriminators.
resource: /design/coordinated-refill/reseed-checkin0.gpt61sxh.md
status: independent-checkin-not-implementation
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: "2026-10-28" }
sources:
  - resource: "opencode://ses_e7b386178ffe35om7FsR11rYky"
    title: Independent architecture check-in from current sources and existing evidence; no builds, tests, benchmarks or native workspace mutations
  - resource: "opencode://ses_e7eb31c84ffev4GUeYjdmZO7mY"
    title: Parent commission, baseline and independently supplied read-only host executable provenance
  - resource: /CONTEXT.md
    title: Domain vocabulary and current versus historical lifecycle obligations
  - resource: /docs/reseed.md
    title: Public operator behavior, copying scope, preparation, recovery and historical verification
  - resource: /docs/architecture.md
    title: Six-crate ownership and distinction between delivered and proposed execution
  - resource: /design/coordinated-refill/README.md
    title: Explicit callable N1 boundary and held coordinated CLI cutover
  - resource: /design/coordinated-refill/assessment1.gpt6a.md
    title: Human direction toward whole-command redesign, upfront routes and ordinary forward failure
  - resource: /design/coordinated-refill/implementation0.gpt6ax.md
    title: Planned native protocol, sharing ledger, policy and subsequent cohort/preparation work
  - resource: /design/coordinated-refill/implementation1.sol61x.md
    title: Attributed native N1, command chronology, real-CoW and authority evidence
  - resource: /design/coordinated-refill/implementation2.sol61x.md
    title: Delivered conditional routing and 0.2 adoption, without CLI replacement
  - resource: /design/coordinated-refill/review0.gpt6ax.md
    title: Prior independent native/policy review and corrected path defects
  - resource: /design/coordinated-refill/corrections0.sol61x.md
    title: Subsequent link-target and ordinary byte-name corrections
  - resource: /app/cli/src/cli.rs
    title: Actual public reseed and prepare dispatch and readiness rendering
  - resource: /refill/engine/src/lib.rs
    title: Public driver and separate temporary refill_one_from export
  - resource: /refill/engine/src/engine.rs
    title: Current planning, inventory, lock, verification and per-target sparse-prefill execution
  - resource: /refill/engine/src/engine/replacement.rs
    title: Callable N1 native protocol, memberships, state adoption and finish
  - resource: /refill/engine/src/engine/prepare.rs
    title: Existing preparation, command barriers and repeated source validation
  - resource: /refill/engine/src/engine/payload.rs
    title: Existing source selection, retained-region walks and observed-entry guard
  - resource: /refill/engine/src/engine/bookmarks.rs
    title: Separate history coverage policy and batched ancestry proof
  - resource: /native/jj/src/jj.rs
    title: Native command profile, lean bindings, state-only adoption and version parsing
  - resource: /native/jj/src/context.rs
    title: Repeated registration and primary-root discovery
  - resource: /workspace/state/src/links.rs
    title: Loaded forward finish beside reversible lifecycle application
  - resource: /transfer/workspace/Cargo.toml
    title: Actual sibling-path exact 0.2 dependencies and optional payload parallelism
  - resource: /transfer/workspace/src/tracked.rs
    title: Listed native membership and strict production leaf installation
  - resource: /transfer/workspace/src/streaming.rs
    title: Source-qualified authority, conditional routing, sequential one-lane cursor and real filesystem tests
  - resource: /transfer/workspace/src/frozen.rs
    title: Immutable ignore matching and tree/excludes-qualified extraction caches
  - resource: /transfer/workspace/src/ignored.rs
    title: Selector path replay, Unicode boundary and existing discovery automaton
  - resource: /transfer/workspace/src/transfer.rs
    title: Existing pair adapter, shared bounded pool and explicit clear ownership
  - resource: /app/cli/tests/refill_one.rs
    title: Actual JJ subprocesses, native result/obstruction checks, command tracing and bounded extent evidence
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/coordinated/mod.rs"
    revision: 10361cf0
    title: Actual cohort binding, group/application barrier and distinct lane reports
  - resource: "file:///home/rektide/src/diff-dirs/tree/compare/src/coordinated/mod.rs"
    revision: 10361cf0
    title: Paired-child frontier membership, rather than source-only recursive sharing
  - resource: "file:///home/rektide/src/diff-dirs/tree/inspect/src/shared.rs"
    revision: 10361cf0
    title: One immediate source listing and demand-unioned source facts per paired group
  - resource: "file:///home/rektide/src/diff-dirs/tree/inspect/src/shared_tests.rs"
    revision: 10361cf0
    title: Counted acquisition and Arc-identity test design inspected, not rerun
  - resource: "file:///home/rektide/src/diff-dirs/tree/inspect/src/visitor.rs"
    revision: 10361cf0
    title: Default stat/literal-link equality, leaf retention and action legality
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/apply.rs"
    revision: 10361cf0
    title: Create-only listed installer, repeated per-target source observation and actual copy metadata handling
  - resource: "file:///home/rektide/archive/jj-vcs/jj/lib/src/local_working_copy.rs"
    title: Archived native stat-cache and skip-obstruction mechanisms; not a new installed-runtime certification
  - resource: "file:///home/rektide/archive/jj-vcs/jj/cli/src/cli_util.rs"
    title: Archived native snapshot and writable-working-copy constraints
verification:
  method: Read-only source inspection and attributed prior verification accounts; owned-document check only
  result: Owned-document Rekon check --okf passed; no executable behavior or latency newly measured
  scope: JW parent 76fb0656, starting working copy 4db9154e; diff-dirs 10361cf0; no peer same-wave report read
---

# Reseed check-in

**Keep the native-aware JW/diff-dirs split and the new streaming authority. Adjust
the product framing before finishing the cohort: renewing tracked sharing,
changing revision, and refreshing dependencies are different jobs.** The callable
route is a credible strict *renew-and-reset* implementation, not yet a demonstrated
fast everyday update. Completing N2 cannot by itself settle that distinction.

This is source-derived at JW `76fb0656` and diff-dirs `10361cf0`. The two starting
uncommitted reports were preserved and not read. No build, test, benchmark, donor
snapshot or workspace mutation was run. Historical check results below are
attributed, not independently repeated.

## 1. What actually runs

The public path remains
[`cli::run_reseed → run_refill`](/app/cli/src/cli.rs) →
[`run_with_diagnostics → engine::run_inner`](/refill/engine/src/lib.rs).
It does **not** dispatch to `replacement::refill_one_from`.

Public execution resolves literal/groups/current targets and the actual repository
owner as policy/default donor, optionally prepares once, snapshots/asserts the
donor, saves targets, realizes selected ignored roots, and builds a separate
`Plan` per receiver. Each receiver then clears sparsity, creates a child of D,
prefills S/D-equal tracked leaves, expands sparsity, verifies, mirrors selected
payload through a pair applier, applies links and verifies again. Execution errors
become per-target failures and later plans still run. Structural preflight blocks
the batch before refill. Dry-run performs real snapshots and recursive planning.

The [new callable seam](/refill/engine/src/engine/replacement.rs), conversely,
requires **exactly one** selected target and rejects prepare/dry-run. It reads
recorded S without default donor snapshot; optional assertion A performs one live
diff. Explicit `--at` pins D; otherwise D=S, not A. After saving O and checking
bookmark coverage, it captures membership/cache/ignore inputs, clears native O,
creates a child of D, strictly installs **all S**, adopts only donor `tree_state`,
optionally restores D, establishes native conformance, runs a real one-lane
`SyncCursor`, and finishes loaded links. It fails forward with phase/effect context.

This is real adoption, not dead experimental code: the sibling dependencies are
exact `=0.2.0`, immutable ignore matching is implemented, and missing routes now use
`CreateIfNeeded`. The lean `workspace_bindings_at` also benefits public inventory.
But cohort orchestration, new preparation handoff, honest cheap preview and CLI
replacement remain undelivered. Feature-enabled N1 is still sequential.

The parent supplied host provenance: installed `jw` 0.8.0 matches the current
release binary hash, but its installation record lacks a source revision. That
does not bind it to this source generation. Installed JJ is the reported
`0.45.1-33d3a595…`. Neither version nor historical October performance establishes
new-route latency.

## 2. Where the cost moves

Let N be receivers and S/D/O their tracked memberships. These are **source-derived
successful-path counts**, excluding optional preparation and failure branches:

| Work | Public driver | Callable replacement / prospective cohort |
| --- | --- | --- |
| Live tracked verification diffs | `1 + 8N`: initial donor; four donor plus four receiver checks per target | Zero default donor; one final receiver diff each; D≠S adds restore's snapshot |
| Full receiver snapshots | Six each: save, planning diff, execution diff, sparse clear, materialization diff, finish diff | Three each for D=S: save, clear, final diff; four for D≠S |
| Empty-sparse snapshots | `new`, then sparse reset | `new`; no reset |
| Tracked installation | S/D-equal subset per target | Every S leaf per target, including later-removed S-only leaves |
| Ordinary iteration | Source-root discovery plus retained-region walks and independent pair runs | No discovery/pre-audit; one-lane runtime today; shared paired frontier only after cohort integration |

The repeated donor proofs are the clearest avoidable architectural tax, especially
at N=1. “Streaming” is useful because it removes redundant work, not because native
membership enumeration, cache deserialization, clone opens or snapshots disappear.

`Inventory::load`, `checked_context` and `validate_target` still rediscover
registrations/primary roots. With R registrations, per-target context discovery can
approach N×R queries. Carry one verified command context; batch `CommitFacts` queries;
share S/D/O indexes, rule bytes and cache bytes. Do not call N1 N times. Current
`tracked::seed` rebuilds a membership map and `seed_leaves` collects/sorts a list
per receiver; useful future prepared-list reuse must preserve strict checks and
per-target effects, not introduce an ordinary-filesystem catalog.

## 3. Native correctness and sharing are distinct promises

The native protocol has a strong authority argument: every imported S membership
leaf was installed create-only by this operation. Matching local S-only names are
obstructions, not permission to absorb/delete them. D-only names are separately
admitted; native shape changes may retain unrelated children and subsequently fail
the final diff. `restore` exit success or full sparse patterns alone is insufficient.

[`adopt_tree_state`](/native/jj/src/jj.rs) never imports donor checkout/name/repository
identity. `verify_native` checks receiving change, exact D parent, registration,
root/`.jj`, repository and full sparsity. Keep these semantics even when reducing
duplicate queries. The version gate admits semantic 0.45.1, including different
40-hex-suffixed builds; it is not exact-build format certification. Broader rollout
needs a deliberately supported native profile.

The final diff is JJ conformance under native stat-cache semantics, **not an
independent byte audit**. Borrowed membership/cache is authoritative input from a
prepared, idle donor. Strict seed checks kind/mode/device and presence, not donor
bytes against repository blobs. A stale donor cache with same-stat edited bytes
is outside that premise; frozen paths and `Arc`s do not freeze live files.

Three sharing claims must remain separate:

- Full-S installation renews private regular-file reflinks. For D≠S, changed/D-only
  files subsequently come from JJ; installed S counts do not quantify final D sharing.
- Clean auxiliary entries are retained by type/size/mtime or literal link text.
  They receive **no new sharing proof or renewal**. A stat-equal ordinary copy can
  remain unshared. Existing hardlink aliases are not checked by the inspected
  visitor/equality path either; blanket post-reseed private-inode claims therefore
  exceed the implemented check. This is a source-derived boundary, not a new probe.
- Newly installed reflinks/private-write tests prove those installations, not
  independence of every pre-existing retained file. Decide whether incremental
  payload preservation or mandatory private-sharing renewal is intended.

## 4. Preserve the useful authority and ownership boundaries

[`SourceQualified`](/transfer/workspace/src/streaming.rs) admits an actual ignored
source directory matching policy. Inside it, missing source children authorize
deletion; if the whole candidate source root is absent, the target root stays.
Empty/disabled policy does no ordinary runtime work. Locals outside admitted scope
stay, except initial JJ auto-tracking may first make them tracked O, and explicit
`--replace-content` authorizes broad clearing. Neither is an implicit mirror-all.

Native S/D leaves and active link names remain separately owned. Conditional
containers solve genuine wildcard discovery without creating unmatched skeletons.
The corrected byte-name ordering and declared symlink/`..` target spelling should
be preserved. Pure frozen ignores avoid per-entry Git calls; they do not require
making ignoredness the permanent product selection rule.

JW owns native transitions, policy, saved history and loaded link finish; diff-dirs
owns staging, no-follow effects, permission restoration and temporary cleanup.
Forward failure does not excuse leaked private staging; generic cleanup does not
authorize pruning unrelated locals or rolling back peers. Lifecycle creation's
reversible link application is intentionally separate from reseed's forward finish.

The operation lock currently serializes **preparation only**, including preview;
plain reseed and N1 have no such lock. Keep the explicit idle-writer contract unless
cooperating concurrency becomes a selected product feature. A repository-wide
reseed mutex would serialize independent targets without protecting against editors
or native JJ writers. It is not a substitute for donor stability.

## 5. Substantial choices worth making

### Choose the frequent operation, not just the next implementation slice

| Typical job | Architecture worth selecting or comparing |
| --- | --- |
| Renew all tracked sharing and reset to the prepared donor | Current strict D=S route is coherent. Finish one command-owned cohort, not another adapter layer. Accept O preservation and three native full snapshots as the current CLI composition's price. |
| Frequently update dependencies while keeping tracked feature work | A distinct auxiliary-refresh operation can use the ordinary visitor without clearing/reinstalling tracked work. This changes the job, rather than pretending reseed already preserves the feature. Keep feature duplicate/rebase native. |
| Change to arbitrary D with mostly equal S/D | Compare an upfront equal-leaf/native-materialization route with strict full-S adoption. It avoids cloning S-only intermediates and unnecessary S-only local obstruction requirements, but may reintroduce native placeholder/hash costs. Do not retain the old planner wholesale. |
| Repeated reset where native checkout is already cheap | Compare native-first D checkout followed by authorized tracked reflink replacement, rather than sparse-clear/full-S/cache import. This needs a tested native-owned replacement boundary; today's create-only installer is not that API. It may duplicate store writes on changed paths, so it is a candidate, not an assumed win. |

My preference is **strict D=S renewal as the clear reseed meaning**, with arbitrary
D explicitly costed and dependency-only refresh considered independently if that
is the actual high-frequency demand. Do not multiply modes speculatively. One
different upfront route is justified only by common use or discriminating evidence.

Removing bookmark coverage is a separate history choice: it does not alone remove
the save snapshot. Removing an explicit save while preserving exact O reporting
and reference coverage needs a different native operation boundary, not deletion
of a guard. If measured native work still dominates after removing duplication,
a JJ-supported batched working-copy transition is a stronger long-term seam than
more private-format manipulation. Adopting `jj-lib` would trade process costs for
tight version/runtime coupling; do not do that without evidence.

### Capture one producer handoff, not a permanent donor cache service

Existing [prepare](/refill/engine/src/engine/prepare.rs) repeatedly audits contents
and verifies after every argv step and again at completion; public combined reseed
then snapshots/proves the source again. A command-local `PreparedSource` should
carry final S, upstream goal separately, bindings and lazily acquired indexes/cache
into refill. Pin after preparation and receiver preservation; related repository
rewrites can change the donor coordinate. No persistent journal/digest/receipt is
needed merely to avoid same-command reproof.

Whether every preparer step must preserve tracked files, or only the final sequence
must, is a real relaxation: the old command rejects an intermediate tracked edit
even if a later command restores it. Likewise cheap nonmutating `PLANNED` preview
is not the current recursive `READY` contract. Land these behavior choices with
help/tests, not hidden as performance-only refactoring. A dedicated prepared donor
workspace is often a simpler operational stability boundary than repeatedly using
an actively edited primary; immutability is still a stated premise, not magic.

## 6. What the library can and cannot amortize

[`SharedRoots::inspect_ready`](file:///home/rektide/src/diff-dirs/tree/inspect/src/shared.rs)
lists one immediate source directory once and acquires source facts demanded by
any participating lane once. Each lane still unions its target, runs early/late
policy and validates actions. Only `paired_children` enter the coordinated frontier.
Source-only copy, conditional routing and replacement interiors remain recursive
**per lane**. Tracked seed also observes/opens each source leaf per receiver.

Consequently “one source to many” currently means shared *paired inspection*, not
one whole source scan, one clone operation, or one policy evaluation. Fresh workers
missing the same large dependency tree are precisely where that distinction matters.
Measure this before expanding the library. If dominant, shared source-only
directory observation with per-lane effects is a worthwhile generic direction;
bound its lifetime to a group/subtree rather than cache an entire donor inventory.

Other targeted candidates: reuse validated native seed lists; reduce repeated
selector-prefix/ignore-ancestor allocations; keep observations, recursive plans and
completed effects visibly distinct. Preserve the current public effect kernel and
pure policy boundary. No persistent cursor/recovery machinery is needed by JW.

## 7. Claim/evidence ledger and next discriminators

| Claim | Evidence actually available | Limit |
| --- | --- | --- |
| New protocol reaches native D and preserves receiving identity | Inspected `refill_one` tests use real JJ, actual trace wrapper, independent final native queries; prior account reports nine passing native cases | N1, one reported native build; not public CLI/N2 latency |
| Real sharing/private writes | Required installation counters, FIEMAP on a 128-KiB common tracked file and private-write assertions | Representative installed file, not all final D/retained payload extents |
| Authority handles one-sided/conditional work | Real production cursor filesystem tests; supplied Known/Unknown policy tests; attributed slice-2/correction results | Supplied Unknown is not forced backend behavior; transfer fixtures are not whole JJ commands |
| Source facts are genuinely shared | Library counted acquisition/Arc-identity test and inspected shared implementation | Instrumented acquisition seam; no current JW cohort acceptance test |
| Whole-command performance improves | Fewer source-derived passes, no new-route measurement here | Historical pair-adapter timings cannot establish current callable or installed-binary performance |

Next experiments, **not executed or authorized as implementation here**:

1. Reuse one bounded fixture for native, uninstrumented N1 D=S and nearby/distant
   D≠S comparisons. Attribute preservation, native materialization, ordinary work
   and finish separately; trace argv in a separate count run. This discriminates
   full-S/cache value from merely removing repeated proofs.
2. With a real cohort entrypoint, compare N1/N2/N4 paired-clean targets against
   identically missing selected subtrees. Count actual source acquisitions separately
   from clone effects. This decides whether shared opaque observation is worth adding.
3. Compare clean tracked renewal, a small tracked change and auxiliary-only refresh
   at equal final intent. Record final sharing, not only installed reflinks. Add a
   stat-equal unshared/hardlinked auxiliary case to settle the retained-data promise.
4. Before cutover, exercise one supported-version boundary, prepare-once handoff,
   genuinely nonmutating preview and failure after native-ready peers. Reports must
   distinguish saved, native-ready, ordinary-complete and link-finished targets.

**Verdict:** the direction is substantially better than optimizing the public
per-target planner. Preserve the work already delivered, but approve the everyday
sharing/update contract before treating the commissioned uniform full-S N2 route
as the inevitable product. Then judge one useful whole command, not adapter test
counts or a source-once slogan.
