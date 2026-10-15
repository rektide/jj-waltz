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
  - resource: "opencode://ses_ec1a8d43bffe02bXuEoA7hEW3l"
    title: Added the native source foundation implementation pointer and distinguished current commission from superseded freshening/default alternatives
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
  disposable native JJ verification. Not yet connected to the existing CLI;
  the pointer records the explicit not-yet-imported-ref integration edge.
- [Ordinary repeated reseeding](ergonomics0.gpt6a.md) — proposed optional baseline
  and implicit current target for borrowing the current donor, retaining target work, existing repeat-policy
  configuration, and frozen recovery. Design for `jjwaltz-reseed-ergonomics`;
  not yet implemented. The later up commission settles the earlier review
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
