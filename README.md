---
type: Guide
title: jj-waltz — create, renew, prepare
description: The canonical guide to creating and navigating JJ workspaces, renewing private CoW sharing, and preparing upstream sources to an exact commit.
resource: /README.md
status: maintained
updated:
  by: model:openai/gpt-6.1-sol#xhigh
  at: 2026-10-17
sources:
  - resource: "opencode://ses_eb75a514cffe7jUOMWNZAIm3fX"
    title: Deleted prepare and reseed journals, receipts, and resume machinery in favor of explicit out-of-band bookmark recovery
  - resource: "opencode://ses_eb867bfdaffefMVLpyaPUadbSp"
    title: Added the built-in all group with donor-aware membership, aggregate blockers, and literal recovery guidance
  - resource: "opencode://ses_eb8801331ffeirQrF4GMFCxJHf"
    title: Implemented aggregate readiness and default-on named bookmark coverage for fresh reseeds, explicit opt-out, and frozen/legacy recovery semantics
  - resource: "opencode://ses_eb899cdcdffecJnB7J1dSGs4ci"
    title: Split the public up command into source-only prepare, restored reseed-only worker semantics, and rewired maintained docs to the prepared-commit workflow
  - resource: /design/up-performance/prepare0.gpt6a.md
    title: Current command-split decision record — prepare/reseed separation and the performance-first slice
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

**Create a workspace. Renew its copy-on-write sharing. Prepare upstream sources to an exact commit.**

