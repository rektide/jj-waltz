---
name: jj-waltz
description: Use for jw or jj-waltz, creating or switching JJ workspaces, requested parallel JJ workspaces, CoW renewal with reseed, upstream source preparation with prepare, workspace links, or JJ workspace inspection and cleanup.
type: Skill
title: jj-waltz — choose and verify the workspace outcome
resource: /skills/jj-waltz/SKILL.md
status: maintained
updated:
  by: model:openai/gpt-6.1-sol#xhigh
  at: 2026-10-29
sources:
  - resource: "opencode://ses_e779f1a9affejTLMoRyE5Nv4xW"
    title: Aligns operative skill with both public native routes, schema 1, final-only preparation and read-only PLANNED previews
  - resource: "opencode://ses_e8720e969ffeR1x2H40WZ36GY6"
    title: Documented explicit reseed preparation, preparation-only locking, and deferred prospective readiness
  - resource: "opencode://ses_eb6ef3ef2ffe4155t844hpUnEL"
    title: Replaced the keep/reset reseed contract with donor-current sync and post-sync jj duplicate/rebase layering
  - resource: "opencode://ses_eb75a514cffe7jUOMWNZAIm3fX"
    title: Replaced recorded recovery with forward-only reseed and deliberate bookmark recovery, and retry-from-start source preparation
  - resource: "opencode://ses_eb867bfdaffefMVLpyaPUadbSp"
    title: Documented reserved all selection, donor exclusions, readiness blockers, and literal recovery
  - resource: "opencode://ses_eb8801331ffeirQrF4GMFCxJHf"
    title: Added aggregate readiness and default-on bookmarked-work coverage, explicit opt-out, and immutable/legacy recovery guidance
  - resource: "opencode://ses_eb899cdcdffecJnB7J1dSGs4ci"
    title: Rerouted the skill from combined up to source-only prepare plus reseed --at consumption and legacy recovery routing
  - resource: /design/up-performance/prepare0.gpt6a.md
    title: Command-split decision — prepare is source-only, reseed keeps worker copying
  - resource: "opencode://ses_ebf068824ffeRuaEQy26Krx5u6"
    title: Synthesized the original, Sol, and G skills into canonical outcome routing, scoped execution, and verification guidance
  - resource: /skills/jj-waltz/SKILL.md
    revision: fceba759ac391599f85137df52f4fc9eb08683d5
    title: Original shell, identity, lifecycle, and parallel-workspace guidance
  - resource: /skills/jj-waltz/SKILL.proposed0.gpt61sol.md
    title: Outcome selection, source-only up, shared controls, and recovery verification
  - resource: /skills/jj-waltz/SKILL.proposed0.glm53m.md
    title: Independent command-model comparison and scannable action branches
  - resource: /README.md
    title: Canonical human-facing guide and shared command comparison
  - resource: "opencode://ses_ec10d60dbffencx7PCZnh1dnjg"
    title: Updated skill to integrated up source/target roles, native upstream selection, preservation bookmarks, and versioned recovery
  - resource: "opencode://ses_ec1394461ffeEXBHM582vTs9VM"
    title: Aligned donor assertions/current-target reseed and phase-specific full-preparation versus individual frozen-target recovery with delivered behavior
  - resource: "opencode://ses_ec1a8d43bffe02bXuEoA7hEW3l"
    title: Updated freshen sequence/recovery and ignored-copy policy, eligible matching, frozen empty selection, and current command exposure
  - resource: "opencode://ses_ec206881bffenBcwNsQxlvfx3J"
    title: Aligned creation guidance with shared add/s destinations, source selection, snapshotted-current base, and mode precedence
  - resource: "opencode://ses_ec37b01d2ffe9ytO0jRsdmoyXA"
    title: Added reseeding, primary-source defaults, explicit freshening, and safe preparation recovery
  - resource: /docs/reseed.md
    title: One-off and batch reseeding operator guide
---

# jj-waltz

Choose the workspace outcome, use the relevant branch, and verify the result.
**Primary** means the registered checkout owning the shared JJ repository;
**variant** means another workspace in it; **donor** means the selected source
supplying files. Verify identity rather than relying on a directory named `default`.

## Choose the action

