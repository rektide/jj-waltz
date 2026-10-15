---
type: Design
title: Wildcard reseed groups with explicit unmatched policy
description: Select registered workspaces by name or path patterns, optionally skipping unrelated sibling checkouts with one aggregate warning.
resource: /design/workspace-reseed/groups0.gpt61sol.md
status: proposed
generated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
sources:
  - resource: "opencode://ses_ec37a1a86ffeYXsS5j3A7Jng0t"
    title: User request for wildcard groups, mixed opencode-prefixed directories, configurable skipping, and one-line diagnostics; inspected current selection implementation
  - resource: /src/reseed/engine.rs
    title: LocalConfig, literal selections, inventory loading, target validation, and existing ignored-directory glob matcher
  - resource: /src/cli.rs
    title: Reseed arguments, resume conflicts, and report rendering
  - resource: /src/reseed.rs
    title: Request and report interfaces
  - resource: /docs/reseed.md#optional-shortcuts
    title: Current literal groups and configuration precedence
  - resource: /docs/architecture.md#in-place-reseeding
    title: Snapshot, identity, ownership, and pending-recovery boundaries
  - resource: /CONTEXT.md#invariants
    title: Explicit selection and preservation contracts
  - resource: /design/workspace-reseed/reseed0.gpt6a.md
    title: Original named-batch design and source/target restrictions
---

# Wildcard reseed groups

## Situation and scope

`jw reseed` is implemented, but `LocalConfig.groups` is currently a map of
literal-name arrays. `selections()` concatenates and deduplicates those names;
it does not interpret wildcards or paths. Unknown names subsequently fail root
lookup. There is no configurable warn-and-skip selection policy. Wildcards in
`copy_ignored` are a separate existing feature, not workspace selection.

The operator wants to select `~/src/opencode-*` even though that directory family
also contains unrelated projects. Matching a pathname is not proof that it is a
workspace of this repository. Select existing registered workspaces only; never
initialize, adopt, create, or mutate unrelated matches.

This document proposes selection changes only. Examples below are **not yet
supported**. It does not change reseed content policy, graph behavior, or ownership.
Existing uncommitted donor/primary-discovery work was inspected, not modified.

## Recommended interface

Keep existing arrays literal and add an extended group table:

```toml
# In the default/primary workspace's ignored .jwreseed.local.toml.
# Existing source, source_at, and copy_ignored settings remain unchanged.

[groups]
workers = ["worker-a", "worker-b"]

[groups.opencode]
path_globs = ["~/src/opencode-*"]
on_unmatched = "warn"
```

```sh
jw reseed --group opencode --dry-run
jw reseed --group opencode

# Override group policy for this invocation, including legacy array groups:
jw reseed --group opencode --on-unmatched error --dry-run
```

Extended groups accept these optional fields, and reject unknown fields:

| Field | Meaning |
| --- | --- |
| `names` | Literal JJ workspace names, just like today's array form. |
| `name_globs` | Patterns over registered JJ workspace names, not directory basenames. |
| `path_globs` | Patterns over filesystem paths, resolved to exact registered workspace roots. |
| `on_unmatched` | `"error"` (default) or `"warn"`; applies only to this group's selection misses. |

At least one selector must be present and nonempty. The three selector lists are
unioned. For example, `name_globs = ["ocw-*"]` works when workspace names, rather
than their locations, share a prefix. Do not guess which namespace a string uses.
The legacy array `["opencode-*"]` continues to mean one literal workspace name.

Policy precedence is CLI `--on-unmatched` > group `on_unmatched` > `error`.
No repository-wide permissive default is needed: opting this broad group into
skipping must not weaken another group. Positional names always remain strict,
even with `--on-unmatched warn`. An unknown group, invalid policy, empty group,
or malformed pattern is always a configuration/usage error.

`--group` remains repeatable. Combine groups and positional names, deduplicate by
exact registered workspace identity, and execute each target once. If the same
miss is requested through both strict and permissive selectors, strict wins;
an explicit positional typo cannot be hidden by a group.

## Pattern and path rules

