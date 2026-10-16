---
type: Guide
title: jj-waltz — create, renew, advance
description: The canonical guide to creating and navigating JJ workspaces, renewing private CoW sharing, and advancing onto prepared upstream work.
resource: /README.md
status: maintained
updated:
  by: model:openai/gpt-6-astra
  at: 2026-10-15
sources:
  - resource: "opencode://ses_ebf068824ffeRuaEQy26Krx5u6"
    title: Synthesized the existing guide and independent Sol and G takes into the canonical README and skill
  - resource: /README.md
    revision: fceba759ac391599f85137df52f4fc9eb08683d5
    title: Original coverage, shell setup, configuration, ownership, and manager guidance
  - resource: /README.proposed0.gpt61sol.md
    title: Outcome-led organization and shared command controls
  - resource: /README.proposed0.glm53m.md
    title: Independent command comparison, policy consolidation, and implementation-checked coverage
  - resource: /docs/reseed.md
    title: Maintained reseed, up, copying, and recovery contract
  - resource: /skills/jj-waltz/references/lifecycle.md
    title: Lifecycle operations and ownership boundaries
  - resource: /CONTRIBUTING.md
    title: Current compatibility pins and development checks
  - resource: "opencode://ses_ec10d60dbffencx7PCZnh1dnjg"
    title: Integrated public up, native inference, source-only preparation, target preservation, and legacy resume migration
  - resource: "opencode://ses_ec1394461ffeEXBHM582vTs9VM"
    title: Fixed nested payload creation preflight and implemented explicit creation assertions, optional reseed assertions, current-variant selection, and frozen recovery checks
  - resource: "opencode://ses_ec14d159effeiOep7U7f2iNQ7E"
    title: Implemented and verified add/s private ignored payloads, primary-owned copy policy, strict CoW and scoped failure cleanup
  - resource: "opencode://ses_ec1a8d43bffe02bXuEoA7hEW3l"
    title: Aligned local command/recovery and shared ignored-copy guidance with implemented policy, eligible selection, and current exposure boundaries
  - resource: "opencode://ses_ec206881bffenBcwNsQxlvfx3J"
    title: Implemented and verified add/s destinations, source selection, CoW defaults, and explicit mode precedence; aligned operator guidance
  - resource: "opencode://ses_ec37b01d2ffe9ytO0jRsdmoyXA"
    title: Implemented reseeding and explicit donor freshening; clarified carried features, installation, and source defaults
  - resource: "opencode://ses_ec37a1a86ffeYXsS5j3A7Jng0t"
    title: Added wildcard reseed groups with opt-in aggregate unmatched warnings
  - resource: /design/workspace-reseed/reseed0.gpt6a.md
    title: In-place reseeding behavior and safety contract
---

# jj-waltz

**Create a workspace. Renew its copy-on-write sharing. Advance onto prepared upstream work.**

