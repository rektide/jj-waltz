---
type: ImplementationAccount
title: Strict native N1 refill and source-qualified transfer checkpoint
description: Implemented callable replacement flow, native command evidence, immutable ignore matching, streaming authority, forward link finish and the remaining conditional-routing limitation.
resource: /design/coordinated-refill/implementation1.sol61x.md
status: implemented-slice1-awaiting-parent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: "2026-10-28" }
sources:
  - resource: "opencode://ses_e7be6b24fffeDISpWdV1xNbqhD"
    title: Implements and verifies the commissioned N1 native and authority slice, preserves the existing CLI pending conditional routing, and returns at the parent checkpoint
  - resource: /design/coordinated-refill/implementation0.gpt6ax.md
    revision: 82b21766
    title: Reviewed executable commission, exact native sequence and slice boundaries
  - resource: /design/coordinated-refill/architecture0.gpt6a.md
    title: Phase ownership and honest native/pass accounting
  - resource: /design/coordinated-refill/assessment1.gpt6a.md
    title: Reseed-first, upfront choices and forward failure
  - resource: /design/coordinated-refill/foundation0.gpt6a.md
    title: Accepted six-crate foundation and dependency direction
  - resource: /design/coordinated-refill/library0.gpt6a.md
    title: Actual production cursor, opaque recursion and reporting contracts
  - resource: /native/jj/src/jj.rs
    title: Lean bindings, exact mutation configuration and forward tree-state adoption
  - resource: /refill/engine/src/engine/replacement.rs
    title: Callable native-to-transfer-to-finish implementation
  - resource: /transfer/workspace/src/frozen.rs
    title: Immutable gix-ignore matcher and retained legacy Git oracle
  - resource: /transfer/workspace/src/streaming.rs
    title: Source-qualified visitor, real one-lane cursor and authority tests
  - resource: /transfer/workspace/src/tracked.rs
    title: Strict native-listed production installer
  - resource: /workspace/state/src/links.rs
    title: Loaded declarations and forward application beside lifecycle's reversible path
  - resource: /app/cli/tests/refill_one.rs
    title: Owned native integration, actual argv chronology, FIEMAP and obstruction tests
  - resource: "file:///home/rektide/archive/jj-vcs/jj/lib/src/gitignore.rs"
    title: Native matcher chaining and ancestor-exclusion requirement
  - resource: "file:///home/rektide/src/diff-dirs/tree/inspect/src/visitor.rs"
    title: Actual presence, Known/Unknown kinds and pure early/late action contract
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/apply.rs"
    title: Strict seed_leaves and eager source-only container behavior
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/coordinated/mod.rs"
    title: Production explicit group/application loop and partial reports
verification:
  method: Required-CoW native fixtures with actual argv tracing and FIEMAP, Git differential corpus, focused nextest and strict affected-crate clippy
  result: Nine native semantic tests passed; refined S=D and S-to-D cases passed with payload-parallel enabled; all 34 transfer-library cases passed across scoped runs; 13 selected native/state/bookmark tests passed
  scope: Linux, installed JJ 0.45.1-33d3a595; callable N1 only, not CLI replacement, cohort sharing, older native versions or performance certification
---

# Native and authority checkpoint

## Delivered boundary

**The first slice is executable, committed and ready for parent review.** It is
not yet the public CLI route. No upstream production files, operator workspaces,
installed binaries, topic README/index or the two starting user drafts were changed.

| Commit | Implemented stage |
| --- | --- |
| `971d0f4c` | Immutable ignore compilation, selector path queries, source-qualified visitor, production one-lane cursor and strict listed-leaf adapter. |
| `7260ce4b` | Real N1 native transition, source/default/assertion handling, shared membership, loaded forward links, reports and native integration tests. |
| `2d5ebda9` | Literal native transitions, colocated-donor fixture, lean-binding evidence and preserved inspection error causes. |

### Actual APIs

