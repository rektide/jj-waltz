---
type: DesignAssessment
title: Seed the integration, then make reseed a coordinated refill
description: A staged jj-waltz adoption of diff-dirs coordinated synchronization, emphasizing understandable ownership, repeated-reseed value, native lifecycle boundaries and discriminating evidence.
resource: /design/coordinated-refill/assessment0.gpt6a.md
status: proposed
generated: { by: "model:openai/gpt-6-astra#high", at: "2026-10-28" }
sources:
  - resource: "opencode://ses_e7c8f24ebffelGRFC0RXsQvf61"
    title: Human requests assessment and plan for returning the revamped diff-dirs design to jj-waltz; parent evaluates ownership, sequence and evidence with two independent G analysts
  - resource: /docs/reseed.md
    title: Current reseed, preparation, auxiliary-selection, failure and preparation-only locking contracts
  - resource: /docs/architecture.md
    title: Current native lifecycle ownership and creation/reseed materialization distinction
  - resource: /design/sync-scope/implementation0.gpt6a.md
    title: Delivered visitor integration and explicit correction leaving broader copying defaults unselected
  - resource: /design/sync-scope/cursor-actions0.gpt6a.md
    title: Human direction toward forward iteration, a good donor, optional continuation and native adapter boundaries
  - resource: /design/sync-scope/visitor0.sol61xh.md
    title: Existing generic kernels, bounded compatibility visitor and native state-adoption experiment
  - resource: /design/up-performance/primary0.sol61m.md
    title: Actual workload shape, limited historical clean-reseed observations and cancelled excessive-fixture campaign
  - resource: /tools/performance/README.md
    title: Existing Rust performance tooling and its reset/import costs
  - resource: "file:///home/rektide/src/diff-dirs/README.md"
    title: Delivered coordinated comparison and synchronization, sharing boundaries and package coordinates
  - resource: "file:///home/rektide/src/diff-dirs/doc/architecture/README.md"
    title: Current component ownership, partial-frontier semantics and native-consumer distinctions
  - resource: "file:///home/rektide/src/diff-dirs/design/architecture/rebuild0.gpt6a.md"
    title: Implemented production cursor/runtime and separately outstanding performance measurement
  - resource: "file:///home/rektide/src/diff-dirs/design/native/graph1.gpt61sxh.md"
    title: Bounded evidence that truthful donor tree-state adoption can change retained-local membership
---

# Seed the integration, then make reseed a coordinated refill

## Recommendation

**Use one small seed integration to establish the new library boundary, then
concentrate the redesign and measurement on reseed.** Make the command read as:

```text
capture donor and target intent
→ establish each receiving JJ workspace
→ reconcile authorized ordinary data across the ready destinations together
→ finish each receiving workspace and report its result
```

The durable simplification is one shared source context, one target-specific
policy, one library-owned transfer loop, and explicit native setup/finish. A
coordinated loop makes that structure practical. Merely changing a dependency
name or putting today's entire per-target operation inside a parallel iterator
would not produce it.

Include **seed as new-workspace creation through `add` and creating `s`**, rather
than first inventing another command. A library's missing-only `seed` operation,
JW's tracked-leaf prefill, and the complete workspace-creation workflow have
different meanings. Name those distinctions in the implementation.