`jw` manages named [Jujutsu](https://github.com/jj-vcs/jj) workspaces: create,
switch, inspect, and refill them in place. Copy-on-write (CoW) files share storage
until either copy changes, so workspaces can have private files and dependencies
without paying for every byte twice.

This working fork of [EzraCerpac/jj-waltz](https://github.com/EzraCerpac/jj-waltz)
extends the workspace switcher with chosen destinations, private dependency copies,
in-place reseeding, and integrated upstream advancement. It works with stock JJ.

## Install and start

Install this checkout's fork:

```sh
cargo install --path . --locked
```

For development, build with `cargo build --bin jw` and run `target/debug/jw`.
Source changes do not update an installed executable. Building requires Rust 1.88+;
supported JJ starts at 0.39. CI pins 0.39.0, 0.44.0, 0.45.1, and 0.46.0, with
newer capabilities gated separately. See [verification limits](docs/reseed.md#verification-and-limits).

### Shell setup

Use the line for your shell and add it to the shell's startup configuration:

```sh
eval "$(jw shell init zsh)"                  # zsh
eval "$(jw shell init bash)"                 # bash
jw shell init fish | source                 # fish
eval (jw shell init elvish | slurp)          # elvish
jw shell init powershell | Out-String | Invoke-Expression  # PowerShell
```

Restart or re-source the shell after setup. The wrapper lets `jw s`, `jw ^`,
`jw -`, and the interactive manager change your current directory; the binary
alone can only report a destination. Generate completions with
`jw shell completions SHELL`. Zim users can use
[rektide/zim-jj-waltz](https://github.com/rektide/zim-jj-waltz) for cached Zsh
integration and automatically refreshed completions.

```sh
jw add feature tests          # create siblings repo.feature and repo.tests
jw s feature                  # enter feature, creating it if missing
jw list
jw status @
jw reseed --dry-run            # preview renewing this variant, keeping its work
```

In a terminal, bare `jw` opens the interactive manager; `jw ui` selects it
explicitly. Outside a terminal, bare `jw` prints help.

## Choose the outcome

The **primary** is the registered checkout owning the shared JJ repository. A
**variant** is another workspace in that repository. The **donor** supplies files;
it can be the primary or an explicitly selected workspace.

| Behavior / control | `add` / `s` (`switch`) | `reseed` | `up` |
| --- | --- | --- | --- |
| Outcome | Create; `s` enters existing or creates missing | Renew CoW sharing in place | Fetch, prepare donor, advance and refill |
| Target history | Empty child of source `@`, or `--at` | Keep snapshotted work, or reset via `--at` | New empty change at the fetched goal |
| Bare command in a variant | Name/path required | Current variant | Current variant |
| Bare command at primary | Name/path required | Names/group required | Donor only |
| Multiple names | Create all; `s` enters last | Refill selected | Advance selected |
| `--group`, `--on-unmatched` | — | Supported | Supported |
| Default donor | Current checkout | Local `source` → verified primary | Same as reseed |
| `--from` | Creation source and command context | Donor only; caller role stays the same | Same as reseed |
| `--path` | One exact destination; name can be inferred | — | — |
| `--source-at` | Donor tracked-tree assertion | Donor tracked-tree assertion | Upstream advancement goal |
| `--at` | Explicit creation base | Explicit reset base | —; always prepared goal |
| Fetch / preparation commands | Never | Never | One pinned upstream / local argv |
| `--copy-ignored`, `--no-copy-ignored` | New-workspace payload | Refill payload | Refill payload |
| `--dry-run`, `--resume` | — | Supported | Supported |
| `--replace-content` | — | Discard unrecorded target contents | Same as reseed |
| Bookmarks | Optional creation bookmark | Existing bookmarks stay | Preserve work left behind; feature bookmarks stay |

**Renew keeps work checked out; advance starts fresh.** `up` is not a rebase.
Creation flags never reset or refill an existing workspace selected by `s`.
Explicit names/groups suppress the implicit current target; empty selection never
means all workspaces. Maintenance names are literal, not navigation shortcuts.

Creation with `--from` can start outside a checkout. Reseed/up require a registered
caller even with `--from`. Ordinary `jj` commands own deliberate history editing,
bookmarks, and remote administration; forge tools own pull requests and CI.

## Create and navigate

### Create at a chosen destination

```sh
jw add --path ~/src/opencode-fix --from /path/to/opencode
jw s --path ~/src/myfix --from /path/to/opencode
```

Under repository `opencode`, these infer `fix` and `myfix`, respectively. You can
also supply an explicit name. Existing registrations take precedence: `s` enters
the registered workspace; `add` reports it already exists. Conflicting registrations
and occupied new destinations are errors. Relative paths resolve from the caller.

Name-only creation puts `repo.NAME` beside the owning checkout. Inspect destinations
with `jw path NAME`; use `jw current` and `jw root` for identity rather than directory
basenames. Navigation accepts `@` (current), `-` (previous), and `^` or `default`
(default workspace). Switching preserves your relative subdirectory when it exists
at the destination.

```sh
jw ^
jw -
jw s --execute opencode feature  # run there without changing the parent shell
```

### Copy-on-write workspaces

Creation defaults to CoW tracked files and JJ-only metadata, even from a colocated
source. APFS uses `clonefile`; Linux filesystems such as Btrfs and XFS use reflinks.
The checkout retains the source's sparse patterns.

Without `--at`, creation snapshots the source and starts an empty child of that
exact `@`, including dirty tracked work or a merge. `--at REVSET` selects one exact
base instead. Optional CLI `--source-at REVSET` asserts the snapshotted donor's
**tracked tree**, independently of the target parent; equal trees need not be the
same commit. Creation does not inherit local maintenance `source`, `source_at`, or
preparation commands. Trunk configuration does not choose the creation base.

Unsupported cloning, Watchman, or rejected copied JJ state cause a warning and
ordinary JJ checkout fallback during **tracked creation**. That is not CoW success.
Private ignored payloads and reseed/up refills require real CoW and have no ordinary
copy fallback.

### Creation-mode precedence

| Selection | New workspace mode |
| --- | --- |
| No mode settings | CoW, JJ-only |
| `--cow` | CoW/JJ-only, overriding configured colocation |
| `--colocate` | Ordinary linked Git worktree, overriding configured CoW; requires JJ capability, generally 0.46+ |
| `--no-cow`, otherwise unset | Ordinary JJ-only checkout |
| Config `colocate = true` alone | Ordinary colocated checkout |
| Config `copy_on_write = true` and `colocate = true` | Conflict unless resolved by CLI |
| `--cow --colocate` | Conflict |

Mode settings live under `[workspace]` in user configuration. Positive CLI modes
take precedence; opt-outs override their corresponding settings. Thus `--no-cow`
with configured colocation keeps colocation, while `--no-colocate` with configured
CoW keeps CoW. Opposite flags for the same setting conflict. Existing workspaces
are never converted by switching.

## Renew: keep your feature work

Pause donor/target writers, then preview:

```sh
jw reseed --dry-run
jw reseed feature --from /path/to/prepared-source --dry-run
```

Remove `--dry-run` when the plan is right. Reseed snapshots real work, retains the
target revision, reflinks equal paths from the donor, and lets JJ materialize feature
differences. The root, `.jj`, registration, bookmarks, and lifecycle identity remain.
Repeating it renews sharing even when bytes are unchanged. It never fetches or builds.

Donor selection is CLI `--from` → local `source` → verified primary. The optional
tree assertion is CLI `--source-at` → local `source_at`; otherwise borrowing uses
the donor's freshly snapshotted `@`. To reset a reusable worker deliberately:

```sh
jw reseed worker --source-at 'dev@origin' --at 'dev@origin' --dry-run
```

This selects an empty child of the base rather than carrying feature edits forward.
Tracked work remains in JJ history, but reseed does not create up's preservation
bookmark. See the [operator guide](docs/reseed.md#one-off-use).

## Advance: fetch, prepare, refill

```sh
jw up --dry-run                    # current variant; donor only at primary
jw up worker-a worker-b --dry-run   # only these targets
jw up worker-a --source-at 'dev@origin' --dry-run
```

Up pins one upstream **before fetching**: CLI/local source goal, otherwise a unique
applicable native tracked pair, otherwise a unique remote bookmark exactly at the
clean donor base, including untracked refs. Ambiguity calls for a one-run
`--source-at`; there is no guessed main/origin. Optional `[freshen].remote`
constrains selection. An explicit missing `branch@configured-remote` can be fetched,
though preview cannot yet know its exact goal.

Unpublished donor work blocks advancement, even beneath an empty scratch change.
The selected remote must fast-forward. Up freezes the fetched goal, prepares the
donor, then puts targets on new empty changes at that same goal. Existing feature
bookmarks stay where they are. Work not contained in the fetched goal gets a
collision-safe `freshen-<UTC timestamp>` bookmark before target clearing or advancement;
`--no-freshen-bookmark` opts out of that bookmark only.

To prepare newer dependencies **while keeping a feature revision**, run source-only
`jw up` at the primary, then `jw reseed feature`. See the
[full up contract](docs/reseed.md#freshen-the-source).

### Local preparation and groups

Keep `.jwreseed.local.toml` ignored in the verified primary checkout:

```toml
# Optional source and source_at overrides can go here.
copy_ignored = ["**/node_modules", "**/dist/"]

[groups]
workers = ["worker-a", "worker-b"]

[freshen]
commands = [
  ["bun", "install", "--frozen-lockfile"],
  ["bun", "run", "build"],
]
```

Commands run sequentially as argv in the donor root, without an implicit shell.
Each must leave tracked files, including lockfiles, unchanged. Use `command = [...]`
for one command or `commands = [[...], ...]`, never both, even when empty. No commands
means no guessed installer/build: selected existing payload can still be borrowed,
but is not freshly built. Preparation commands are repository-local, run only via
`up`, and execute trusted programs rather than a sandbox.

Use `jw up --group workers --dry-run` or `jw reseed --group workers --dry-run`.
Groups can also select registered name/path patterns and opt into warn-and-skip for
unrelated candidates. Direct names and target safety failures stay strict. Group
patterns differ from ignored-copy patterns; see [wildcard groups](docs/reseed.md#wildcard-groups).

## Private dependencies and shared copying policy

### Private dependencies on creation

```sh
jw add feature --copy-ignored node_modules
jw reseed --copy-ignored 'packages/*/node_modules' --dry-run
jw up --no-copy-ignored --dry-run
```

The same controls apply to new workspaces, reseed, and up. Selected ignored roots
are **replaced** with private reflinks, not hard links or shared-directory symlinks.
Other target contents stay unless explicitly erased. Existing switches never refill.

Lists replace in **CLI → primary-local → global → none** order. Missing local policy
inherits; local `copy_ignored = []` stops inheritance. `--no-copy-ignored` disables
even an explicit CLI list. Creation reads only the local copy list, not maintenance
source or preparation settings.

Quote patterns: `*`, `?`, `**`, and trailing `/` are supported. Missing matches
succeed; nonignored source matches are skipped/reported; nested selections copy once.
Selected roots must be ignored on both sides, without tracked overlap or escapes
through directory symlinks. Creating actual payload inside its donor is refused;
choose an external destination. `--no-cow` and `--colocate` control tracked creation,
not the independently required real-CoW payload phase. Pause donor writers first.

Dependencies reflect the donor's installed state. If feature manifests differ,
run that project's frozen install in the feature. Symlinks are copied as links;
source-bound absolute links are reported, not rewritten. See
[ignored-data rules](docs/reseed.md#ignored-data).

### Workspace links

For deliberate sharing, define symlinks in the default workspace's `.jwlinks.toml`:

```toml
[[link]]
source = "data"
target = "../repo/data"
required = true
```

`add` and `switch` apply links; `jw links apply` applies them manually.
`.jwlinks.local.toml` overrides entries with the same `source`. Sources stay inside
the receiving workspace; relative targets resolve from its root. `--no-links`
skips application for the operation. Private payloads are installed before links;
an overlapping shared link is an error. The [links reference](skills/jj-waltz/references/links.md)
explains local overrides, conflicts, and `jw doctor` link checks.

## Preview and recover

Refills require full, conflict-free eligible JJ-only targets, verified ownership,
and real reflinks. A normal colocated donor is allowed; linked Git/app-owned donors
cannot be freshened. Pause agents, builds, editors, monitors, and other writers;
JW's advisory lock does not lock arbitrary JJ processes.

Dry-run can snapshot real work to preserve it. It does not fetch, create new changes
or bookmarks, run preparation, or refill files. `--replace-content` explicitly
permits deleting unrecorded target contents, including `.env`, except `.jj`.

After interruption, **resume before snapshotting a half-filled target**:

```sh
jw up worker-a worker-b --resume  # original order until target plans freeze
jw up worker-a --resume           # individual/subset recovery after plan freeze
jw up --resume                   # current target; source-only at primary
jw reseed feature --resume       # ordinary reseed or legacy preparation
```

Recovery uses recorded intent without new donor, revision, copy, replacement, group,
or bookmark overrides. A frozen goal is not refetched/rebuilt; changed recorded
source/payload state must be restored rather than adopted. Completed command steps
are skipped; interrupted in-flight commands may retry after partial side effects.
Arbitrary commands are not exactly-once. Partial batch progress is reported separately.

`reseed --freshen` is removed. Legacy v1/v2 batches retain their original history
intent through `reseed --resume`. `[prepare]` remains a warned alias for `[freshen]`;
rename it after pending recovery finishes, since policy bytes are frozen. See
[recovery details](docs/reseed.md#safety-and-recovery).

## Inspect and manage

```sh
jw context . --format=json
jw list
jw list --format=json --refresh=none
jw status feature --refresh=none
jw doctor
```

`context` separates Git topology and JJ identity without refreshing. Human `list`
stays compact; JSON opts into a versioned snapshot. Status and JSON list refresh
current by default; `--refresh=none` avoids that refresh. List's `--refresh` requires
JSON. Doctor reports repository, metadata, trunk, and link problems without repair;
status does not check links. See [lifecycle guidance](skills/jj-waltz/references/lifecycle.md).

The [interactive manager](docs/workspace-manager.md) provides navigation, filters,
creation, health reports, and reviewed cleanup. Use `?` for controls. Its “in trunk”
status means graph ancestry, not squash-merge equivalence or forge merge status.

`jw adopt NAME --base REVSET` records lifecycle metadata for an existing JJ workspace;
`jw repair NAME --base REVSET (--bookmark NAME | --no-bookmark)` corrects an existing
record. Neither moves revisions. Native apps retain ownership of their checkouts,
including after JJ adoption. Give each checkout one writer; an orchestrator inspects
workers read-only. See [app ownership](skills/jj-waltz/references/codex.md).

### Removing workspaces

```sh
jw remove feature --keep-dir
```

Without `--keep-dir`, removal deletes ignored/private files too; JJ's operation log
is not a directory backup. Associated bookmarks are kept by default; only an
explicit `--delete-bookmark` deletes them, and removal never prompts.
For jw-created colocated workspaces, keeping the directory still disconnects its
Git link. Native removal is used when supported; failures may leave partial progress.
Prune forgets missing registrations and should run only when every removal is intended.
Externally owned checkouts require their app's cleanup, then explicit
`jw reconcile-external NAME` for leftover JJ state. Read the
[cleanup safeguards](skills/jj-waltz/references/lifecycle.md#remove) before removal.

## User configuration

User settings live at `$XDG_CONFIG_HOME/jj-waltz/config.toml`, otherwise
`~/.config/jj-waltz/config.toml`. For example:

```toml
# Root keys precede tables. Ignored copying is opt-in.
default_command = "ui" # "ui", "list", or "help"
copy_ignored = ["**/target", "**/node_modules", "**/dist/", "**/build/"]

[workspace]
create_bookmark = true
bookmark_template = "wip/{workspace}"

[trunk]
revset = "trunk()"
```

Bookmark templates default to `{workspace}`. Explicit `--bookmark NAME` is
single-workspace only; batch creation can use the template. `--no-bookmark`
suppresses creation. The trunk revset must resolve to one revision and informs
status, not creation or up inference. Creation modes are
[described above](#creation-mode-precedence); local donor/preparation/group settings
belong in the primary's `.jwreseed.local.toml`.

## Further reading and development

- [Agent skill](skills/jj-waltz/SKILL.md): task routing, execution, and verification.
- [Reseed/up operator guide](docs/reseed.md): detailed payload, group, and recovery contracts.
- [Architecture](docs/architecture.md) and [working concepts](CONTEXT.md): module and semantic boundaries.
- [Changelog](CHANGELOG.md) and [design evidence](design/workspace-reseed/README.md): carried changes and their rationale.
- [Herdr plugin](plugins/herdr/README.md): the bundled UI integration.
- [Contributing](CONTRIBUTING.md): root and separate plugin checks; [terminal verification](docs/workspace-manager.md#development-verification).

Use `cargo nextest run`, `cargo clippy --locked --all-targets --all-features -- -D warnings`,
and `cargo fmt --all --check` for development. Real-CoW fixtures need a reflink-capable
filesystem with `JW_TEST_REQUIRE_COW=1`; a skipped unsupported-CoW fixture is not
proof of sharing. Adopted-worktree ownership fixtures additionally use
`JW_TEST_PINNED_JJ` pointing to build `ede10cda453017def68e672a5715220ddf10c09b`
with `cargo test --locked --test external_owner`.

Inspired by [Worktrunk](https://github.com/max-sixty/worktrunk). The project is actively
developed and supports AI-assisted contributions subject to review and verification.

## License

MIT — see [LICENSE](LICENSE).