- `jj_waltz_refill_engine::refill_one_from(cwd, ReseedRequest, diagnostic)` is a
  doc-hidden temporary callable seam. It resolves existing selection/config/native
  ownership and requires exactly one receiver. Prepare/dry-run are explicitly
  unavailable here, rather than using the old expensive planner.
- `RefillOneReport` retains actual tracked `Progress`, optional ordinary
  `LaneReport`, explicit-clear progress, links, exact coordinates and phase/wall
  durations. Ordinary observed comparison and actual recursive effects stay separate.
- Native `workspace_bindings_at` returns only names/change IDs/commit IDs.
  `Inventory::load` now uses it instead of the diff/stat-rich facts template.
  `run_refill`, `read_tree_state` and `adopt_tree_state` own native mechanics.
- Transfer `tracked::seed` calls production `seed_leaves` directly, with required
  reflinks and recorded kind/executable/device checks. It has no creation-style
  pre-stat skip filter. `streaming::sync_one` drives the actual production cursor
  at workspace roots and stops at the first surfaced error, retaining partial reports.
- `LoadedWorkspaceLinks` normalizes once, exposes active relative names and applies
  through `apply_loaded_workspace_links`. Source-equals-target rules remain private
  directories. The old reversible lifecycle interface remains separate.

The new runtime is deliberately sequential, including feature-enabled builds.
There is no persistent cursor, fallback ladder, rollback or peer-completion promise.

## Native sequence and evidence

Admission uses root/native/config/topology facts and one small real CoW probe.
Initial profile: Unix local working copies, full sparse patterns, no monitor,
JJ-only receiver, same repository, disjoint roots, supported file/symlink trees.
Colocated donors are supported and exercised; their `.git` is never transferred.

The new seam currently admits **semantic version 0.45.1 only**. Evidence is for
`0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`, not a release/build matrix.
Other versions fail before preservation/clearing; unrelated lifecycle commands
retain their existing compatibility policy.

All commands below after preservation use:

```text
--config=fsmonitor.backend=none
--config=snapshot.auto-track=none()
--config=snapshot.auto-update-stale=false
```

| Order | Actual work |
| --- | --- |
| Source | Recorded S, no ordinary donor snapshot. Optional A resolves once and runs one live `diff --from @ --to commit_id(A) --summary`; successful resulting WC is S. Explicit D is pinned before receiver preservation; otherwise D=S. |
| Preserve | `status` with monitor/stale overrides, retaining initial auto-tracking semantics. Save/emit O; check current bookmark coverage once, or explicit bypass. One snapshot-free donor-coordinate check follows preservation. |
| Native inputs | Capture S/D/O membership and source `tree_state` bytes. Native empty-single-parent equality aliases membership and ignore extraction; no changed-path/equal-leaf selection. Compile needed ignore inputs and validate active-link/native conflicts. |
| Clear | `sparse set --clear`; require recorded WC still O. Refuse residual listed O paths through no-follow parents, never recursively repair them. |
| Child | `new commit_id(D)` while sparse-empty; save the receiving change ID and verify its D parent. |
| Explicit clear | Production `clear_with_visitor` only with `replace_content`, retaining receiving `.jj`. |
| Install | Strict production installation of every S leaf, returning actual counts. Existing names are obstructions, even byte/stat-equal S-only locals. |
| Admission | Inspect only D-not-S coordinates/ancestors. Refuse unrelated D-only names; permit S-owned shape changes. No directory-content census. |
| Adoption | Write captured source `tree_state` only. Receiving checkout bytes are compared immediately across this write. |
| Convergence | D=S: directly final live diff. D≠S: directly `restore --from commit_id(D)`, then final live diff. No adoption status or sparse reset. |
| Establish | Empty live `diff --from @ --to commit_id(D) --summary`; snapshot-free exact parent/empty/change/name/registration/repository/root/`.jj`/full-sparse checks. Commit ID may legitimately change through adoption. |
| Finish | Real selected transfer, loaded links against actual resulting paths, then snapshot-free native identity/result checks. No final donor proof or tracked snapshot. |

