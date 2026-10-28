---
type: Review
title: Independent review of slice-one native refill and ordinary authority
description: Two reproduced correctness defects, remaining native and policy costs, and consequential product choices before the coordinated CLI replacement.
resource: /design/coordinated-refill/review0.gpt6ax.md
status: completed-review-of-pinned-slice1
generated: { by: "model:openai/gpt-6-astra#xhigh", at: "2026-10-28" }
sources:
  - resource: "opencode://ses_e7bb7c240ffegKXHiRXMZA04bW"
    title: Independent committed-source review, two isolated reproductions, and reseed product steering; no production changes or delegated agents
  - resource: /design/coordinated-refill/implementation1.sol61x.md
    revision: ec7fe09f
    title: Delivered slice and attributed verification account
  - resource: /design/coordinated-refill/implementation0.gpt6ax.md
    revision: 549d2beb
    title: Selected implementation commission and later slice boundaries
  - resource: /design/coordinated-refill/architecture0.gpt6a.md
    revision: 549d2beb
    title: Parent checkpoint, phase ownership and explicit costs
  - resource: /design/coordinated-refill/assessment1.gpt6a.md
    revision: 549d2beb
    title: Human's reseed-first and forward-failure direction
  - resource: /refill/engine/src/engine/replacement.rs
    revision: 2d5ebda9
    title: Actual native orchestration, membership, admission and finishing
  - resource: /refill/engine/src/engine.rs
    revision: 2d5ebda9
    title: Inventory, repeated topology discovery, preservation and native profile helpers
  - resource: /refill/engine/src/engine/bookmarks.rs
    revision: 2d5ebda9
    title: Retained history gate and batched ancestry proof
  - resource: /transfer/workspace/src/streaming.rs
    revision: 2d5ebda9
    title: Pure Q-ancestor policy and one-lane sequential production cursor
  - resource: /transfer/workspace/src/frozen.rs
    revision: 2d5ebda9
    title: Frozen ignore matching, control-file extraction and differential corpus
  - resource: /transfer/workspace/src/ignored.rs
    revision: 2d5ebda9
    title: Selector automaton, Unicode conversion and original directory discovery
  - resource: /transfer/workspace/src/tracked.rs
    revision: 2d5ebda9
    title: Listed strict installation and no-follow parent checks
  - resource: /native/jj/src/jj.rs
    revision: 2d5ebda9
    title: Exact native arguments, semantic version parsing and tree-state-only adoption
  - resource: /native/jj/src/context.rs
    revision: 2d5ebda9
    title: Root discovery's repeated workspace queries
  - resource: /workspace/state/src/links.rs
    revision: 2d5ebda9
    title: Loaded-target normalization and forward versus legacy application
  - resource: /app/cli/tests/refill_one.rs
    revision: 2d5ebda9
    title: Native result, obstruction, extent and argv assertions inspected without rerunning
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/apply.rs"
    revision: 5c4192e7
    title: Strict seed installer and opaque source-copy, removal and replacement invocation forms
  - resource: "file:///home/rektide/src/diff-dirs/tree/inspect/src/visitor.rs"
    revision: 5c4192e7
    title: Actual early and late presence, kinds and logical paths
  - resource: "https://github.com/jj-vcs/jj/blob/33d3a595429b7e3b54a8ea769560247913bc8a1b/lib/src/commit.rs"
    title: Conflict flag and ordinary single-parent tree-equality cost, read from local archive
  - resource: "file:///home/rektide/src/jj-waltz/.test-agent/coordinated-refill/review/README.md"
    title: Owned isolated probes against extracted committed sources and a separate target directory
verification:
  method: Pinned source inspection and one isolated probe executable using committed JW and diff-dirs snapshots
  result: Reproduced unselected non-UTF-8 refusal and changed link destination; prior native and library test results attributed to Sol
  scope: Slice-one code through 2d5ebda9, account ec7fe09f, parent checkpoint 549d2beb; excludes later slices, throughput, broad test reruns and native-version certification
---

# Independent slice-one review

**Continue the selected native design; correct two reproduced authority/finish
defects before CLI adoption.** The central namespace protocol is substantially
sound under the prepared, idle donor premise. The larger steering opportunity is
deciding what everyday reseed should promise: this implementation renews tracked
CoW first, moves to D when requested, and incrementally mirrors selected auxiliary
directories. Those are distinct jobs with different costs.

All JW code references below mean **`2d5ebda9:<path>`**; line ranges refer to that
revision. The diff-dirs contract is **`5c4192e7`**. Mutable work from subsequent
slices was neither reviewed nor treated as delivered. The two user-owned drafts
were untouched. Parent retains index ownership.

## Correctness findings requiring action

### 1. P2 — loaded link normalization silently changes the destination

**Location:** [links.rs](/workspace/state/src/links.rs),
`LoadedWorkspaceLinks::new`, lines 100–104; `normalize_lexical`, 694–715.

The new seam replaces the declared target with a lexically normalized path before
inspection and application. On Unix, removing `component/..` is not equivalent to
resolving that path when `component` is a symlink.

