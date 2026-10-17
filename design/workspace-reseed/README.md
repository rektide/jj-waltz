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
  by: model:zai-coding-plan/glm-5.3-flash#high
  at: 2026-10-17
sources:
  - resource: "opencode://ses_eb8a7487affeE2smndZ914OQMS"
    title: Connected the delivered performance fixes and native end-to-end measurements
  - resource: "opencode://ses_eb899cdcdffecJnB7J1dSGs4ci"
    title: Indexed the prepare/reseed command-split decision while preserving the commissioned-up draft as historical evidence
  - resource: /design/up-performance/prepare0.gpt6a.md
    title: Current decision record — delete public up, source-only prepare, reseed-only worker copying
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

- [Delivered performance fixes](../up-performance/measurement1.gpt6a.md) — native
  prepare, preview, and two-worker refill timings, implementation checkpoints,
  verification scope, and the maintained assessment tools.
- [Prepare/reseed command split](../up-performance/prepare0.gpt6a.md) — **current
  decision**: the public `up` command is deleted in favor of source-only
  `prepare` (select/fetch upstream, prepare donor, report the exact frozen
  commit; never chooses or advances workers), while `reseed` keeps worker
  copying semantics (`--at` resets deliberately; feature rebasing stays JJ).
  Prioritizes the shared engine's core performance over build-cache design.
  Supersedes the combined-up commission below, whose draft remains as evidence.
- [Commissioned up and shared copying policy](up0.gpt6a.md) — historical draft
  of the removed combined up: native upstream selection, sequential local
  freshening commands, unpublished target bookmarks, global ignored-copy policy,
  and symmetric command controls. Retained as design evidence for the
  foundations `prepare` and legacy `reseed --resume` recovery reuse; its
  combined fetch+advance workflow was never released and is superseded by the
  split above.
- [Native source foundation](../../docs/architecture.md#native-freshening-source-foundation) —
  implemented operation-bound upstream identity and post-fetch graph proofs, with
  disposable native JJ verification. Includes frozen explicit not-yet-imported
  `branch@remote` intent with unknown IDs and deferred publication checks.
  Connected to `prepare` before preparation/fetch, with v4 frozen selection;
  legacy v3 journals resume through `reseed --resume`.
- [Local command sequences and recovery](../../docs/architecture.md#local-freshening-commands-and-recovery) —
  implemented `[freshen]` single/ordered argv policy in the existing preparation
  engine, durable per-step progress, deprecated `[prepare]` alias, and v1 journal
  compatibility. Activated through `prepare`; legacy preparation and removed-up
  batches still use `reseed --resume`.
- [Target history preservation](../../docs/architecture.md#target-history-preservation-foundation) —
  implemented effective work-tip/selected-fetched-goal ancestry, shared UTC
  timestamp-name allocation, and create-only exact-pair replay in existing target
  and preparation intent. No new command plans preservation anymore; the
  retained semantics serve legacy v3 journals recovered through `reseed --resume`,
  leaving existing feature bookmarks and opt-out JJ history intact.
- [Shared ignored-copy policy and selection](../../docs/architecture.md#shared-ignored-copy-policy-and-selection) —
  root global/local/CLI precedence with explicit empty lists, one compiled
  selection traversal, skipped nonignored source matches, and hard selected-root
  guards. Reseed/preparation, prepare payload verification, and add/s
  new-workspace payloads use it. Actual creation
  payload selection rejects nested donor destinations before creation.
- [Up performance incident and handoff](../up-performance/handoff0.glm53m.md) —
  measurement evidence behind the split: per-file verification census, phantom
  fresh-recovery, journal costs, and the test-shim recursion. Superseded fix
  plan items that depended on a combined up; the split adopts its core slices.
- [Ordinary repeated reseeding](ergonomics0.gpt6a.md) — design evidence for optional baseline
  and implicit current target for borrowing the current donor, retaining target work, existing repeat-policy
  configuration, and frozen recovery. Implemented for `jjwaltz-reseed-ergonomics`;
  see the [operator contract](../../docs/reseed.md#one-off-use) for delivered
  default borrowing/current-target selection and [creation assertions](../../README.md#copy-on-write-workspaces).
  The review junction is now settled by the command split: ordinary reseed keeps
  target work or resets deliberately via `--at`; preparation is `prepare`.
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
  alternative was superseded by the combined-up commission and then by the
  prepare split; use the [operator guide](../../docs/reseed.md#prepare-the-source)
  for the current CLI syntax.

Existing behavior: [CoW creation](../../README.md#copy-on-write-workspaces).
Integration boundary: [architecture](../../docs/architecture.md#module-map).

Operator-facing implementation guidance: [reseed an existing workspace](../../docs/reseed.md)
starts with one-off named use and covers optional configuration, explicit ignored
replacement, full JJ-only target restrictions, and pending-operation recovery.
The model-suffixed proposal above remains the design and experiment evidence,
not a mutable command reference.
