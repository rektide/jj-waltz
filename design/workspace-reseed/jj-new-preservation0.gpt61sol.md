---
type: Research
title: Ordinary JJ new preserves unchanged checkout files and their CoW sharing
description: Source inspection and a native Btrfs experiment establish the preservation boundary for advancing prepared donors and workspaces.
resource: /design/workspace-reseed/jj-new-preservation0.gpt61sol.md
status: investigated
generated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
sources:
  - resource: "opencode://ses_ec1feb5c1ffe6ndPdipJnsLfEF"
    title: Investigated JJ new checkout equality and verified unchanged reflinks, dirty snapshotting, same-tree transitions, exec-only replacement, and edit isolation on Btrfs
  - resource: https://github.com/jj-vcs/jj/tree/803ef00a0a6ee4ddc275d3138d5f9e7326399906
    title: Archived JJ source inspected without modification
  - resource: https://github.com/jj-vcs/jj/tree/33d3a595429b7e3b54a8ea769560247913bc8a1b
    title: Installed JJ revision; compared relevant checkout source against archive HEAD
  - resource: /design/workspace-reseed/source-freshen0.gpt61sol.md
    title: Existing source-advancement design that benefits from incremental checkout
  - resource: /design/workspace-reseed/README.md
    title: Reseed design index and current relationship to ordinary advancement
---

# Ordinary JJ new already leaves unchanged regular files alone

**Yes: ordinary `jj new <foo>` performs an incremental working-copy checkout.**
At a path whose JJ tree value is unchanged, it does not unlink, rewrite, or chmod
the regular file. Thus an existing inode, mtime, and reflink sharing survive that
checkout. No special preservation flag is required.

This is an implementation property of JJ's **local working copy**, verified at
the revisions below, not a claim about every future version or custom backend.
It is not a claim that the whole command performs no filesystem I/O: snapshotting
can read files, and repository and working-copy metadata change.

## Evidence and exact equality

Requested `~/a/jj-dev/jj` did not exist. The archive is `~/a/jj-vcs/jj`, resolving
to `/mnt/bitblaster/archive/jj-vcs/jj`, HEAD
`803ef00a0a6ee4ddc275d3138d5f9e7326399906`. All line references in this section
are to that immutable revision:

- [`cli/src/commands/new.rs:167–212, 265–284`](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/cli/src/commands/new.rs#L167-L212):
  load the workspace helper, resolve parents, merge their trees, create a commit;
  ordinary mode edits that new commit and finishes the transaction. A single
  parent supplies its tree; multiple parents can introduce merges/conflicts.
- [`cli/src/cli_util.rs:460–516, 2100–2167`](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/cli/src/cli_util.rs#L460-L516):
  normal helper acquisition snapshots local changes **before** selecting the
  destination. Those changes become the old working-copy commit's tree, rather
  than following into the new empty child of a different parent. No manual
  `jj commit` is needed to save ordinary tracked dirt before leaving it.
- [`lib/src/local_working_copy.rs:2984–2995`](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/lib/src/local_working_copy.rs#L2984-L2995):
  identical tree IDs **and conflict labels** skip tree checkout entirely,
  irrespective of commit identity, parents, or descriptions.
- [`lib/src/local_working_copy.rs:2290–2295, 2543–2579`](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/lib/src/local_working_copy.rs#L2543-L2579):
  otherwise update only the sparse-matched filesystem diff between the current
  tree and new tree.
- [`lib/src/merged_tree.rs:431–445`](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/lib/src/merged_tree.rs#L431-L445)
  filters entries through `Diff::is_changed()`, which is `before != after`
  ([`core/src/merge.rs:101–106`](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/core/src/merge.rs#L101-L106)).
  This is tree-value equality, **not comparison of disk mtimes or inodes**.
- [`core/src/backend.rs:391–410`](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/core/src/backend.rs#L391-L410):
  derived equality includes entry type; for a regular file, content `FileId`,
  executable bit, **and `CopyId`**. Identical bytes alone do not establish that
  checkout will skip it. A path move is also not same-path preservation.
- [`lib/src/local_working_copy.rs:2422–2481, 2168–2204`](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/lib/src/local_working_copy.rs#L2422-L2481):
  a changed tracked regular file is removed, then created anew and populated
  from store contents. This is not a block-level patch or reflink checkout.
  Even an executable-bit-only tree change follows this replacement path.

Snapshot's separate fast path uses file type, mtime and size, with a racy-mtime
guard; fsmonitor limits which paths snapshot examines
(`local_working_copy.rs:308–312, 1307–1325, 1936–1989`). Neither makes checkout
replace equal regular-file tree entries. Reading during snapshot can affect
atime; no atime guarantee is asserted here.

## Native Btrfs verification

Installed binary: `jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`.
Relevant `new.rs` and `merged_tree.rs` source is identical to archive HEAD;
the local-working-copy checkout body is also unchanged (line offsets differ,
and unrelated sparse-ignore/snapshot code differs). At the installed revision,
the entrypoint is `local_working_copy.rs:2889`, update `2230`, and writer `2071`.

An isolated `jj git init --no-colocate` fixture under
`.test-agent/jj-new-preservation/` used two 1 MiB regular files, a base and target
revision differing in only one file, and `cp --reflink=always` peers on Btrfs.
`filefrag -v` after `sync` exposed physical extents and `shared` flags; peers had
distinct inodes. Fsmonitor was explicitly disabled. Results:

| Transition/check | Observed |
|---|---|
| `jj new target`, equal file | Inode `253886642` and mtime `1792041729966675018` ns unchanged; all eight physical extent mappings and `shared` flags unchanged |
| Same transition, changed file | Inode `253886768` → `253886789`, new mtime/content; rewritten extents no longer marked shared |
| Another `jj new target`, same tree | Both files' inode, mtime and extent reports unchanged |
| Dirty new file, then `jj new target` | File removed from destination; old content retained by snapshot commit `16dd58c780380bda19bb395cd3ff1a49ccfcb595` |
| Exec-only change, same bytes | Inode `253886915` → `253886935`; mode `0644` → `0755`; previous sharing lost |
| Private 8-byte edit after reflinking | Peer retained all original bytes; edited first block became private while remaining extents stayed shared |

The scratch README, script and `results.json` retain reproducibility details;
they are intentionally ignored local evidence, not required project tooling.
The initial scratch run reached all main preservation assertions but failed on
an invalid auxiliary revset; the corrected complete run passed. No live donor,
workspace, or archived source was changed.

## Limits and implications for jw

- **Advance a prepared donor normally.** Unchanged tracked files keep sharing
  with old workspace copies. Changed files are recreated privately, leaving old
  workspace contents intact. Ordinary ignored dependencies remain unless they
  collide with newly tracked paths or another process/preparation changes them.
- **Preservation is not renewal.** If a file was edited and later returned to
  identical bytes, JJ does not seek a donor extent and re-share it. Likewise,
  advancing a workspace to content already present in a donor writes changed
  paths from the object store; it does not borrow those donor extents. Explicit
  reseeding remains useful to renew sharing and distribute prepared dependencies.
- **Sparse:** ordinary checkout touches only included paths. Changing sparse
  patterns explicitly materializes/removes paths (`2299–2323`); clearing and
  restoring sparsity is not a preservation no-op.
- **Untracked collisions:** ordinary snapshot may auto-track eligible new files.
  Files still absent from the old tree (ignored/untracked) are not blindly
  overwritten: exclusive-create probes cause checkout to skip occupied paths
  and record placeholders (`771–819, 2430–2449`). Skipping is not successful
  materialization of the requested target.
- **Conflicts:** equal tree IDs alone are insufficient. Changed conflict labels
  can rematerialize otherwise identical conflicted entries (`2556–2587`). Normal
  resolved regular-file preservation is the useful claim, not a blanket claim
  for conflict marker files.
- **Races/failure:** the workspace is locked and expected tree state checked
  (`lib/src/workspace.rs:461–485`), but arbitrary concurrent file writers are not
  excluded; checkout explicitly has a TODO about checking edits before overwrite
  (`local_working_copy.rs:2458`). This is not an atomic tree transaction or
  universal crash-recovery guarantee. Directory metadata can also change.

Relationship: [source freshening](source-freshen0.gpt61sol.md) uses precisely
this ordinary incremental advancement before dependency preparation;
[the reseed index](README.md) distinguishes the explicit sharing-renewal work.
