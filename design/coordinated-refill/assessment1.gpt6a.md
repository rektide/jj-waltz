---
type: DesignProposal
title: Reseed-first lifecycle redesign with upfront execution choices
description: Replaces compatibility-led migration with a whole-command redesign around frequent reseeding, one upfront execution choice, forward failure and explicit operational hazards.
resource: /design/coordinated-refill/assessment1.gpt6a.md
status: revised-direction-with-proposed-contract-details
generated: { by: "model:openai/gpt-6-astra#high", at: "2026-10-28" }
sources:
  - resource: "opencode://ses_e7c8f24ebffelGRFC0RXsQvf61"
    title: Human redirects lifecycle scope and challenges hidden iteration in preparation; parent records revised direction and develops its technical architecture with explicit native pass accounting
  - resource: /design/coordinated-refill/assessment0.gpt6a.md
    title: Prior compatibility-led plan and verified API findings; lifecycle-preservation and sequencing recommendations superseded here
  - resource: /design/coordinated-refill/current0.gpt6a.md
    title: Native lifecycle census, repeated verification and distinct creation rollback behavior
  - resource: /design/coordinated-refill/library0.gpt6a.md
    title: Verified coordinated API, actual loop, sharing limits and twenty-two passing focused tests
  - resource: /src/lifecycle.rs
    title: Current batch rollback, capability-based checkout choice, metadata/link ordering and switch recording
  - resource: /src/workspace.rs
    title: Current native setup, state adoption, destroy-and-recreate fallback and receiving identity
  - resource: /docs/reseed.md
    title: Delivered behavior and operator assumptions, distinct from the proposed new lifecycle
  - resource: /README.md
    title: Maintained operator entry point receiving the concrete hazards account and redesign pointer
  - resource: /design/sync-scope/cursor-actions0.gpt6a.md
    title: Earlier good-donor and forward-iteration direction
  - resource: "file:///home/rektide/src/diff-dirs/design/native/graph1.gpt61sxh.md"
    title: Concrete tracked-membership consequence of donor cache adoption with retained locals
---

# Reseed-first lifecycle redesign

The subsequent [technical architecture](architecture0.gpt6a.md) makes this direction
concrete and refines its “capture” language: root-level decisions perform no all-file
inventory, while native snapshots and any route-required repository enumeration are
explicitly costed execution. It also specifies streaming authority and the actual
fail-fast cursor loop. This document retains the lifecycle direction and its rationale.

## Direction

**Build the useful new reseed operation end to end.** Its common successful path,
cost and comprehensibility decide the architecture. Seed is occasional reuse of
that machinery, not a prerequisite or the organizing workflow.

The human has clarified the design parameters:

- Reassess the lifecycle promises themselves, rather than preserving every current
  behavior while swapping in a different filesystem loop.
- Unexpected failure can terminate the command. Elaborate rollback, recovery and
  successful-peer continuation are not prerequisites.
- Decide execution strategy up front. Runtime failures should not cause repeated
  capability assessment or opportunistic switching to another implementation.
- Know the hazards and explain them plainly in the README. Do not turn every
  hazard into a machinery-heavy guarantee.

This supersedes the [previous plan](assessment0.gpt6a.md)'s preservation of batch
creation rollback, runtime fallback, mandatory peer completion and its narrow
payload-adapter-first ambition. That document's source findings, API example and
test evidence remain useful. Production behavior has not yet changed.

## Scope: the whole reseed command

The intended pipeline is:

```text
resolve intent and assess the environment once
→ choose a concrete native/transfer route
→ capture source and old target revisions
→ run the receiving native transition and coordinated transfer
→ complete requested links/bookkeeping and verify successful results
```

Design and replace these boundaries together: source preparation handoff, native
input capture, tracked materialization, ordinary-data authority, coordinated
iteration, result reporting and failure semantics. Existing modules and tests are
evidence about useful behavior; their current decomposition is not the target.

The result must be useful for both the common one-target invocation and explicit
multi-target batches. Sharing paired source observations benefits batches. Removing
repeated native work and simplifying the tracked transition also benefits N=1.
The source-derived `1+8N` verification-call structure is a redesign target, not a
baseline contract to preserve for several implementation stages.

## Lifecycle parameters

These are proposed concrete defaults implementing the human's direction. They
should be reviewed as product semantics, not rediscovered from old tests.