- Name patterns are case-sensitive whole-name matches supporting `*` and `?`.
  They match the registration inventory, including registrations with missing
  roots, so stale workspaces cannot disappear silently.
- Path patterns support literal components, `*`, and `?`; wildcards do not cross
  separators. For the initial delivery reject `**`, character classes, braces,
  backslash escapes, and `..` components rather than inventing undocumented
  recursive or shell semantics. The separate `copy_ignored` syntax is unchanged.
- Expand a leading `~/` using the user's home directory. Reject `~otheruser`;
  do not expand environment variables, run shell code, or perform brace expansion.
  Absolute paths stay absolute; relative paths start at the configuration's
  default/primary workspace, never the calling subdirectory.
- Enumerate only the path components specified; do not recursively search inside
  a matching candidate for `.jj`. Never traverse symlink components. A symlink
  candidate is a selection miss, not an alias granting target identity.
- A matched real directory selects a workspace only when it is the exact root
  of one registered workspace in this repository, with matching native identity.
  Do not infer the workspace name from the basename. A child directory inside a
  workspace, unrelated Git checkout, independent JJ repository, or plain directory
  is not a target. Unmanaged but natively registered workspaces remain selectable.
- Sort matches within each pattern. Preserve positional order, group invocation
  order, and field order `names`, `name_globs`, `path_globs`; then deduplicate.
  Freeze the resulting identity set for the run. Do not re-expand during execution.

## What may be skipped

With effective `warn`, selection misses are skipped and collected:

| Selection miss | Diagnostic reason |
| --- | --- |
| Unknown group-member literal name | `unknown workspace` |
| Name/path pattern with no candidates | `no matches` (report the expression) |
| Matching path that is not a registered root of this repository | `not a workspace of this repository` |
| Matching file or symlink instead of a real root directory | `not a real workspace directory` |

A path pattern which finds three unrelated checkouts reports those three paths,
not a redundant fourth "no matches" message. A pattern with no filesystem
candidates reports the pattern once. Directories outside the pattern are neither
examined nor listed: "non-matches" here means candidates that fail workspace
membership, not every other sibling directory.

**These are not skip-eligible:** an actual selected workspace with a stale/missing
registered root, inconsistent `.jj` identity, conflicting registrations, wrong
repository pointer, external ownership, unsupported colocation/sparsity/monitor,
conflicts, pending reseed, or failed CoW probe. Primary/source selection and
overlap also remain hard errors. Permission/I/O failures are errors, not evidence
that a candidate is unrelated. Validate actual targets using the existing safety
pipeline; never catch all target-validation errors and relabel them as misses.

Inventory must retain missing-root registrations separately from usable roots.
Today's `Inventory::load()` omits failed root lookups/canonicalizations, so it
cannot alone distinguish an unknown name from a broken registered target.
If an enumerated path corresponds to a recorded root but native validation fails,
fail rather than classifying it as an unrelated directory. Revalidate selected
identities before snapshot and before destructive execution as today.

## One-line diagnostics and exit behavior

Accumulate all skip-eligible misses across all groups before emitting anything.
Deduplicate by candidate (or unmatched expression) and reason, retaining selector
origins internally; sort deterministically. Render **one physical stderr line**:

```text
warning: skipped 2 unmatched group candidates: "/home/me/src/opencode-notes" (not a workspace of this repository), "/home/me/src/opencode-tools" (not a workspace of this repository)
```

List every skipped candidate, without truncation or per-directory warning lines.
Escape quotes, newlines, control characters, and non-UTF-8 path bytes so unusual
filenames cannot split or forge diagnostics. Diagnostic escaping must preserve
distinct paths rather than using lossy conversion.

- Emit the aggregate warning once before normal reseed planning/snapshots, so it
  remains visible even if a later preflight fails. Other unrelated existing
  diagnostics need not be collapsed into this line.
- With strict policy, aggregate selection errors too and abort the batch before
  source/target snapshotting. Warn-policy skips still get their one warning line.
- Warn-policy skips do not make an otherwise successful nonempty run fail.
  Retain skipped candidates as structured selection data, not failed targets.
