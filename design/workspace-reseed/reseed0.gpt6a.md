---
type: Design
title: Repeatable in-place reflink reseeding with stock JJ
description: A batch jw reseed workflow that keeps workspace roots and JJ identity, shares a prepared source tree, and needs no JJ adoption fork.
resource: /design/workspace-reseed/reseed0.gpt6a.md
status: proposed
generated:
  by: model:openai/gpt-6-astra
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec38a6d5fffeiVzJMUoRCYm9YM"
    title: User request, implementation inspection, successful local reflink experiments, and implementation handoff
  - resource: "https://github.com/EzraCerpac/jj-waltz/blob/cd7bb1eb6451b09c06853b0347f856b4203f7c89/README.md#copy-on-write-workspaces"
    title: Existing creation-only CoW behavior, inspected in the local checkout
  - resource: "https://github.com/EzraCerpac/jj-waltz/blob/cd7bb1eb6451b09c06853b0347f856b4203f7c89/src/workspace.rs"
    title: materialize_clone and adopt_source_tree_state
  - resource: "https://github.com/EzraCerpac/jj-waltz/blob/cd7bb1eb6451b09c06853b0347f856b4203f7c89/src/jj.rs"
    title: Tracked-file listing and private tree_state copying
  - resource: "https://github.com/EzraCerpac/jj-waltz/blob/cd7bb1eb6451b09c06853b0347f856b4203f7c89/src/cow.rs"
    title: Existing filesystem clone implementation and capability probe
  - resource: "https://github.com/EzraCerpac/jj-waltz/blob/cd7bb1eb6451b09c06853b0347f856b4203f7c89/CONTEXT.md"
    title: Refresh, creation-base, and ownership contracts
  - resource: "https://github.com/EzraCerpac/jj-waltz/blob/cd7bb1eb6451b09c06853b0347f856b4203f7c89/docs/architecture.md"
    title: Lifecycle, adapter, and repository-scoped metadata boundaries
  - resource: "file:///home/rektide/src/jj-reflink/cli/src/commands/workspace/add.rs"
    title: Local adoption patch at commit 8f05f259; initial adoption rather than repeated refresh
  - resource: "file:///home/rektide/src/jj-reflink/lib/src/local_working_copy.rs"
    title: Sparse materialization leaves existing paths in place with placeholder file state
verification:
  date: 2026-10-14
  scope: Disposable Linux Btrfs experiment using installed jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b; its workspace add has no --adopt option
  result: Two in-place keep-revision passes and an explicit new-base pass succeeded; filefrag confirmed shared extents
  limits: Not a production implementation, release compatibility matrix, or throughput benchmark
---

# Repeatable in-place reflink reseeding

## Recommendation

Implement **`jw reseed` in jj-waltz**, using ordinary JJ sparse commands and
filesystem reflinks. Keep each target's root directory, `.jj`, registration,
workspace name, and lifecycle metadata. Refill its tracked tree from a prepared
source plus JJ's checkout of the differences. Copy selected ignored directories
as private CoW data when configured.

**No `workspace add --adopt`, forget/re-add cycle, private `tree_state` copying,
or maintained JJ fork is required.** The essential mechanism was exercised
successfully on this machine, including actual physical extent sharing.

The user-facing payoff is one repeatable command for a named batch. The operator
prepares `~/a/a/opencode` at `dev@origin` once, then distributes its filesystem
contents to many existing workspaces. Source preparation and network fetch remain
ordinary JJ/package-manager operations.

This is an implementation proposal. The commands beginning `jw reseed` below
do not exist yet.

## What the existing tools actually do

| Tool | Available behavior | Missing for this use |
| --- | --- | --- |
| Local JJ patch | `workspace add --adopt` registers an existing directory without `.jj`, snapshots it, and restores it to the requested parents' tree | Existing workspace names are rejected; no repeated batch refresh or ignored-content policy |
| `jw add/switch --cow` | Creates a sparse-empty workspace, clones the current workspace's tracked files, copies its private `tree_state`, snapshots, then restores from the new workspace's parent | Creation only; no arbitrary prepared source; excludes ignored/untracked data; private-state and Watchman compatibility paths |
| `jw adopt` | Records lifecycle metadata for an existing JJ workspace | Does not adopt filesystem contents or change JJ state |

Today's waltz CoW path already avoids the JJ fork, but it does not satisfy the
repeated-use request. Its private-state fast path also need not become a
requirement of the new command.

## Operator flow

### Prepare the source

Use a source tree dedicated to the baseline and its installed dependencies:

```sh
jj -R ~/a/a/opencode git fetch --remote origin
jj -R ~/a/a/opencode new dev@origin
cd ~/a/a/opencode
bun install --frozen-lockfile
```

