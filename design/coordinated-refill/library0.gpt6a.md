---
type: LibraryAssessment
title: Production diff-dirs coordinated refill API and bounded consumer evidence
description: Current source-to-many comparison and synchronization capabilities, process-local consumption, root and policy boundaries, verification, and a concrete JW integration recommendation.
resource: /design/coordinated-refill/library0.gpt6a.md
status: assessed-not-implemented
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-28 }
sources:
  - resource: "opencode://ses_e7c8e2242ffekpPV7a8RX0rEOH"
    title: Independently assesses current library APIs, verifies JW manifest failure, and runs a bounded production multi-target consumer and shared-acquisition tests
  - resource: "file:///home/rektide/src/diff-dirs/README.md"
    revision: 5c4192e7
    title: Current specification, package coordinates, equality and coordinated sharing contracts
  - resource: "file:///home/rektide/src/diff-dirs/tree/api/src/lib.rs"
    title: Facade exports and shared public type identities
  - resource: "file:///home/rektide/src/diff-dirs/tree/compare/README.md"
    title: Coordinated frontier, group controls, runtime loans and parallel boundaries
  - resource: "file:///home/rektide/src/diff-dirs/tree/compare/src/coordinated/mod.rs"
    title: Actual binding, next-group, settling and lane control implementation
  - resource: "file:///home/rektide/src/diff-dirs/tree/inspect/src/shared.rs"
    title: Source-once immediate inspection and within-group metadata threshold
  - resource: "file:///home/rektide/src/diff-dirs/tree/inspect/src/shared_tests.rs"
    title: Counted acquisition, shared fact lifetime and independent lane error tests
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/README.md"
    title: Production sync, coordinated root restrictions, completion and reporting contracts
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/coordinated/mod.rs"
    title: SyncRuntime, SyncCursor, current application barrier and failure handling
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/coordinated/state.rs"
    title: LaneTotals, End, application dispositions and optional data-only continuation
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/coordinated/tests.rs"
    title: Read production regression coverage for continuation, opaque ownership, routes and sparse lanes
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/lib.rs"
    title: Applier builders and separate whole-root initialization/replacement
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/apply.rs"
    title: Finalized-plan application, one-sided recursive ownership and production effects
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/visitor.rs"
    title: Seed, Update, Prune and compatibility steering composition
  - resource: "file:///home/rektide/src/diff-dirs/examples/reconcile/src/main.rs"
    title: Concrete production driver, failure reporting and optional application-owned transport
  - resource: "file:///home/rektide/src/diff-dirs/examples/reconcile/src/lib.rs"
    title: Target-specific frozen policy implementation and runtime reconstruction
  - resource: "file:///home/rektide/src/diff-dirs/design/architecture/rebuild0.gpt6a.md"
    title: Current delivered design sequence and explicit separation of traversal continuation from in-flight recovery
  - resource: /Cargo.toml
    title: Current stale dependency package paths and feature forwarding
  - resource: /src/cow/transfer.rs
    title: Narrow compatibility seam for existing Auxiliary visitor and command-owned pool
  - resource: /design/sync-scope/design0.gpt6a.md
    title: Prior authority, tracked-tree and transfer boundary proposal
  - resource: /design/sync-scope/cursor-actions0.gpt6a.md
    title: Human forward-iteration direction and prospective extensible action distinctions
verification:
  method: Six tiny external production-consumer tests, sixteen selected upstream acquisition tests, explicit stable compilation and current JW build attempt
  result: All 22 selected tests passed; stable consumer check passed; current JW Cargo check fails at dependency manifest loading
  scope: Linux current local library checkout; API and semantic evidence, not native JJ end-to-end or throughput evidence
---

# Current library assessment

## Recommendation

**Use production `SyncCursor` plus `SyncRuntime` for JW's process-local auxiliary
refill, with one frozen target policy per destination.** The library already owns
the shared paired traversal and the real recursive transfer kernel. No serialized
cursor, journal, new scheduler or example-derived copier is needed.

This assessment examined diff-dirs at delivered parent `5c4192e7` on October 28.
The independent JW census owns the wider command implementation; this document
only reads its existing transfer adapter and earlier sync-scope context. No other
same-wave assessment was read. The actual author is Astra, so this file uses its
required `gpt6a` suffix rather than the commissioned filename's GLM suffix.

