---
type: Guide
title: Reseed an existing workspace in place
description: Keep-history CoW reseeding, source-only upstream preparation to an exact commit, private payloads, and frozen recovery.
resource: /docs/reseed.md
status: maintained
generated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
updated:
  by: model:openai/gpt-6.1-sol#high
  at: 2026-10-17
sources:
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
    title: Explicit fetch, source advancement, dependency preparation, and separately deferred default activation
  - resource: /CONTEXT.md
    title: Workspace identity, lifecycle metadata, and external ownership contracts
---

# Reseed an existing workspace

`jw reseed` replenishes copy-on-write sharing in existing JJ workspaces. It keeps
the workspace root, `.jj`, registration, name, bookmarks, and lifecycle metadata.
It never fetches upstream changes or installs dependencies. Use **`jw prepare`**
to fetch and prepare a source and report its exact frozen commit; the combined
`up` command was removed.

## One-off use

**No groups, configuration, or baseline assertion are required.** From a variant,
borrow the primary checkout's freshly snapshotted tracked state and refill the
current registered workspace:

```sh
jw reseed
jw reseed my-feature  # explicit target; needed at the primary checkout
```

`--from` defaults to the verified primary checkout of the current JJ repository,
not the current feature workspace or an arbitrary directory under `~/archive`.
An explicit `--from` or configured `source` overrides that default. For a separate
prepared source in the same repository:

```sh
jw reseed my-feature --from /path/to/prepared-source --source-at 'dev@origin'
```

By default, the target keeps its snapshotted working-copy revision, including
feature differences. Unchanged paths are reflinked from the prepared source;
JJ materializes paths that differ. A repeated run renews sharing even when file
contents have not changed.

Source and target revisions do **not** need to be identical: each unchanged path
can be reflinked independently. The donor tree assertion is **CLI `--source-at` >
local `source_at` > freshly snapshotted donor `@`**. A supplied assertion must
resolve to exactly one revision and match the donor's tracked tree, not necessarily
its commit identity. A mismatch stops before target snapshotting or clearing;
omission does not require the donor to be unchanged from an earlier baseline.
Donor and assertion are frozen at one operation. No source advancement happens.

To put a reusable workspace onto a new empty child of an explicit base instead:

```sh
jw reseed my-worker --from /path/to/prepared-source \
  --source-at 'dev@origin' --at 'dev@origin'
```

`--at` is explicit reset behavior, not a rebase or bookmark move. Existing tracked
work is snapshotted first. It remains in JJ history rather than being copied into
the new child. An already empty child of the same base can be reused.

Multiple positional names select a batch. With no names or groups, only the current
non-primary workspace is selected, including invocation from its subdirectory.
The primary requires names or groups. Explicit names **or any explicit group**
suppress the current-target default, even when the group matches nothing; no
selection ever means all workspaces. Names remain literal, not navigation tokens.

## Ignored data

Other ignored or untracked contents stay in the target unless explicitly selected.
With no CLI, local, or global selection, no ignored payload is copied. To replace
specific ignored dependency directories with private CoW copies:

```sh
jw reseed my-feature --from /path/to/prepared-source --source-at 'dev@origin' \
  --copy-ignored node_modules --copy-ignored 'packages/*/node_modules'
```

Quote patterns so your shell does not expand them. Matching **nonignored source
directories are skipped and reported**, not replaced as payload. For example,
`**/build/` can pass through a normal tracked `build/` and still find eligible
ignored matches deeper inside it. A selected source root must be ignored and
cannot overlap tracked files. If ignore rules cover forced-tracked files inside
it, that overlap is a hard error, not permission to clone the tracked tree as payload.
Selected roots must also be ignored in the target, with no tracked overlap or
unsafe obstruction; those target checks remain hard failures.
Directory patterns support literal components, `*`, `?`, and recursive `**`;
trailing `/` is accepted. Selection never follows directory symlinks or enters
`.jj`/`.git`. Missing matches succeed and are reported as skipped unless their
potential matches are already covered by a selected outer root. Selected roots
are **replaced**, so old
entries disappear. They are not hard links or shared-directory symlinks: edits in
one checkout do not alter the other.

### Ignored-copy policy layers

The root of the user config file (`$XDG_CONFIG_HOME/jj-waltz/config.toml`, or
`~/.config/jj-waltz/config.toml`) may declare:

