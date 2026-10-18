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
  the library's streaming loop. Non-default `payload-parallel` borrows one pool
  per command, reused across roots/targets and created only for actual payload
  work. Workers are capped at `min(available_parallelism, 4)`, J matches workers
  and K is 256; JJ and rule caches remain serial. No per-root pools or runtime
  configuration framework were introduced.
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

After upstream approval, parent diagnosed that routing failure as the malformed
test environment: a Git-only sibling under this checkout inherits the enclosing
real `.jj`. The verified artifact's physical file list matched its historical
tracked tree at operation `a85936a20b65`, and `jj --at-op a85936a20b65 status`
reported no changes. `jw remove unregistered` deliberately removed that exact
registration/directory; subsequent workspace listing and existence checks confirm
removal. No bookmarks were requested for deletion and no other workspace was swept.

The remaining eight integration binaries were then run with a correctly built
runtime binary and a genuinely external temp root:

```sh
PATH=/usr/local/bin:$PATH TMPDIR=/tmp/opencode \
  CARGO_BIN_EXE_jw=$PWD/target/debug/jw cargo nextest run -p jj-waltz \
  --test cli --test colocate --test context --test context_routing \
  --test external_owner --test links_stress --test shell_ui --test ui_cli \
  --no-fail-fast
```

158/159 passed without skips; only the previously recorded missing-workspace
cleanup test remains failing. Git-only discovery passes. Do **not** globally
place those discovery fixtures inside a JJ checkout merely to enable reflinks.
The focused required-CoW suite above and external-discovery suite need separate
temporary roots. The updated real bind-mount regression passed with required
mount namespaces; package-wide all-target/all-feature Clippy and changed-file
format checks also passed after the review corrections.

## Limits and next evidence

Parent review resumed this integration only after explicit approval of library
`8811c0561e90c84b29e22faa8b26c9a5cc6f76a1` (runtime `243be8e2`), recorded in
[the library review](diffdirs-review0.gpt6a.md). The first review correction refuses
duplicate mountpoints anywhere in a workspace's covering ancestor chain, not
just the deepest lexical prefix: an overmounted ancestor may hide that prefix.
Applicable autofs roots/ancestors also refuse before recursive traversal, since
traversal itself can invalidate fixed topology. Four focused topology tests pass,
including the discriminating hidden-child alias fixture. Checks remain upfront.
The retained-region validator now precomputes selected absolute boundaries once
into a small set, rather than joining every candidate root at every entry. Its
focused pruning/prefix-neighbor regression passes; no payload inventory is built.

Deep policy/reflink failures can follow tracked changes or completed peers. The
offender is untouched; earlier accepted work can remain. Existing JJ/bookmark
recovery applies; no journal, receipt, automatic repair or resume was added.
Stat-key convergence does not repair mode-only or same-size/same-mtime byte edits.
Stable paths and mount topology remain required, not a hostile-writer sandbox.

The accepted Rust tools now drive bounded sequential native synthetic controls;
their immutable inputs, scope and raw locations are recorded in
[integrated controls](integrated-controls0.sol61m.md): 16/16 campaigns and 64/64
samples passed, including clean candidate identity invariants. Mixed descriptive
timings do not select a default. Main-workload import and
separate instrumented process-count evidence remain outstanding. No new benchmark
framework was written. Library test reports were accepted from the parent, not
redundantly rerun here.

The experimental pool integration passed clean inode/mtime/ctime, sparse/subtree,
selected leaf-root, and equal/one-sided refusal tests in default and feature
builds. A two-root/two-target regression passed in both variants; the pool's
lazy/capped construction test passed. Updated package all-target/all-feature
Clippy passed. Default selection remains serial pending integrated evidence.