The native fixture's instrumented command chronology asserts:

- Default donor: **zero snapshotting calls**. Explicit A: exactly one donor diff.
- Receiver D=S: `status → sparse set --clear → new D → diff`.
- Receiver D≠S: `status → sparse set --clear → new D → restore D → diff`.
- Dirty-O default fixture: two file lists, S=D and O. Explicit-D fixture: two
  file lists, S and D; native-empty O reuses S. Default selected-payload fixture:
  one pinned `.gitignore` extraction, no runtime `git check-ignore`.

### Real costs, not a one-snapshot claim

Default D=S has **three full receiver snapshots** (save, clear, final diff) plus
new's empty-sparse snapshot. D≠S adds restore's full adopted-S snapshot and native
store materialization. Native metadata queries may deserialize tracked state;
cache capture/write is proportional to that state. Membership/control-file
extraction, listed O/D route checks and required per-leaf clone/stat/open work
remain real costs. Membership sharing follows exact coordinates and native-proven
empty-parent equality, not content-comparing unrelated revisions to deduplicate them.

Ordinary work has no preceding payload census, root realization, eligibility
audit or Git-query cache-fill walk. Opaque source-only copy, target-only removal
and replacement removability remain production-kernel work, not shared traversal
magic. No timing/hashed-byte/speedup claim follows from the tests.

## Ordinary outcomes and coverage

`FrozenIgnore` owns immutable gix-ignore searches. Cargo selected `0.24.1`, whose
declared MSRV is 1.88, matching this workspace. Inputs remain pinned tracked
`.gitignore` bytes, global excludes and backend `info/exclude`. Matching evaluates
ancestor directories before descendants, with global < info < root-to-deeper
precedence, negation, last-line wins, sensitive case and precious patterns disabled.
It performs no filesystem/process calls. Legacy creation retains its old adapter.

| Path situation | Implemented outcome / evidence |
| --- | --- |
| Empty, explicit `[]`, disabled policy | No ordinary runtime/binding/walk. Unit test uses nonexistent roots to prove this; native tests retain local cache and `.env`. |
| Existing ignored selected source directory | Mirror; source-only/changed data copied, target-only descendants removed. Real cursor, required reflinks and clean-repeat zero effects tested. |
| Source lacks candidate root | Target root and descendants retained; absence inside an admitted root instead authorizes deletion. Both opaque invocation forms covered. |
| Candidate source file/directory link | Ineligible; preserve rather than follow. |
| Nonignored ancestor with viable deeper selector | Search through existing paired routing directories; nested eligible root still transfers. |
| Direct selected root collides with target leaf/link | Normal production staged replacement; refusal during opaque preparation keeps the old link. |
| Selected source boundary overlaps S/D native or active links | Refuse after actual source-directory eligibility, before silently suppressing the boundary. Known/Unknown early kinds and late facts tested. Native leaves blocking a structural route refuse. Nonselected declared link names are preserved. |
| Native leaf ancestors with ordinary siblings | Route; do not hide the whole ancestor. Private-directory link overrides do not exclude explicitly selected payload. |
| Root `.jj`/`.git` | Preserve pair-wide before child metadata; hidden socket verifies the boundary. |
| Nested administration/special entry in admitted work | Refuse, including opaque replacement preparation. Late checks also cover equal facts and device boundaries. Outside admitted branches no interior audit is performed. |
| **Missing unselected routing directory** | **Explicit refusal, zero eager skeleton creation.** Deliberately incomplete until slice 2. No pre-realized root discovery substitutes for it. |
| `replace_content` | Separate explicit full clear; can delete `.env`/local cache but does not broaden donor selection. Native test distinguishes it from empty ordinary policy. |

