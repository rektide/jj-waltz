---
type: Assessment
title: JJ-waltz boundaries for coordinated seed and reseed
description: Current creation and reseed protocols, semantic and performance constraints, and a staged plan for shared-source multi-destination iteration without conflating transfer policy or JJ state adoption.
resource: /design/coordinated-refill/current0.gpt6a.md
status: independent-assessment-not-implementation
generated: { by: "model:openai/gpt-6-astra#high", at: "2026-10-28" }
sources:
  - resource: "opencode://ses_e7c8e7875ffehVlS4O73b8Ddto"
    title: Independent JJ-waltz-side source assessment, historical-design reconciliation, manifest-resolution check, and coordinated-refill implementation plan
  - resource: /CONTEXT.md
    title: Domain vocabulary and invariants, including stale statements identified in this assessment
  - resource: /docs/reseed.md
    title: Current donor-sync, preparation-only lock, auxiliary selection, partial-progress, and verification contracts
  - resource: /src/cli.rs
    title: Actual add, switch, reseed and prepare command surface
  - resource: /src/lifecycle.rs
    title: Creation policy, batch orchestration, exact base resolution and rollback
  - resource: /src/workspace.rs
    title: Native workspace initialization, tracked cloning, donor tree-state adoption and fallback
  - resource: /src/jj.rs
    title: Snapshot-free tracked listing and native tree-state copy helpers
  - resource: /src/lifecycle/ignored.rs
    title: Creation payload planning, per-target validation, create-only transfer and links ordering
  - resource: /src/reseed/engine.rs
    title: Current planning, shared and target-local facts, sparse tracked refill and sequential payload execution
  - resource: /src/reseed/bookmarks.rs
    title: Work-tip coverage and explicit out-of-band recovery messages
  - resource: /src/reseed/payload.rs
    title: Fused donor root discovery, outside-region validation and streaming entry guard
  - resource: /src/cow.rs
    title: Listed-leaf installer adapter and distinct creation/cleanup probe behavior
  - resource: /src/cow/transfer.rs
    title: Current pair-bound auxiliary visitor, command-owned execution and clear visitor
  - resource: /src/cow/ignored.rs
    title: Policy precedence, adaptive selection and frozen-root validation
  - resource: /src/cow/payload.rs
    title: Shared rule cache, persistent Git oracle and native tracked inventory
  - resource: /tests/reseed.rs
    title: Current semantic coverage and non-production S-to-D cache-adoption experiment
  - resource: /tests/creation_ignored.rs
    title: Creation-specific policy, sparse/fallback boundaries and owned rollback tests
  - resource: /tests/cli.rs
    title: Exact-base creation, sparse inheritance and fallback tests
  - resource: /tests/prepare.rs
    title: Current preparation-only lock and explicit combined preparation tests
  - resource: /design/sync-scope/design0.gpt6a.md
    title: Proposed authority-based transfer, independent questions and cost model
  - resource: /design/sync-scope/implementation0.gpt6a.md
    title: Parent correction preserving unselected transfer defaults and actual compatibility checkpoint
  - resource: /design/sync-scope/visitor0.sol61xh.md
    title: Delivered visitor integration and bounded native cache-convergence evidence
  - resource: /design/sync-scope/jj-state-review0.gpt6a.md
    title: Corrected native identity/cache mechanics and sparse placeholder costs
  - resource: /design/sync-scope/cursor-actions0.gpt6a.md
    title: Later good-donor and forward-iteration direction, without production policy selection
  - resource: /design/up-performance/inline-policy0.gpt6a.md
    title: Streaming guards, root topology and repeated-work reduction rationale
  - resource: /design/up-performance/measurement1.gpt6a.md
    title: Historical synthetic timing conditions and separation from raw copying
  - resource: /design/up-performance/primary0.sol61m.md
    title: Cancelled actual-workload campaign, useful census and bounded observed results
  - resource: /design/up-performance/handoff0.glm53m.md
    title: Existing uncommitted historical incident account, read without modification
  - resource: /design/reseed-locking/lock-contention0.glm53max.md
    title: Existing uncommitted contention investigation, compared with subsequently changed code
  - resource: /tools/performance/README.md
    title: Native Rust measurement tooling and regression-control boundaries
  - resource: /Cargo.toml
    title: Current path dependencies and experimental parallel feature
  - resource: /tools/performance/Cargo.toml
    title: Additional sibling-library tooling dependency requiring migration review