The principal limits for planning are:

1. **Root containers already exist.** Coordinated sync does not initialize an
   absent destination root or replace a leaf root. Binding at the real workspace
   root fits JW; its selected payload subdirectories can be missing.
2. **Source-once sharing covers immediate paired-directory groups.** Copying a
   missing subtree is an opaque, separately recursive operation per destination.
   Empty new workers therefore get less sharing than existing similarly shaped
   workers. The implementation is not a single shared source walk through every
   recursively copied file.
3. **Lanes are independent semantically, but run behind a group barrier.** One
   slow lane delays the next shared group. Current production cursor application
   is one destination request at a time, with optional parallelism inside it.
4. **Seed means create missing names, not reject every collision.** `Seed` retains
   existing leaves/type collisions and descends paired directories. JW's current
   creation visitor instead refuses an existing selected root; choosing missing-
   only seed changes that policy and should be explicit.
5. **JW does not currently build.** Its old package paths fail before compilation.
   The new public facade and sync crate compile together in the external probe.

## Package and dependency seam

[The root manifest](file:///home/rektide/src/diff-dirs/Cargo.toml) is a virtual
workspace. Actual production package coordinates are:

| Role | Package | Path from JW |
| --- | --- | --- |
| Convenient comparison/visitor exports | `diff-dirs-tree-api` | `../diff-dirs/tree/api` |
| Traversal owner, if directly needed | `diff-dirs-tree-compare` | `../diff-dirs/tree/compare` |
| Immediate shared inspection owner | `diff-dirs-tree-inspect` | `../diff-dirs/tree/inspect` |
| Filesystem effects and coordinated sync | `diff-dirs-transfer-sync` | `../diff-dirs/transfer/sync` |

All reviewed package versions remain `0.1.0`; path/package identity changed.
The [facade exports](file:///home/rektide/src/diff-dirs/tree/api/src/lib.rs), lines
34–47, include `coordinated`, `Execution`, `Visitor`, `Names`, `Operations`,
`Action` and compatible existing walker types. The facade reexports native types;
the sync crate's visitor type is the same type, not a separate conversion layer.
`SharedRoots` itself is a lower-level inspect/compare export, not a facade export.
JW does not need it for production coordinated sync.

Current [JW manifest](/Cargo.toml), lines 28–29, still points `diff-dirs` at the
now-virtual root and `diff-dirs-sync` at the missing `sync/Cargo.toml`. A bounded
first change can retain Rust import names by using Cargo package aliases:

```toml
diff-dirs = { package = "diff-dirs-tree-api", version = "0.1.0", path = "../diff-dirs/tree/api" }
diff-dirs-sync = { package = "diff-dirs-transfer-sync", version = "0.1.0", path = "../diff-dirs/transfer/sync" }
```

Those aliases also retain existing `diff-dirs/parallel` and
`diff-dirs-sync/parallel` feature-forwarding keys. Alternatively adopt canonical
dependency/import names in one explicit migration. Update the receiving lockfile
and verify JW's other workspace member then; this assessment did not edit them
or claim that package repair alone proves the whole application builds.

## Actual production caller loop

The relevant owner is
[`transfer/sync/src/coordinated/mod.rs`](file:///home/rektide/src/diff-dirs/transfer/sync/src/coordinated/mod.rs):

| API | Source lines | Caller responsibility |
| --- | --- | --- |
| `SyncTarget { id, descriptor, applier }` | 104–110 | Give each destination a stable identity and frozen policy description. |
| `SyncRuntime::new(Vec<SyncTarget<…>>)` | 112–164 | Own executable per-lane policies/appliers. All sources must match. |
| `SyncCursor::new(&runtime, Execution)` | 210–225 | Constructs and binds; no extra `bind` is needed for a fresh run. |
| `next_group(&runtime)` | 374–441 | Returns clean and divergent groups and target-tagged comparison errors. |
| `pending()` and `apply(&runtime, &token, &target)` | 365–372, 611–696 | Dispose every current unstarted lane using the cursor's cached plan. |
| `settle()` | 274–284 | Close a disposed control window and fold ready returns, without I/O. |
| `reports()` | 350–363 | Obtain lane-local observed coverage, completed work and satisfaction. |
| `drive(&runtime, Limits)` | 709–760 | Existing convenience driver; failure events do not prevent healthy peers finishing. |

The simplest owned process-local form is:

```rust,ignore
let runtime = SyncRuntime::new(targets); // Applier::new(...).visitor(policy)
let mut cursor = SyncCursor::new(&runtime, Execution::Sequential)?;
let outcome = cursor.drive(&runtime, Limits {
    groups: u64::MAX,
    applications: u64::MAX,
})?;
// Surface outcome.failures, even when outcome.stop == Stop::Done.
let reports = cursor.reports();
// Complete only lanes whose report.satisfied is true.
```

JW can instead pull a group, report each comparison error, apply every `pending`
target with the group token, collect terminal application failures, settle, then
pull again. This exposes useful progress/cancellation boundaries. Do not put `?`
on a lane-local failure if the command intends healthy targets to continue.
Global binding/control failures remain distinct command errors.

**Do not skip `apply` for a clean diff or `coverage.planned == 0`.** Every ordinary
successful group is pending, even when no immediate namespace work was counted.
A kept target-only directory can have recursively removable children. The probe
proves exactly this case. `next_group` refuses `PendingApplications` until each
application is accepted, failed or abandoned.

`SyncRuntime<C,S,A,V>` uses one concrete type for each policy parameter across its
vector. Different target values are supported directly: a single JW policy struct
can contain its lane's tracked exclusions, selected roots and mode. For genuinely
different behavior families use a small enum/struct implementation of `Visitor`,
as the probe does. Existing `VisitorRef<'a, Auxiliary>` can borrow policies held
in a separate outer vector; owning policies directly also works. Temporary
`Steering` loans inside library calls avoid self-referential structures.

The caller still supplies `descriptor: String` even for ephemeral use. It is a
frozen compatibility promise, not serialized executable code or introspected
configuration. Match its content to behavior-affecting inputs; there is no need
to add a transport schema solely to satisfy this field.

## Roots, shape and policies

### Binding requirements

`Applier::new` accepts an absent target coordinate if its real parent exists;
`SyncCursor::new` subsequently rejects it because coordinated comparison needs
real directory roots. This distinction was directly tested. Source roots must
exist. Leaf roots require the separate explicit replacing-root applier path;
coordinated binding still does not accept them.

Sync's binding rejects source/target and cross-target equality or nesting, checks
real ancestors, and on Unix rejects equal root device/inode identities. Plain
read-only coordinated comparison permits aliases that effects reject. These are
stable-tree structural checks, not hidden-mount alias detection or hostile-race
confinement. Retain JW's native topology ownership.

**Prefer binding at existing workspace roots.** Native workspace setup establishes
the container and `.jj`; early policy preserves administrative paths. Missing
`node_modules`, `dist` or routing directories are then ordinary source-only
children handled by production recursive application. Initializing each selected
root first with `Applier::initialize()` would already perform its whole copy,
then a coordinated scan would repeat comparison unnecessarily. If a caller truly
needs a standalone absent selected root, dispatch initialization separately and
account for it separately, or deliberately create the structural container under
caller-owned policy before binding. Do not pretend initialization occurred inside
the coordinated cursor.

### Missing-only seed and reseed

[`Seed`](file:///home/rektide/src/diff-dirs/transfer/sync/src/visitor.rs), lines
46–67, is already a production `Visitor`:

* Source-only entries retain default creation/copy actions.
* Both-sided directories keep their container and descend to fill missing names.
* Existing leaves, type collisions and target-only trees are preserved.
* The same policy reaches source-only recursive copying, not just paired plans.

Combine this with early administrative/local preservation in a JW visitor and
delegate the late missing-only rule to `Seed.late(batch)`. Existing
[`Auxiliary`](/src/cow/transfer.rs), lines 119–162, already implements suitable
frozen target-local root/routing policy for bounded reseed. Passing a different
`Auxiliary` per destination is compatible with the new runtime. Its
`new_targets_only` refusal is a different creation contract from `Seed` retention.

Reseed can use default mirror actions inside authorized roots, `Update` or
`NeverRemove` for no deletion, or the existing custom scope/absence/visitor
composition. No upstream change is required to express those policies. Early
preservation is symmetric and avoids admitted-child metadata/readlink; late
retention can keep a container while allowing child work. Keep target-specific
S/D tracked boundaries in JW. The library has no JJ revision, ignore, link-owner
or administrative-name knowledge.

General missing-only seed also differs from `seed_leaves`: the latter consumes
JW's already-known tracked leaf list, refuses existing destination leaves, and
does no discovery of unrelated source names. It remains a separate sequential
installer, not a source-to-many listed-leaf cursor. The coordinated walker cannot
force-install stat-equal leaves to renew CoW sharing, and stat equality excludes
mode-only changes. Preserve the native tracked materialization/prefill decision
instead of claiming the filesystem cursor replaces it.

## What is actually shared

[`SharedRoots::inspect_ready`](file:///home/rektide/src/diff-dirs/tree/inspect/src/shared.rs),
lines 391–484, performs:

1. One source listing for the immediate group.
2. One target listing and early policy per ready lane.
3. A union of demanded source entries across admitted lanes.
4. One acquisition of each demanded source entry, retaining `Arc<Facts>` across
   per-lane plans; literal source links are acquired once where needed.
5. Independent target facts, late policy, validation, error and plan per lane.

The counted upstream test at `shared_tests.rs:187–259` verifies one source list,
one stat per demanded entry, one source link acquisition, zero acquisition for
universally preserved entries, identical source fact identity across lanes, and
release after reduced publication. All sixteen selected shared tests passed.
The external production probe independently observed identical borrowed source
fact addresses for both lanes in the root group; that establishes reuse there,
not a syscall count for the whole command.

Only **paired child directories** enter the merged DFS membership sequence. For
lane A, `pkg/` may be paired and join later groups; for B, the same path may be
source-only or collide with a leaf and be fully handled in B's current opaque
application. B does not join later paired traversal merely because applying its
copy created a directory. Read production regression
`opaque_replacement_does_not_join_later_paired_traversal_after_becoming_a_directory`
in [the sync tests](file:///home/rektide/src/diff-dirs/transfer/sync/src/coordinated/tests.rs).

`apply_plan_with` reuses the current shared immediate plan and invokes the existing
kernel with paired recursion disabled. In `copy_directory_selected`, each lane
then separately inspects its one-sided source subtree. Therefore:

| Situation | Source-sharing result |
| --- | --- |
| Similar populated destinations | Paired directory listings and demanded source facts shared per group. |
| One destination lacks a subtree | Other paired lanes share subsequent groups; missing lane copies that interior separately. |
| Every destination lacks a large subtree | Parent entry facts shared; subtree contents are recursively enumerated/copied once per destination. |
| Every lane preserves a name | Parent names known; no admitted child facts or subtree traversal. |
| Restored pending group | Fresh source-sharing refresh epoch; not lifetime memoization. |

Bindings and no-follow route checks are additional I/O. Sync revalidates source
and target routes before each application. A claim of “every source stat once for
the command” would be false. Physical copying still opens/reads/clones for each
destination. No benchmark was run and source-once acquisition is not a measured
wall-time win.

## Scheduling, pruning, finish and failure

The coordinated frontier is **sequential between groups**. With `parallel`, shared
inspection dispatches sufficiently wide metadata vectors in the current Rayon
pool: current `indexed_map` uses `len >= 64` and `with_min_len(8)` in
[`shared.rs`](file:///home/rektide/src/diff-dirs/tree/inspect/src/shared.rs), lines
575–582. Source listing, target-listing preparation and lane-level finalization
are not independent whole-tree jobs. Sync uses the same execution selection to
enable kernel batch parallelism within one destination's application.

`SyncCursor` has no borrowed-pool constructor or pair-frontier J/K parameters.
Wrap the relevant sequence in `pool.install(|| ...)`, including application,
instead of reusing the old `ParallelLimits` argument. JW's existing P/J/K adapter
cannot be carried over unchanged. `&mut SyncCursor` and the current barrier also
mean callers cannot simply `par_iter` over `cursor.apply` to run destinations
concurrently. No new destination scheduler is justified for the first integration.

Groups and applications join started work before returning. Memory remains
width-sensitive: per-target unions/full plans, exact selected decisions, child
membership suffixes and summaries are retained. `reports()` clones cumulative
summary vectors; use it at meaningful progress/completion points rather than per
file. There is no fixed-byte, syscall, or finite latency bound.

* `prune_here(token, targets)` affects only those lanes' paired child membership;
  it does not cancel the immediate batch or its opaque recursion. Explicit prune
  makes `satisfied == false` even when the remaining scan exhausts.
* `stop_lanes(targets)` abandons future obligations and pending applications for
  selected lanes. It cannot interrupt a copy currently executing inside `apply`.
* `settle()` folds ready returns only after pending applications are disposed.
  Exhaust with `next_group -> None`, or `drive -> Stop::Done`, before treating
  the run as terminal. A final applied batch alone need not have closed its root.
* Lane comparison errors appear in group outcomes; application failure returns
  its actual partial `Failure`, marks the lane terminal and leaves healthy lanes
  runnable. Fresh construction/binding errors can reject the whole attempted
  runtime before any group; omit independently failed setup targets deliberately.
* Failed lanes are not retryable within that cursor. A fresh comparison consumes
  the actual post-failure filesystem state. No rollback or internal recursive
  recovery position is promised.

Production kernel staging/permissions finish before each application returns.
Directory-to-leaf preservation conflicts receive the existing local removability
precheck; arbitrary failed writes can still leave completed earlier work. There
is no generic per-directory custom/generated-output finish callback in this
production cursor. Earlier extensible action and generated-entry designs remain
prospective. JW can perform its own native lane-finalization after successful
refill and keep that result distinct from filesystem satisfaction.

## Accounting and ephemeral use

`LaneReport.observed` is the accepted **pre-application comparison**. Its counts
include an opaque source-only directory once, not all leaves later copied inside
it. It does not become clean merely because copying succeeded.

`LaneTotals` separately reports application count, planned work, actual completed
files/bytes/reflinks/removals/directory permissions, exclusions, recursive
retention, `End` and explicit prune status. Cumulative work uses `u128` while
individual kernel progress uses `u64`; conversions in JW reporting need to respect
that distinction. `LaneReport.satisfied` means the selected policy exhausted
successfully, not full-root equality or proof of the JJ tree.

`Drive::Stop::Done` means all lanes are terminal, including failures. Inspect
`Drive.failures` and lane satisfaction. A failed application can report created
directories or completed copies; retain those numbers instead of replacing them
with the observed comparison counts or treating its target as untouched.

There is no mandatory serialization path: construct runtime/cursor, loop, report,
drop. Production crates do not require the reconcile example's JSON transport.
`parts`/`into_parts` and `from_parts` are optional data interfaces. Avoid repeatedly
round-tripping them during an ephemeral run: that discards cached plans and forces
pending decisions to refresh. The
[reconcile consumer](file:///home/rektide/src/diff-dirs/examples/reconcile/src/main.rs)
is useful precedent for construction and error handling, not an argument to add
its checkpoint publication to JW.

Likewise there is no production coordinated recursive dry-run entry point.
Per-target `Applier::preview()` uses the same policies and covers one-sided
interiors without effects. Read-only `Coordinated` supplies shared paired
comparison but deliberately leaves those interiors opaque. Choose explicitly
which dry-run coverage JW promises; do not use opaque comparison counts as a
complete recursive work prediction or run preview before ordinary execution.

## Bounded verification performed

Scratch consumer:
[`library-probe`](file:///home/rektide/src/jj-waltz/.test-agent/coordinated-refill/library-probe/README.md).
Its source, Cargo lock and build products are ignored scratch; only this assessment
is committed. Fixtures use `TempDir` beneath that directory and are removed after
each test. The largest fixture has fewer than 256 entries across its source and
two destinations, with short literal contents. No real workspace was copied or
modified. Upstream selected tests use their own removable `/tmp/opencode` fixtures;
their build products were redirected into the same JW scratch subtree.

| Command | Observed result |
| --- | --- |
| `cargo check --locked --offline -p jj-waltz` from JW | Exit 101 before compilation: missing `/home/rektide/src/diff-dirs/sync/Cargo.toml`. |
| `cargo nextest run --offline` from `library-probe` | Six tests passed; none skipped. |
| `cargo nextest run --offline --locked --manifest-path /home/rektide/src/diff-dirs/Cargo.toml --target-dir /home/rektide/src/jj-waltz/.test-agent/coordinated-refill/library-probe/upstream-target -p diff-dirs-tree-inspect --features parallel -E 'test(shared::tests)'` | Sixteen shared-inspection tests passed; thirteen unrelated tests filtered out. |
| `cargo +stable check --offline --tests` from `library-probe` | Passed, establishing stable compilation of the concrete consumer. |

The harness environment overrides JW's stable toolchain file with
`nightly-2026-10-14`; the test executions above used that active toolchain. This
prompted the separate explicit stable compile check. Native-dirent was not enabled.

The six external tests establish:

1. **Mixed target policy, shared facts and clean repeat:** seed in A, mirror with
   a lane-specific preserve in B; both preserve `.jj`; type collision, source-only
   recursion, paired descent, 70-file wide metadata batch and two-worker current
   pool; both sequential and parallel modes. Barrier refusal before applications,
   exact target contents, two examined paired directories, then zero actual writes
   on repeated policy-satisfied runs.
2. **Binding:** missing target and duplicate destination roots reject before
   effects; the missing target remains absent.
3. **Opaque failure:** one deep refusal leaves its authorized created directory,
   marks only that lane `ApplicationFailed`, reports one created directory, and
   lets the peer finish.
4. **Prune/finish:** root batch with zero immediate writes still gets applied;
   pruning A prevents its child work and satisfaction while B finishes.
5. **Zero planned is not zero recursive work:** keep `old/`, remove `old/stale`,
   and report actual removal despite zero immediate planned work.
6. **Actual required reflinks:** both targets report one reflink and zero ordinary
   copies; an in-place edit to A leaves donor and B contents unchanged. No skip
   or fallback was allowed for this test.

These checks establish public API fit and the specified small semantic cases.
They do not establish full native-JJ behavior, 1.88 MSRV compatibility, portability,
all existing JW tests, broad production robustness or a performance improvement.

## Suggested implementation sequence

1. **Repair package coordinates and receiving locks.** Compile existing default
   and `payload-parallel` JW first. Preserve a reviewable dependency-only stage.
2. **Create one batch-owned policy/runtime seam.** Capture each destination's
   native selected/tracked/excluded facts, finish prerequisite native preparation,
   then build `SyncTarget`s at disjoint existing workspace roots. Shared source
   path must match exactly after binding. Exclude failed preparations explicitly.
3. **Replace the target-by-target auxiliary loop with the production cursor.**
   Keep current bounded authority for the first behavioral-equivalent reseed
   slice. Use the existing shared caller pool with coordinated execution semantics.
   Keep progress outside pure visitors; consume lane failures without losing peers.
4. **Select seed semantics deliberately.** Missing-only seed can use `Seed` now.
   Retaining existing selected roots is a contract change from current creation
   refusal; do not obscure it behind the package migration. Keep tracked known-
   leaf seeding/native JJ setup separate unless that outcome is separately changed.
5. **Complete native phases per lane.** Successful library satisfaction makes
   auxiliary refill complete; JW still owns link setup, tracked-tree outcome,
   workspace identity and command exit/reporting. Capture partial lane effects
   without introducing serialized recovery machinery.
6. **Verify the discriminators in JW's small native tests.** Two destinations
   with different policies/shapes; missing payload but existing roots; source
   administrative state hidden; target extras preserved outside authority;
   one failed lane plus one successful lane; zero-write repeat; selected revision
   differing from donor; real required reflinks. Keep fixtures tiny.

Only after that integration would a small equal-scope comparison answer whether
shared source acquisition offsets group barriers and changed parallelism for
JW's workload. For bulk new-workspace copying, the current opaque-recursion
boundary is a meaningful remaining library opportunity. It is not a prerequisite
for consuming the delivered cursor, nor evidence that it already shares the
entire source seed traversal.

## Connections

* [Current rebuild](file:///home/rektide/src/diff-dirs/design/architecture/rebuild0.gpt6a.md)
  distinguishes delivered production coordination from earlier component and
  generated-action proposals; this assessment supplies a bounded JW-side consumer
  check against those delivered capabilities.
* [Earlier JW scope design](/design/sync-scope/design0.gpt6a.md) supplies tracked
  authority and broad-transfer alternatives. Its source-to-many future direction
  is now partly delivered; its product-default choices do not become accepted
  merely because the mechanism exists.
* [Cursor/actions context](/design/sync-scope/cursor-actions0.gpt6a.md) records the
  human's forward-iteration failure model. Optional traversal parts fit it, but
  are unnecessary for this process-local integration; opaque effects still have
  no internal saved recovery position.