The bounded Git-oracle differential corpus covers directory-only and anchored
rules, `**`, negation, ignored parents, nested rules, escaped punctuation/space,
Unicode, and global/info precedence. Forced-tracked exclusion is separately tested
as membership authority, not inferred from ignoredness. This is bounded equivalence
evidence, not certification of every Git edge case.

## Verification and demonstrated failure

- `JW_TEST_REQUIRE_COW=1`, `/usr/local/bin` first: nine native semantic cases
  passed in `--test refill_one`. The ignored child entry is a subprocess driver,
  invoked by those tests, not a skipped semantic case. Refined S=D/colocated-donor
  and explicit-D/literal-transition cases then passed with `payload-parallel`.
- Actual tracked installation: seven regular-file reflinks, zero ordinary copies
  in each success fixture; literal symlinks, executable mode, file/directory and
  file/link transitions, S-only removal, D-only addition and saved O content.
  Linux FIEMAP proves shared physical extents; private target writes preserve donor
  bytes. Receiving identity and donor checkout/cache retention are asserted.
- Equal-byte/equal-mtime S-only obstruction fails strict installation before cache
  adoption. Equal-byte D-only local fails namespace admission. A directory collapse
  with a retained child reaches native skipped checkout, then **fails final diff**;
  the child survives. It is not reported as healthy merely because sparse is full.
- A required missing link fails after native and selected-payload effects; the
  error retains phase, O and known effects. State's late-conflict test proves the
  forward link seam keeps earlier links/directories while reversible lifecycle
  tests still pass.
- All 34 transfer-library tests passed across split runs; 13 selected native,
  state/link and bookmark tests passed. An initial broad required-CoW transfer run
  put two legacy TempDirs on non-CoW `/tmp` (errno 95). Those and its cancelled
  cases passed with **scoped transfer-only** scratch TMPDIR. Discovery tests never
  received a blanket project TMPDIR override.
- One existing real CLI reseed regression passed after changing inventory to lean
  bindings. Strict affected-library all-target clippy, new app-test clippy and
  `cargo fmt` passed. Five focused feature-enabled matcher/streaming tests passed.
  No broad benchmark campaign or inherited colocation-cleanup repair was attempted.

## Next coherent slice, after review

The demonstrated unresolved issue is the missing unselected route: `**/node_modules`
cannot yet discover through an absent `packages/` without refusal. Existing diff-dirs
`Container::Create` creates that route eagerly, including when no match exists.

Slice 2 should add the planned narrow `CreateIfNeeded` production effect in
diff-dirs inspection validation and source-only application/preview. Pending
ancestors stay depth-sized; an admitted descendant or explicitly selected empty
directory materializes them. No match creates nothing. Preserve production staging,
permission restoration, logical paths and actual-progress accounting; no prewalk,
create-then-prune or new scheduler. Test it there, adopt the receiving lock, then
replace this visitor's explicit missing-route refusal and exercise wildcard routes.

Do not extend N2 by calling `refill_one_from` repeatedly: the subsequent CLI slice
must freeze shared native/config/cache inputs once, run serial native prefixes,
then bind a real cohort runtime. Preparation handoff, honest dry-run, CLI/help/docs,
legacy driver removal and bounded N2/fault/process-count evidence remain later
commissioned slices, not claimed delivered here.

## Connections

- The [executable commission](implementation0.gpt6ax.md#9-coherent-build-slices-and-parent-checkpoints)
  owns the slice boundary; this account supplies its native/authority evidence,
  not acceptance of the subsequent CLI replacement.
- The [architecture](architecture0.gpt6a.md#9-data-lifetime-and-cost-accounting)
  explains why metadata, native snapshots and ordinary iteration are distinct
  costs. The concrete command trace above realizes that distinction.
- The [library assessment](library0.gpt6a.md#what-is-actually-shared) limits sharing
  to paired immediate groups. N1 success here does not establish cohort sharing.
- The [accepted foundation](foundation0.gpt6a.md#reviewed-completion) remains the
  six-owner boundary; this work does not reopen that structural decision.
