---
name: jj-waltz
description: Use when the user mentions jw, jj-waltz, or .jwlinks.toml; asks to manage jj workspaces or worktrees; or requests parallel work in separate workspaces. Covers jw add/switch (CoW creation), jw reseed (in-place CoW refill), and jw up (fetch, prepare, advance).
updated:
  by: model:zai-coding-plan/glm-5.3
  at: 2026-10-15
sources:
  - resource: "opencode://ses_ebf063316ffeZf7tZ07pQTNqbG"
    title: Independent skill rewrite take with unified command-model routing and scannable reseed/up rules
  - resource: /README.md
    title: Maintained command surface verified for routing
  - resource: /docs/reseed.md
    title: Reseed and up operator contract behind the deep rules
  - resource: /docs/workspace-manager.md
    title: Interactive manager reference
  - resource: /src/cli.rs
    title: Flag surface and conflicts verified for this skill
---

# jj-waltz

Use `jw` when the user requests JJ workspace lifecycle or navigation work. Its
lifecycle management is optional. Keep native app tasks in the checkout the app
created, use its actual Git or JJ capability, and give each checkout one writer.
An orchestrator inspecting a worker checkout stays read-only. Do not create
another JJ workspace merely to route an agent; no mandatory inspection ritual or
adoption step is required for ordinary coding.

## Command model

| | `jw add` / `jw s` | `jw reseed` | `jw up` |
| --- | --- | --- | --- |
| Outcome | New workspace; `s` also enters it | Existing workspace, CoW sharing renewed | Donor freshened; targets advanced |
| History | Empty child of snapshotted source `@`, or `--at` | Keeps target revision, or resets via `--at` | New empty change at the fetched goal; no `--at` |
| Targets | `NAME...` or `--path DESTINATION` | Bare in a variant targets it; primary needs names/`--group` | Bare in a variant updates it; bare at primary is source-only |
| `--source-at` | Donor tree assertion | Donor tree assertion | Upstream advancement goal |
| Network / commands | Never | Never | Fetches one pinned bookmark; runs `[freshen]` argv |
| Unpublished target work | — | Snapshotted and kept | `freshen-<UTC>` bookmark unless `--no-freshen-bookmark` |

Use `jw` for requested JJ workspace lifecycle actions and ordinary `jj`
commands for revision history, bookmarks, and remotes. The native app owns its
created Git checkouts, including those already adopted into JJ.

## Workflow

### 1. Inspect

Identify the JJ repository, current directory, requested workspace names, and
whether the task creates, switches, diagnoses, adopts, links, reseeds, advances,
or removes workspaces.

Before a mutation, inspect the relevant state with `jw list`, `jw current`,
`jw path <name>`, or `jw status <name> --refresh none`. Use `jw --help` and the
relevant subcommand help for current syntax instead of relying on a cached
command list.

When installed behavior may differ from the repository or documentation, inspect
any shell wrapper and resolve the executable beneath it before making
version-specific claims. Use the current shell's executable-only lookup:
`type -P jw` in Bash, `whence -p jw` in Zsh, or `command -s jw` in Fish. Then
verify the resolved path and version:

```bash
resolved_jw="$(whence -p jw)"  # zsh; use the lookup above for bash or fish
realpath "$resolved_jw"
"$resolved_jw" --version
```

When checkout identity is unclear, `jw context [PATH]` inspects it read-only;
use `--format=json` when another tool consumes the result. Keep Git checkout
topology and JJ workspace identity separate. For a requested lifecycle change
to a native app checkout, [`references/codex.md`](references/codex.md) explains
ownership and capability checks. An older installed `jw` may lack `context`
even when this skill is current; check `jw --help` first.

Complete inspection when the executable identity, target workspaces, current
state, and intended mutation are unambiguous.

### 2. Route and act

Choose only the branch the request needs. Load only the reference linked by
that branch; treat `evals/` and unrelated references as test data, not runtime
instructions.

