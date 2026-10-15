---
name: jj-waltz
description: Use when the user asks about jw or jj-waltz, creating or switching JJ workspaces, renewing CoW sharing, advancing with up, workspace links, or requested workspace lifecycle and cleanup.
type: Skill
title: jj-waltz — choose the workspace outcome
resource: /skills/jj-waltz/SKILL.proposed0.gpt61sol.md
status: superseded
generated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
sources:
  - resource: "opencode://ses_ebf068824ffeRuaEQy26Krx5u6"
    title: Incorporated this take into the canonical three-take synthesis and recorded its supersession
  - resource: "opencode://ses_ec10d60dbffencx7PCZnh1dnjg"
    title: Implemented up, compared command UX, and drafted a replacement skill separating creation, renewal, advancement, and frozen recovery
  - resource: /skills/jj-waltz/SKILL.md
    title: Existing activation scope, app ownership, shell, routing, and verification guidance
  - resource: /docs/reseed.md
    title: Maintained command behavior, native upstream selection, payload policy, and legacy recovery
  - resource: /skills/jj-waltz/references/lifecycle.md
    title: Lifecycle diagnosis, creation fallback, adoption, removal, and external ownership boundaries
  - resource: /src/cli.rs
    title: Implemented CLI arguments and command distinctions
---

# jj-waltz

> **Superseded by the [canonical skill](SKILL.md) and
> [README](../../README.md).** Preserved with its paired
> [README draft](../../README.proposed0.gpt61sol.md) as synthesis evidence;
> load `SKILL.md` for current guidance.

## Choose the action before the command

**Primary** means the registered checkout owning the shared repository; **variant**
means another workspace in it; **donor** means the selected source supplying files.
Do not substitute a directory named `default` for verified primary identity.

| Requested outcome | Use | Do not confuse with |
| --- | --- | --- |
| Create a separate workspace | `jw add NAME` | Entering it |
| Enter existing/create missing | `jw s NAME` | Refreshing its files or dependencies |
| Renew private CoW sharing, keep feature revision | `jw reseed` | Fetching, installing, rebasing, or advancing |
| Fetch/prepare source and advance target | `jw up` | Rebasing feature edits onto upstream |
| Prepare source only | Bare `jw up` at primary | Updating every registered workspace |
| Inspect/diagnose | `context`, `list`, `status`, `doctor` | Repair or cleanup authorization |
| Remove/forget/reconcile | Requested lifecycle action | Automatic tidying after another task |

If “update” leaves keep-versus-advance unclear, ask which outcome is intended.
Up's preservation bookmark does not make advancement equivalent to keeping work
checked out. Do not select up merely to get newer dependencies for a feature.

## Establish only the context you need

- Use relevant subcommand help; installed `jw` may lag repository documentation.
  If behavior/version is uncertain, inspect the shell wrapper and resolve the
  executable (`whence -p jw` in Zsh, `type -P jw` in Bash, `command -s jw` in Fish),
  then check its real path/version. Source edits do not update that binary.
- Determine the actual caller workspace, repository, donor, and targets. Useful
  queries include `jw context PATH`, `jw list`, `jw current`, `jw path NAME`, and
  `jw status NAME --refresh=none`. Don't infer identity from directory basenames.
- There is no mandatory adoption or inspection ceremony for ordinary coding.
  Keep work in the app-created checkout when applicable. JJ adoption does not
  transfer a native app's Git-worktree ownership to jw.
- Give each checkout one writer. An orchestrator examining a worker stays read-only.
  Workspace creation is not permission to delegate; follow the user's agent policy.

For ambiguous Git/JJ topology or native app checkouts, read
[app ownership and routing](references/codex.md). For lifecycle mutations, read
[lifecycle safeguards](references/lifecycle.md). Load only references needed for
the task; `evals/` are not runtime instructions.

## Shared controls—and important differences

| Control | `add` / new `s` | Existing `s` | `reseed` | `up` |
| --- | --- | --- | --- | --- |
| Source default | Current checkout | No donor refill | Local `source` → primary | Same as reseed |
| `--from` | Creation source/context | Selects command context | Donor only; caller still selects target | Same as reseed |
| `--source-at` | Snapshotted donor tree assertion | No assertion | Donor tree assertion | Upstream advancement goal |
| `--at` | Target base | No reset | Explicit reset base | Not available |
| Ignored-copy flags | Creation payload | No refill | Refill payload | Refill payload |
| Mode/bookmark creation flags | Creation only | No conversion/bookmark creation | Not creation controls | Preservation opt-out is separate |
| `--dry-run`, `--resume` | Not available | Not available | Supported | Supported |

Reseed/up need a registered caller even with `--from`; creation with an explicit
source can start outside a checkout. Avoid passing creation flags to an existing
switch as though they would refresh it.

### Create or switch

```sh
jw add feature tests
jw s feature
jw add --path /chosen/destination --from /path/to/source
jw s --execute opencode feature
```

- Name-only creation uses `repo.NAME` beside the owning checkout. `--path` selects
  one exact destination and can infer its name; registered identity wins. Under
  `opencode`, `opencode-fix` infers `fix`; `myfix` remains `myfix`.
- Default creation is CoW/JJ-only, an empty child of freshly snapshotted source `@`.
  Dirty tracked work and a merge `@` can participate. `--at` independently chooses
  an exact-one base; `--source-at` asserts tracked-tree equality, not commit equality.
  Local maintenance source/assertion/build settings are not creation defaults.