`jw` manages named [Jujutsu](https://github.com/jj-vcs/jj) workspaces: create,
switch, inspect, refill them in place, and prepare upstream sources. Copy-on-write
(CoW) files share storage
until either copy changes, so workspaces can have private files and dependencies
without paying for every byte twice.

This working fork of [EzraCerpac/jj-waltz](https://github.com/EzraCerpac/jj-waltz)
extends the workspace switcher with chosen destinations, private dependency copies,
in-place reseeding, and source-only upstream preparation. It works with stock JJ.

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

| Behavior / control | `add` / `s` (`switch`) | `reseed` | `prepare` |
| --- | --- | --- | --- |
| Outcome | Create; `s` enters existing or creates missing | Renew CoW sharing in place | Select/fetch one upstream, prepare the donor, report the exact commit |
| Target workspaces | Created or entered | Refilled in place; names/groups select | None — never selects, refills, or advances workers |
| Target history | Empty child of source `@`, or `--at` | Keep snapshotted work, or reset via `--at` | — |
| Bare command in a variant | Name/path required | Current variant | Source-only; same meaning anywhere |
| Bare command at primary | Name/path required | Names/group required | Source-only |
| Multiple names | Create all; `s` enters last | Refill selected | — |
| `--group`, `--on-unmatched` | — | Supported | — |
| Default source | Current checkout | Local `source` → verified primary | Same as reseed |
| `--from` | Creation source and command context | Donor only; caller role stays the same | Source root |
| `--path` | One exact destination; name can be inferred | — | — |
| `--source-at` | Donor tracked-tree assertion | Donor tracked-tree assertion | Upstream goal |
| `--at` | Explicit creation base | Explicit reset base | — |
| Fetch / preparation commands | Never | Never | One pinned upstream / local argv |
| `--copy-ignored`, `--no-copy-ignored` | New-workspace payload | Refill payload | Verify selected payload |
| `--dry-run` | — | Supported | Supported |
| `--format=json` | — | — | Machine-readable report |
| `--replace-content` | — | Discard unrecorded target contents | — |
| Bookmarks | Optional creation bookmark | Existing bookmarks stay | No preservation bookmarks; fetch may update refs |

**Renew keeps work checked out; `--at` resets a worker deliberately.** `prepare`
never selects or refills a worker. Creation flags never reset or refill an existing
workspace selected by `s`. Explicit names/groups suppress the implicit current
reseed target; empty selection never means all workspaces. Maintenance names are
literal, not navigation shortcuts. Feature rebasing remains ordinary `jj` work:
pointing a worker at a new baseline does not rebase its feature commits.

Creation with `--from` can start outside a checkout. Reseed requires a registered
caller even with `--from`; prepare accepts any registered workspace of the
repository, always preparing only its source. Ordinary `jj` commands own
deliberate history editing, bookmarks, and remote administration; forge tools own
pull requests and CI.

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
Private ignored payloads, reseed refills, and prepare payload verification require
real CoW and have no ordinary
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

Fresh reseeds require **bookmarked work by default**, in both keep and `--at`
modes. A present, unconflicted local or remote bookmark must point at the
snapshotted work tip or a descendant; a bookmark on an older base does not cover
new edits. Dry-run reports the covering name and tip, or blocks the batch.
`jw` never creates or moves a bookmark for this guard. Deliberately name your work
with JJ, or explicitly bypass only this requirement:

```sh
jj bookmark create my-feature -r @       # run inside the feature workspace
jw reseed --dry-run
jw reseed --allow-unbookmarked --dry-run  # explicit exception; other guards remain
```

Dry-run can record normal JJ snapshots but does not refill/reset/delete files.
See [readiness and bookmark coverage](docs/reseed.md#check-readiness-before-refilling).

Donor selection is CLI `--from` → local `source` → verified primary. The optional
tree assertion is CLI `--source-at` → local `source_at`; otherwise borrowing uses
the donor's freshly snapshotted `@`. To reset a reusable worker deliberately:

```sh
jw reseed worker --source-at 'dev@origin' --at 'dev@origin' --dry-run
```

This selects an empty child of the base rather than carrying feature edits forward.
Tracked work remains in JJ history, and reseed never creates preservation bookmarks.
See the [operator guide](docs/reseed.md#one-off-use).

## Prepare the source

```sh
jw prepare --dry-run
jw prepare --source-at 'dev@origin'
jw prepare --format=json
```

`prepare` selects and fetches one pinned upstream, prepares the donor, and reports
the **exact frozen commit**. It is source-only: it never chooses targets, refills
files, or advances workers, and it has the same meaning from any registered
workspace. Dry-run explains the prospective fetch and preparation commands without
fetching or building.

Upstream selection pins one upstream **before fetching**: CLI/local source goal,
otherwise a unique applicable native tracked pair, otherwise a unique remote
bookmark exactly at the clean donor base, including untracked refs. Ambiguity
calls for a one-run `--source-at`; there is no guessed main/origin. Optional
`[freshen].remote` constrains selection. Unpublished donor work blocks
preparation, even beneath an empty scratch change, and the selected remote must
fast-forward.

The reported commit is frozen. Reuse it directly instead of preparing again:

```sh
jw prepare                                       # prints the exact prepared commit
jw reseed worker-a --at <commit> --copy-ignored node_modules   # reset a worker onto it
jw reseed feature                                # or keep a feature tree
```

Feature rebasing stays an ordinary `jj` operation. `--at` starts an empty child of
the chosen base; it does not rebase the workspace's old feature commits, and
reseed without `--at` keeps them checked out. See the
[full prepare contract](docs/reseed.md#prepare-the-source).

### Local preparation and groups

`jw reseed --group all --dry-run` selects all registered workers except the
verified primary and chosen donor, with no configuration required. Unsafe or
unbookmarked members remain BLOCKED; one blocker prevents every refill. `all`
is reserved against local group definitions. See [all registered workers](docs/reseed.md#all-registered-workers)
for donor selection, snapshot precautions, and literal-name recovery.

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
Each must leave tracked files, including lockfiles, unchanged and tolerate retry.
After failure, fix the source and run prepare again; every command may run again.
Use `command = [...]`
for one command or `commands = [[...], ...]`, never both, even when empty. No commands
means no guessed installer/build: selected existing payload can still be borrowed,
but is not freshly built. Preparation commands are repository-local, run only via
`prepare`, and execute trusted programs rather than a sandbox.

Use `jw reseed --group workers --dry-run` to select refill targets; `prepare` has
no groups because it has no targets. Groups can also select registered name/path
patterns and opt into warn-and-skip for
unrelated candidates. Direct names and target safety failures stay strict. Group
patterns differ from ignored-copy patterns; see [wildcard groups](docs/reseed.md#wildcard-groups).

## Private dependencies and shared copying policy

### Private dependencies on creation

```sh
jw add feature --copy-ignored node_modules
jw reseed --copy-ignored 'packages/*/node_modules' --dry-run
jw prepare --no-copy-ignored --dry-run
```

The same controls apply to new workspaces, reseed, and prepare. Selected ignored roots
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
cannot be prepared. Pause agents, builds, editors, monitors, and other writers;
JW's advisory lock does not lock arbitrary JJ processes.

Dry-run can snapshot real work to preserve it. It does not fetch, create new changes
or bookmarks, run preparation, or refill files. `--replace-content` explicitly
permits deleting unrecorded target contents, including `.env`, except `.jj`.

Verifying an intact workspace is a bounded JJ tracked-tree check: it may snapshot
(and record a rejected edit) before reporting a mismatch, and it never tracks
additional files; automatic stale-tree updates are disabled. Sparse or mid-refill
targets are refused before snapshotting; `jw` never repairs them.

After an interrupted refill, recover **out of band** in the affected root using
the covering bookmark printed by the failed command:

```sh
jj new <bookmark>
jj sparse reset
# Then run the intended jw reseed command again.
```

The default bookmark guard keeps your work named before mutation. With explicit
`--allow-unbookmarked`, use the printed prior commit or JJ's operation log instead.
Do not run ordinary snapshotting commands on a half-filled checkout before this
deliberate recovery. Successful peers in a failed batch stay complete.

There are no progress journals, receipts, migrations, or `--resume` flags. Old
files are ignored and left untouched. If prepare fails, fix its donor and run
prepare again; commands may repeat and the upstream is selected/fetched anew.
`[prepare]` remains a warned alias for `[freshen]`.
See [recovery details](docs/reseed.md#safety-and-recovery).

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
status, not creation or upstream inference. Creation modes are
[described above](#creation-mode-precedence); local donor/preparation/group settings
belong in the primary's `.jwreseed.local.toml`.

## Further reading and development

- [Agent skill](skills/jj-waltz/SKILL.md): task routing, execution, and verification.
- [Reseed/prepare operator guide](docs/reseed.md): detailed payload, group, and recovery contracts.
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
