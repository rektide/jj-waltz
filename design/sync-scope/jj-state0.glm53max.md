---
type: ResearchReport
title: "JJ workspace state, portability, and move-aside/restore as a reseed optimization"
description: Source-grounded inventory of the state a JJ workspace needs, upstream jj-waltz vs our fork for creation and in-place reseed, and a concrete evaluation of moving crucial directories aside and restoring them over the result, with lawful sequences and remaining costs. Corrected after parent source review; v1 conclusions about sparse-reset materialization, adoption preconditions, staleness, and placeholders were wrong and are retracted in §0.
resource: /design/sync-scope/jj-state0.glm53max.md
status: draft-proposed-for-review
generated: { by: "model:zai-coding-plan/glm-5.3#max", at: 2026-10-18 }
updated: { by: "model:zai-coding-plan/glm-5.3#max", at: 2026-10-18, note: "v2: corrected after parent source review of e2418219" }
sources:
  - resource: "opencode://ses_eaf5dbdfdffe75JGNrHHbcAtrD"
    title: Commissioned G (glm-5.3 max) research wave on JJ state requirements, upstream-vs-fork comparison, and move-aside/restore optimization; read JJ and jw sources, ran controlled tiny probes, wrote this report, then corrected it against parent source review (new.rs tree inheritance, check_stale freshness, placeholder hashing, find_workspace_dir ancestor fallback, upstream cow test)
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

## 0. Correction record (v1 `e2418219` → this version)

Parent source review found eight material errors in v1. All are corrected in
the body; the retracted claims are recorded here so the delta is reviewable.

1. **v1 claimed `jj new <base>` records an empty *tree*, so the fork's
   `sparse reset` "materializes nothing" and "all tracked content arrives via
   clones", with a post-reseed auto-track hash pass.** Wrong: `jj new`
   constructs the child with the merged **parent tree**
   (`cli/src/commands/new.rs:208-213` at S and S33: `new_commit(parent_ids,
   merge_commit_trees(parents))`); "empty child" means empty *change*, and the
   fork's own `exact_empty_child` check asserts exactly that. The recorded
   tree at reset time is the **full base tree**, so `sparse reset` *does* run
   the add-diff (§4.2). Probe shapes in v1 were misread as a special case;
   §1.3 records what the probes actually exercised.
2. **v1 claimed upstream adoption requires donor tree == desired tree.**
   Wrong: upstream's own comment says restoring `@` "rewrites every file that
   differs from the creation base" (`src/workspace.rs:690-696` U), and its test
   `add_with_cow_checks_out_exactly_the_base` (`tests/cli.rs:1064-1104` U)
   edits the donor (README modified, scratch added) before cloning and asserts
   the result matches the **base** with donor edits absent. Adoption is
   truthful about the *cloned disk*, not about the final tree; `jj restore`
   converges to base, paying writes only for donor≠base paths (§4.1).
3. **v1 claimed other workspaces' operations "guarantee" staleness /
   `update-stale` after any jj-free window.** Wrong:
   `WorkingCopyFreshness::check_stale` (`lib/src/working_copy.rs:361-399` at S
   and S33) returns **Fresh** for an ancestor operation whenever the recorded
   tree ids+labels equal the current working-copy commit's tree. Staleness
   requires this workspace's wc-commit tree to have changed, not mere op-head
   advance (§2.2, §5b).
4. **v1 claimed placeholder states resolve "stat-only, no hashing".** Wrong:
   `FileState::placeholder()` records **zero mtime and zero size**
   (`lib/src/local_working_copy.rs:314-324` at S, identical S33), which can
   never satisfy `is_clean` against real metadata, so the next snapshot
   **hashes** those paths (`get_updated_tree_value` → `write_path_to_store`,
   `local_working_copy.rs:1943-1990`). Placeholders defer a hash, not avoid
   one (§2.2, §4.2, §7).
5. **v1 claimed a missing `.jj` "fails closed".** Wrong: workspace discovery
   walks ancestors and falls back to the first ancestor containing `.jj`
   (`cli/src/cli_util.rs:2895-2900` at S; `2897-2901` S33); inside a enclosing
   real repo, commands bind to that repo. Absent `.jj` is safe only with an
   explicit identity boundary (§2.3, §5b).
