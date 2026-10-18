---
type: ResearchReport
title: "JJ workspace state, portability, and move-aside/restore as a reseed optimization"
description: Source-grounded inventory of the state a JJ workspace needs, upstream jj-waltz vs our fork for creation and in-place reseed, and a concrete evaluation of moving crucial directories aside and restoring them over the result, with lawful sequences and remaining costs.
resource: /design/sync-scope/jj-state0.glm53max.md
status: draft-proposed-for-review
generated: { by: "model:zai-coding-plan/glm-5.3#max", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eaf5dbdfdffe75JGNrHHbcAtrD"
    title: Commissioned G (glm-5.3 max) research wave on JJ state requirements, upstream-vs-fork comparison, and move-aside/restore optimization; read JJ and jw sources, ran controlled tiny probes, wrote this report
  - resource: /design/sync-scope/recovery0.gpt6a.md
    title: Human brief recovering the synchronization-boundary questions; benchmark cancellation facts
  - resource: /design/sync-scope/design0.gpt6a.md
    title: Authority-based scoped-filesystem reconciliation proposal; recommended defaults treated as unselected
  - resource: /design/up-performance/streaming0.sol61m.md
    title: Delivered streaming reseed integration and its contracts
  - resource: /design/up-performance/primary0.sol61m.md
    title: Cancelled actual-workload campaign; copying/hash overhead and disk-pressure facts
---

# JJ workspace state, portability, and move-aside/restore

Written to answer the operator's questions: *what state does JJ need; what does jw
do upstream vs our fork; and can we move crucial directories aside before the
operation and move them back over the top afterwards as a performance
optimization?* Research only; nothing here authorizes implementation or revives
the cancelled benchmark.

## 1. Exact references and claim labels

Every factual claim below is labeled:

- **(S)** read in JJ source at `/home/rektide/a/jj-vcs/jj` (symlink to
  `/mnt/bitblaster/archive/jj-vcs/jj`), commit `803ef00a0a6ee4ddc275d3138d5f9e7326399906`
  (`v0.45.1-99-g803ef00a0`, dated 2026-09-27, "sparse-gitignore: Take .gitignore
  into account in sparse copies"). File paths cited relative to that root.
- **(B)** behavior of the **installed binary** `/usr/local/bin/jj` →
  `/usr/local/src/jujutsu-git/repo/target/release/jj`, reporting version
  `0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`. `33d3a595` (2026-09-08,
  "cargo: bump jsonschema") is an ancestor of `803ef00a`; the binary is **80
  commits behind** the source checkout. Where they can differ, I verified the
  binary's source via `git show 33d3a595:...` in `/usr/local/src/jujutsu-git/repo`
  (label **(S33)**).
- **(P)** observed live, either on real workspace state (read-only) or in
  disposable probe repos under `/tmp/opencode/jj-probe*` using the installed
  binary with the fork's exact config flags
  (`fsmonitor.backend=none`, `snapshot.auto-track=none()`,
  `snapshot.auto-update-stale=false`). Probes are functional evidence, not
  benchmarks.
- **(U)** read in upstream jj-waltz at
  `/home/rektide/src/jj-waltz-upstream-state-research`, a pristine tracked
  checkout of `cd7bb1eb6451b09c06853b0347f856b4203f7c89` ("Merge pull request #59
  from EzraCerpac/wip/native-workspace-remove", 2026-10-14, `main` of
  `github.com/EzraCerpac/jj-waltz`).
- **(F)** read in our fork at `/home/rektide/src/jj-waltz` (same repository, our
  working copy; heavily modified relative to `cd7bb1eb`: 26 paths differ, plus
  fork-only `reseed/`, `freshen/`, `cow/ignored.rs`, `cow/payload.rs`,
  `lifecycle/ignored.rs`).

Live topology fact used throughout **(P)**: all three checkouts of this repo —
the colocated primary at `~/archive/EzraCerpac/jj-waltz` (`default`), our fork
(`jj-waltz`), and the research workspace (`upstream-state-research`) — share one
repo store. Each secondary workspace's `.jj/repo` is a **file** containing a
relative path (`../../../archive/EzraCerpac/jj-waltz/.jj/repo`); the primary's
store holds `git_target` → `../../../.git`; our fork additionally has a `.git`
gitfile `gitdir: ../../archive/EzraCerpac/jj-waltz/.git/worktrees/jj-waltz`
(a Git worktree registration, created by jw's colocate path), while the
no-frills research workspace has none.

## 2. What state JJ needs (inventory and portability)

### 2.1 The pieces

| State | Location (this jj generation) | Bound to | Move out & return unchanged? | Copyable from a donor? |
| --- | --- | --- | --- | --- |
| Workspace→store pointer | `.jj/repo`: file with relative-or-absolute path to the shared repo dir | relative form is bound to the position of `.jj` **(S** `lib/src/workspace.rs:572-598`, written relative by `init_workspace_with_existing_repo`, `workspace.rs:374-413`**)** | yes as a file, but a whole-workspace move invalidates the relative form; absolute form survives workspace moves, not store moves | n/a (must point at the target's own store) |
| Working-copy impl selector | `.jj/working_copy/type` (`local`) **(P)** | nothing | yes | identical everywhere |
| Checkout binding | `.jj/working_copy/checkout`: protobuf `Checkout{operation_id, workspace_name}` **(S** `lib/src/protos/local_working_copy.proto:69-77`, reader/writer `lib/src/local_working_copy.rs:2676-2726`**)** | `operation_id` must be an ancestor of the current op head, else the working copy is *stale* (recovered by `jj workspace update-stale` / auto-update; `docs/working-copy.md` "Stale working copies")**;** `workspace_name` keys `View.wc_commit_ids` | yes (position-independent; staleness after absence is by-design recoverable) | **no** — donor's op line and name describe the donor |
| Tree state (stat cache + tree binding) | `.jj/working_copy/tree_state`: protobuf `TreeState{tree_ids, conflict_labels, file_states(path→mtime_millis,size,file_type,conflict data), sparse_patterns, watchman_clock}` **(S** `local_working_copy.proto:49-67`; load/save `local_working_copy.rs:1107-1225`**)** | tree ids must resolve in the store; file states are mtime/size/type-bound to disk content, **not inode-bound**; whole-file paths are repo-relative | **yes, exactly**: a whole-workspace rename preserves inode, mtime, size, so every clean stat stays clean; `tree_state`'s own mtime (race guard) also survives rename | only when donor's tree equals the target's desired tree **and** the target's disk files carry the donor's stats — this is upstream jw's validated creation trick (§4), not a general operation |
| Sparse patterns | inside `tree_state` (no separate file in this generation; observed live **(P)**) | workspace choice, not revision | with tree_state | with tree_state (upstream relies on this) |
| Watchman clock | inside `tree_state` | the checkout that recorded it | with tree_state | meaningless across checkouts; jw refuses watchman everywhere it matters **(U** `src/jj.rs:667-679`**)(F** `src/reseed/engine.rs:800-813`**)** |
| Mutation lock | `.jj/working_copy/working_copy.lock` (flock) **(S** `local_working_copy.rs:2759-2762`**)** | transient | n/a | n/a |
| Workspace registration | shared store `workspace_store/index`: protobuf `Workspaces{name, path}`, path stored **repo-relative** when possible **(S** `lib/src/simple_workspace_store.rs:151-183`; `simple_workspace_store.proto`**)**; verified live, the installed binary also writes relative paths **(P)** | position of each workspace relative to the repo dir | the file moves trivially, but a moved workspace leaves a **stale registration**; no public command re-registers a path (`jj workspace add` requires an empty destination; `forget` deliberately leaves the path entry — `cli/src/commands/workspace/forget.rs:92-96` **(S)**) | n/a |
| WC-commit mapping | `View.wc_commit_ids: map<workspace_name, commit>` in op-store views **(S** `simple_op_store.proto:98-99`**)** | name, not path | survives workspace moves | n/a |
| Op history | `op_store/`, `op_heads/heads/` (a directory) **(P)** | append-only, shared | moves with the store | n/a |
| Colocated Git linkage | `store/git_target` (relative); for colocated secondary workspaces a `.git` gitfile plus the primary's `.git/worktrees/<name>/gitdir` back-pointer — both relative **(P)** | paths on both sides | git-side links break on move; `git worktree repair` is the native fix; upstream jj itself calls it when adding a worktree to a populated dir **(S** `lib/src/git.rs:1864-1894` `create_worktree`**)** | n/a |

### 2.2 What the snapshot and checkout actually trust

- **Clean test (S** `local_working_copy.rs:1943-1956`, `is_clean` at `308-312`,
  identical at **S33)**: a tracked file is trusted without hashing iff
  `file_type ∧ mtime ∧ size` match the recorded state **and** the file's mtime
  is strictly older than `tree_state`'s own mtime (the race guard). There is no
  inode or device component. **Consequence:** any transfer that preserves mtime
  and size (reflink + explicit `utimens`, or plain rename) keeps snapshots
  stat-only; any transfer that rewrites files (jj checkout included) forces
  re-stats but not re-hashes for tracked paths whose stats were re-recorded by
  the checkout itself.
- **Deletion semantics (S** `local_working_copy.rs:1903-1933` `emit_deleted_files`**)**:
  snapshot records a deletion for every recorded (tracked) path absent on disk.
  Untracked/ignored paths have no file states, so **their absence is invisible
  to snapshots**. This is the load-bearing fact for move-aside.
- **Checkout writes are unconditional in the lib (S** `local_working_copy.rs:2458`
  `// TODO: Check that the file has not changed before overwriting/removing it`,
  same text at **S33:2361)** — but the *diff driving them* is tree-vs-tree, and
  paths that exist on disk as **untracked** content are skipped with
  `skipped_files` and a placeholder state ("Warning: N of those updates were
  skipped because there were conflicting changes in the working copy")
  **(P)**, probe-verified twice: prefilled files kept byte-identical content and
  a 2020 mtime across `jj sparse reset`; only missing files were written with
  current mtimes. Placeholder states mean "re-stat on next snapshot"
  **(S** `local_working_copy.rs:314-316`**)**, after which matching stats stay
  clean — no hashing pass.
- **Stale detection (S** `docs/working-copy.md`; `cli/src/commands/workspace/update_stale.rs`**)**:
  the `operation_id` in `checkout` vs the op head. Other workspaces' jj commands
  advance the shared op head, so any window in which the target runs no jj makes
  it stale; recovery is a supported command, and costs one working-copy rebuild
  (tree-diff based; cheap when tree_state and disk agree).

### 2.3 Answers to "what can move out and return unchanged"

- **Entire ignored/untracked subtrees** (dependency caches, build outputs):
  fully invisible to jj while absent; rename out and back preserves everything
  jj never knew about them. Constraints: same filesystem (rename `EXDEV`
  otherwise), same mount topology the fork already guards
  **(F** `src/reseed/topology.rs`**)**, and no *tracked* paths inside (a
  tracked leaf inside a moved root reads as deleted to any snapshot that runs
  while it is gone — during reseed the snapshots run with the sparse selection
  empty and `auto-track=none()`, so they are blind by construction **(F**
  `src/reseed/engine.rs:1447-1454`**)**).
- **The workspace's own `.jj`**: must stay in place for any jj command to find
  the workspace (discovery walks up for `.jj`); the metadata *files* inside are
  individually position-independent, which is what makes option (b) in §5
  possible at all, but nothing is gained by moving them.
- **From a donor, unsupportable**: `checkout` (op-line + name mismatch → stale
  then converge, but claims a tree the donor didn't build for this workspace);
  `tree_state` with a different tree (stats describe the wrong content; the
  next snapshot re-hashes and converges, but the in-between state can drive a
  wrong materialization diff). Upstream jw transplants `tree_state` only in the
  one situation where it is truthful, validates with a real jj command, and
  rolls back on rejection **(U** `src/jj.rs:698-731`, `src/workspace.rs:716-732`**)**.

## 3. What stock JJ can and cannot do (no guessed commands)

Verified in installed-binary help and in source at both revisions:

- `jj workspace add`: refuses a non-empty destination **(S**
  `cli/src/commands/workspace/add.rs:139-145`**)**; supports
  `--sparse-patterns {copy,full,empty}` (default **copy** — inherits the current
  workspace's patterns), `-r/--revision`, and `--colocate/--no-colocate`
  (colocate creates a real Git worktree via `create_worktree`) — present in both
  **(B help)** and **(S)**. Installed subcommands: `add forget list rename root
  update-stale`; `remove` exists only in newer source **(P)** — upstream jw
  feature-detects it **(U** `src/workspace.rs:424`**)**.
- There is **no** stock command that adopts pre-populated content for an
  existing workspace, and **no** command that establishes destination state
  without a full traversal of whatever it is doing: every snapshot stats all
  in-sparse paths; every checkout walks the full tree diff it is applying. The
  only traversal-free state change is the private-layout `tree_state`
  transplant, which is exactly what upstream jw does at creation with
  validation and rollback.
- `jj restore` with no arguments restores the working copy from its parent(s)
  **(B help)** — the load-bearing step of upstream's adoption flow.
- `jj sparse set --clear` deletes tracked files from disk; `jj sparse reset`
  re-includes everything, writing only paths that are absent or tracked-dirty,
  skipping present untracked paths as "conflicting changes" **(P)**.

## 4. Upstream jw vs our fork

### 4.1 Creation (`jw add`, upstream `cd7bb1eb`) — the adoption flow

**(U** `src/workspace.rs:544-732`, `src/jj.rs:648-731`, `src/cow.rs`**)**:

1. Resolve the full base commit id before any mutation; `jj workspace add
   --name N --revision <base> [--sparse-patterns empty] [--colocate|--no-colocate] <path>`.
   With `clone_from`, `--sparse-patterns empty` makes JJ write **no** files.
2. `cow::clone_files` reflinks (FICLONE/clonefile) every file the source tracks,
   then **restores the source's mtime onto the new inode** — without this the
   stat cache would never match.
3. `jj::copy_tree_state` copies the source's `tree_state` protobuf into the new
   workspace — carrying tree ids (source's tree == base tree), truthful file
   stats, sparse patterns. Private layout, explicitly acknowledged as such, with
   `jj status` as the validator and `write_tree_state(previous)` rollback →
   fall back to an ordinary full add **(U** `src/workspace.rs:630-643`**)**.
4. `jj restore` — @'s tree becomes the parent (base) tree; against the adopted
   `tree_state` the checkout diff is **empty**, so nothing is written and no
   hashing happens. Watchman workspaces are refused outright.
5. Provenance (`operation_id`), optional bookmark + `.jj/jw-bookmark`, Git
   worktree ownership verification for colocate **(U** `src/ownership.rs`**)**,
   rollback of partial creations.

Net cost shape: one reflink per tracked file + one stat pass. No full-content
writes, no hashing. This *is* donor-state reuse, and it is honest because donor
tree == desired tree by construction.

### 4.2 Our fork's additions

**(F)**: the fork keeps the upstream creation flow (same `materialize_clone` /
`adopt_source_tree_state` at `src/workspace.rs:623-660`) and adds:

- **`reseed/`** (`src/reseed/engine.rs`, 2205 lines): in-place refill of
  *existing* workspaces through stock jj's sparse interface. Sequence
  (`engine.rs:1421-1566`): quiescence checks (fsmonitor none, no watchman,
  `800-813`); sparse-list must already be `.` (`815-825`); tracked-edit
  verification against a frozen operation (`verify_disk`, `1024-1052`);
  bookmark batch proofs; `jj sparse set --clear`; manual removal of recorded
  old∪desired paths; `jj new <base>` proven to be an exact empty child;
  optional `replace_content` purge; `cow::clone_files` prefill;
  `jj sparse reset`; ignored-roots mirror via diff-dirs `sync`
  `new_replacing_checked(ReflinkRequired)`; links application; final
  identity/topology/empty-child re-verifications. Plus: repository-scoped
  operation lock (`jj-waltz/reseed/operation.lock`), device+inode identity
  pinning, frozen ignore oracle (`cow/payload.rs`), selection groups
  (`selection.rs`), symlink support proofs via native git diff, out-of-band
  recovery policy (never automatic repair).
- **`freshen/`**: pin one upstream identity (bookmark/commit) before fetch so
  reseed never re-discovers a moving line.
- **`lifecycle/ignored.rs`, `cow/ignored.rs`**: copy_ignored selection
  (patterns, outermost eligible roots) and policy plumbing.
- Removed/changed relative to upstream: manager/snapshot surface changes,
  ~4180 deletions across shared files (diff summary over `src/`).

Two mechanics worth stating precisely, because they answer "can JJ establish
destination state without another full traversal, and what does donor-state
reuse buy":

- In the fork's reseed, after `jj new <base>` the recorded tree is the empty
  child, so `jj sparse reset` materializes nothing (empty-tree diff); **all
  tracked content arrives via `clone_files`**. The tracked cost is one reflink
  pass; the first *ordinary* jj command afterwards (default
  `snapshot.auto-track=all()`) must hash the untracked-on-disk files once to
  build @'s tree — a read pass, deduplicated in the store, converging to a
  clean tree equal to base. My probe of the alternate shape (reset against a
  full recorded tree) shows the skip-conflicting path: prefilled files are left
  untouched and get placeholder states that resolve stat-only **(P)** — that is
  the shape upstream's creation would exhibit *without* tree-state adoption.
- Upstream's creation adoption (`copy_tree_state` + `jj restore`) avoids even
  that hash pass: stats match, tree matches, everything stays stat-only. That
  is the full value of donor state reuse, and it is only available when the
  donor's tree **is** the desired tree. For reseed where the target moves from
  old tree O to desired tree D ≠ donor S's tree, adoption of S's `tree_state`
  would be false state; the fork does not do it, correctly.

## 5. Move-aside / restore algorithms, concretely

The operator's framing — "move some crucial directories out of the way before
our operation, then move them back-over-top" — evaluated as four distinct
algorithms. "Crucial directories" classified per §2: native `.jj` (and `.git`
linkage) must stay; ignored/untracked roots and dependency caches are the
movable class; runtime cache state (watchman clock) is refused already.

### (a) Keep `.jj` in place; stash only untracked/preserved roots — **viable, recommended**

1. **Before:** for each selected root R (ignored in target, containing no
   tracked paths — provable against the frozen tracked sets the engine already
   computes): `rename(root/R, stash/R)`. Stash is a **same-mount sibling** of
   the workspace root (e.g. `<root>.jw-stash/`), invisible to every jj walk.
   A manifest (list of moves) is written next to the stash.
2. **During:** the existing reseed executes unchanged, minus the mirror work
   for stashed roots (they are not there to mirror into, and are selected out).
   No jj command may treat the absence as deletion: with the sparse selection
   empty and `auto-track=none()`, snapshots are blind to untracked absences by
   construction **(S** §2.2 deletion semantics**)**.
3. **After:** restore-over-top per root (§6 sequence). Then the normal
   `verify_disk` and `require_full` checks run; a stashed root that contains a
   path now tracked in the desired tree must surface as a rejected edit rather
   than being silently clobbered — the engine's tracked-verification already
   provides this if the restore happens before the final `verify_disk`.

Preserves: workspace registration, root identity, `.jj`, git linkage, the
desired tracked tree. Nothing about jj's state changes. Renames are O(entries)
namespace operations — at root granularity, O(#roots).

### (b) Stash the target's own `.jj` while a filesystem transfer replaces the root's contents; restore the same metadata — **workable but strictly dominated**

Lawful only if **zero** jj commands run while `.jj` is absent (workspace
discovery fails first, which is fail-closed, not corrupting). Restoring the
identical `.jj` files is position-independent (§2.1) and mtime-preserving, so
the stat cache resumes clean; the only guaranteed artifact is staleness
(`operation_id` behind the shared op head) → one `update-stale` equivalent on
next use, a supported recovery. Nothing in (a) requires this, and per-path
renames never need the root itself vacated, so (b) buys nothing except risk.

### (c) Copy donor working-copy state (`tree_state`, stats) into the target — **only at creation, already shipped upstream; invalid for reseed D≠S**

This is upstream's `--cow` creation flow (§4.1), including validation and
rollback. For a reseed whose desired tree differs from the donor's, it is
false state (§2.3). A hybrid — adopt donor stats *after* materializing D —
gains nothing: materialization already re-records stats for everything it
writes, and skipped-prefill paths already stay stat-clean via placeholders
**(P)**.

### (d) Stage a whole replacement workspace root elsewhere; swap roots — **lawful but disk-hostile**

Swapping whole roots by rename is O(1) and keeps the final path (hence
registration) intact, but: staging a full parallel tree is exactly the disk
doubling that hurt at 99% utilization **(F** `primary0.sol61m.md` cancellation
facts**)**; the staged `.jj/repo` pointer and Git worktree gitfile are
path-bound both ways **(P** §1**)**, so a staged-then-swapped root needs `.jj`
pointer repair and `git worktree repair` afterwards; and an interrupted swap
leaves two candidate roots. Dominated by (a) for every goal stated.

### Which jj commands may run while paths are absent

- With **tracked** paths absent: any snapshot-capable command records
  deletions (§2.2). Never allow this outside the reseed's guarded window.
- With **untracked/ignored** paths absent: any jj command is safe; snapshots do
  not see them.
- With `.jj` absent: no command finds the workspace (fail-closed).
- During the fork's actual window (sparse empty, `auto-track=none()`,
  `auto-update-stale=false`, fsmonitor none): tracked absences are also not
  observed, which is precisely why the engine can clear and refill in place.

## 6. "Move back-over-top" is not a filesystem union

`rename(2)` over a populated directory fails (`ENOTEMPTY`); over an empty
directory it succeeds; a file/symlink at the target is atomically replaced; a
dir onto a file fails `ENOTDIR`. So restore-over-top must define, per top-level
stash entry, one of **preserve** (result wins; drop stash entry), **replace**
(stash wins; move result's entry aside first), or **merge** (recurse; only as
deep as policy is defined). Restoring the *whole old root* instead of merging
would discard the operation's tracked result — never that.

Smallest lawful sequence per root R, policy **replace** (the natural reading of
"preserved cache roots"):

1. If `root/R` absent → `rename(stash/R, root/R)`; done.
2. If `root/R` is a file/symlink or empty dir → `rename` replaces it (Linux
   semantics); done — but only after asserting R is not tracked in the desired
   tree (else this is clobbering tracked authority; refuse).
3. If `root/R` is a non-empty dir → the result produced content there (mirrored
   payload from donor, or new tracked tree): per policy, move result's entry
   into `stash/R.new` first, then rename stash entry in (a swap, both O(1)),
   leaving both versions discoverable; or recurse with an explicit merge rule.

Edge conditions that must be stated, not implied: parent-path/type conflicts
(a stashed dir `pkg/cache` under a result where `pkg` is now a *file* — the
engine's type-transition preflights already model this class); symlinks rename
as links (never followed — same rule as everywhere in jw); modes are preserved
by rename; same-filesystem only (`EXDEV` must fail loudly, mirroring the
ReflinkRequired discipline); mount boundaries and autofs/aliasing per the
existing topology guard; concurrent readers of a stashed root see it vanish
(quiescence is already a reseed precondition — the window simply includes the
stash moves); crash/interruption leaves the stash + manifest as the
discoverable staged state, and recovery is deliberately out-of-band (matching
the fork's stated "never automatic repair" policy) — a manifest plus the
existing `operation.lock` and bookmark-guard recovery notes are sufficient; no
journal framework is needed or proposed.

A move is O(1) namespace work per entry. It does not by itself prove jj state
correct (that is what the engine's final `verify_disk`/`require_full`/
empty-child checks are for), and it does not make later snapshots free (they
were already stat-only for tracked paths; stashed roots were never tracked).

## 7. Performance comparison — what actually disappears

Against the delivered streaming reseed **(F** `streaming0.sol61m.md`**)**,
for a root that exists in the target and is pure-ignored:

| Approach | Walks | Copies/writes | Hashing | Disk pressure | Notes |
| --- | --- | --- | --- | --- | --- |
| Current mirror (`copy_ignored` → diff-dirs sync, ReflinkRequired) | both trees, divergent entries | reflink per differing file; delete extras | none (ignored) | ~divergence-sized new extents | donor→target sync semantics |
| **Preserve-in-place** (skip the root entirely; no moves at all) | none | none | none | none | zero-cost when target's copy is already wanted; this is "exclude during streaming" as a first-class policy |
| Move-aside + restore-over-top | none during op | O(1) renames ×2 per root + collision handling | none | none (namespace only) | only needed when the operation would otherwise destroy/replace the root (purge, type transitions) |
| Let jj materialize (no CoW) | full tree diff | full write pass (unconditional per §2.2) | none during checkout; stat-only after | full content duplication | the disk-pressure failure mode already observed |

Unavoidable work that remains in every variant: the frozen-plan preflights
(tracked-set computation, ignore-rule freezing — already batched once per
group), one reflink per *tracked* file when the desired tree differs from disk,
and — in the fork's current end-state (empty child @) — one content-hash pass
on the first ordinary snapshot afterwards, which store dedup keeps read-only.
No timing claims are made here; the cancelled campaign's fixture overhead is
not re-litigated.

## 8. Recommendation and human decisions

**Recommended viable approach:** treat "preserved roots" as a policy sibling
of `copy_ignored` — *preserve-in-place* by default for ignored roots that the
operation has no reason to touch (zero moves, zero cost, no new semantics
beyond selection), and *move-aside/restore-over-top* (algorithm (a) + §6
sequence) only for roots that must be vacated (explicit `replace_content`, or
tracked-type transitions at/under those paths). Static per-root decision at
plan-freeze time, recorded in the manifest the engine already snapshots; no
rule interpreter, no new runtime, no preflight accumulation.

Decisions that are human's, not mine:

1. **What is being preserved:** the target's *own* copy of an ignored root
   (preserve-in-place / move-aside), versus donor authority (current mirror
   semantics), versus both by region. These are different answers to "whose
   cache wins", and the prior authority-based design
   (`design0.gpt6a.md`) deliberately left the default unchosen.
2. **Final desired-tree identity:** the fork's current end state is an exact
   empty child of base plus disk content that the next ordinary snapshot
   auto-tracks and hashes once. If that one-time hash pass is unacceptable,
   the alternative is upstream-style truthful adoption — only available when
   donor tree == desired tree (e.g. reseeds pinned `--at` the donor's own
   revision), which is a scope decision, not a code tweak.
3. **Collision policy on restore** (§6): replace-with-swap, preserve-result,
   or refuse-and-report when the operation produced content at a preserved
   root's path.
4. Whether stash collision "merge" is ever offered at all, or always refused
   (my recommendation: refuse; keep the sequence static and reviewable).

## 9. Unsupported, unknown, and not claimed

- I did not benchmark anything; every cost statement is structural (walks,
  writes, renames, hashes) from source semantics, plus the already-recorded
  campaign facts. No new fixtures were created outside `/tmp/opencode` probes.
- The installed binary's exact internal ordering that produces the
  skip-conflicting behavior is evidenced by probes and by the placeholder
  machinery in source; I did not trace the 33d3a595 CLI call graph to the last
  frame. The behavior itself is confirmed twice with the fork's exact flags.
- `jj workspace remove`, repo-relative workspace-store paths, and
  `workspace add --colocate`'s `create_worktree` repair behavior exist in
  source ≥ the checked-out revision; **our installed binary lacks `remove`**
  and I verified relative registration paths live. Any fork feature-detection
  code should keep distinguishing these (upstream jw already does).
- Nothing here touches the user's uncommitted `design/up-performance/handoff0.glm53m.md`;
  the upstream research workspace was left pristine and unmodified.

## 10. Connections to prior design

- `recovery0.gpt6a.md` commissioned exactly this question set after the
  campaign cancellation; this report supplies the jj-state and move-lawfulness
  halves. `design0.gpt6a.md`'s authority/exclusion vocabulary is the right
  frame for §8's decisions: preserve-in-place is "target authority for this
  region"; the current mirror is donor authority; move-aside is target
  authority with a temporary absence.
- `streaming0.sol61m.md` delivered the mirror machinery this report compares
  against; `primary0.sol61m.md`'s disk-pressure record is the empirical reason
  algorithm (d) is rejected and preserve-in-place leads.
