---
type: WorkingAssessment
title: Reconciliation check-in — staged architectural discussion
description: Human steering, concrete native-transition analysis, verified observation reuse and separately scoped follow-up discussions on persistent execution, inherited decisions and comparison policy.
resource: /design/coordinated-refill/discussion0.gpt6ax.md
status: discussion-in-progress
generated: { by: "model:openai/gpt-6-astra#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e7eb31c84ffev4GUeYjdmZO7mY"
    title: Human asks for a concrete architectural discussion in separate turns, retaining advanced persistence, inherited judgments and modular comparison; parent verifies observation reuse and opens the native-transition thread
  - resource: /design/coordinated-refill/reseed-checkin0.gpt6ax.md
    title: Completed cross-project assessment retained as the baseline rather than replaced by another broad review
  - resource: /design/coordinated-refill/implementation1.sol61x.md
    title: Actual native command sequence, snapshot boundaries and strict full-S installation
  - resource: /refill/engine/src/engine/replacement.rs
    revision: 76fb0656
    title: Native clear, child creation, full-S installation, tree-state adoption, D restore and final conformance
  - resource: "file:///home/rektide/src/diff-dirs/tree/observe/src/listing.rs"
    revision: 10361cf0
    title: Retained backend handles, Linux dirent hints, Windows cached metadata and deferred no-follow acquisition
  - resource: "file:///home/rektide/src/diff-dirs/tree/observe/src/facts.rs"
    revision: 10361cf0
    title: Explicit listing-fact provenance, Unknown and zero-I/O accessors
  - resource: "file:///home/rektide/src/diff-dirs/tree/inspect/src/shared.rs"
    revision: 10361cf0
    title: One admitted source acquisition per shared inspection epoch
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/apply.rs"
    revision: 10361cf0
    title: Actual metadata handoff into leaf installation and prepare_file
  - resource: /transfer/workspace/Cargo.toml
    revision: 76fb0656
    title: Current receiving features do not enable native-dirent
  - resource: "file:///home/rektide/src/diff-dirs/design/transfer/modes0.gpt6am.md"
    title: Prior inherited-region and residual-obligation research, not delivered general mode selection
---

# Staged architectural discussion

The [assessment](reseed-checkin0.gpt6ax.md) remains the baseline. This document
captures subsequent steering and develops one problem at a time. CLI adoption
standing is recorded there; it is not the architectural argument for this discussion.

## Human steering now in force

- Explain concrete native behavior and useful alternative compositions, rather than
  reducing the review to whether the replacement has reached the public command.
- **Retain advanced persistence and one coherent execution model.** Adding lightness
  means right-sizing representation, observation and execution, not making a separate
  less-capable fast engine the default answer.
- At an admitted region, determine what no longer needs to be asked. Carry a selected
  judgment and the remaining exceptions into children instead of restarting general
  policy from the root at every name.
- Preserve cheap fact reuse and make comparison/policy extensible. Full strict
  equality is not a new default requirement; the architecture should be able to grow.
- Separate concrete requirements into tickets. Historical research completion must
  not be mistaken for implemented optimization.
- Develop recommendations needed before advancing, plus a broader prospectus,
  through several focused discussions rather than settle everything in this pass.

<a id="native-transition"></a>

## Thread A — what the native reseed sequence buys and costs

### The current protocol establishes a known state

Let O be the receiver's saved old revision, S the donor's tracked revision, and D
the requested tracked result. The callable implementation performs:

```text
snapshot receiver O
→ sparse-clear its tracked contents
→ create the receiving child of D while sparse-empty
→ strictly install all S-tracked leaves from the donor
→ adopt the donor's tree_state, retaining receiving checkout identity
→ if S differs from D, let JJ restore D
→ establish native conformance with a live diff
→ reconcile selected ordinary contents and finish links
```

This is a coherent way to make the donor's native content knowledge applicable.
We installed the tracked S files ourselves, preserved the metadata used by the
cache, and retained the receiver's own checkout/registration identity. The native
tree state can therefore describe the actual installed S tree under the prepared,
stable-source premise. It is not merely a table of harmless stat hints: it carries
tracked membership too.

The implementation's strict create-only installation also gives an intelligible
namespace rule: an unrelated occupied S path is an obstruction, not permission to
adopt and later delete that local object. The final native diff remains necessary;
full sparsity or a successful restore exit alone does not establish the desired tree.

### Clean reset: cheap bytes still mean real per-file work

Suppose O, S and D already have the same tracked contents. This route still clears
and reinstalls the S leaves. A regular-file reflink avoids copying its data blocks,
but still needs source/output opens, clone and metadata operations, namespace
publication and eventual cleanup of replaced objects. Ordinary selected payload
can simultaneously be a genuine no-write repeat.

That is not accidental failed change detection. It obtains renewed donor sharing
and a source-shaped state suitable for cache adoption. The question is whether
every invocation needs that particular postcondition and whether clearing first
is the cheapest faithful way to obtain it. **Requiring renewal does not by itself
prove that complete clearing is the optimal implementation.**

### S-to-D example

```text
S: same.txt, edit.txt@S, old.txt
D: same.txt, edit.txt@D, new.txt
```

The current route installs all three S files, adopts S state, then asks JJ to
remove `old.txt`, replace `edit.txt`, and materialize `new.txt`. Only `same.txt`
necessarily retains the installed donor sharing. The intermediate installs are
useful to the chosen native protocol; their count is not a final-D sharing count.

An equal-leaf/native route could prefill only `same.txt` and let JJ materialize
the other desired leaves. But it cannot simply skip two installations and still
import an unmodified full-S cache as if all S leaves had been installed. It must
establish truthful native state by another supported procedure. The previous
equal-leaf/sparse route is useful comparison material, including its placeholder
and native validation costs; it is not automatically cheaper.

### Alternatives to discuss, without selecting a mode matrix

| Composition | Potential saving | Proof or cost still owed |
| --- | --- | --- |
| Current clear/install-S/adopt/restore-D | Simple known S state and explicit new donor reflinks | Per-leaf reinstall, native snapshots, and S-only/changed intermediates when D differs. |
| Native update to D, retain satisfactory tracked leaves | Avoid deleting/reinstalling already-correct files | Does not automatically renew donor sharing; still establish desired native state and selected storage guarantees. |
| Native D transition plus authorized reflink replacement of chosen tracked leaves | Explore renewal without clearing every tracked pathname first | Today's listed installer is create-only; replacement authority, cache validity, conflicts and possible duplicate native writes need a real protocol. |
| A native-supported materialization/adoption operation | Potentially combine passes and keep native cache ownership explicit | Stronger JJ integration/version commitment; only justified by demonstrated remaining cost. |

The architectural choice is **postcondition → truthful native protocol → necessary
work**, not necessarily different command names. The earlier assessment's separate
refresh operation is one possible product shape, not a decision imposed here.

### Recommendations we can already retain

Keep receiving identity separate from donor content knowledge. Keep tracked/native
authority out of the generic filesystem comparator. Count actual snapshot-triggering
commands and installs, not just named phases. Compare D=S and differing D separately.
Do not claim every snapshot hashes every byte: native stat-cache reuse affects cost.
Evaluate a less destructive renewal composition before equating the strong storage
promise with the current clear-first algorithm.

Still open: which storage/result guarantees should routine reseeding establish;
which S/D shapes dominate real use; and which native protocol achieves those
guarantees with the least duplicated work. No new behavior is selected by this turn.

<a id="fact-reuse"></a>

## Direct fact check — enumeration and stat reuse

The optimization was retained, with an important platform distinction:

| Backend | Facts already available from listing | Additional work for size/mtime equality |
| --- | --- | --- |
| Default std Unix/Linux | Native name and cached inode; early kind deliberately Unknown | One explicit no-follow metadata acquisition per admitted present entry. Linux `getdents` does not provide full size/mtime stat data. |
| Opt-in Linux `native-dirent` | Native name, cached `d_type` and inode; genuine `DT_UNKNOWN` remains Unknown | Full metadata is still needed for file size/mtime, device/modes or other requested attributes. Kind-only/name-based exclusion can avoid it. |
| Windows std backend | Cached enumeration metadata, including early kind | The cached `Metadata` is reused for the late phase, rather than fetched again solely for comparison. |

The std backend retains `DirEntry` until explicit acquisition; the native backend
retains the shared directory handle and performs metadata acquisition relative to
it. Neither discards the listing handle and blindly reconstructs each lookup from
the full path. Hint accessors do no hidden I/O. Shared inspection acquires each
required source entry once for the current group, not once per lane.

The internal sync path hands already-observed source `Metadata` to leaf installation;
`prepare_file` uses it for modes/timestamps. That is distinct from public-plan
revalidation, restored unstarted-batch refresh, route checks, and operations a copy
backend may itself need. Those are separately chargeable observations, not evidence
that the listing cache disappeared.

**JW currently does not enable `native-dirent`.** It forwards its parallel feature,
but not that optional Linux/nightly backend. Thus its default early Unix view does
not receive cached `d_type`. Enabling the native feature would help some early
classification/exclusion; it would not turn Linux size/mtime equality into a
zero-stat operation. We should track activation and demand shaping separately.

## Queued threads — seeds, not designs selected today

<a id="lightness"></a>

### B. Lightness inside the persisted design

One semantic machine remains the requirement. Distinguish authoritative decision
state from duplicate views and wire projection. A supported yield must still be
checkpointable with its partial parents, lane memberships and unresolved choices.
Lazy projection, immutable sharing and cheaper live views are candidates only if
they retain exactly that behavior and do not rerun policy to reconstruct a lost
decision. Next discussion: trace one wide group through live execution, checkpoint
and restore, and identify which copies are actually necessary.

<a id="inherited-modes"></a>

### C. Inherited judgments and residual work

The user's intuition directly matches the existing
[inherited-mode study](file:///home/rektide/src/diff-dirs/design/transfer/modes0.gpt6am.md).
For a known empty target region, comparison and target discovery may disappear
while source discovery, copy attributes and actual installations remain. A region
mode can carry a narrowed exception set; only an exceptional child needs broader
assessment again. The old study is research, not proof that generic inherited
mode specialization is delivered. Next discussion: a selected subtree with one
preserved exception, then a mixed paired/missing destination.

<a id="comparison-policy"></a>

### D. Modular comparison, policy and fact demand

Separate the equality predicate, overwrite/removal authority, storage requirements
and observation demands. Retain cheap metadata equality as the default while making
stronger predicates an explicit later capability. Today's visitor steers work over
an immutable built-in comparison; it is not yet a pluggable comparison predicate.
Next discussion: the minimum seam that admits stronger comparison without fetching
every possible fact or letting pure policy perform hidden I/O.

## Ticket scope checkpoint

`diffdir-tree-coordinated` records delivered core/continuation work; its remaining
measurement task does not cover every newly requested optimization.
`diffdir-tree-observe-facts` and `diffdir-tree-transfer-modes` are closed **research**
tasks, with their historical completion retained. `diffdir-tree-observe-dirent-tuning`
and `-dirent-pool` remain open; tuning buffers is distinct from retaining facts and
from selecting the backend in JW.

New requirement/design tickets now retain the separate threads:

| Requirement | Ticket |
| --- | --- |
| One persisted model with lighter live execution | `diffdir-tree-coordinated-light-state` |
| Inherited judgment with narrowed residual work | `diffdir-tree-coordinated-region-context` |
| Modular comparison, authority and fact demands | `diffdir-tree-coordinated-comparison-policy` |
| Retained cached facts, acquisition census and explicit backend activation | `diffdir-tree-observe-fact-reuse` |

Their creation records requirements and questions, not permission to implement all
of them in parallel. Detailed pre-advance recommendations and the broader
prospectus will be refined after the corresponding focused discussions.
