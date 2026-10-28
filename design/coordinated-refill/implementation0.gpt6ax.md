---
type: ImplementationPlan
title: End-to-end reseed implementation commission
description: Executable native-state adoption, streaming source-qualified authority, coordinated execution, concrete library gap and build slices for the six-crate foundation.
resource: /design/coordinated-refill/implementation0.gpt6ax.md
status: proposed-executable-plan-with-bounded-native-proof
generated: { by: "model:openai/gpt-6-astra#xhigh", at: "2026-10-28" }
sources:
  - resource: "opencode://ses_e7bff960fffeUDrw4Sxqg0bvbZ"
    title: Human-commissioned implementation planning; reads current six-crate source and production APIs, proves shortened native adoption with required reflinks, identifies conditional routing gap, and writes the Sol commission
  - resource: /design/coordinated-refill/architecture0.gpt6a.md
    title: Current architecture and explicit native/pass accounting
  - resource: /design/coordinated-refill/assessment1.gpt6a.md
    title: Reseed-first, upfront route selection and forward failure direction
  - resource: /design/coordinated-refill/foundation0.gpt6a.md
    revision: a89f56e1
    title: Accepted six-crate foundation and attributed passing verification
  - resource: /design/coordinated-refill/current0.gpt6a.md
    title: Historical engine census; operational paths now relocated by the foundation
  - resource: /design/coordinated-refill/library0.gpt6a.md
    title: Delivered coordinated interfaces, opaque recursion and reporting limitations
  - resource: /refill/engine/src/engine.rs
    revision: a89f56e1
    title: Current run_inner, Plan, execute, native verification, discovery and repeated proofs to replace
  - resource: /refill/engine/src/engine/prepare.rs
    title: Current source producer, repeated verification and payload audits
  - resource: /refill/engine/src/freshen/source.rs
    title: Pinned upstream selection and ancestry rules retained for explicit preparation
  - resource: /refill/engine/src/lib.rs
    title: Actual request, result and source-only entrypoints
  - resource: /native/jj/src/jj.rs
    title: Native queries, expensive workspace facts template and private tree-state copying
  - resource: /transfer/workspace/src/transfer.rs
    title: Auxiliary::legacy, routing actions, execution pool and explicit clearing policy
  - resource: /transfer/workspace/src/ignored.rs
    title: Existing selector grammar, precedence and source-qualified root selection
  - resource: /transfer/workspace/src/frozen.rs
    title: Frozen rule extraction and impure persistent Git oracle
  - resource: /transfer/workspace/src/tracked.rs
    title: Repository entry enumeration and native file-content extraction
  - resource: /workspace/state/src/links.rs
    title: Loaded link configuration, normalization and application seam
  - resource: /app/cli/src/cli.rs
    title: Current reseed and prepare options and rendering adapter
  - resource: /app/cli/tests/reseed.rs
    title: Existing S-to-D adoption evidence, retained locals, incremental payload and required-CoW fixtures
  - resource: /tools/performance/README.md
    title: Existing native/instrumented performance controls; build inputs repaired by the foundation
  - resource: /design/sync-scope/jj-state-review0.gpt6a.md
    title: Receiving identity versus reusable membership/cache, and sparse placeholder cost
  - resource: "file:///home/rektide/src/diff-dirs/design/native/graph1.gpt61sxh.md"
    title: Truthful source state can absorb and remove a retained S-only local
  - resource: "file:///home/rektide/src/diff-dirs/tree/inspect/src/visitor.rs"
    revision: 5c4192e7dc5cb5b4ebb056e0c7277ea6e5800c99
    title: Pure early/late contract, actual presence, typed directory actions and missing conditional creation
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/apply.rs"
    revision: 5c4192e7dc5cb5b4ebb056e0c7277ea6e5800c99
    title: Strict seed_leaves, one-sided recursion and eager routing-directory creation
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/coordinated/mod.rs"
    revision: 5c4192e7dc5cb5b4ebb056e0c7277ea6e5800c99
    title: SyncRuntime, SyncCursor, group/application barrier and lane reports
  - resource: "https://github.com/jj-vcs/jj/blob/33d3a595429b7e3b54a8ea769560247913bc8a1b/lib/src/gitignore.rs"
    title: Installed-runtime source read from the local archive; pure gix-ignore buffer matching and ancestor-ignore caveat
  - resource: "file:///home/rektide/archive/jj-vcs/jj/cli/src/cli_util.rs"
    revision: 803ef00a0a6ee4ddc275d3138d5f9e7326399906
    title: Snapshotting command setup, working-copy mutation and prohibition on mutation with ignore-working-copy
  - resource: "file:///home/rektide/archive/jj-vcs/jj/lib/src/local_working_copy.rs"
    revision: 803ef00a0a6ee4ddc275d3138d5f9e7326399906
    title: Checkout skips obstructions, retains nonempty directories, and uses cached file states subject to mtime rules
  - resource: "file:///home/rektide/src/jj-waltz/.test-agent/coordinated-refill/implementation-probe/README.md"
    title: Owned tiny native probe, production required-reflink installer and reproducible source
verification:
  method: Current-source inspection and one two-receiver native protocol test using production seed_leaves
  result: D=S adoption and direct S-to-D restore both passed on native JJ 0.45.1; six required regular-file reflinks per receiver, zero ordinary copies
  scope: Planning evidence, not implemented JW reseed, ignore-matcher equivalence, lazy routing, throughput or cross-version certification
---

# End-to-end reseed implementation commission

## 1. Decision and deliverable

