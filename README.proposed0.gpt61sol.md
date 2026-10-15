---
type: Guide
title: jj-waltz — create, renew, advance
description: Proposed replacement README organized around workspace outcomes, shared CLI controls, and explicit safety boundaries.
resource: /README.proposed0.gpt61sol.md
status: proposed
generated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
sources:
  - resource: "opencode://ses_ec10d60dbffencx7PCZnh1dnjg"
    title: Implemented up integration, assessed CLI consistency, and drafted paired README and agent-skill replacements at the user's request
  - resource: /README.md
    title: Existing installation, creation, shell, configuration, status, ownership, and cleanup guidance
  - resource: /docs/reseed.md
    title: Maintained reseed and up operator contract, copying policy, recovery, and verification limits
  - resource: /src/cli.rs
    title: Actual command arguments and shared report adapter
  - resource: /skills/jj-waltz/references/lifecycle.md
    title: Creation fallback, lifecycle ownership, diagnosis, and deletion safeguards
---

# jj-waltz

**Create a workspace. Renew its copy-on-write sharing. Advance it when you're ready.**

`jw` makes Jujutsu workspaces convenient to create, navigate, inspect, and refresh.
This fork extends the workspace switcher with private dependency copies and
recoverable, in-place CoW refills. It works with stock JJ; it does not fork Jujutsu.

The **primary** is the registered checkout that owns the shared JJ repository;
a **variant** is another workspace in that repository. The **donor** is the source
checkout supplying files; it can be the primary or an explicitly selected workspace.

> **Proposed replacement, not the active README.** This document describes
> implemented behavior, not proposed CLI changes. The paired
> [agent skill proposal](skills/jj-waltz/SKILL.proposed0.gpt61sol.md) turns the same
> model into task-routing guidance. Neither replacement has been activated.

## Choose the outcome

| You want to… | Command | Result |
| --- | --- | --- |
| Create without switching | `jw add feature` | New workspace, empty child of the source's snapshotted `@` |
| Enter a workspace, creating if missing | `jw s feature` | Select existing workspace or create it first |
| Renew sharing without leaving feature work | `jw reseed` | Refill the current variant while keeping its revision |
| Fetch, prepare, and advance | `jw up` | Prepare the donor; start the current variant on a new empty change at that goal |
| Prepare only the donor | `jw up` **at primary** | Fetch/prepare source without changing any target |

**Renew is not advance.** `reseed` never fetches or runs installation/build commands.
`up` does, and target advancement is **not a rebase**. Ordinary `jj` commands still
own deliberate history editing, bookmarks, and remote administration; forge tools
own pull requests and CI.

## Install and start

To install this checkout's fork:

```sh
cargo install --path . --locked
```

