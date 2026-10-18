---
type: ImplementationReport
title: Streaming reseed integration
description: Delivered payload synchronization, frozen native ignore contexts, topology guards and correctness evidence; performance measurement remains separate.
resource: /design/up-performance/streaming0.sol61m.md
status: implemented-awaiting-parent-review
generated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2158009ffeyJ0q8NuTd6g1RP"
    title: Implemented and verified authorized streaming reseed integration without changing performance tooling
  - resource: /design/up-performance/inline-policy0.gpt6a.md
    title: Scope partition, topology front guard, frozen selection and streaming refusal design
  - resource: /design/up-performance/diffdirs-review0.gpt6a.md
    title: Parent-approved checked library seam and caller obligations
  - resource: "file:///home/rektide/src/diff-dirs/sync/README.md"
    title: Checked mirror, explicit replacing root, required reflinks and partial-progress contracts
  - resource: /docs/reseed.md
    title: Maintained operator guarantees and out-of-band recovery
---

# Streaming reseed integration

## Delivered

- Cargo path dependencies on `../diff-dirs` and its `sync` crate. Normal payload
  execution calls `new_replacing_checked(..., ReflinkRequired, guard).mirror()`;
  no intermediate tree map, audit or clone probe. Ordinary serial execution uses
  the library's streaming loop; no additional pool/scheduler is introduced.
- `reseed::payload::Guard` checks native names and observed file types, including
  equal entries and recursive one-sided actions. Only rare absolute source-link
  diagnostics format paths and lock a deduplicated set. No Git in the checker.
- Selected payload survives tracked clearing/refill, except actual old tracked
  leaves. This preserves supported old-tracked-to-payload transitions. Desired
  and donor tracked overlap remain errors. Root files/links are staged replacements,
  never followed. Unrelated target contents remain unless explicitly replaced.
- Initial and execution region validation stops at selected payload boundaries.
  Dry-run audits donor roots once, then each target; donor failure means not
  assessed, never READY. Replace-content retains conservative broader validation.
- Linux mountinfo is captured before selection/traversal, with native octal bytes,
  optional-field separator parsing, regular-file bind boundaries and filesystem-
  relative alias checks. Workspace-root mounts are allowed; nested mounts and
  covered/stacked roots fail closed. Root admin regions are excluded only where
  the operation never copies/deletes them. No per-entry Linux mount lookups remain
  on this worker path. Source-only preparation keeps its existing conservative
  checks across external command boundaries. Other platforms keep native checks.
- Command-owned frozen rule contexts reuse tracked ignore extraction, isolated Git
  sandboxes and memoized native oracles. Whole-tree equality already established
  by JJ allows distinct equal-tree coordinates to share extraction; effective
  backend/global excludes still key contexts independently. Native framed queries
  handle nonmatches and negation; stderr is drained and the child/reader joined.
  Effective excludes are command-frozen, not reread at every execution barrier.

## Correctness evidence

`PATH=/usr/local/bin:$PATH`, with `TMPDIR=$PWD/.test-agent/reseed-tests` so unit
fixtures use the reflink-capable checkout filesystem:

```sh
JW_TEST_REQUIRE_COW=1 JW_TEST_REQUIRE_MOUNTS=1 cargo nextest run \
  --lib --test reseed --test reseed_bookmarks --test prepare \
  --test freshen_source --test creation_ignored --no-fail-fast
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

279 tests passed, no skips; Clippy and formatting passed (existing Cargo homepage
manifest warning remains). After equal-tree cache reuse, its focused cache test
also passed. New native coverage establishes clean payload inode/mtime/ctime
preservation, sparse deltas/stale removal, absent/file/link roots, opaque/equal
administrative refusal, special files, frozen oracle reuse/error/negation, and
real directory/file binds inside an owned private user/mount namespace.

The unconfigured full nextest invocation passed 335/438: 101 legacy tests failed
because runtime `CARGO_BIN_EXE_jw` was unset, plus two colocation failures. One is
the already-recorded missing-workspace cleanup failure; the other routed a Git-only
fixture to the actual repository and created
`/home/rektide/archive/EzraCerpac/jj-waltz.unregistered`. Parent was notified;
no cleanup or repeat of that routing test was attempted. Earlier unit failures
with `/tmp` were unsupported reflinks, resolved by setting TMPDIR as above.

## Limits and next evidence

Deep policy/reflink failures can follow tracked changes or completed peers. The
offender is untouched; earlier accepted work can remain. Existing JJ/bookmark
recovery applies; no journal, receipt, automatic repair or resume was added.
Stat-key convergence does not repair mode-only or same-size/same-mtime byte edits.
Stable paths and mount topology remain required, not a hostile-writer sandbox.

No latency benchmarks or new benchmark framework were run. Incoming reviewed Rust
tooling should measure process counts, clean zero-write behavior, sparse/one-sided
changes and real-shaped multi-target workloads against the preserved binary.
Library test reports were accepted from the parent, not redundantly rerun here.