6. **v1's restore matrix contradicted itself** (§6 said dir-over-file fails
   `ENOTDIR`, then a step claimed a stashed dir can replace a file/symlink).
   Fixed matrix in §6, plus the "manifest" is now correctly presented as part
   of the *proposal* with crash windows — the engine today has in-memory plans,
   `operation.lock`, and bookmark-guard recovery notes, **no persistent
   manifest**.
7. **v1 let restore-over-top happen and relied on the final `verify_disk` to
   notice clobbering.** Corrected: disjointness from the frozen desired tree
   (and its ancestor paths) is a pre-restore gate (§5a, §6).
8. **v1's "`.jj` must stay in place, always" was too absolute** given §5b's
   workable stash-and-restore; and "restored metadata resumes clean" needs the
   *final disk* to match the recorded stats, not merely the metadata's own
   mtimes (§2.3, §5b).

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
  (label **(S33)**). All four load-bearing lib behaviors cited here (tree
  inheritance in `new`, `check_stale` freshness rule, placeholder contents,
  checkout write/skip paths, `find_workspace_dir`) are **identical at S and
  S33** unless noted.
- **(P)** observed live, either on real workspace state (read-only) or in
  disposable probe repos under `/tmp/opencode/jj-probe*` using the installed
  binary. Probe scope, honestly stated (this is the full extent; no probes
  were added after v1): probes 1–2 exercised `sparse set --clear` → prefill →
  `sparse reset` with default and then fork config flags; probes 3–4 attempted
  the fork's full sequence but the `jj new` step **failed or did not run** in
  both, so every probe observed reset behavior with a **full recorded tree**
  — which, per corrected source reading (§0.1), is the fork's actual
  reset-time state. No probe ever exercised a genuinely empty recorded tree;
  v1's "materializes nothing" claim did not come from a probe and was a source
  misreading.
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
| Checkout binding | `.jj/working_copy/checkout`: protobuf `Checkout{operation_id, workspace_name}` **(S** `lib/src/protos/local_working_copy.proto:69-77`, reader/writer `lib/src/local_working_copy.rs:2676-2726`**)** | `operation_id` participates in freshness (§2.2); `workspace_name` keys `View.wc_commit_ids` | yes (position-independent; freshness re-evaluated per §2.2 — an unchanged tree stays Fresh) | **no** — donor's op line and name describe the donor |
| Tree state (stat cache + tree binding) | `.jj/working_copy/tree_state`: protobuf `TreeState{tree_ids, conflict_labels, file_states(path→mtime_millis,size,file_type,conflict data), sparse_patterns, watchman_clock}` **(S** `local_working_copy.proto:49-67`; load/save `local_working_copy.rs:1107-1225`**)** | tree ids must resolve in the store; file states are mtime/size/type-bound to disk content, **not inode-bound**; whole-file paths are repo-relative | **yes, exactly**: a whole-workspace rename preserves inode, mtime, size, so every clean stat stays clean — provided the *disk content the stats describe also moved with it*; stats describe content, not metadata-file mtimes | conditionally: truthful only when the target's disk files carry the donor's stats (upstream's creation flow, §4.1); a mismatched tree is not "corrupt" but forces re-hashing and can drive wrong materialization diffs — upstream validates and rolls back |
| Sparse patterns | inside `tree_state` (no separate file in this generation; observed live **(P)**) | workspace choice, not revision | with tree_state | with tree_state (upstream relies on this) |
| Watchman clock | inside `tree_state` | the checkout that recorded it | with tree_state | meaningless across checkouts; jw refuses watchman everywhere it matters **(U** `src/jj.rs:667-679`**)(F** `src/reseed/engine.rs:800-813`**)** |
| Mutation lock | `.jj/working_copy/working_copy.lock` (flock) **(S** `local_working_copy.rs:2759-2762`**)** | transient | n/a | n/a |
| Workspace registration | shared store `workspace_store/index`: protobuf `Workspaces{name, path}`, path stored **repo-relative** when possible **(S** `lib/src/simple_workspace_store.rs:151-183`; `simple_workspace_store.proto`**)**; verified live, the installed binary also writes relative paths **(P)** | position of each workspace relative to the repo dir | the file moves trivially, but a moved workspace leaves a **stale registration**; no public command re-registers a path (`jj workspace add` requires an empty destination; `forget` deliberately leaves the path entry — `cli/src/commands/workspace/forget.rs:92-96` **(S)**) | n/a |
| WC-commit mapping | `View.wc_commit_ids: map<workspace_name, commit>` in op-store views **(S** `simple_op_store.proto:98-99`**)** | name, not path | survives workspace moves | n/a |
| Op history | `op_store/`, `op_heads/heads/` (a directory) **(P)** | append-only, shared | moves with the store | n/a |
| Colocated Git linkage | `store/git_target` (relative); for colocated secondary workspaces a `.git` gitfile plus the primary's `.git/worktrees/<name>/gitdir` back-pointer — both relative **(P)** | paths on both sides | git-side links break on move; `git worktree repair` is the native fix; upstream jj itself calls it when adding a worktree to a populated dir **(S** `lib/src/git.rs:1864-1894` `create_worktree`**)** | n/a |

### 2.2 What the snapshot, checkout, and freshness machinery actually do

- **Clean test (S** `local_working_copy.rs:1943-1956`, `is_clean` at `308-312`,
  identical S33**)**: a tracked file is trusted without hashing iff
  `file_type ∧ mtime ∧ size` match the recorded state **and** the file's mtime
  is strictly older than `tree_state`'s own mtime (the race guard). No inode or
  device component. **Consequence:** transfers that preserve mtime and size
  (reflink + explicit `utimens`, or plain rename) keep existing clean stats
  clean; they cannot create new clean stats out of nothing (see placeholders
  below).
- **Placeholders defer a hash, they do not avoid one (S**
  `local_working_copy.rs:314-324` placeholder = zero mtime/size; `2405-2407`
  skip path pushes placeholder into `changed_file_states`; identical S33 at
  `2305-2309`**)**: when checkout declines to overwrite present untracked
  content ("skipped … conflicting changes", observed **(P)**), the recorded
  state is a placeholder, which can never be `is_clean` against real metadata;
  the next snapshot over that path therefore **reads and hashes the file**
  (`get_updated_tree_value` → `write_path_to_store`, `1943-1990`). If the
  hashed value equals the tree's value, no change is recorded and the *real*
  stats replace the placeholder — subsequent snapshots are stat-only. If it
  differs, the difference enters the working-copy commit as an edit (the
  fork's `verify_disk` turns exactly that into a hard failure).
- **Deletion semantics (S** `local_working_copy.rs:1903-1933`
  `emit_deleted_files`**)**: snapshot records a deletion for every recorded
  (tracked) path absent on disk. Untracked/ignored paths have no file states,
  so **their absence is invisible to snapshots**. This is snapshot-safety
  only: a command that *materializes* (e.g. `jj sparse reset`, any checkout)
  may create a tracked path right where an absent directory used to live.
- **Checkout writes are unconditional in the lib (S**
  `local_working_copy.rs:2458` — `// TODO: Check that the file has not changed
  before overwriting/removing it`, same at S33:2361) — **but** the *diff driving
  them* is tree-vs-tree, and paths present on disk as **untracked** content are
  skipped with placeholders **(P)**: prefilled files kept byte-identical
  content and a 2020 mtime across `jj sparse reset`; only missing files were
  written with current mtimes.
- **Freshness/staleness (S** `lib/src/working_copy.rs:361-399`
  `WorkingCopyFreshness::check_stale`, identical S33**)**: if the recorded
  `operation_id` is an ancestor of the op head, jj compares the **recorded tree
  ids+labels against the current wc commit's tree**; equal → **Fresh** (no
  reload, no update-stale). `WorkingCopyStale` arises only when this
  workspace's wc-commit *tree* changed between the recorded op and now — e.g.
  the workspace's own commit was rewritten elsewhere — not from other
  workspaces' routine operations. (A snapshot or checkout that rewrites @ also
  advances the recorded op id, restoring Freshness.) So: an untouched
  workspace with matching disk stays Fresh indefinitely across repo activity;
  conversely, restoring old metadata whose recorded tree no longer matches the
  current wc-commit tree *does* require the stale path
  (`jj workspace update-stale`, a supported recovery).

