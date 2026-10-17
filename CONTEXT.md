---
type: Context
title: jj-waltz domain context
description: Workspace lifecycle vocabulary and safety contracts.
resource: /CONTEXT.md
status: maintained
updated:
  by: model:openai/gpt-6.1-sol#xhigh
  at: 2026-10-17
sources:
  - resource: "opencode://ses_eb75a514cffe7jUOMWNZAIm3fX"
    title: Replaced journal-based recovery with forward-only reseed, bookmark-based out-of-band recovery, and preparation retries from the start
  - resource: "opencode://ses_eb899cdcdffecJnB7J1dSGs4ci"
    title: Redefined source preparation as source-only prepare with an exact prepared commit, reseed-only worker semantics, and legacy-journal recovery routing
  - resource: /design/up-performance/prepare0.gpt6a.md
    title: Command-split decision — prepare is source-only, reseed keeps worker copying
  - resource: "opencode://ses_ec10d60dbffencx7PCZnh1dnjg"
    title: Connected up activation, native upstream inference, immutable target preservation, source-only receipts, and legacy recovery
  - resource: "opencode://ses_ec1394461ffeEXBHM582vTs9VM"
    title: Defined delivered donor assertions/current-variant targeting and corrected phase-specific preparation versus frozen-target recovery intent
  - resource: "opencode://ses_ec1a8d43bffe02bXuEoA7hEW3l"
    title: Defined freshening sequence/retry and shared ignored-copy precedence, eligible payload selection, and frozen empty-policy meanings
  - resource: "opencode://ses_ec206881bffenBcwNsQxlvfx3J"
    title: Aligned workspace/variant identity and creation contracts with verified placement, CoW defaults, and snapshotted source @
  - resource: "opencode://ses_ec37b01d2ffe9ytO0jRsdmoyXA"
    title: Defined reseeding, safe snapshots, primary-source defaults, and explicit source preparation
  - resource: "opencode://ses_ec37a1a86ffeYXsS5j3A7Jng0t"
    title: Defined wildcard selection and bounded warn-and-skip without weakening target safety
  - resource: /design/workspace-reseed/reseed0.gpt6a.md
    title: Reseeding identity, content, and recovery contract
---

# Domain context

`jj-waltz` makes Jujutsu workspaces feel like named places a developer can create,
enter, inspect, link, and remove safely.

This glossary defines the shared vocabulary used by the CLI, JSON, Herdr, and
future workspace-lifecycle workflows. A defined term is a semantic contract, not
by itself a claim that a command exposing that concept already exists.

## Terms

- **Workspace** — a named Jujutsu working copy with its own filesystem root.
- **Default workspace** — the registered `default` workspace when present,
  otherwise the registered JJ storage-owning checkout, selected by `^` or
  `default`. Removal is never allowed. It need not identify the repository's
  storage owner when another workspace was explicitly named `default`.
- **Variant** — a workspace of the same JJ repository, possibly at an arbitrary
  registered location. New destination-name inference uses the verified storage
  owner's basename, not the current variant's basename.
- **Current workspace** — the workspace containing the command's current root,
  selected by `@`.
- **Previous workspace** — the last workspace visited from the current one,
  selected by `-`.
- **Workspace inventory** — one consistent view of all workspace names, roots,
  and current/default/previous roles for one command.
- **Repository snapshot** — the immutable repository, trunk, and workspace facts
  used by one command. It records the final JJ operation ID after any requested
  working-copy refresh. Status is derived from this snapshot; renderers never
  launch fresh JJ queries.
- **Trunk revset** — the configured JJ revset used as the comparison line,
  defaulting to `trunk()`. It must resolve to exactly one revision. It is not a
  writable bookmark, and resolving trunk never grants permission to move one.
- **Managed workspace** — a JJ workspace with repository-scoped lifecycle
  metadata recorded by `jw`, including its recorded creation base. Management
  records `jw` intent; it does not replace JJ's graph or workspace state.
- **Unmanaged workspace** — a JJ workspace without `jw` lifecycle metadata. It
  remains usable by existing workspace commands. Display code may label derived
  facts as inferred, but destructive or publication automation must not treat
  inferred intent as authoritative.
