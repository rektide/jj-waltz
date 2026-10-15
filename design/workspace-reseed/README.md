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
  - resource: "opencode://ses_ec38a6d5fffeiVzJMUoRCYm9YM"
    title: User commissioning and verified no-adoption reseeding design
  - resource: "opencode://ses_ec37b01d2ffe9ytO0jRsdmoyXA"
    title: Implementation delivery, one-off guidance, and source-freshening design with separate default activation
  - resource: "opencode://ses_ec37a1a86ffeYXsS5j3A7Jng0t"
    title: Added wildcard-group design with configurable unmatched selection policy
---

# In-place workspace reseeding

- [Repeatable in-place reflink reseeding](reseed0.gpt6a.md) — recommended
  `jw reseed` interface, prepared-source workflow, ignored dependencies, stock-JJ
  sparse-prefill mechanism, local verification, and implementation acceptance checks.
- [Wildcard reseed groups](groups0.gpt61sol.md) — proposed name/path patterns,
  optional warn-and-skip for unrelated sibling checkouts, and one aggregate
  warning line; preserves strict target safety and current literal array groups.
- [Freshen the donor before reseeding](source-freshen0.gpt61sol.md) — design-only
  fetch, source advancement, and configured dependency preparation under explicit
  `--freshen`; default-on activation is a separate dependent delivery. No live
  source preparation or feature implementation was commissioned here.

Existing behavior: [CoW creation](../../README.md#copy-on-write-workspaces).
Integration boundary: [architecture](../../docs/architecture.md#module-map).

Operator-facing implementation guidance: [reseed an existing workspace](../../docs/reseed.md)
starts with one-off named use and covers optional configuration, explicit ignored
replacement, full JJ-only target restrictions, and pending-operation recovery.
The model-suffixed proposal above remains the design and experiment evidence,
not a mutable command reference.
