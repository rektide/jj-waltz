---
type: ImplementationAccount
title: Narrow corrections to loaded link targets and ordinary byte-name authority
description: Review-driven preservation of declared link target resolution and classification ordering for unselected native-byte names, without CLI or lifecycle changes.
resource: /design/coordinated-refill/corrections0.sol61x.md
status: implemented-two-corrections-awaiting-parent-review
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
  result: 29 focused state tests and all nine streaming tests in default and payload-parallel builds pass; strict affected-crate Clippy, workspace formatting and this account's OKF check pass
  scope: Tiny Linux fixtures only; no native workspace mutation, upstream changes, CLI cutover or performance claim
---

# Review corrections

This account records only the two correctness fixes commissioned after the
[independent review](review0.gpt6ax.md#correctness-findings-requiring-action).
The [N1 implementation](implementation1.sol61x.md#delivered-boundary) and
[conditional-routing adoption](implementation2.sol61x.md#exact-action-and-ownership)
remain the delivered boundary. No slices 3–4, native sequence, lifecycle/default,
version/history, copying breadth or recovery changes are authorized here.

The link correction is commit `eee3fdb7`; the ordinary-name correction and final
verification account are the following separate commit.

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

`SourceQualified::disposition` first tests selection/ignoredness on the **proper
ancestors**, before decoding the current basename. The existing reachability
invariant remains decisive: visiting an interior of Q means its real source
directory was already admitted, including in opaque production recursion.

The classification order is now:

1. Reject administration inside Q, or preserve it outside Q, as before.
2. Inside Q, retain normal mirror authority regardless of source presence and
   without decoding arbitrary descendant byte names.
3. Outside Q, preserve source-absent entries and known source leaves before
   attempting selection matching on their names.
4. If an early source kind is Unknown and its basename is not UTF-8, observe it
   for late metadata classification, rather than treating it as absent or
   attempting directory selection prematurely. A resulting leaf is preserved;
   a real directory still faces the selector's existing UTF-8 restriction.
5. For directory candidates, retain the existing source/desired ignoredness,
   native/link conflict and viable-routing checks. Conditional routing is unchanged.

This changes no path bytes, uses no lossy conversion and adds no admission map,
payload census, filesystem query or broader selection. Failure causes still flow
through the existing slot-refusal and ordinary partial-report chain.

Four focused regressions establish:

- Empty ordinary source, empty frozen ignores, target-only regular basename
  byte `FF`, selector `cache`: succeeds, preserves its bytes, and reports zero
  effects through the actual `sync_one` cursor.
- Source-present non-UTF-8 regular and symlink leaves preserve both target-local
  collisions and target absence; no unselected donor leaf is installed.
- Within an admitted source `cache`, target-only byte files and byte-named
  directories are removed normally. The actual cursor removes three files and
  three directories across paired inspection and opaque target-only recursion;
  a second run has zero effects. This rejects the tempting but wrong fix of
  preserving every source-absent entry.
- Pure supplied-kind tests cover absent, known leaf, Unknown and Directory cases,
  nested byte-name mirror authority and the unchanged directory UTF-8 error.
  Unknown is explicitly deferred to Observe, not silently preserved. This is a
  deterministic policy test, not a claim to have forced a filesystem's listing
  backend to emit Unknown hints.

Before correction, both local-preservation tests and the supplied-kind
classification test failed with `directory selection entry not UTF-8`; the
selected-interior deletion control already passed. After correction, all
**nine streaming tests passed in both
default and `payload-parallel` builds**, including existing wildcard/conditional
routes, empty-policy behavior, native/link overlap, administration/special-entry
refusals and selected replacement behavior:

```sh
env JW_TEST_REQUIRE_COW=1 PATH=/usr/local/bin:$PATH cargo nextest run --locked -p jj-waltz-transfer-workspace -E 'test(streaming::)'
env JW_TEST_REQUIRE_COW=1 PATH=/usr/local/bin:$PATH cargo nextest run --locked -p jj-waltz-transfer-workspace --features payload-parallel -E 'test(streaming::)'
cargo clippy --locked -p jj-waltz-transfer-workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

The existing streaming fixture uses its owned CoW-capable project scratch and
the production required-reflink policy. These are small transfer cases, not native
reseed reruns. A byte-literal test lint was corrected before strict Clippy passed;
no lint was suppressed. Feature-enabled verification does not change the N1
runtime's sequential execution contract.

## Ownership and limits

The two starting user drafts, independent reviewer report, topic README/index and
overarching architecture documents are untouched. Broad native and upstream test
results remain attributed to the preceding accounts rather than rerun. No install,
live workspace mutation or large fixture/performance campaign is part of this work.

No new semantic fork or demonstrated blocker remains in these two corrections.
Non-UTF-8 **routing directories** remain the existing supported-name question,
not a newly accepted input policy. Runtime evidence is Linux-only. The review's
[product decisions](review0.gpt6ax.md#product-decisions-worth-human-steering) remain
with the human and parent; these fixes do not resolve or preempt them.
