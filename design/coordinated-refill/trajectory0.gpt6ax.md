---
type: WorkingNotes
title: Forward trajectory — general brain dump
description: Open forward-looking thoughts on turning the implemented native and transfer pieces into useful everyday reseeding, before refining an executable technical plan.
resource: /design/coordinated-refill/trajectory0.gpt6ax.md
status: exploratory-recommendations
generated: { by: "model:openai/gpt-6-astra#xhigh", at: "2026-10-29" }
sources:
  - resource: "opencode://ses_e7c8f24ebffelGRFC0RXsQvf61"
    title: Human asks for the trajectory ahead, beginning with a general brain dump and then a more precise technical plan; parent integrates the new cross-project assessments without selecting unapproved product defaults
  - resource: /design/coordinated-refill/README.md
    title: Current implementation, review and cross-project assessment map
  - resource: /design/coordinated-refill/reseed-checkin0.gpt6ax.md
    revision: a4df2bf2
    title: New cross-project synthesis of native renewal, shared command inputs, live library costs and unresolved product choices
  - resource: /design/coordinated-refill/reseed-checkin0.gpt61sxh.md
    title: Consumer-side assessment of actual native passes, storage promises and delivery boundary
  - resource: "file:///home/rektide/src/diff-dirs/design/fit/jw-checkin0.gpt61sxh.md"
    title: Library-side assessment of live decision overhead, repeated route checks and opaque source-sharing limits
  - resource: "file:///home/rektide/src/diff-dirs/doc/consumer/jj-waltz.md"
    title: Maintained library-side account of actual adoption and prospective improvements
  - resource: /design/coordinated-refill/review0.gpt6ax.md
    title: Independent native/authority review and questions about the everyday operation
  - resource: /design/coordinated-refill/implementation2.sol61x.md
    title: Delivered conditional routing and remaining whole-command handoff
  - resource: /design/coordinated-refill/corrections0.sol61x.md
    title: Completed correctness corrections, keeping the discussion boundary intact
---

# Forward trajectory — general brain dump

This is the deliberately broad first pass. It is a set of judgments, questions
and possible directions, not a new implementation commission. The next writing
step turns these thoughts into a concrete sequence, dependencies and decisions.

## The thing I want us to finish

I want a command we actually use: point it at a prepared source and some receiving
workspaces, understand what it intends to change, let it do that work, and get a
useful result or ordinary error. Its cost should be explained by the job, rather
than layers of rediscovery, verification, compatibility scaffolding and hidden
duplicate traversal.

The current code gives us a strong foothold: six usable crates; a working native
N1 refill; source-qualified transfer without a discovery prewalk; conditional
directory creation; and two reviewed path fixes. The normal CLI still uses the old
driver. That is enough history for this plan. The next accomplishment should be a
working replacement command, not another capability inventory.

## We should finish a product without freezing every assumption into it

There is a real fork between **renewing tracked sharing** and **making a workspace
current cheaply**. They often lead to the same visible file contents, but not the
same inode/extent state or cost. Our new native route deliberately clears tracked
contents and reflinks every donor tracked leaf. It is a renewal operation even on
a clean repeat.

My current recommendation is to keep the meaning of `reseed` honest: deliver the
renewal operation we have substantially built, while refusing to call it the
optimal everyday refresh merely because it exists. If the operator's actual job
is mainly native advancement or dependency refresh, that can select a different
front of work before rollout. We should not implement an assortment of modes just
to postpone deciding which job matters.

This is still a recommendation, not the human's answer. In particular, we have not
received an answer that mandates full tracked renewal on every routine invocation.

## The next unit is a source and a batch, not one workspace helper

N1 remains important. Reusing configuration, native bindings, membership indexes
and loaded links helps one receiver before source sharing helps several.

For N2, calling `refill_one_from` twice would preserve almost all the duplicated
work. The actual next shape is one command-owned source context, distinct receiver
state, serial native transitions initially, and one transfer cohort. The public
command should use this shape for both N1 and N2.

This is also where we should delete general-context rediscovery. If we already
know the repository, registration and primary root, a later phase should not ask
a general discovery function to find them again. Carry facts for their real
validity interval; read mutable facts at the transitions that change them.

## Preparation needs two clear meanings

Donor preparation is real production work: fetch, native advancement, install and
build. Root-level command resolution is inexpensive selection and route choice.
Those should stay visibly different in the API and command output.

An explicit `reseed --prepare` should consume the result of that one producer
invocation. It should not immediately snapshot, rediscover and re-prove the donor.
An in-memory handoff is sufficient. A persistent donor-generation service, receipts
or a per-file cache are separate possible projects, not prerequisites.

