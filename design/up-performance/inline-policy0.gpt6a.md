---
type: DesignProposal
title: "Reseed with frozen selection facts and inline payload guards"
description: jj-waltz integration map for the shared diff-dirs inspection-gate proposal, including scope partitioning, rule reuse, failure timing, and alternatives.
resource: /design/up-performance/inline-policy0.gpt6a.md
status: proposed
generated: { by: "model:openai/gpt-6-astra", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb262a81effea1HgF0pPS3f1sy"
    title: Designed frozen facts and inline guards, then moved mount checks to batch topology preflight and reflink feasibility to actual application following human direction
  - resource: "https://man7.org/linux/man-pages/man5/proc_pid_mountinfo.5.html"
    title: Verified namespace-local mount topology and bind/stacked mount semantics for the revised front guard
  - resource: "file:///home/rektide/src/diff-dirs/design/fit/policy0.gpt6a.md"
    title: Main design, API semantics, recursive coverage, concurrency, and alternatives
  - resource: "file:///home/rektide/src/diff-dirs/design/fit/policy0.glm53m.md"
    title: Starting inquiry into merging preflight with diff execution
  - resource: /src/reseed/engine.rs
    title: Inspected current selection, validation, sparse refill, and payload replacement call sites
  - resource: /src/cow.rs
    title: Recursive validation, mount-ID implementation, inline cloning checks, and removal preflight
  - resource: /src/cow/payload.rs
    title: Frozen ignore policy with repeated construction and child-process queries
  - resource: /src/cow/ignored.rs
    title: Adaptive outermost-root selection and tracked-path disjointness
  - resource: /docs/reseed.md#safety-and-recovery
    title: Active preflight, quiescence, and out-of-band recovery contracts
  - resource: /design/up-performance/measurement1.gpt6a.md
    title: Prior measured reduction of repeated coordination work; synthetic fixture limitations
---

# Reseed with frozen selection facts and inline payload guards

**Recommendation:** freeze ignore/selection facts and assess mount topology once
per command; preserve selected payload through tracked refill; let comparison and
recursive application perform cheap name/type guards using observations they
already need. Actual reflink failures surface at application, without mandatory
synthetic probes or per-entry mount-ID checks.

The detailed design and alternatives live in
[diff-dirs: inline inspection policy](file:///home/rektide/src/diff-dirs/design/fit/policy0.gpt6a.md).
This companion identifies the jw-specific changes. Both documents are proposals.

## The split

### Keep before target mutation

- Workspace/repository ownership and root identities; target/source separation.
- Full sparse state, conflicts, stale-state checks, paused-writer contract.
- Snapshotted tracked work and whole-batch bookmark coverage.
- Selected-root path confinement and a batch mount-topology front guard covering
  source traversal, tracked refill, and payload copy/removal footprints.
- Selected roots ignored under both frozen source and desired rules; no tracked
  overlap under the trees whose paths will be removed or materialized.
- Tracked-path obstructions and configured-link overlap/future existence.

### Move to the payload's actual traversal

- Nested administrative names on either side, including identical clean entries.
- Unsupported special entries, using the file type already inspected.
- Absolute-source-link diagnostics using the literal link already inspected.
- Actual reflink success/failure during staged copying; an unchanged payload
  requires no synthetic clone probe.

The diff walker handles paired directories. The sync copier/remover handles
one-sided descendants, absent-target initialization, and type transitions.
A walker-only hook cannot replace all current guards.

### Front-load topology, keep it out of the loop

Read `/proc/self/mountinfo` once after binding workspace roots and before recursive
selection/inspection. Reject nested mount points, including same-device bind
mounts and mounted regular files, in every operated scope. Allow a mount at the
workspace root itself; a mount at a selected payload root is still a nested
boundary relative to that workspace. Include broader deletion scope for
`--replace-content`. Keep source/target identity and overlap checks; different
mount IDs do not prove separate storage.

The main design specifies parsing, stacked mounts, bind aliases, platform limits,
and unavailable-procfs behavior. This is a process-namespace topology assessment
under stable mounts/path bindings, not a per-entry check or a guarantee against
concurrent mount changes. Selection and payload walks then do no mount syscalls
or per-entry mount-table lookups. Preparation that can change mount topology
requires a fresh assessment after its external-command boundary.

Drop mandatory `cow::probe_between` from the proposed fast path. Keep an optional
small early-diagnostic probe if useful, but let real staged reflink operations
report unsupported clones. A clone failure preserves the old destination leaf;
earlier completed work remains partial progress. `ReflinkRequired` still means
no silent copy fallback. Clone failures cannot protect recursive deletion, which
is why the topology front guard remains important.

Place an `INSERT MORE CHECKS HERE` comment at the front-guard seam, directing
future stronger checks there rather than into the inner loop. The current code
comment is a placement marker; it does not itself change existing validation.

## Scope partition is the prerequisite

Build a prefix index of selected outer roots and old/desired tracked paths.
The tracked-phase validator can stop at a payload boundary only after proving:

1. that exact boundary and all existing ancestors are real, authorized directories
   on the permitted mount, or the target root's absent/leaf replacement case has
   been explicitly classified;
2. no old tracked leaf to clear and no desired tracked leaf to materialize overlaps
   the selected root in either direction;
3. no configured link action or other cleanup action overlaps it;
4. the normal tracked phase will preserve that subtree.

In particular, desired-tree disjointness alone is insufficient: today's
`old_paths` removal must not touch the supposedly deferred region. Initially
reject an old-tracked overlap; a deliberate migration from tracked to payload
ownership can be designed separately.

Validate the remaining non-payload tree under existing rules, including unrelated
untracked content where today's broad rejection contract requires it. This is
not simply “scan the ~9k tracked files.” Its size depends on retained unselected
content. The savings come from cutting out the large selected payload interiors.

`--replace-content` has a different footprint: it deletes other content as well.
Retain its broad validation initially. Do not use the incremental-mode scope cut
for a wholesale sweep unless that sweep also moves to guarded recursive removal
with an explicitly selected refusal-timing contract.

## Concrete call-site map

Based on `324606bed3b562d866d57b0adc449f90c355c37c`:

| Site | Proposed treatment |
| --- | --- |
| `run_inner`: early `validate_contents(source/target)` | Split immediate root/state checks from recursive validation; perform scope-aware validation once frozen selection is available. Leaving these full walks intact defeats fusion. |
| `validate_same_mount` in selection/validation/copy/removal | Replace repeated per-entry calls with the batch topology assessment on the Linux fast path. Keep checks until that front guard is implemented. |
| planning `cow::probe_between` | Remove the mandatory probe; actual staged clones report capability failures. An optional early-diagnostic mode may probe once per relevant filesystem pair. |
| `IgnoreRules::new` during source/target planning | Build a command-owned context per distinct frozen tree + effective excludes; share target contexts where equal. |
| `Patterns::select` | Keep adaptive outermost-root semantics; use a persistent native Git oracle, with memoized answers, rather than one process per candidate. |
| initial `validate_payload_root` loop | Move interior guards and link warnings to guarded execution; use the same inspection kernel in read-only audit for dry-run. |
| `validate_source` | Retain identity, config, commit and tracked-tree checks; consume frozen root eligibility rather than rebuilding rules and re-querying each root. |
| `execute`: pre-sparse `validate_contents` | Validate outside deferred payload interiors and validate their boundaries. Recheck cheap target state at the existing sparse-mutation barrier. |
| `execute`: early non-replace removal of `plan.ignored` | Remove this deletion so the previous payload generation survives. |
| post-materialization `IgnoreRules::new` | Reuse desired policy after proving the materialized tree equals its pinned coordinate. An empty child at that tree does not require new rule extraction. |
| final remove + `clone_tree` loop | Use guarded incremental sync; explicitly handle absent and non-directory selected targets. |

An absent target root is initialized. An existing target file/link must be
classified before use: the current sync root constructor rejects it, so an
authorized whole-root transition needs an adapter or a new guarded root operation.
Do not silently follow a target symlink. Staging the new directory before removing
the old leaf preserves useful existing transition behavior.

Frozen contexts contain actual rule bytes/effective inputs, not an assertion that
ignored payload is immutable. Effective excludes may differ by workspace/config
even for equal trees. Choose command-frozen excludes semantics explicitly; if
detecting mid-command external config edits remains required, compare those small
inputs at barriers rather than rebuilding Git sandboxes and redoing every query.

The native Git oracle is a jw concern. It never runs inside a diffdirs worker
callback. `--verbose --non-matching -z --stdin` supplies a framed response even
for a negative answer; negated patterns must be interpreted as not ignored.
Known selected-root/link queries can be batched; discovery needs incremental
answers because a nonignored matched directory may contain eligible descendants.

## Refusal timing: recommended change and strict alternative

The recommended fast path checks all targets' structural eligibility before
starting, then checks payload interiors as it processes them. This deliberately
allows a deep violation in target B after target A completed, or after earlier
entries of B changed. It cannot preserve the current whole-batch “all static
blockers discovered before refill” promise.

The violating entry is not acted on. Permission changes on surviving accepted
ancestors are restored on failure. Report partial progress and the original
bookmark/commit recovery coordinates. A fresh diff converges after the violation
is resolved, subject to the ordinary JJ remedy if tracked refill was interrupted.
There are no new journals, receipts, or resume cursors.

If strict before-any-mutation refusal is required, use the main design's
**inspect → retain operation plan → apply** alternative. It must inspect opaque
one-sided interiors too. This retains the policy guarantee at the cost of plan
memory and first-write latency; it does not make subsequent I/O transactional.

Dry-run remains a complete read-only payload audit, not a root-only fast preview
that says READY. It uses the same guards and covers opaque descendants. Its
existing JJ snapshot behavior is separate from payload writes.

## Landing and evidence

1. Freeze/reuse rules and remove query process fan-out. This improvement works
   with either refusal-timing contract.
2. Preserve payload and integrate incremental sync while retaining current guards.
3. Select streaming or strict-plan refusal timing; implement the complete
   comparison-plus-recursive-action guard coverage.
4. Remove redundant interior walks and amend the maintained operator contract.
5. Measure zero per-entry mount calls and zero mandatory probe work, then
   shared-source multi-target streaming independently.

Use native end-to-end measurements and separate syscall/process attribution, as
in the [previous performance assessment](measurement1.gpt6a.md). That earlier
fixture had only 100 payload files: it does not establish performance on the
192k-entry motivating payload. The initial targets are zero clean-payload writes,
no repeated payload-wide guard pass, and process counts independent of candidate
root count. Latency improvements need actual integrated measurements.

Further alternatives in the main design: a caller-owned inspected-frame cursor
for stateful control, a uniform one/two-sided tree engine, coordinated source-to-many
streaming, and an in-process ignore matcher verified against Git.