| Concern | Proposed new contract | Implementation consequence |
| --- | --- | --- |
| Successful reseed | The selected receiver retains its identity, has the requested JJ tree/parent, and receives the explicitly authorized donor data. | Keep correctness checks that establish this result. |
| Failure | Stop on the first surfaced unexpected error; retain effects already performed. | Ordinary error propagation with workspace/phase/cause; no batch undo or guaranteed peer completion. |
| Batch | A coordinated set of destinations, not a transaction. | Some receivers can be complete, some partly changed and some unstarted when the command fails. |
| Capability choice | Select a concrete execution route before destructive refill. | No destroy-and-recreate retry, speculative execution to choose a route, or mid-command downgrade. |
| CoW | A requested required-reflink route either performs required reflinks or fails. | Ordinary checkout/copy, if offered, is an explicitly selected or upfront-disclosed mode. |
| Source | One good, quiescent donor supplies the run. | Capture useful native facts once; remove repeated whole-donor proofs from target loops. |
| Concurrency | Caller keeps source, receiving workspaces and topology stable. | No new locking protocol or attempt to prove absence of every external writer. |
| History | Snapshot the old tracked work and report its exact revision before refill. | Reassess mandatory named-bookmark coverage separately from saving tracked work; see below. |
| Local finish | Requested links and bookkeeping are normal ordered steps. | A finish failure fails the command; it does not erase the completed workspace. |
| Creation | Native initialization followed by shared transfer/finish mechanisms. | Successful new receivers are not deleted merely because another creation or navigation update failed. |
| Progress | Report actual phases and accepted work, plus a concise failure location. | No transactional effects ledger, elaborate persistent states or universal repair protocol. |

The successful-result contract still matters. Accepting partial failure does not
authorize a successful return with the wrong tracked tree, copied donor `.jj`, or
unrequested deletion. Those are ordinary correctness conditions, not corner-case
recovery promises.

### History is a separate product choice

Recommend retaining the initial native snapshot and reporting old target revisions
and any existing bookmark names. A mandatory covering bookmark is a convenience/
retention policy, not a filesystem prerequisite. Consider removing that gate rather
than automatically carrying it into the new command. This specific CLI/default
change remains a proposal, not an inference that the human has already selected it.

Saved JJ history is not a backup of ignored files, and an unreferenced revision is
not an indefinite retention guarantee. Explain that distinction directly. Do not
invent automatic bookmark creation or another recovery system to replace the gate.

## Upfront assessment, precisely

Use a single command-owned assessment over the actual selected source and targets:

1. Resolve roots, repository/workspace bindings, desired revision, transfer scope
   and explicitly requested preparation. Complete preparation before freezing the
   source used by refill.
2. Classify the supported native profile: real root directories, distinct source
   and receivers, required JJ capabilities, relevant sparse/monitor/colocation
   state, and source/destination filesystem compatibility.
3. Select one native materialization route and one transfer mode for each target.
   If actual reflink capability is uncertain, a small disposable probe per relevant
   source/destination filesystem pairing can resolve that question. A known
   unsupported combination is diagnosed before clearing target contents.
4. Present or retain this small in-memory choice as the execution input. Every
   stage uses it; stages do not renegotiate it.

This is **bounded assessment of the requested operation**, not a recursive proof
of every payload entry or a promise all later writes will succeed. Existing root
and topology knowledge can be used once. Disk exhaustion, permission errors,
unsupported individual entries, late I/O errors and concurrent changes can still
fail during real execution. Then stop with the original cause and phase.

Actual per-entry inspection needed to copy safely and legally is ordinary work.
It is different from repeatedly asking whether to use CoW or another algorithm.
Likewise, rejecting an unusable borrowed JJ cache is not permission to silently
recreate a destination and try the whole operation another way.

### Where fallback belongs

If automatic alternative selection is worth offering at all, it belongs in the
upfront mode decision and must be visible. For the primary reseed use case, the
simple default is required CoW and failure on unsupported setup. Existing explicit
ordinary-checkout modes can remain distinct choices. No new matrix of ongoing
fallback handlers is justified by this redesign.

## Redesign the native transition too

Do not limit the commission to replacing today's auxiliary `Applier` call while
leaving all sparse-clear/prefill/verification orchestration intact by default.
Compare two concrete native compositions using the existing bounded evidence:

- **Source-shaped refill with validated donor-state reuse.** Especially promising
  for normal D=S reseed: renew tracked donor sharing and reuse already-computed
  native tree knowledge. Keep receiving registration/checkout identity. Choose
  this route upfront only for the supported native profile; rejection later is
  an error, not an automatic second execution strategy.
- **Known equal-leaf prefill with native desired-tree materialization.** Already
  proven for the current production semantics, including D≠S. Retain it where it
  is the simpler supported route, while removing duplicated capture/validation.

Select the default from a complete successful-path prototype and its cost. The
library cannot make stat-equal tracked leaves renew sharing automatically, and it
does not own JJ membership. Those facts determine the adapter; they do not require
preserving the current engine's surrounding lifecycle framework.

The concrete source-state hazard is known: importing S's tracked membership can
absorb a target-local path and remove it when restoring D. For D≠S, decide whether
the intended path ownership permits that transition; otherwise refuse that route
upfront or select the other route upfront. Do not make a generic promise to
preserve all arbitrary locals through every native transition. State which local
regions are actually promised and test the relevant overlap. D=S removes this
specific S-only restoration hazard, not all possible path conflicts.

## Coordinated transfer and ordinary failure

Use production `SyncCursor`/`SyncRuntime` with frozen target policies. JW owns
native setup and finish; diff-dirs owns shared inspection, ordered traversal and
production effects. Keep user effects outside the pure visitors.