**Reproduced through the actual public APIs:** receiver `jump` links to
`outside/deep`; both receiver `shared` and `outside/shared` exist. With
`source='shortcut', target='jump/../shared'`, loaded forward finish succeeds and
points at **receiver/shared**. The same declaration through the existing
`apply_workspace_links` points at **outside/shared**, as filesystem resolution
requires. This is a successful wrong result, not an interruption hazard.

**Action:** preserve the declared target's filesystem meaning through finish.
Keep private-directory classification separate from rewriting the target that
will be installed. A focused regression should compare these two destinations;
no payload prewalk or new recovery behavior is involved.

### 2. P2 — an unselected target-only byte filename aborts ordinary transfer

**Location:** [streaming.rs](/transfer/workspace/src/streaming.rs),
`candidate`/`disposition`, lines 36–50 and 66;
[ignored.rs](/transfer/workspace/src/ignored.rs), `path_prefix`, 186–194.

`disposition` evaluates selector candidates before deciding that a source-absent
name outside an admitted root must be preserved. `path_prefix` requires UTF-8.

**Reproduced through `sync_one`:** empty source, target containing one regular file
whose basename is byte `FF`, selector `cache`, empty frozen ignores. The cursor
fails with `directory selection entry not UTF-8`, despite that file having no
source and no selected ancestor. It remains intact and ordinary effects are zero.
In the complete refill, this phase follows native mutation. The prior discovery
path does not visit this target-local leaf at all.

**Action:** avoid decoding an ineligible leaf as a possible selected directory;
preserve source-absent unselected names while retaining deletion authority inside
an already reachable Q ancestor. A genuinely needed non-UTF-8 routing directory
is a separate supported-name question. Do not broaden selection or add a census
to fix this specific regression.