#### Create, switch, or execute

- `jw add <name>...` creates workspaces without switching; `jw switch <name>...`
  creates missing workspaces and selects the last one; `jw s` is the alias.
- Both accept `--path DESTINATION` for one workspace without repeating `NAME`,
  and optional `--from SOURCE` (default: current checkout). From repository
  `opencode`, `--path ~/src/opencode-fix` infers `fix`; `--path ~/src/myfix`
  infers `myfix`. Existing registrations take precedence. Name-only creation
  remains `repo.NAME` beside the verified owning checkout, not an automatic
  `~/src` destination.
- Without `--at`, creation snapshots the source and uses its current `@` as the
  base, including dirty tracked work and a merge `@`. Explicit `--at REVSET`
  keeps exact-one frozen revision selection. Optional `--source-at REVSET`
  asserts the donor's snapshotted tracked tree without replacing the target
  parent. Creation inherits only the local ignored-copy list, not reseed's
  source/assertion/freshening fields; existing-only switches skip assertions.
- Creation defaults to CoW/JJ-only. `--colocate` alone selects an ordinary
  colocated checkout, overriding default/configured CoW; `--cow` overrides
  configured colocation. `--cow --colocate`, `--cow --no-cow`, and
  `--colocate --no-colocate` conflict. See
  [mode precedence](../../README.md#creation-mode-precedence) for config
  opt-outs. Ordinary-checkout fallbacks warn; never claim CoW success after a
  warning.
- `--bookmark` applies to one workspace. For batch creation, use the configured
  `bookmark_template` or omit bookmarks.
- Prefer `jw switch --execute <command> <name>` when a tool or agent should run
  inside the target; it runs there without changing the parent shell directory.
- Actual selected payload (`--copy-ignored`) forbids destinations nested inside
  the donor: choose an external `--path` or `--no-copy-ignored`. Existing
  switches never refill payloads. Payload copies always require real CoW, even
  with `--no-cow` or `--colocate`.

These are this checkout's implemented interfaces; source changes do not update
an installed executable. Inspect the actual binary before relying on them.

#### Reseed existing workspace contents

For requested in-place CoW updates, read the [reseeding guide](../../docs/reseed.md).

- One-off use is `jw reseed NAME`, or bare `jw reseed` inside the current
  variant; the primary requires names or `--group`. Explicit names or groups
  suppress implicit current selection even if no group matches. Names are
  literal, not navigation tokens; no selection ever means all workspaces.
- Donor: CLI `--from` > local `source` > verified primary checkout. Assertion:
  CLI `--source-at` > local `source_at` > freshly snapshotted donor `@`.
  Assertions compare tracked trees, not commit IDs, and never advance the donor.
- Keep mode preserves the target's snapshotted feature revisions; explicit
  `--at REVSET` starts an empty child instead. Pause writers, then use
  `--dry-run` to inspect replacement scope. `--replace-content` destroys
  unrecorded contents except `.jj` — explicit authorization, never assumed.
- If interrupted, use `jw reseed NAME --resume` (or bare `--resume` for the one
  current target) **before** ordinary JJ commands that snapshot partial files.
  Recovery reuses recorded IDs and frozen policy; it rejects new defaults,
  groups, and replacement flags.
- Ignored-copy lists replace in CLI > local > root global order; missing local
  selection inherits, explicit `[]` does not, and `--no-copy-ignored` disables
  copying. Matching nonignored source directories are skipped and reported.
  Read the [policy guide](../../docs/reseed.md#ignored-copy-policy-layers).

#### Advance with up

Use `jw up` only when source **and** target advancement is requested.

- Bare up in a variant updates it; at primary it prepares the donor only, with a
  real completion receipt and no dummy target. Names/groups select only explicit
  targets; `--from` changes the donor, never the caller's role. Run inside a
  registered workspace.
- Pin before fetch: CLI/local source goal, otherwise one applicable native
  tracked local/remote pair related to the clean donor base, otherwise one exact
  remote bookmark (including untracked refs), otherwise stop and ask for a
  one-run `--source-at`. Do not guess main/origin, mine operation logs, or
  invent following state. Unpublished source work blocks advancement.
- Up runs local `[freshen]` `command` or ordered `commands` argv sequentially in
  the donor root; both fields together are an error even when empty. There are
  no global or guessed build commands. With no commands, selected existing
  ignored payload may be borrowed but is not freshly built.
- Targets start **new empty changes on the exact fetched goal**, not rebases; up
  has no independent `--at`. Unpublished target work gets a collision-safe
  `freshen-<UTC timestamp>` bookmark unless `--no-freshen-bookmark` was frozen
  before preparation. Existing feature bookmarks never move. Preview with
  `--dry-run`; the exact post-fetch goal may be unknown in preview.
- Preparation settings alone never activate up. While source preparation has no
  frozen target plans, resume with all original literal targets in their
  recorded order. Once plans freeze, individual targets or subsets resume
  without refetching or rebuilding; bare `--resume` selects only the current
  recorded target. Do not add overrides or borrow partially installed ignored
  dependencies during recovery. Completed command steps are skipped; an
  in-flight step may retry, not exactly-once.
- `[prepare]` is a deprecated alias: rename it only after recovery finishes,
  since frozen policy bytes must match. `reseed --freshen` is removed; legacy
  v1/v2 batches still use `reseed ORIGINAL-NAMES --resume` with their original
  keep-target/explicit-base intent, never reinterpret them as up.

#### Shell navigation

The binary cannot change its parent shell. Shell integration turns the path
returned by `jw switch` into a directory change; `--execute` intentionally
bypasses that behavior.

```bash
eval "$(jw shell init zsh)"     # zsh
eval "$(jw shell init bash)"    # bash
jw shell init fish | source     # fish
```

After adding shell init, restart or re-source the shell before retrying the
switch. Use `jw current`, `jw root`, and `jw path <token>` instead of inferring
identity from directory names: `@` is current, `-` is previous, and `^` or
`default` resolves the default workspace. Switching from a subdirectory carries
that relative path to a sibling workspace when it exists.

#### Links

For `.jwlinks.toml`, shared ignored directories, missing targets, or link
conflicts, read [`references/links.md`](references/links.md) before changing
configuration or retrying link creation.

#### Status, adoption, cleanup, or recovery

For `status`, `doctor`, `adopt`, `remove`, `prune`, or `reconcile-external`,
read [`references/lifecycle.md`](references/lifecycle.md) before acting. It
contains the mutation gates and removal safeguards. The interactive manager has
its own [guide](../../docs/workspace-manager.md).

#### Explicit parallel workspaces

When the user explicitly requests separate JJ workspaces, use one writer per
workspace, reuse a clearly matching workspace, and keep tightly coupled or
sequential work together. For native app tasks, use its authorized checkouts
and keep the orchestrator read-only in worker checkouts. Follow the host's
policy for agents and parallel execution; workspace creation does not grant
permission to spawn agents.

Complete this stage when the requested branch either succeeds or stops with the
exact unresolved state and no unintended workspace mutation.

### 3. Verify

Verify the boundary the user will rely on:

- creation or switching: confirm `jw list` and `jw path <name>`; use `jw current`
  only after shell-integrated navigation or from a command executed inside the
  target workspace;
- executed tools: confirm their exit status and working directory;
- shell navigation: confirm from the initialized interactive shell, not from the
  child binary alone;
- reseed or up: confirm target status, kept feature revisions or freshen
  bookmarks, and no pending recovery record before resuming ordinary JJ
  snapshots;
- links or lifecycle work: use the completion checks in the branch reference.

Report the resulting workspace names and paths, what changed, and any shell,
executable-version, directory, link, or bookmark state that remains unverified.

Complete the task only when observed state matches the requested workspace
outcome.
