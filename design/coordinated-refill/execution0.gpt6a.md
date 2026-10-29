---
type: ImplementationDesign
title: Coordinated refill execution — shared command and two renewal routes
description: Sol-ready command, producer, inherited-policy, native replacement, reporting and public-cutover contracts, with bounded installed-JJ replacement evidence.
resource: /design/coordinated-refill/execution0.gpt6a.md
status: commissioned-design-with-bounded-protocol-proof
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e7a26147bffeOIk9jHWPYXJrHI"
    title: Designs the JW execution boundary, proves same-tree-child authorized replacement and native reparenting on the installed JJ, and prepares implementation/API handoff
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions Astra designs then Sol xhigh execution of annealing, pulling forward architectural contracts and supporting both proactive renewal routes
  - resource: "file:///home/rektide/src/diff-dirs/design/architecture/annealing0.gpt6ax.md"
    title: Selected renewal baseline, two mechanisms, four lifetimes and contextual persisted-provider requirements
  - resource: /design/coordinated-refill/trajectory1.gpt6ax.md
    title: Whole-command destination, shared inputs, preparation, preview, N2 and bounded evidence
  - resource: /design/coordinated-refill/work-map0.gpt6ax.md
    title: Existing native/provider work ownership, revised by the annealing commission
  - resource: /design/coordinated-refill/implementation0.gpt6ax.md
    title: Clear-first protocol, source-qualified ordinary authority and prior native proof
  - resource: /design/coordinated-refill/assessment1.gpt6a.md
    title: Human-selected upfront routes, whole-command redesign and forward failure
  - resource: /design/coordinated-refill/reseed-checkin0.gpt6ax.md
    title: Native/source-sharing cost boundaries and equality versus storage evidence
  - resource: /design/sync-scope/jj-state-review0.gpt6a.md
    title: Separate receiving identity, tracked membership and reusable cache
  - resource: "file:///home/rektide/src/diff-dirs/design/native/graph1.gpt61sxh.md"
    title: Truthful borrowed membership still absorbs matching retained locals; native conformance alone cannot certify preservation
  - resource: /docs/reseed.md
    title: Existing public preparation, payload-verification and snapshotting-preview promises requiring deliberate cutover
  - resource: "file:///home/rektide/src/diff-dirs/doc/consumer/jj-waltz.md"
    title: Maintained consumer requirements, native/provider boundary and current sharing limits
  - resource: /refill/engine/src/engine/replacement.rs
    revision: 22a8224e
    title: Current callable N1 native sequence, membership cache and finish
  - resource: /refill/engine/src/engine.rs
    revision: 22a8224e
    title: Registry discovery, current target/profile guards, preparation lock and legacy driver
  - resource: /refill/engine/src/engine/prepare.rs
    title: Existing producer checks, per-step conformance and repeated payload audits
  - resource: /refill/engine/src/freshen/source.rs
    title: Pinned upstream identity, publication and fast-forward checks
  - resource: /transfer/workspace/src/streaming.rs
    title: Current prefix-replayed source qualification, residual guards and one-lane runtime
  - resource: /transfer/workspace/src/frozen.rs
    title: Frozen ignore semantics and tree-plus-effective-excludes sharing
  - resource: /transfer/workspace/src/tracked.rs
    title: Native leaf enumeration, confinement and strict required-clone adapter
  - resource: /native/jj/src/jj.rs
    title: Fixed native mutation configuration, exact-ID queries and forward tree-state adoption
  - resource: /workspace/state/src/links.rs
    title: Loaded declaration semantics and forward finish losing structured partial counts today
  - resource: /app/cli/src/cli.rs
    title: Actual request conversion, preview wording and report cutover sites
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/apply.rs"
    title: Production opaque execution, retained descendants, staging and returned progress
  - resource: "https://github.com/jj-vcs/jj/blob/33d3a595429b7e3b54a8ea769560247913bc8a1b/lib/src/rewrite.rs"
    title: Installed-build source inspected from the archive; identical rebase destination and changed tree resolve to that tree and empty commits can be kept
  - resource: "file:///home/rektide/src/jj-waltz/.test-agent/coordinated-refill/replacement-probe/README.md"
    title: Owned bounded native-protocol prototype and local receipt
verification:
  method: Source inspection at JW parent 22a8224e and six bounded protocol/obstruction cases with installed JJ and required Linux FICLONE
  result: D=S, D!=S, clean renewal and the three modeled obstruction cases passed; exact native parent/membership, history, identity, donor stability and private-write checks passed
  scope: Native-sequence and small authority-model evidence; replacement uses a scratch effect model, not a production diff-dirs replacement implementation, version matrix or timing result
extensions:
  related_work: [jjwaltz-reseed-coordinated, jjwaltz-reseed-coordinated-context, jjwaltz-reseed-coordinated-contract, jjwaltz-reseed-coordinated-cohort, jjwaltz-reseed-coordinated-prepare, jjwaltz-reseed-coordinated-preview, jjwaltz-reseed-coordinated-cli, jjwaltz-reseed-coordinated-evidence]
---

# One command, two constructive renewal routes