`jj new` preserves previous work in history while selecting a new empty child.
These are explicit operator actions, not implicit effects of reseeding. The
source is ready when its **tracked tree equals the resolved `dev@origin` tree**;
its `@` commit need not be the remote commit itself.

Observed during this session: this path resolves to
`/home/rektide/archive/anomalyco/opencode`; its recorded `@` is nonempty and is
not `dev@origin`. Nothing was changed there. Do not assume the current source
already satisfies the baseline contract.

### First use: explicit inputs

```sh
# Re-share files while keeping each feature workspace's current revision.
jw reseed ocw1 ocw2 ocw3 --from ~/a/a/opencode --source-at dev@origin

# Reset reusable worker slots onto a fresh empty child of dev@origin.
jw reseed ocw1 ocw2 ocw3 --from ~/a/a/opencode \
  --source-at dev@origin --at dev@origin
```

`--at` means **start a fresh working-copy change there**, using `jj new` while
sparse-empty. It does not rebase a feature stack or move its bookmarks. Without
`--at`, reseeding preserves the snapshotted target revision, including its own
feature differences. Updating a feature stack onto the new upstream remains an
explicit JJ rebase; reseed can run afterward to restore sharing.

The default therefore also gives a useful one-time conversion of existing
workspaces into CoW copies without disturbing their graph positions.

### Repeated use: save the boring inputs

Read an optional **`.jwreseed.local.toml` in the default workspace**, following
the existing default-workspace/local-file convention for links. Keep this
machine-specific file ignored. Initial schema:

```toml
source = "/home/rektide/a/a/opencode"
source_at = "dev@origin"
copy_ignored = ["node_modules", "packages/*/node_modules"]

[groups]
workers = ["ocw1", "ocw2", "ocw3"]
```

```sh
# After advancing/preparing the baseline again:
jw reseed --group workers --at dev@origin

# Or keep feature revisions and just replenish sharing/dependencies:
jw reseed --group workers

# Explain the selected source, targets, revisions, and ignored replacements.
jw reseed --group workers --at dev@origin --dry-run
```

No interactive confirmation is needed for ordinary configured reseeding.
Positional names and named groups select literal JJ workspace names; validate
and deduplicate them before mutation. Never select every workspace implicitly.
Command-line source arguments override file values. Repeatable `--copy-ignored`
arguments replace the configured list; `--no-copy-ignored` disables it for a run.
Keep `--at` explicit: configuration should not quietly turn a keep-revision
operation into a reset operation.

## Content contract

1. **Always retain the target root inode and its `.jj`.** Never remove or replace
   the root with a renamed staging directory or subvolume. The primary store and
   prepared source are not eligible targets. Working inside the target root is
   allowed; child-directory identities are not preserved.
2. Tracked files end up exactly matching the intended target revision and its
   saved sparse patterns. Identical paths are reflinked from the source; differing
   paths are materialized by JJ. File type and executable-bit differences count.
3. By default, retain other ignored/untracked contents. Replace only configured
   `copy_ignored` roots with fresh CoW copies; replacing means obsolete entries
   under those roots disappear too. This is explicit replacement of named data,
   not opportunistic cleanup of the repository.
4. Offer **`--replace-content`** for the user's wipe-and-refill case: after
   snapshotting tracked work, clear all remaining target contents except `.jj`
   (and `.git` once colocation is supported), then refill tracked files and the
   configured ignored roots. The flag explicitly authorizes losing unrecorded
   files, without adding a second prompt. It does not mean “copy every source
   dotfile.” The root directory still survives.
5. Never copy source `.jj` or `.git`, including nested repository administrative
   entries encountered in copied directories. Clone symlinks as symlinks, never
   traverse them. Reject unsupported special entries or nested workspaces before
   clearing the target. Cross-filesystem mounted children need explicit handling;
   the first implementation should reject them.
6. Ignored selections must be ignored in both source and target and must not
   overlap either revision's tracked paths or administrative paths. Match globs
   against source directories, with missing matches reported as skipped. Clone
   regular files with separate inodes; preserve executable modes and symlink
   targets. Never substitute hard links or shared-directory symlinks.
7. A copied dependency directory is an installed-source snapshot, not proof it
   matches every feature's package manifests. If those differ, run the project's
   frozen install in that target. Relative package-manager links usually survive;
   absolute links back into the source must be reported before copying as a
   self-contained dependency tree. Package-manager rewriting stays outside jw.

First implementation: **JJ-only target workspaces**, matching the current CoW
restriction. A colocated source is fine because its metadata is not copied.
Preserving a target `.git` inode alone is not sufficient to prove its Git index
and ownership stay correct; add colocated targets after native-JJ integration
tests. Do not convert existing targets implicitly.

## Public-CLI materialization algorithm

### Freeze and plan once

- Require quiescent source and targets while reseeding; pause agents/builds that
  write them. A jw advisory operation lock serializes jw callers, not arbitrary
  editors or independent JJ processes. Do not claim process-wide isolation.