- **Creation base** — the exact commit from which `jw` created or explicitly
  adopted a managed workspace. It is historical metadata; ordinary lifecycle
  operations do not rewrite it. Explicit metadata repair may replace the recorded
  value after validating a new exact base.
- **Working-copy revision (WC revision)** — the literal revision associated with
  a workspace's `@`. It may be empty and therefore may differ from the revision
  containing work intended for publication.
- **Publish tip** — the revision selected for a publication plan. The default
  semantic rule is non-empty `@`, otherwise the sole parent of an empty `@`.
  Empty merge revisions, conflicts, or divergent changes make selection
  ambiguous or unsafe and must be reported rather than hidden.
- **Workspace stack** — revisions reachable from the publish tip but not the
  recorded creation base, excluding revisions already reachable from resolved
  trunk. A safe stack is connected and has one publish head; merges, shared
  revisions, conflicts, divergence, or missing bases are explicit hazards.
- **Publication anchor** — the non-trunk mechanism used to share work: an
  associated bookmark, a generated bookmark, or no bookmark yet. Bookmark
  association is `jw` metadata, not a current-branch inference.
- **Integration** — evidence describing how a publish tip relates to resolved
  trunk, such as the same revision or an ancestor of trunk. Tree equivalence and
  forge-reported merge state are weaker evidence and must not silently become
  ancestry proof.
- **Cleanup** — a separately planned retirement of workspace registration,
  directory, metadata, and any explicitly selected bookmark state. Integration
  does not imply cleanup, and directory removal does not imply bookmark removal.
- **Refresh** — allowing JJ to snapshot selected working-copy files before the
  final operation ID is captured. Refresh changes the freshness of input facts;
  it is separate from pure in-memory status derivation. When refresh is skipped,
  possibly stale rows must say so.
- **Creation workflow** — ordered policy, workspace creation, bookmark creation,
  link application, rollback, and switch-state recording.
- **Removal plan** — workspace, directory action, and associated bookmarks known
  before destructive work begins.
- **Associated bookmark** — a local bookmark recorded by `jw` for one workspace.
  `jw` records this relationship and asks before deleting the bookmark.
- **Metadata repair** — an explicit correction of an existing managed record's
  creation base and bookmark association. It does not change JJ workspace, commit,
  bookmark, or working-copy state.
- **Workspace link** — a configured source-target relationship owned by the
  default workspace's `.jwlinks.toml` and optional `.jwlinks.local.toml`.
  `source` is inside each receiving workspace; a relative `target` is resolved
  from that receiver. A link is satisfied by a symlink or ordinary path that
  resolves canonically to the target.
- **Workspace-link health** — the doctor result for one configured link in one
  managed workspace: satisfied, missing, skipped, or conflicting. An optional
  missing target is skipped only when the source is absent or is the correct
  dangling link; an occupied source is conflicting.
- **Shell adapter** — shell-specific syntax that gives the shared `jw` command
  policy native completion and parent-shell directory changes.
- **Herdr container** — a Herdr workspace or tab associated with one JJ workspace
  through a provenance marker.
- **Bookmark integration** — whether the recorded associated bookmark points to
  a revision reachable from resolved trunk. A missing or conflicted bookmark
  cannot establish integration; an absent association is a separate state.
- **Workspace integration** — whether the workspace's publish tip is reachable
  from resolved trunk. It is independent of the associated bookmark's position
  and does not establish freshness of files on disk.
- **Removal eligibility** — the per-target result of revalidating whether a
  workspace and its selected directory and bookmark actions may be removed.
  Shared bookmarks are not eligible for incidental deletion.
- **Removal selection** — the named workspaces chosen for a batch. Selection
  survives search and filter changes; hidden selections remain part of the batch.
- **Reseed** — an explicit in-place refill of an existing workspace using a
  prepared source's CoW files and JJ materialization of differences. It preserves
  root, registration, and lifecycle identity; it is not status refresh or adoption.
- **Prepared source** — a full, conflict-free source workspace whose snapshotted
  tracked tree equals an exact selected baseline. Ignored dependency contents are
  separate installed-source data, not proof of compatibility with a feature tree.
- **Reseed pending record / receipt** — repository-scoped recovery intent for an
  incomplete reseed / provenance of a completed one. Neither changes historical
  creation-base metadata. Resume reads pending intent before target snapshotting.