The harder question is actual usage. If several agents issue independent N1
commands, an in-process N2 engine cannot share their source work. First make explicit
batching easy and use it where the workflow already has a batch. Cross-process reuse
would need an explicit generation/lifetime contract: S identifies tracked content,
not every installed ignored byte. A daemon would be premature before that need is
demonstrated.

## We must distinguish three storage promises

Equal contents, independent inodes and shared extents are different properties.
New required-reflink installs give us useful private sharing. An auxiliary leaf
retained because type/size/mtime compare equal has not been recloned or certified
to have an independent inode.

This matters because language such as “private dependencies” can suggest a stronger
postcondition than the incremental implementation establishes. We should test a
stat-equal ordinary copy and a source hardlink, then describe the result accurately.
That experiment need not turn into automatic privacy repair. If repair is wanted,
it is a deliberate effect with real cost, not a reason to falsify comparison facts.

## Ordinary authority is now implementable; it is still a product choice

The implemented source-qualified rule is coherent: a present ignored donor directory
grants authority inside itself. Donor absence inside that directory removes extras;
absence of the entire source root leaves the receiving root alone.

There is a plausible simpler mental model in declared mirrors and explicit preserves.
There is also a plausible broader-update model. Those would change what disappears,
what is copied and why. We should discuss them through actual paths—`.env`,
`node_modules`, a vanished build directory—not through more abstract policy types.

For the next integrated command, retaining source-qualified behavior is the simplest
working assumption. The machinery is now available and it avoids a simultaneous
copying/deletion-default change. That is not a lifelong commitment to ignoredness.

## The library has earned its place, but its live cost has to earn its place too

The reusable effects are valuable: staged replacement, literal links, exact supported
timestamps, required reflinks, conditional routes and normal partial-error handling.
Those should remain shared rather than become another JW copier.

The cross-project reviews identify a separate concern: live coordinated execution
constructs portable decision representations, clones them and rechecks routes while
JW is simply running forward in one process. It may be paying for useful pause
capabilities at a point where this consumer does not use them.

I would first compare useful work against those costs. If they matter, favor a
lighter live view of the same inspector/frontier/kernel, with exact checkpoint
projection at the boundary that needs it. Do not start by deleting pause semantics
or designing an entirely separate execution framework. The existing paired driver
is also a useful N1 control, using the same policy and effects.

Source-only work is another limit: two empty receivers still copy a large missing
subtree separately. A source-shared effect cohort could help, but it is a different
investment from paired-directory comparison. Let real cold/mixed receiver evidence
decide whether it is the next library job. A more independent scheduler is further
downstream still; the current group barrier is understandable and may be sufficient.

## Avoid a new perfection trap

We do not need every promise resolved or every native version certified to produce
a useful first release. We need an explicitly supported operating profile and an
honest successful-result contract. Failures can stop after effects; not every peer
must complete. Full sparse state is not proof that an interrupted adopted workspace
is healthy. The README should say those things directly.

The current JJ-0.45.1 equality check is an interim implementation boundary, not a
proof about every build carrying that version. Decide a usable profile from focused
evidence, without turning a speculative universal version matrix into the project.
Likewise, bookmark coverage and saving old tracked work are separate choices. We
can retain the current gate for the first integration without rebuilding repeated
coverage proofs or calling it an unavoidable property of reseeding.

## What would constitute a useful next result?

One replacement CLI handling ordinary N1, a real mixed N2 batch, explicit donor
preparation, an honest cheap preview, and a failure after visible work. Its report
distinguishes tracked renewal, auxiliary effects and native completion. Its source
and target queries are shared where meaningful. The superseded reseed driver is
removed; creation can keep genuinely shared legacy helpers until later reuse.

Then a small matched comparison tells us where time actually remains. Measure
native work, paired acquisition, routing, policy and effects separately. Keep
fixtures reusable. The old expensive full-copy/hash campaign is a lesson, not a
recipe to repeat. A tiny result establishes mechanics; a bounded representative
observation is needed before calling the command fast for the user's workload.

## Tentative order coming out of this dump

1. Make the first-release outcome explicit, especially full tracked renewal.
2. Extract one command-owned resolution/native-input context from the N1 seam.
3. Form a real receiver cohort and finish the public prepare/preview/reseed path.
4. Establish correct results, understandable failures and attributable costs.
5. Optimize the dominant remaining term: native work, policy, live cursor overhead
   or opaque source sharing. Avoid choosing from architecture preference alone.
6. Reuse the successful machinery for occasional seed and any separately selected
   cheaper refresh operation.

The broad aim is **a small, useful product core with extensible shared mechanisms**.
The narrowing question for the technical plan is: what must change to deliver that
first whole command, and what should remain a conditional next investment?
