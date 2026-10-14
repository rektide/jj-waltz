---
type: Index
title: In-place workspace reseeding design
description: Implementation handoff for repeatable CoW refresh of existing JJ workspaces.
resource: /design/workspace-reseed/README.md
status: proposed
generated:
  by: model:openai/gpt-6-astra
  at: 2026-10-14
sources:
  - resource: "opencode://ses_ec38a6d5fffeiVzJMUoRCYm9YM"
    title: User commissioning and verified no-adoption reseeding design
---

# In-place workspace reseeding

- [Repeatable in-place reflink reseeding](reseed0.gpt6a.md) — recommended
  `jw reseed` interface, prepared-source workflow, ignored dependencies, stock-JJ
  sparse-prefill mechanism, local verification, and implementation acceptance checks.

Existing behavior: [CoW creation](../../README.md#copy-on-write-workspaces).
Integration boundary: [architecture](../../docs/architecture.md#module-map).