| | `add` / `s` (`switch`) | `reseed` | `prepare` |
| --- | --- | --- | --- |
| Outcome | Create; `s` enters existing or creates missing | Renew sharing by syncing to the donor | Select/fetch one upstream, prepare the donor, report the exact commit |
| Target workspaces | Created or entered | Refilled in place; names/groups select | None — only the donor; never selects or refills a worker |
| Target history | Empty child of source `@`, or `--at` | Empty child of the donor's current commit, or `--at` override | — |
| Bare in variant / primary | Name/path required | Current / explicit targets required | Source-only / source-only |
| Donor | Current checkout or `--from` | `--from` → local `source` → primary | Same as reseed |
| `--source-at` | Donor tree assertion | Donor tree assertion; upstream goal with `--prepare` | Upstream goal |
| Fetch / preparation | Never | Only with `--prepare` | Pinned upstream / local argv |
| Preview | — | `--dry-run` | `--dry-run` |

Updating follows three steps in order: `jw prepare` updates the seed (fetch +
build in the donor), `jw reseed NAME` syncs a workspace onto the donor's current
commit (whole tree + selected ignored payload), then the feature is layered back
in jj — `jj duplicate -r <bookmark> --onto <base>` keeps the old line and its
dated bookmarks, while `jj rebase` drags bookmarks with the commits. `prepare`
never selects or refills a worker; layering is not jw's work. [Full
comparison](../../README.md#choose-the-outcome).

## Establish the relevant context

- Use live subcommand help. When installed behavior may differ from source, inspect
  the shell wrapper and executable: `whence -p jw` in Zsh, `type -P jw` in Bash,
  or `command -s jw` in Fish. Check its real path and `--version`; source edits do
  not update that binary.
- Identify caller, donor, and targets using relevant queries: `jw context PATH`,
  `jw list`, `jw current`, `jw path NAME`, or `jw status NAME --refresh=none`.
  Context separates Git topology from JJ identity without refreshing. An older
  binary may lack it; check help first.
- Keep native app tasks in their app-created checkout using its actual Git/JJ
  capability. Adoption does not transfer ownership. No inspection or adoption
  ceremony is required for ordinary coding. Read [app ownership](references/codex.md)
  when topology or lifecycle ownership matters.
- Give each checkout one writer. For requested parallel JJ workspaces, reuse a
  clearly matching workspace, separate independent tasks, and keep coupled work
  together. Orchestrators inspect workers read-only. Follow the host's delegation
  policy; creating a workspace does not grant permission to spawn agents.

Load only the references needed by the selected branch. `evals/` are test data,
not runtime instructions. Ordinary `jj` handles deliberate history and bookmark
editing; `jw` handles the requested workspace lifecycle.

## Create, switch, or execute

```sh
jw add feature tests
jw s feature
jw add --path /chosen/destination --from /path/to/source
jw s --execute opencode feature
```

- Name-only creation uses `repo.NAME` beside the owning checkout. `--path` selects
  one exact destination and can infer the name: under `opencode`, `opencode-fix`
  becomes `fix`, while `myfix` stays `myfix`. Registered identity wins. Relative
  destinations resolve from the caller; explicit `--from` permits outside-checkout
  creation.
- Creation defaults to CoW/JJ-only and an empty child of freshly snapshotted source
  `@`, including dirty tracked work or a merge. `--at` independently selects one
  exact base; CLI `--source-at` asserts tracked-tree equality, not commit equality.
  Local maintenance source/assertion/preparation settings are not creation defaults.
- `--cow` selects CoW/JJ-only; `--colocate` selects ordinary linked Git creation
  when supported. Both positive flags conflict. `--no-cow` alone gives an ordinary
  JJ-only checkout. See [mode precedence](../../README.md#creation-mode-precedence).
  Tracked-creation fallback warns; never report that as CoW success.
- Explicit `--bookmark` is single-workspace only; batches can use the configured
  bookmark template. `--no-bookmark` suppresses creation. Existing switches do not
  reset revisions, convert checkout mode, assert donor trees, or refill payloads.
- `switch` creates missing names and selects the last. `--execute` runs a child
  command there; check its exit status and cwd.

### Shell navigation

The binary cannot change its parent shell's directory. Initialize the user's shell:

```sh
eval "$(jw shell init zsh)"  # zsh
eval "$(jw shell init bash)" # bash
jw shell init fish | source # fish
```

Restart or re-source after setup. [Other shells](../../README.md#shell-setup).
Navigation tokens are `@` (current), `-` (previous), and `^` or `default` (default).
Inspect with `jw path TOKEN`; a switch carries the relative subdirectory if present
at the destination. A printed path or successful `--execute` does not prove the
parent shell changed directory.

## Renew, or prepare the source

Read the [operator guide](../../docs/reseed.md) before an in-place refill. Pause
source/target writers, builds, monitors, and external JJ processes. Preview the plan:

```sh
jw prepare --dry-run               # prospective fetch and preparation commands
jw reseed --dry-run                # sync current variant onto the donor's current commit
jw reseed worker --dry-run         # literal target; --at overrides the sync base explicitly
jw reseed --group all --dry-run    # all registrations except verified primary/donor
```

Reseed requires a registered caller, even with `--from`. Names/groups suppress
implicit current selection; empty selection never means all. Names are literal, not
navigation tokens. `--from` changes the donor, not the caller's primary/variant role.

Built-in `--group all` needs no configuration. It excludes the verified primary
and effective donor by root identity, not the name `default`; unsafe or
unbookmarked registrations remain BLOCKED, and any blocker prevents all refills.
Local group `all` definitions are rejected.
See [all registered workers](../../docs/reseed.md#all-registered-workers).

**Reseed:** CLI `--source-at` → local `source_at` supplies an optional tracked-tree
assertion; the sync base is otherwise the donor's freshly snapshotted `@`. Plain
reseed makes the workspace's content the donor's content — no feature logic, no
branch restoration. Explicit `--at` overrides the target commit only. Plain reseed
never fetches or prepares dependencies. Explicit `reseed --prepare` runs the same
preparation as `jw prepare` once, then plans and refills from the verified donor.
Source, upstream goal (`--source-at`), local commands, and ignored-copy settings
apply to that preparation too.

The repository operation lock covers preparation only and ends before reseed
planning/refill; plain reseed acquires no operation lock. The caller must keep
donor and target writers quiescent. `reseed --prepare --dry-run` explains the
prospective preparation as PLANNED, never readiness. All previews perform no
snapshot, live assertion, fetch, builder, payload inspection, membership/clone probe,
lock or metadata write. History preservation and obstruction checks remain pending.

Fresh reseeds require a present, unconflicted **local or remote bookmark** covering
the snapshotted work tip (at the tip or a descendant). An old
base bookmark cannot cover new work. Ordinary undescribed empty single-parent
scratch uses its parent; described empty changes and merges count themselves.
Execution records covering history after preservation; preview does not prove it. Do not auto-create or
move bookmarks to defeat a refusal. The operator can deliberately name the work
or explicitly choose `--allow-unbookmarked`; that bypasses only this requirement,
not ownership, content, sparse-state, or CoW checks.

**Prepare** is source-only from any registered workspace — plain `jw prepare` at
the primary prepares the primary donor:

1. Pin upstream before fetch: CLI/local goal, otherwise one applicable native
   tracked pair, otherwise one exact remote at the clean donor base, including
   untracked refs. Ambiguity requires a one-run `--source-at`. Do not guess main/origin,
   mine operation logs, or invent following state. `[freshen].remote` constrains
   selection. A missing explicit `branch@configured-remote` can have an unknown
   preview goal.
2. Unpublished donor work blocks preparation, even beneath empty scratch work;
   the selected remote must fast-forward. Preserve that refusal rather than
   abandoning source work to force progress.
3. Local `[freshen]` provides `command` or ordered `commands`, never both even when
    empty. Run argv sequentially in donor cwd, without an implicit shell. Later steps
    may repair tracked edits before one final conformance check; identity guards
    still run between steps. All steps must tolerate retry from the start. No commands means existing payload, not a fresh
   build; do not guess an installer or use global build commands.
4. Completion reports **actual S separately from fetched upstream goal**, also in
   schema-1 `--format=json` as `.source.source_commit_id` and
   `.source.upstream_goal_commit_id`. The old ambiguous top-level key is removed.
   Prepare performs no payload census/verification and rejects ignored-copy flags.
   Prepare never selects or refills a worker and creates no worker-preservation
   bookmarks (native fetch can update refs). Consume the prepared donor with
   plain `jw reseed worker`; layer the feature back afterward in jj —
   `jj duplicate -r <bookmark> --onto <base>` keeps the old line and its dated
   bookmarks, while `jj rebase` drags bookmarks with the commits. Neither step
   is jw's.

Refills require full, conflict-free eligible JJ-only targets and real CoW.
`--replace-content` destroys unrecorded target contents, including `.env`, except
`.jj`; use it only for that requested outcome. Native route is independent:
`--renewal-route=clear-then-populate` (default) or `replace-authorized`. Both use
the same shared command/cohort and gate S-state adoption on successful complete
full-S coverage; replacement has no clear-first fallback. Plain donor input is
assumed prepared with no implicit snapshot. The exact supported native build is
`0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`.

## Private payloads and shared links

Ignored-copy lists replace in CLI → primary-local → root global `copy_ignored`
order; absence inherits, local `[]` does not. `--no-copy-ignored` disables even an
explicit CLI list. This policy applies to creation and reseed, not prepare.

Quote patterns (`*`, `?`, `**`, trailing `/`). Missing matches succeed; nonignored
source matches are skipped/reported. Selected roots must pass source/target ignore,
tracked-overlap, confinement, and real-CoW checks. Actual payload creation inside
its donor is refused. `--no-cow`/`--colocate` do not relax payload's CoW requirement.
Existing switches never refill. See [copy policy](../../docs/reseed.md#ignored-copy-policy-layers).

For shared-directory links, `.jwlinks.toml`, or private-copy/link collisions, read
[links](references/links.md). Reflinks and shared symlink destinations have different
mutation behavior; do not substitute one for the other or overwrite a conflict.

## Recover out of band

If a refill stops, avoid snapshot-capable JJ commands in a half-filled target.
`jw` refuses sparse/mid-refill targets without automatic repair. Full sparse can
still be incomplete after replacement failure. Use captured saved O/history/root
and choose deliberate recovery, for example the covering bookmark in that root:

```sh
jj new <bookmark>
jj sparse reset
# Then re-run the intended jw reseed command.
```

The default bookmark guard preserves named work before mutation. For an explicit
`--allow-unbookmarked` run, recover the printed prior commit or use JJ's operation
log. `jw` never repairs partial contents. There are no journals, receipts,
migrations, or `--resume`; old files are ignored without cleanup.

If prepare fails, fix the source and run prepare again. Every configured command
may repeat; do not borrow incomplete dependencies. No command execution is
exactly-once. `[prepare]` remains a deprecated alias for `[freshen]`.
[Recovery contract](../../docs/reseed.md#safety-and-recovery).

## Inspect, adopt, repair, or clean up

Read [lifecycle guidance](references/lifecycle.md) for status, doctor, adoption,
metadata repair, removal, pruning, and external reconciliation. Doctor diagnoses
without repair. Adoption/repair change metadata, not revisions. Removal can delete
ignored/private files permanently; inspect owner, contents, and bookmark policy.
Use the owning app for app-created checkout cleanup. Consult the
[manager guide](../../docs/workspace-manager.md) for terminal interaction.

## Verify and report

- **Create/switch:** confirm registration/path and actual execution cwd. Use
  `jw current` only inside the target or after shell-integrated navigation. Check
  warnings before claiming CoW or complete payload setup.
- **Reseed:** confirm the sync — target `@-` is the pinned base (donor's current
  commit or `--at`), the tree matches the donor, and payload/refill completed.
- **Prepare:** confirm actual S, distinct upstream goal and final tracked conformance;
  a source-only success has no target. Consume it via reseed
  and verify the worker separately.
- **Structured outcomes:** `--format=json` emits one complete schema-1 document
  on normally returned parsed success/failure. Counts, durations and step indexes
  are checked-u128 decimal strings; unavailable facts are null. Decode exact Unix
  path hex rather than display text. Never infer current disk state from failed
  native child coordinates (last observed only), or final-D sharing from S installs.
  Read [the outcome guide](../../docs/reseed.md#structured-outcomes).
- **Recovery:** distinguish completed and failed targets. Recover failed refills
  deliberately before retrying; partial progress is not batch success.
- **Links/lifecycle:** use the branch reference's completion checks; directory,
  bookmark, and ownership state must match the requested outcome.

Report resulting names/paths, what happened to history and files, checks actually
performed, warnings, and unresolved state. Preview, source preparation, and a printed
path each prove only their own part of the workflow.
