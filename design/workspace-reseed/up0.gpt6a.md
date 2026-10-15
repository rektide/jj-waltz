---
type: Design
title: Native-upstream freshening and CoW workspace advancement
description: Commissioned up workflow, native source selection, sequential freshening commands, unpublished-target bookmarks, and symmetric copying controls.
resource: /design/workspace-reseed/up0.gpt6a.md
status: commissioned
generated:
  by: model:openai/gpt-6-astra
  at: 2026-10-15
extensions:
  ticket: jjwaltz-variants-up
  related_work: [jjwaltz-variants, jjwaltz-reseed-ergonomics]
sources:
  - resource: "opencode://ses_ec226a15effeUM4c1j7gwYPFHF"
    title: Human commissioning of up, native upstream inference, local sequential commands, global ignored copying, and unpublished-target preservation; parent implementation handoff
  - resource: /design/workspace-reseed/bookmark-tracking0.gpt61sol.md
    title: Native tracked-reference queries and graph checks, including corrected ambiguity and remote fast-forward rules
  - resource: /design/workspace-reseed/jj-new-preservation0.gpt61sol.md
    title: Ordinary JJ new preserves unchanged files and sharing
  - resource: /design/workspace-reseed/ergonomics0.gpt6a.md
    title: Earlier repeat-reseed design and explicit target-history review junction
  - resource: /docs/reseed.md
    title: Existing donor, payload, preparation and recovery behavior being extended
  - resource: /src/reseed/engine.rs
    title: Existing selection, payload matching, frozen plans, receipts and target reset mechanism
  - resource: /src/reseed/prepare.rs
    title: Existing source preparation phases, single-command configuration and source/target proof boundaries
---

# Commission: up and shared copying policy

The human has authorized Sol xhigh implementation in ordered, reviewable commits.
This design supersedes the earlier proposal to add more freshening flags to
creation/reseed. It introduces **`up`**, removes **`reseed --freshen`**, and keeps
ordinary reseed focused on CoW renewal. Existing mechanisms should be reused;
there is no new workspace or bookmark-tracking system.

## Command outcomes

| Command | History and files |
|---|---|
| `add` / `s` | Create an empty child of source current state, or explicit `--at`; private CoW tracked copies by default; `s` also enters |
| `reseed` | Keep existing target work by default and renew sharing; explicit `--at` retains existing new-base behavior |
| `up` | Select/fetch upstream, freshen source, start target at prepared upstream revision and refill with CoW |

Bare `reseed` inside a variant selects that current workspace. From the primary,
it needs named targets (the store owner cannot be reseeded). Bare `up` inside a
variant freshens the primary donor and updates that variant. At the primary,
bare `up` freshens the source itself only; explicit names/groups add targets.
An explicit group must not also select the current workspace implicitly.

Keep `--from`, `--source-at`, ignored-copy controls, and applicable preview/resume
semantics consistent across commands. `--path` belongs to creation, not relocation
of existing targets. Do not force every flag onto every command.

For ordinary borrowing, `--source-at` is an optional assertion about the donor's
tracked tree. Add it to add/s, preserving independent target `--at`. Without an
assertion, use freshly snapshotted donor state. For `up`, `--source-at` selects
the upstream goal before fetch; the goal is resolved again after fetch and
frozen. Explain this distinction as assertion versus source advancement, not
as unrelated meanings of the source argument.

## Source selection without new following state

Explicit `--source-at` wins. Existing explicit local `source_at` configuration
can remain an override; it must not become a mandatory new per-source association.
Otherwise use:

1. An **applicable, unambiguous native tracked local/remote pair** related to the
   donor's clean base. Merely having a tracked bookmark elsewhere is insufficient.
   Account for a prior fetch that moved the tracked pair ahead of the base.
2. If there is no applicable tracked relationship, a **unique remote bookmark
   whose recorded target equals the donor base**, including untracked remote
   bookmarks. A local remote reference is known even when it is untracked.
3. Otherwise request a one-run `--source-at`. Do not select main/trunk, mine the
   operation log, or save a new standing following preference.

Inference happens **before fetching**. Pin the selected remote/name and IDs for
the operation. Server-side commits do not change the local reference until fetch.
Ambiguity at one tier is not permission to silently choose another line at the
next tier. In tracked-graph inference, an exact tracked match must not outrank
another applicable descendant tracked line: the main-at-B/dev-ahead counterexample
in the research is a required design constraint.

Source work stops advancement. Snapshot actual bytes and check unpublished
history, not only whether `@` is empty: an empty working copy can sit above local
commits. Support the usual empty child of a published base and a source directly
at a published revision. Do not force unpublished local bookmark movement or
repair tracking automatically. After fetch require resolved references and
fast-forward movement of the selected remote, including old-remote ancestry in
the new remote. Revalidate the donor before advancing it.