---

# Coordinated refill: current JJ-waltz assessment

## Recommendation

**Make reseed's selected auxiliary transfer the first substantive multi-destination
integration. Keep JJ transitions explicit and serial; coordinate the filesystem
phase over all ready targets.** Share common native tree facts too. This reaches
the primary repeated workload without first selecting a broad-copy default or a
new existing-workspace cache-adoption protocol.

A small create-only seed adapter can precede this as an API proving slice, if the
new library makes that particularly direct. Rewriting all of `add` first is not
automatically the simpler route: creation has batch rollback, sparse inheritance,
ordinary-checkout fallback and switch-state publication that reseed does not.

Four changes must be kept explicit in the implementation plan:

1. **Iteration:** one donor observation shared across destination lanes.
2. **Policy:** which data may be copied, replaced, retained or deleted.
3. **JJ protocol:** sparse-prefill expansion versus donor-cache adoption and S→D convergence.
4. **Validation schedule:** repeated live donor proofs versus one good-donor capture.

The commission makes (1) the opportunity; existing design records do not silently
settle (2) or (3). The later good-donor direction supports simplifying (4), but its
changed failure-detection timing should be stated rather than hidden in an API port.

## Evidence and standing

Source inspected at starting parent `2866e6f7` (working-copy identifier `e817961e`
at initial inspection). The only starting uncommitted files were the two named
historical notes; neither was edited or committed here. No other new same-wave
report was read. The requested filename carried a GLM suffix, but this actual
session runs Astra; this report uses its own model suffix and provenance.

All current implementation findings below are **source-derived**, not fresh
runtime performance measurements. Test descriptions mean tests inspected, not
tests executed in this session. Historical numbers are explicitly attributed.

One fresh, non-mutating build-graph check was run:

```sh
cargo metadata --locked --offline --no-deps --format-version 1
```