For development, build with `cargo build --bin jw` and use `target/debug/jw`.
Changing source files does not upgrade an installed executable. Rust 1.88+ is
required to build; JJ 0.39+ is supported within the tested compatibility window.
Individual capabilities, such as linked Git-worktree creation, require newer JJ.
See [verification and limits](docs/reseed.md#verification-and-limits) rather than
assuming every new feature has been tested on every supported release.

Enable directory changes in your shell:

```sh
eval "$(jw shell init zsh)"     # zsh
eval "$(jw shell init bash)"    # bash
jw shell init fish | source   # fish
```

Use the line for your shell, and add it to that shell's startup configuration.
Elvish and PowerShell are also supported through `jw shell init SHELL`.
Without integration, a child binary cannot change your parent shell's directory.
Completions are available through `jw shell completions SHELL`.

```sh
jw add feature
jw s feature
jw list
jw status @
```

In a terminal, bare `jw` opens the interactive manager by default; outside a
terminal it prints help. `jw ui` selects the manager explicitly. Use `?` inside
the manager for controls and review any removal preview before confirming it.

## CLI at a glance

“Creation only” means an existing switch does **not** reset the workspace, assert
its donor tree, convert its mode, or refill dependencies.

| Behavior / option | `add` | `s` / `switch` | `reseed` | `up` |
| --- | --- | --- | --- | --- |
| Bare command in a variant | Name/path required | Name/path required | Current variant | Current variant |
| Bare command at primary | Name/path required | Name/path required | Targets/group required | Source only |
| Multiple names | Create all | Create missing; select last | Refill selected | Advance selected |
| `--group`, `--on-unmatched` | — | — | Yes | Same |
| Default source | Current checkout | Current checkout for creation | Local `source`, otherwise verified primary | Same |
| `--from` | Select creation source/context | Select source/context | Select donor; retain caller role | Same as reseed |
| `--path` | Exact destination; infer name | Creation destination; infer name | — | — |
| Default target base | Source `@` | Source `@`, creation only | Keep target revision | Exact prepared upstream goal |
| `--at` | Override creation base | Creation only | Explicit reset base | — |
| `--source-at` | Donor tree assertion | Creation only | Donor tree assertion | Upstream advancement goal |
| Ignored-copy controls | Creation only | Creation only | Yes | Same |
| CoW / colocation switches | Creation mode | Creation only | Real CoW required | Real CoW required |
| `--dry-run`, `--resume` | — | — | Yes | Yes |
| `--replace-content` | — | — | Explicit destructive replacement | Same |
| Bookmark controls | `--bookmark`, `--no-bookmark` | Creation only | Existing bookmarks unchanged | Automatic preservation; `--no-freshen-bookmark` opts out |

Creation can start outside a checkout with an explicit `--from`. Reseed and up
must be invoked inside a registered workspace, even with `--from`. For maintenance,
explicit names or groups suppress implicit current selection; nothing ever means
“all workspaces.” Their positional names are literal, not navigation tokens.

## Create and navigate

### Copy-on-write workspaces

New workspaces default to private CoW tracked files and JJ-only metadata. Without
`--at`, creation snapshots the source and starts an empty child of that exact `@`,
including any recorded dirty work or merge. Trunk configuration does not choose
the creation base.

```sh
jw add feature tests
jw s feature
jw s --execute opencode feature
jw ^                         # default workspace
jw -                         # previous workspace
```

`--execute` runs in the selected workspace without changing the parent shell.
Switching carries your relative subdirectory when it exists at the destination.
Use `jw current`, `jw root`, and `jw path NAME` instead of inferring identity from
directory names.

### Create at a chosen destination

```sh
jw add --path ~/src/opencode-fix --from /path/to/opencode
jw s --path ~/src/myfix --from /path/to/opencode
```

Under repository `opencode`, these infer `fix` and `myfix`, respectively. Existing
registrations take precedence. Name-only placement remains `repo.NAME` beside the
owning checkout, not an implicit `~/src` convention. Relative destinations are
resolved from the caller, not from `--from`.

`--source-at REVSET` optionally asserts the donor's snapshotted **tracked tree**;
it does not select the target parent. Use `--at REVSET` for that independent choice.
Creation does not inherit local maintenance `source`, `source_at`, or build commands.

### Creation-mode precedence

| Choice | New workspace |
| --- | --- |
| No mode settings | CoW, JJ-only |
| `--cow` | CoW/JJ-only, overriding configured colocation |
| `--no-cow`, otherwise unset | Ordinary JJ-only tracked checkout |
| `--colocate` | Ordinary linked Git worktree; requires native JJ support, generally 0.46+ |
| `--cow --colocate` | Conflict |

User configuration can set `[workspace].copy_on_write` and `colocate`. Colocation
alone opts out of default CoW; both explicitly true require a resolving CLI choice.
Positive CLI modes take precedence; opt-outs override their corresponding settings.
Opposite flags for the same setting conflict. Existing workspaces are never converted.

CoW **creation** has warned ordinary-checkout fallbacks for unsupported cloning,
Watchman, or rejected copied JJ state. Such a warning is not CoW success.
Selected ignored payload and all reseed/up refills require real CoW: no silent
ordinary-copy fallback is allowed for those phases.

## Renew: keep your feature work

Pause source/target writers, then preview:

```sh
jw reseed --dry-run
jw reseed feature --from /path/to/prepared-source --dry-run
```

Remove `--dry-run` when the plan is right. Reseed snapshots real tracked work,
retains the target revision, reflinks equal paths from the donor, and lets JJ
materialize feature differences. Root, `.jj`, registration, and lifecycle identity
remain in place. Repeating the command renews sharing even if bytes are unchanged.

The donor assertion is CLI `--source-at` → local `source_at` → snapshotted donor
`@`. A supplied assertion compares trees, not commit identities. To reset a worker
deliberately instead of retaining its revision:

```sh
jw reseed worker --source-at 'dev@origin' --at 'dev@origin' --dry-run
```

This starts an empty child; it is not a rebase and does not automatically create
up's preservation bookmark. [Full reseed guide](docs/reseed.md#one-off-use).

## Advance: fetch, prepare, refill

```sh
jw up --dry-run                         # current variant; source only at primary
jw up worker-a worker-b --dry-run        # only these targets
jw up worker-a --source-at 'dev@origin' --dry-run
```

Up selects an upstream **before fetching**: CLI/local goal override, otherwise one
applicable native tracked pair, otherwise one exact remote bookmark at the clean
source base, including untracked refs. Ambiguity requires a one-run `--source-at`.
There is no guessed main/origin, operation-log mining, or new following preference.
Optional local `[freshen].remote` constrains selection, never redirects an explicitly
chosen remote or supplies a default `origin`.
An explicit missing `branch@configured-remote` can be fetched; preview cannot know
its exact goal yet.

Unpublished source work blocks advancement, even beneath an empty scratch change.
The selected remote must fast-forward. Up freezes the validated fetched goal,
prepares its donor child, then creates new empty target changes on that same goal.
It does not carry feature edits into the new changes or move feature bookmarks.

Unpublished target work gets a collision-safe `freshen-<UTC timestamp>` bookmark
before clearing/advancement. An ordinary undescribed empty one-parent target uses
its parent as its work tip; meaningful empty changes and merges retain `@`.
Publication is ancestry, not equal bytes. `--no-freshen-bookmark` suppresses only
that automatic bookmark; JJ history and source-work checks remain.

### Local preparation and groups

Keep `.jwreseed.local.toml` ignored in the verified primary checkout:

```toml
# Optional source/source_at overrides; neither is required for native inference.
copy_ignored = ["**/node_modules", "**/dist/"]

[groups]
workers = ["worker-a", "worker-b"]

[freshen]
commands = [
  ["bun", "install", "--frozen-lockfile"],
  ["bun", "run", "build"],
]
```

Use `jw up --group workers --dry-run` or `jw reseed --group workers --dry-run`.
Groups may also use [name/path selectors](docs/reseed.md#wildcard-groups) and
explicit warn-and-skip selection policy; target safety failures remain errors.

Commands run sequentially as argv in donor cwd, without an implicit shell. Use
either `command = [...]` or `commands = [[...], ...]`, never both, even when empty.
No commands means no guessed installer/build. Existing ignored payload may still
be borrowed, but is not freshly built. There are no global build commands; the
local table alone never activates preparation. This executes trusted programs,
not a sandbox. [Full up contract](docs/reseed.md#freshen-the-source).

## Private dependencies on creation

The same copying controls apply to add/s creation, reseed, and up:

```sh
jw add feature --copy-ignored node_modules
jw reseed --copy-ignored 'packages/*/node_modules' --dry-run
jw up --no-copy-ignored --dry-run
```

Root user configuration in `$XDG_CONFIG_HOME/jj-waltz/config.toml` (otherwise
`~/.config/jj-waltz/config.toml`) may declare:

```toml
copy_ignored = ["**/target", "**/node_modules", "**/dist/", "**/build/"]
```

Lists replace: **CLI → primary-local → global → none**. Missing local policy
inherits; local `[]` stops inheritance. `--no-copy-ignored` disables even an explicit
CLI list. Eligible ignored roots are replaced with private reflinks—not hard links
or shared-directory symlinks. Other target contents remain unless explicitly erased.

Patterns support `*`, `?`, `**`, and trailing `/`; quote them against shell expansion.
Nonignored source matches are skipped/reported, missing matches succeed, and nested
selected roots collapse. Selected roots cannot overlap tracked files or escape
through directory symlinks. Target ignore/confinement checks still apply. Actual
payload creation inside its donor is refused. `--no-cow` affects tracked creation,
not the independently required real-CoW payload phase. Existing switches never refill.

Shared `.jwlinks.toml` links are a different policy from private copies. Avoid
placing a shared-directory link over a private payload; see the
[link guide](skills/jj-waltz/references/links.md) and
[ignored-copy safety rules](docs/reseed.md#ignored-data).

## Preview, failure, and recovery

Reseed/up require full, conflict-free, eligible JJ-only targets, verified ownership,
and real reflinks. A normal colocated donor is allowed; external/app-owned or linked
Git donors cannot be freshened. Pause agents, builds, editors, monitors, and other
writers first. JW's advisory lock does not lock arbitrary JJ processes.

**Preview is not entirely snapshot-free:** normal snapshots can preserve real work.
It does not fetch, create new changes/bookmarks, run preparation, or refill files.
`--replace-content` explicitly permits deleting unrecorded target contents,
including `.env`, except `.jj`; it is not required for ordinary refills.

After interruption, do not snapshot a half-filled target or run repository-wide
undo. Resume the original workflow:

```sh
jw up worker-a worker-b --resume  # original order until source prep freezes plans
jw up worker-a --resume           # individual/subset recovery after plan freeze
jw up --resume                   # current target; at primary, source-only batch
jw reseed feature --resume       # ordinary reseed or legacy preparation
```

Frozen recovery accepts no new donor/revision/copy/replacement/bookmark overrides.
It does not refetch or rebuild a frozen goal, reallocate preservation bookmarks,
or adopt changed payload bytes. Restore the recorded state when validation refuses.
Completed commands are skipped; an interrupted in-flight command may retry after
partial side effects. Arbitrary programs are **not exactly-once**.

`reseed --freshen` is removed. Old v1/v2 preparation still uses `reseed --resume`
with its original keep-target/explicit-base intent. `[prepare]` remains a warned
local alias; rename it only after pending recovery completes because policy bytes
are frozen. [Recovery details](docs/reseed.md#safety-and-recovery).

## Inspect, manage, and clean up deliberately

```sh
jw context . --format=json
jw list
jw list --format=json --refresh=none
jw status feature --refresh=none
jw doctor
jw remove feature --keep-dir --keep-bookmark
```

`list` keeps its compact human format; JSON opts into a versioned snapshot. Normal
status refreshes current by default; `--refresh=none` avoids that refresh. Doctor
reports repository, metadata, trunk, and link problems without repairing them.
Trunk defaults to `trunk()` and can be set with `[trunk].revset` in user config;
it informs status, not creation or up inference. Bare-command behavior is configured
with root `default_command = "ui"`, `"list"`, or `"help"`.

Adoption is optional metadata recording, not a checkout conversion or transfer of
ownership. Repair changes recorded base/bookmark association, not revisions.
Native app Git checkouts remain app-owned even after JJ adoption. Give each checkout
one writer; a separate orchestration session stays read-only there.

Removal without `--keep-dir` deletes ignored/private files as well as recorded
files. JJ's operation log is not a directory backup. Bookmarks are kept unless
deletion is explicitly selected/confirmed. Prune forgets missing registrations;
run it only when those removals are intended. Externally owned checkouts require
their owning app's cleanup and explicit reconciliation, not a native-command bypass.
Read [lifecycle safeguards](skills/jj-waltz/references/lifecycle.md) before cleanup.

## Further reading and development

- [Operator guide](docs/reseed.md): detailed source, payload, groups, and recovery behavior.
- [Architecture](docs/architecture.md): module boundaries, journals, ownership, and verification evidence.
- [Working concepts](CONTEXT.md): identity, snapshots, lifecycle metadata, and publication vocabulary.
- [Changelog](CHANGELOG.md), [contributing](CONTRIBUTING.md), and [design evidence](design/workspace-reseed/README.md).

For development, use `cargo nextest run`, `cargo clippy --all-targets --all-features`,
and `cargo fmt --all -- --check`. Real-CoW cases need disposable workspaces on a
reflink-capable filesystem with `JW_TEST_REQUIRE_COW=1`; a skipped unsupported-CoW
fixture is not evidence of reflink delivery. The project is actively developed
and uses AI-assisted contributions subject to review and verification.

## License

MIT — see [LICENSE](LICENSE).