### 2.3 Answers to "what can move out and return unchanged"

- **Entire ignored/untracked subtrees** (dependency caches, build outputs):
  invisible to snapshots while absent (§2.2 deletion semantics); rename out
  and back preserves everything jj never recorded. Constraints: same
  filesystem (`EXDEV` otherwise), same mount topology the fork already guards
  **(F** `src/reseed/topology.rs`**)**, no *tracked* paths inside or at the
  restore destination (§6 pre-restore gate), and awareness that materializing
  commands may legitimately create tracked paths in the vacated location while
  the root is away — restore order and disjointness, not snapshot-invisibility,
  are what make this safe.
- **The workspace's own `.jj`**: the *recommendation* is to keep it in place
  (every jj command in the directory needs it, and see the discovery hazard
  below), but it is *physically* possible to stash and restore the identical
  `.jj` (position-independent files, rename preserves the stat cache the same
  way), with two hard caveats: (1) while `.jj` is absent, jj commands run in
  the directory do **not** reliably fail — discovery walks ancestors
  (`cli/src/cli_util.rs:2895-2900` S, `2897-2901` S33) and will bind to an
  enclosing repo if one exists (we hit exactly this in a recent test with a
  nested TMPDIR); the window therefore requires "no jj commands" *plus* an
  explicit identity boundary (e.g. verified absence of an ancestor `.jj`), not
  a promise of fail-closed behavior; (2) the restored stat cache is only
  *correct* if the final disk content still matches what it describes — if the
  operation changed disk content while `.jj` was away, the first snapshot
  re-hashes mismatches (self-healing, with a potentially large diff), and if
  the wc-commit tree changed, freshness requires the update-stale path (§2.2).