```toml
# Root key, not inside [workspace] or [freshen].
copy_ignored = ["**/target", "**/node_modules", "**/dist/", "**/build/"]
```

This is opt-in configuration; `jw` does not write it for you. Effective lists are
**CLI explicit list > local explicit list > global list > none**. Lists replace,
not accumulate. A missing local `copy_ignored` inherits global policy; local
`copy_ignored = []` disables that inheritance. `--no-copy-ignored` disables copying
for the run, even alongside an explicit CLI list. There are no global
freshening/build commands.

| Global | Local | CLI | Effective selection |
| --- | --- | --- | --- |
| `["**/target"]` | missing | absent | `["**/target"]` |
| `["**/target"]` | `[]` | absent | `[]` |
| `["**/target"]` | `["cache"]` | `--copy-ignored node_modules` | `["node_modules"]` |
| any | any | `--no-copy-ignored` | `[]` |

The same policy and selector apply to `add`/`s` **new workspaces**, `reseed`, and
`prepare`. Creation reads
only the verified primary store owner's local copy list, without adopting local
source or freshening settings. It creates selected payloads after the tracked
checkout and before workspace links; an existing switch never refills payloads.
`--no-cow` controls tracked creation, not the independently required real-CoW
payload phase. See [creation's private dependencies](../README.md#private-dependencies-on-creation)
for examples, link conflicts, and failure cleanup.

All patterns are compiled together for each selection traversal.
The traversal keeps possible prefixes, skips impossible subtrees, and stops at
selected outer roots so nested payloads are not copied twice. It continues through
nonignored matches when another pattern can match below them. Frozen ignore-rule
assembly, topology/content validation, stamps, payload digests, CoW probing and
copying still have their own justified passes: this is not a promise of exactly
one filesystem pass for the entire operation.

Dependencies reflect the prepared source's installed state. If a feature changes
package manifests, run that project's frozen install in the feature workspace.
Symlinks are copied without traversal; source-bound absolute links are reported
because they are not a self-contained dependency snapshot. `jw` does not rewrite
package-manager links.

`--replace-content` explicitly authorizes deleting other unrecorded target
contents, including `.env`, except `.jj`, before refilling. It does not copy
arbitrary source dotfiles. There is no additional confirmation prompt.

## Prepare the source

`jw prepare` selects and fetches one pinned native upstream bookmark, selects an
empty donor child of the exact fetched goal, runs configured preparation commands,
and reports the **exact frozen commit** of the prepared source. It is strictly
source-only: it never selects targets, refills files, or advances a workspace.
It works only on its selected donor — which may be the caller's own checkout, so
plain `jw prepare` at the primary prepares the primary — and has the same meaning
from any registered workspace of the repository. It takes no names, no `--group`,
no `--at`, and creates no worker-preservation bookmarks (native fetch can update
refs).

Consume a prepared commit with ordinary reseed and an explicit `--at`:

```sh
jw prepare                       # prints the exact prepared commit when done
jw reseed worker-a --at <prepared-commit> --copy-ignored node_modules
jw reseed feature                # keeps its feature tree; no reset
```

Reuse of the printed commit needs no second fetch or preparation. Feature
rebasing remains an ordinary `jj` operation: `--at` starts an empty child of the
chosen base, it does not rebase the workspace's previous feature commits, and
reseed without `--at` keeps them checked out.

`--from` selects a different donor. Without it, the donor is the configured source
or the verified primary checkout — possibly the caller's own workspace.

### Native upstream selection

CLI `--source-at`, then local `source_at`, overrides automatic selection. Otherwise
select before fetching: one applicable native tracked local/remote pair related
to the clean donor base; if none, one remote bookmark exactly at that base,
including untracked refs. Ambiguity stops and asks for a one-run `--source-at`.
There is no default-main/origin inference, operation-log mining, or new following
state. Optional `[freshen].remote` constrains selection; it cannot redirect a
different explicitly selected remote. Fetch addresses only the pinned name/remote.

Unpublished donor work blocks preparation, including local commits beneath an
empty scratch change. Directly published source revisions and ordinary empty
children of published bases work. Post-fetch checks require publication and a
fast-forward selected remote. A missing goal may use the explicit simple form
`--source-at introduced-later@origin` when that native remote is configured;
dry-run leaves its exact goal unknown. No source advance occurs until fetch proves
the base is published there. The fetched goal is frozen once for the run and
reported as the exact commit.

Unlike reseed's **tree assertion**, prepare's `--source-at` selects the source's
**upstream goal**. Neither command writes a standing following preference.

### Local commands

Declare the project-specific preparation argv in the primary checkout's ignored
`.jwreseed.local.toml`:

```toml
# Optional override; omit to use native upstream inference.
source_at = "dev@origin"
copy_ignored = ["node_modules", "packages/*/node_modules"]

[freshen]
remote = "origin"
commands = [
  ["bun", "install", "--frozen-lockfile"],
  ["bun", "run", "build"],
]
```

For one command, `command = ["cargo", "build", "--locked"]` is also supported.
`command` and `commands` cannot both be present, **even if either is empty**.
Every command must be a nonempty argv with a nonempty program; `commands = []`
means no commands. Arguments are not implicitly shell-expanded. Commands run
sequentially in the selected donor root, including with `--from`, and stop on the
first failure. Each successful step must leave the tracked baseline unchanged,
including lockfiles, before it is recorded complete or a later command runs.
Command stdout is forwarded to stderr, so `--format=json` output stays clean.

Command policy is **repository-local only**. No package manager, installer, or
builder is guessed, and no global command configuration is read. This is trusted
execution policy: programs may run project/package scripts, and `jw` is not a sandbox.

Legacy `[prepare]` remains a deprecated alias with a warning; `[prepare]` and
`[freshen]` together are an error, including empty tables. Rename the table to
`[freshen]` **after any pending preparation finishes**. Recovery still requires
the exact frozen local policy bytes, so renaming the table mid-operation stops it.

```sh
# Preview the prospective fetch and commands without performing them.
jw prepare --dry-run

# Explicit upstream override, verifying selected ignored payload.
jw prepare --source-at 'dev@origin' --copy-ignored node_modules --dry-run

# Prepare a different donor; commands run there, not in the caller.
jw prepare --from /path/to/source --source-at 'dev@origin' --dry-run
```

Preparation settings alone never activate preparation: use `prepare` explicitly.
`prepare` has no target controls at all — no names, groups, `--at`, or
`--replace-content`. Consume a prepared commit with reseed `--at`, as shown above;
feature rebasing stays an ordinary `jj` operation.

Remove `--dry-run` only when the source, remote, argv/cwd, and payload policy are
right. A prepare dry-run does not fetch, select a new source change, install, or
refill files; its post-fetch goal is prospective and the exact commit may be unknown.
The donor's real work is snapshotted to preserve it, and writers must be paused.
Installation failures or tracked-file changes
stop the run before the source proof. Partial dependency installation
requires recorded recovery before the donor can be borrowed again.

Only the pinned bookmark on its selected native remote is fetched.
With no commands, preparation performs fetch/source advancement only. Selected
existing ignored payload may still be verified, but is reported as
**existing data, not freshly built**. Its presence does not require a builder;
its suitability for changed manifests remains your responsibility. A successful
command that changes tracked files still fails the baseline check. Source progress
is not automatically rolled back.

For interrupted source preparation, use `jw prepare --resume`. It finds the active
source-only journal before snapshotting — including when the recorded donor is
another workspace — and runs the same from any registered workspace. Do not add
new source, goal, or payload overrides; restore recorded state instead. A second
unfinished source-only preparation is refused until the first finishes.

```sh
jw prepare --resume   # continue the active source-only journal; goal not refetched
```

A completed source-only preparation records a verified completion receipt with an
explicitly empty target plan, not a dummy target. Legacy preparation journals are
different: the removed combined `up` command and older reseed preparation left
target-bearing batches whose recovery stays behind
`jw reseed ORIGINAL-NAMES --resume`, with their recorded order, target intent,
and history/bookmark plans intact. `prepare --resume` never reinterprets them,
and it never mutates a target.

### Command progress and older preparation journals

New prepare journals use version 4 and freeze native source selection and the
entire argv sequence,
completed count, and whether the next command was in flight. Their target plan is
always an explicitly empty array, so a v4 journal can never carry target
mutation. A step is durably
completed only after successful exit and baseline/source checks. Resume
skips completed steps and does not refetch a frozen goal.

**An interrupted or failed in-flight command may retry.** Its earlier attempt may
already have performed side effects, including after a successful exit but before
durable acknowledgement. Arbitrary external commands are not exactly-once.
Programs must tolerate retry against partially produced outputs, or you must
restore the required partial state deliberately. A later command may have changed
earlier commands' ignored outputs; completed steps are still not replayed. Until
all commands and the final source/payload proof succeed, borrowing is blocked.

**v1/v2 legacy reseed preparation** and **v3 combined-up journals** remain
resumable with `jw reseed ORIGINAL-NAMES --resume`, retaining their original
keep-target/explicit-base or frozen-goal history intent. The `up` subcommand is
gone; legacy recovery belongs to `reseed --resume`, and legacy progress keeps its
recorded semantics rather than acquiring prepare's source-only shape. The removed
`--freshen` flag is not needed for recovery. A v1 single command becomes a one-step
sequence. A v1 `Verified` preparation is already completed and is not rerun;
otherwise completion was not recorded, so an unfinished command may retry. The
v2 representation is written on real resumed progress, not merely on lookup or
`--resume --dry-run`. Frozen source/target proofs and policy bytes are retained.
Target pending records
and receipts retain their schema and optional immutable `history` field, including
the freshen bookmark intent recorded by removed up batches.

An unfinished batch blocks new borrowing or preparation from its donor. Restore
installer-modified tracked bytes to the recorded goal before retrying; the same
recorded change and frozen parent are required, not an unrelated tree-equivalent
change. After preparation was verified, changed ignored payloads must also be
restored rather than silently accepted. Configuration and ownership changes stop
recovery. Keep credentials out of the configured argv: policy and arguments are
retained in the local preparation journal.

See the [command-split decision](../design/up-performance/prepare0.gpt6a.md) for
why `up` became source-only `prepare` and what the first performance slice
changes, the
[source freshening design](../design/workspace-reseed/source-freshen0.gpt61sol.md)
for the earlier single-command evidence, and the
[current engine contract](architecture.md#local-freshening-commands-and-recovery)
for the sequence and migration implementation.

## Optional shortcuts

To avoid repeating source arguments, put `.jwreseed.local.toml` in the verified primary
workspace and keep it ignored:

```toml
source = "/path/to/prepared-source"
source_at = "dev@origin"
copy_ignored = ["node_modules", "packages/*/node_modules"]

[groups]
workers = ["worker-a", "worker-b"]
```

This permits either individual or group use:

```sh
jw reseed my-feature
jw reseed --group workers
jw reseed --group workers --at 'dev@origin' --dry-run
```

Array groups remain lists of literal workspace names; even `"worker-*"` in an
array is literal. Positional names and groups can be combined; duplicates are
removed. CLI source arguments override configuration. Repeated `--copy-ignored`
flags replace the configured selection;
`--no-copy-ignored` disables copying it. Reset behavior cannot be configured:
`--at` must be supplied for that run.

### All registered workers

The built-in `all` group needs no configuration and works from any registered
workspace, including a worker subdirectory:

```sh
jw reseed --group all --dry-run
jw reseed --group all
# Use an alternate prepared donor, excluding it from the targets:
jw reseed --group all --from /path/to/prepared-source --dry-run
```

Membership is every registered workspace except the **verified primary** and
the effective donor. Donor precedence remains CLI `--from` (relative to caller
cwd), local `source` (relative to primary), then primary. Existing path aliases
are compared by canonical identity. A renamed primary is still excluded; a
worker merely named `default` is not. Names are expanded in stable sorted order,
deduplicated with other names/groups, and retain their selection origins.

`all` is not an eligibility filter: missing/unusable roots, external ownership,
colocation, conflicts, pending recovery, and unbookmarked work remain **BLOCKED**
in readiness, even with `--on-unmatched warn`. No target is refilled if any
member fails preflight. It neither adopts nor prunes workspaces. Pause writers
before previewing too: dry-run may record JJ working-copy snapshots.

The group name `all` is reserved: defining `[groups].all` locally is an error.
An empty worker membership is an error, not implicit target selection.
Positional `jw reseed all` still names only the literal workspace `all`.
Recovery uses recorded literal names (`jw reseed NAME... --resume`), never
`--group all --resume`; later registrations cannot join a frozen recovery plan.

### Wildcard groups

Use a group table to match workspace names or filesystem locations. For example,
when `~/src/opencode-*` also contains unrelated projects:

```toml
[groups.opencode]
path_globs = ["~/src/opencode-*"]
on_unmatched = "warn"
```

```sh
jw reseed --group opencode --dry-run
jw reseed --group opencode

# Require every group candidate to match for this run:
jw reseed --group opencode --on-unmatched error --dry-run
```

Only exact registered workspace roots **in this JJ repository** are selected;
directory basenames need not equal JJ workspace names. Plain directories,
independent JJ repositories, and unrelated Git checkouts are not adopted or
modified. With `warn`, all skipped candidates appear together in one escaped
stderr line, without truncation:

```text
warning: skipped 2 unmatched group candidates: "/home/me/src/opencode-notes" (not a workspace of this repository), "/home/me/src/opencode-tools" (not a workspace of this repository)
```

Group tables support `names` (literal names), `name_globs` (registered workspace
names), and `path_globs` (root locations). Lists are unioned; repeated groups and
selectors run each workspace only once. For instance:

```toml
[groups.workers]
names = ["special-worker"]
name_globs = ["ocw-*"]
```

Patterns are case-sensitive whole-name or whole-path matches. `*` and `?` match
within one path component. Recursive `**`, character classes, braces, backslash
escapes, and `..` path components are rejected. A leading `~/` expands the home
directory; other relative paths start at the configuration's default/primary
workspace, regardless of the caller's workspace or subdirectory. There is no
environment-variable or shell expansion. Symlinks are not traversed, including
in a pattern's prefix. Files and symlinks are reported as non-directory misses.
This group syntax is separate from `copy_ignored`, which still supports `**`.

`on_unmatched` defaults to `"error"`. A group can opt into `"warn"`, or
`--on-unmatched error|warn` overrides all selected groups, including legacy
arrays. The policy covers unknown group-member names, unmatched expressions,
and paths that are not workspace roots of this repository. Direct positional
names are **always strict**. Unknown/empty groups and malformed patterns always
fail. Strict selection misses abort before source or target snapshots; warning
skips are reported before later source/target preflight, even if it fails.

A broken registered workspace is not an unrelated checkout. Missing/stale roots,
inconsistent identity, primary/source selection, external ownership, unsupported
colocation/sparsity/monitoring, pending recovery, I/O errors, and CoW failures
remain errors under `warn`. Path selection fails closed if a registration's root
cannot be resolved at all. If no targets remain, the command fails with
`no workspaces selected; nothing reseeded` rather than selecting everything or
claiming success. Dry-run shows selected names, paths, and selector origins.
Resume requires literal names and rejects groups or `--on-unmatched` overrides.

## Safety and recovery

Pause agents, builds, editors, and independent JJ processes that write the source
or targets. The repository-scoped advisory lock serializes reseed callers only;
it does not isolate arbitrary writers.

The first version requires full, conflict-free checkouts and JJ-only targets.
The primary checkout, prepared source, colocated/external targets, nested
repositories, mounted children, unsupported special files, and unverified
identities are rejected. A colocated source is permitted; its metadata is never
copied. Unsupported reflinking fails before clearing files rather than silently
falling back to an ordinary copy. Watchman-enabled mutation is not supported.
Resolve stale targets explicitly with JJ before retrying.

`--from` names a workspace root. A relative CLI path is resolved from the calling
directory; a relative configured `source` is resolved from the default workspace.
Source selection is CLI `--from`, then configured `source`, then the verified
primary checkout. Missing or inconsistent primary registrations fail closed:
repair the repository/workspace identity before retrying. An explicit source
chooses another donor; it does not bypass those identity checks. Ordinary borrowing
uses its snapshotted `@` unless a CLI/configured tree assertion is supplied.

The Git executable is used for isolated ignore-rule probes, including nested
`.gitignore` files and backend/global excludes. These probes do not create Git
metadata in a target workspace.
Symlinked `.gitignore` files are rejected when their rules cannot be verified.
On Linux, mount-ID verification requires `statx` support (kernel 5.8 or newer);
unavailable or blocked topology queries fail closed.

### Check readiness before refilling

```sh
jw reseed --group workers --dry-run
jw reseed worker-a worker-b --dry-run
jw reseed worker-a worker-b --at <exact-commit> --dry-run
```

For a new reseed, `--dry-run` uses the actual preflight checks and reports each selected
target as `PLAN …: READY` or `BLOCKED`. One blocked worker does not hide the
others' results. It exits nonzero if any target is blocked. If the shared donor
proof fails, targets that could not be checked say **not assessed**, never READY.
Checks include unfinished refill/preparation, ownership and registration, sparse/stale state,
conflicts, CoW support, ignored rules, obstructions, and configured links.
Readiness describes the observed state, not a reservation against later writes;
the real command checks again and never refills a valid subset of a batch whose
preflight is blocked.

No fetch, preparation command, refill, checkout reset, or deletion runs during
this check. **It is not strictly read-only:** normal JJ snapshots can record
working-copy changes, including auto-tracking under JJ's configured policy.
Output says whether a snapshot recorded new updates and whether the working-copy
tree differs from its parents. Unfinished refill/preparation targets are never
snapshotted by readiness; other unsafe targets fail their prerequisite checks.

Dirty work is not itself a blocker **if its work tip is bookmarked**. Without `--at`, `KEEP` preserves the current
feature tree, including snapshotted uncommitted changes. With `--at`, `RESET`
means the real run will replace the checkout with a fresh child of the exact
chosen base, **not rebase the feature**; the old work remains in JJ history at
the commit printed in the preview. Dry-run does not perform that reset. Unrelated
ignored/untracked contents are retained unless they are selected payload roots
(which will be replaced) or `--replace-content` was explicitly requested.

#### Bookmarked work is required by default

Both preview and real fresh reseeds, with or without `--at`, require a present,
unconflicted **named local or remote bookmark in JJ's current view** covering the
snapshotted work tip. The bookmark must point to that tip or one of its descendants.
A bookmark only on an older ancestor/base does **not** cover new edits. Arbitrary
snapshots, equal trees, tags, and inferred publication are not substitutes for a
bookmark. No fetch is added: remote coverage uses JJ's recorded remote ref, not
a new network check. Deleted/conflicted refs themselves do not count.

An ordinary undescribed empty single-parent working change uses its parent as the
work tip. Described empty changes and merges count themselves. Output identifies
the exact work tip and covering bookmark, including `@remote` where applicable.
Normal JJ snapshots can move a bookmark along with a rewritten named change;
`jw` respects that normal behavior but never creates or moves a bookmark for this
guard. If any target lacks coverage, the entire fresh batch is blocked before
refilling any target, including otherwise-ready peers.

Deliberately name the printed work tip using JJ (from any checkout of this
repository), or explicitly bypass **only** this guard:

```sh
jj bookmark create my-feature -r <reported-work-tip>
jw reseed worker --dry-run
jw reseed worker --allow-unbookmarked --dry-run
jw reseed worker --allow-unbookmarked
```

Ownership, source/content checks, pending recovery, ignored rules, links, and CoW
requirements remain enforced with `--allow-unbookmarked`. `prepare` is unaffected.
The bookmark inventory and target facts are frozen after snapshots; ancestry
checks are batched, not per file. Coverage is rechecked before execution and each
target's clearing boundary, so writers must still be paused.

New pending records freeze either the required reference/work tip or the explicit
opt-out. Resume does not accept `--allow-unbookmarked` as a policy override. The
recorded required bookmark must still cover the original frozen work tip; forward
moves are allowed, while deletion, conflict, or moving off that lineage stops
recovery before repairing or snapshotting a partial target. Restore the recorded
reference before retrying. Legacy pending records without this policy retain
their old recovery semantics; they are not retroactively subjected to this guard.

Normal snapshots are intentional: they preserve your real tracked work before
replacement. Recovery avoids recording only the temporary half-filled tree;
after reconstructing and verifying the recorded tree, normal JJ snapshots are
safe again. There is no general `--skip-snapshot` flag in this implementation.

### How intact workspaces are verified

Verifying a full-sparse, intact workspace is a **bounded JJ tracked-tree check**,
not a per-file raw-byte audit. `jw` snapshots the workspace once with auto-tracking
disabled (`snapshot.auto-track=none()`, `snapshot.auto-update-stale=false`,
`fsmonitor.backend=none`) and then requires one empty
`jj diff --from @ --to <frozen commit> --summary`. Equality is JJ's tracked-tree
semantics — including its normal stat cache and file representation — not an
independent raw-byte audit; see JJ's
[tree-state notes](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/docs/technical/architecture.md#treestate)
on the working-copy state cache. Because the snapshot runs first, a mismatch can be
reported **after** a snapshot was taken, possibly recording a rejected edit; the
earlier "no snapshot performed" rejection promise is retired on this normal path.
These snapshots never track additional files, and automatic stale-tree updates are
disabled; JJ may still refresh metadata-only stale state when the tree identity is
unchanged.

Recovery keeps a stricter, explicit invariant. A target whose recorded phase shows
mutation began (or whose sparse selection is incomplete) cannot be proven by an
empty diff: before any snapshot-capable command runs, recovery reconstructs every
recorded tracked path and raw-verifies each recorded byte, type, and mode against
pinned immutable data. The resume-before-snapshot guidance above is unchanged.

Interrupted operations retain repository-scoped pending records outside the
replaceable contents. **Do not run ordinary snapshotting JJ commands in a
half-filled target.** Resume the recorded plan instead:

```sh
jw reseed my-feature --resume
# From the same pending variant, including one member of a frozen batch:
jw reseed --resume
```

Current workspace identity and pending intent are resolved without snapshotting.
Resume revalidates the recorded source, root, registration, revision, and policy;
it neither reevaluates source defaults nor accepts replacement flags. A changed
source or target stops recovery and retains the pending record. Restore the frozen
source before retrying. A successful tracked refill alone is not success if ignored copying or link
application failed. Batch output distinguishes completed, failed, and pending
targets and exits nonzero for partial failure. Recovery never uses repository-wide
`jj undo`.

Preparation freezes the effective ignored-copy list, including explicit `[]`,
before commands run. **Legacy combined batches** — v1/v2 reseed preparation and
removed-up v3 journals — also freeze their verified root set once source
preparation is proven, and their frozen target plans use that exact set: resume
does not inherit new global patterns, and the frozen selection is not re-expanded
under a newer matcher. A new `prepare` receipt records its root set as part of the
source proof only; it does **not** govern later runs. A separate `reseed` invoked
afterwards resolves its own effective policy fresh, following the ordinary
CLI → local → global precedence at that run. Existing v1/v2 preparation request
arrays keep that
meaning: a recorded `[]` remains empty, not a missing value asking for defaults.
Verified root sets are not re-expanded under a newer matcher: a legacy pattern
that previously selected nothing does not suddenly add payload during recovery.
New matching roots outside a legacy frozen set remain unselected; changes inside a
selected root still require exact payload restoration.

## Related material

- [CoW creation](../README.md#copy-on-write-workspaces) creates a new workspace;
  reseeding refreshes an existing one without private working-copy-state copying.
- [Design and acceptance checks](../design/workspace-reseed/reseed0.gpt6a.md)
  explain why public sparse-prefill works and the verification obligations.
- [Command-split decision](../design/up-performance/prepare0.gpt6a.md) records
  why the public `up` command was deleted in favor of source-only `prepare`,
  reseed-only worker semantics, and the performance-first verification slice.
- [Incident handoff](../design/up-performance/handoff0.glm53m.md) preserves the
  measurement evidence that motivated the split.
- [Performance assessment](../tools/performance/README.md) retains the
  synthetic fixtures and subprocess-count budgets used to judge reseed/prepare
  costs and regressions.
- [Architecture](architecture.md#module-map) separates the command adapter,
  reseed coordination, JJ operations, filesystem cloning, links, and metadata.
- [Source freshening design](../design/workspace-reseed/source-freshen0.gpt61sol.md)
  records preparation/freeze ordering and the separately deferred default policy.
- [Wildcard-group design](../design/workspace-reseed/groups0.gpt61sol.md) preserves
  the selection rationale; this guide describes the implemented interface.

## Verification and limits

### Combined-up verification (historical)

The records below were taken against the **removed** combined `up` command at its
checkpoint. They evidence the underlying source-selection, preparation, and
recovery foundations that `prepare` and legacy `reseed --resume` now reuse; they
are not runtime verification of the prepare/reseed split, the v4 journals, the
`--format=json` surface, or the bounded tracked-tree verification change.

All 39 cases in `tests/freshen.rs` passed on installed JJ
`0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`, with disposable local
remotes, fake argv preparers, Btrfs temporary workspaces and `JW_TEST_REQUIRE_COW=1`.
Coverage includes tracked/non-origin and untracked-exact inference, missing-ref
preview/fetch/resume, unpublished source rejection, source-only preparation and
completion recovery, sequential command retries, frozen target subsets, exact
bookmark replay/opt-out, legacy v1/v2 keep-target/explicit-base recovery, and the
removed-flag migration error. Private inodes, shared FIEMAP extents and independent
writes are verified for tracked and globally selected ignored payloads. The focused
35-case engine/CLI nextest run passed; format and all-target/all-feature Clippy
checks passed (the existing manifest homepage warning remains).

The final combined native run passed all 39 preparation cases and two ordinary
reseed checks before its command timeout cancelled two further ordinary-reseed
checks; both cancelled checks passed on their focused retry (four ordinary reseed
checks total). This is not a new full JJ release
matrix, live-source exercise, installation, or unrelated cleanup-suite run.

### Current split status

The prepare/reseed split is implemented across three source commits —
`bed02439` (replace up with source-only prepare plus batch tracked-tree
verification), `25710456` (preserve JJ shim identity in verification process
counting), and `9bf436c5` (runnable preparation/resume guidance) — with focused
suites passing on the established baseline (installed JJ
`0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`, Linux Btrfs, real CoW
required): 71 CLI/reseed cases plus 27 prepare cases, 25 engine cases, all-target
Clippy, and formatting. Parent full-integration verification was still underway
with test-environment adjustments at documentation time, so treat the integrated
surface as implementation-led until that run lands. Repeatable measurement lives
in the [performance assessment guide](../tools/performance/README.md), whose
subprocess-count budgets are the regression metric for this slice.

### Earlier verification

The [native integration cases](../tests/reseed.rs) were verified on Linux Btrfs
with actual reflinks required, including FIEMAP shared-extents checks. The original
22-case official JJ 0.39.0, 0.44.0, 0.45.1, and 0.46.0 release matrix covered one-off and batch use,
source restoration, ignored replacement, symlink bytes, and interruption recovery.
The final optimized build passed all 22 cases on both 0.39.0 and 0.46.0;
0.44.0 and 0.45.1 were fully verified before the source-proof reuse optimization.
Primary-source inference and final-operation revalidation expanded the suite to
30 cases, all passing on official JJ 0.46.0 with real CoW required.
Explicit donor freshening adds 23 local-remote/fake-installer cases, verified on
official JJ 0.39.0 and 0.46.0 with real Btrfs CoW required. They cover source/target history,
unknown-goal previews, fetch/installer failures, changed source ownership,
verified-payload restoration, and interrupted batch handoff. The final existing
regression run passed 350 tests; the four CLI cases passed separately.
The freshening build has not yet been rerun on JJ 0.44.0/0.45.1 or non-Linux platforms.
Focused engine tests cover every write-ahead boundary and safe tracked-only type
transitions. Existing integration regressions, Clippy, and formatting also passed.

Local command sequencing and v1 migration extend the preparation suite to 28 cases,
all passing on official JJ 0.46.0 with real Btrfs CoW required. The no-command
existing-payload case also verifies shared extents and private inodes. Focused
configuration checks, Clippy, and formatting pass. This change did not repeat the
earlier release matrix or unrelated suites.

Shared ignored-copy policy and selection were checked with focused native cases
on official JJ 0.46.0 and required real Btrfs CoW: mixed tracked `build/` trees,
eligible descendants, replacement/empty/disabled precedence, missing matches,
forced-tracked source/target rejection, and unchanged topology/target-ignore guards.
Freshening checks include global no-builder payload borrowing and legacy frozen
`[]` recovery after global defaults change. Unit checks exercise the policy and
compiled selector. Shared extents, private inodes, and independent writes were
verified; the full release matrix was not repeated for this stage.

Wildcard groups at implementation checkpoint `9bd3a1c1` add six native cases
(36 total) and nine selection unit cases. All 353 regression tests passed with
official JJ 0.46.0 using an external temporary directory and explicit nextest
binary lookup. Separately, all 81 CoW/reseed tests passed on Linux Btrfs with
`JW_TEST_REQUIRE_COW=1` on official JJ 0.39.0 and 0.46.0, so unsupported-cloning
early returns in general temporary-directory fixtures are not counted as
real-CoW evidence.
The wildcard build has not yet been rerun on 0.44.0 or 0.45.1.

This is not a representative opencode throughput benchmark or proof of live
Watchman behavior. Watchman-enabled mutation, sparse targets, and colocated targets
remain unsupported. No live operator source or target was reseeded during these
checks.
