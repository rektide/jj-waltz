---
type: Index
title: In-place workspace reseeding design
description: Implementation handoff for repeatable CoW refresh of existing JJ workspaces.
resource: /design/workspace-reseed/README.md
status: proposed
generated:
  by: model:openai/gpt-6-astra
  at: 2026-10-14
updated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
sources:
  - resource: "opencode://ses_ec10d60dbffencx7PCZnh1dnjg"
    title: Connected implemented up orchestration to native selection, command sequences, target history, and operator guidance
  - resource: "opencode://ses_ec11dfc0dffeV1y2Am4AyutJlT"
    title: Connected delivered target-history preservation and exact bookmark replay foundation to its remaining up integration boundary
  - resource: "opencode://ses_ec1394461ffeEXBHM582vTs9VM"
    title: Connected delivered source-assertion/current-target semantics and creation payload integration to maintained guides
  - resource: "opencode://ses_ec1a8d43bffe02bXuEoA7hEW3l"
    title: Linked native source, command/recovery, and shared ignored-copy foundations with their current integration boundaries
  - resource: "opencode://ses_ec226a15effeUM4c1j7gwYPFHF"
    title: Commissioned up and shared copying implementation after repeat-reseed design and independent JJ preservation/tracking research
  - resource: "opencode://ses_ec38a6d5fffeiVzJMUoRCYm9YM"
    title: User commissioning and verified no-adoption reseeding design
  - resource: "opencode://ses_ec37b01d2ffe9ytO0jRsdmoyXA"
    title: Implementation delivery, verified explicit source freshening, and separately deferred default activation
  - resource: "opencode://ses_ec37a1a86ffeYXsS5j3A7Jng0t"
    title: Designed and implemented wildcard groups with configurable unmatched selection policy
---

# In-place workspace reseeding

- [Commissioned up and shared copying policy](up0.gpt6a.md) — current handoff:
  native upstream selection, sequential local freshening commands, unpublished
  target bookmarks, global ignored-copy policy, and symmetric command controls.
  Replaces the proposal to keep adding freshening flags to reseed/creation.
- [Native source foundation](../../docs/architecture.md#native-freshening-source-foundation) —
  implemented operation-bound upstream identity and post-fetch graph proofs, with
  disposable native JJ verification. Includes frozen explicit not-yet-imported
  `branch@remote` intent with unknown IDs and deferred publication checks.
  Connected to `up` before preparation/fetch, with v3 frozen selection.
- [Local command sequences and recovery](../../docs/architecture.md#local-freshening-commands-and-recovery) —
  implemented `[freshen]` single/ordered argv policy in the existing preparation
  engine, durable per-step progress, deprecated `[prepare]` alias, and v1 journal
  compatibility. Activated through `up`; legacy preparation still uses `reseed --resume`.
- [Target history preservation](../../docs/architecture.md#target-history-preservation-foundation) —
  implemented effective work-tip/selected-fetched-goal ancestry, shared UTC
  timestamp-name allocation, and create-only exact-pair replay in existing target
  and preparation intent. Public up freezes preservation and opt-out before mutation;
  existing feature bookmarks and opt-out JJ history are left intact.
- [Shared ignored-copy policy and selection](../../docs/architecture.md#shared-ignored-copy-policy-and-selection) —
  root global/local/CLI precedence with explicit empty lists, one compiled
  selection traversal, skipped nonignored source matches, and hard selected-root
  guards. Reseed/preparation and add/s new-workspace payloads use it. Actual creation
  payload selection rejects nested donor destinations before creation.
- [Ordinary repeated reseeding](ergonomics0.gpt6a.md) — design evidence for optional baseline
  and implicit current target for borrowing the current donor, retaining target work, existing repeat-policy
  configuration, and frozen recovery. Implemented for `jjwaltz-reseed-ergonomics`;
  see the [operator contract](../../docs/reseed.md#one-off-use) for delivered
  default borrowing/current-target selection and [creation assertions](../../README.md#copy-on-write-workspaces).
  The later up commission settles the earlier review
  junction: ordinary reseed keeps target work; up advances to the prepared goal.
- [JJ new preserves unchanged files](jj-new-preservation0.gpt61sol.md) — inspected
  checkout algorithm and native Btrfs evidence: ordinary advancement preserves
  equal-file inodes and sharing, but does not renew sharing for rewritten files.
- [Native tracked bookmarks for donor updates](bookmark-tracking0.gpt61sol.md) —
  existing local/remote tracking and graph relationships preceding commissioned `up`,
  including already-fetched movement, unpublished work, and ambiguous source
  lines. No new persistent following state or operation-log inference.
- [Repeatable in-place reflink reseeding](reseed0.gpt6a.md) — recommended
  `jw reseed` interface, prepared-source workflow, ignored dependencies, stock-JJ
  sparse-prefill mechanism, local verification, and implementation acceptance checks.
- [Wildcard reseed groups](groups0.gpt61sol.md) — design evidence for name/path patterns,
  optional warn-and-skip for unrelated sibling checkouts, and one aggregate
  warning line. Implemented under `jjwaltz-reseed-groups`; current interface:
  [wildcard groups](../../docs/reseed.md#wildcard-groups).
- [Freshen the donor before reseeding](source-freshen0.gpt61sol.md) — rationale and
  local delivery checkpoint for explicit fetch, source advancement, configured
  dependency preparation, and recoverable handoff. Its deferred default-on
  alternative is superseded by the up commission; use the operator guide for
  the still-current CLI syntax until that migration lands.

Existing behavior: [CoW creation](../../README.md#copy-on-write-workspaces).
Integration boundary: [architecture](../../docs/architecture.md#module-map).

Operator-facing implementation guidance: [reseed an existing workspace](../../docs/reseed.md)
starts with one-off named use and covers optional configuration, explicit ignored
replacement, full JJ-only target restrictions, and pending-operation recovery.
The model-suffixed proposal above remains the design and experiment evidence,
not a mutable command reference.
