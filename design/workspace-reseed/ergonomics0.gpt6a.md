---
type: Design
title: Ordinary repeated reseeding from the donor already on disk
description: Make the source baseline optional for ordinary CoW refill while preserving target work, explicit dependency policy, and frozen recovery.
resource: /design/workspace-reseed/ergonomics0.gpt6a.md
status: proposed
generated:
  by: model:openai/gpt-6-astra
  at: 2026-10-15
extensions:
  ticket: jjwaltz-reseed-ergonomics
  related_work: [jjwaltz-variants, jjwaltz-variants-defaults, jjwaltz-variants-freshen, jjwaltz-reseed-freshen-default]
verification:
  scope: Installed jw 0.8.0 and jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b on Linux Btrfs, disposable fixtures
  result: Two explicit current-donor reseeds preserved target edits and root/admin inodes; 1 MiB remained shared with separate file inodes; target writes did not alter source
  limits: Explicit --source-at @ exercised; omitted-baseline proposal and freshening changes were not implemented or tested; installed binary version is not a source-revision attestation
sources:
  - resource: "opencode://ses_ec226a15effeUM4c1j7gwYPFHF"
    title: Human commission for CoW-first variants and parallel reseed design; inspection of current donor, baseline, payload, and recovery behavior
  - resource: /docs/reseed.md
    title: Existing one-off, repeat-use, ignored payload, and freshening contracts
  - resource: /design/workspace-reseed/reseed0.gpt6a.md
    title: Original prepared-baseline design and public-JJ refill mechanism
  - resource: /design/workspace-reseed/source-freshen0.gpt61sol.md
    title: Source preparation ownership and explicit deferral of optional borrowing baselines
  - resource: /src/reseed/engine.rs
    title: Baseline requirement, frozen source capture, target preservation, registration, and journal implementation inspected at delivery baseline 561d7462
  - resource: /src/reseed/selection.rs
    title: Snapshot-free literal-name and group selection from actual registrations
  - resource: /src/reseed.rs
    title: Request and report interfaces
  - resource: /src/cli.rs
    title: Existing reseed options and their separation from add/s
  - resource: /docs/architecture.md
    title: Separate lifecycle and public-JJ reseed execution boundaries
---

# Ordinary repeated reseeding

## Recommendation

Make **`jw reseed fix` borrow the verified primary donor's current tracked
state, preserving `fix`'s current work**, without demanding a source-baseline
argument. Keep the existing source override, configuration, dependency selections,
batch selection, reset option, and recovery machinery.

The implementation delta is optional baseline selection before the existing
frozen plan. It is not a new refill engine or a unification of creation and
reseed transactions. This document proposes that delta; the command still
requires a baseline today. Implementation has not been commissioned here.

## Where we start

The current engine already does the important part correctly:

- Names resolve to actual registered workspace roots, including locations under
  `~/src` outside the primary checkout's directory. Basenames do not select or
  rename existing workspaces.
- Donor precedence is CLI `--from`, local `source`, verified primary checkout.
  The primary is proven by registration and ownership of the actual JJ store,
  not merely by a workspace called `default`.
- Each target keeps its root, `.jj`, identity, bookmarks, and snapshotted work.
  Source/target tree differences are materialized by JJ; only equal paths are
  borrowed. The source is not blindly overlaid onto feature work.
- Real CoW capability is required before clearing a target. There is no ordinary
  copy fallback in reseeding. Repeating the operation renews sharing even when
  tracked contents have not changed.
- Default behavior preserves other ignored/untracked target contents. Configured
  `copy_ignored` roots are explicitly replaced by private CoW data. Existing
  `--no-copy-ignored` overrides that policy for one run.
- Pending reseeds and source preparation are recovered before ordinary snapshots
  can mistake an incomplete filesystem for intended work.

