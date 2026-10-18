---
type: ResearchReview
title: "JJ state research: conclusions to carry into the move design"
description: Parent review of the corrected G report, distinguishing verified state mechanics from unselected move/adoption protocols.
resource: /design/sync-scope/jj-state-review0.gpt6a.md
status: reviewed-research-not-implementation
generated: { by: "model:openai/gpt-6-astra", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb262a81effea1HgF0pPS3f1sy"
    title: Checked JJ and upstream JW source, identified material errors in the first G draft, commissioned correction and bounded the conclusions usable for a future move protocol
  - resource: "opencode://ses_eaf5dbdfdffe75JGNrHHbcAtrD"
    title: G's upstream/JJ research and source-based correction pass
  - resource: /design/sync-scope/jj-state0.glm53max.md
    revision: 0faadab2
    title: Corrected report; the predecessor e2418219 is retracted where its mechanics conflict
  - resource: "file:///home/rektide/src/jj-waltz-upstream-state-research/src/workspace.rs"
    revision: cd7bb1eb6451b09c06853b0347f856b4203f7c89
    title: materialize_clone and adopt_source_tree_state, including status validation and restore to the requested base
  - resource: "file:///home/rektide/src/jj-waltz-upstream-state-research/tests/cli.rs"
    revision: cd7bb1eb6451b09c06853b0347f856b4203f7c89
    title: add_with_cow_checks_out_exactly_the_base explicitly supplies edited donor data and expects the base result
  - resource: "file:///home/rektide/a/jj-vcs/jj/cli/src/commands/new.rs"
    revision: 803ef00a0a6ee4ddc275d3138d5f9e7326399906
    title: New commits inherit merged parent trees; empty change does not mean empty tree
  - resource: "file:///home/rektide/a/jj-vcs/jj/lib/src/working_copy.rs"
    revision: 803ef00a0a6ee4ddc275d3138d5f9e7326399906
    title: WorkingCopyFreshness compares operation ancestry and working-copy tree identity
  - resource: "file:///home/rektide/a/jj-vcs/jj/lib/src/local_working_copy.rs"
    revision: 803ef00a0a6ee4ddc275d3138d5f9e7326399906
    title: FileState placeholders, clean-state predicates, content reads and sparse-pattern add/remove diffs
  - resource: "file:///home/rektide/a/jj-vcs/jj/cli/src/commands/sparse/mod.rs"
    revision: 803ef00a0a6ee4ddc275d3138d5f9e7326399906
    title: Sparse update obtains the working-copy mutation lock, applies patterns and finishes state
  - resource: /src/reseed/engine.rs
    title: Current sparse-empty prefill followed by expansion and live tracked-tree verification
---

# Conclusions to carry into the move design

Use [G's corrected report](jj-state0.glm53max.md), commit `0faadab2`, with
the qualifications here. Its first version's empty-tree, mandatory-staleness,
equal-donor/desired-only adoption and rename claims were not usable. Source
review established the important corrections without new parent experiments.

## The useful separation

**Target identity and reusable content/cache state are different things.**
The target retains its own workspace name, repository linkage and checkout/op
binding. A donor `tree_state` can describe disk content copied from that donor,
without the donor tree being the eventual desired revision.

Upstream JW already exploits that distinction during creation: populate from
the donor, adopt its tree state, validate through `status`, then `restore` the
requested base. The upstream test deliberately supplies an edited donor and
expects the base's contents. Different donor and desired trees are therefore
not, by themselves, a reason to reject the idea.

Extending this to existing-workspace reseed is a **new protocol**, not an already
verified feature. It must account for local retained data, snapshot policy,
sparse state, native locks, operation freshness, failure and the source of each
piece of state. Never replace the target's `checkout` with the donor's binding.

## Why this could save work

Today's `jj new <base>` produces an empty **change containing the base tree**.
Sparse-empty patterns suppress materialization; they do not empty the commit.
Expansion runs an empty-to-desired add-diff over newly included paths. Existing
prefills can be retained with placeholder file states; missing paths are written
from the store.

Placeholder zero stats normally fail the next snapshot's clean-state predicate,
so prefilled recorded files are read/hashed during reconciliation. This is not
new-file auto-tracking, and `snapshot.auto-track=none()` does not exempt recorded
paths. Our engine already performs a live verification after expansion.
Validated cache adoption is worth investigating because it can avoid re-reading
known cloned content. Do not claim a measured saving or that copying metadata
alone proves arbitrary disk content correct. Race-mtime rules and the source's
actual state still matter.

## Moves: precise scope before any algorithm

- **Preserve in place** avoids both traversal and moves when the operation can
  exclude the whole region.
- **Stash and restore an existing target-owned root** can be a cheap namespace
  operation when that region must temporarily be vacated. Restoring it intentionally
  keeps the old target data; it cannot also refresh that same region from the donor.
- **Stash the target's own `.jj`** is physically possible, but no JJ command may
  run during that gap. Discovery can bind to an enclosing workspace. Restoring
  the metadata does not declare newly copied disk content to be the desired tree.
- Source/target cache adoption and moving preserved directories are independent
  optimizations; neither implies the other.

For a saved root `R` and desired tracked leaves `D`, the complete path conflict
test is: reject if any `d` equals `R`, is beneath `R`, or is a file/symlink ancestor
of `R`. Apply the equivalent current-tracked test if the region is claimed to
be wholly untracked while absent. A desired tracked file *inside* a saved root
is a conflict before moving, not something to discover only at restoration.
Shared directory ancestors such as `packages/` are not themselves conflicts.
The report's prose about ancestor sets should be read through this exact test.

Restore is not a union. A saved directory cannot simply rename over a file/link
or nonempty directory. Displacing the newly produced entry first is a separate
selected action, with a failure window. Refusal is another valid collision policy.

A future move record must be durable **before** the associated destructive name
transition if recovery depends on it. An in-memory plan is gone after a crash.
A deterministic stash path helps a human find saved data, but is not by itself
proof of ownership or completed phase. This can be a small root-level recovery
record; it need not become a general resumable workflow engine. The report's
manifest/crash-window discussion is a proposal, not an implementation-ready proof.

## Scope of the recommendation

Keep-in-place is the simplest fit to our current root-preserving engine. It does
not prove that every alternative root-swap strategy is universally inferior.
A single bounded staged tree and the cancelled experiment's repeated seven-copy
samples have different costs. Any whole-root swap must also address root inode,
open cwd/handles, native registration/linkage and the selected data-retention
policy; no such alternative is selected here.

Likewise, do not generalize snapshot cost to every JJ configuration: sparse
matchers and filesystem-monitor paths affect observations. This research concerns
our full checkout with filesystem monitoring disabled and the cited JJ versions.

**Next decision:** identify the exact target-owned directories to retain, and
whether the desired operation replaces ordinary contents or needs the root itself
vacant. Then design one state transition around those facts. No production
changes, benchmarks, new fixtures or live moves are authorized by this review.