- **From a donor, unsupportable**: `checkout` (op-line + name describe the
  donor; copying it would claim a tree and history the target did not build).
  `tree_state` from a donor is supportable **as upstream does it** — truthful
  about cloned disk, validated by a real jj command, rolled back on rejection,
  then converged to the desired tree by an explicit rewrite (§4.1) — not as a
  blind copy; v1's blanket "only when donor tree == desired tree" was wrong
  (§0.2).

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
  in-sparse recorded paths (and hashes the dirty ones); every checkout walks
  the full tree diff it is applying. The only traversal-free *stat* change is
  the private-layout `tree_state` transplant, which upstream jw does at
  creation with validation and rollback (§4.1).
- `jj restore` with no arguments restores the working copy from its parent(s)
  **(B help)** — the convergence step of upstream's adoption flow.
- `jj sparse set --clear` deletes tracked files from disk; `jj sparse reset`
  re-includes everything, writing paths that are absent, skipping present
  untracked paths with placeholders **(P)**.

## 4. Upstream jw vs our fork

### 4.1 Creation (`jw add`, upstream `cd7bb1eb`) — adoption with donor≠base support

**(U** `src/workspace.rs:544-732`, `src/jj.rs:648-731`, `src/cow.rs`,
`tests/cli.rs:1064-1104`**)**:

1. Resolve the full base commit id before any mutation; `jj workspace add
   --name N --revision <base> --sparse-patterns empty [--colocate|--no-colocate]
   <path>`. The new workspace's @ is created with the **base tree**
   (`cli/src/commands/workspace/add.rs:254-256` S: `new_commit(parent_ids,
   merge_commit_trees(parents))` — same inheritance rule as `jj new`), but
   materializes nothing because the sparse selection is empty.
2. `cow::clone_files` reflinks (FICLONE/clonefile) every file the **source's
   working-copy commit tracks** — i.e. the donor's *current* tree S, including
   its uncommitted edits — then restores the source's mtime onto each new
   inode. Ignored/untracked donor content (`.env`, `node_modules` in the test)
   is never cloned.
3. `jj::copy_tree_state` copies the source's `tree_state` into the target:
   tree ids of S, file stats matching the clones, sparse patterns. Private
   layout, validated by a real `jj status`; on rejection the previous state is
   restored and creation falls back to an ordinary full add
   (`src/workspace.rs:630-643`). The `checkout` file is **not** copied — the
   target keeps its own name and op line.
4. `jj restore` (no args = restore @ from parent(s) = base). The adoption was
   truthful about the cloned disk, so this step's checkout diff is **S vs
   base**: paths equal in both keep their clones untouched (no write, stats
   stay clean); paths where the donor differed are rewritten from the store;
   donor-only paths are deleted. Upstream's own words: "Restoring `@` then
   rewrites every file that differs from the creation base, so the result
   matches a full checkout" (`src/workspace.rs:690-696`), and the test
   (`add_with_cow_checks_out_exactly_the_base`) proves it against a
   deliberately edited donor: result has the base README, no scratch file, and
   an empty `jj diff`.
5. Provenance (`operation_id`), optional bookmark + `.jj/jw-bookmark`, Git
   worktree ownership verification for colocate **(U** `src/ownership.rs`**)**,
   rollback of partial creations. Watchman donors are refused outright.

