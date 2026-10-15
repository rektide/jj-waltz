---
type: Research
title: Use native tracked bookmarks to select a donor update line
description: A graph-and-tracking algorithm for jw up without command-history mining, a new following state, or mandatory branch configuration.
resource: /design/workspace-reseed/bookmark-tracking0.gpt61sol.md
status: proposed
generated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
sources:
  - resource: "opencode://ses_ec1feb5c1ffe6ndPdipJnsLfEF"
    title: Native bookmark research and fixtures, then correction of exact-match selection, explicit remote fast-forward checks, and recovery-state boundaries
  - resource: https://github.com/jj-vcs/jj/tree/803ef00a0a6ee4ddc275d3138d5f9e7326399906
    title: Read-only archived JJ source for tracking metadata, fetch merging, structured templates, and bookmark advancement
  - resource: https://github.com/jj-vcs/jj/tree/33d3a595429b7e3b54a8ea769560247913bc8a1b
    title: Installed JJ revision used in native experiments and relevant source comparison
  - resource: /design/workspace-reseed/jj-new-preservation0.gpt61sol.md
    title: Incremental donor advancement preserves equal tracked files and existing sharing
  - resource: /design/workspace-reseed/source-freshen0.gpt61sol.md
    title: Existing donor advancement and dependency preparation context
  - resource: /src/jj.rs
    title: Existing public-CLI adapter and JSON-encoded native template precedent
---

# Track the native bookmark, infer the donor line from the graph

**The user's intuition is right: native bookmark tracking supplies the upstream
relationship.** For a normal donor on an empty child of tracked `dev`, `jw up`
can find `dev` and its tracked remote, fetch that remote, create an empty child
of the updated tip, run `[freshen].commands` in order, and hand the prepared
donor to target advancement/CoW refill. No new following record, op-log search,
or mandatory `source_at` configuration is needed.

An already-fetched bookmark can still be identified when there is **one tracked
pair across all local targets at or descending from the donor's clean base**,
not only the labels currently printed at `@-`. An exact match cannot outrank a
qualifying descendant: both remain candidates. This is a bounded selection
policy, not an assertion that JJ secretly stores a checked-out bookmark name.

## What JJ actually records and does

Archive: `/home/rektide/a/jj-vcs/jj` resolves to
`/mnt/bitblaster/archive/jj-vcs/jj`, inspected at
`803ef00a0a6ee4ddc275d3138d5f9e7326399906`. Source references below use this
revision. Installed experiment binary is
`jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`; relevant behavior was
also checked against its Git objects without changing the archive.

- `lib/src/op_store.rs:59–110`: a remote bookmark has its target and `New` or
  `Tracked` state. `tracked_target()` gives the merge base for synchronization.
- `lib/src/repo.rs:1916–1925`: `jj bookmark track dev@origin` **merges** the
  remote target into same-named local `dev` and marks the remote tracked. It can
  create the local bookmark or conflict with an existing divergent target;
  tracking is not merely a workspace annotation. Do not run it automatically
  to repair an ambiguous donor.
- `lib/src/git.rs:708–726`: fetch/import takes old tracked remote target as
  base and merges new remote target into local `dev`. Straight remote advance
  moves local `dev`; local work can leave it ahead or conflicted.
  `lib/src/refs.rs:95–117,150–174` supplies trivial/ancestry merge resolution.
- `lib/src/op_store.rs:164–177`: local bookmark targets, remote views and
  workspace commit IDs are available in the same native view. There is no
  workspace-to-bookmark field; this does not prevent the graph selection above.
- `cli/src/commands/git/fetch.rs:50–67,106–111,187–195`: native fetch supports
  `--tracked`, or selection of a particular remote and branch. Bare fetch has
  its own `git.fetch`/origin defaults; those are fetch selection, not evidence
  of which branch a particular workspace intended to follow.

Native convenience already exists: clone can track its selected default branch
(`cli/src/commands/git/clone.rs:447–454`), and
`remotes.<name>.auto-track-bookmarks` can opt matching newly imported bookmarks
into tracking (`lib/src/git.rs:1125–1141`, `cli/src/revset_util.rs:423–438`).
Those are existing JJ facilities, not required new jw configuration. An
untracked custom line can be deliberately established once with native
`jj bookmark track release@origin`; `jw up` should consume that relationship,
not alter tracking behind the operator's back.

### Bookmark advancement is useful, but a different direction