Both findings were confirmed in one small
[isolated probe](file:///home/rektide/src/jj-waltz/.test-agent/coordinated-refill/review/README.md)
using extracted committed sources, an independent Cargo target, offline dependencies
and two build jobs. No native workspace or production code was changed.

## Architectural observations

### Highest leverage: root/query reuse before N2 expansion

The replacement has **no ordinary payload-discovery or eligibility prewalk**.
However, root preparation still reuses expensive general discovery:

- [engine.rs](/refill/engine/src/engine.rs) `Inventory::load` (690–740) queries every
  registered root, then calls `checked_context` for caller and primary.
- [replacement.rs](/refill/engine/src/engine/replacement.rs) calls it again for the
  donor (104), and `validate_target` calls it for the receiver.
- [context.rs](/native/jj/src/context.rs) `probe_jj`/`find_primary_workspace`
  (274–335, 406–432) repeat workspace listings and root queries to find the owner.
  Caller, donor and primary commonly identify the same checkout.
- `verify_native` runs twice (replacement lines 260, 285), each with sparse-state,
  binding, name, root and commit queries. `ensure_quiescence` reads the monitor
  setting twice per root. Ownership validation also re-queries registrations.

These are metadata/process costs, **not filesystem-wide payload passes**. Retain
the actual identity checks, but carry resolved facts rather than rediscovering
them. A per-target port of these helpers can grow toward N×registered-workspaces
queries; repeatedly calling the entire N1 seam is worse. The planned command-owned
resolution/cohort is the right next integration boundary. No new framework is
needed.

### The native ledger is honest, with real irreducible work in this route

The inspected sequence agrees with Sol's trace: D=S has three full receiver
snapshots—preservation `status`, sparse clear, final diff—plus `new`'s empty-sparse
snapshot. D≠S adds restore's adopted-S snapshot and native S→D materialization.
Default consumption does not snapshot the donor; explicit A adds one live diff.
These stock-command setup costs cannot be removed by relabeling them as queries.

`Membership::capture` reuses exact coordinates and proven empty-single-parent
trees. Ordinary single-parent `empty` is index/tree-ID comparison in the inspected
JJ source; `conflict` is a tree-ID flag, not a hidden full conflict-file listing.
Merge-parent equality can require merging parent trees. Real `file list`
enumerations, `.gitignore` `file show` calls, cache read/write and per-leaf install
remain separately chargeable. Residual O and D-addition checks stat listed routes,
not all retained directory contents.

### Reuse data structures as well as repository queries

The membership `Arc<Vec<Entry>>` sharing is useful. But the path union clones S/D
names; `tracked::seed` builds a new membership map; `seed_leaves` owns and sorts
another known-leaf list; `admit_additions` builds another S set. O membership and
captured cache bytes remain scoped to the whole function. These are bounded,
honest O(tracked-path) structures, not an ordinary-file catalog; repeating their
construction per receiver is still avoidable. Share indexes where actually used
and release native-only inputs after the native phase.

Policy work is pure but not cheap by construction: `candidate` replays the
selector from the root at every prefix, then early and late repeat it. The
automaton allocates sets/vectors; `FrozenIgnore::ignored` builds an ancestor vector
and searches applicable parent matchers. Broad matching with no eligible ancestor
can compound depth costs substantially. Paired N2 source-fact sharing does not
share those per-lane decisions. This is a measurement/optimization candidate,
**not a blocker or a reason to introduce mutable admission history**.

## Product decisions worth human steering

1. **Is renewing tracked CoW on every reseed the defining promise?** The current
   route clears and reinstalls every S leaf, even on a clean repeat. That is a
   tracked-sharing regeneration operation, not an incremental tracked update.
   Selected auxiliary data is incremental and can produce zero clean-repeat
   effects. If the desired job is instead principally “move this workspace to D
   and refresh dependencies,” that would justify comparing a different upfront
   native route. It would change the sharing promise; silently skipping equal
   tracked files would not satisfy the present one.

2. **How central is arbitrary D≠S?** Full-S prefill makes membership authority
   explainable and protects locals by refusing obstructions. It also clones
   S-only files just to remove them, clones changed files before native replacement,
   and refuses an occupied S-only local even when D never wants that name.
   Installed-reflink counts are not a census of final D sharing: D-only/changed
   files come from native materialization. Keep this uniform route if these costs
   and restrictions are acceptable; a distinct upfront D≠S route is a consequential
   product choice, not an error-triggered fallback.

3. **Should selected auxiliary names mean source-qualified refresh or declared
   mirrors?** Current behavior preserves a receiver's `cache/` if the donor's
   entire `cache/` disappears, but deletes stale children while an eligible source
   `cache/` exists. Empty selection copies nothing; `.env` remains local unless
   native initial auto-tracking claims it or explicit clearing is requested.
   This is coherent and preserves selected-ignored scope. Declared-root deletion
   on donor absence, broader copying, or bypassing ignoredness are real authority
   changes. None is necessary to finish the commissioned implementation.

4. **Which everyday admission constraints are intentional?** The history gate
   requires a named reference covering the saved work tip, including dirty work;
   a bookmark only on an older ancestor does not qualify. Saving O and requiring
   that reference are separate policies. Removing the gate alone does not remove
   the current preservation snapshot. Separately, the new route rejects every
   semantic JJ version except 0.45.1. It admits any 40-hex-suffixed build reporting
   that version, while evidence covers one build. Decide a usable supported range
   with focused native evidence before CLI rollout; the present equality check is
   an interim boundary, not an exact-build compatibility proof.

## Positive evidence and limits

- **Native namespace ownership is explicit.** Strict `seed_leaves` rejects occupied
  S leaves, including stat/byte-equal locals, before importing membership. D-only
  routes refuse unrelated names. Residual O paths are refused through no-follow
  parents. S-owned shape transitions remain native work; retained children can
  make final conformance fail without being recursively erased.
- **Adoption preserves receiving identity.** Only `tree_state` is written; checkout
  bytes are checked across that write. Exact D parent, receiving change/name/root,
  repository, registration and full sparse state are checked. The live final diff,
  rather than successful restore or full sparse alone, establishes the native
  result. Recorded S remains an assertion about a prepared donor, not a byte proof.
- **Q-ancestor authority is defensible at workspace-root binding.** Actual source
  directory eligibility precedes boundary admission; Unknown kinds defer to late.
  Native/link overlap is checked before mirror admission. Target-only candidates
  stop; target-only descendants of admitted roots may be removed. The pinned
  kernel preserves logical paths during opaque source-copy, target-removal and
  replacement-removability recursion, so these forms do not need a mutable map.
  Rebinding an arbitrary interior subtree would invalidate that argument.
- **Matcher and failure evidence is meaningful but bounded.** Frozen matching has
  no per-entry I/O, handles ancestor exclusion and layered precedence, and has a
  bounded Git differential corpus. Sol reports nine required-CoW native tests,
  34 transfer cases, FIEMAP/private-write evidence and focused feature checks;
  those results are trusted, not rerun here. No throughput conclusion follows.
- **Delivered boundary is still callable N1.** The CLI uses the old engine at this
  pin; new prepare/preview/cohort seams are absent. Missing unselected routes
  explicitly refuse. New cursor execution is sequential even with the feature
  enabled. Conditional routing belongs to the separately commissioned next slice;
  this review neither duplicates nor certifies it.

The selected plan aligns with the human's whole-reseed request **if CLI replacement,
single preparation handoff, honest preview, shared N2 inputs and old-driver removal
remain the landing milestone**. The current seam is useful native/authority
evidence, not completion of the everyday product. Fix the two narrow defects,
settle the few promises above, and evaluate whole-command native versus ordinary
cost at that milestone.

## Connections

- The [implementation account](implementation1.sol61x.md) supplies the attributed
  native evidence; this review adds independent contract inspection and two
  reproduced gaps, rather than superseding its historical account.
- The [commission](implementation0.gpt6ax.md#9-coherent-build-slices-and-parent-checkpoints)
  locates the remaining delivery boundary. The
  [architecture's data lifetime section](architecture0.gpt6a.md#9-data-lifetime-and-cost-accounting)
  is the basis for the root-query and shared-index observations above.
- [Human lifecycle direction](assessment1.gpt6a.md#direction) grounds the steering
  questions: everyday reseed usefulness and forward failure decide the outcome,
  while broader ordinary-data authority still requires an actual choice.