The fail-fast command should use the explicit group/application loop: surface a
group error, or an `apply` error, and return. The convenience `drive` loop normally
continues healthy lanes, which is useful library behavior but not the required
JW command policy. A returned call has joined its started work; the command can
leave through ordinary error propagation. If a group already observed several
lanes or prior applications wrote data, that work is not undone.

Keep a compact native context: source/desired revisions and shared path indexes,
target identities/old revisions, chosen modes and current phase. On failure,
print target/path where available, phase and underlying cause. Successful peer
outcomes can be shown when already known; finishing all peers is not obligatory.

Shared-source inspection still covers paired directory groups. Opaque source-only
copies remain per destination, and source/target routes still receive library
checks. Reuse these mechanisms as delivered; make additional sharing a response
to measured remaining cost rather than a prerequisite for a useful command.

## Ordinary-content policy is in scope

Write a small path-outcome table before implementation: tracked content, selected
auxiliary content, target-local retained data, native metadata and configured links.
For each, say who may overwrite it and whether donor absence authorizes deletion.
That table should drive both native materialization and the transfer visitor.

The existing selected-root policy is an available starting interpretation, not a
mandate to keep its selector/ignore machinery forever. Broad update or broad mirror
can simplify the operation when their actual desired outcomes are selected. The
present lifecycle clarification alone does not choose between retaining or deleting
target-only ordinary data. Compare that question using `.env`, dependency roots,
build outputs and a vanished donor directory—not an abstract framework or a large
policy language. Record the selected answer in the README before changing defaults.

## Revised implementation sequence

1. **Set the successful-path and lifecycle contract.** Use the parameter table,
   one upfront-choice flow and the concrete path-outcome table. Identify current
   tests that establish valuable results versus tests that encode rollback,
   fallback or other deliberately retired behavior.
2. **Restore the dependency boundary and build one complete reseed prototype.**
   Adopt the current crate layout and choose the native transition from small D=S
   and D≠S examples. Take an existing receiver from initial capture to final native
   result through the real transfer kernel. Delete repeated donor/desired-fact
   work as part of this architecture, not a remote cleanup milestone.
3. **Extend that flow to coordinated targets.** Shared command facts, serial native
   transitions initially, one shared ordinary-data traversal, normal finish. Keep
   receivers fully native-materialized before a potentially long auxiliary phase
   when using the sparse route. Fail fast without unwinding earlier targets.
4. **Demonstrate the daily-use value.** Verify N=1 and a mixed N=2 case, clean-repeat
   payload zero writes, real private reflinks, exact JJ result and retained-region
   behavior. Measure native setup/materialization, transfer and finish separately.
   Compare whole-command cost with the repaired baseline using reused bounded
   fixtures; add a representative workload observation without the old repeated
   full-copy/hash campaign.
5. **Finish a usable replacement.** Land CLI/help/README and supported-profile
   changes with the implementation, retire superseded code and adjust tests to the
   selected new contract. Reuse the simpler engine in occasional creation; creation
   does not block delivery of useful frequent reseeding.

An end-to-end useful reseed is the landing unit. A successful library probe or a
package rename is evidence along the way, not the product milestone.

## Verification proportionate to the contract

Exercise the common successful path, one desired-tree difference, meaningful
namespace collisions, actual CoW and one multi-destination run. Add one known
unsupported upfront-choice case and a deliberate failure after some writes: the
latter must return failure with useful context and leave prior effects, without
switching strategies or deleting successful peers. A failed native finish must
not become a successful command merely because copying finished.

Reuse existing relevant tests and the 22 passing library/consumer checks. Replace
old batch-rollback and automatic-fallback expectations when their behavior is
deliberately removed. Do not preserve them as an accidental scope veto or replace
them with a comprehensive crash/recovery test matrix.

## README obligations and hazards

The README should explain normal operation first, followed by a short usable
account of these actual limits:

- Work against a prepared, quiescent donor and idle receivers. Native snapshots
  and upfront checks do not create an atomic filesystem snapshot.
- Reseed changes tracked contents and explicitly selected ordinary data. Ignored
  files removed by a mirror or explicit clear are not recoverable from JJ history
  merely because tracked work was saved.
- Failure may leave partial files or native state and several partly processed
  targets. Stop and inspect the reported phase; blindly snapshotting an interrupted
  tracked refill can record incomplete contents. Out-of-band advice is guidance,
  not a universal automatic-recovery promise.
- Upfront mode selection cannot guarantee later I/O. Required reflinks can still
  fail; the operation does not silently downgrade or recreate a target.
- Auxiliary equality uses type/size/mtime or literal symlink text. It does not
  detect stat-preserving byte edits or mode-only drift; directory mtimes are not
  subtree proofs.
- Donor dependencies/build outputs reflect the donor's environment. They are not
  a compatibility guarantee for a different requested revision.
- Spell out the supported native profile and intentional exclusions. Some topology
  and concurrent-writer hazards are operating assumptions, not automatically
  prevented conditions.

The root README now links this revised direction and groups the already-delivered
operational hazards. Its current command documentation remains descriptive until
the redesign lands; proposed fail-fast/mode-choice defaults are labeled here.
