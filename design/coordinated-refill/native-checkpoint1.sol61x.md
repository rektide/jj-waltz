---
type: VerificationCheckpoint
title: Native foundation verified against diff-dirs 0.3.0
description: Receiving dependency adoption, final native handoff gates, cold producer-error correction and bounded command-failure reporting; explicitly not contextual cohort or replacement delivery.
resource: /design/coordinated-refill/native-checkpoint1.sol61x.md
status: verified-bounded-checkpoint
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e79dd6eaeffeNFNkKJzrJKre9x"
    title: Adopts the reviewed provider foundation, rebuilds and runs the pending native gates, fixes oversized preparation errors, replaces receiver Debug dumps and verifies bounded text with original causal chains
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions this bounded receiving adoption and verification slice, requests the reporting correction and holds the next provider rewrite
  - resource: /design/coordinated-refill/execution0.sol61x.md
    revision: deb7fb75
    title: Native implementation handoff, exact pending commands and unresolved provider/public integration interfaces
  - resource: "file:///home/rektide/src/diff-dirs/design/architecture/execution0.sol61x.md"
    revision: 509e2704
    title: Reviewed provider 0.3.0 foundation and its explicit contextual traversal, sharing and listed-execution limits
  - resource: /design/coordinated-refill/verification0.sol61x.md
    title: Tiny owned fixture restrictions and distinction between native correctness gates and future two-route/performance evidence
verification:
  method: Fresh focused Nextest integration binaries, five error-report unit discriminators, strict native/state/engine Clippy, formatting and explicit Rekon Markdown/provenance checking
  scope: Linux default-feature dev/test builds using the exact installed JJ profile; no full workspace, contextual ordinary cohort, ReplaceAuthorized, public CLI cutover or performance claim
  result: All final selected gates passed; the initial strict Clippy failure and its separate correction are recorded below
---

# Native foundation receiving checkpoint

**The pending native handoff gates now pass against the reviewed provider 0.3.0
foundation.** Fresh binaries establish the clear-first command, producer, PLANNED
preview, preservation barriers and forward partial-finish behavior. They do not
establish contextual cohort transfer or authorized tracked replacement.

## Owned commits

| Commit | Change |
| --- | --- |
| `516c4071` | Adopt exact `=0.3.0` provider pins in CLI dev-dependencies, refill engine and workspace transfer; update the five receiving product entries in `Cargo.lock`. |
| `8a934bd8` | Box the cold `PreparationFailure.completed` report and adjust the command's report projection; preserve progress and original error chains. |
| `7d1e9b8c` | Replace `CommandFailure`'s all-receiver Debug dump with bounded summary text; add four report discriminators and strengthen the existing ordinary-effect assertion. |

Provider inputs were the reviewed `fb4719cc` and `48d50af0` implementation
checkpoints, with handoff `509e2704`. **No provider Rust API migration was needed.**
No provider source, public CLI dispatch, parent-owned ticket or maintained guide
was edited. Both unrelated starting drafts remain untouched and uncommitted.

## Corrections and failure semantics

The first strict Clippy run found the predecessor's `PreparationFailure` result
error at **144 bytes** (`clippy::result_large_err`). Boxing its structured report
fixes that gate without suppressing the lint or erasing the cause. The new unit
test verifies saved progress, direct typed-cause access and the original I/O cause
through the enclosing error chain. This was not a native correctness failure.

[`CommandFailure::fmt`](/refill/engine/src/engine/command/report.rs) now prints the
phase, optional receiver, original cause chain, receiver-completion/native-state
counts, and small actual file/link counters for the failed receiver. Unknown
ordinary partial progress remains explicitly unavailable, never fabricated as
zero. The full public `report` and typed source are unchanged.

The summary is independent of report path-vector growth: a synthetic two-receiver
test adds 256 retained path entries and 256 attempted phases, verifies identical
text below 512 bytes, and checks that the rich paths remain accessible. This bounds
the **report contribution**, not arbitrary receiver names or original error text;
neither names nor causes are silently truncated. Other tests cover absent receiver
labels and preservation of the original typed I/O error after wrapping.

## Exact commands and results

All commands below ran from `/home/rektide/src/jj-waltz`, with no blanket `TMPDIR`.
The existing isolated tests use owned scratch roots, one donor and at most two
other native workspaces, with tiny ordinary payloads. No new native fixture,
route emulation, imported workload or full-workspace suite was introduced.

Dependency adoption:

```sh
PATH=/usr/local/bin:$PATH cargo update --offline \
  -p diff-dirs-tree-api -p diff-dirs-transfer-sync
PATH=/usr/local/bin:$PATH cargo check --locked -p jj-waltz-refill-engine
```