Net cost shape: one reflink per donor-tracked file + one stat pass + store
writes proportional to the donor↔base **divergence**. No full-content write
pass, no hashing of paths where S==base (their stats are real and match). The
v1 claim that this only works when S==D is retracted; what *is* required is
that the cloned disk carry the donor's stats — which the clone step guarantees.

### 4.2 Our fork's additions, with the corrected mechanics

**(F)**: the fork keeps the upstream creation flow (same `materialize_clone` /
`adopt_source_tree_state` at `src/workspace.rs:623-660`) and adds:

- **`reseed/`** (`src/reseed/engine.rs`, 2205 lines): in-place refill of
  *existing* workspaces through stock jj's sparse interface. Sequence
  (`engine.rs:1421-1566`): quiescence checks (fsmonitor none, no watchman,
  `800-813`); sparse-list must already be `.` (`815-825`); tracked-edit
  verification against a frozen operation (`verify_disk`, `1024-1051`);
  bookmark batch proofs; `jj sparse set --clear` (deletes tracked files,
  empties file states); manual removal of recorded old∪desired paths;
  `jj new <base>` — which records the **full base tree** (§0.1) and is proven
  to be an exact *empty-change* child (`exact_empty_child`, `1415-1419`);
  optional `replace_content` purge; `cow::clone_files` prefill of desired
  tracked paths; `jj sparse reset`; ignored-roots mirror via diff-dirs `sync`
  `new_replacing_checked(ReflinkRequired)`; links application; final
  identity/topology/empty-child re-verifications. Plus: repository-scoped
  operation lock (`jj-waltz/reseed/operation.lock`), device+inode identity
  pinning, frozen ignore oracle (`cow/payload.rs`), selection groups
  (`selection.rs`), symlink support proofs via native git diff, out-of-band
  recovery policy (never automatic repair).
- **`freshen/`**: pin one upstream identity (bookmark/commit) before fetch.
- **`lifecycle/ignored.rs`, `cow/ignored.rs`**: copy_ignored selection and
  policy plumbing.

Corrected mechanics of the tracked path through reseed (this replaces v1's
false "reset materializes nothing" account):

1. After `sparse set --clear`, file states are empty and the recorded tree is
   still the old @'s; `jj new <base>` then **sets the recorded tree to the
   full base tree** (tree inherited; the empty-pattern matcher suppresses disk
   writes during that checkout).
2. `clone_files` prefills desired paths (untracked; mtimes preserved).
3. `sparse reset` runs `set_sparse_patterns([root])`, whose add-side update is
   empty-tree → base-tree over **all** paths **(S**
   `local_working_copy.rs:2299-2325`**)**: prefilled paths exist as untracked
   content → **skipped with placeholder states**; missing paths → written from
   the store with real stats **(P** probes, §1.3**)**. (Note the probes
   demonstrated exactly this recorded-full-tree shape, by accident: their
   `jj new` step never ran.)
4. The engine's `verify_disk` (`jj diff --from @ --to <frozen base>` with
   `snapshot.auto-track=none()`, live working copy) **snapshots the recorded
   paths**: each placeholder is dirty by construction (zero stats) and is
   **hashed once**; equal content → no diff, real stats recorded; divergent
   content → non-empty diff → "tracked tree differs from frozen desired tree"
   hard failure. After this snapshot, all stats are real and subsequent
   snapshots (the engine's later `verify_disk` calls and the user's ordinary
   commands) are stat-only. @ remains the exact empty-change child of base.

So the fork's tracked-side cost is: one delete pass (clear), one reflink pass
(prefill), store writes only for paths missing from the prefill, and **one
content-hash pass over prefilled paths inside the engine's own verification**
— not, as v1 claimed, a hash pass deferred to "the first ordinary command"
(that claim assumed an empty-tree @ that never exists). An open question the
engine's own comments do not settle (untraced by me, stated as such): whether
the placeholder-hash pass and the `jj restore`-style convergence could be
combined or elided in some ordering — that is a design question, not an
observation.

### 4.3 What donor-state reuse buys, correctly stated

- At **creation** (upstream): truthful adoption makes S==base paths write-free
  *and* hash-free immediately; divergence (S≠base) is paid as store writes at
  `jj restore`. Works for arbitrary donor working-copy states.
