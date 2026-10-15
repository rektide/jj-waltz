---
type: Guide
title: jj-waltz — CoW reseeding fork
description: Independent proposed replacement README for this jj-waltz fork, organized around one create/reseed/up command model with progressive disclosure into focused guides.
resource: /README.proposed0.glm53m.md
status: proposed
generated:
  by: model:zai-coding-plan/glm-5.3
  at: 2026-10-15
sources:
  - resource: "opencode://ses_ebf063316ffeZf7tZ07pQTNqbG"
    title: Independent README rewrite take grounded in implemented create/reseed/up behavior
  - resource: /README.md
    title: Current maintained fork README used as coverage baseline
  - resource: /docs/reseed.md
    title: Maintained reseeding and up operator guide
  - resource: /docs/workspace-manager.md
    title: Focused interactive manager reference now carrying manager details
  - resource: /skills/jj-waltz/SKILL.md
    title: Current agent skill used as coverage baseline
  - resource: /CHANGELOG.md
    title: Carried unreleased changes
  - resource: /src/cli.rs
    title: Command and flag surface verified for this guide
  - resource: /src/reseed/engine.rs
    title: Target selection, donor precedence, and resume behavior verified
  - resource: /src/lifecycle.rs
    title: Creation-mode precedence resolution verified
  - resource: /src/workspace.rs
    title: Destination naming and registration resolution verified
  - resource: /design/workspace-reseed/up0.gpt6a.md
    title: Commissioned up command-outcome comparison this guide unifies
---

# jj-waltz — working fork