Native `jj bookmark advance` moves the closest ancestor bookmarks **to a known
target**, defaulting to `@`; its default source revset is
`heads(::to & bookmarks())`. It refuses backwards/sideways movement:
`cli/src/commands/bookmark/advance.rs:36–52,118–175` and
`cli/src/config/revsets.toml:20–21`.

The `experimental-advance-branches.enabled-branches`/`disabled-branches`
settings make eligible bookmarks advance during `jj commit`/`jj new`:
`cli/src/cli_util.rs:913–942,2660–2695`, `commands/new.rs:194–203,279–282`.
They do not pull the workspace to a fetched bookmark. Do not add or change
them for `jw up`; they are neither required nor a branch-selection oracle.

## Concrete minimal algorithm

`W` is the snapshotted donor `@`; `B` is its clean base; `L` is a tracked local
bookmark target; `R` is the corresponding tracked remote target. `X <= Y` means
equal or ancestor. Selection creates **no standing following state**. The
existing in-flight preparation journal may persist chosen names and IDs for
interrupted-operation recovery; that is not a permanent source association.

1. **Snapshot, inspect, and reject local donor work.** An ordinary native query
   snapshots working files first. Read `W`, parents, `empty`, description and
   conflict properties as structured data. Normal case: one-parent, conflict-free,
   undescribed empty `W`, with `B = W-`. Require `B` to be published on the
   selected remote line, as checked below; empty `W` alone is not sufficient.
   A described empty local commit is intentional work: stop conservatively.
   Also support a checkout directly at a published/bookmarked/immutable commit:
   use `B = W` if `W` is contained in the selected upstream history, even though
   that commit's changes relative to its parents are nonempty. Immutability or a
   local label alone is **not** proof of publication. Nonempty unpublished `W`,
   a merge scratch checkout, or conflicts stop.
2. **Select the line from native tracking.** Enumerate tracked remote refs
   excluding synthetic `@git`, join each to its same-named local bookmark.
   Collect **all** distinct `(local name, remote)` pairs with `B <= L`, including
   equality; require exactly one. Never prioritize an exact match over another
   qualifying descendant, or filter away a pair merely because its publication
   checks fail: stop rather than selecting another line. For conflicted local
   targets, inspect added and removed alternatives conservatively: any related
   alternative keeps the pair in consideration and blocks automatic selection.
   Likewise, a relevant conflicted remote is a stop, not a removable candidate.
   If conflict relevance cannot be established safely, stop for clarification.
   This handles `release` as naturally as `dev`, without consulting trunk.
3. **Check publication, then fetch the selected pair.** Resolved present targets
   must satisfy `B <= R` and `L <= R`: no unpublished base or local bookmark
   work. If local is ahead of the last-fetched remote, a fetch-only refresh of
   an **already uniquely selected pair** may establish it was published
   elsewhere; otherwise stop before moving the donor or running commands.
   Fetch cannot recover ambiguous source intent and must not be used to pick
   between candidate lines. Pin the selected names and old `B`, `L`, and `R`,
   fetch that remote/branch using native JJ, then reread the same pair and donor
   facts. Explicitly require **old `R` <= new `R`**, as well as old `B` <= new
   `R` and old `L` <= new `R`; the latter two alone would permit an upstream
   rewind when local `L` lagged remote `R`. Require local/remote references to
   be resolved, present and synchronized. Reject divergence, deletion or a
   non-fast-forward upstream move rather than choosing a side or another pair.
   Recheck donor work before advancement; a fetch changes the repo view and can
   abandon/rewrite affected commits under native policy.
4. **Advance and prepare.** `jj new <exact fetched commit ID>` in the donor;
   execute `[freshen].commands` sequentially, stopping on the first failure.
   Only a successfully prepared donor proceeds to target advancement and the
   existing CoW refill path. This proposal does not authorize discarding target
   work or alter the target-placement contract.

Thus, a donor at `empty @ -> A`, where `dev` and `dev@origin` have already moved
to `C` on `A..C`, selects that pair through `B <= L` **only if no other tracked
pair qualifies**. An empty child of local
work `P` fails `P <= R`, **even when `@` itself is empty**. Comparing trees alone
would miss local commit history, including committed edits later reverted.

The unique-descendant rule is sound for **safe published advancement under this
declared default policy**. It cannot reconstruct intent that no longer exists
in the graph. If several tracked lines descend from the same old base, it must
ask rather than invent a "closest", newest, or trunk tie-breaker.

**Correction to the first checkpoint (`214a2ac3`):** exact-match preference was
unsafe. Suppose donor base `B` originally had both tracked `main` and `dev`,
then another fetch moves `dev` to descendant `C` while `main` stays at `B`.
Exact-first selects `main` and silently changes lines. The corrected rule sees
both `main@origin` and `dev@origin` and reports ambiguity. Neither an exact
label nor a fetch is permission to infer the missing source intent.

