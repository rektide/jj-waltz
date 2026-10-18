---
type: ImplementationReview
title: "Checked diff-dirs integration review"
description: Parent source review of Sol's static checker, recursive application, audit, and root-transition seams before jj-waltz integration; performance assessment awaits interleaved tooling.
resource: /design/up-performance/diffdirs-review0.gpt6a.md
status: reviewed-for-integration
generated: { by: "model:openai/gpt-6-astra", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb262a81effea1HgF0pPS3f1sy"
    title: Reviewed the delivered library against the user's streaming and static-specialization direction and identified caller integration obligations
  - resource: "opencode://ses_eb24f98a3ffewdVqa9lao6tXQu"
    title: Sol xhigh implementation and verification handoff, with performance judgment deferred under unstable host I/O
  - resource: "file:///home/rektide/src/diff-dirs/src/check.rs"
    title: Generic Check and NoCheck specialization, borrowed observations and lazy relative paths
  - resource: "file:///home/rektide/src/diff-dirs/src/pair.rs"
    title: Checks over every immediate observation before publishing comparison children
  - resource: "file:///home/rektide/src/diff-dirs/src/audit.rs"
    title: Explicit one-tree read-only traversal including one-sided interiors
  - resource: "file:///home/rektide/src/diff-dirs/sync/src/lib.rs"
    title: Checked binding, explicit replacing root, streaming mirror, and separate audit
  - resource: "file:///home/rektide/src/diff-dirs/sync/src/apply.rs"
    title: Shared generic recursive checks, staged root/leaf transitions, directory-local metadata reuse, and restoration
  - resource: "file:///home/rektide/src/diff-dirs/sync/src/paths.rs"
    title: Real ancestor binding and no-follow final leaf handling
  - resource: "file:///home/rektide/src/diff-dirs/design/sync/checked0.sol61x.md"
    title: Builder verification, precise contracts, immutable controls and benchmark limitations
  - resource: /design/up-performance/inline-policy0.gpt6a.md
    title: Intended jj-waltz scope partition and topology/ignore front guards
---

# Checked diff-dirs integration review

Reviewed delivery through `8811c056`; runtime changes end at `243be8e2`.
**The library seam is suitable for the intended integration.** Performance
selection remains pending the user's requested interleaved tooling and Sol's
independent reassessment. This review is source inspection, not a new test run.

## Fit to the selected direction

- `Check` is statically dispatched and accepts a borrowed observation. `NoCheck`
  disables construction/dispatch at compile time. There is no interpreted rule
  list or full-tree operation plan.
- The ordinary comparison sees equal entries too. It checks a directory before
  returning its eligible paired children to the existing frontier; speculative
  inspection does not evade refusal.
- One-sided descendants are checked in the actual copier/remover. Initialization
  and type transitions use those kernels, closing the walker-only coverage gap.
- `Applier::mirror*` streams compare/apply without invoking audit. Retained
  immediate metadata is directory-local, not a payload inventory.
- Explicit replacing-root binding supports a selected target file or symlink.
  It validates real ancestors, leaves the final link unfollowed, stages the new
  directory, and removes the old leaf only after successful preparation.
- Failed recursive operations preserve completed-progress reporting and restore
  modes on surviving existing ancestors. Private staging cleanup uses its own
  ownership rather than applying user policy to implementation artifacts.
- No mount-ID lookups, mount-table searches, or reflink probes were added to the
  entry path. Those remain caller front guards or actual clone outcomes.

The builder reports 66 default and 94 all-feature required-reflink tests, default/
all-feature Clippy and formatting, and 6/7 doctests passing. Those checks are
accepted as reported rather than rerun merely for confirmation.

## Obligations for G's jj-waltz integration

1. Use checked mirror (or a checked walker and the same borrowed checker), never
   an unchecked walk with only a checked applier: that would miss clean entries.
2. Keep normal execution free of `audit` and remove the existing early full-
   payload validation walks as well as destructive payload removal. Keep actual
   tracked-path and unrelated-content obligations explicit.
3. For dry-run, audit selected source roots **once per command**, then each
   existing target root. Calling pair `Applier::audit` for every target repeats
   the same source scan; the lower-level `diff_dirs::audit` provides the needed
   separation. Shared donor failure leaves targets not assessed, not READY.
4. Implement topology assessment before source/target traversal, including file
   bind mounts, selected-root mounts, broader replace-content footprint, and
   obvious source/target bind aliases. Remove recursive mount calls from the
   Linux fast path only when that front guard actually covers them.
5. Capture frozen ignore inputs once and reuse native Git answers/processes.
   Preserve adaptive root selection, desired-tree ignore checks, and supported
   old-tracked-to-payload transitions; the latter is not automatically invalid.
6. A custom checker should use the native basename and existing file type. Only
   rare absolute source symlinks need path formatting/diagnostic collection;
   don't lock or compute relative paths for every ordinary file. Collection must
   not affect policy decisions or present speculation as committed progress.
7. Describe metadata-key convergence accurately: mode-only edits and byte edits
   retaining size/mtime are not repaired. Copied files preserve modes/mtime, but
   an unchanged payload is deliberately not an archive-metadata repair sweep.
8. Preserve early structural refusals while documenting that deep payload
   refusal can follow tracked changes or successful peers. Use existing
   out-of-band JJ recovery; there is no new journal/resume mechanism.

## Performance evidence still required

The actual donor is the user's `~/a/a/opencode`, including nested selected roots
and several destination variants. Library-level clean traces establish unchanged
enumeration/metadata counts and zero writes; they do not establish integrated
reseed latency. Source snapshotting, ignore queries, tracked refill, and duplicate
validation can still dominate if integration leaves them intact.

Use immutable baseline/candidate artifacts and balanced interleaved runs under
the acknowledged background I/O. Separate first fill, clean repeat, sparse delta,
and subtree operations. Report process/syscall counts independently of timings.
New owned realistic fixtures support mutation tests; live donor inspection and
existing feature branches remain read-only during benchmark setup.
