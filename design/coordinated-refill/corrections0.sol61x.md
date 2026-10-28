---
type: ImplementationAccount
title: Narrow corrections to loaded link targets and ordinary byte-name authority
description: Review-driven preservation of declared link target resolution and classification ordering for unselected native-byte names, without CLI or lifecycle changes.
resource: /design/coordinated-refill/corrections0.sol61x.md
status: first-correction-verified-second-pending
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: "2026-10-28" }
sources:
  - resource: "opencode://ses_e7bae73c1ffeS9xaw29hyAhrNJ"
    title: Implements and verifies the two specifically authorized review corrections while CLI and product steering remain paused
  - resource: /design/coordinated-refill/review0.gpt6ax.md
    revision: efa19e67
    title: Independent pinned reproductions and narrowly requested corrections
  - resource: /design/coordinated-refill/implementation1.sol61x.md
    title: Delivered callable N1 scope, native evidence and forward-link contract
  - resource: /design/coordinated-refill/implementation2.sol61x.md
    title: Delivered conditional-routing scope that these corrections preserve
  - resource: /workspace/state/src/links.rs
    title: Loaded declarations, classification and separate forward/reversible application
  - resource: /workspace/state/tests/links_stress.rs
    title: Actual public-API target-resolution and private-directory regressions
  - resource: /transfer/workspace/src/streaming.rs
    title: Source-qualified ordinary authority and production cursor
  - resource: /transfer/workspace/src/ignored.rs
    title: Existing UTF-8 selector boundary
  - resource: "file:///home/rektide/src/diff-dirs/tree/inspect/src/visitor.rs"
    title: Supplied early presence/kinds and late facts; no fabricated callback data
  - resource: "file:///home/rektide/src/jj-waltz/.test-agent/coordinated-refill/review/README.md"
    title: Independent isolated-probe procedure read as review evidence, not modified or rerun
verification:
  method: Failing focused regressions before correction, focused nextest after correction, affected strict Clippy and formatting
  result: Loaded-link correction passes all three new API regressions and existing link tests; ordinary-name correction pending
  scope: Tiny Linux fixtures only; no native workspace mutation, upstream changes, CLI cutover or performance claim
---

# Review corrections

This account records only the two correctness fixes commissioned after the
[independent review](review0.gpt6ax.md#correctness-findings-requiring-action).
The [N1 implementation](implementation1.sol61x.md#delivered-boundary) and
[conditional-routing adoption](implementation2.sol61x.md#exact-action-and-ownership)
remain the delivered boundary. No slices 3–4, native sequence, lifecycle/default,
version/history, copying breadth or recovery changes are authorized here.

## 1. Preserve declared link target resolution

`LoadedWorkspaceLinks::new` no longer applies `normalize_lexical` to the bound
target. Source validation, administration/overlap rejection and receiver-relative
target binding remain unchanged. Both `inspect` and forward application now see
the declared `jump/../shared`, not a rewritten `shared`. Filesystem resolution can
therefore follow `jump` before processing `..`, just as the legacy API does.

Private-directory ownership still uses source-equals-target declarations; ordinary
relative targets and `./private` self-targets work before and after the resulting
directory exists. A target that only **lexically** collapses to the source is not
mistaken for a private-directory declaration. No new canonicalizing load/preflight
or traversal was introduced. Existing path-comparison diagnostics and forward
versus reversible failure behavior are unchanged.

Three public-API regressions establish:

- The same `shortcut → jump/../shared` declaration, applied by new forward and
  legacy APIs to receivers with both possible destinations present, preserves
  the installed link spelling and resolves to **outside/shared**. Loaded inspection
  retains that spelling before application and reports satisfied afterward.
- `shared → jump/../shared` remains an active link and conflicts with an unrelated
  receiving `shared` directory rather than silently treating it as private.
- Receiver-relative `shared` and source-equals-target `./private` retain their
  behavior, including declaration-only ownership before directory creation.

The first two tests failed on the predecessor; the self-target/relative control
passed. After the fix:

```sh
cargo nextest run --locked -p jj-waltz-workspace-state -E 'test(links)'
cargo clippy --locked -p jj-waltz-workspace-state --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

**29 tests passed**, including the three new cases, forward partial-effects,
legacy rollback, missing-target and symlink-parent diagnostics. The filter also
included one existing ownership/symlink case. Strict affected-crate Clippy and
formatting passed. Cargo emitted only the pre-existing redundant-homepage manifest
warning; no source lint was suppressed.

## 2. Ordinary byte-name classification

Pending in the next separate correction commit: decide ancestor authority before
attempting selection decoding of an ineligible final name; preserve unrelated
target-only and source-leaf names while retaining mirror authority inside an
already admitted Q root. The existing supported-name boundary for a genuinely
needed non-UTF-8 routing directory is not broadened.

## Ownership and limits

The two starting user drafts, independent reviewer report, topic README/index and
overarching architecture documents are untouched. Broad native and upstream test
results remain attributed to the preceding accounts rather than rerun. No install,
live workspace mutation or large fixture/performance campaign is part of this work.