### Native machine-readable interfaces, not display scraping

Verified on the installed binary:

```sh
jj bookmark list --tracked -T 'json(self) ++ "\n"'
jj log --no-graph -r @ -T 'json(commit_id) ++ " " ++ json(parents.map(|c| c.commit_id())) ++ " " ++ json(empty) ++ "\n"'
```

The first produces local records and tracked-remote records even when synced:

```json
{"name":"dev","target":["<id>"]}
{"name":"dev","remote":"origin","target":["<id>"],"tracking_target":["<local-id>"]}
```

`cli/src/commit_templater.rs:1560–1647` defines this serialization.
`CommitRef` also exposes `name`, `remote`, `tracked`, `tracking_present`,
`conflict`, `normal_target`, `added_targets`, `removed_targets`, `synced`, and
tracking ahead/behind counts (`1665–1746,1867–1995`). On a **remote** record,
`tracking_ahead_count` counts remote-only commits and `tracking_behind_count`
counts local-only commits; do not reverse those names. Local records' `tracked`
is not how to test whether a local bookmark has tracked remotes: enumerate the
remote relationships instead.

Implementation can JSON-encode these selected properties instead of depending
on raw target-array serialization. Check `conflict()` rather than guessing its
meaning from array length. The existing `src/jj.rs` adapter already uses native
JSON-encoded templates. This investigation tested 0.45.1, not the project's
entire supported-version matrix.

For ancestry and publication use native revsets with exact IDs:
`L & B::` for candidate descent, `B ~ ::R` and `L ~ ::R` for unpublished work.
`tracked_remote_bookmarks()` supplies commits but loses name/remote pairing;
combine native ref enumeration with revsets instead of guessing the pairing.

## Verification and remaining choices

Disposable fixture `.test-agent/bookmark-tracking/experiment.py` and
`results.json` exercised real local Git commits, JJ clone, explicit untrack/track,
and native fetch:

- Tracking restored the local/remote relationship; `--tracked` omitted it while
  untracked and included synced targets afterward.
- Fetch advanced `dev` and `dev@origin` from `33a5cbe2` to `5b1db81a` while donor
  `@ = c0b87da6` and its old parent remained unchanged. Descendant selection
  found `dev`; `jj new dev` advanced the donor afterward.
- A separate `release` line advanced from `5f6cd4c7` to `952688bd`, retaining the
  old empty donor checkout; only `release`, not `dev`, descended from that base.
- A dirty donor became nonempty on snapshot. After committing that work and
  moving local `release`, donor `@` was empty again but both its parent and
  local `release` failed publication against `release@origin`.
- Concurrent local/remote moves produced a **bookmark conflict**, not a file
  conflict. Native `CommitRef` output retained both local alternatives.

The initial fixture run reached the divergent case but queried a conflicted
bookmark as a single symbol; using `bookmarks(exact:"release")` corrected that
auxiliary check, and the full run passed. Direct-at-immutable checkout support
above is source/graph reasoning, not a separate executed fixture case.

A separate targeted native fixture, `exact-match-counterexample.py`, verified
the correction without rerunning the broader suite: donor stayed on empty
`@ -> 787e2c5c`; tracked `main@origin` remained at that base while fetching only
`dev` moved local/remote `dev` to `53809d6d`. Both targets matched `B::`.
`exact-match-results.json` records the native references: exact-first would
choose `main`, whereas the corrected algorithm must report both pairs.

**Unavoidable ambiguities:** multiple tracked names at the anchor; multiple
tracked remotes for the chosen name (even if they currently point to equal
commits); several descendant tracked lines after labels moved; or no usable
tracking relationship. A one-run `--source-at dev@origin` can resolve line
selection, while preserving the no-local-work checks. It need not create config
or a persistent following record. Force-push/divergence is a safety stop, not a
reason to guess harder.

The parent's supplied live archive facts (`@=8addb276`, parent `04aed8ed` carrying
capture labels, local `dev=bc1fd063`, differing origin/upstream tips) were not
re-snapshotted or modified here. They describe a nonstandard donor that may
correctly fail these checks; they do not justify making ordinary tracked-donor
use configure a new upstream state.

Relation: [ordinary checkout preservation](jj-new-preservation0.gpt61sol.md)
establishes why the donor's unchanged reflinks survive step 4;
[source freshening](source-freshen0.gpt61sol.md) supplies the preparation context.