- If no targets remain, exit nonzero with `no workspaces selected; nothing
  reseeded`. Still emit the single skip warning. Never fall back to all workspaces
  or declare an all-skipped run a success.
- Dry-run performs identical expansion and diagnostics and shows each selected
  name/root and its selector origin. Existing dry-run planning can snapshot
  working copies; selection-only expansion must not.
- Resume continues requiring explicit literal names and the frozen pending plans.
  Reject groups and `--on-unmatched` with `--resume`; never discover a different
  recovery target set from a mutable glob.

## Implementation slice

1. Introduce an untagged array-or-table group enum in `LocalConfig`, a typed
   `UnmatchedPolicy`, and optional CLI/request override. Keep legacy array parsing
   and positional literal semantics unchanged.
2. Extract group resolution into `src/reseed/selection.rs`. Return a structured
   `SelectionPlan` containing selected names/roots/origins, warnings, and strict
   issues. Keep matching/classification independent of reseed mutation. Reuse the
   existing `*`/`?` matcher after extraction, not the ignored-directory traversal
   with its different recursion and containment contract.
3. Extend snapshot-disabled inventory to retain all registered names and recorded
   roots or lookup failures. Resolve membership against that one inventory; do not
   run normal JJ commands in candidate directories to discover their contents.
   Membership probes, if needed, must remain snapshot-disabled and root-specific.
4. Render the aggregate diagnostic at the selection boundary, before returning
   selection errors or entering ordinary planning. Use a diagnostic callback from
   the CLI so library callers can capture it even when later planning fails;
   preserve the existing `run(request)` convenience interface with a stderr sink.
   Carry structured skips on successful reports without altering target statuses.
5. Feed only selected identities into pending detection and the existing
   validate/freeze/execute pipeline. Keep the operation lock and all identity,
   ownership, content, and source checks. Selection misses must never create
   pending records or receipts for unrelated paths.
6. Update the operator guide, CLI help, and examples when implemented. No new
   cleanup, fleet registry, background refresh, cross-repository reseed, or graph
   operation is part of this delivery.

## Acceptance checks

- Legacy groups, literal wildcard-containing names, strict positional names,
  repeatable groups, deterministic deduplication, unknown fields, and precedence.
- Name and path patterns resolve the intended same-repository workspace when its
  JJ name differs from the directory basename; unmanaged registrations work.
- A mixed `opencode-*` fixture contains two targets, a plain directory, an
  unrelated Git checkout, and an independent JJ checkout. Warn selects only the
  two targets, reports all three unrelated paths in exactly one warning line,
  and leaves unrelated data and Git/JJ administrative state unchanged.
- Strict mode with the same fixture fails before any working-copy snapshot or
  content change. A missing registered target remains an error under warn.
- Zero matches, all skipped, empty/invalid group, explicit typo combined with
  warn groups, symlink aliases, permission failures, and paths inside a workspace.
- Selected primary/source, colocated/external target, pending record, and changed
  identity remain hard failures under warn; no unsafe target is quietly omitted.
- Home and relative expansion are independent of caller workspace/subdirectory;
  no shell/environment expansion, symlink traversal, or recursive scan occurs.
- Warning escaping handles newline/control/non-UTF-8 filenames without truncating
  candidates; mixed group policies still emit at most one skip warning line.
- Warning survives later source/preflight failure; dry-run and real execution
  select the same frozen set; resume rejects selection-policy changes.

## Connections

- [Current group usage](../../docs/reseed.md#optional-shortcuts) is the mutable
  operator home; keep current literal behavior distinct from this proposal.
- [Original reseeding design](reseed0.gpt6a.md#repeated-use-save-the-boring-inputs)
  established named batches. This extends their selection without relaxing the
  [source/target content contract](reseed0.gpt6a.md#content-contract).
- [Reseeding architecture](../../docs/architecture.md#in-place-reseeding) owns
  snapshot and recovery ordering; discovery must not snapshot a half-filled tree.
- [Domain invariants](../../CONTEXT.md#invariants) require explicit selection and
  preservation. Patterns are explicit selectors, not permission to adopt or
  modify every similarly named checkout.