- At **in-place reseed** (fork): there is no fresh workspace to adopt into;
  the target's own `tree_state` survives (clear empties file states, not the
  tree binding, and `jj new` rebinds the tree). The fork achieves write-avoidance
  for prefilled paths via the skip-with-placeholder mechanism, paying one hash
  pass in verification. A **conditional** transplant of a donor's `tree_state`
  (donor disk cloned stat-identically, then converge @ to the desired tree
  with a restore-style rewrite) would eliminate that hash pass for S==D paths —
  at the cost of importing upstream's validate-and-rollback discipline into
  reseed, an extra @ rewrite operation, and private-layout dependence that
  must re-verify the installed jj generation. Whether that trade is wanted is
  a human decision (§8); v1's framing ("donor==desired or false state") was
  wrong in both directions.

## 5. Move-aside / restore algorithms, concretely

The operator's framing — "move some crucial directories out of the way before
our operation, then move them back-over-top" — evaluated as four distinct
algorithms. "Crucial directories" classified per §2: ignored/untracked roots
and dependency caches are the movable class; native metadata (`.jj`, Git
linkage) is the keep-in-place recommendation; watchman state is refused
already.

### (a) Keep `.jj` in place; stash only untracked/preserved roots — **viable, recommended**

1. **Before (gated):** compute the frozen desired tracked set D (the engine's
   `plan.entries` already is this). For each candidate root R, require
   **upfront disjointness**: R and every ancestor path of R is not a tracked
   path in D, and R contains no tracked paths of the current tree (provable
   against the frozen sets). Only then `rename(root/R, stash/R)`. Stash is a
   **same-mount sibling** of the workspace root (e.g. `<root>.jw-stash/`),
   invisible to every jj walk, plus a manifest of moves. (The manifest is part
   of this proposal; the engine today has in-memory plans and an operation
   lock, not a persisted move manifest — see §6 crash notes.)
2. **During:** the existing reseed executes unchanged, minus the mirror work
   for stashed roots. Snapshot-safety of the absences holds by §2.2 (untracked
   paths have no file states); note this is *snapshot*-safety — the engine's
   own `sparse reset` and materializations may create tracked paths at or
   under R's location while R is away, which is exactly why the §6 pre-restore
   gate re-checks disjointness against the *final* D before restoring.
3. **After:** restore-over-top per root (§6 sequence), re-verified against
   frozen D, then the engine's existing final checks. The end state is the
   same as today's reseed plus the preserved roots.

Preserves: workspace registration, root identity, `.jj`, git linkage, the
desired tracked tree. Renames are O(#roots) namespace operations.

### (b) Stash the target's own `.jj` while a filesystem transfer replaces the root's contents; restore the same metadata — **physically workable, strictly dominated, and not fail-closed**

Lawful only under two hard conditions: **zero jj commands** run while `.jj` is
absent — and since discovery falls back to an **enclosing** repo's `.jj` when
one exists (`cli_util.rs:2895-2900` S / `2897-2901` S33; we observed this
mis-binding in a recent nested-TMPDIR test), the window needs an explicit
identity boundary, not trust in a clean error — and the restored metadata must
describe the **final** disk (stats that no longer match content are re-hashed
on the next snapshot, and if the wc-commit tree changed, freshness requires
`update-stale`, §2.2; an unchanged tree stays Fresh with no update-stale at
all — v1's "guaranteed update-stale" was wrong). Per-path renames never need
the root vacated, so (b) buys nothing over (a) except risk.

### (c) Copy donor working-copy state into the target — **upstream-shipped at creation; a conditional alternative for reseed, not a default**

This is §4.1: truthful about cloned disk, validated, rolled back, converged by
`jj restore`. For reseed it would be a deliberate extension with the costs in
§4.3 — not something to recommend by default, and never the `checkout` file.

### (d) Stage a whole replacement workspace root elsewhere; swap roots — **lawful but disk-hostile**

Swapping whole roots by rename is O(1) and keeps the final path (hence
registration) intact, but: staging a full parallel tree is exactly the disk
doubling that hurt at 99% utilization **(F** `primary0.sol61m.md`**)**; the
staged `.jj/repo` pointer and Git worktree gitfile are path-bound both ways
**(P** §1**)**, so a staged-then-swapped root needs `.jj` pointer repair and
`git worktree repair` afterwards; and an interrupted swap leaves two candidate
roots. Dominated by (a) for every stated goal.

### Which jj commands may run while paths are absent

- **Tracked** paths absent: any snapshot-capable command records deletions
  (§2.2). Never outside the reseed's guarded window.
- **Untracked/ignored** paths absent: snapshot-safe, but not *generally* safe —
  materializing commands (sparse expansion, checkout) may create tracked paths
  in the vacated location; restore must therefore be gated (§6), not assumed.
- `.jj` absent: no reliable failure — ancestor discovery may bind elsewhere
  (§5b); treat as "no jj commands, with an identity boundary".

## 6. "Move back-over-top" is not a filesystem union

`rename(2)` cases that matter (Linux; `std::fs::rename` = same semantics):

- old is a **file or symlink**: replaces an existing file/symlink target
  atomically; fails against any directory target (`EISDIR`/`ENOTDIR`).
- old is a **directory**: succeeds if the target is absent, or is an **empty**
  directory (replaced); fails `ENOTEMPTY` against a non-empty directory, and
  `ENOTDIR` against a file/symlink target. (v1's step claiming a stashed
  directory could replace a file/symlink contradicted this; corrected.)

So restore-over-top must define, per top-level stash entry, one of
**preserve** (result wins; drop stash entry), **replace** (stash wins; displace
result's entry first), or **merge** (recurse; only as deep as policy is
defined). Restoring the *whole old root* instead of reconciling would discard
the operation's tracked result — never that.

Smallest lawful sequence per root R, policy **replace**, with the pre-restore
gate applied first:

0. **Gate:** R is disjoint from the frozen desired tracked set D and from all
   ancestors of D's paths (re-checked against the *final* tree, before any
   restore rename — the final `verify_disk` is a safety net, not permission to
   clobber and notice afterwards).
1. If `root/R` absent → `rename(stash/R, root/R)`; done.
2. If `root/R` is an **empty** directory → `rename` replaces it; done.
3. If `root/R` is a **file or symlink**: a stashed *file/symlink* R renames
   over it atomically; a stashed *directory* R cannot (type conflict) —
   displace the result's entry (move it into the stash area or remove it per
   policy), then rename the stash entry in. Displacement is *chosen*, never
   implied.