Passed. The committed lock changes only the five provider product versions; no
unrelated registry dependency update was retained.

Final native gates, exactly as requested by the handoff:

```sh
PATH=/usr/local/bin:$PATH cargo nextest run --locked \
  -p jj-waltz-refill-engine --test command -p jj-waltz --test refill_one \
  --test-threads 1 --no-fail-fast
PATH=/usr/local/bin:$PATH cargo clippy --locked \
  -p jj-waltz-native-jj -p jj-waltz-workspace-state -p jj-waltz-refill-engine \
  --all-targets -- -D warnings
PATH=/usr/local/bin:$PATH cargo fmt \
  -p jj-waltz-native-jj -p jj-waltz-workspace-state -p jj-waltz-refill-engine -- --check
```

- **20 passed, 2 skipped**: 11 command cases plus 9 existing `refill_one` cases.
  The skipped tests are isolated subprocess entrypoints, invoked by the fixtures,
  not unverified semantic cases. Final Nextest run:
  `e1811d6d-1ebe-4b09-a0cf-ad214d8b7d34`. Cargo rebuilt engine and integration
  binaries after both corrections; no stale binary stood in for the final run.
- Strict owner Clippy and formatting **passed** after correction. The existing
  Cargo manifest warning about redundant homepage/repository remains; no Rust
  warning was allowed past `-D warnings`.
- An earlier receiving run also passed all 20 cases, before the report correction;
  it is not an additional feature matrix.

Focused producer/report tests:

```sh
PATH=/usr/local/bin:$PATH cargo nextest run --locked \
  -p jj-waltz-refill-engine --lib \
  -E 'test(engine::command::report::tests::) | test(preparation_failure_keeps_progress_and_original_cause)' \
  --test-threads 1 --no-fail-fast
```

**5 passed, 35 skipped by selection**; run
`f520da47-54dc-4edb-8b4c-39e1221ebe1b`. No other engine unit suite was claimed.

## Compiler, features and native identity

Actual compiler: `rustc 1.101.0-nightly`, commit
`822182db4c4a1827256881d8c7aa685f0bcf2b11`, host `x86_64-unknown-linux-gnu`,
LLVM `23.1.3`. Although `rust-toolchain.toml` says `stable`, this receipt names the
compiler actually found with `/usr/local/bin` first. It does not verify the
declared Rust 1.88 minimum or a separate stable toolchain.

These are unoptimized **dev/test** builds with no optional JW/provider features
selected. `cargo tree --locked -p jj-waltz-refill-engine -e normal --prefix none
--format '{p} features=[{f}]' --depth 3` confirmed empty feature sets for the
engine and provider product crates. Neither `payload-parallel` nor provider
`native-dirent` was enabled. The exact **native JJ profile** is a different
property from provider enumeration backend/features.

`/usr/local/bin/jj --version`, queried from `/`, returned
`jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`, the selected supported
profile. Its SHA-256 was
`0b7462db466ffb0394cb30b0bb56480da3bce10a91f98b80b109302b0ec610d8`.
No release build, timing comparison or performance benefit is asserted.

## Remaining integration boundary

This receipt closes only the former provider-build blocker in the
[native handoff](/design/coordinated-refill/execution0.sol61x.md#exact-remaining-integration-interfaces).
That handoff's unresolved interfaces remain unresolved:

1. Prepared desired/owned provider inputs and actual complete `ManifestCoverage`
   must replace the strict-seed projection before cache adoption changes.
2. `ReplaceAuthorized` remains refused before probes/mutation, with no fallback.
3. A contextual ordinary adapter must use one source session/cohort runtime and
   retain actual partial cursor reports. The legacy N1 adapter still cannot expose
   structured ordinary failure progress.
4. Public dispatch, standalone producer cutover, route/JSON/help/schema/docs and
   old-driver retirement remain parent-owned later integration.

The [provider checkpoint](file:///home/rektide/src/diff-dirs/design/architecture/execution0.sol61x.md#next-coherent-slice)
supplies the complementary prerequisite list, not evidence those features already
exist. The [verification preparation](/design/coordinated-refill/verification0.sol61x.md#small-later-integration-slice)
still governs future real-command two-route and bounded performance evidence.

Document gate:

```sh
cargo run --quiet --manifest-path /home/rektide/src/rekon/tool/rust/Cargo.toml -- \
  check --okf design/coordinated-refill/native-checkpoint1.sol61x.md
```

The explicit Markdown/provenance gate passed. Parent-owned index/guide promotion
is deliberately left to the parent rather than rewriting the predecessor receipt.