The first useful landing retains the current transfer outcome. The broader
ordinary-content policy remains a meaningful follow-on design choice; the prior
[correction](../sync-scope/implementation0.gpt6a.md#correction-the-jw-policy-and-state-transition-are-not-settled)
explicitly records that neither whole-root mirror nor broad update was selected.

## What the new library actually buys

`diff-dirs` now has a production one-source/many-destination comparison frontier
and a production synchronization cursor/runtime. This is beyond the earlier
cursor and walking examples. Its
[current contract](file:///home/rektide/src/diff-dirs/README.md#coordinated-multiple-destinations)
shares source directory listings and demanded source facts within each immediate
inspection group, while retaining destination-local policy, child membership,
errors and summaries.

That is especially useful when several existing destinations already have the
same large dependency-directory structure:

```text
Before: for each target, walk source against that target
After:  for each shared directory, inspect source once against its active targets
```

Target listings and target facts still cost work for every destination. Actual
installs/removals also remain per destination. Source-only subtrees are opaque to
the coordinated frontier: the production executor recursively copies them per
destination. Consequently:

| Workload | Expected opportunity, not a measured speedup |
| --- | --- |
| Several clean or slightly changed existing payloads | Strongest direct fit: much shared paired traversal, few writes. |
| One existing destination | Ownership simplification; no cross-destination observation saving. Cursor overhead and the old parallel walk need comparison. |
| Several empty destinations | Good semantic integration case; deep missing-subtree source work is not yet shared. |
| Mostly replaced/one-sided subtrees | Effects and per-destination recursive work can dominate; directory-group sharing is a smaller fraction. |
| Several separate `jw reseed` processes | No sharing between invocations. The callers must submit a batch to share observations. |

For a completely paired shape with comparable source and target observation costs,
the inspection component moves roughly from **2N tree observations to N+1**.
That suggests 25% fewer such observations at N=2, approaching 50% at large N—not
an N-fold command speedup. Policy work, native operations, routing checks, copying
and one-sided recursion sit outside that simple estimate.

The existing multi-name/group interface already supplies a natural batch. The
first implementation needs no daemon or persistent source catalog. Normal
one-target reseed remains a one-lane use of the same conceptual operation.

### Coordination is not automatic parallel execution

Current coordinated traversal advances directory groups in order. Parallel mode
can share a caller-owned pool for within-group observation; it does not inherit
the pair walker's speculative J/K scheduler. The current group's slow opaque
application delays the next group. Independent successful lanes can continue
after another lane returns a failure, but this does not promise independent
latency while a filesystem call is blocked.

Initially choose the most legible execution path, then compare the existing
experimental `payload-parallel` variant separately. Carrying its `J=P, K=256`
numbers onto an unrelated scheduler would be misleading.

## The ownership boundary

| Owner | Concrete responsibility |
| --- | --- |
| JJ | Snapshots, graph/parent choice, tracked membership, sparse state and native materialization. |
| JW command | Selection, source/desired revision, history coverage, topology and target identity, copying authority, native phase ordering, links and user result. |
| diff-dirs inspection/traversal | Source sharing, comparison, pure visitor decisions, child membership, ordered groups and scoped coverage. |
| diff-dirs synchronization | Required-reflink installation, recursive copy/removal, staged replacement, metadata handling and actual-work accounting. |

Use existing modules until a split improves this boundary. There is no need for
one new trait/module per phase or an application-level duplicate of the library
frontier.

### Tracked data and auxiliary data remain different inputs

Let **S** be the captured donor tracked tree, **D** the selected desired tracked
tree, and **Oᵢ** each old destination tree. Plain reseed selects D=S; `--at` may
select something else. The whole donor filesystem is therefore not always the
desired destination filesystem.

The tracked prefill's purpose includes renewing CoW sharing. Stat-equal leaves
are normally no-ops for a directory synchronizer; that does not renew their
sharing. Keep the generic **listed-leaf installer** for the known S/D-equal
tracked set and let JJ materialize D. Use the coordinated directory cursor for
authorized auxiliary data. They share effect kernels without pretending to
share one equality rule.

Creation already has a different native adapter involving donor `tree_state`
reuse; existing reseed uses sparse-empty/prefill/reset. Unify their transfer
plumbing, not their native-state transitions by assumption.

The later [native membership experiment](file:///home/rektide/src/diff-dirs/design/native/graph1.gpt61sxh.md)
matters here: a truthful source cache can import tracked membership and cause a
retained ignored local to be removed on S→D restoration. Exact final D and clean
status alone did not detect that loss. Cache reuse is a credible later reduction
of native work, especially D=S, but requires its own explicit local-ownership
transition. It is not a prerequisite for adopting coordinated transfer.

## Proposed command structure

The following names describe responsibilities, not required public Rust types.

### 1. Capture one command context

Resolve configuration and source; optionally run explicitly requested preparation;
capture the good, stable donor and exact revision/tree facts once. Resolve targets
and their wanted revisions, bookmark coverage, retained regions and links. Share
immutable donor entries, ignore inputs and topology facts rather than reconstructing
them in each lane. Today's reseed selects one D for the batch; old-tree,
retained-path and receiving-identity facts remain target-local.

This step produces bounded native facts and per-target intent, not a recursive
payload inventory or a serialized operation journal. Repeated proof of the same
unchanged donor should be justified by an intervening writer or distinct claim.
The earlier [human direction](../sync-scope/cursor-actions0.gpt6a.md#direction-and-standing)
assumes a good donor and forward iteration.

### 2. Establish native receiving state

For creation, initialize the actual JJ receiver and perform its native tracked
materialization/cache-validation sequence. For reseed, preserve the root/`.jj`,
perform the tracked sparse-prefill transition, and return the receiver to full
sparse state before entering the auxiliary batch.

Prepare these receivers serially initially. Each successful receiver becomes
eligible for the transfer cohort; its identity and desired tracked state are
already concrete. **Do not sparse-clear every target and then leave them all
half-materialized while a large payload traversal runs.** That would spread the
most awkward native intermediate state merely to gain source sharing elsewhere.

Keep preflight's current all-selected-target readiness meaning explicit. A later
execution failure is different from a preflight blocker. Native setup failures
retain target-specific outcomes and the existing out-of-band recovery information.

### 3. Bind the ready destinations and drive one transfer

The source is the workspace root. Each lane binds the receiving workspace root
and its own frozen auxiliary visitor. Both already exist as real directories;
missing payload subdirectories are ordinary entries handled by the effect kernel.
This avoids trying to bind each absent payload directory as a coordinated root.

The current `Auxiliary` policy is an appropriate first projection of authorized
regions: preserve native metadata and out-of-scope names early; retain routing
directories; refuse routing leaf obstructions; reconcile selected descendants.
Creation's create-only collision contract must remain explicit—it is stronger
than silently skipping an occupied leaf with a generic missing-only seed policy.

Inside the loop, JW consumes directory-group outcomes, applies pending lane work,
and records accepted results. Visitors read frozen facts and supplied observations;
they never run JJ, print, update progress or mutate filesystem state. The library
owns paired recursion and opaque one-sided recursion in their existing places.

For example, at `node_modules/pkg` with two existing receivers:

1. Inspect the source directory once and each receiver's directory separately.
2. Acquire the source facts needed by either lane. A sees `index.js` equal;
   B sees it changed. Each lane still makes its own decision.
3. Accept A's no-op application and reflink the changed leaf into B.
4. Advance the shared traversal to the next admitted child with its lane membership.

If B instead lacks the whole `pkg` directory, B's current application recursively
copies that subtree. A's paired descent remains the frontier's responsibility.
The two source visits in that mixed case are a real limit of today's sharing,
not something an application-side loop should conceal or try to fix by recursing
over the same edges itself.

Use the production cursor process-locally. Owned continuation is available if a
later operator workflow needs pausing, but ordinary seed/reseed does not require a
transport format. A traversal checkpoint is not recovery for half a JJ transition
or the internal progress of a failed subtree copy.

### 4. Finish each successful receiver

Apply configured links and complete the native tree/parent/identity checks.
Creation records its lifecycle result; reseed preserves the existing lifecycle
identity. A completed transfer is necessary but not sufficient for a successful
workspace operation. Report failures at their actual phase, and preserve successful
peer outcomes when other lanes fail. Any failure makes the overall command fail.

Recommend allowing healthy peers to finish after a returned lane-local failure;
state that behavior deliberately in the command tests and result format. A shared
source/binding failure may affect many lanes and must retain that provenance.

### Results that explain what happened

Keep a compact per-workspace result with native phase, final status, source/desired
commit, transfer coverage and actual work. For human output, the useful distinctions
are `not started`, `native refill failed`, `transfer failed`, `finish failed`, and
`complete`; exact labels are an implementation choice. Observed differing entries,
selected actions and completed recursive installs/deletes are different counters.

Progress comes from returned groups and accepted applications. An opaque subtree
can be large, so a group count is not a percentage or latency bound. Collecting a
total with an extra pre-counting traversal would defeat the purpose.

Keep dry-run's existing joint read-only preview contract, including one-sided
interiors and type-replacement removability. Reuse the same frozen policy even
if preview initially remains one library call per target. A read-only coordinated
comparison alone does not inspect every opaque subtree and cannot substitute for
that preview. Execution must not gain an obligatory full payload preview pass.

## Landing sequence and review gates

Each stage should leave a coherent, committed implementation and explain the work
removed. No calendar estimates are needed.

### A. Restore a buildable integration baseline

Update package/path coordinates and feature wiring for the relocated library;
update the lockfile and native performance-tool consumers as needed. Capture the
exact application/library revisions and run the focused current-contract checks.
This baseline must still use the existing paired execution, so subsequent
comparisons isolate coordination from package/API repair.

**Gate:** one reproducible baseline and a complete call-site/dependency map.

### B. Seed one vertical slice

Add the thin batch-transfer adapter using the production cursor/runtime. Exercise
it through newly created receivers with N=1 and N=2, including the actual native
finish and per-target creation failure/cleanup behavior. Keep discovery, links and
registration with their owners. Reuse the listed-leaf installer for tracked files.

**Gate:** explain the new source→group→lane-application loop in one short function;
prove exact tracked result, private copied payload, exclusions and collision
behavior. Stop expanding seed once this shared seam is established. Deep first-copy
fanout optimization is a separate library opportunity, not this stage's payoff.

### C. Make reseed consume the same batch adapter

Reshape the current per-target orchestration into shared capture, serial native
establishment, coordinated auxiliary reconciliation, and per-target finish. Preserve
selected payloads through tracked cleanup so repeated reseeds actually compare
existing data. Keep source/desired distinctions and explicit replacement behavior.
Convert errors into named per-target outcomes instead of losing the partial batch
behind the first returned error.

**Gate:** N=1 behavior, mixed N=2 destinations, and the existing native semantic
regressions pass. Repeating a clean payload transfer performs zero payload writes;
tracked reflink renewal is accounted separately. The cohort contains only full,
native-ready receivers. Failed lanes do not become successful from comparison
exhaustion alone.

### D. Remove redundant orchestration and prove the gain

Hoist equivalent source facts/proofs out of target loops; retain genuinely
target-specific checks. Remove superseded paired-loop plumbing and duplicate
policy construction once the coordinated route is verified. Preserve the reason
for each remaining native check beside its phase. A code-size reduction without
fewer concepts or repeated operations is insufficient.

Compare the dependency-updated paired baseline with coordinated execution under
identical transfer policy. Start with one donor and at most two tiny receivers;
include clean, small-delta, missing-subtree and mixed-policy shapes. Attribute
shared source acquisitions, target acquisitions, transfer work and JJ/Git calls
separately from elapsed command time.

**Gate:** correct outputs, demonstrated sharing in the intended repeated-reseed
case, no hidden growth in native per-file processes, and an explicit N=1 comparison.
If coordinated overhead or lost inter-directory parallelism loses on an important
shape, choose the execution strategy within this adapter from evidence rather than
forking the policy/native lifecycle implementation.

### E. Select the next simplification from the remaining cost

There are two substantial independent choices after the useful integration:

1. **Transfer authority:** keep selected roots; broadly update ordinary data while
   retaining target-only locals; or mirror all non-preserved ordinary data. State
   source-present overwrite, source-absent deletion and retained-local outcomes
   using concrete examples before retiring `copy_ignored`. An old empty selection
   cannot quietly become copy-everything.
2. **Native materialization:** investigate source-state reuse or another native
   transition if tracked clearing/renewal/hash work dominates. Test the already
   identified retained-membership edge as well as D=S and D≠S.

These are ways to make JW substantially smaller, beyond the batch adapter.
The measurements and desired product behavior should select which one is next.

## Evidence plan

### Semantic cases

Use tiny owned fixtures with a fixed maximum of 256 payload entries and 1 MiB
payload bytes per case, including staging. Cases should discriminate:

- two destinations with different missing/changed/extra paths and exclusions;
- a directory paired in one lane and absent or a leaf collision in another;
- complete source-only subtree copying with exclusions enforced inside it;
- unchanged payload inode/mtime, changed private-reflink bytes, literal symlinks;
- target-local failure with healthy-peer completion and honest overall failure;
- native source/desired disagreement, tracked deletion/mode/type changes, retained
  locals, root/`.jj` identity, exact final parent and history coverage;
- creation fallback versus strict auxiliary/reseed reflink requirements;
- explicit replace-content and empty/disabled copy selection;
- link failure after successful transfer, reported as a native finish failure.

Existing upstream coordination tests can establish library acquisition and lane
semantics; JW needs native composition evidence, not a duplicate library suite.
Use the existing nextest stack and native executable controls. Avoid test shims
resolving recursively through mise; the prior tests use `/usr/local/bin` first.

### Performance and resource evidence

The historical [actual workload](../up-performance/primary0.sol61m.md#cancelled-campaign-retained-observations-not-completed-matrix)
had roughly 181k payload leaves and five completed native samples before the
campaign was cancelled. It supports the importance of preserving unchanged
payloads, not a speed claim for this new coordinator.

The first new measurement should reuse a tiny fixture, isolate operation time
from setup/checking, and establish source sharing and N=1 overhead. Extend the
existing Rust tooling only where necessary; its old full-tree resets and imports
are not a suitable default for this question. An eventual representative warm
reseed should have one declared question, one reused donor/receiver set and a
bounded run sequence. Record preparation, tracked refill, auxiliary transfer,
finish, CPU and peak memory separately. Never attribute the whole command gain
to fewer source stats without that breakdown.

## Connections and open decisions

- [Current operator contract](/docs/reseed.md) owns the actual default and
  preparation-only locking. The older contention investigation describes a prior
  global-lock implementation; removing that lock is not a prerequisite here.
- [Visitor checkpoint](../sync-scope/visitor0.sol61xh.md) shows that recursive copy,
  listed-leaf installation and clearing already use the library. The remaining
  opportunity is coherent orchestration and shared observation, not extracting
  the same kernels a second time.
- [Earlier authority design](../sync-scope/design0.gpt6a.md) supplies the transfer
  policy alternatives; [the correction](../sync-scope/implementation0.gpt6a.md)
  owns their unselected standing.
- [Library architecture](file:///home/rektide/src/diff-dirs/doc/architecture/README.md)
  owns the delivered sharing/barrier boundaries. Create a library-side
  `doc/consumer/jj-waltz.md` when landing adoption, linking these results back to
  native implementation. Record one-sided source sharing, batch leaf installation
  and preview support as consumer needs only where the integration actually shows
  their value.

**First implementation commission:** A→B→C, with review after the seed slice and
after the actual reseed cohort works. The target is one understandable refill
pipeline and evidence of repeated-reseed value, followed by measured simplification.