**Build one replacement reseed engine: strict full-S tracked installation, donor
`tree_state` adoption, native convergence to D, then one streaming ordinary-data
traversal shared across receivers.** Use this protocol for both ordinary D=S and
explicit D≠S. Keep receiving native identity. Any surfaced failure stops the run.

Recommended ordinary authority remains **source-qualified `copy_ignored`**:
empty/disabled selection copies nothing; an existing, ignored source directory
matching the selector authorizes mirroring that directory; an absent source root
does not authorize deleting its target counterpart. This avoids selecting broader
copying or deletion while fixing the implementation. Two concrete additions make
that genuinely streaming:

1. Compile frozen ignore bytes into an immutable in-process matcher.
2. Add a narrowly scoped **create-if-needed routing container** to diff-dirs, so
   discovering a match through absent ancestors does not create unselected empty
   directory trees. The current kernel eagerly creates these containers.

The milestone is the actual CLI completing N=1 and heterogeneous N=2 reseeds,
including native result, private CoW, selected payload, links, failure reporting
and operator documentation. Seed/add reuse follows later. The accepted foundation
is complete; dependency relocation, lazy global configuration and performance
build-input repairs are not remaining tasks.

This plan inspected JW at parent `a89f56e1`, diff-dirs at `5c4192e7dc5c…`, and the
native sources cited above. Only this document is a tracked planning output. The
two starting uncommitted research files remain untouched.

### Terms and fixed ownership

| Name | Exact meaning here |
| --- | --- |
| S | Donor's recorded working-copy commit at consumption; its live tracked files/cache are assumed prepared and quiescent. |
| D | Exact desired parent: explicit `--at`, otherwise S. |
| A | Optional `--source-at` assertion on plain reseed; tree equality with A does not choose D. |
| Oᵢ | Receiver's saved working-copy commit after its initial preservation snapshot. |
| K(X) | Validated native tracked-leaf membership of revision X, including file/symlink kind. |
| Local | A remaining path outside native ownership and admitted ordinary authority; not every file that happened to be untracked before a native snapshot. |

## 2. Cheap resolution and route admission

Keep literal/group selection, primary/donor exclusions, explicit policy precedence,
same-repository ownership and root separation. Load config and normalize link
declarations once. Capture root/`.jj` identity, repository pointer, workspace name,
full sparse patterns, monitor settings and mount topology. Use required CoW and a
small actual source/receiver clone probe before clearing; probe once per relevant
pair, with no runtime downgrade. Dry-run does not create probe files.

**Fix a hidden cost in the actual inventory path:** `Inventory::load` currently
calls `workspace_target_facts_at`; `WORKSPACE_FACTS_TEMPLATE` includes
`target.diff().files().len()`, diff statistics and conflicted-file counts. A
snapshot-free query is not automatically cheap. Add a lean native binding query
returning names and commit IDs only, and reuse the current root/registration
discovery. Do not call the status-rich inventory to make root decisions.

Pin S/D and the route before payload execution. Route admission uses supported
local-working-copy layout, full sparse state, monitoring configuration, native
capabilities, root ownership and required-CoW availability. It does not use a
payload census or a speculative clone attempt of real files. Reading sparse state
may deserialize the tracked-state file; charge that native metadata work.

Initial supported production profile: Unix, native local working copies, full
checkouts, no filesystem monitor, JJ-only receivers, one repository, disjoint
roots, conflict-free file/symlink trees. A colocated donor is allowed when its
native topology is valid; its `.git` is never transferred. Reject unknown native
profiles before clearing. The shortened sequence below is proven on JJ 0.45.1;
exercise older advertised versions before admitting them to this new route. A
temporary explicit reseed-version floor is preferable to claiming untested cache
compatibility or implementing a fallback. Other lifecycle commands need not lose
their existing version support.

Tree-entry validation belongs to the next, explicitly costed native phase. It can
reject conflicts, submodules or unsupported leaves without a live payload walk.
There is one selected donor-state route, not a per-error switch to sparse expansion.
Path obstructions discovered later fail that route.

## 3. Native protocol: executable sequence

### Shared inputs, saved receivers and assertions

1. **Plain default:** read recorded S without snapshotting the donor. No
   `validate_contents`, `payload::select_outside`, donor `verify_disk`, or status.
   Prepared donor correctness is an operating premise.
2. **Explicit A:** resolve A and run one native live tracked-tree diff against A,
   with auto-tracking disabled, to establish the requested assertion. That command
   snapshots the donor and may read/hash tracked files. On success pin the resulting
   donor WC commit as S; default D is that S, not A. A failed assertion may have
   recorded donor edits and fails before target clearing.
3. Snapshot each selected receiver once with current initial-snapshot semantics;
   record and print Oᵢ. Retain the existing default bookmark-coverage gate and
   `--allow-unbookmarked` bypass, checking coverage once against these saved facts.
   This avoids an independent history-policy change. Native clear will also
   snapshot implicitly; see the ledger rather than claiming a single physical scan.
4. After all preservation snapshots, make one snapshot-free donor-coordinate check.
   A target snapshot can rebase related commits in the shared repository; if it
   changed the pinned donor, fail before clearing rather than consume stale state.
   Explicit D stays pinned to the request's exact coordinate.
5. Read K(S) once, K(D) once when different, and K(Oᵢ) once per distinct old commit.
   Reuse indexes for all receivers, installation, ignore extraction and ordinary
   exclusion. Do not additionally request S→D changed paths: full-S adoption does
   not need equal-leaf selection. Read donor `tree_state` bytes once for the batch.
