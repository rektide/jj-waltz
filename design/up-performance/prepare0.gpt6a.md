---
type: Design
title: Separate donor preparation from workspace refill
description: Replace up with source-only prepare and prioritize the shared reseed engine's process and recovery costs.
resource: /design/up-performance/prepare0.gpt6a.md
status: implementation-in-progress
generated:
  by: model:openai/gpt-6-astra
  at: 2026-10-17
sources:
  - resource: "opencode://ses_eb8a7487affeE2smndZ914OQMS"
    title: Operator selected prepare, explicitly deleted up, and prioritized core reseed performance over build and command-policy expansion
  - resource: /design/up-performance/handoff0.glm53m.md
    title: Incident evidence identifying per-file verification, phantom recovery, repeated scans, and test shim recursion
  - resource: /src/reseed/engine.rs
    title: Existing donor-equal file selection, per-file verification, and write-ahead recovery phases
  - resource: /src/reseed/prepare.rs
    title: Existing native upstream preparation and persisted command progress
  - resource: /docs/reseed.md
    title: Maintained keep-feature refill and source preparation contracts before the command split
---

# Separate donor preparation from workspace refill

## Direction

The operator selected **`prepare` and deletion of `up`**, not a deprecated alias.
Preparation selects/fetches upstream, prepares the donor, and reports an exact
commit. It has no worker selection or worker advancement. Invocation from a
variant must have the same source-only meaning as invocation from the primary.

`reseed` already provides the desired copying operation: retain the selected
feature tree, reflink paths equal to the donor, and materialize differences through
JJ. `reseed --at COMMIT` deliberately starts a worker at a chosen baseline. Feature
rebasing remains an ordinary JJ operation; checking out an old feature after a new
baseline does not rebase it.

An exact prepared commit lets workers consume one baseline without requesting
another fetch/preparation. This split does not require receipt adoption, automatic
build reuse decisions, a new artifact cache, or a new workspace placement policy.
Those were possible follow-ons in the incident handoff, not prerequisites for
fixing the core algorithm.

## First performance slice

1. Replace normal intact-workspace per-file byte extraction with bounded JJ
   tracked-tree verification. Keep auto-tracking and stale-workspace updates out
   of proof snapshots. This adopts JJ's tracked-tree semantics, including its
   normal stat cache, rather than promising an independent raw-byte audit.
2. Use recorded target mutation phase to distinguish untouched preparation from
   interrupted refill. A preparation ID alone does not justify reconstruction.
3. Preserve reconstruction before snapshots of partially materialized targets.
   An empty diff under an incomplete sparse selection cannot prove a full tree.
4. Keep the existing per-command tracked-tree invariant; optimizing its proof
   does not require allowing one command to modify files for another to restore.

Repeated payload hashing, ignore-rule construction, recursive stamps, journal
size, and per-file reflink costs remain separate measurable costs. Removing the
dominant subprocess multiplier does not itself establish a five-second refill.

## Persisted state

Deleting a command does not erase pending target mutations recorded by that
command. New preparation must never create target plans. Legacy target recovery
belongs behind `reseed --resume`, retaining its recorded history and bookmark
intent. `prepare --resume` must not reinterpret a legacy target batch as a
source-only operation. Existing preparation configuration stays in `[freshen]`;
configuration renaming is independent of this command change.

## Connections

- [Incident handoff](handoff0.glm53m.md): evidence and original fix proposals;
  this decision replaces its combined-up workflow recommendation.
- [Reseed operator guide](../../docs/reseed.md): maintained command behavior and
  recovery contract, updated with implementation.
- [Reseed design index](../workspace-reseed/README.md): preceding source-selection,
  sparse refill, and target-history work whose implementations are reused.