- Snapshot source and selected targets normally **before** changing sparsity.
  Resolve stale workspaces explicitly before retrying rather than guessing their
  intended revision. Capture one final operation and exact commit IDs.
- Resolve `source_at` and optional `--at` once for the entire batch. Validate the
  source tracked tree against `source_at` after snapshot, ignoring extra ignored
  content. Require a full, conflict-free source in the first implementation.
- Save each target's exact WC commit and sparse patterns. Reject conflicted
  targets initially; normal sparse checkout can materialize conflicts but this
  optimization needs dedicated coverage. Preserve creation-base history and
  associated bookmarks; reseeding is not metadata adoption or repair.
- Probe reflinking from the actual source filesystem to every target filesystem
  before clearing any contents. Reseed promises CoW: unavailable cloning is an
  error before mutation, not a silent ordinary-copy fallback.

For source revision `S` and desired target tree `T`, compute clone candidates:

```text
source tracked paths that exist in the source on disk
  intersect target sparse selection
  minus every path changed between S and T
```

Tree comparison, not ancestry, determines equality. Exclude **both ends** of
renames/copies, type transitions, mode changes, and removed paths. A source path
absent at T must never be copied, even if T's ignore rules would hide it.
Use JSON-escaped or NUL-delimited paths, not line-splitting human `--summary`.
The installed JJ supports:

```sh
jj --ignore-working-copy diff --from "$S" --to "$T" \
  -T 'json(source.path()) ++ "\n" ++ json(target.path()) ++ "\n"'
jj --ignore-working-copy file list -r "$S" -T 'json(path) ++ "\n"'
```

Use frozen-operation queries in the implementation. Verify these capabilities
against the supported release matrix rather than assuming every supported JJ
has identical template methods. A capability gap should be explicit.

### Execute each target

```text
record pending intent outside the replaceable contents
jj sparse set --clear                # removes tracked files; revision survives
if --at: jj new <frozen base ID>      # sparse-empty, so no full checkout
clear selected ignored roots (or all non-administrative contents if requested)
reflink only the clone candidates into the existing target root
restore the exact saved sparse patterns using jj sparse set
snapshot and verify the intended tracked revision before copying ignored data
clone configured ignored roots; apply configured links after conflict preflight
verify again and record success
```

For a full checkout, restoring patterns is simply `jj sparse reset`.
Do not use that shortcut for a sparse target. Reuse JJ's sparse semantics rather
than approximating them with shell globs; a first slice can require full targets
until exact pattern handling is tested.

**Why this works:** sparse expansion attempts to create the desired files. For
paths already prefilled, JJ leaves the existing inode in place and records
placeholder file state. Its next snapshot reads those files. Because only
tree-identical paths were copied, the snapshot changes no tracked content.
Missing paths are written normally by JJ, including feature modifications.
There is no transient “baseline replaces the feature commit” operation.

JJ currently emits a generic “updates were skipped because there were conflicting
changes” warning for prefilled paths. That is not proof of success or failure.
Capture diagnostics, verify the exact resulting revision/tree, and explain
verified prefill in jw's success output. Do not blanket-suppress stderr or treat
a successful `status` exit code as tree equality. Unexpected obstruction remains
an error with the original diagnostics.

The snapshot may hash all prefilled tracked files. This is deliberately simpler
than importing private stat-cache state. It still avoids writing a full checkout
before reflinking, and ignored dependencies avoid JJ hashing. Benchmark before
claiming a speedup; per-file cloning and deletion are not O(1) subvolume snapshots.
Implement cloning in-process through `cow`, not one `cp` process per file.

### Verification, interruption, and repeatability

- Keep-revision success requires the target's post-snapshot commit ID to equal
  the captured WC ID, with matching root identity and sparse patterns. For
  `--at`, capture the fresh child's ID before materialization and require it
  unchanged afterward, empty with the exact frozen parent. Check ignored copy
  failures independently; a correct tracked tree alone is not complete success.
- Retain a small per-target pending record in repository-scoped jw storage:
  source identity/revision, old WC ID, desired WC ID, saved sparsity, selected
  ignored roots, root identity, and completed step. Do not put it in a directory
  `--replace-content` will erase. Store a separate last-reseed receipt; don't
  rewrite historical creation metadata.
- After interruption, **`jw reseed --resume NAME...`** uses the pending record
  before ordinary target snapshotting. Revalidate source and target identity,
  clear incomplete prefill while still sparse-empty, and finish the recorded
  operation. Never snapshot a half-filled tree as a new user change. If source
  identity/content changed, retain the pending record and explain that the frozen
  source must be restored or recovery must materialize from JJ without CoW.
  Recovery after sparse expansion must validate against the recorded desired
  tree before snapshotting; remove/refill planned paths or return to sparse-empty
  without recording partial content. Prove that path with fault-injection tests.