The existing in-flight recovery journal may retain selected names/IDs to finish
an interrupted operation. That is not new permanent following state.

## Freshening commands

Repository-local configuration remains in the existing local policy file; do
not add another config file or global build-command configuration.

```toml
[freshen]
commands = [
  ["bun", "install", "--frozen-lockfile"],
  ["bun", "run", "build"],
]
```

Accept `command = ["cargo", "build", "--locked"]` for one command. Reject
`command` and `commands` together, even if one is empty. Run argv arrays
sequentially in the donor root with no implicit shell. A command failure stops
target advancement/refill. Preserve the existing check that commands did not
change tracked files away from the frozen source goal.

The native selected pair normally provides the remote; an explicit `remote`
setting may constrain selection but must not silently fetch an unrelated remote.
No commands means no installer/build is guessed. Existing selected ignored
payload may still be copied, reported as existing source data, not freshly built.
Global ignored patterns alone must not force every project to declare a builder.

Replace `[prepare]` with `[freshen]`. Give existing users a clear migration path;
do not silently ignore old configuration or render existing pending preparation
records unrecoverable. Persist the frozen command sequence and completion
position for resume, with an honest retry contract for interrupted external
commands; never claim exactly-once execution of arbitrary build commands.

## Unpublished target preservation

After target work is snapshotted and before target advancement, check whether
its work tip is an ancestor of/equal to the selected fetched upstream commit.
If not, create **`freshen-<datetimestamp>`** pointing to the work being left.
`--no-freshen-bookmark` disables this automatic bookmark only, not JJ history
preservation or any source-work checks.

For an ordinary undescribed empty one-parent target `@`, use its sole parent as
the work tip: do not create bookmarks merely because a scratch empty child is
unpublished. Otherwise retain `@`, including meaningful empty/merge changes.
Use ancestry, not equal file trees, as the publication test.

Never overwrite an existing bookmark. Names must be collision-safe across batch
targets and quick repeats. Record chosen name/tip before mutation and recognize
that exact pair on resume so a retry neither duplicates nor repoints preservation
bookmarks. Keep existing feature bookmarks where they are. Target advancement
starts a new empty change on the prepared baseline; this is not a rebase.

## Shared ignored-copy policy

Global user config may declare:

```toml
copy_ignored = ["**/target", "**/node_modules", "**/dist/", "**/build/"]
```

Use CLI selection > repository-local selection > global selection > none.
Lists replace rather than accumulate; absent and explicit empty lists differ.
`--no-copy-ignored` disables copying for that run. Expose the applicable same
controls on add/s, reseed, and up. Do not modify the operator's personal global
file merely to deliver the capability.

Compile/match patterns together during one selection walk, not a complete donor
walk per pattern. Accept trailing slashes, collapse nested roots, and skip
missing matches successfully. Retain confinement, no traversal through symlinked
directories, metadata exclusions, and ignored/tracked-overlap validation. Copy
selected roots privately with real reflinks; do not substitute directory links.
One selection walk does not eliminate the existing correctness/payload scans.

Broad global patterns must be useful in mixed repositories. Resolve how
non-ignored directories with matching names are treated explicitly during
implementation review; they must never be replaced as ignored payload. Existing
target ignored data outside selected roots remains intact.

## Delivery and recovery boundaries

Land foundations before orchestration: native source resolution; command/config
policy and shared ignored selection; source-argument/current-target symmetry;
target bookmark preservation; then up integration and removal of the old flag.
Each coherent commit comes back for review before dependent work proceeds.

Use current source/target proof and reseed journals rather than wrapping shell
commands around partially mutated targets. Source-only up needs a legitimate
preparation completion path without creating a dummy target. Dry runs explain
prospective fetch/commands/target/bookmark intent without performing them.
Recovery must freeze names, revisions, payload, commands, and preservation
bookmarks; it must not rediscover upstream or refetch after the plan is frozen.

Existing code/docs/tests and supported JJ capabilities are evidence, not a reason
to preserve an awkward interface. Keep native verification tied to actual
behavioral risks and real CoW sharing. Do not fetch/reset/install in live archive
or operator workspaces while developing this feature.

## Related evidence

- [Native bookmark research](bookmark-tracking0.gpt61sol.md) supplies structured
  reference queries and graph boundaries; the untracked exact-match fallback
  above incorporates the subsequent live opencode inspection.
- [JJ new preservation](jj-new-preservation0.gpt61sol.md) establishes ordinary
  advancement already leaves equal tracked entries and their sharing alone.
- [Earlier reseed ergonomics](ergonomics0.gpt6a.md) remains the record of the
  keep-target discussion. Up now supplies the distinct advance-and-refill action.