The friction is at `engine.rs`'s source selection: it requires `--source-at` or
configured `source_at`, then asserts that the donor's tracked tree matches that
revision. This was useful for distributing a prepared upstream baseline, but is
unnecessary when the request is simply to borrow the donor already on disk.

**An explicit version of the proposed behavior already works today:**

```sh
jw reseed fix --source-at @
```

`@` is resolved in the selected donor, not the target or invoking variant. The
proposed default makes that intention available without a mandatory argument.

## Ordinary interface and precedence

```sh
jw reseed fix                        # primary donor as-is; preserve fix
jw reseed fix other                  # same donor, separate preserved targets
jw reseed fix --from /path/to/donor  # explicit same-repository donor
jw reseed fix --source-at dev@origin # assert a prepared baseline
jw reseed fix --source-at @          # override a configured baseline for one run
jw reseed --group workers            # existing repeatable batch selection
```

These are examples of the proposed ordinary contract; only omission of the
otherwise-required baseline is new. Explicitly selected source/target restrictions
continue to apply.

| Choice | Ordinary reseed rule |
|---|---|
| Target selection | Existing literal registered names and configured groups; no implicit all-workspaces selection |
| Donor root | `--from` > configured `source` > verified primary checkout |
| Donor baseline | `--source-at` > configured `source_at` > donor's freshly snapshotted `@` |
| Desired target tree | Preserve target's freshly snapshotted tree; `--at` explicitly selects a new base |
| Ignored payload | CLI replacement selection > configured selection > none; `--no-copy-ignored` disables copying |
| Network/install | None without existing explicit freshening activation |

CLI relative donor paths remain caller-relative; configured relative paths remain
primary-relative. Existing baseline configurations keep their assertion semantics;
they are not silently ignored after the change. Remove `source_at` to adopt the
new ordinary default, or override it with `--source-at @` for one invocation.

No new navigation aliases or destination flag are needed for this slice. `fix`
is the same registered workspace identity that creation recorded and other
commands use. Reseed's existing literal selection contract stays explicit,
including in recovery.

## The useful differences from creation

Creation starts a new workspace from the chosen source state; reseed preserves
an existing target state and borrows only content equal to it. Making both
default to the donor's revision would turn a refill into a reset.

Creation's default source is where the operator stands. Reseed's default donor
is the primary checkout, so invoking `jw reseed fix` while working inside `fix`
does not try to clone the target from itself. An explicit `--from` continues to
override that choice, with existing self/overlap checks.

The original source of creation does not become an implicit permanent donor.
Creation provenance records what happened; `.jwreseed.local.toml` declares repeat
policy. A one-off `--from` does not rewrite future behavior or silently become a
last-used default.

## Dependencies and repeat policy

Keep the existing ignored local configuration as the persistence mechanism:

```toml
# .jwreseed.local.toml in the primary checkout
# source is optional when the primary is the intended donor.
copy_ignored = ["node_modules", "packages/*/node_modules"]

[groups]
workers = ["fix", "other"]
```

This is already supported. With the proposed optional baseline, no `source_at`
entry is needed for ordinary borrowing. An explicit configured baseline remains
valuable when the donor must have a particular tracked tree.

Do not infer dependency roots, automatically run an installer, or turn a normal
invocation into a configuration write. Replacing a selected ignored root remains
the operator's declared action. Retain the existing ignored-root checks and
payload digest. Matching tracked files, with or without a baseline assertion,
does not prove that an installed dependency tree matches a feature's manifests.

Whether creation should also expose private ignored-payload copying is a separate
capability question, not solved by changing reseed defaults. This design neither
adds source symlinks nor labels linked dependencies as private CoW data.

## Minimal implementation boundary

1. Keep effective `source_at` optional during ordinary selection. In freshening,
   continue to require an explicit/configured source goal. Do not give freshening
   a hidden `@` fallback or change its source-advancement semantics.
2. After the existing pending checks, donor/root checks and donor snapshot, freeze
   `source_wc` at the recorded source operation. With no baseline, set
   `source_commit = source_wc`. Otherwise resolve the effective assertion in the
   donor and retain the current tracked-tree equality check.