- Do not run repository-wide `jj undo` for batch rollback: other workspaces may
  have advanced. Independent targets may complete after another fails; exit
  nonzero for partial failure and list succeeded, failed, and pending targets.
- Repeating a completed reseed re-establishes sharing, even if the logical tree
  is unchanged. Keep mode leaves graph state alone; `--at` need not create a new
  child when the target is already an empty single-parent child of that exact
  base. Physical content can have lost sharing despite equal bytes.
- Foreground calls can request a full scan with `fsmonitor.backend=none`; no
  Watchman clock is copied. This alone does not stop a background snapshot
  trigger. Such writers must be paused or rejected before sparse mutation until
  targeted monitor/trigger tests establish a supported path.

## Implementation placement and delivery

Keep the public command thin. Put reseed planning/execution in a cohesive
`reseed` module, with lifecycle coordinating configuration, links, and metadata.
Use `JjClient` for all JJ invocations, `cow` for filesystem cloning, and existing
workspace inventory/ownership checks for identity. Extend `cow` to clone named
ignored directory trees and preserve modes explicitly. Validate source-to-target
capability; the current same-parent probe alone is not a cross-subvolume proof.

Deliver a useful first slice: explicit source, named batch, full JJ-only targets,
keep/`--at` behavior, ignored directory replacement, root preservation, dry-run,
verification, and resumable failure handling. Add the small local configuration
and groups in the same usable feature delivery. Exact sparse targets and
colocated targets can follow with their checks; don't make a new fleet manager,
automatic rebaser, background refresh daemon, or garbage collector a prerequisite.

Later, the same public-CLI prefill backend could serve `jw add --cow`, replacing
private-state copying. Native adoption can become an optional optimization if it
lands upstream. Neither is a dependency for repeatable reseeding.

## Evidence and acceptance checks

### Observed here

Scratch harness:
`/home/rektide/archive/EzraCerpac/jj-waltz/.test-agent/reseed/probe.py`.
Successful fixture: `.test-agent/reseed/run-2dwf3tu7/`.
These local scratch paths are not distributed tooling; the sequence and results
above are the durable evidence needed to reproduce the experiment.

On Linux Btrfs, installed JJ
`0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`:

- Two consecutive keep-revision passes preserved the exact target WC commit,
  root inode, `.jj` inode, and each newly cloned shared-file inode across sparse
  expansion and snapshot.
- Feature modifications, additions, deletion, executable-bit change, an unchanged
  executable, and an unchanged relative symlink all ended correctly.
- A sparse-empty `jj new <baseline>`, prefill, expansion, and snapshot produced
  an unchanged empty-child commit; copied ignored cache content stayed ignored.
- `/usr/sbin/filefrag -v` reported the same physical extent range
  `93326204..93326233` with `shared` for both source and target sample files.
- The tested binary does not expose `workspace add --adopt`. This verifies the
  no-adoption mechanism on that build, not every upstream release.

### Required implementation verification

Exercise actual reflinks, not just mocks or content equality:

- Batch reuse across two source revisions, old target-only files disappearing in
  `--at` mode, keep mode preserving feature changes, and repeated calls.
- Root inode and native registration unchanged; bookmarks and lifecycle creation
  fields preserved; current root permitted; primary/source target rejected.
- Source dirty/mismatched before mutation; source changes during capture; target
  stale or changed between plan and execution; aliases resolving to one path.
- Ignored dependency refresh removes old entries, leaves source isolated after
  edits in either direction, retains unrelated `.env` by default, and deletes it
  only under `--replace-content`. Changed ignore rules must not admit dependencies
  into JJ. No administrative directories enter copied payloads.
- Rename/copy both endpoints, file-directory and symlink transitions, executable
  mode, whitespace/newline filenames, and sparse patterns outside clone coverage.
- Unsupported CoW, permissions, disk-full, and interruption after every destructive
  boundary; resume never records partial files as intended history. Root remains
  present after every failure. Failure after sparse expansion is especially key.
- Real JJ release compatibility and Watchman foreground/background behavior.
  Report files/bytes cloned versus materialized and end-to-end duration on a
  representative prepared opencode tree; include ignored dependency directories.

## Cross-references

- [Existing CoW user guide](../../README.md#copy-on-write-workspaces): this proposal
  adds an existing-workspace operation and explicit source; it does not change
  what today's creation flag promises.
- [Architecture](../../docs/architecture.md#module-map): reuse the JJ adapter,
  filesystem adapter, and lifecycle boundary rather than embedding shell recipes
  into CLI parsing.
- [Domain context](../../CONTEXT.md#terms): `refresh` already means snapshotting
  working-copy facts. `reseed` deliberately names the different filesystem action;
  creation base remains historical, while a reseed receipt records new provenance.