It failed with exit 101 because `/home/rektide/src/diff-dirs/sync/Cargo.toml` is
absent. The application manifest still points there. This confirms a current
integration prerequisite, not that changing one path completes the API migration.
The [current guide](/docs/reseed.md#verification-and-limits) already records that
the last verified application checkpoint used the library's pre-relocation
`3db9293` layout in scratch. No native tests, live reseeds, benchmarks or fixture
imports were run for this assessment.

## What “seed” currently means

There is **no `jw seed` CLI command**. `src/cli.rs:37–82` exposes `add`,
`switch`/`s`, `reseed` and `prepare` as distinct operations.

| Usage of seed | Actual mechanism | Important distinction |
| --- | --- | --- |
| Create/seed new workspaces | `add` or creation branches of `switch` | A new registered JJ workspace, lifecycle metadata and optional bookmark/links are established. |
| Seed tracked leaves | `cow::clone_files` → `diff_dirs_sync::seed_leaves` | Create-only filesystem installation of an explicit list; it does not initialize a workspace or choose a revision. |
| Update the seed/donor | `prepare`, or explicit `reseed --prepare` | Fetch/build in the source; not destination creation. |
| Seed ignored data | `IgnoredPlan::copy_to` using `Auxiliary::new_targets_only` | Selected private payload only; independent of tracked checkout mode. |

Creation donor selection is current checkout unless CLI `--from` chooses another;
reseed defaults to configured source, then verified primary. Creation borrows the
primary-local ignored-copy list, not reseed's donor/assertion/preparation defaults.
An existing `s` does not refill or even compile creation-only payload policy.

### Current creation sequence

Concrete owners: `lifecycle.rs::add_workspaces` (244–308),
`switch_workspaces_with_creation` (319–484), `resolve_creation_base` (651–684),
`create_workspace` (714–774); `workspace.rs:488–660`.

1. Resolve names, placement, mode and one exact base. Implicit base snapshots the
   donor first; explicit `--at` alone resolves without that initial snapshot.
   Explicit `--source-at` snapshots and asserts donor **tree** equality without
   choosing the destination parent. Merge `@` is an allowed implicit base.
2. Preflight every creation, decide tracked CoW versus ordinary JJ/colocated
   checkout, and prepare one shared ignored-payload plan.
3. For each target, execute `jj workspace add --revision <base>`; CoW creation adds
   `--sparse-patterns empty` and JJ-only mode.
4. `materialize_clone` lists the donor's current tracked paths **per target** using
   snapshot-free `tracked_files`. `clone_files` checks listed leaves, skips absent
   or non-leaf donor entries, and calls the generic create-only installer. It does
   not enumerate/copy ordinary donor data.
5. Copy donor **`tree_state` only**, validate with native `status`, and `restore`
   the new workspace's desired tree. Target checkout/name/repository binding stays
   native. Source sparse patterns are inherited through that state.
6. Capture creation operation before bookmark creation; establish optional
   bookmark and lifecycle record. Copy private ignored payload, then apply links.
7. Finish the batch; switch additionally records navigation state. On a creation
   error, unwind owned earlier creations, including their links/payload/metadata.

Tracked CoW creation visibly falls back to ordinary JJ creation when the early
capability decision or supported materialization fallback requires it. Nested
source/destination paths, Watchman and rejected tree-state adoption have explicit
fallback paths. A real cloning error is not universally swallowed into fallback.
Ignored-payload copying independently requires CoW even with `--no-cow` or
`--colocate`; it refuses occupied selected roots and link overlaps.

Creation is therefore **already a donor-state-adoption consumer**. Its source
comments' “only stats” explanation is motivation, not measured proof that every
JJ version avoids every content read. `jj.rs:651–668` reads and writes JJ's private
`working_copy/tree_state` file directly, then the caller validates through JJ.

## Current reseed sequence and actual semantics

Let **S** be the asserted donor tracked tree, **D** the desired tracked tree,
**Oᵢ** each target's snapshotted old tree, and **P** the selected auxiliary roots.
The donor WC commit may have a different ID from S while its tree is equal.

**D is common to this whole invocation:** explicit `--at`, otherwise the donor's
current WC commit. It is not Oᵢ. `run_inner:434–439` and
`tests/reseed.rs::plain_sync_replaces_feature_tree_types_modes_and_literal_paths`
establish this. Feature layering is a subsequent JJ operation.

### Planning (`src/reseed/engine.rs::run_inner`, 180–662)

- Discover inventory without snapshotting partial targets; resolve literal/group
  selection and donor. Empty selection never implies all. Built-in `all` excludes
  the actual primary and donor, not a hard-coded workspace name.
- Check same repository, target identity/ownership, non-overlap, full sparse state,
  monitoring restrictions and command-local mount topology. Optional preparation
  runs explicitly, under a preparation-only lock; it finishes before refill.
- Snapshot/assert the donor before target snapshots. Snapshot real target work,
  freeze one final native operation for tree facts, and establish bookmark coverage.
- Extract source entries/rules once. `payload::select_outside` fuses source-root
  discovery with validation outside selected interiors. Selection is source-driven:
  an absent source root is not selected and does not authorize deleting a target root.
- Per target, read desired entries/diff, get target effective ignore rules, build
  equal S/D clone paths, read Oᵢ, verify the intact target, and check tracked
  obstructions, symlink extraction and future links. Dry-run additionally invokes
  joint auxiliary preview using the execution visitor.
- Any structural preflight blocker prevents **all** refills. Preview aggregates
  target-local failures; a failed shared proof marks unchecked targets not assessed.

### Execution (`execute`, 1463–1599), currently target by target

```text
identity/source/target/bookmark rechecks
  → sparse set --clear
  → remove residual recorded leaves only
  → jj new D, confirm exact empty child
  → optional explicit clear except target .jj
  → create-only reflink prefills equal in S and D
  → sparse reset; native tracked-tree verification
  → incremental mirror of P using Auxiliary visitor
  → links; final native tree/full-sparse/parent/identity checks
```

The root, target `.jj`, registration and lifecycle identity stay in place. No
reseed lifecycle-record rewrite appears in `execute`. Old work is saved in JJ
history and default-on bookmark coverage is checked before clearing. Reseed
neither moves those bookmarks nor promises rollback. Runtime failure is reported
for that target and the loop proceeds to peers. No progress journal, receipt or
resume mechanism exists; old storage files are ignored without cleanup.

`remove_recorded_file` (1601–1623) is intentionally **leaf-only**. It does not
recursively erase a directory when a former tracked filename is now a container
holding unrelated target data. A generic recursive clear is not a replacement.

### Auxiliary semantics that must survive an iteration rewrite

Current `Auxiliary` in `src/cow/transfer.rs:69–163` is a bounded compatibility
visitor. It admits selected roots and routing ancestors, preserves other names
on both sides before child observation, and rejects routing file/link collisions.
Selected roots may replace target files/symlinks without following them. Inside
selected roots, target-only descendants are removed; unrelated roots remain.

- CLI list → local list → global list → none; explicit `[]` and disable mean no
  auxiliary copying. Missing/nonignored source matches do not broaden authority.
- P must be ignored in frozen source and desired contexts and disjoint from both
  tracked leaf sets in both prefix directions. Effective excludes can differ by
  target even when D is common. Old tracked content becoming payload **is supported**;
  `tests/reseed.rs:1451–1497` covers it. Do not promote an earlier proposal to reject
  all Oᵢ/P overlap into a new compatibility restriction.
- Tracked prefills renew CoW sharing even when bytes are equal. Auxiliary comparison
  uses type/size/exact mtime or literal link text; unchanged auxiliary files are
  deliberately left alone. Byte edits with preserved keys and mode-only auxiliary
  changes are not repaired. Copying every equal auxiliary file would regress that.
- `--replace-content` authorizes a separate broad target clear, not copying all
  donor dotfiles. It is neither a synonym for whole-workspace mirror nor permission
  to transplant `.jj`.
- Normal execution validates payload interiors inline; deep failure can follow
  completed writes or peers. Preview inspects one-sided interiors too but promises
  neither reflink capability nor future write success.
- Initial native snapshots may auto-track nonignored local files. Later transfer
  exclusions do not retroactively preserve them as untracked data.

## Resolve stale documents before using them as requirements

| Earlier statement | Current evidence and implication |
| --- | --- |
| Omitting `--at` preserves the feature revision | `engine.rs:434–439`, CLI help and maintained guide select donor-current sync. Several old test names still say “reset”; inspect assertions rather than names. |
| A global lock covers reseed and preparation | `PreparationLock` is acquired only at 325 inside the explicit preparation branch and dropped before worker planning. `tests/prepare.rs:478–577` exercises held/free OS-lock boundaries. The existing contention note describes a predecessor. |
| Real CoW must be probed before clearing | Current reseed has no mandatory synthetic clone probe. Actual required clones can fail after tracked changes. `cow.rs` still has a broadly worded probe helper comment; creation/cleanup usage does not establish reseed behavior. |
| Journals, receipts, resume and frozen recovery policy | `CONTEXT.md` terms 140–160 retain historical descriptions, contradicted by its later invariant and current code/guide. Neither migration nor cleanup belongs in the coordinated rewrite. |
| Preparation only via `prepare` | Current `reseed --prepare` explicitly composes source preparation; plain reseed stays offline. Its dry-run marks future readiness not assessed. |
| Per-file tracked verification and payload hashing dominate today | The handoff describes the old incident. Current `verify_disk` is a bounded native diff; persistent ignore oracle and removed payload hashing already address major historical causes. |
| Broad update/mirror or `copy_ignored` retirement is already selected | `sync-scope/implementation0` explicitly retracts that interpretation. Current visitor integration preserves the old bounded scope. |

The [later cursor/action assessment](/design/sync-scope/cursor-actions0.gpt6a.md#direction-and-standing)
records a good-donor, forward-iteration direction with concurrency/locking
out-of-band. It supports deleting repetitive proof work rather than designing a
new shared-donor lock or recovery framework. It does not choose ordinary-content
transfer breadth. The existing lock note's receipt-retention suggestion is also
not a reason to introduce automatic pruning.

## Costs and simplification opportunities

### Source-derived accounting, not elapsed-time claims

For N targets, T desired tracked leaves and P admitted payload entries:

| Work now | Why coordination helps, or does not |
| --- | --- |
| One root-bound pair `Applier` per target | Donor directory listing/stat/readlink work repeats N times. Shared-source frames can amortize those observations; target observations remain per destination. |
| One command-owned optional pool, borrowed by all transfers | Pool reuse already exists. Replacing it with N concurrent pair runs is not coordinated iteration and still repeats source inspection. |
| Desired `entries`, `changed_paths(S,D)`, target-path set and clone vector rebuilt in every planning loop | D is common. Compute these once, share them with target plans; no new library facility is needed. Effective target excludes and Oᵢ remain target-specific. |
| Four `validate_source` calls per successful target | Each includes a full-sparse native tracked-tree check, frozen-root revalidation and native commit lookups. Shared auxiliary iteration alone does not remove these costs. |
| Target intact verification in planning plus three checks during execution | Keep-before-clear and final correctness have distinct purposes; whether the post-materialization check is redundant depends on the exact auxiliary exclusion proof. |
| Source outside-selection walk; target outside-selection walks in planning and execution | These exclude selected payload interiors but still inspect retained/unselected data. Coordination does not magically remove them. Broader `--replace-content` adds more inspection/removal work. |
| Forced tracked leaf cloning and sparse expansion | N×T target installs remain necessary to renew sharing. Expansion may leave placeholder stats requiring subsequent JJ content reads; multi-destination inspection does not solve that native cache cost. |
| Frozen rule extraction and native ignore oracle | Already shared/memoized by tree plus effective excludes. Do not reintroduce per-entry Git/JJ queries in library callbacks. |

For a successful **plain** non-preview reseed, static call-site counting gives
`1 + 8N` calls to `verify_disk`: one initial donor, one planned old target per
target, four execution donor checks per target, and three execution target checks
per target. This excludes additional `snapshot`, `require_full`, revision and
bookmark subprocesses, preparation and failure paths. It is a control-flow count,
not a measurement of hashes or runtime. `verify_disk:1061–1084` already uses one
snapshot-capable native diff rather than `file show` per tracked leaf.

Coordinated payload traversal can reduce roughly N source-observation passes to
one over the common admitted source space. It cannot eliminate N destination
observations, N independent destination inodes/reflinks, or target-local deletes.
Absent-destination/source-only mode can avoid useless destination comparisons;
it still enumerates/copies source entries. Memory should be active directory
frames plus destination lane state and native tracked indexes, not a retained
payload-wide operation catalog. A destination failure must not turn valid source
presence into apparent absence for its peers.

### Historical measurements worth carrying, with limits

- [Synthetic measurement1](/design/up-performance/measurement1.gpt6a.md#measurements):
  8,555 regular data files, tracked ignore file, 43 links, two workers and only 100
  small payload files; native reseed 6.64 s. This predates subsequent cuts.
- [Maintained checkpoint](/docs/reseed.md#verification-and-limits) reports later
  synthetic reseed 5.08 s and a 30,000-file/123 MB creation payload dropping from
  3.56 s to 1.53 s after removing hashes/stamps. Not a new result of this session.
- [Actual-workload census](/design/up-performance/primary0.sol61m.md#imported-census-and-provenance):
  8,557 tracked paths versus 171,567 regular payload files, 9,561 payload links,
  24,252 payload directories and 11.36 GB logical payload bytes across 68 roots.
  This strongly motivates the auxiliary source-to-many seam.
- The [cancelled campaign](/design/up-performance/primary0.sol61m.md#cancelled-campaign-retained-observations-not-completed-matrix)
  completed only five samples: two serial N2 clean observations at 17.64/21.15 s
  versus paired predecessors at 99.11/108.31 s; one unpaired P4 sample at 8.81 s.
  It does not establish a coordinated-path speedup or choose a parallel default.
  Setup/verification overwhelmed useful measurement. Do not revive the import
  campaign to prove this adapter's basic semantics.

## Proposed ownership and execution boundaries

### Shared command facts and target lanes

Refactor the current `Plan` into a command-owned source/desired context and small
target-local plans. Names here describe proposed responsibilities, not mandatory
new public APIs:

- **Shared:** source root/identity, S and donor WC identity, D/base, common desired
  entries, equal tracked clone list, selected roots and routes, source rules,
  frozen link configuration, topology and execution resource.
- **Per target:** root/`.jj` identity, registration/ownership, Oᵢ/old paths,
  bookmark proof, effective desired rules, desired child ID, diagnostics and stage.
- **Filesystem adapter:** source and destination bindings, selected-region visitor,
  target-specific device facts, required-reflink policy and per-lane outcomes.
- **JJ adapter:** all native commands, tracked path authority, exact parent/tree
  assertions and outward recovery coordinates. Keep `Rc<IgnoreRules>`/`RefCell`
  oracle state on this serial side; freeze callback decisions before parallel work.

Do not pass the whole `Plan` into library worker closures or demand that the
generic library understand bookmarks, JJ file modes, source assertions or links.

### Restructure the target-at-a-time executor at a concrete seam

1. Complete existing batch selection and structural preflight, including coverage.
2. Run each target's tracked prefix through sparse expansion and intact desired
   tree verification. Record ready targets and local failures; leave failed targets
   out of the filesystem cohort. Do not pre-clear all N targets and leave them
   sparse-empty merely to begin the first payload copy sooner.
3. Run **one coordinated auxiliary transfer** over the ready cohort, using the
   same selected roots and target-local outcomes. Keep JJ subprocesses outside it.
4. Apply links and final native verification to successful lanes; report any
   failures with their target coordinates. Tracked success alone is not success.

This changes scheduling, not the desired final state. The current loop completes
all phases of target A before starting B; the new loop can leave several targets
at a full tracked D while auxiliary work is incomplete. The forward-only contract
permits partial progress, but progress messages should name this phase honestly.

**Required library fit question for the companion assessment:** can a destination
failure detach only that lane while shared source iteration continues? Current
JW continues peers after a runtime target failure. If the new driver only has a
global terminal error, preserving this behavior needs a native per-lane facility
or an explicitly reviewed behavior change; repeatedly restarting the whole source
walk for remaining targets is a fallback with reduced sharing, not the promised
single coordinated traversal. Shared source failure is different: stop affected
lanes and report incomplete/unverified work, rather than claiming their success.

Preview should use the same coordinated policy path and preserve target-local
READY/BLOCKED aggregation. It must not introduce an execution-time full preview
pass. Keep command wall time separate from per-target stage times: today's
`execute` elapsed measurement cannot simply be duplicated onto every lane and
summed once a stage is shared.

### Donor-check simplification

Under the recorded good-donor/quiescence direction, capture the donor once for the
batch and use cheap identity/commit/config checks at native phase barriers; avoid
four full donor proofs per destination. A final shared donor check can establish
the chosen end condition. If it fails, retain completed work and report affected
results unverified; it cannot establish that all live payload reads saw an atomic
donor generation. No stronger concurrency promise or new locking scheme follows.

First extract the phases with checks unchanged, then make this reduction an
explicit coherent correction with failure-timing tests. That isolates useful
amortization from accidental weakening of target pre-clear or bookmark checks.

## Seed first versus reseed first

### Recommended first value: reseed auxiliary cohort

It has existing targets and all plans already collected before execution; the
only scheduler is a `for plan in plans` loop. The repeat workload's large payload
dominates the historical census, and incremental clean behavior matters most here.
Keep tracked prefix serial initially. Shared desired facts and donor-check reduction
then cut native overhead independent of the library's copying speed.

### Valid smaller seed-first slice

An explicit-list, multi-destination tracked installer is a useful first adapter
test if offered by the revamped library. It must preserve `clone_files`' creation
skip behavior, absent-leaf collision refusal, literal links and real-reflink
failure. For reseed's common D, all destinations use the same equal-leaf list.
This proves source-to-many installation; it does not by itself batch `jw add`.

To coordinate full creation, split `create_workspace`/`add_workspace` into native
initialization, file installation/adoption, and publication/setup boundaries.
Initialize compatible new targets, seed a cohort, then validate/adopt/restore
each, create bookmarks/metadata, copy auxiliary payload and apply links. Preserve
ordinary-JJ/colocated fallback separately. Every successfully initialized target
must enter owned rollback tracking **before** any later batch step can fail;
today `PendingWorkspace` is only returned after complete setup. Final switch-state
failure must still unwind the newly created batch and reversible links.

This is more orchestration work than merely replacing `seed_leaves` with a many-
destination call. The next phase should use the same filesystem adapter for
creation payloads, including `new_targets_only`, instead of inventing a second
coordinated copier. Existing switches remain outside creation cohorts.

## Separate optional JJ protocol: clone S, adopt cache, converge to D

The [native test](/tests/reseed.rs),
`existing_workspace_can_validate_donor_tree_state_then_restore_a_different_desired_tree`
(1188–1259), actually clones **all S-tracked leaves** into an existing sparse-empty
target, copies only `tree_state`, checks unchanged target `checkout` bytes, runs
native status and restores D. It checks parent D, types/deletions/mode, retained
locals, original root/metadata identities, saved work and an unchanged clone inode.
It is substantially stronger evidence than assuming adoption only works when S=D.

However, adopting this for reseed is not needed to use coordinated iteration:

- S-only paths are temporarily materialized even when absent in D. The obstruction
  gate must cover retained-local collisions with **S as well as D**, in both prefix
  directions. Current desired-only preflight is insufficient for that protocol.
- The fixture disabled auto-tracking before adding local data and constructed safe
  overlaps; it is not a production admission implementation.
- Copying full donor tree-state changes sparse/cache interpretation before final
  convergence. A failure after that copy can no longer be recognized simply by
  today's “non-full sparse means partial” gate. Specify what the next invocation
  may inspect/snapshot before choosing this protocol; do not silently add a journal.
- Creation fallback destroys only newly owned state. Existing-target failure cannot
  reuse that rollback strategy. The target's checkout binding must never be copied.
- Neither the test nor source comments measure avoided file reads/hashes. Native
  cache rules and race-mtime behavior remain relevant.

This is a high-value later optimization for tracked read/hash cost, especially
when N=1 and shared observation offers little. It deserves a separate complete
transition design, rather than being smuggled into the multi-destination adapter.

## Coherent migration plan and acceptance boundaries

| Stage | Exact scope | Completion evidence |
| --- | --- | --- |
| 0. Re-establish the dependency boundary | Adopt the reviewed new library coordinate/layout in `Cargo.toml`, `Cargo.lock`, `tools/performance/Cargo.toml` and affected feature/caller wiring. Assess existing pair wrappers before deleting them. | Locked metadata/build resolution; focused existing semantics compile and pass. The companion agent owns exact new API fit. |
| 1. Expose shared facts and execution seams | Refactor `engine.rs::Plan/run_inner/execute`; share D inventory/diff/clones; split tracked prefix, auxiliary stage and finalization, initially with current checks and one-target behavior. | Existing donor-sync, `--at`, bookmarks, obstruction, source-change and interruption tests unchanged in meaning. |
| 2. Deliver coordinated reseed payload | Replace repeated `Applier` runs at the auxiliary seam with the new coordinated driver; adapt `Auxiliary`, guards, preview and target-local outcome collection. | Genuine shared source observation at N>1; heterogeneous destination actions; unchanged clean payloads; local failure does not suppress healthy peer completion. |
| 3. Remove repeated donor proof work | Batch donor capture/verification schedule under good-donor assumptions; retain target mutation barriers and final correctness. | Bounded native call-count checks distinguish shared from per-target work; source-change errors remain explicit with honest affected-lane status. |
| 4. Reuse in creation and tracked seeds | Multi-target listed-leaf installation and/or creation payload cohort, then batch creation orchestration if worthwhile. Preserve sparse/state adoption, fallback, metadata/bookmark ordering and owned rollback. | Existing creation tests plus a later-target failure after earlier initialization/payload, with no unrelated cleanup. |
| 5. Select independent policy/protocol changes | Broad transfer/`copy_ignored` retirement or production reseed cache adoption only after concrete selection and protocol review. | New semantic tables, affected native documentation/config consumers and targeted tests; no claim that stages 0–4 selected these. |

Stages 1 and 2 are the main implementation path. A bounded seed API proving slice
can move before stage 2; full creation need not. Serial execution remains a valid
first coordinated implementation: source-sharing is not synonymous with parallelism.
Retain one resource owner if experimental parallelism is used, with no per-target
pool multiplication and no native JJ parallelization requirement.

## Targeted verification plan

After implementation, use small native fixtures on an actual reflink filesystem,
`JW_TEST_REQUIRE_COW=1`, and a real JJ executable before mise shims in PATH. The
existing suite documents why blindly relocating discovery fixtures inside an
enclosing JJ checkout or capturing the mise shim as real JJ gives false failures.

Existing behavior-bearing tests to preserve:

- `tests/reseed.rs`: donor-current/default and `--source-at` cases; `plain_sync_*`;
  `reset_batch_across_two_sources_keeps_roots_bookmarks_and_lifecycle`; readiness
  aggregation; target-tracked-to-payload transition; retained secret/directory
  obstructions; required future links; incremental clean inodes; selected leaf-root
  replacement; streaming equal/one-sided guards; out-of-band interruption refusal.
- `tests/prepare.rs:478–645`: preparation-only lock, release before refill,
  single preparation, failure isolation and prospective dry-run.
- `tests/cli.rs:1038–1181`: dirty implicit source, S≠D exact-base creation, sparse
  inheritance and Watchman/rejected-cache fallback.
- `tests/creation_ignored.rs`: local policy/source separation, existing-switch
  exemption, base ignore/overlap, link boundary, required CoW and readonly rollback.
- `src/cow/transfer.rs` tests: selected routing, clean zero writes, explicit empty
  selection, both prefix overlap directions, create-only collisions and failure.

Add meaningful multi-destination cases rather than duplicating every unit test:

1. Same donor, two/three targets: clean payload, absent payload, changed file,
   target-only subtree and file↔directory/link transition at the same logical path.
2. Per-target effective ignore mismatch blocks the whole structural preflight;
   deep runtime target-only refusal fails its lane while healthy peers finish.
3. Source-side failure is shared; no lane with incomplete transfer is printed OK.
4. Count source directory observations at the adapter/library boundary to prove
   reuse, separately from per-target action counts. Constant JJ process count with
   increasing tracked-file count is an existing regression, not proof of reuse.
5. For creation, failure after cohort initialization or after one payload succeeds
   rolls back all owned new state without touching the donor or pre-existing peers.

Use at most a few hundred payload entries and about 1 MiB for initial integration
fixtures. A successful tiny suite establishes semantics and observation sharing,
not production throughput. Later measurement should hold scope and result equal,
compare N=1 and N>1 with init/clean/sparse-change cases, and separate native JJ,
filesystem transfer, setup and verification costs. Reuse bounded fixtures and
avoid repeated full-copy/hash campaigns. A library-only benchmark cannot establish
whole-command improvement while the `1 + 8N` native-verification structure remains.

## Connections and return

- [Current operator contract](/docs/reseed.md) is the maintained behavior home;
  [CONTEXT](/CONTEXT.md) needs a focused later reconciliation of the stale terms
  enumerated above, not a reinterpretation of its older glossary as new authority.
- [Scope proposal](/design/sync-scope/design0.gpt6a.md#cost-model-and-concrete-work-removed)
  anticipated source-to-many and shared donor proofs. This assessment makes the
  actual current JW phase boundaries concrete while retaining the
  [policy correction](/design/sync-scope/implementation0.gpt6a.md#correction-the-jw-policy-and-state-transition-are-not-settled).
- [Visitor checkpoint](/design/sync-scope/visitor0.sol61xh.md) establishes the
  existing compatibility integration; [JJ review](/design/sync-scope/jj-state-review0.gpt6a.md)
  explains why cache reuse is promising but independent of filesystem coordination.
- [Forward cursor direction](/design/sync-scope/cursor-actions0.gpt6a.md) supports
  a small forward driver; a persisted cursor is optional work, not a requirement
  for source-sharing or a reason to restore removed recovery machinery.
- [Native tools](/tools/performance/README.md) are reusable evidence infrastructure;
  the [cancelled actual-workload account](/design/up-performance/primary0.sol61m.md)
  sets the proportionality lesson for later validation.

This report is the sole owned tracked output. Its links and OKF/session provenance
are checked with Rekon's explicit-file document checker before committing. Source
and test inspection plus the failed locked metadata check are the verification
performed here; no production implementation or runtime speedup is claimed.