6. Compile frozen source/desired ignore bytes if payload or links require them.
   Validate configured link/native conflicts against K(S)∪K(D) before destructive
   work. These are declaration/tree checks, not payload discovery.

All later native mutators use:

```text
--config=fsmonitor.backend=none
--config=snapshot.auto-track=none()
--config=snapshot.auto-update-stale=false
```

Use `commit_id_revset()` for exact IDs, not bare names with ambiguous resolution.
Every repository-only read uses `--ignore-working-copy` or `run_at`. Do not put
`--ignore-working-copy` on sparse/restore mutations: native JJ forbids that route.

### Per receiver, serially

| Step | Actual operation and condition |
| --- | --- |
| 1. Clear tracked checkout | `jj sparse set --clear`. This implicitly snapshots the full checkout again, then removes native tracked leaves. Require its resulting recorded commit still be Oᵢ under quiescence. |
| 2. Refuse residual native leaves | Inspect only the recorded K(Oᵢ) coordinates through no-follow parents. If a regular file/link remains where native clearing should have vacated it, fail with that path. Do not delete a D-only local, recursively erase a former leaf now holding a directory, or restart clearing another way. This replaces today's broad O/D residual-removal loop. |
| 3. Establish receiving child | `jj new <exact D>` while sparse-empty. It creates an empty **change with D's tree**, not an empty repository tree. Record its change ID and check the exact parent using snapshot-free facts. |
| 4. Optional explicit clear | Only for `--replace-content`, call production `clear_with_visitor` at the workspace root, retaining its own `.jj`. This is a separately requested full target traversal. Keep existing administrative refusal and explain deletion of ignored/local data. |
| 5. Install full S | Call production `seed_leaves(source, receiver, K(S), ReflinkRequired, native_leaf_check)` directly. Do not use `cow::clone_files`, whose creation contract pre-stats and skips missing/non-leaf source entries. Install every S leaf or fail; collect actual `Progress`. Existing target leaves, including byte-identical S-only locals, are obstructions. |
| 6. Admit D additions | For D leaves not exactly present in K(S), check their actual no-follow path routes now. Existing S leaves/ancestors are operation-owned and may undergo native shape changes; existing unrelated leaves are refused. A directory containing S descendants can be handled by native checkout, which retains unrelated children and may subsequently fail conformance. No recursive census of that directory is required. |
| 7. Adopt state | Write the captured donor `tree_state` bytes into the receiver's local state file. Never write donor `checkout`, workspace name, repository pointer or `.git`. Assert target checkout bytes are unchanged **across this write**, not across legitimate native operations. |
| 8a. D=S | Go directly to the final native diff below. No separate adoption `status`, `restore` or `sparse reset`. |
| 8b. D≠S | `jj restore --from <exact D>`. Its ordinary command setup snapshots the borrowed S state into the receiver's own change, then restores D. There is no preceding `status`. |
| 9. Establish native result | Run the live `jj diff --from @ --to <exact D> --summary` with the three configuration overrides. Require empty output, then use snapshot-free facts to require a full-sparse empty single-parent child of exact D, receiving registration/name/repository and unchanged root/`.jj` identity. A skipped native checkout is not success just because `restore` exited zero. |

Keep the receiver's change identity through adoption/convergence. Its commit ID can
legitimately change during S→D snapshots; compare the final tree/parent and change
identity, not an intermediate child commit ID. Root and `.jj` remain in place.

