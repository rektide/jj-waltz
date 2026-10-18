---
type: Guide
title: Reseed an existing workspace in place
description: Keep-history CoW reseeding, source-only upstream preparation, private payloads, and deliberate out-of-band recovery.
resource: /docs/reseed.md
status: maintained
generated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
updated:
  by: model:openai/gpt-6.1-sol#medium
  at: 2026-10-18
sources:
  - resource: "opencode://ses_eb2158009ffeyJ0q8NuTd6g1RP"
    title: Integrated static checked streaming payload reconciliation, batch Linux topology guards and frozen persistent native ignore queries; revised refusal timing and convergence guarantees
  - resource: "opencode://ses_eb262a81effea1HgF0pPS3f1sy"
    title: Added navigation to the proposed inline-payload-guard design and its explicit preflight-timing tradeoff
  - resource: "opencode://ses_eb6ef3ef2ffe4155t844hpUnEL"
    title: Replaced the keep/reset reseed contract with donor-current sync and post-sync jj duplicate/rebase layering
  - resource: "opencode://ses_eb75a514cffe7jUOMWNZAIm3fX"
    title: Deleted recorded plans and progress recovery, retained live safeguards, and documented forward-only reseed and retry-from-start preparation
  - resource: "opencode://ses_eb867bfdaffefMVLpyaPUadbSp"
    title: Implemented and tested the reserved all group, donor-aware stable membership, unchanged readiness/bookmark guards, and literal recovery
  - resource: "opencode://ses_eb8801331ffeirQrF4GMFCxJHf"
    title: Implemented aggregate readiness, dirty keep/reset reporting, default-on bookmark coverage and opt-out, frozen proof rechecks, legacy recovery compatibility, and the fast journal/identity path
  - resource: "opencode://ses_eb899cdcdffecJnB7J1dSGs4ci"
    title: Replaced the combined up contract with source-only prepare, documented reseed-only worker semantics, legacy journal recovery, and the bounded tracked-tree verification split
  - resource: /design/up-performance/prepare0.gpt6a.md
    title: Command-split decision record — prepare is source-only, reseed keeps worker copying, performance first
  - resource: "opencode://ses_ec10d60dbffencx7PCZnh1dnjg"
    title: Integrated up with native source selection, source-only receipts, frozen preservation intent, legacy recovery, and native real-CoW verification
  - resource: "opencode://ses_ec1394461ffeEXBHM582vTs9VM"
    title: Implemented optional donor assertions, current-variant targeting, and phase-specific recovery with full preparation selection and independent frozen targets
  - resource: "opencode://ses_ec14d159effeiOep7U7f2iNQ7E"
    title: Connected ignored-copy policy documentation to delivered add/s payload integration and primary policy ownership
  - resource: "opencode://ses_ec1a8d43bffe02bXuEoA7hEW3l"
    title: Implemented local command/recovery foundations, then shared ignored-copy precedence, single-walk eligible selection, and frozen empty-selection compatibility
  - resource: "opencode://ses_ec37b01d2ffe9ytO0jRsdmoyXA"
    title: Implemented and verified reseeding, source defaults, and explicit donor freshening with recoverable preparation
  - resource: "opencode://ses_ec2e49e63ffe72f71k65378obu"
    title: Sol medium implemented explicit freshening and verified 23 native preparation/recovery cases on JJ 0.46
  - resource: "opencode://ses_ec2dd9386ffeDc0UNdIZu1bFbF"
    title: Independent source preparation, ownership, payload binding, receipt, and restoration review
  - resource: "opencode://ses_ec376c0e6ffewASuAvkLjXLTTU"
    title: Verified native sparse and symlink queries plus real-CoW integration across official JJ releases
  - resource: "opencode://ses_ec31c20c0ffeQNfMhRNezv4uuM"
    title: Implemented primary-source inference and verified 30 real-CoW integration cases on official JJ 0.46
  - resource: "opencode://ses_ec37a1a86ffeYXsS5j3A7Jng0t"
    title: Designed and implemented wildcard groups, strict or warn selection policy, and aggregate diagnostics
  - resource: /design/workspace-reseed/reseed0.gpt6a.md
    title: Prepared-source contract and stock-JJ sparse-prefill design
  - resource: /design/workspace-reseed/source-freshen0.gpt61sol.md
    title: Earlier fetch, source advancement, and dependency preparation design
  - resource: /CONTEXT.md
    title: Workspace identity, lifecycle metadata, and external ownership contracts