- **Source preparation** — explicit donor fetch to a pinned upstream baseline and
  configured preparation, performed only by `prepare` from the `[freshen]`
  configuration. It is source-only: it never selects targets, refills, or
  advances workspaces, and it is distinct from status refresh.
- **Prepared commit** — the exact frozen upstream commit `prepare` reports on
  completion. Workers consume it through `reseed --at` without another fetch or
  preparation; it is not standing following state.
- **Preparation batch** — source-keyed write-ahead intent retaining trusted policy,
  a frozen source goal, verified dependency payload, and legacy target plans until
  receipts account for completion. New prepare journals freeze an explicitly empty
  target plan. Partial installation is not a prepared donor.
- **Freshening command sequence** — repository-local ordered argv, normalized from
  one `command` or a `commands` list, never both. Absence means no builder is
  guessed; selected ignored contents remain explicitly existing source data.
- **Completed / in-flight command** — a step durably acknowledged after successful
  exit and source/target checks / an attempted step without that acknowledgement.
  Resume skips completed steps; an in-flight step may retry after partial side
  effects. External command execution is not exactly-once.
- **Ignored-copy policy** — one replacing list selected from CLI, repository-local,
  then root global config, or none. Missing selection inherits; explicit `[]`
  does not. `--no-copy-ignored` disables copying for the run. Recovery uses the
  recorded effective list, not newly introduced defaults.
- **Eligible ignored payload root** — a matched source directory proved ignored
  under frozen source rules and separate from tracked files. Nonignored source
  matches are skipped/reported while viable descendants remain discoverable;
  actually selected roots retain hard target and CoW safety checks.

## Invariants

- One status or planning command derives from one final repository snapshot and
  names its JJ operation ID.
- Trunk resolves to exactly one revision; no ordinary `jw` operation moves a
  trunk bookmark.
- Refresh completes before snapshot capture. Status derivation performs no
  process execution or filesystem mutation.
- Managed metadata records only `jw` lifecycle intent. Unmanaged inference stays
  visibly inferred.
- `jw adopt` is insert-only. `jw repair` requires an existing readable managed
  record and a matching JJ workspace registration at one frozen operation; a usable
  checkout path is not required.
- Metadata repair changes only the recorded creation base and bookmark association.
  Historical fields survive, and a failed or concurrent write leaves the old record
  intact.
- Creation either completes required setup or removes newly created state.
- `jw doctor` checks configured links in every managed workspace. It reports
  missing or conflicting links instead of treating the repository as healthy;
  stale or missing workspace paths are reported separately and link inspection
  is skipped for those paths.
- Link sources stay inside the target workspace.
- Link configuration comes from the default workspace; local entries override
  shared entries with the same source. Relative targets are resolved per
  receiving workspace.
- Removal preserves bookmarks unless the user confirms deletion.
- Publication and cleanup remain separate workflows.
- Core workspace lifecycle and local status remain offline unless explicit donor
  preparation is requested with `prepare`. That requested fetch/install
  is a prerequisite, not optional forge enrichment.
- JSON stdout contains only its versioned document; diagnostics use stderr.
- Herdr provenance survives until Herdr close succeeds.
- Shell adapters implement the same switching behavior.
- Bare reseed selects only the current registered non-primary workspace before
  snapshots. The primary requires literal names or groups; either explicit form
  suppresses the current-target default even if no group matches. No selection
  ever implies all workspaces, and literal names are not navigation tokens.
- Reseed groups can explicitly select registered workspace names or root-path
  patterns. Opt-in warn skips selection misses in one diagnostic, never actual
  target safety failures. Positional names stay strict; no remaining targets is
  an error. Selection does not adopt unrelated checkouts or traverse symlinks.
- Reseed donor selection uses explicit `--from`, configured `source`, then a
  verified primary checkout of the same repository. It never guesses an archive
  location or silently substitutes the current feature checkout. The source
  baseline is CLI `--source-at`, then configured `source_at`, otherwise freshly
  snapshotted donor `@`; tree assertions do not require equal commit identities.
  Differences from a target are handled per path, not treated as a reason to
  discard feature changes.