- `--colocate` explicitly requests ordinary linked Git creation when supported;
  `--cow` requests CoW/JJ-only. Both positive modes conflict. `--no-cow` alone
  yields ordinary JJ-only tracked checkout. Read the README proposal's
  [mode table](../../README.proposed0.gpt61sol.md#creation-mode-precedence) for config combinations.
- Creation's ordinary-checkout fallbacks report warnings. Do not claim CoW success
  after fallback. Ignored payload independently requires real private CoW.
- Explicit `--bookmark` is single-workspace only. Batch creation can use the
  configured bookmark template; `--no-bookmark` suppresses creation.
- A binary cannot change the parent shell. Initialize the user's shell when
  directory switching is wanted; `--execute` deliberately runs in a child instead.
  Never claim the parent changed directory based only on a printed path.

### Renew or advance

Read the [operator guide](../../docs/reseed.md) before an in-place refill. Pause
source/target writers, builds, monitors, and external JJ processes. Preview first
unless the user has already supplied an inspected plan.

```sh
jw reseed --dry-run                # keep current variant's revision
jw up --dry-run                    # advance current variant; source-only at primary
jw up worker-a worker-b --dry-run  # only these targets
```

Bare reseed at primary requires explicit targets/groups. Bare up at primary is
source-only. Names/groups suppress implicit current selection, even if a group
matches nothing. Names are literal, not `@`, `^`, or previous-workspace shortcuts;
empty selection never means all. `--from` does not change these caller roles.

For reseed, assert donor tree with CLI `--source-at` → local `source_at` → freshly
snapshotted donor `@`. Target feature differences are retained per path. Explicit
`--at` instead authorizes a new empty child at that base, not a rebase.

For up:

1. Pin upstream before fetch: CLI/local source goal, otherwise a unique applicable
   native tracked pair, otherwise a unique exact remote at the clean source base
   including untracked refs. Ambiguity requires a one-run `--source-at`.
2. Never guess main/origin, mine operation logs, invent following state, or repair
   native tracking to force a match. An explicit missing simple `branch@remote`
   requires that remote already be configured; its preview goal remains unknown.
   Optional local `[freshen].remote` constrains selection, not an unrelated fallback.
3. Unpublished source work blocks advancement, including work beneath an empty
   scratch. Do not bypass refusal by abandoning source work or disabling backups.
4. Commands run sequentially in donor cwd from local `[freshen]`: one `command`
   or ordered `commands`, never both even when empty. No implicit shell, global
   builder, or guessed installer. No commands means existing payload, not a build.
5. Targets start new empty changes on the exact prepared goal. Existing feature
   bookmarks never move. Unpublished target work gets a frozen collision-safe
   `freshen-<UTC timestamp>` bookmark unless explicitly opted out with
   `--no-freshen-bookmark`; opt-out does not discard JJ history or source checks.

Full/conflict-free eligible JJ-only targets and real CoW are required. Do not
substitute ordinary copies or directory links after refusal. `--replace-content`
destroys unrecorded target data, including `.env`, except `.jj`; use it only when
that destruction is authorized. Dry-run can snapshot real work, but does not
fetch, create changes/bookmarks, build, or refill.

### Private ignored data versus shared links

CLI `--copy-ignored` lists replace primary-local lists, which replace root global
`copy_ignored`; absence inherits, local `[]` does not. `--no-copy-ignored` disables
even an explicit list. The policy is shared by creation, reseed, and up.

Quote patterns. `*`, `?`, `**`, and trailing `/` are supported. Missing matches
succeed; nonignored source matches are skipped/reported. Selected ignored roots
must pass tracked-overlap, target-ignore, confinement, and real-CoW checks. Actual
payload creation nested inside its donor is refused. Existing switches do not copy.

For shared-directory links, `.jwlinks.toml`, or private-copy/link collisions, read
[links](references/links.md). Private reflinks and shared symlink destinations are
different ownership/mutation choices; do not silently substitute one for the other.

## Recover before ordinary snapshots

If a refill stops, do not run snapshot-capable JJ commands in a half-filled target.
Resume the stored workflow without new source, revision, payload, replacement, or
bookmark overrides:

```sh
jw up worker-a worker-b --resume  # complete original order until plans freeze
jw up worker-a --resume           # frozen individual/subset recovery
jw up --resume                   # current target; source-only when at primary
jw reseed feature --resume       # reseed or old preparation
```

After plan freeze, recovery uses exact intent and receipts without refetch/build or
bookmark reallocation. Restore recorded source/payload state if checks refuse;
don't adopt changed bytes or use repository-wide undo. Completed command steps are
skipped; an in-flight command may retry after partial side effects—not exactly-once.

`reseed --freshen` is removed. Legacy v1/v2 batches still use `reseed --resume`
with their original target-history intent, not `up --resume`. Rename legacy
`[prepare]` to `[freshen]` only after recovery: frozen local policy bytes must match.

## Verify the requested outcome, then report

- **Create/switch:** confirm registration/path and actual selected execution cwd.
  Check warnings before claiming CoW or complete payload setup.
- **Reseed:** confirm kept revision/tree and requested payload/refill result.
- **Up:** confirm exact goal, new target parent, preservation bookmark or recorded
  opt-out, and command/payload completion. Source-only success has no target.
- **Recovery:** distinguish complete, failed, pending, and already-receipted targets.
  Never turn partial progress into a batch-success claim.
- **Cleanup:** inspect owner, contents, and bookmark policy first. Removal can
  permanently delete ignored/private files; app-owned checkouts remain with their
  owner. Consult lifecycle safeguards rather than bypassing a refusal with JJ/Git.

Report names/paths, what happened to history and files, checks actually performed,
warnings, and unresolved state. A preview, source preparation, or printed path alone
does not prove complete target refill or parent-shell navigation.