**Membership authority is enforced by strict installation.** Source `tree_state`
imports K(S), not just stat hints. Every imported leaf was newly installed by this
operation after native clearing. An occupied S-only path therefore fails before
adoption, even when its bytes match perfectly. This directly excludes the
[retained-local counterexample](file:///home/rektide/src/diff-dirs/design/native/graph1.gpt61sxh.md).
It can reject D≠S cases the equal-subset route previously accepted. That is an
explicit supported-route restriction, not permission to delete the local or select
a fallback after failure. D-only obstructions get the analogous refusal.

Native checkout's no-follow/skip behavior remains useful, but does not establish
namespace authority by itself: a matching pre-existing D-only local must not be
silently accepted as a newly tracked result. Tests must exercise that case.

### Evidence already obtained; what still needs proof

The owned [probe](file:///home/rektide/src/jj-waltz/.test-agent/coordinated-refill/implementation-probe/README.md)
ran one tiny test with two receivers using `/usr/local/bin/jj` version
`0.45.1-33d3a595…` and production `seed_leaves`. Both branches above passed:
six regular-file reflinks and one literal symlink per receiver, zero ordinary
copies; exact final parent/tree, source-only removal, desired-only addition,
directory→file transition, executable-bit change, saved old contents, unchanged
common clone inode and native identity, retained `.env`, and private writes.
Donor commit and `tree_state` stayed unchanged. No timing inference follows.

This settles the direct-restore uncertainty beyond the existing adoption test.
Sol still must prove the production admission/obstruction checks, full app result,
CoW extents where the existing fixture supports them, and supported-version matrix.
The residual-refusal and D-addition rules are planned code, not probe coverage.

## 4. Streaming ordinary authority: actual pure policy

### Preserve current useful outcomes

| Example with selection `**/node_modules` or `cache` | Required outcome |
| --- | --- |
| No selection, explicit `[]`, or `--no-copy-ignored` | Skip ordinary runtime construction; no ordinary writes or discovery walk. |
| Source `cache/` is selected and ignored in S and receiving D | Mirror it: copy changed/source-only entries and remove target-only descendants. |
| Source lacks `cache/`; target has `cache/local` | Preserve the target root and its contents. |
| Source match is a file or directory symlink | Not an eligible root; preserve that name rather than follow it. |
| Source `packages/a/node_modules` exists, routing parents missing at target | Discover through source-only parents during actual iteration; create only necessary ancestors and selected contents. |
| Source `packages/a/src` exists but no matching descendant | Leave no new empty target `packages/a/src` skeleton. |
| `.env` or build output outside admitted roots | Retain target; do not copy donor counterpart. |
| Selected source directory is not ignored in S | Leave it unselected; still search viable descendant patterns. |
| Selected/ignored source root overlaps tracked S/D or active link-owned region | Refuse that root before its effects. Old tracked O becoming payload remains allowed after native clearing. |

### Frozen matcher, not an oracle disguised as a callback

`FrozenRules::get` currently extracts tracked `.gitignore` files and captures
backend/global excludes, then builds `IgnoreRules` with a subprocess and `RefCell`
answer cache. It is neither pure nor `Sync`. Do not put it behind a mutex inside
the visitor, fill its answers by prewalking payload, or invoke it between a group
and application: policies already participate in inspection and opaque recursion.

Implement `FrozenIgnore` in `transfer/workspace/frozen.rs`:

- Preserve the current rule inputs: pinned tracked `.gitignore` bytes, effective
  global excludes and backend `info/exclude`. No search for live untracked ignore
  files. Extract bytes once per distinct tree and effective-excludes context.
- Parse buffers with `gix-ignore` directly, following JJ's `GitIgnoreFile::chain`
  and matching mechanics. No dependency on the whole `jj-lib` is needed. The local
  available `gix-ignore` 0.24 API provides `Search::add_patterns_buffer` and
  `pattern_matching_relative_path`; its MSRV is 1.88. Use the actual selected
  release's fallible API, not an older copied signature.
- Store immutable per-directory matchers, rooted by repository-relative prefix,
  with global < info < root-to-deeper `.gitignore` precedence and last-line wins.
  Disable precious-pattern extensions; use the supported case-sensitive semantics.
- `ignored(path, is_directory)` evaluates every ancestor directory in order.
  Once an ancestor is ignored, descendants remain ignored: a deeper negation
  cannot reopen an excluded parent. Otherwise apply the applicable chains to the
  next component. JJ's exact-path matcher explicitly leaves this recursion to its
  caller. Matching only the leaf is wrong.
- Retain the current Git oracle as a **test reference** for a bounded differential
  corpus: directory-only rules, `**`, anchored paths, negation, ignored parents,
  nested files, escaping/whitespace, global/info precedence and forced-tracked
  overlaps. Native membership is an independent exclusion, never inferred from
  ignoredness. Old creation callers can keep their old adapter until later reuse.

Compilation reads control-file bytes and repository trees, not all ordinary
payload entries. The production visitor owns immutable values/`Arc`s only. Rule
equivalence is a meaningful first-slice verification obligation, not an assertion
that all Git edge cases are already proven. Resolve discrepancies before changing
selection outcomes; an intentional ignore-policy change is a human choice.

### Stateless selection from paths and actual directory facts

Extend `Patterns` with pure path-prefix queries: `matches(path)` and
`may_match_below(path)`, reusing its existing `*`, `?`, `**` grammar and Unicode
component behavior. Replaying the small automaton over a path is sufficient;
do not build a filesystem-wide matched-root set.

Define Q(p) = selector matches p and frozen **source** rules ignore p as a directory.
For each path find its outermost Q ancestor from the immutable path string. This
is a *candidate*, not evidence that an arbitrary directory exists on the source.
Admission works as follows:

1. Preserve root `.jj`/`.git` pair-wide before child observation. Preserve declared
   active link names and whole native leaf names; route through shared directory
   ancestors rather than hiding ordinary siblings. Check a Q boundary's actual
   source-directory eligibility and overlap **before** silently suppressing it as
   native-owned: Unknown source kind defers that decision to late. If a native
   target leaf blocks a source directory that the selector must route through,
   refuse that incompatible route rather than enter or replace the native name.
   This can conservatively refuse before learning whether a deeper match exists;
   it must not silently claim that a conflicting selection was satisfied.
2. **Outside a previously reachable Q ancestor:** if the name is target-only,
   preserve its whole tree. It cannot contain a source-qualified match. If the
   source is known non-directory, preserve it; if kind is Unknown, defer to late.
   Preserve branches the selector cannot reach.
3. At a source-present real directory p with Q(p), validate disjointness in both
   prefix directions against K(S)∪K(D) and active link names; require ignoredness
   under that receiver's frozen D rules. Now accept normal mirror actions. A
   target file/link at this selected root can be replaced by production staging.
4. Below that outermost Q ancestor, accept mirror actions, including **target-only
   descendants**. Their source absence is deletion authority inside the admitted
   root, unlike target-only candidate roots in step 2.
5. An unselected source directory with possible matches below is structural:
   paired directories use `Directory { Keep, descend: true }`; source-only ones
   use the conditional routing action below. A target file/link blocking such a
   route is refused rather than replaced with an unselected directory.
6. In admitted ordinary work, reject nested `.jj`/`.git`, special entries and
   supplied device-boundary violations. Check actual facts, including equal leaves.
   Outside admitted branches, preserve without an audit of their interiors.

Why no mutable ancestor-admission map is needed: a child below Q can only be
reached after its real source directory was admitted, or by production recursion
within that admitted root. A target-only Q root is stopped at its own boundary.
Non-directory candidates are stopped too. This invariant must hold for source-only
copy, target-only removal and replacement removability, not only paired traversal.
Test all those invocation forms. Do not bind an interior subtree as a fresh policy
root and assume a missing admission history.

Link rules with source equal to target are the existing **private-directory
override**, not an active symlink owner. They remain private but can be refreshed
when the operator explicitly selected their directory as payload. Active link
names are excluded and checked against native membership. No new local-preserve
configuration language is required.

### Concrete library gap: missing source-only routing containers

Current `Action::Directory { container: Create, descend: true }` calls
`copy_directory_selected`, which creates the directory **before** examining its
children (`transfer/sync/src/apply.rs:590–611`). `Container::Keep` is invalid with
an absent target. Thus a visitor alone cannot preserve the old final footprint
while discovering matches beneath missing routes. A full pre-discovery pass would
hide rather than solve the gap.

Add one production action variant, provisionally
`Container::CreateIfNeeded`, for **source-only directory + descend=true**:

- Inspect its source children in the existing opaque source-only kernel with a
  depth-sized chain of pending routing ancestors. No target directory is required
  for this source-only inspection; logical paths remain the final paths.
- Materialize pending ancestors only when an admitted descendant requires a real
  namespace effect. An explicitly selected empty directory counts: it uses ordinary
  `Create`, so its routing ancestors are established too.
- Reuse normal staging, leases, copy/removal and permission restoration once the
  containing directory exists. Restore source permissions on routing directories
  actually created; do not chmod nonexistent or preserved target names.
- Exhaustion with no admitted effect creates nothing. Do not create then prune,
  run a prior subtree scan, or add automatic cleanup of existing empty directories.
- Keep this recursion owned by the applier. It does not turn a formerly one-sided
  lane into a paired cursor member. Handle preview and partial progress with the
  same action; count only actual created containers as effects.

This is a small concrete effect capability, not a new scheduler or general action
framework. Implement it in diff-dirs' `tree/inspect` validation and `transfer/sync`
kernel, update affected exhaustive matches/tests and JW's receiving lock. Use the
library's version policy. No other upstream feature is prerequisite to reseed.

## 5. One command-owned coordinated loop

After all receivers are native-ready, bind `SyncTarget`s at the actual existing
workspace roots, using one concrete immutable visitor type with lane-specific D
ignore context, links and device facts. Share source native indexes/matchers by
`Arc`. `descriptor` can be a stable versioned textual description of these frozen
inputs; no serialized cursor or persistence format is needed.

```rust,ignore
let runtime = SyncRuntime::new(targets);
let mut cursor = SyncCursor::new(&runtime, Execution::Sequential)?;
while let Some(group) = cursor.next_group(&runtime)? {
    for lane in &group.outcomes {
        if let Err(error) = &lane.outcome {
            return Err(contextual_inspection_failure(lane, error));
        }
    }
    for target in cursor.pending() {
        cursor.apply(&runtime, &group.token, &target)
            .map_err(|error| contextual_application_failure(target, error))?;
    }
    cursor.settle()?;
}
let reports = cursor.reports();
require_all_satisfied(&reports)?;
```

Context helper names above are proposed; cursor methods are actual production APIs.
Always consume pending applications, including zero immediate planned work. Do not
use `drive`'s healthy-peer continuation policy. On error retain returned partial
progress and stop, without running another filesystem walk to construct a report.

Finish requested links using the already loaded configuration. Add an
`apply_loaded_workspace_links` seam in state rather than reload/compare config
files throughout the command. Link existence is checked against the actual final
tree; remove `FutureContent` and its speculative symlink extraction/prewalk.
Absent required targets are ordinary finish errors. Native and link/payload
exclusions establish that these effects cannot alter tracked D; finish needs only
snapshot-free native identity/parent queries, not another full snapshot.

The current `LinkPlan::apply` rolls back its newly created links/directories on
failure. Give the reseed loaded-config seam forward-only application; keep the
reversible wrapper only for existing lifecycle callers. No rollback of reseed
files or earlier link work is required. Print saved O coordinates before mutation
and emit the known partial phase/effect summary before propagating the original
error. Receivers that are merely native-ready are not reported as completed.

Keep observed comparison and actual effects separate. `LaneReport.observed` is
pre-application and treats opaque subtrees as opaque; `totals.completed` includes
recursive writes/removals. Native `seed_leaves::Progress` is a separate phase.
Use u128-compatible aggregate fields; never label list length as successful
reflinks, zero diff as zero recursive work, or lane satisfaction as a JJ proof.
Report command wall time and phase times once, not summed overlapping lane times.

Default is sequential. `payload-parallel` may borrow the existing one command-owned
pool through `pool.install` around coordinated calls and use the library's
`Execution::Parallel`; do not port pair-only J/K settings or parallelize whole
target commands. Missing subtrees are still recursively observed/copied per lane.
N=2 sharing claims are limited to paired immediate-directory groups.

## 6. Preparation, dry-run and exact command semantics

Separate `prepare_source(...) -> PreparedSource` from target refill. Keep current
`SourceSelection`'s pinned remote/bookmark selection and ancestry checks; its
source-only work does not require loading target inventories, payload realization
or future target links.

- `jw prepare`: under the existing preparation-only lock, explicitly snapshot old
  source work; select/fetch the pinned upstream; establish the intended empty
  source child; run configured argv programs sequentially; perform **one final**
  native tracked-tree/parent/change check after all commands. Keep error/exit and
  identity checks at real boundaries. Remove repeated `validate_contents`, ignored
  root selection/audits and per-command whole-tree proofs. The producer promises
  its successful final tracked result, not proof that every intermediate command
  left tracked files untouched.
- Return actual donor WC commit S, upstream goal separately, identities and the
  established native facts. Acquire cache bytes/tree indexes once when needed.
- `reseed --prepare`: compose that producer once, release its lock, then consume
  the handoff without resnapshotting/re-proving S. Ordinary reseed never fetches,
  installs, builds or prepares implicitly.
- `--at`: choose exact D; omission selects S. `--source-at` on plain reseed remains
  the explicit live assertion A; with preparation it retains its upstream-goal
  meaning. Preserve CLI > local precedence and explicit-empty copy policy.
- `--replace-content`: remains explicit broad target clearing; it never changes
  the donor copy selection. It can delete `.env`, caches and other local contents.
- `--allow-unbookmarked`: retain current meaning. Removing the bookmark gate is
  an optional later history decision, not necessary for this architecture.

Make `--dry-run` a **non-mutating root/native-plan preview**, clearly rendered
`PLANNED; payload/path obstructions not inspected`, never `READY`. It does not
snapshot either side, probe CoW with writes, enumerate realized payload roots,
run configured preparation, or claim exact copy/removal counts. Show roots,
recorded S/D, selected mode, declarations, prospective native phases and static
blockers. A supplied source assertion is shown as pending live validation.
With `--prepare`, show the pinned prospective fetch/argv work and explicitly mark
post-prepare S/refill readiness unknown. Do not let old readiness tests force the
execution prewalk back. Detailed recursive preview can remain a later explicit
option; production per-target `Applier::preview` exists, coordinated recursive
preview does not. It is not part of ordinary execution.

These source and dry-run changes implement the commissioned cheap-resolution and
prepared-donor direction. They must land in help, README and tests with the engine.

Preparation's own native cost is explicit: initial source `status` is a full
snapshot; current `fetch_bookmark` uses snapshotting `git fetch`; `new` adds another
full snapshot when advancement is needed; final diff is one full snapshot after
the argv sequence. Thus the stock-command composition has three full source
snapshots, or four with `new`, plus fetch/checkout and configured-program work.
Confirm this command schedule with the instrumented native test. Removing the
old audits/repeated proofs does not make those implicit native snapshots free.

## 7. Pass ledger and performance discriminator

| Work | Multiplicity and actual cost |
| --- | --- |
| Root/config/binding/topology reads | Per command and receiver; no payload enumeration. Native state deserialization is O(tracked-state size), not constant metadata magic. |
| Ordinary donor snapshot | **Zero by default.** Explicit A adds one full native snapshotting diff; preparation has its own producer ledger. |
| Native membership | One S file-list enumeration; one D if distinct; one O per distinct saved old commit. Share identical inputs. |
| Ignore inputs | One extraction per tracked `.gitignore` per distinct tree plus effective excludes; native file-show calls/control-file reads, not a payload scan. No runtime Git ignore queries. |
| Donor cache | One byte read of `tree_state`, O(tracked cache); per-receiver byte write. Not a proof of live files. |
| Preserve target work | One explicit full native snapshot per receiver. Changed tracked contents can be read/hashed; configured initial auto-tracking may add new work. |
| Sparse clear | Another implicit full native snapshot, then native O removal/tree processing. Stock CLI does not expose a writable skip-snapshot mode. |
| Residual check | One no-follow listed O-leaf pass per receiver; no directory contents enumerated. This deliberately refuses instead of repairing skipped clear. |
| Native new | One implicit **empty-sparse** snapshot and repository/checkout transition per receiver. Do not call it zero native work. |
| S install | One required production install per S leaf per receiver, including needed stats, parent checks, link reads and clone opens; no preceding source stat-filter pass. |
| D-addition admission | Listed D-not-S coordinates/ancestors only, during native execution; no retained-subtree census. |
| D=S establishment | One full native snapshotting diff per receiver. Reused file states can avoid tracked content reads; racy mtimes and changed facts can still cause them. |
| D≠S establishment | Restore adds one full adopted-S snapshot and native S→D store materialization; the final diff adds one full D snapshot. Changed D files may require reads. |
| Ordinary transfer | One coordinated paired frontier; source facts shared per paired group. One-sided interiors and actual CoW effects remain per receiver; local replacement removability can add a subtree inspection. |
| Explicit broad clear | An additional full target traversal only for `--replace-content`. |
| Finish | Link-name checks/application and snapshot-free identity/parent queries; no final donor proof or extra tracked snapshot. |

Thus default D=S has **three full receiver snapshots** (save, clear, final diff),
plus an empty-sparse snapshot; D≠S has four full receiver snapshots, plus that
empty-sparse snapshot. This is deliberately more honest than “snapshot once.”
The initial save/bookmark contract explains one otherwise removable pass. Neither
this plan nor the successful native probe measures actual hashed bytes.

The decisive evidence is: zero default donor snapshots; no runtime payload
selector/audit pass; no repeated desired-tree extraction; correct native final
result; zero clean ordinary writes; shared paired source facts at N2. Time the
whole command and native/ordinary phases, then determine whether source-state
reuse and coordination produce a useful reduction. Do not optimize by silently
changing selection, skipping final native establishment or measuring only copying.

## 8. Owners and replacement map

| Owner | Implement/change | Retire from reseed |
| --- | --- | --- |
| `native/jj` | Lean binding query; named native snapshot/clear/new/restore/verify operations; capture/install local tree-state bytes; native profile/capability evidence. Keep exact-ID quoting. | `copy_tree_state`'s restore-on-rejection obligation for this caller; creation may keep its old helper. No donor checkout transplant. |
| `workspace/state` | Loaded link normalization/application seam; existing config precedence, metadata and ownership readers. | Repeated config reads caused solely by engine barriers; no new lifecycle state/journal. |
| `transfer/workspace` | Immutable frozen ignore matcher; pure selector queries and source-qualified visitor; strict `seed_leaves` adapter returning progress; coordinated runtime/loop and conditional-route use. | `Auxiliary::legacy`/realized-root construction and pair `Execution::run` in reseed. Keep them only for still-existing creation callers. |
| `refill/engine` | One resolved command, shared native inputs, small receiver states; serial native prefixes, one cohort transfer, finish; separate prepare handoff and honest error context. | Current monolithic per-target `Plan`, `execute` loop, repeated `validate_source`/`verify_disk`, `select_outside`/`outside`, `preflight_obstructions` recursive audit, `FutureContent`, symlink-diff capability extraction used only for predictions, batch readiness/continuation promises. |
| `app/cli` | Request conversion, cheap preview rendering, phase/failure output and help/docs/tests. | `READY` claims from cheap preview and guaranteed later-peer completion expectations. |
| `workspace/lifecycle` | No reseed orchestration dependency. Later reuse the new transfer/finish capabilities for add/seed. | No creation rewrite prerequisite; old creation behavior is not a requirement for the new engine. |
| diff-dirs | `CreateIfNeeded` action validation, source-only effect/preview support, reporting and focused tests. | No copied filesystem kernel in JW and no new generic scheduling framework. |

Use a few concrete structures, for example `ResolvedReseed`, `NativeInputs`,
`Receiver` and `PreparedSource`, not public provider traits. Keep native workflows
in JW and filesystem effects in diff-dirs. The old route is useful as a preserved
baseline binary and test evidence, not a permanent second reseed driver.

## 9. Coherent build slices and parent checkpoints

### Slice 1 — real N1 native transition plus real selected transfer

Implement the native sequence and a callable replacement orchestration seam in
`refill/engine`; use production `seed_leaves` and `SyncCursor` with **one lane**.
Implement frozen matching and the source-qualified visitor for already-existing
routing directories/direct selected roots. Exercise a real native N1 fixture
through tracked result, `cache/` reconciliation and links. Keep empty policy exact.
This is an executable vertical slice, not a crate/trait refactor.

Until slice 2 supplies missing-route support, explicitly reject an encountered
missing *unselected routing* container in the new internal seam; do not eagerly
create it or silently pre-discover. Do not publish a permanent CLI mode or move
the whole CLI onto this temporarily incomplete surface. The old/new overlap is
limited to these integration commits and is removed at slice 3.

**Parent review checkpoint:** read the native sequence, S/D/local obstruction
tests, pure ignore/selection contract, source-only route gap and actual native
call ledger. Confirm strict S membership owns every adopted leaf and inspect the
real successful N1 result before expanding to cohorts.

### Slice 2 — finish streaming missing-route semantics

Land the narrow production diff-dirs change and its tests in that repository,
then adopt it in JW's receiving lock and visitor. Complete wildcard/path routing,
source-only selection, target-only retention/removal and opaque replacements.
The full ordinary policy now works without realizing all roots first. These are
coherent library and consumer commits within one build slice, not a new project.

### Slice 3 — usable N2 CLI replacement and preparation handoff

Connect the actual `run`/`prepare` entrypoints to the replacement, coordinate all
native-ready targets, integrate loaded links/reporting and new dry-run behavior.
Remove the old reseed driver and its unused discovery/proof/prediction machinery
in the same landing. Keep shared helpers used by creation under their native
owners. Land README/help and affected tests with this behavior.

**Parent working end-to-end checkpoint:** run N1 and mixed N2 using the real CLI,
including one surfaced failure after effects. Inspect command output, native
identity/result, authority outcomes and counters. End-to-end usefulness decides
acceptance, not adapter completion.

### Slice 4 — bounded evidence and final corrections

Run affected checks, capture native versus instrumented counts and a small
equal-policy before/after observation, fix only demonstrated gaps, and write the
implementation account with revisions and limitations. Commit corrections
separately. No seed rewrite, package install or actual-workload import is required.

## 10. Bounded verification

Use the current native fixtures and `/usr/local/bin` ahead of mise shims:

```sh
PATH="/usr/local/bin:$PATH" JW_TEST_REQUIRE_COW=1 \
  cargo nextest run --locked -p jj-waltz \
  --test reseed --test prepare --test reseed_bookmarks
PATH="/usr/local/bin:$PATH" \
  cargo nextest run --locked -p jj-waltz-refill-engine --test freshen_source
```

These are the current test-target names. Run affected library tests, formatting
and clippy, then the relevant app tests. Use owned filesystem fixtures;
do not globally relocate discovery tests inside the checkout. Repeat with
`payload-parallel` for the focused coordinated cases, not the entire matrix by
habit. The already-reproduced colocation cleanup failure is baseline evidence,
not this work's unexplained regression.

Required semantic cases can share two small native fixtures:

1. D=S and D≠S, real reflinks/private writes, symlink/mode/type transitions,
   S-only deletion, D-only addition, old tracked history and root/`.jj`/registration
   retention. Final native diff and exact parent are asserted independently.
2. Occupied S-only local with **matching bytes/stats**, occupied D-only local,
   and a directory-collapse obstruction with retained child. Refuse without
   adopting/deleting the local; native progress elsewhere may remain.
3. N2 mixed `cache/`: paired clean target, missing subtree, changed leaf,
   target-only subtree and selected-root leaf collision. Source absence at a
   selected root retains it; source absence *inside* an admitted root deletes.
4. Empty/disabled policy, nonignored match, forced-tracked overlap, active link
   conflict, nested administrative/special entry. Equal/opaque entries must obey
   the same rules. Include a nonignored ancestor with a valid descendant match.
5. Conditional route: no selected descendant produces **zero writes and no empty
   directories**; a selected empty directory creates its necessary ancestry;
   nested source-only copy and staged replacement use final logical paths.
6. One root-known unsupported mode fails before native clearing; one required
   clone or deep apply failure after visible progress stops the command without
   downgrade, second algorithm, recreation, rollback or processing later peers.
7. Prepare runs once and releases its existing lock before refill; the handoff
   causes no donor resnapshot. Plain default performs no donor snapshot (an
   unprepared live donor is outside its contract); explicit A detects tracked divergence.
   Cheap dry-run performs no source/target snapshot or writes and does not say READY.

Use instrumented argv/cwd chronology to assert zero default donor snapshots,
single shared S/D enumeration and the declared per-receiver snapshot schedule.
Count Git rule **capture** separately from forbidden per-payload Git queries.
Use existing library acquisition tests plus one JW N2 observation-sharing check;
effects counters alone cannot prove source sharing. Do not build a production
telemetry framework to count the test fixture.

Performance tooling is already repaired. Before editing, preserve a pinned
baseline using `jw-perf build` if a new baseline is needed; after coherent commits,
build the candidate the same way. Use a small synthetic fixture (hundreds of
entries, a few MiB), N1 and N2 clean/sparse-change samples, a bounded balanced block
count, and equal selection/native results. Native timing uses the real tools;
instrumented process counts are a separate run and not native latency. Retain
setup/verification cost separately. No fresh workload snapshot/import, repeated
11-GB copies, full-file hashing campaign or throughput claim from this plan.

## 11. Human choices versus immediately executable work

**No new ordinary breadth/deletion decision is required to build the recommended
source-qualified contract.** Its pure matcher and conditional routing preserve
the useful existing outcomes; later discovery of a blocker replaces the old
all-target preflight guarantee under the human's fail-fast direction.

Sol can start slice 1 now. The real native protocol, strict installer, frozen
matching, existing-route N1 transfer and test evidence do not await a policy vote.
The following are genuine optional outcome choices, not invented routine approvals:

| Choice | Concrete outcome / implementation consequence |
| --- | --- |
| **Preserve source-qualified scope (recommended)** | `.env` outside selection stays local; vanished donor `cache/` leaves target `cache/`; missing routes with no matches leave nothing. Implement the small lazy-route library effect. |
| Permit empty discovery routes | Same leaf/deletion authority, but `**/node_modules` can create empty target `packages/a/src` or other source-only routing trees even with no match. Existing `Create` suffices; this is a visible namespace-policy relaxation requiring human selection. |
| Make selected regions declared mirrors | A configured `cache` can remove target `cache/` when donor `cache/` vanished; ignoredness no longer necessarily gates selection. This needs an explicit selection/ignore contract and documentation. It still does **not** make empty selection broad copying. |

If the parent wants to avoid the identified library addition, the **minimum human
question** is whether empty unselected routing directories are acceptable. Silence
means preserve the existing footprint, not permission to create them. A new broad
update/root-mirror default is unnecessary for the milestone. Removing the bookmark
gate is also optional and independent; retain it in this commission.

## 12. README obligations and Sol-ready commission

Explain normal use first, then these concrete limits: prepared/idle donor and idle
receivers; three or four actual native full snapshot passes; required CoW with
ordinary I/O failure; strict S-only local obstructions even for D≠S; ordinary scope
and donor-absence examples; stat/literal-link equality, not byte certification;
ignored deletions absent from JJ history; dependencies may not suit a different D;
and failure can leave partial native/full-sparse state and partial peers. After
adoption, full sparse patterns alone do not certify a healthy interrupted target.
An ordinary later JJ snapshot can record the partial state. Give inspection advice,
not a promised resume/recovery protocol or durable journal.

**Commission:** implement slices 1–4 in the current six owners. Use the shortened
native protocol proven here, preserve source-qualified ordinary authority with a
pure frozen matcher and the narrow production lazy-route effect, then replace the
CLI driver with fail-fast coordinated N1/N2 execution and a source-only preparation
handoff. Return at the native+authority and usable end-to-end checkpoints. Commit
coherent owned paths, keep the two starting research files untouched, and report
actual checks/costs rather than restating the architecture. Parent owns topic-index
and commissioning integration. No subagents are implied by this planning document.

### Connections

- [Architecture](architecture0.gpt6a.md) owns the overall phase/pass boundaries;
  this plan selects the concrete native sequence and resolves its pure-policy
  implementation question without selecting broader ordinary authority.
- [Library assessment](library0.gpt6a.md) establishes the delivered coordinated
  API. The conditional missing-route finding above narrows what its current
  routing actions can express; it does not invalidate the verified sharing loop.
- [Current census](current0.gpt6a.md) remains the source-derived baseline and
  repeated-work evidence. Its earlier peer-continuation and creation-first
  sequencing recommendations are superseded by the human's
  [lifecycle direction](assessment1.gpt6a.md) and this commission.