- Prepare selects/pins upstream before fetch: explicit CLI/local goal, otherwise a
  unique applicable tracked pair, otherwise a unique exact remote at the clean
  source base (including untracked refs), otherwise require `--source-at`. No
  default-main, new following state or operation-log mining. Unpublished source
  work blocks. It is source-only from any registered workspace: no target
  selection, no refill, no `--at`, and no worker-preservation bookmarks; it
  mutates only its selected donor — possibly the caller's own checkout — leaving
  other workspaces untouched. Completion reports the exact commit, including
  `source_commit_id` in JSON. Failure leaves progress in place; fix the donor and
  run prepare again. Every configured command may repeat and must tolerate retry.
- Reseed requires actual CoW capability before clearing contents; it never falls
  back silently. Root and `.jj` survive failure, and partial prefill must not be
  snapshotted as user work. Default-on bookmark coverage is checked before mutation;
  `--allow-unbookmarked` explicitly bypasses only coverage. Failed refills name the
  covered work tip, bookmark, and root: recover there with `jj new <bookmark>` and
  `jj sparse reset`, then re-run reseed. Sparse/mid-refill targets are refused
  before snapshots, never repaired. There are no progress journals, receipts,
  migrations, or `--resume`; old files are ignored without cleanup.

## Milestone-zero command contract

- `jw list`, `jw l`, and `jw ls` keep the historical human output and fast path.
  `--format=json` opts into a schema-versioned repository snapshot; its default
  refresh is `current`, with `none` and `all` available explicitly.
- `jw status [workspace]` selects `@` by default and derives human or JSON output
  from one snapshot. `--refresh=none` does not reconcile a working copy and marks
  its state unknown unless a hazard such as staleness is known.
- `jw doctor` uses a separate versioned report. Every check renders even when
  another fails, and an unhealthy report exits nonzero only after stdout is
  complete.
- Doctor human output uses `PASS`, `WARN`, `FAIL`, and `SKIP` for link health.
  Optional omissions are warnings; missing required targets and conflicts are
  failures. The `workspace-link` diagnostic code is additive within doctor
  schema version 1. `jw status` remains a one-workspace snapshot and does not
  include link health.
- `jw adopt NAME --base REVSET [--bookmark BOOKMARK]` adds managed metadata for
  an existing usable workspace. It never moves revisions or bookmarks and never
  refreshes a working copy. Creation-base and current-revision facts are reported;
  stack analysis remains milestone-one work.
- `jw repair NAME --base REVSET (--bookmark BOOKMARK | --no-bookmark)` updates an
  existing readable record only. `NAME` is literal, the base resolves to exactly
  one revision, and a requested bookmark must already exist locally. It validates
  the workspace and inputs at one frozen operation, preserves historical fields,
  and performs no JJ or working-copy mutation. It has no JSON or `jw status` output
  contract beyond the ordinary human command result.
- `add` and `s` share creation policy. Optional `--from SOURCE` chooses the source;
  otherwise it is the current checkout. `--path DESTINATION` selects one exact
  destination with optional name inference; existing registrations stay authoritative.
- New workspace creation defaults to CoW/JJ-only. Implicit creation snapshots the
  source before freezing its current `@` as the exact base, including a merge `@`.
  Explicit `--at REVSET` keeps the existing exact-one frozen-resolution path.
  Optional CLI `--source-at` snapshots and asserts the donor tree at the same
  operation without selecting the target parent; local reseed source/assertion/
  freshening settings are not creation defaults. Existing switches skip assertions.
  Mode precedence and visible ordinary-checkout fallback are described in the
  [operator guide](README.md#creation-mode-precedence); these creation defaults do
  not change reseed's separate shipped semantics.


## Checkout lifecycle ownership

Native apps own the checkouts and Git registrations they create. JJ adoption keeps
that owner. `jw` lifecycle use is optional; a task uses the actual Git or JJ capability
in its checkout without creating another workspace for agent routing. Each checkout
has one writer, and an orchestrator inspects worker state read-only.

Actual linked Git topology is ineligible for `jw` removal, forgetting (including
`--keep-dir`), manager removal, pruning, and rollback. Schema 2 lifecycle metadata
optionally persists the external checkout root, Git admin directory, and common
directory; schema 1 remains readable and live topology checks cover absent records.
Metadata repair preserves ownership. The owning app removes checkout and Git
registration first; `jw reconcile-external NAME` then explicitly removes leftover
JJ registration and its matching external metadata while retaining commits and
bookmarks. Unverifiable or changed state fails closed.