4. If `root/R` is a **non-empty directory** (the operation produced content
   there — mirrored payload, or new tracked tree): per policy, swap (move
   result's entry to `stash/R.new`, then rename stash entry in — both O(1),
   both versions discoverable) or refuse. No implicit union.

Edge conditions stated, not implied: parent-path/type conflicts (stashed dir
`pkg/cache` under a result where `pkg` is now a file — the engine's
type-transition preflights already model this class and the gate in step 0
catches the tracked cases); symlinks rename as links (never followed); modes
are preserved by rename; same-filesystem only (`EXDEV` must fail loudly,
mirroring the ReflinkRequired discipline); mount boundaries and autofs/aliasing
per the existing topology guard; concurrent readers of a stashed root see it
vanish (quiescence is already a reseed precondition — the window includes the
stash moves). **Crash/interruption:** this is a proposal, and its windows must
be accounted: (i) after a root is stashed but before its manifest entry is
recorded — recoverable because the stash location is deterministic and the
workspace root is missing an entry the plan knows about; (ii) mid-restore —
the displaced `stash/R.new` and the manifest distinguish result content from
preserved content; (iii) manifest write failure — treat unmanifested stash
entries as suspect and stop. Recovery is deliberately out-of-band (matching
the fork's stated "never automatic repair" policy); the existing
`operation.lock` and bookmark-guard recovery notes cover the jj side, and the
proposed manifest covers the move side. No journal framework is needed or
proposed.

A move is O(1) namespace work per entry. It does not by itself prove jj state
correct (that is what the engine's final verifications are for), and it does
not make later snapshots free (clean stats were already stat-only; stashed
roots were never tracked).

## 7. Performance comparison — what actually disappears

Against the delivered streaming reseed **(F** `streaming0.sol61m.md`**)**,
for a root that exists in the target and is pure-ignored:

| Approach | Walks | Copies/writes | Hashing | Disk pressure | Notes |
| --- | --- | --- | --- | --- | --- |
| Current mirror (`copy_ignored` → diff-dirs sync, ReflinkRequired) | both trees, divergent entries | reflink per differing file; delete extras | none (jj never hashes ignored content) | ~divergence-sized new extents | donor→target sync semantics |
| **Preserve-in-place** (skip the root; no moves) | none | none | none | none | zero-cost when the target's copy is already the wanted one; "exclusion during streaming" as policy |
| Move-aside + restore-over-top | none during op | O(1) renames ×2 per root + collision handling | none | none (namespace only) | only for roots the operation would destroy/replace (purge, type transitions) |
| Let jj materialize (no CoW) | full tree diff | full write pass (unconditional per §2.2) | none during checkout; stat-only after | full content duplication | the disk-pressure failure mode already observed |

Tracked-side costs are invariant across these rows (they address ignored
roots only): today's fork pays clear-deletes + reflink prefill + store writes
for unprefilled paths + **one hash pass over prefilled paths at `verify_disk`**
(§4.2); a conditional adoption variant (§4.3) could remove that hash pass for
donor==desired paths at the cost of a tree_state transplant discipline. No
timing claims; the cancelled campaign's fixture overhead is not re-litigated.

## 8. Recommendation and human decisions

**Recommended viable approach:** treat "preserved roots" as a policy sibling
of `copy_ignored` — *preserve-in-place* by default for ignored roots the
operation has no reason to touch (zero moves, zero cost), and
*move-aside/restore-over-top* (algorithm (a) + §6 sequence, gated by upfront
frozen-D disjointness) only for roots that must be vacated (explicit
`replace_content`, or tracked-type transitions at/under those paths). Static
per-root decision at plan-freeze time; no rule interpreter, no new runtime, no
preflight accumulation. The move manifest is new (proposal), sized to the
sequences in §6, not a journal.

Decisions that are human's, not mine:

1. **What is being preserved:** the target's *own* copy of an ignored root
   (preserve-in-place / move-aside), versus donor authority (current mirror),
   versus both by region. The prior authority-based design (`design0.gpt6a.md`)
   deliberately left the default unchosen.
2. **Tracked-side verification cost vs adoption:** the fork's current flow
   includes one content-hash pass over prefilled tracked paths inside
   `verify_disk` (§4.2). A conditional donor-`tree_state` transplant for
   reseeds pinned to a donor-checked-out revision (S==D) would make those
   paths hash-free, importing upstream's validate-and-rollback discipline and
   a restore-style convergence step into reseed. Real trade, not automatically
   recommended (v1 neither described the current cost correctly nor evaluated
   this fairly).
3. **Collision policy on restore** (§6): replace-with-swap, preserve-result,
   or refuse-and-report when the operation produced content at a preserved
   root's path. (My recommendation: refuse; keep the sequence static and
   reviewable.)
4. Whether stash "merge" is ever offered at all (recommendation: no).

## 9. Unsupported, unknown, and not claimed

- No benchmarks; cost statements are structural (walks, writes, renames,
  hashes) from traced source semantics plus recorded campaign facts. Probe
  scope is exactly §1.3 and predates this correction; no probes were added.
- Traced at both S and S33: `new` tree inheritance, `check_stale`
  tree-equality freshness, placeholder zero-stats and their skip-path
  recording, unconditional-write TODO, `find_workspace_dir` ancestor fallback,
  `set_sparse_patterns` add/remove updates. **Not traced end-to-end**: the
  exact CLI snapshot ordering inside the installed binary's `sparse reset`
  (which snapshot observes which placeholder when, relative to pattern
  application); the §4.2 account of the fork's flow is source-derived from
  the traced pieces plus the fork's own code and probe observations of the
  reset-time shape, and the one open ordering question is flagged in §4.2.
- Upstream test evidence covers creation-time adoption with donor edits
  (`tests/cli.rs:1064-1104` U); no equivalent upstream test covers an in-place
  reseed adoption variant, because none exists — §4.3 is a design option, not
  a shipped behavior.
- Installed binary lacks `jj workspace remove` (source-only); relative
  workspace-store registration paths verified live on the installed binary.
- Nothing here touches the user's uncommitted
  `design/up-performance/handoff0.glm53m.md`; the upstream research workspace
  remains pristine and unmodified.

## 10. Connections to prior design

- `recovery0.gpt6a.md` commissioned this question set after the campaign
  cancellation; this report supplies the jj-state and move-lawfulness halves.
  `design0.gpt6a.md`'s authority/exclusion vocabulary is the right frame for
  §8: preserve-in-place is "target authority for this region"; the current
  mirror is donor authority; move-aside is target authority with a temporary
  absence.
- `streaming0.sol61m.md` delivered the mirror machinery this report compares
  against; `primary0.sol61m.md`'s disk-pressure record is the empirical reason
  algorithm (d) is rejected and preserve-in-place leads.
