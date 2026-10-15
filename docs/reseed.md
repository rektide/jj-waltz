---
type: Guide
title: Reseed an existing workspace in place
description: One-off and batch copy-on-write reseeding, opt-in donor freshening, replacement policy, and interrupted-operation recovery.
resource: /docs/reseed.md
status: maintained
generated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
updated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
sources:
  - resource: "opencode://ses_ec1394461ffeEXBHM582vTs9VM"
    title: Implemented optional donor assertions, current-variant target selection, and snapshot-free frozen recovery with strict preparation batch selection
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
Without `--freshen`, it neither fetches upstream changes nor installs dependencies.

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
its legacy `--freshen` preparation path. Up remains unintegrated. Creation reads
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

## Freshen the source

Freshening prepares the donor before distributing it: fetch one declared remote,
select an empty JJ child of the requested source baseline, then run any configured
preparation commands in that donor. It does not automatically reset feature targets.

Declare the project-specific preparation argv in the primary checkout's ignored
`.jwreseed.local.toml`:

```toml
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

Command policy is **repository-local only**. No package manager, installer, or
builder is guessed, and no global command configuration is read. This is trusted
execution policy: programs may run project/package scripts, and `jw` is not a sandbox.

Legacy `[prepare]` remains a deprecated alias with a warning; `[prepare]` and
`[freshen]` together are an error, including empty tables. Rename the table to
`[freshen]` **after any pending preparation finishes**. Recovery still requires
the exact frozen local policy bytes, so renaming the table mid-operation stops it.

```sh
# Prepare the donor, but keep this feature's snapshotted revision.
jw reseed my-feature --freshen --source-at 'dev@origin' --dry-run

# Also start the target on an empty child of the updated baseline.
jw reseed worker-a --freshen --source-at 'dev@origin' \
  --at 'dev@origin' --copy-ignored node_modules --dry-run

# Explicit donor override; preparation happens here, not in the caller.
jw reseed worker-a --from ~/a/a/opencode --freshen \
  --source-at 'dev@origin' --copy-ignored node_modules --dry-run
```

With `--freshen`, CLI `--source-at` or local `source_at` is still a **required** goal
resolved after fetching, not a claim that the old donor is already prepared. This
legacy freshening path never defaults its goal to donor `@`. Without the flag,
assertions are optional as described above. Target `--at` and `--replace-content`
stay separate.

Preparation settings alone do not activate freshening: pass `--freshen` explicitly.
The commissioned `up` replacement is not integrated yet; the current public
interface remains `reseed --freshen`. Default-on freshening is not planned.

Remove `--dry-run` only when the source, remote, argv/cwd, and target policy are
right. A freshening dry-run does not fetch, select a new source change, install, or
refill files; its post-fetch goal is prospective and the exact commit may be unknown.
Normal real-work snapshots are
preserved, and writers must be paused. Installation failures or tracked-file changes
stop the batch before target contents are cleared. Partial dependency installation
requires recorded recovery before the donor can be borrowed again.

Only the declared remote is fetched, with `origin` as the fallback when it exists.
With no commands, freshening performs fetch/source advancement only. Selected
existing ignored payload may still be copied privately, but is reported as
**existing data, not freshly built**. Its presence does not require a builder;
its suitability for changed manifests remains your responsibility. A successful
command that changes tracked files still fails the baseline check. Source progress
is not automatically rolled back.

For interrupted source preparation, use `--resume` with all original literal target
names in their recorded order. Do not add `--freshen`, new source/goal overrides,
or replacement flags. The complete ordered selection remains required after
target plans are frozen; bare current-target resume never infers the rest of a
multi-target batch. Completed target receipts prevent repeating successful refills,
fetches, or installation.

```sh
jw reseed worker-a worker-b --resume
```

### Command progress and older preparation journals

New source preparation journals use version 2 and freeze the entire argv sequence,
completed count, and whether the next command was in flight. A step is durably
completed only after successful exit and baseline/source/target checks. Resume
skips completed steps and does not refetch a frozen goal.

**An interrupted or failed in-flight command may retry.** Its earlier attempt may
already have performed side effects, including after a successful exit but before
durable acknowledgement. Arbitrary external commands are not exactly-once.
Programs must tolerate retry against partially produced outputs, or you must
restore the required partial state deliberately. A later command may have changed
earlier commands' ignored outputs; completed steps are still not replayed. Until
all commands and the final source/payload proof succeed, borrowing is blocked.

Existing v1 journals remain resumable. Their single command becomes a one-step
sequence. A v1 `Verified` preparation is already completed and is not rerun;
otherwise completion was not recorded, so an unfinished command may retry. The
v2 representation is written on real resumed progress, not merely on lookup or
`--resume --dry-run`. Frozen source/target proofs and policy bytes are retained.
Target pending records and their receipts retain their existing schema.

An unfinished batch blocks new borrowing or freshening from its donor. Restore
installer-modified tracked bytes to the recorded goal before retrying; the same
recorded change and frozen parent are required, not an unrelated tree-equivalent
change. After preparation was verified, changed ignored payloads must also be
restored rather than silently accepted. Configuration and ownership changes stop
recovery. Keep credentials out of the configured argv: policy and arguments are
retained in the local preparation journal.

See the [source freshening design](../design/workspace-reseed/source-freshen0.gpt61sol.md)
for the earlier single-command evidence; the
[current engine contract](architecture.md#local-freshening-commands-and-recovery)
explains the sequence and migration implementation.

## Optional shortcuts

To avoid repeating source arguments, put `.jwreseed.local.toml` in the default
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

`--dry-run` validates and reports the plan without clearing or refilling target
contents. Planning can snapshot working copies to freeze their tracked work.

Normal snapshots are intentional: they preserve your real tracked work before
replacement. Recovery avoids recording only the temporary half-filled tree;
after reconstructing and verifying the recorded tree, normal JJ snapshots are
safe again. There is no general `--skip-snapshot` flag in this implementation.

Interrupted operations retain repository-scoped pending records outside the
replaceable contents. **Do not run ordinary snapshotting JJ commands in a
half-filled target.** Resume the recorded plan instead:

```sh
jw reseed my-feature --resume
# From the same pending variant, for a single-target operation:
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
before commands run. Once source preparation is verified, its actual root set is
frozen too and is used directly for target plans. Resume does not
inherit new global patterns. Existing v1/v2 preparation request arrays keep that
meaning: a recorded `[]` remains empty, not a missing value asking for defaults.
Verified root sets are not re-expanded under a newer matcher: a legacy pattern
that previously selected nothing does not suddenly add payload during recovery.
New matching roots outside the frozen set remain unselected; changes inside a
selected root still require exact payload restoration.

## Related material

- [CoW creation](../README.md#copy-on-write-workspaces) creates a new workspace;
  reseeding refreshes an existing one without private working-copy-state copying.
- [Design and acceptance checks](../design/workspace-reseed/reseed0.gpt6a.md)
  explain why public sparse-prefill works and the verification obligations.
- [Architecture](architecture.md#module-map) separates the command adapter,
  reseed coordination, JJ operations, filesystem cloning, links, and metadata.
- [Source freshening design](../design/workspace-reseed/source-freshen0.gpt61sol.md)
  records preparation/freeze ordering and the separately deferred default policy.
- [Wildcard-group design](../design/workspace-reseed/groups0.gpt61sol.md) preserves
  the selection rationale; this guide describes the implemented interface.

## Verification and limits

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