---

# Reseed an existing workspace

`jw reseed` renews copy-on-write sharing without replacing the workspace root,
`.jj`, registration, name, bookmarks, or lifecycle identity. It never fetches or
builds. `jw prepare` separately fetches and prepares only its donor and reports
the exact prepared commit. The combined `up` command is gone.

The update flow has three steps, in this order:

1. **Update the seed** — `jw prepare` fetches and builds in the donor.
2. **Sync a workspace to the seed** — `jw reseed NAME` makes the workspace's
   content be the donor's content: a fresh empty child of the donor's current
   commit, the whole tracked tree reflinked from the donor, and selected ignored
   payload copied. No feature logic runs inside reseed.
3. **Layer the feature back** — in jj, after the sync: `jj duplicate -r
   <bookmark> --onto <base>` keeps the old feature line and its dated bookmarks
   while copying it onto the synced base, or `jj rebase` moves commits and drags
   their bookmarks. Resolve conflicts in jj. This step is not jw's.

## One-off use

No groups, configuration, or donor assertion are required:

```sh
jw reseed --dry-run            # current registered variant
jw reseed my-feature           # literal target; primary requires explicit selection
jw reseed worker-a worker-b --from /path/to/prepared-source --dry-run
```

Remove `--dry-run` when the plan is right. Reseed snapshots real work and syncs
the workspace onto the donor's current working-copy commit: equal paths are
reflinked from the donor and JJ materializes the tracked tree. Selected ignored
payload is reconciled incrementally; unchanged payload is not recopied.

Donor precedence is CLI `--from` → primary-local `source` → verified primary
checkout of the same JJ repository. CLI relative paths start at the caller;
configured relative paths start at the primary. The donor must name a workspace
root, not an arbitrary subdirectory. Caller identity is independent of donor
selection; `--from` does not change the process cwd or caller role.

The optional donor **tracked-tree assertion** is CLI `--source-at` → local
`source_at` → freshly snapshotted donor `@`. A supplied assertion must resolve to
one exact commit whose tracked tree matches the donor. Equal trees need not have
equal commit IDs. A mismatch stops before target snapshots or mutations. The
assertion never changes the sync target.

`--at REVSET` is an optional explicit override of the target commit; without it
the target is the donor's current commit. `--at` creates a fresh empty child of
the selected base. It is **not a rebase** or bookmark move. Old tracked work
remains in JJ history and must be covered by a bookmark unless explicitly opted
out below. Names/groups suppress implicit current selection, even if they match
nothing. Empty selection never means all; maintenance names are literal, not
navigation tokens.

## Ignored data

Ignored/untracked target contents stay unless selected or explicitly erased:

```sh
jw reseed feature --copy-ignored node_modules --copy-ignored 'packages/*/node_modules'
```

Quote patterns. Components support `*`, `?`, recursive `**`, and trailing `/`.
Missing matches succeed and are reported; matching nonignored source directories
are skipped while eligible descendants remain searchable. Selected outer roots
include nested matches once. Selection never follows directory symlinks or enters
administrative directories. A selected root must be ignored under frozen donor
and desired rules and pass confinement/topology checks. Donor/desired tracked
overlap is an error; old target tracked files becoming payload are supported.

Selected roots are **mirrored incrementally** with private reflinks, not hard
links or shared directory symlinks. Target-only entries inside those roots are
deleted; unrelated contents outside them are not pruned. Symlinks are copied as links;
source-bound absolute links are reported, not rewritten. Dependencies reflect
the installed donor state. If feature manifests differ, run the project's frozen
install in the feature workspace.

