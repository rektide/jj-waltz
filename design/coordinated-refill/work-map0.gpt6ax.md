---
type: WorkMap
title: Reseed and diff-dirs — the actionable work map
description: Three focused work lanes separating unfinished product delivery, bounded efficiency work and longer-range architectural options, with native ticket ownership and dependencies.
resource: /design/coordinated-refill/work-map0.gpt6ax.md
status: planned-work
generated: { by: "model:openai/gpt-6-astra#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e7eb31c84ffev4GUeYjdmZO7mY"
    title: Human asks to turn known work and additional wants into usable epics and tickets; parent inventories both trackers and separates ready work from later architectural options
  - resource: /design/coordinated-refill/trajectory1.gpt6ax.md
    title: Existing proposed delivery sequence, shared command inputs and explicit public-contract boundary
  - resource: /design/coordinated-refill/discussion0.gpt6ax.md
    title: Human steering, concrete decision-ownership candidate and separate inherited-policy/comparison requirements
  - resource: /design/coordinated-refill/reseed-checkin0.gpt6ax.md
    title: Architecture assessment and native, live-state and observation cost attribution
  - resource: /design/coordinated-refill/implementation2.sol61x.md
    title: Delivered routing/library adoption checkpoint, not completed command integration
  - resource: "file:///home/rektide/src/diff-dirs/doc/consumer/jj-waltz.md"
    title: Maintained cross-project ownership and library improvement requests
---

# The work map

**We do not need to solve every architectural question to make useful progress.**
There is known integration work, one concrete small library improvement, and a
bounded backend experiment. The larger ideas remain visible without becoming
prerequisites for all three.

## Recommended immediate queue

1. **`jjwaltz-reseed-coordinated-context` — share command inputs.** Extract one
   source/binding/native-input context from the working N1 seam. Reuse it across
   receivers instead of repeating discovery, membership/index construction and
   ignore compilation. Preserve the existing native behavior while doing this.
2. **`diffdir-tree-coordinated-light-state` — share selected decisions.** Remove
   duplicate decision ownership between the current barrier and returned group,
   retaining the same persisted machine and all supported checkpoint boundaries.
3. **`jjwaltz-reseed-dirent` — expose and test optional native-dirent use.** A bounded
   consumer experiment: compare early exclusion and metadata acquisition under
   actual JW policy. No inode-based reflink test and no assumed zero-stat equality.

This is a recommended selection order, not permission to run simultaneous builds.
The public native outcome decision can be discussed alongside these tasks; it does
not block source-context reuse or the two library/observation improvements.

## What is already done

- Domain crate separation in both projects.
- Shared immediate source observation, coordinated paired traversal and independent
  destination memberships/results.
- Owned partial-frontier continuation, exact saved decisions and production opaque
  sync applications, with actual fresh-process verification.
- Callable JW native N1 refill, pure source-qualified ordinary policy, conditional
  routing and the reviewed path corrections.

The delivered library tickets stay closed against that scope. Earlier fact-demand,
mode and cohort **research** tickets also retain their historical closure. New work
must not be confused with redoing those achievements.

## Lane 1 — finish the useful JW operation

New native JW epic: **`jjwaltz-reseed-coordinated`**, under `jjwaltz-reseed`.
The table abbreviates that prefix after its first row.

| Ticket | Concrete result | Depends on |
| --- | --- | --- |
| `jjwaltz-reseed-coordinated-context` | One resolved command/source context; reusable native inputs/indexes; native receiver operation extracted from N1. | Delivered N1 foundation. |
| `…-contract` | A short explicit first-cut outcome table: native storage/result promise, supported profile, preparation/preview and history semantics. | Discussion; does not block context work. |
| `…-cohort` | One real N1/N2 flow and one ordinary runtime; loaded finish and compact partial-failure reports. | Context. Uses the current explicit callable profile as its control. |
| `…-prepare` | Source-only producer result handed directly into refill, rather than immediately rediscovered/reproved. | Context. Preserve existing producer semantics unless deliberately changed. |
| `…-preview` | Explicit nonmutating route/intent preview, with uninspected contents labelled honestly. | Context. Public semantics settle in the contract task. |
| `…-cli` | Public dispatch/rendering/help uses the integrated flow; superseded reseed code retires when no longer used. | Cohort, prepare, preview, contract. |
| `…-evidence` | Bounded native CLI evidence for N1/N2, D=S/D≠S, clean/sparse/missing payloads and a returned failure. | Integrated CLI. |

```text
context ──→ cohort ────────────────┐
        ├─→ prepare ──────────────┤
        └─→ preview ──────────────┼─→ CLI integration ─→ bounded evidence
contract discussion ─────────────┘
```

The outcome task does not require designing every future mode. The current strict
native route is the control; changed public promises must be explicit. The existing
`jjwaltz-reseed-matrix` owns broader version coverage. A focused supported-profile
check for this route is not a demand to finish that entire matrix first.

## Lane 2 — make the same diff-dirs engine lighter

Existing epic: **`diffdir-tree-coordinated`**. Keep its IDs and history; no duplicate
efficiency epic or replacement state machine is needed.

| Ticket | Standing / next result |
| --- | --- |
| `diffdir-tree-coordinated-light-state` | **Next bounded change:** share one immutable selected decision between barrier and group views. Trace API/ownership changes, preserve checkpoint equivalence and compare allocation/copy costs. The later selected-fields/facts split is a review point, not bundled automatically. |
| `diffdir-tree-coordinated-measure` | Existing direct-library measurement task: coordinated versus independent comparisons, with actual route, policy, decision and effect costs separated. It is distinct from whole-JW CLI evidence. |
| `diffdir-tree-coordinated-region-context` | **Later design:** inherit an admitted region's judgment and narrow residual exceptions; stop asking already-settled questions. |
| `diffdir-tree-coordinated-comparison-policy` | **Later design:** separate extensible equality, work authority, storage requirements and explicit observation demand. Keep the cheap default. |
| `diffdir-tree-coordinated-opaque-sharing` | **Conditional prospect:** share bounded source batches during opaque source-only/replacement deliveries, if cold/mixed-target evidence justifies the added execution machinery. |

The latter three are not gates on the first decision-sharing improvement or on JW
cohort integration. Persistence remains a requirement throughout. Native protocol
alternatives remain an explicit discussion; they are not hidden inside this library
optimization epic.

## Lane 3 — retain and exploit cheap observations

Existing provider epic: **`diffdir-tree-observe`**; receiving changes belong in JW.

| Ticket | Scope |
| --- | --- |
| `diffdir-tree-observe-fact-reuse` | Protect existing cached facts/handle/metadata handoff; identify missing cross-stage acquisition evidence. Distinguish entry acquisition, route checks, explicit revalidation and restore refresh. |
| `jjwaltz-reseed-dirent` | Add deliberate optional feature forwarding and test it under JW's actual selector. Default behavior and the Linux/nightly requirement remain explicit. |
| `diffdir-tree-observe-dirent-tuning` | Existing reader/buffer/path tuning and bounded measurements; backend activation is not the same work as buffer sizing. |
| `diffdir-tree-observe-dirent-pool` | Existing shared resource-lifetime/configuration design; not a prerequisite for trying native-dirent. |
| `diffdir-tree-observe-btrfs-csums` | Longer-range filesystem comparison/identity inquiry. Inodes alone do not establish reflink sharing. |

For the dirent experiment, count known-kind early exclusions separately from
admitted mirrored files that still require full metadata. A useful result can be
“helps routing but not the payload interior”; it need not justify a default switch.

## What to select before advancing, and what can wait

**Before public behavior changes:** finish the small contract task, establish the
chosen native profile and ensure preview/reporting describe the real operation.

**Before calling the path faster:** collect like-for-like, attributable evidence.
Do not repeat the old expensive multi-copy/full-hash campaign to obtain it.

**Not prerequisites:** generalized inherited modes, arbitrary stronger equality,
shared opaque recursion, opportunistic scheduling, a new native JJ API, a donor
service or automatic reflink/privacy repair. These remain possible future directions,
selected from actual pressure rather than accumulated into the next release.

## Using the map

JW tickets are queried from the JJ-waltz checkout; library tickets from diff-dirs.
Their trackers retain native ownership. Cross-project relationships use the linked
documents and ticket notes rather than fabricated cross-database dependency edges.
An open/ready ticket means its prerequisites are available, not that every open
idea has been selected for immediate implementation.

This map is the short operational entrance. The [discussion](discussion0.gpt6ax.md)
and [technical trajectory](trajectory1.gpt6ax.md) retain the arguments and questions.