**Build both `ClearThenPopulate` and `ReplaceAuthorized` around shared native
inputs, then run one ordinary cohort and one loaded finish.** Retain clear-first
as the default/control while correctness and matched benchmarking establish the
tradeoff. Renewal is selected: equality alone never skips a tracked renewal.

This is the JW-side implementation design for the commissioned
[annealing assessment](file:///home/rektide/src/diff-dirs/design/architecture/annealing0.gpt6ax.md).
The independently commissioned provider design owns final inspect/compare/sync
signatures; this document specifies the consumer obligations and required handoff.
No production implementation is included in this stage.

## 1. Result contract

| Concern | Selected execution meaning |
| --- | --- |
| S / D / Oᵢ | Prepared donor WC commit / exact desired parent, default S / preserved receiver WC commit. Commit identity and tree equivalence remain separate. |
| K(X) | Validated recorded tracked-leaf membership of revision X, including file/link kind and executable bit; not a live payload census. |
| Native success | Original root, `.jj`, repository and workspace registration survive; receiver is a full-sparse, conflict-free empty single-parent child of exact D, with its own newly created change identity. |
| Tracked renewal | Install every K(S) leaf proactively. Regular files are fresh private required clones; symlinks reproduce literal text. Never import S membership after a partial/subset installation. |
| D≠S | Install full S, then native restore establishes D. S-only installs can disappear; changed/additional D leaves can come from the store. The report promises **S installations**, not donor sharing for every final D leaf. |
| Ordinary data | An existing real source directory selected and ignored under frozen S/D rules admits a mirror. Vanished source roots retain target contents. Empty/disabled selection does no ordinary traversal. |
| Ordinary storage | Changed/new regular leaves use required private clones. Stat-equal retained leaves are not certified for private inodes, bytes or donor extent sharing. |
| Locals | Unowned occupied S-only/D-only names remain obstructions, even with matching content/stats. Preserved descendants veto coupled replacement. `.env` outside admitted authority survives. |
| History | Initial receiver snapshots and existing default bookmark coverage/bypass remain. Report exact O and covering references; create no bookmarks automatically. |
| Failure | Stop at the first surfaced error, retaining actual returned effects and the original cause. No route fallback, batch rollback or automatic resume. |

`--replace-content` remains its separate explicit broad-clear authority. It does
not broaden donor selection. Neither renewal route authorizes colocated receivers,
externally owned workspaces, or pruning unrelated empty directories.

## 2. Concrete command/source ownership

Use concrete structures in the existing six owners. The following Rust-like
shapes are implementation coordinates; wrappers for IDs can be lightweight newtypes.

```rust,ignore
enum RenewalRoute { ClearThenPopulate, ReplaceAuthorized }

struct BoundWorkspace {
    name: String,
    root: PathBuf,
    root_identity: Identity,
    jj_identity: Identity,
    repository: Arc<RepositoryBinding>,
    native_profile: NativeProfile,
    ownership: CheckedOwnership,
}
struct ResolvedCommand {
    repository: Arc<RepositoryBinding>,
    bindings: RegistrySnapshot,           // one operation, roots and registrations
    source: Arc<BoundWorkspace>,
    receivers: Vec<ReceiverIntent>,      // stable selection order, no payload census
    policy: Arc<EffectiveDeclarations>,  // config, patterns, argv, loaded link config
    route: RenewalRoute,
    source_intent: SourceIntent,         // Existing { assertion } or Prepare { goal }
    desired: DesiredIntent,              // Source or ExplicitRevset
}
struct PreparedSource {
    binding: Arc<BoundWorkspace>,
    at_operation: OperationId,
    source_commit: CommitId,             // actual resulting donor WC S
    source_change: ChangeId,
    upstream_goal: Option<CommitId>,      // never silently substituted for S
    evidence: SourceEvidence,            // AssumedPrepared / LiveAssertion / Produced
    preparation: Option<PreparationReport>,
}
struct NativeTree {
    key: TreeKey,                        // exact/equivalent coordinate with proof
    leaves: Arc<[TrackedEntry]>,
    index: Arc<NativePathIndex>,         // leaf lookup + ancestor/descendant queries
}
struct NativeInputs {
    source: Arc<NativeTree>,
    desired: Arc<NativeTree>,
    old_trees: BTreeMap<TreeKey, Arc<NativeTree>>,
    donor_tree_state: Arc<[u8]>,
    source_desired_ownership: Arc<NativePathIndex>,
    desired_additions: Arc<[LeafId]>,     // D names not exactly in S
    ordinary: Arc<OrdinarySourceInputs>, // patterns, S ignores, S/D native union
}
struct SavedReceiver {
    binding: Arc<BoundWorkspace>,
    old_commit: CommitId,
    history: HistoryCoverage,
    old_tree: Arc<NativeTree>,
    policy: Arc<OrdinaryLaneInputs>,     // effective D ignores, active links, devices
    links: LoadedWorkspaceLinks,
}
```

`PreparedSource` is an in-process producer result, not a persistent freshness token
for ignored payload. `SourceEvidence::AssumedPrepared` explicitly records the
plain-reseed premise. It must not be formatted as a live donor proof.

### Capture and release rules

1. Capture one lean registry/root view and root configuration. Reuse canonical
   root identities when caller, primary and donor coincide. Refactor context
   discovery to accept the supplied registration/repository view instead of
   rediscovering the primary for every receiver. Actual target ownership checks
   remain per target. Existing `workspace_bindings_at` is already lean.
2. Separate stable binding facts from changing native revisions. A native command
   refreshes revision facts at its new operation, not the whole root/config context.
3. Resolve declarations and source choice first. The preview branch returns here.
   Execution selects the supported route once and performs required-CoW probes
   before destructive refill. Probe once per genuinely equivalent filesystem route;
   sharing a device number alone need not prove all route-specific permissions.
4. Produce S or consume recorded S. A plain explicit A performs one named live
   diff, then pins the resulting donor WC as S. Resolve explicit D at the intended
   post-producer repository view; omitted D becomes S.
5. Snapshot receivers serially, recording their O values immediately. Check
   history coverage from one subsequent repository view. At the final preservation
   barrier require source S and saved receiver bindings still name their captured
   coordinates. A later snapshot can rewrite shared history; refuse movement
   before destructive refill instead of silently repinning earlier receivers.
6. Capture K(S), K(D), and K(Oᵢ) once per distinct proven tree. Current empty-child
   equivalence is useful; it is not a global tree-ID service. Build indexes once,
   including source/desired union and desired-addition list. Do not re-collect path
   sets in each native helper. Share rule compilation by **tree and effective
   excludes**, not tree alone.
7. Capture donor `tree_state` once after the preservation barrier. Freeze relevant
   control bytes/excludes once. Validate native/active-link overlaps from declarations
   and memberships. There is still no ordinary payload prewalk.
8. Run complete native transitions serially. Once all succeed, release donor cache
   bytes, old-tree membership and native-only plans. Retain O coordinates/history
   in reports and only the S/D ordinary exclusion index/rules required below.

Command-owned runtime resources are separate from semantic inputs: optional worker
pool, observation backend and buffers do not enter policy context or report meaning.
No durable native-workflow store is needed.

## 3. Source-only preparation and handoff

Extract `prepare_source(ResolvedSourceIntent) -> Result<PreparedSource,
PreparationFailure>` from the old `run(request, source_only)` branch. Standalone
prepare must not enumerate receiver inventory, load receiver link plans, construct
ordinary visitors or realize payload roots. It may resolve the primary/source
registration needed for ownership and native source selection.

Retain the existing preparation-only lock, pinned remote/bookmark identity,
publication/fast-forward checks and configured ordered argv. Release the lock
before consuming the returned handoff in refill. Refill's donor-coordinate barrier
is a read of that coordinate, not another tracked-tree conformance proof.

The successful producer contract becomes **final tracked conformance**:

1. Save source work with the existing explicit snapshot/publication guard.
2. Select and fetch the pinned upstream, establish the intended source child.
3. Run every configured argv in order, checking ordinary process exit and cheap
   root/registration/native-change identity at appropriate boundaries.
4. Perform one final live tracked diff against the exact goal, with auto-tracking
   disabled, then verify child/change/parent/full-sparse facts. Return actual S and
   goal separately.

An intermediate command may edit a tracked file if a later command restores it.
A failing command stops immediately; later commands do not get a repair attempt.
Changing the native workspace/change identity remains an error. The final check
also catches tracked changes from a command that exited successfully. This is a
deliberate semantic change from per-argv conformance, and needs a real two-command
test. No successful preparation claim is made for partially installed ignored data;
with no configured builder it remains existing source data.

Remove repeated payload validation/selection, `verify_disk`, and full source proofs
between argv steps. Keep actual native costs visible: initial snapshot, snapshotting
fetch, possible `new`, final diff, and program work. Do not turn final conformance
into a claim that those stock native commands avoid their implicit snapshots.

Public `prepare` should have its own `PrepareReport`, containing `source_commit_id`
for actual S and `upstream_goal_commit_id`. Existing scripts that used the reported
goal receive a documented schema transition; successful tree equivalence does not
make the two commit coordinates interchangeable.

## 4. Native routes and state truth

Both routes use `run_refill`'s three configuration overrides: monitoring disabled,
auto-track `none()`, stale auto-update disabled. Initial preservation keeps existing
auto-tracking policy. Quote exact coordinates through `commit_id_revset()`.

All mutation runs against the receiving checkout without `--ignore-working-copy`.
Repository-only reads use a captured operation. The target's `checkout` bytes,
workspace name and repo pointer are never copied from the donor.

### 4.1 ClearThenPopulate — executable control

Retain the current `replacement.rs` sequence:

```text
saved O
→ sparse set --clear; assert recorded O stayed fixed
→ refuse residual listed O leaves
→ new exact D while sparse-empty; capture new change C
→ optional explicit replace-content
→ strict full-S required installation
→ admit D additions against installed S / retained locals
→ adopt captured S tree_state, keeping receiving checkout identity
→ if D≠S: restore --from exact D
→ live final diff against D + full-sparse/empty-child/identity facts
```

Every adopted S membership entry is owned by a successful installation. A leftover
unrelated S-only name blocks strict installation even if equal. Keep the existing
listed D-addition and residual-refusal semantics. A full-sparse flag alone is not
proof of a healthy interrupted refill.

### 4.2 ReplaceAuthorized — same-tree child, renewal, convergence, reparent

**Do not simply omit sparse clear and run `new D`.** That would ask native checkout
to materialize D before renewal, changing ownership and cost. Use this separately
proved sequence instead:

| Step | Operation and invariant |
| --- | --- |
| R1 | `new exact O` creates a new child C with O's tree. Existing tracked files remain. Check parent O, empty change and unchanged O; retain C's change ID. Saved O will not be rewritten by later snapshots of C. |
| R2 | Optional explicit `--replace-content` runs here, retaining `.jj`. If used, subsequent S installation is strict-create into its actually cleared scope; this does not switch the selected native route. |
| R3 | Execute authorized K(O)→K(S) renewal. Replace common leaves even when equal; populate absent S leaves; remove obsolete O leaves. Namespace authority is derived from K(O), not from equality or donor membership alone. |
| R4 | Admit D additions using the same installed-S/retained-local rules as clear-first. Only after complete renewal and admission, adopt full S `tree_state`; receiving checkout bytes remain unchanged across this write. |
| R5 | If D≠S, run `restore --from exact D`. Its initial snapshot records installed S in C; its native checkout establishes D. O is still preserved. |
| R6 | `rebase -r @ -d exact D`, without `--skip-emptied`, `-s`, branch default, or insert flags. This reparents **only the freshly created receiving change**. D=S uses this command's initial snapshot to reconcile adopted S. D≠S enters it with D's tree after restore. |
| R7 | Live final diff against exact D, then verify empty single-parent child, full sparse, same C change ID, registration/root/repository identity. A successful restore/rebase exit is insufficient by itself. |

Why R6 is meaningful: C's tree entering the rebase is D. Rebase merges new-base D,
old-base O, changed tree D; identical sides resolve to D. The result is an empty
child of D with C's change identity. When O and D have equal trees, native rebase
can skip the merge. Only the new leaf change is moved; saved O, existing bookmarks
and existing descendants are not selected for rewriting. Under the idle-repository
premise it has no outside descendants. Use native checks and fail if that premise
or the recorded change identity changed; never broaden to a branch rebase.

This sequence deliberately pays an extra full snapshotting native step relative
to the clear-first control. It avoids full tracked clearing and same-name unlink/
recreate gaps during normal leaf renewal, but has no demonstrated latency advantage.

### 4.3 Replacement ownership program

JW lowers memberships into one immutable native renewal specification:

```rust,ignore
struct NativeRenewalSpec {
    desired: Arc<NativeTree>,       // every S leaf must be installed
    owned_before: Arc<NativeTree>,  // only O grants occupied-name authority
    devices: (DeviceId, DeviceId),
    destination: DestinationCondition,
}
enum DestinationCondition {
    StrictAvailable,                // clear-first, or successful explicit clear
    ReplaceOwned,                   // same-tree C still contains O
}
```

The provider-facing lowering needs explicit desired-leaf obligations and target
ownership, not a comparison trick. Preferred integration is a listed renewal seam
sharing the provider's normal selected-operation validation, staging and effect
kernels. It accepts desired paths plus a pure path-scoped authorization/context
adapter; it does not know JJ commits. If implemented through contextual inspection
instead, it must retain exactly these manifest coverage and preservation properties:

- Every desired regular/link coordinate is an obligation even when absent on both
  physical sides. Missing or wrong-kind/mode S fails; a name-union traversal must
  not silently omit it. Return full manifest coverage separately from effects.
- A fresh private clone is prepared before replacing a common owned regular/link
  name. Do not truncate/write through the old inode. Equal or unassessed content
  still selects installation under explicit renewal authority.
- S-only occupied leaves not owned by O fail. Existing directories can be routing
  containers, but an S leaf replacing a directory requires removability of the
  **whole coupled destination** under O authority. A retained local descendant
  vetoes that transition before any child of the blocked subtree is removed.
- O-leaf→S-directory expansion stages selected S descendants and replaces the
  owned old leaf through the existing coupled transition. It must not sweep in
  unselected physical source children merely because they share that directory.
- Remove obsolete exact O leaves. Keep structural target directories unless their
  removal is required by an authorized coupled transition. This introduces no
  background empty-directory pruning. A former O leaf now holding an unrelated
  directory is refused, not recursively erased.
- During directory collapse, subordinate O removals belong to the coupled
  replacement; do not first run a global O-minus-S deletion pass that partially
  empties a subtree before discovering a preserved descendant.
- Observe supported kinds, device boundaries, no-follow ancestry and administration
  at the actual operation boundary. O membership grants scope, not a permanent
  physical-route certificate. Fresh-file metadata must match the donor cache's
  reusable attributes; preserve the provider's existing timestamp/mode handoff.
- Complete coverage is mandatory before adoption. On failure retain exact returned
  progress, keep S cache unadopted, and report C/native state as incomplete. No
  post-failure census is needed to manufacture a success/failure report.

Before adoption, old native state can temporarily be stale relative to JW's partial
effects, as in existing refill. No unrelated JJ command is run during that window.
The design's truthfulness is: only import a state describing a **complete actual
S installation**, then require native reconciliation/conformance; never assert
that an interrupted mixed O/S tree already conforms to either tree.

### 4.4 Installed-build proof receipt

The local [prototype README](file:///home/rektide/src/jj-waltz/.test-agent/coordinated-refill/replacement-probe/README.md)
and `probe.py` exercised R1–R7 on
`jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`.
Binary SHA-256: `0b7462db466ffb0394cb30b0bb56480da3bce10a91f98b80b109302b0ec610d8`.

| Case | Observed result |
| --- | --- |
| O differs, D=S | Seven required FICLONEs and one symlink; three obsolete O leaves removed; expansion/collapse/link transitions; final native tree, exact parent and receiving C identity passed. |
| O differs, D≠S | Same S renewal; native source-only removal, desired-only addition, directory→file and executable-mode change; final exact D and C identity passed. |
| Clean repeat | Seven required FICLONEs again, one link, zero obsolete O removals; common inode changed during renewal and survived native reparenting. |
| Matching S-only local | Refused; local inode/bytes retained, prior renewal remained, no donor-cache adoption. |
| Matching D-only local | Refused before adoption/restore; retained local remained. |
| Collapse with ignored local descendant | Refused before coupled subtree removal; both recorded old descendant and local remained; no adoption. |

Success cases checked saved O/bookmark and old-file retrieval, full-sparse state,
native recorded leaf membership, unchanged root/`.jj`/repo pointer and registration,
unchanged donor coordinate/cache, private writes and native detection of a later
tracked edit. One donor/two receivers were reused. Peak ordinary fixture residency
including clone staging was **31 entries / 65,812 bytes**. Owned fixtures were removed
on exit. No blanket `TMPDIR` setting or production workspace operation was used.

Final script SHA-256:
`95ebb35568827f5dc73558aa1bc9c6b8f2b70e2fd39d206a968c3181181d558b`.
Receipt JSON SHA-256:
`5f3f6bb5e710cb26a7db5d95216e4c73c414819d6d817a4374dffc82c858bfa4`.
The receipt keeps actual IDs and command chronology in the scratch directory.

This proves a native transition is available. The scratch replacement model is
not production filesystem code, comprehensive authority verification, extent-range
measurement, macOS evidence, or a benchmark. Sol must exercise the same protocol
through the production effect primitive, including its coupled/failure semantics.

## 5. Inherited ordinary policy — implement now

Replace `SourceQualified::candidate`'s root-prefix replay with a small closed
policy-owned context. Bind one immutable source input and lane-specific rules.

```rust,ignore
enum OrdinaryRegion {
    Search(SearchState),
    SelectedMirror { root: RelativePath, admission: AdmissionKey },
}
struct SearchState {
    selector: SelectorState,             // remaining pattern automaton positions
    source_ignore: IgnoreSearchState,    // active rule chain / ancestor exclusion
    desired_ignore: IgnoreSearchState,
    ownership: OwnershipSearchState,     // narrowed native/link prefixes
}
```

`AdmissionKey` identifies this immutable policy meaning, not a mutable side table
of discovered roots. An in-process `Arc` can share context; the provider's saved
state must contain its reconstructible value/descriptor and validate it on rebind.
Do not bake pointer identity into persistence. Ordinary JW does not acquire a
public resume command merely because it uses that persisted-capable machine.

At a possible Q boundary, demand real source-directory facts before admission;
Unknown is not absence. Check selection, source/desired ignoredness and both-way
native/link overlap once. An actual source-qualified Q switches to SelectedMirror.
Below it do not rerun selector/ancestor-ignore/native-overlap admission. Descendants
carry the admitted context through paired traversal, opaque source copying,
target-only deletion and replacement-removability inspection. Residual device,
special-kind and nested `.jj`/`.git` guards still apply to equal leaves too.

Search-state descent advances selector components and the relevant frozen ignore
chain; ignored-parent state is sticky. A deeper negation cannot reopen an excluded
parent. Preserve existing UTF-8 directory-selector limits and byte-named interior
support. Active link sources remain whole-name exclusions; a private-directory
source==target rule is not an active link. Keep the reviewed symlink/`..` target
spelling semantics in loaded links.

Outside Q, target-only roots remain preserved, even when their names match a
selector. Native-owned source-directory collisions are checked before hiding the
name. Structural missing routes use `CreateIfNeeded`; no match produces no skeleton,
whereas a selected empty directory creates its ancestry. For target-only removals
inside Q, inherited admission supplies authority despite source absence.

The provider context must cross **both** frontier and opaque recursion. A context
returned only as a lowering result, or a root visitor reused inside every opaque
call, is insufficient. Restore must validate policy interpretation and pending
decisions without replaying accepted parent admission. JW's device guard can still
require metadata; inherited reasoning is not a claim that payload stats vanish.

## 6. One actual ordinary N1/N2 cohort

`sync_cohort(source, lanes, runtime_resources)` replaces `sync_one`. For any nonempty
ordinary policy, bind one `SyncRuntime` and one `SyncCursor` over **all native-ready
selected receivers**, with stable target IDs and their distinct policy values.
N1 uses the same function. Native operations remain serial; do not clear all peers
before completing the first native receiver.

```text
next_group
→ inspect every returned lane outcome; surface first error in stable lane order
→ apply every pending lane with that group's token, stopping on returned failure
→ settle
→ continue to exhaustion
→ require lane satisfaction, then finish receivers serially
```

Apply pending lanes even when immediate work count is zero. A selected opaque
operation can still do recursive work. Do not use healthy-peer continuation from
`drive`, a loop around N1, or a second lightweight traversal model.

The critical N2 case has A missing Q and B paired with stale content inside Q.
A gets inherited context in opaque population; B carries its context in the
paired frontier and removes only its authorized stale interior. A does not gain a
new paired-recursion owner after creation. Paired source acquisition is shared;
opaque interiors remain per lane and are counted separately. A different D-ignore
context in one receiver must not silently become the other receiver's admission.

After ordinary satisfaction, apply the already loaded links against actual final
paths, then perform snapshot-free native identity/parent/change checks. Reuse
stable bindings rather than calling full discovery again. Native exclusions and
link validation establish that ordinary/finish effects cannot modify tracked D.
Completion requires finish success. If native receiver B fails, A stays native-ready;
if ordinary B fails after A wrote, those writes stay; if finish B fails after A
finished, A can be reported complete. No later receiver is started after the error.

## 7. Structured phase reports and forward errors

Converge the temporary report and public API. Preserve one original `anyhow::Error`
chain in the failure; expose a compact serializable projection rather than parsing
diagnostic strings or serializing arbitrary library errors.

```rust,ignore
struct CommandFailure {
    report: CommandReport,
    location: FailureLocation,           // phase, optional target/path/argv step
    source: anyhow::Error,
}
struct ReceiverReport {
    name: String,
    root: PathBuf,
    route: RenewalRoute,
    old: Option<SavedWork>,
    desired: Option<CommitId>,
    child: Option<NativeChild>,
    reached: Phase,
    status: ReceiverStatus,              // Planned/NotStarted/Partial/Complete/Failed
    native: NativeOutcome,
    ordinary: OrdinaryOutcome,
    finish: FinishOutcome,
}
struct NativeOutcome {
    installed: Progress,                // actual S work, not final-D sharing count
    removed_owned: Progress,
    explicit_clear: Progress,
    manifest: Coverage,
    state: NativeEstablishment,          // Unstarted/InProgress/Established
    timings: NativeTimings,
}
```

`OrdinaryOutcome` distinguishes disabled, unstarted, partial and satisfied; it
keeps observed comparison separate from recursive completed effects. Keep command
wall time once, shared setup times once, and per-native/per-finish durations. The
ordinary cohort has one wall duration; lane application durations need not sum to
it. Retained caller views and shared buffers can extend lifetimes—do not promise
lower peak memory merely because an `Arc` is introduced.

Native CLI failures may have effects that a process exit cannot enumerate. Report
the command as attempted/failed and counts as unknown where appropriate. Do not
label K(O).len() as a successful deletion count or a successful child query as a
clone count. For provider calls use returned `Failure.completed`; for an inspection
failure take already-held cursor reports before returning.

**Fix an existing reporting loss:** loaded link finish currently returns an
`anyhow` string containing partial counts, and `LinkPlan.report.linked` is a planned
count. Add `LinkFinishFailure { completed, source, path }` and track actual created
links/directories as they succeed. Keep reversible lifecycle callers' wrapper.
Apply the same structured principle to preparation steps: selected/started/succeeded
argv and final-conformance status, without claiming exactly-once execution.

Use a command envelope with `schema_version`, `command`, `status`, source/selection,
per-receiver results, timings and optional error projection. Both successful and
failed JSON invocation emit one complete document on stdout; diagnostics go to
stderr and failure exits nonzero. An early resolution error can have zero resolved
receivers; do not invent rows or verified coordinates. Large progress integers
should use a documented lossless wire representation, such as decimal strings;
Rust aggregation remains u128-capable.

## 8. Truly nonmutating PLANNED preview

Preview shares read-only declaration/binding resolution and returns before producer,
CoW probes, preservation, cache adoption or transfer construction. Audit the entire
helper call graph: `--ignore-working-copy` is necessary but a command that integrates
divergent operations can still mutate repository state. Prefer reads at captured
`--at-operation` coordinates; metadata-store constructors must not create directories,
lock files or migration output. No preparation lock is acquired in preview.

Allowed work: read root identity, native layout/profile/sparse settings, current
recorded revisions, config/link declarations, group selection and prospective
source-goal references. No `status`, live diff, fetch, builder, `file list`, recursive
payload audit, filesystem write/probe, or all-target readiness preflight.

Render **`PLANNED — payload/path obstructions not inspected`**. Structured fields
carry the distinction too:

- Plain source S is *recorded/assumed prepared*, not live verified. A is *pending
  live assertion*. O preservation and coverage for the resulting O are pending.
- With prepare, show selected prospective remote/goal/argv; resulting S is unknown.
  Omitted D is *future prepared S*. An explicit D revset is *pending resolution in
  the post-prepare view*; a current preview resolution can be shown as provisional.
- Required clone capability is unprobed; selected native route is an intent.
  Static profile/topology errors may mark a plan blocked and exit nonzero.
- Planned payload declarations are not a root census or exact effect estimate.

Prove no mutation with an argv recorder and before/after repository operation,
working-copy state bytes, metadata paths and fixture contents. Include pending
assertion and prepare preview. Avoid atime as a mutation oracle; read-only inspection
can legitimately affect filesystem access timestamps. Do not resurrect old tests'
`READY` guarantee by quietly restoring snapshots or recursive planning.

## 9. Native support and public cutover

Put profile classification in `native/jj`, not a string check inside the command.
Keep raw JJ build identification in addition to parsed semantic version. Initial
profile is Unix local working copies, full sparse, no monitor/background snapshot,
same repository, disjoint roots, supported tracked files/symlinks, JJ-only receivers,
and actual required-clone capability. Retain existing source/receiver ownership and
colocation distinctions. A valid colocated donor is not permission for a colocated
receiver.

For initial deployment, admit the actually proved build
`0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b` in an explicit native-profile table;
record its raw version and profile ID in reports. This is a tested-build selection,
not cryptographic certification from version text. Widen that table with focused
production protocol tests of both routes. The legacy lifecycle-wide JJ minimum
does not automatically certify this private-state protocol. Known unsupported
profiles refuse before destructive refill. No runtime fallback or blanket unsafe
override is needed to make the installed supported profile usable.

Proposed CLI spelling: `--renewal-route=clear-then-populate|replace-authorized`,
default clear-first. This is an execution mechanism selector; `--replace-content`
continues to mean explicit broad content deletion. Use the same enum in request,
plan and report. Add `reseed --format=json` beside the existing prepare format.

Public cutover is one coherent stage:

- `run_reseed` dispatches to the shared command; `run_prepare` to the source-only
  producer; both share preview/report projections and documented failure output.
- Remove temporary `refill_one_from` export and its one-receiver restrictions.
  Retire old `Plan`/`execute` orchestration, payload realization/audit, repeated
  donor verification and `FutureContent` prediction once unreferenced by this job.
- Keep creation-owned helpers with actual remaining callers, using ordinary
  dependencies rather than retaining a second reseed engine as an adapter.
- Change help, `docs/reseed.md`, README, JSON schema/fixtures and affected behavior
  tests together. Ordinary new interface/version changes use the project's
  coordinated pre-alpha version policy and receiving exact dependency locks.
- Describe prepared/idle inputs, actual ordinary authority, temporary-S/final-D
  storage limits, partial native/full-sparse failure and final-only preparation.
  Release/build/install are distinct; parent chooses deployment.

## 10. Cost ledger and bounded evidence

| Term | ClearThenPopulate | ReplaceAuthorized |
| --- | --- | --- |
| Shared registry/config/source indexes | Once per command; native membership once per distinct proven tree | Same shared inputs |
| Default donor snapshots | Zero; named A/producer adds its own work | Same |
| Receiver preserve | Full snapshot | Full snapshot |
| First native transition | Sparse clear: full snapshot plus removal; new D: empty-sparse snapshot | New O: full snapshot, same-tree checkout |
| Tracked transfer | Strict full-S installation | Full-S renewal plus owned obsolete removal and targeted coupled-removability work |
| Optional S→D | Restore: full S snapshot plus native materialization | Same |
| Reparent | None | Rebase only C: full snapshot; tree-equal native reparent result |
| Final native establishment | Full snapshotting diff | Full snapshotting diff |
| Full receiver snapshot boundaries | D=S: 3, D≠S: 4, plus empty-sparse new | D=S: 4, D≠S: 5 |
| Ordinary/finish | One cohort, loaded forward finish, snapshot-free native facts | Same |

Snapshot boundaries are not measured byte hashes. Native tree-state parsing,
membership extraction, rule-byte extraction, clone opens, route checks, names,
metadata and user-space policy/decision work remain separate costs.

Required acceptance evidence, using owned fixtures within 256 ordinary entries/
1 MiB and at most one donor/two receivers:

1. Production N1 D=S/D≠S for each route; clean repeat still renews all tracked
   leaves, clean ordinary data writes zero. Native truth, identity/history, clone
   privacy and S-only/D-only/type-transition restrictions hold.
2. Real N2 paired/paired and missing/paired cases. Instrument source acquisition
   to show shared paired facts; count opaque work separately. Give lanes different
   effective excludes or target shapes so accidental policy sharing fails visibly.
3. A production provider error after visible effects, native failure in the second
   receiver, and link failure after creation: structured reports retain work and
   cause, and further phases/receivers stop. No false complete rows.
4. Producer once with direct handoff; final-conformance two-step success and final
   divergence failure; truly nonmutating preview including prospective prepare.
5. Provider continuation discriminator: accepted A/unstarted B within a group and
   inherited selected-region target-only removal survive fresh restore. Changed
   policy/comparator binding refuses. This is provider evidence, not a JW resume CLI.

After those pass, compare both routes on the same reused small fixtures with
balanced run ordering, fixed source/binary identities and equal semantic outcomes.
Include native clearing/reparenting/setup and final checks in wall time. Report
setup/checking separately, and measure instrumented counts separately from native
timing. Record wall/CPU/RSS and phase durations, native argv/snapshot count,
membership/excludes extraction, per-side observations, policy admissions, recursive
effects and route work where actually instrumented. Tiny timing is a control, not
representative throughput. Select a bounded representative question only after
this evidence identifies a reason to expand it.

## 11. Sol xhigh slices and interface handoff

| Slice | Owned implementation | Concrete return |
| --- | --- | --- |
| A — shared command/source | Extract bindings/declarations, source producer input/result, memberships/indexes and native receiver function; retain clear-first behavior. | One N1 call with counted shared capture, loaded finish and structured partial report; old unrelated drafts preserved. |
| B — contextual policy/provider adoption | Adopt inherited region, explicit comparison/renewal authority and fact-lifetime APIs. Implement Search→SelectedMirror, manifest renewal adapter and residual guards. | Parent reviews provider/API agreement, absent manifest leaf handling, opaque context propagation and retained-descendant semantics. |
| C — both native routes and actual cohort | Implement R1–R7 using production effects; serial native receivers then one ordinary runtime/cursor. | Production two-route N1 and heterogeneous N2, actual source-sharing counter, native/ordinary failure with retained work. |
| D — public command | Final-only producer conformance/handoff, nonmutating preview, report/JSON/help, native profile, CLI cutover and old-driver deletion. | Real CLI tests for preparation once, both routes, PLANNED preview, failure output and exact native result. |
| E — comparison and corrections | Bounded matched route evidence, affected tests/lints, documented native profile and release candidate. | Explain cost attribution and remaining limits; retain clear-first default unless evidence plus parent selection changes it. |

These are coherent commits, not invitations to stop at a helper. Parent reviews at
the provider contract, working cohort and public-command boundaries. Preserve
predecessors before review-driven corrections; use explicit owned commit paths.

### Required provider handoff

1. **One persisted contextual contract:** typed immutable lane/region context in
   accepted children/return chains, pending decisions and opaque application;
   rebind checks interpretation. JW supplies Search/SelectedMirror values.
2. **Explicit selected work:** default stat/literal-link ordinary comparator;
   tracked renewal can be NotAssessed with installation authority for equal leaves.
   Do not fabricate a difference. Shared semantic validation must distinguish
   comparison, preservation, work permission and selected operation.
3. **Complete listed obligations:** a desired/owned renewal seam or equivalent
   contextual lowering with manifest coverage. Missing S cannot disappear from
   name-union inspection; physical unselected source names do not create desired
   content. Reuse production fresh-clone/staging/coupled-removal kernels.
4. **Residual guard facts:** supply kind/attributes/link demand and already-acquired
   source facts to effects; JW still requires device facts. Unknown acquisition
   and stronger lane demand must retain deterministic lane-local failures.
5. **Partial results:** failure returns actual recursive effects; reports distinguish
   selected obligations, coverage, observed comparison and completed operations.
   Existing control windows and fresh-process continuation remain valid.

The manifest seam and contextual native lowering are alternatives for **provider
implementation**, with the above behavior fixed. Parent must settle that Rust API
with the independent provider design before assigning B; Sol should not invent a
JW-local copier while waiting. Slices A and producer/preview extraction can advance
against the settled ownership shapes without reading another author's same-wave
draft prematurely.

### Remaining product decisions

No open renewal-versus-refresh decision remains. Both routes are commissioned;
clear-first can remain the default. The later empirical default choice, additional
JJ profiles and actual release/install are parent decisions. Broader vanished-root
deletion, retained ordinary alias repair, colocation, permanent prepared generations
and a public recovery/resume workflow require separate intent and do not enter this
implementation implicitly.

## Connections

- [Trajectory1](trajectory1.gpt6ax.md) supplies the whole-command destination. This
  design settles its renewal question and makes replacement a concrete native path.
- [Implementation0](implementation0.gpt6ax.md) remains the clear-first control and
  source-qualified policy evidence. Its prefix replay is replaced by inherited
  context; its single native route becomes two explicitly selected mechanisms.
- [Work map](work-map0.gpt6ax.md) keeps ticket ownership. Annealing pulls contextual
  policy/comparison into this build, rather than postponing them until after CLI
  hardening. Parent owns tracker/index integration.
- [Provider consumer account](file:///home/rektide/src/diff-dirs/doc/consumer/jj-waltz.md)
  should receive the adopted contract and actual cost evidence after implementation;
  desired API capabilities here are not claims that the provider has delivered them.
- [Truthful-cache counterexample](file:///home/rektide/src/diff-dirs/design/native/graph1.gpt61sxh.md)
  is the direct authority discriminator: matching local content plus a correct final
  D tree can still conceal unauthorized local deletion. This design's matching-local
  refusal tests and full installed-membership gate address that specific failure.
- [Current operator guide](/docs/reseed.md#prepare-the-source) describes per-step
  conformance, selected-payload verification and snapshotting preview. Cutover must
  replace those promises explicitly, rather than treating the changes as invisible
  performance improvements.