Comparison uses file type, size and exact mtime, or literal symlink text, not
content hashes. Clean payload preserves inode/mtime and requires no writes.
Mode-only changes and byte edits retaining size/mtime are **not repaired**;
this is not an archive-metadata/ownership/ACL/xattr sweep. Newly copied files
preserve permissions and mtime. Absent roots and selected target file/link roots
are supported; final target symlinks are replaced, never followed.

The default build reconciles payload serially. Experimental Cargo feature
`payload-parallel` uses one command-owned borrowed pool across all selected roots
and targets, capped at four available workers, with matching directory admission
and 256 speculative directories. JJ operations remain serial. The feature is a
measurement variant, not a new runtime option or the selected default.

`--replace-content` explicitly permits deleting all non-administrative target
contents, including `.env`, except `.jj`, before refill. It does not copy arbitrary
source dotfiles and has no further confirmation prompt.

### Ignored-copy policy layers

The root of `$XDG_CONFIG_HOME/jj-waltz/config.toml` (otherwise
`~/.config/jj-waltz/config.toml`) may contain:

```toml
copy_ignored = ["**/target", "**/node_modules", "**/dist/", "**/build/"]
```

Lists replace in **CLI explicit list → primary-local list → global list → none**
order. Missing local policy inherits; local `copy_ignored = []` does not.
`--no-copy-ignored` wins even over an explicit CLI list. `jw` never writes this
policy for you. There are no global build commands.