Manage named [Jujutsu](https://github.com/jj-vcs/jj) workspaces with `jw`: create
one, jump into it, refill it with private copy-on-write files without changing
its location, or advance it onto freshly prepared upstream work.

**This is a working fork of [EzraCerpac/jj-waltz](https://github.com/EzraCerpac/jj-waltz).**
It carries chosen-destination CoW creation, in-place workspace reseeding, and
integrated upstream advancement on top of upstream's workspace switcher.
It forks **jj-waltz, not Jujutsu**: everything works with stock JJ; no patched
`workspace add --adopt` command is required.

## The command model: create, reseed, up

Three commands share one donor/refill vocabulary. `add` and `s` create; `reseed`
renews an existing workspace in place; `up` freshens the donor and advances
targets onto the result.

| | `jw add` / `jw s` | `jw reseed` | `jw up` |
| --- | --- | --- | --- |
| Outcome | New workspace; `s` also enters it | Existing workspace, CoW sharing renewed | Donor freshened; targets advanced onto the prepared goal |
| Starting history | Empty child of the snapshotted source `@`, or explicit `--at REVSET` | Keeps the target's snapshotted revision by default, or resets via `--at REVSET` | New empty change at the exact fetched goal; there is no `--at` |
| Target selection | `NAME...` or `--path DESTINATION` | Bare in a variant targets it; the primary requires names or `--group` | Bare in a variant updates it; bare at the primary is source-only |
| Donor | Current checkout, or `--from SOURCE` | CLI `--from` > local `source` > verified primary | Same precedence; `--from` changes the donor, never the caller's role |
| `--source-at` | Donor tracked-tree assertion | Donor tracked-tree assertion | Selects the upstream advancement goal |
| Network | Never | Never | Fetches one pinned native bookmark |
| Local commands | Never | Never | `[freshen]` argv run sequentially in the donor |
| Unpublished target work | — | Snapshotted and kept | `freshen-<UTC timestamp>` bookmark unless `--no-freshen-bookmark` |
| Private ignored payloads | `--copy-ignored` (real CoW, external destination) | Same controls | Same controls |
| Preview and recovery | Failed creations roll back their new workspaces | `--dry-run`, `--resume` | `--dry-run`, `--resume` |

All three require actual copy-on-write support for their file-sharing claims:
APFS `clonefile(2)` on macOS and reflinks on Linux filesystems such as Btrfs and
XFS. Nothing here falls back to byte-copying or shared-directory symlinks when
CoW is required and unavailable.

Upstream already provides switching, workspace links, shell integration, the
interactive manager, and lifecycle checks. See [what this fork
adds](#what-this-fork-adds), the [reseeding guide](docs/reseed.md) for the full
contract, and the [changelog](CHANGELOG.md) for carried changes.

## What this fork adds

- **Create where you choose:** `jw add --path DESTINATION` and `jw s --path DESTINATION`
  infer a workspace name, with optional `--from SOURCE` for the donor.
- **Everyday creation defaults:** CoW, JJ-only, and an empty child of the source's
  current snapshotted `@`; explicit mode and revision selections remain available.
- **Refill an existing workspace:** `jw reseed NAME` renews CoW sharing while
  keeping the root directory, `.jj`, registration, bookmarks, and workspace identity.
- **Keep features or reset worker slots:** keep the target revision by default;
  add `--at REVSET` to start an empty child of a chosen base. Real work is
  snapshotted before replacement.
- **Advance with `up`:** infer and pin a native upstream, fetch, run local
  preparation, preserve unpublished target work with timestamp bookmarks, and
  start targets on new empty changes at the exact prepared goal.
- **Private dependency copies:** `--copy-ignored node_modules` replaces selected
  ignored directories with independent CoW copies, not shared-directory symlinks.
- **One-offs first, batches when useful:** named targets need no groups. Local
  configuration and groups are optional shortcuts; `--dry-run` previews the plan
  and `--resume` safely finishes an interrupted refill.
- **Wildcard batches without touching unrelated checkouts:** group `name_globs`
  or `path_globs` select registered workspaces. Opt into `on_unmatched = "warn"`
  to skip unrelated candidates and list them in one warning line; see
  [wildcard groups](docs/reseed.md#wildcard-groups).

## Install this fork

From this checkout, install the binary carrying these changes:

```sh
cargo install --path . --locked
```

Source commits do not update an already installed `jw`. For development, use
`cargo build --bin jw` and invoke `target/debug/jw` explicitly instead of
installing. The [upstream distribution commands](#upstream-distributions)
install upstream and may not carry this fork's additions.

Build from source with Rust 1.88 or newer.

## Everyday commands

```sh
jw add my-feature               # CoW/JJ-only sibling repo.my-feature, at source @
jw s my-feature                 # print its path; shell setup enables cd
jw ^                            # default workspace
jw -                            # previous workspace
jw list
```

Reseeding borrows the verified primary checkout by default. From a variant:

```sh
jw reseed --dry-run             # this variant; keep its work, borrow donor @
jw reseed my-feature --dry-run  # explicit target; required at the primary

# Optionally assert the donor's tracked tree matches BASE.
jw reseed my-feature --source-at BASE --dry-run

# Reset a reusable worker to that base and copy its installed dependencies.
jw reseed worker-a --source-at BASE --at BASE \
  --copy-ignored node_modules --dry-run
```

Replace `BASE` with a revset such as `main@origin`, and remove `--dry-run` when
the plan is right. Use `--from /path/to/prepared-source` for another source.
No fetch or package install happens during reseed. Use `jw up --dry-run` to
preview freshening and advancement of this variant (at the primary:
source-only). See [advancing with `up`](#advancing-with-up).

## Creating workspaces

`add` and `s` share source, placement, and creation policy. From the `opencode`
repository, for example:

```sh
cd ~/src/opencode
jw add --path ~/src/opencode-fix  # workspace name: fix; no duplicate NAME needed
jw s --path ~/src/myfix          # workspace name: myfix; no clear repo- prefix
```

Or select the source explicitly, including from outside a checkout:

```sh
jw s --from ~/src/opencode --path ~/src/opencode-review  # workspace name: review
```

Without `--from`, the current checkout supplies the repository, files, and
revision context. Repository identity follows verified JJ storage ownership,
not the current variant's directory name or a workspace merely named `default`.
Relative destinations are resolved from the caller's directory, before entering
`--from`.

### Create at a chosen destination

`--path` selects one exact destination. An existing registration there supplies
its actual workspace name; `s` enters it without recreation, while `add` reports
that it already exists. Conflicting name/path registrations and occupied new
destinations are errors. An optional explicit `NAME` can name a new destination
instead of inference.

Name inference strips a leading `<repo>-` prefix from the destination basename:
`opencode-fix` under repository `opencode` becomes `fix`; `myfix` stays `myfix`.
Name-only usage is unchanged: `jw add fix` creates a sibling named `repo.fix`
beside the verified owning checkout. No automatic `~/src` destination is
selected. Shell initialization is needed for `s` to change the parent shell
directory; the raw binary only reports the selection.

### Creation base and donor assertion

Without `--at`, `jw` snapshots the source before freezing its current `@` as the
base, so dirty tracked changes are included. Normal JJ auto-tracking rules apply
during this snapshot. The new workspace starts as an empty child of that exact
commit; a merge working-copy commit is itself a valid single base.

`--at REVSET` instead selects one explicit base through a frozen operation; it
is not replaced by the current source revision, and it must resolve to exactly
one revision. To select a sole parent deliberately, use `--at 'parents(@)'`;
multiple parents remain an error for that explicit revset.

Optional `--source-at REVSET` asserts that the donor's freshly snapshotted
tracked tree equals one exact revision, even when commit identities differ. It
does not select the target parent: without `--at`, that remains donor `@`; with
`--at`, the explicit target base wins. Assertion and base resolve at one frozen
operation.

Creation does not inherit local reseed `source`, `source_at`, or freshening
settings; only the shared ignored-copy policy applies. Existing-only switches
skip assertions.

### Copy-on-write workspaces

`jw add` and `jw switch` default to cloning the source checkout's tracked files
instead of letting JJ write each one. New workspaces are JJ-only, even when the
source is colocated. Clones share storage until either copy changes. Use
`--no-cow` for an ordinary JJ checkout, or disable CoW in user configuration:

```toml
[workspace]
copy_on_write = false
```

The result matches an ordinary JJ checkout of the creation base, including the
source checkout's sparse patterns. The tracked checkout clones only files JJ
tracks; ignored data is copied only when selected by [ignored-copy
policy](#ignored-copy-policy). Use [workspace links](#workspace-links) only when
deliberate sharing is wanted; tracked directories are never replaced by
whole-directory links.

When the filesystem cannot clone, `jw` warns and uses ordinary JJ workspace
creation. With Watchman configured, or if JJ rejects the copied working-copy
state, `jw` also warns and recreates through the ordinary path. These are
ordinary checkouts, not CoW successes. CLI warnings go to stderr, including when
shell switching reserves stdout for a path; the interactive manager shows a
warning report. Every fallback preserves the source checkout's sparse patterns.

### Creation-mode precedence

Mode settings are user-scoped, not repository-scoped. Omitted settings mean CoW
and JJ-only. Explicit `false` remains distinct from omission.

| Selection | New workspace mode |
| --- | --- |
| No mode settings | CoW, JJ-only |
| CLI `--colocate` | Ordinary colocated checkout, overriding configured CoW |
| CLI `--cow` | CoW/JJ-only, overriding configured colocation |
| CLI `--cow --colocate` | Conflict |
| Config `colocate = true` alone | Ordinary colocated checkout |
| Config `copy_on_write = true` and `colocate = true` | Conflict unless a CLI override resolves it |
| `--no-cow`, otherwise unset | Ordinary JJ-only checkout |

Positive CLI mode selections take precedence over configuration. Otherwise CLI
opt-outs override their corresponding config fields: `--no-cow` plus configured
`colocate = true` keeps colocation; `--no-colocate` plus configured CoW keeps
CoW. `--cow --no-cow` and `--colocate --no-colocate` are invalid combinations.
These settings affect only newly created workspaces; switching to an existing
workspace never converts it.

### Private dependencies on creation

```sh
jw add --path ../repo-fix --from /path/to/source \
  --copy-ignored node_modules --copy-ignored '**/target/'
jw s another --no-copy-ignored  # override local/global payload selection
```

`add` and `s` share repeatable `--copy-ignored` and `--no-copy-ignored`
controls. Selected directories must be ignored in both the snapshotted donor and
the exact target base, with no tracked overlap. Nonignored source matches are
reported and skipped; nested selections copy once, and missing matches succeed.
Payload copies require real private CoW even with `--no-cow` or `--colocate`:
those flags control the tracked checkout only. There is no payload byte-copy or
shared-link fallback.

When an actual payload is selected, destinations nested inside the donor are
rejected before creation, including aliases through symlinked parents. Choose an
external `--path` or use `--no-copy-ignored`; selections with no matches retain
normal nested placement. This keeps the frozen donor proof intact and prevents
recursive payload overlap.

Payloads are installed before links. A configured link overlapping a selected
private root is an error; use a source-equals-target `.jwlinks.local.toml`
override to keep it private. A failed creation rolls back only its new
workspaces, including read-only dependency copies. Pause donor writers while
creating copies. Switching to an existing workspace never refills dependencies
from policy or creation flags.

## Reseeding an existing workspace

One-off use needs no groups or configuration. Inside a variant, omit the name to
refill that current registered workspace; at the primary, name targets or a
group:

```sh
jw reseed
jw reseed my-feature
jw reseed my-feature --source-at 'dev@origin'  # optional tree assertion
```

Explicit names or groups never also select the current variant, even if a group
matches nothing. No selection ever means all workspaces. Literal target names
are not navigation tokens.

This preserves the target's snapshotted working-copy revision and feature
changes, root directory, `.jj`, registration, bookmarks, and lifecycle metadata.
Unchanged paths are reflinked from the donor; JJ materializes paths that differ.
Source and target revisions do not need to be identical. A repeated run renews
sharing even when file contents have not changed. Add `--at 'dev@origin'` to
start an empty child of that base instead; existing tracked work is snapshotted
first and remains in JJ history rather than being copied into the new child.

### Donor and assertion precedence

When run inside a JJ workspace, `--from` defaults to its verified primary
checkout. An explicit `--from` or configured `source` takes precedence; the tool
does not search arbitrary archive directories. `--from` names a workspace root:
a relative CLI path resolves from the calling directory, a relative configured
`source` from the default workspace.

The donor tree assertion is CLI `--source-at`, then local `source_at`,
otherwise its freshly snapshotted `@`. A supplied assertion must resolve to one
exact revision and match the donor's tracked tree — tree equality, not commit
identity. A mismatch stops before target snapshotting or clearing. Donor and
assertion are frozen at one operation; no source advancement happens.

Use `--copy-ignored node_modules` to replace an ignored dependency directory
with a private CoW copy, or `--replace-content` to explicitly discard all other
unrecorded target contents except `.jj`. Pause writers before reseeding.
`--dry-run` explains the plan; `--resume` finishes an interrupted recorded
operation. Unsupported CoW fails before clearing files rather than falling back
to copying.

### Restrictions and recovery

Reseeding requires full, conflict-free checkouts and JJ-only targets. The
primary checkout as a target, colocated or external targets, nested repositories,
mounted children, unsupported special files, and unverified identities are
rejected. A colocated *source* is permitted; its metadata is never copied.
Watchman-enabled mutation is not supported. On Linux, mount-ID verification
requires `statx` (kernel 5.8 or newer). The Git executable is used for isolated
ignore-rule probes; symlinked `.gitignore` files whose rules cannot be verified
are rejected.

Interrupted operations retain repository-scoped pending records outside the
replaceable contents. **Do not run ordinary snapshotting JJ commands in a
half-filled target.** Resume the recorded plan instead:

```sh
jw reseed my-feature --resume
# From the same pending variant, including one member of a frozen batch:
jw reseed --resume
```

Resume revalidates the recorded source, root, registration, revision, and
policy; it neither reevaluates source defaults nor accepts replacement flags.
A changed source or target stops recovery and retains the pending record.
Recovery never uses repository-wide `jj undo`. See the
[reseeding guide](docs/reseed.md) for optional configuration, groups, and the
full safety contract, and the [design](design/workspace-reseed/README.md) for
the sparse-prefill mechanism and acceptance checks.

## Advancing with up

`jw up` fetches one pinned native upstream bookmark, selects an empty donor
child of the exact fetched goal, runs configured preparation commands, then
starts each target on a **new empty change at that same goal** and refills it
with private CoW files. This is not a rebase and does not move existing feature
bookmarks.

| Invocation (inside this JJ repository) | Selection |
| --- | --- |
| `jw up` in a variant | Freshen the primary/configured donor and update this variant |
| `jw up` at primary | Freshen the donor only; no target or dummy workspace |
| `jw up worker-a worker-b` or `jw up --group workers` | Only explicit targets, never also current |

Both `up` and `reseed` require invocation within a registered workspace, even
with `--from`. At the primary, bare `up --from OTHER` prepares `OTHER` only.
Empty explicit group results remain errors.

### Native upstream selection

CLI `--source-at`, then local `source_at`, overrides automatic selection.
Otherwise `up` selects **before fetching**: one applicable, unambiguous native
tracked local/remote pair related to the clean donor base; if none, one remote
bookmark exactly at that base, including untracked refs. Ambiguity stops and
asks for a one-run `--source-at`. There is no default-main/origin inference,
operation-log mining, or new standing following state. Optional
`[freshen].remote` constrains selection; fetch addresses only the pinned
name/remote.

Unpublished donor work blocks advancement, including local commits beneath an
empty scratch change. Post-fetch checks require publication and a fast-forward
selected remote. The fetched goal is frozen once for commands and targets. When
no unique native line exists, supply a one-run `--source-at`; a missing goal may
use the simple form `--source-at introduced-later@origin` when that native
remote is configured, with the exact goal unknown in preview.

Unlike reseed's tree assertion, up's `--source-at` selects the source's
**advancement goal**.

### Freshening commands

Declare the project-specific preparation argv in the primary checkout's ignored
`.jwreseed.local.toml`:

```toml
[freshen]
remote = "origin"
commands = [
  ["bun", "install", "--frozen-lockfile"],
  ["bun", "run", "build"],
]
```

For one command, `command = ["cargo", "build", "--locked"]` is also supported.
`command` and `commands` cannot both be present, even if either is empty.
Commands run sequentially in the selected donor root and stop on the first
failure; each successful step must leave the tracked baseline unchanged,
including lockfiles. Command policy is repository-local only: no package
manager or builder is guessed, and there is no global command configuration.
This is trusted execution policy — programs may run project/package scripts;
`jw` is not a sandbox.

With no commands, freshening performs fetch and source advancement only.
Selected existing ignored payload may still be copied privately but is reported
as existing data, not freshly built. A successful command that changes tracked
files still fails the baseline check. A freshening dry-run does not fetch,
select a new source change, install, or refill files; its post-fetch goal is
prospective.

### Preserving work left behind

Before any target is cleared or advanced, all target history plans are saved
atomically. Work not contained in the exact fetched goal gets a collision-safe
UTC bookmark `freshen-YYYYMMDDTHHMMSS.nnnnnnnnnZ` (suffix `-2`, etc. if needed).
An ordinary undescribed empty one-parent target uses its parent as its work tip;
described empty changes and merges retain `@`. Publication uses ancestry, not
equal file trees. `--no-freshen-bookmark` disables the automatic bookmark only;
JJ history preservation and source-work checks remain. The choice is frozen
before preparation, and resume reuses the exact stored name and tip.

To freshen a donor yet keep a feature revision, run source-only `up` at the
primary, then ordinary `reseed` for the feature.

### Resume and legacy batches

```sh
jw up worker-a worker-b --resume  # required until target plans freeze
jw up worker-a --resume           # one frozen target
jw up --resume                    # current frozen target, or source-only at primary
```

Completed command steps are skipped on resume; an interrupted in-flight command
may retry — arbitrary external commands are not exactly-once. Until all commands
and the final source/payload proof succeed, borrowing is blocked. An unfinished
batch blocks new borrowing or freshening from its donor.

While source preparation has no frozen target plans, resume with all original
literal target names in their recorded order. Once plans freeze, individual
targets or subsets resume independently without refetching or rebuilding; bare
`--resume` inside a variant selects only the current recorded target. Source-only
preparation records a verified completion receipt with an empty target plan, not
a dummy target. Do not add freshening or selection overrides during recovery.

`[prepare]` remains a deprecated warned alias; rename it to `[freshen]` only
after pending recovery finishes, since frozen policy bytes must match.
`reseed --freshen` is removed. Legacy v1/v2 reseed preparation batches still use
`jw reseed ORIGINAL-NAMES --resume`, retaining their original
keep-target/explicit-base intent; never reinterpret them as up. See
[source freshening](docs/reseed.md#freshen-the-source) for the full contract.

## Ignored-copy policy

One policy family controls private ignored payloads across `add`/`s` creation,
`reseed`, and `up`. The root of the user config file
(`$XDG_CONFIG_HOME/jj-waltz/config.toml`, or `~/.config/jj-waltz/config.toml`
when unset) may declare:

```toml
# Root key, before any [workspace] or [trunk] table.
copy_ignored = ["**/target", "**/node_modules", "**/dist/", "**/build/"]
```

This is opt-in; `jw` does not write it for you. Effective lists are **CLI
explicit > local explicit > global > none**. Lists replace, not accumulate:

| Global | Local | CLI | Effective selection |
| --- | --- | --- | --- |
| `["**/target"]` | missing | absent | `["**/target"]` |
| `["**/target"]` | `[]` | absent | `[]` |
| `["**/target"]` | `["cache"]` | `--copy-ignored node_modules` | `["node_modules"]` |
| any | any | `--no-copy-ignored` | `[]` |

A missing local `copy_ignored` inherits global policy; local `copy_ignored = []`
disables that inheritance; `--no-copy-ignored` disables copying for the run,
even alongside an explicit CLI list. The repository-local list lives in the
verified primary store owner's `.jwreseed.local.toml`, not an arbitrary
workspace named `default` or the donor's local file. Creation reads only that
ignored-copy list from local configuration — not `source`, `source_at`, or
freshening settings.

Selected roots must be ignored with no tracked overlap on both sides; matching
nonignored source directories are skipped and reported while eligible ignored
descendants remain discoverable. Directory patterns support literal components,
`*`, `?`, and recursive `**`; trailing `/` is accepted. Selection never follows
directory symlinks or enters `.jj`/`.git`; missing matches succeed. Selected
roots are **replaced** with private CoW copies: edits in one checkout do not
alter the other. Dependencies reflect the prepared source's installed state; if
a feature changes package manifests, run that project's frozen install in the
feature workspace. Source-bound absolute symlinks are reported because they are
not a self-contained dependency snapshot; `jw` does not rewrite package-manager
links. See [policy examples and safety boundaries](docs/reseed.md#ignored-copy-policy-layers).

## Target groups

To avoid repeating target names, put `.jwreseed.local.toml` in the default
workspace and keep it ignored:

```toml
source = "/path/to/prepared-source"
source_at = "dev@origin"
copy_ignored = ["node_modules", "packages/*/node_modules"]

[groups]
workers = ["worker-a", "worker-b"]
```

```sh
jw reseed my-feature
jw reseed --group workers
jw reseed --group workers --at 'dev@origin' --dry-run
```

Array groups remain lists of literal workspace names; even `"worker-*"` in an
array is literal. Positional names and groups can be combined; duplicates are
removed. CLI source arguments override configuration. Reset behavior cannot be
configured: `--at` must be supplied for that run.

### Wildcard groups

Use a group table to match workspace names or filesystem locations:

```toml
[groups.opencode]
path_globs = ["~/src/opencode-*"]
on_unmatched = "warn"

[groups.workers]
names = ["special-worker"]
name_globs = ["ocw-*"]
```

Group tables support `names` (literal), `name_globs` (registered workspace
names), and `path_globs` (root locations); lists are unioned and each workspace
runs only once. Only exact registered workspace roots **in this JJ repository**
are selected: plain directories, independent JJ repositories, and unrelated Git
checkouts are not adopted or modified.

Patterns are case-sensitive whole-name or whole-path matches; `*` and `?` match
within one path component. Recursive `**`, character classes, braces, backslash
escapes, and `..` components are rejected. A leading `~/` expands the home
directory; other relative paths start at the default workspace regardless of
the caller's location. There is no environment-variable or shell expansion, and
symlinks are not traversed. This group syntax is separate from `copy_ignored`,
which still supports `**`.

`on_unmatched` defaults to `"error"`: unknown names, unmatched expressions, and
non-workspace paths abort before source or target snapshots. A group can opt
into `"warn"` — or `--on-unmatched error|warn` can override all selected groups,
including legacy arrays — to skip unrelated candidates and list them together in
one escaped stderr warning line. Direct positional names are always strict.
Unknown or empty groups and malformed patterns always fail. If no targets
remain, the command fails with `no workspaces selected; nothing reseeded`.
Dry-run shows selected names, paths, and selector origins. Resume requires
literal names and rejects groups or `--on-unmatched` overrides. See
[wildcard groups](docs/reseed.md#wildcard-groups).

## Shell setup

Initialize your shell so `jw switch`, `jw ^`, and `jw -` can change the current
shell directory:

```bash
# bash
eval "$(jw shell init bash)"

# zsh
eval "$(jw shell init zsh)"

# fish
jw shell init fish | source

# elvish
eval (jw shell init elvish | slurp)

# PowerShell
jw shell init powershell | Out-String | Invoke-Expression
```

Without shell initialization, the raw `jw` binary can only print the target path
or status; it cannot change the directory of the parent shell process.
Switching from a subdirectory carries that relative path to a sibling workspace
when it exists; otherwise the destination is the target workspace root.

Zsh users on [Zim](https://zimfw.sh/) can instead install
[rektide/zim-jj-waltz](https://github.com/rektide/zim-jj-waltz): it caches and
precompiles the generated wrapper, keeps completions regenerated on `fpath`, and
follows binary upgrades automatically. Fish has the richest built-in
integration. To generate completions manually:

```bash
jw shell completions fish
jw shell completions zsh
jw shell completions bash
jw shell completions elvish
jw shell completions powershell
```

## Workspace links

If you keep large ignored data in one workspace and want it accessible from
others, define link relationships in the default workspace's `.jwlinks.toml`:

```toml
[[link]]
source = "data"
target = "../ezra-cerpac/data"
required = true
```

When you run `jw switch`, `jw` creates symlinks in the receiving workspace
unless you pass `--no-links`. You can also run `jw links apply` manually from
any workspace; it still uses the default workspace's configuration.
`.jwlinks.local.toml` overrides the shared file for an entry with the same
`source`. Relative targets resolve from the workspace receiving the links, so
the same rule can work across sibling and nested workspaces.

Sources must stay inside the receiving workspace: absolute sources and parent
traversal such as `../outside` are rejected. All rules are checked before `jw`
changes the filesystem, so a later conflict does not leave earlier links behind.
A missing target fails when `required = true`; an optional missing target is
skipped only when the source is absent or is the correct dangling link.

`jw doctor` checks these relationships in every managed workspace. A configured
source is `PASS` when it resolves canonically to its target, including an
ordinary path. A missing source when its target exists, a missing required
target, or a source occupied by a private path or wrong link is `FAIL`. A stale
or missing managed path gets a separate workspace diagnostic and link inspection
is `SKIP`. See the [links reference](skills/jj-waltz/references/links.md) for
the full classifier and remedies.

## Inspecting state

```bash
jw context                     # Git topology and JJ identity, no refresh
jw list                        # compact legacy human list
jw list --format=json          # versioned snapshot (refresh=current by default)
jw list --format=json --refresh=none
jw status @                    # one workspace; tokens @, -, ^, default
jw status feature-api --format=json --refresh=none
jw doctor                      # repository, trunk, metadata, link consistency
jw doctor --format=json        # complete diagnostics; report written before exit 1
```

`list` and `status` JSON use schema version 1 and include frozen JJ operation
and resolved trunk IDs; known missing values stay explicit `null`, and JSON
contains no ANSI escapes. `--refresh` for `list` requires `--format=json`.
`doctor` uses its own schema-versioned report because it must remain valid even
when trunk or metadata is broken; it adds the `workspace-link` diagnostic code,
and consumers should ignore unknown additive codes and fields. `jw status` does
not inspect or report link health; use `jw doctor` for the repository-wide
check.

`jw context [PATH]` reports Git topology and JJ identity without changing
either. Its JSON uses schema version 1 with separate `git` and `jj` objects,
nullable identity fields, and diagnostics. Git-only and non-repository paths
are valid results; discovery does not refresh a working copy. Use it to inspect
an unfamiliar checkout before deciding who owns it — see
[native app checkouts](#native-app-checkouts).

Additional lifecycle commands: `jw path <name>`, `jw root`, `jw current`,
`jw adopt <name> --base <revset>`, `jw repair <name> --base <revset>
(--bookmark <bookmark> | --no-bookmark)`, `jw remove <name>...`, `jw prune`,
and `jw reconcile-external <name>`. See the
[lifecycle reference](skills/jj-waltz/references/lifecycle.md) for their gates
and safeguards.

## Interactive workspace manager

In a terminal, bare `jw` opens the Ratatui workspace manager by default;
`jw ui` selects it explicitly. The top-level user configuration can select the
legacy list or help view instead:

```toml
default_command = "ui" # "ui", "list", or "help"
```

A bare invocation outside a terminal prints help; explicit `jw ui` requires a
terminal. Use arrows or `j`/`k` to move, `Space` to mark rows, `/` to search,
`Enter` to switch, `f` to cycle filters, `n` to create, `d` to preview removal,
and `?` for help.

Bookmark integration and workspace work are separate columns: a merged bookmark
can still have newer workspace work, and "in trunk" means local graph ancestry,
not squash-merge equivalence or GitHub merge status. Removal previews are
deliberate: risky rows are excluded until `r` includes them, files not recorded
by JJ require an explicit delete-or-skip choice, and local-bookmark keep/delete
preference is remembered across repositories. Each target is revalidated before
mutation; no batch rollback or automatic repair is performed.

See the [interactive manager guide](docs/workspace-manager.md) for the full key
table, filters, color semantics, and cleanup safeguards.

## Removing workspaces

When a workspace has a bookmark created by `jw`, `jw remove` asks before
deleting that bookmark; the safe default is to keep it. Scripts can choose
explicitly:

```bash
jw remove --delete-bookmark feature-api
jw remove --keep-bookmark feature-api
```

Removal is planned before mutation, so default/current-workspace checks and
bookmark choices happen before cleanup. On JJ 0.46+ or a build advertising
`workspace remove`, directory removal uses that native command to snapshot the
target's tracked changes before deletion. A stale target fails safely; inspect
it and run `jj workspace update-stale` there before retrying. Older supported
JJ versions use guarded forgetting and directory deletion. `--keep-dir` and
already-missing directories use forgetting; `--keep-dir` preserves files.
Ignored and other files outside JJ's snapshot are deleted with the directory;
use `--keep-dir` when they need preservation.

Native errors are checked for partial progress and never trigger a deletion
fallback. If JJ forgets the workspace but only warns that directory deletion
failed, `jw` reports the remaining path as a partial removal after cleaning
bookmarks and lifecycle metadata. Removing the default workspace is refused;
deleting the current workspace's directory is refused until you switch away
(`--keep-dir` can still forget it). `jw prune` forgets workspaces whose paths
are already missing; it does not share `remove`'s default-workspace guard —
inspect `jw list` first and run it only when every missing entry is intended
for forgetting.

### Colocated workspaces

`jw add feature --colocate` and `jw switch feature --colocate` create a JJ
workspace with a linked Git worktree on JJ 0.46 or another build advertising
that capability. Git-aware tools can use the new workspace's `.git` link.
`--colocate` alone opts out of default CoW. To select colocation in user
configuration:

```toml
[workspace]
colocate = true
```

That setting alone also opts out of default CoW. Omit it or set
`colocate = false` for JJ-only creation; see [mode
precedence](#creation-mode-precedence). Explicit colocation requests on
unsupported JJ versions fail before creation and advise upgrading to JJ 0.46+
or using `--no-colocate`. Non-Git backends also fail before creation.

`jw` records provenance only for Git worktrees it creates. `jw remove` cleans
that workspace's Git registration, and `--keep-dir` retains its files while
removing the Git link. For native removal, `jw` temporarily detaches only its
validated Git link to prevent JJ's repository-wide pruning from deleting
unrelated missing registrations, then cleans only the matching owned
registration. A failed removal restores the link when the workspace is still
registered; unverifiable outcomes retain lifecycle metadata and the owned
registration for inspection. If a process is interrupted after detaching
`.git`, inspect `.jj/jw-detached-git` and the retained metadata before
recovering the link or finishing cleanup.

`jw doctor` validates recorded ownership against live topology and the marker,
and reports invalid provenance as a metadata-consistency error. Lifecycle
metadata writes use schema 3; schemas 1 and 2 remain readable without inferring
ownership, and older `jw` versions refuse schema 3 records. Preserve the
metadata and Git administrative ownership marker for cleanup, and use a current
`jw` version.

### Native app checkouts

Native apps own the Git checkouts they create, including checkouts adopted into
JJ. Keep agent work there, use the actual Git or JJ capability present, and
give each checkout one writer; an orchestrator stays read-only in a worker's
checkout. `jw context` and `jw adopt` do not initialize or adopt a Git worktree
into JJ, and `jw` lifecycle management is optional — no second workspace is
needed to route an agent.

Linked Git topology without valid `jw` creation provenance makes removal,
forgetting (even `--keep-dir`), UI removal, and pruning ineligible. `jw adopt`
persists the external owner for damaged or missing checkout metadata; live
topology is protected without adoption. Let the owning app remove both checkout
and Git registration; then `jw reconcile-external NAME` can explicitly forget
leftover JJ state and remove the external metadata, retaining commits and
bookmarks. Native JJ forgetting can remove Git registration on some builds, and
undo does not restore that side effect. The
[native app skill reference](skills/jj-waltz/references/codex.md) describes the
capability and ownership checks.

## Config

`jw` reads user config from `$XDG_CONFIG_HOME/jj-waltz/config.toml`, or
`~/.config/jj-waltz/config.toml` when `XDG_CONFIG_HOME` is not set.

| Key | Meaning |
| --- | --- |
| `default_command` | `"ui"` (default), `"list"`, or `"help"` for bare `jw` |
| `copy_ignored` (root) | Global ignored-copy patterns; see [ignored-copy policy](#ignored-copy-policy) |
| `[workspace] copy_on_write` | `false` opts out of CoW creation |
| `[workspace] colocate` | `true` selects colocated creation on JJ 0.46+ |
| `[workspace] create_bookmark` | Create a bookmark whenever `switch` creates a workspace |
| `[workspace] bookmark_template` | Bookmark name template; defaults to `{workspace}` |
| `[trunk] revset` | Trunk comparison line; defaults to `trunk()` |

The `{workspace}` token is replaced with the resolved workspace name, so
templates like `wip/{workspace}` are valid. For one command, `jw switch
--bookmark custom-name feature-a` overrides the config, and `jw switch
--no-bookmark feature-a` suppresses configured bookmark creation. Explicit
`--bookmark` is single-workspace only; for batch `add` or `switch`, use
`bookmark_template`.

The trunk revset must resolve to exactly one revision; it is a comparison line
for status, not the creation base. Repository-local reseed/up settings
(`source`, `source_at`, `copy_ignored`, `[groups]`, `[freshen]`) live in the
default workspace's ignored `.jwreseed.local.toml`.

## Semantic contracts

[`CONTEXT.md`](CONTEXT.md) defines workspace, snapshot, trunk, metadata, stack,
publication, integration, cleanup, and refresh vocabulary. The
[architecture notes](docs/architecture.md) define dependency direction, metadata
storage, and JSON compatibility. A defined roadmap concept does not imply that
an unlisted command is available; the feature list and `jw --help` are the
current command surface.

`jw` supports JJ 0.39 and newer within its tested compatibility window. CI pins
the oldest supported release, 0.39.0, and newer compatibility targets, 0.44.0
and 0.45.1, instead of following a moving `latest` label. On JJ 0.45 and newer,
doctor's divergence remedy can use `jj converge --no-interactive`; older
supported versions retain the manual merge-or-abandon guidance.

## Why

Jujutsu workspaces are powerful, but the raw workflow is still more manual than
it needs to be. `jj-waltz` makes switching feel intentional: create or jump in
one command, preserve your current subdirectory, and integrate cleanly with
your shell.

This project is directly inspired by
[Worktrunk](https://github.com/max-sixty/worktrunk), which set a high bar for
ergonomic worktree tooling. Worktrunk is a quality benchmark, not this project's
command specification: `jj-waltz` stays focused on JJ-native workspace lifecycle
and cross-workspace coordination.

The responsibility split is deliberate:

- `jj` owns revision-graph work, bookmarks, fetch/push, and operation-log recovery.
- `jw` owns workspace creation, navigation, links, lifecycle metadata, and safe
  cross-workspace coordination.
- Forge tools such as `gh` and `glab` own pull requests, merge requests, and CI.

Core workspace operations remain offline unless source preparation is explicitly
requested with `jw up`. That requested fetch/preparation must succeed before
distribution. Optional forge information remains nonessential enrichment.

## Upstream distributions

The commands below install **upstream**, not this working checkout. They may not
carry this fork's reseeding additions. Use [the local install](#install-this-fork)
when you want the features described here.

### Homebrew

```bash
brew install EzraCerpac/tap/jj-waltz
```

### Cargo

```bash
cargo install --git https://github.com/EzraCerpac/jj-waltz --locked
```

### Herdr plugin

Install the bundled Herdr UI to create and remove `jw` workspaces from Herdr:

```bash
herdr plugin install EzraCerpac/jj-waltz/plugins/herdr
```

The plugin builds the `jw` binary from the same repository revision and
delegates all workspace paths, links, bookmarks, and removal checks to it. See
[`plugins/herdr`](plugins/herdr) for keybindings and local development.

## Development checks

Run `cargo test --locked --all-targets` for the Rust tests. To exercise the real
PR 9943 adopted-worktree ownership fixtures, set `JW_TEST_PINNED_JJ` to the exact
`ede10cda453017def68e672a5715220ddf10c09b` build when running
`cargo test --locked --test external_owner`. Those fixtures also demonstrate
native forget/undo registration loss and ordinary `jw` creation/removal. On
Unix, build with `cargo build --locked`, then run
`uv run --with pyte tests/terminal_ui.py` for real terminal interaction, shell
switching, terminal restoration, and a 50-workspace responsiveness check. The
terminal checks use disposable repositories. See
[contributing](CONTRIBUTING.md) for formatting, Clippy, and the separate Herdr
workspace checks.

## AI usage note

This project supports AI-assisted development workflows, and portions of its
implementation and documentation may be created with AI assistance. All shipped
behavior is intended to be human-reviewed, tested, and maintained to production
standards.

## Status

`jj-waltz` is under active development. The core workflow is already functional,
and the project is being hardened into a complete standalone CLI with robust
testing, release automation, and public distribution.

## License

MIT