3. Run the existing conflict, disk-content, source-stamp, ignored-payload and
   target validations. Freeze target snapshots at the final shared operation and
   retain the existing source-unchanged check across that capture.
4. Feed the same exact source/target IDs into the existing `Pending` plan. Resume
   must read those frozen IDs; it must not reevaluate the new default `@` or a
   changed configuration. No journal-schema change is needed merely to represent
   `source_commit == source_wc_commit`.
5. Update help and operator examples to distinguish optional ordinary assertions
   from required freshening goals. Existing reports already expose the source ID,
   number of clone paths, JJ-materialized paths, and ignored selections; reuse
   them rather than invent a reporting subsystem.

An ordinary donor may contain its own snapshotted edits. That is acceptable:
source-only edits reduce sharing for those paths, not overwrite different target
work. If few paths match, the refill may offer little sharing; existing path
counts should tell that truth. No minimum shared percentage is promised.

There is a separate JJ graph condition: if the edited donor change is an ancestor
of the target, snapshotting it can rebase that target and make its working copy
stale. The existing engine stops rather than guessing how to refresh it. Retain
that behavior and the native `jj workspace update-stale` guidance; optional
baseline selection is not automatic graph reconciliation. Keeping a dedicated
donor change separate from the target's mutable ancestors avoids this coupling.

Do not replace reseed's snapshot-free inventory with creation's inventory merely
to reduce duplication. Incomplete targets require discovery before any ordinary
snapshot. Shared primary-identity helpers are worth extracting only when they
preserve that boundary and actually simplify the implementing change.

## Delivery and remaining choices

Recommended delivery is one bounded implementation of ordinary current-donor
selection, followed by its operator documentation. Exercise the actual behavioral
difference: a dirty donor and dirty target, two repeated runs, explicit baseline
mismatch, and frozen resume. Establish that the target work stays intact and
equal regular files really share extents with independent inodes. Freshening's
missing-goal error and existing goal behavior must remain intact.

The recommendation leaves no new user-facing mode to select. Decisions for human
review are whether to accept this optional-baseline default and whether any
particular project should retain a configured upstream-baseline assertion.
Freshening implementation and persistent freshen activation stay with their
existing/future tickets.

## Local evidence

An installed-binary experiment in `.test-agent/reseed-design/` invoked
`jw reseed fix --source-at @` from inside the target twice. Donor and target had
different edits to `work.txt`; both edits survived each pass in their own
checkout. `shared.bin` had distinct file inodes, while Btrfs reported 1 MiB
shared and zero exclusive bytes on both passes. Target root and `.jj` inodes
survived, and a subsequent target-file write left the donor unchanged.

The first fixture deliberately retained in that scratch area exposed the stale
condition above: editing the donor parent led to a native stale-workspace error
before refill. A second fixture used a new donor change, separate from the
target's parent, and completed both passes. This evidence supports the existing
refill mechanism and clarifies its graph boundary; it does not prove the
unimplemented optional-baseline default or general release compatibility.

## Cross-references

- [Original reseed design](reseed0.gpt6a.md#content-contract) supplies the retained
  keep-target and private-payload contract; this proposal relaxes mandatory
  operator baseline declaration, not physical source verification.
- [Source-freshening design](source-freshen0.gpt61sol.md#explicit-interface-first-delivery)
  explicitly left optional borrowing baselines for separate work. This fills that
  gap while retaining freshening's distinct post-fetch goal.
- [Current operator guide](../../docs/reseed.md) remains authoritative for shipped
  syntax; its configured groups, donor precedence, and ignored-copy controls are
  reused rather than replaced.
- [Architecture](../../docs/architecture.md#module-map) explains why lifecycle
  creation and journaled public-JJ refill remain separate execution paths even
  when their naming and source terminology agree.