The same selector and policy apply to new `add`/`s` workspaces, reseed, and prepare.
Creation inherits only the local copy list, not donor or preparation settings.
Existing switches never refill payloads. Neither path hashes or persists a
payload inventory. Rules come from pinned tracked `.gitignore` files plus frozen
effective backend/global excludes. Reseed reuses those inputs and a persistent,
memoized native Git ignore oracle; copying/comparison validates entries inline. See
[creation's private dependencies](../README.md#private-dependencies-on-creation).

## Prepare the source

```sh
jw prepare --dry-run
jw prepare --source-at 'dev@origin'
jw prepare --from /path/to/donor --format=json
jw reseed worker --copy-ignored node_modules
```

Prepare is source-only from any registered workspace. It selects/fetches a pinned
upstream, selects an empty donor child of the exact goal, runs local commands,
verifies the tracked tree and selected payload, then reports that exact commit.
JSON retains `source_commit_id`; installer stdout goes to stderr. It never selects,
snapshots, refills, or advances other workers, nor creates preservation bookmarks.
Native fetch may update refs. Reuse the reported commit without preparing again;
feature layering after a sync stays an ordinary JJ operation (`jj duplicate` keeps
the old line and its dated bookmarks; `jj rebase` drags bookmarks).

### Native upstream selection

CLI `--source-at`, then local `source_at`, overrides automatic selection. Otherwise
pin one applicable native tracked pair before fetch; if none, one remote bookmark
exactly at the clean donor base, including untracked refs. Ambiguity asks for a
one-run override. There is no guessed main/origin, operation-log inference, or new
following preference. `[freshen].remote` constrains selection, never redirects an
explicitly selected remote. Fetch addresses only that pinned name/remote.

Unpublished donor work blocks, including local commits beneath empty scratch.
The selected remote must fast-forward. A not-yet-imported goal may use simple
`branch@configured-remote`; dry-run leaves its exact commit unknown. Fetch must
prove publication before source advancement. Prepare's `--source-at` is an
upstream **goal**, unlike reseed's tracked-tree assertion.

### Local commands

Keep `.jwreseed.local.toml` ignored in the verified primary:

```toml
source_at = "dev@origin"  # optional native-selection override
copy_ignored = ["node_modules", "packages/*/node_modules"]

[freshen]
remote = "origin"       # optional constraint
commands = [
  ["bun", "install", "--frozen-lockfile"],
  ["bun", "run", "build"],
]
```

For one program use `command = ["cargo", "build", "--locked"]`. `command` and
`commands` cannot both be present, even when empty. Every command needs a nonempty
argv/program; `commands = []` means no commands. Execution is sequential in donor
cwd, without an implicit shell, and stops on first failure. Every successful step
must leave tracked files, including lockfiles, unchanged before the next runs.
These are trusted programs, not sandboxed code.

**Commands must tolerate retry.** If prepare fails, fix the donor and run prepare
again. All commands may repeat, including earlier successful steps; upstream is
selected and fetched anew. Tracked installer edits require deliberate restoration.
No source progress is automatically rolled back, and no durable completion or
partial-install health is recorded. Do not borrow incomplete dependencies.

With no commands, only fetch/source advancement runs. Selected existing payload
can be verified but is reported as **existing data, not freshly built**. There is
no guessed installer or automatic pruning. `[prepare]` remains a deprecated warned
alias for `[freshen]`; both together are an error. Settings alone never activate
preparation. Prepare accepts no target names, groups, `--at`, or `--replace-content`.

## Optional shortcuts

```toml
source = "/path/to/prepared-source"
source_at = "dev@origin"
copy_ignored = ["node_modules"]

[groups]
workers = ["worker-a", "worker-b"]
```

```sh
jw reseed feature
jw reseed --group workers --dry-run
```

Array groups contain literal names, including strings such as `"worker-*"`.
Names and repeated groups can combine; selected workspaces are deduplicated.
The sync target is never configured: supply `--at` to override it for that run.

### All registered workers

```sh
jw reseed --group all --dry-run
jw reseed --group all --from /path/to/prepared-source
```

Built-in `all` needs no configuration and works from a worker subdirectory too.
It expands sorted registrations, excluding only the verified primary and effective
donor by canonical root identity, not the name `default`. Unsafe/missing/external/
colocated/sparse/conflicted/unbookmarked members remain BLOCKED, even under
`--on-unmatched warn`; any blocker prevents all refills. It neither adopts nor
prunes registrations. Local `all` definitions and empty worker membership fail.
Positional `jw reseed all` still means the literal workspace `all`.

### Wildcard groups

```toml
[groups.opencode]
path_globs = ["~/src/opencode-*"]
on_unmatched = "warn"

[groups.workers]
names = ["special-worker"]
name_globs = ["ocw-*"]
```

```sh
jw reseed --group opencode --dry-run
jw reseed --group opencode --on-unmatched error --dry-run
```

Only exact registered roots in this JJ repository match. Plain directories,
independent repos, and unrelated Git checkouts are not adopted or modified.
Table groups union `names`, `name_globs`, and `path_globs`. Patterns are case-sensitive
whole-name/path matches with `*` and `?` within a component. Recursive `**`, classes,
braces, backslash escapes, and `..` are rejected for groups. Leading `~/` expands
HOME; other relative paths start at the primary, with no environment/shell expansion
or symlink traversal. This differs from ignored-copy patterns, which allow `**`.

`on_unmatched` defaults to `error`; `warn` emits all escaped skipped candidates in
one stderr warning, even if later preflight fails. CLI overrides selected groups'
miss policy. Direct positional names and target safety failures stay strict.
Unknown/empty groups and invalid patterns always fail. Broken registrations are
blockers, not unrelated candidates to skip. No remaining targets is an error.

## Safety and recovery

Pause agents, builds, editors, monitors, and independent JJ writers on donor and
targets, including before previewing. The advisory lock serializes `jw` reseed/
prepare calls, not arbitrary writers. Full, conflict-free checkouts and JJ-only
targets are required. Primary/source/overlapping targets, external ownership,
nested repositories, mounted children, special files, and unverifiable identities
are rejected. Colocated donors are allowed; linked Git/app-owned donors cannot be
prepared. Real reflinks are required; there is no ordinary-copy fallback. Watchman
mutation is unsupported. Resolve stale state explicitly with JJ.

Linux reads `/proc/self/mountinfo` once per worker batch before recursive
selection/traversal, rejecting nested mounts including same-device directory/file
bind mounts and selected-root mounts. A mount at the workspace root is allowed;
obvious overlapping filesystem-relative bind aliases, covered/stacked ancestors,
and applicable autofs roots/ancestors are refused. Autofs traversal can change
topology without another writer, so it cannot use this fixed snapshot. Only root `.jj` and permitted donor `.git` regions, which are never
copied/deleted, are excluded. The guard conservatively covers retained regions
too, and fails closed when procfs topology is unavailable. Other platforms keep
their existing native boundary checks. Paths and mount topology must stay stable;
this is not protection against hostile concurrent path/mount replacement.

### Check readiness before refilling

`--dry-run` uses the real preflight and reports each selected target READY or
BLOCKED, exiting nonzero for any blocker. Target-local failures do not hide peers.
A failed shared donor proof marks unchecked targets **not assessed**, never READY.
The real command rechecks structural guards and never refills just the valid
subset of a structurally blocked batch. Checks include ownership, registration,
sparse/stale state, conflicts, bookmarks, ignored rules, obstructions, and links.
Preview additionally audits each donor payload root once and every selected
target payload, including opaque one-sided interiors. It does not probe reflink
capability or prove later writes will succeed.

Normal execution does **not** audit payload interiors first. The checked streaming
comparison/copier/remover rejects administrative names and special entries even
when equal, but deep refusal or actual required-reflink failure can follow tracked
changes, earlier payload progress, or completed peers. The offending entry is not
acted on; there is no rollback. `--replace-content` retains broader eager content
validation because its deletion scope is larger. Resolve the violation and use
the recovery instructions below before retrying.

Preview does not fetch/build/refill/reset/delete, but normal JJ snapshots can
record real work and auto-track under JJ's policy. Sparse/mid-refill targets are
refused **before snapshotting**. Output explains new snapshot updates and the
single SYNC consequence, including where tracked work is saved in JJ history.
Unrelated data stays unless selected or erased
with explicit `--replace-content`.

#### Bookmarked work is required by default

A present, unconflicted named local or remote bookmark must cover the snapshotted
work tip or a descendant, in the default sync and `--at` modes alike. An older
ancestor/base,
equal tree, tag, or arbitrary snapshot does not count. Remote coverage uses JJ's
recorded view without fetching. An ordinary undescribed empty single-parent
scratch uses its parent; described empty changes and merges count themselves.
The preview names the work tip and covering bookmark.

```sh
jj bookmark create my-feature -r <reported-work-tip>
jw reseed worker --dry-run
jw reseed worker --allow-unbookmarked --dry-run  # deliberate exception only
```

`jw` never creates or moves bookmarks for this guard. It captures coverage after
snapshots, checks the whole batch before execution, and checks each target again
before clearing. Forward bookmark movement on the same lineage is allowed;
deletion, conflict, or movement off it stops mutation. `--allow-unbookmarked`
bypasses only coverage, not other safety checks. Prepare is unaffected.

### How intact workspaces are verified

Verification is a bounded JJ tracked-tree check, not a per-file raw-byte audit:
snapshot with `fsmonitor.backend=none`, `snapshot.auto-track=none()`, and
`snapshot.auto-update-stale=false`, then require empty
`jj diff --from @ --to <pinned commit> --summary`. Equality uses JJ's normal
[tree-state/stat-cache semantics](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/docs/technical/architecture.md#treestate).
A mismatch can be reported after recording a rejected edit. No additional files
are tracked; automatic stale-tree updates are disabled.

### Interrupted refills

Execution is one forward pass: validate, check bookmarked work, clear sparse
selection, `jj new` at the pinned base, prefill equal paths with
reflinks, reset sparse selection, reconcile selected ignored roots, apply links, and
verify. Commit-ID checks between JJ calls remain in memory. There is no durable
intermediate state and **jw never repairs a partial workspace**.

On failure after mutation starts, the error names the covered work tip, covering
bookmark, affected root, and this out-of-band remedy:

```sh
# In the affected workspace, using the bookmark printed in the error:
jj new <bookmark>
jj sparse reset
# Then re-run the intended jw reseed command.
```

Do not run ordinary snapshotting commands on a half-filled target first. The next
reseed refuses non-full sparse selection and prints the same two-command remedy
without repairing or snapshotting it. If coverage was deliberately bypassed, use
the printed prior commit or recover through JJ's operation log instead. Successful
peers stay complete; the batch exits nonzero on any failure. A successful tracked
refill is not complete if ignored copying or links fail.

There are no journals, receipts, legacy readers, migrations, or `--resume` flags.
Leftover old files are ignored and left untouched, never automatically pruned.
Each new prepare/reseed resolves its own policy and source selection.

## Related material

- [CoW creation](../README.md#copy-on-write-workspaces): new workspaces, rather than refilling existing ones.
- [Architecture](architecture.md#in-place-reseeding): ownership and live safety boundaries.
- [Performance assessment](../tools/performance/README.md): retained fixtures and process-count gates.
- [Command-split decision](../design/up-performance/prepare0.gpt6a.md): why preparation and worker copying are separate.
- [Inline payload-guard design](../design/up-performance/inline-policy0.gpt6a.md):
  rationale and alternatives; the streaming integration now follows the
  [reviewed library seam](../design/up-performance/diffdirs-review0.gpt6a.md).
- [Streaming implementation](../design/up-performance/streaming0.sol61m.md):
  integration boundaries, native correctness checks and outstanding measurements.
- [Incident handoff](../design/up-performance/handoff0.glm53m.md): preserved measurement evidence.
- [Earlier reseed design](../design/workspace-reseed/reseed0.gpt6a.md) and [source design](../design/workspace-reseed/source-freshen0.gpt61sol.md): historical rationale, not the current recovery contract.
- [Wildcard-group design](../design/workspace-reseed/groups0.gpt61sol.md): selection rationale.

## Verification and limits

Native tests use disposable local repositories/remotes and fake installers, never
operator workspaces. Real-CoW runs require `JW_TEST_REQUIRE_COW=1` on a reflink-capable
filesystem and check shared extents, private inodes, independent writes, ignored
replacement, symlink boundaries, readiness aggregation, bookmark guards, and
out-of-band recovery refusals. The bounded verifier retains zero-file-show-call
and batched symlink process-count regressions.

The journal-deletion checkpoint passed 258 native cases (library, reseed,
prepare, bookmark guards, and upstream selection) on Linux Btrfs with actual CoW
required. The separate CLI/colocation run passed 112 of 113 cases; the
`owned_cleanup_removes_only_target_and_keep_dir_preserves_files` failure also
reproduces with the preserved pre-cut executable (`workspace not found: missing`)
and is outside this change. Formatting and all-target/all-feature Clippy with
`-D warnings` passed; the existing Cargo manifest homepage warning remains.

The requested native synthetic fixture (8,555 files, 43 symlinks, two workers,
100 × 4,096-byte payload files) measured **5.93s** for reseed after journal
deletion, versus the supplied 6.64s baseline. A matched preserved-binary run
measured 7.46s before / 5.93s after; prepare measured 5.39s / 2.78s.
Instrumented 100 → 8,555-file scaling passed with zero JJ, Git, and
`file show` call growth. See
[retained measurement evidence](../tools/performance/README.md#journal-deletion-checkpoint).

A follow-up pass removed payload hashing and stat-stamping entirely (creation
digest/stamp walks, the untracked-`.gitignore` tree hunt, double-walk cloning,
and the per-target symlink scan): 269 native cases passed under the same
requirements, `jw add --copy-ignored` dropped from 3.56s to 1.53s on a
30,000-file/123 MB payload, and the same fixture measured **5.08s** reseed and
2.42s prepare. That pass responds to `jw add`/`reseed --dry-run` timeouts
against a large real `node_modules` donor.

Historical release-matrix results predate this deletion; they are not a new matrix
verification. Watchman mutation, sparse targets, and colocated targets remain
unsupported. Native synthetic timings are measurements of that fixture, not a
representative opencode throughput guarantee.
