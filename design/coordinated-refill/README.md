---
type: Index
title: Coordinated seed and reseed
description: Assessment and evidence for adopting diff-dirs source-to-many synchronization in jj-waltz.
resource: /design/coordinated-refill/README.md
status: proposed
generated: { by: "model:openai/gpt-6-astra#high", at: "2026-10-28" }
sources:
  - resource: "opencode://ses_e7c8f24ebffelGRFC0RXsQvf61"
    title: Integrated independent evidence and lifecycle corrections, then authored the technical architecture separating root decisions, native pass costs and streaming transfer
  - resource: /design/coordinated-refill/architecture0.gpt6a.md
    title: Current technical architecture, actual cursor loop, streaming authority and explicit traversal accounting
  - resource: /design/coordinated-refill/assessment1.gpt6a.md
    title: Current lifecycle redesign direction and useful end-to-end implementation sequence
  - resource: /design/coordinated-refill/assessment0.gpt6a.md
    title: Parent ownership assessment, concrete command structure and staged implementation plan
---

# Coordinated seed and reseed

## Start here

- [Technical architecture](architecture0.gpt6a.md): root-level route selection,
  explicitly costed native snapshots and repository enumeration, streaming path
  authority, the actual coordinated loop, and forward failure. Defines what each
  phase does and where remaining all-file work really occurs.
- [Revised scope and plan](assessment1.gpt6a.md): redesign the whole frequent reseed
  operation; choose execution routes up front, fail normally, remove inherited
  lifecycle guarantees and document hazards. The end-to-end working command is the
  milestone; occasional seed reuses its machinery.
- [Initial assessment](assessment0.gpt6a.md): retained source/API findings and
  caller-loop example. Its compatibility-led lifecycle and sequencing plan is
  superseded by the revised direction.

## Independent evidence

- [Current JJ-waltz census](current0.gpt6a.md): native creation/reseed phases,
  different failure contracts, shared-fact opportunities and `1+8N` verification
  calls; the dependency-layout failure was reproduced.
- [Library fit and bounded tests](library0.gpt6a.md): actual production APIs,
  sharing and application boundaries, six consumer tests and sixteen upstream
  shared-inspection tests passing, plus explicit stable compilation.

Both jobs were dispatched through G but ran as Astra according to their native
session records; filenames and provenance reflect the actual model. These are
independent assessments, followed by the parent's integrated recommendation.

## Existing contracts and prior inquiry

- [Reseed operator guide](/docs/reseed.md): delivered copying authority, native
  transitions, preparation and failure behavior.
- [Architecture](/docs/architecture.md): current module and lifecycle ownership.
- [Visitor integration checkpoint](../sync-scope/visitor0.sol61xh.md): existing
  generic transfer kernels and bounded native cache experiment.
- [Policy-selection correction](../sync-scope/implementation0.gpt6a.md): broad
  transfer defaults remain open; library capabilities do not select them.
- [Current diff-dirs architecture](file:///home/rektide/src/diff-dirs/doc/architecture/README.md):
  delivered coordinated traversal, effect ownership and continuation.
- [Historical actual-workload evidence](../up-performance/primary0.sol61m.md):
  partial results and the cancelled expensive-fixture campaign informing the new
  bounded verification approach.
