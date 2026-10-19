---
type: ImplementationCheckpoint
title: Root-bound visitors and the existing-workspace JJ transition
description: Bounded compatibility integration, native donor-state convergence evidence, and the transfer authority still requiring selection.
resource: /design/sync-scope/visitor0.sol61xh.md
status: implemented-compatibility-slice-pending-policy-selection
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-19 }
sources:
  - resource: "opencode://ses_ead124fcaffedy7mtUij1h6iei"
    title: Implemented root-bound early/late visitors and shared kernel integration; tested donor-S cache adoption and native convergence to different desired D without copying target identity
  - resource: "opencode://ses_eb262a81effea1HgF0pPS3f1sy"
    title: Parent commission preserving human authority over transfer defaults and requiring a concrete JJ/filesystem protocol
  - resource: /design/sync-scope/jj-state-review0.gpt6a.md
    title: Corrected identity/cache separation and exact path-overlap gate
  - resource: /design/sync-scope/jj-state0.glm53max.md
    revision: 0faadab2
    title: Corrected research; not the retracted first account
  - resource: "file:///home/rektide/src/jj-waltz-upstream-state-research/src/workspace.rs"
    revision: cd7bb1eb6451b09c06853b0347f856b4203f7c89
    title: materialize_clone and adopt_source_tree_state (lines 690–731)
  - resource: "file:///home/rektide/src/jj-waltz-upstream-state-research/tests/cli.rs"
    revision: cd7bb1eb6451b09c06853b0347f856b4203f7c89
    title: add_with_cow_checks_out_exactly_the_base (lines 1064–1104), explicitly donor S different from requested base D
  - resource: "file:///home/rektide/a/jj-vcs/jj/cli/src/commands/new.rs"
    revision: 803ef00a0a6ee4ddc275d3138d5f9e7326399906
    title: Parent file-tree inheritance (208–213); this source is newer than the tested installed binary
  - resource: "file:///home/rektide/a/jj-vcs/jj/lib/src/working_copy.rs"
    revision: 803ef00a0a6ee4ddc275d3138d5f9e7326399906
    title: Freshness depends on operation ancestry and tree identity (361–399), not operation age alone
  - resource: "file:///home/rektide/a/jj-vcs/jj/lib/src/local_working_copy.rs"
    revision: 803ef00a0a6ee4ddc275d3138d5f9e7326399906
    title: File-state clean predicate and zero-stat placeholders (305–324)
---

# Concrete protocol and scope checkpoint

## Shipped by this slice

**Before:** retain the existing repository/identity/topology, source/desired-tree,
bookmark, obstruction and link proofs. Snapshot target work with normal JJ policy.
Freeze native source/desired tracked leaves and the existing selected auxiliary
boundaries. No full-tree operation map or per-entry native process queries.

**During:** retain the root and `.jj` in place; sparse-empty; remove recorded
residual leaves; `jj new D`; optional explicit generic clear except `.jj`;
create-only generic tracked prefills; sparse reset and native verification;
one root-bound auxiliary visitor; links; final native tree/identity/parent proofs.

**After:** a fresh empty change containing D's file tree, target identity intact,
selected auxiliary contents reconciled, unrelated local contents retained unless
`--replace-content` explicitly authorized their deletion. Failures can be partial;
there is no journal, metadata rollback claim or automatic repair.

The early context preserves names/subtrees outside the compatibility boundary,
including native metadata, before child stat/readlink. Late uses immutable facts
to keep routing containers while admitting selected descendants, refusing local
routing leaf collisions. The library's finalized action set drives **both** work
and descent through matched, one-sided and staged-replacement paths.

`copy_ignored` is presently an explicitly bounded compatibility adapter. Missing,
empty and disabled selections retain their old precedence and zero-copy meanings.
No preserve/mirror lists or broad donor-copy default are invented. Creation uses
the same visitor in create-only mode; existing switches do not refill. Prepare
remains source-only and keeps its existing validation/selection contract.

The command pool remains shared: experimental P≤4, J=P, K=256; default execution
is serial. Linux topology is shared with creation; other devices are checked from
supplied metadata, not fresh per-entry mount calls. The generic listed-leaf
installer replaces JW's installer; the obsolete recursive copier is removed.
Cleanup's validate-before-removal contract remains distinct from streaming clear.

## Native experiment: truthful S cache, then converge to D

`tests/reseed.rs::existing_workspace_can_validate_donor_tree_state_then_restore_a_different_desired_tree`
passes on installed JJ `0.45.1-33d3a595` with required real reflinks. This source
checkout's `803ef00a` is 80 commits newer; the runtime evidence is labeled to the
installed binary, not a claim of cross-version validation.

The tiny fixture exercises an **existing**, previously populated target:

1. Snapshot S and old target T; name T with a bookmark. Disable new-file automatic
   tracking before creating ignored and nonignored local names. Prove disjointness
   by construction in this fixture, not a production adoption preflight.
2. Sparse-empty the target; `jj new D` under empty patterns; clone **all S-tracked
   leaves**, not merely S∩D. No source or target native metadata is transferred.
3. Copy only S's `tree_state`; assert target `checkout` bytes unchanged. A checked
   native `status` validates/snapshots S into the target's new change. This state
   truthfully describes cloned S even when S≠D.
4. Checked `restore --from D` converges S→D. Verify exact parent D and empty tree
   diff; tracked deletion, new desired path, directory→file, executable mode;
   native root/`.jj` inode, repo pointer, marker and workspace-name binding;
   ignored/unignored locals and the unchanged cloned file's inode.

This maps upstream creation's validated adoption/convergence onto a tiny existing
workspace. It is **not production dispatch**, a hash-free measurement, a general
concurrency proof, or a proved failure/rollback protocol. Real production adoption
would need the actual S/D/local overlap gate, native mutation-lock treatment,
quiescence/sparse policy and a concrete post-copy failure boundary. No whole `.jj`
stash, root swap, move record or blind metadata transplant was added.

## Remaining selection

The human still owns which ordinary source-present and target-only data the new
operation should refresh, retain or remove. Neither broad update nor whole-root
mirror follows from library capabilities or the successful S→D experiment. The
compatibility slice is useful infrastructure, **not completion of that product
choice**. Production cache adoption and eliminating the sparse clear/recopy/hash
cycle require a reviewed next protocol, not another default guessed by an agent.

## Connections

- [Maintained operator contract](/docs/reseed.md#ignored-data): current effective
  scope and comparison limitations; [architecture](/docs/architecture.md#in-place-reseeding)
  owns module integration and target identity.
- [Parent correction](/design/sync-scope/implementation0.gpt6a.md#correction-the-jw-policy-and-state-transition-are-not-settled)
  explains why prior broad-update/mirror proposals are not authorization.
- [JJ review](/design/sync-scope/jj-state-review0.gpt6a.md#why-this-could-save-work)
  supplies the motivation for testing S-cache convergence rather than dismissing
  adoption because S differs from D.
