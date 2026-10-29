---
type: ImplementationHandoff
title: Pure structural validation of restored ordinary contexts
description: Review correction separating recomputable integrity from safe selector, ignore-chain and native/link reference interpretation, with resealed malformed-context and bounded receiving checks.
resource: /design/coordinated-refill/context-validation1.sol61x.md
status: verified-review-correction
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e7930131cffemJnROGPjnIatHG"
    title: Implements frozen-owner structural reference checks, verifies resealed restored Search values before missing-source I/O, and records bounded receiving checks, the stable provider receipt and a later transient provider compile failure
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent identifies the checksum-only restoration gap and commissions pure structural validation without admission replay or provider edits
  - resource: /design/coordinated-refill/execution1.sol61x.md
    revision: 8b9f152b
    title: Preserved contextual cohort predecessor and its checksum-only context-validation account
  - resource: /design/coordinated-refill/execution0.gpt6a.md
    title: Section 5 requires independently reconstructible context interpretation without replaying accepted admission
  - resource: /transfer/workspace/src/streaming.rs
    revision: 3860eccc
    title: Corrected Search interpretation, compact trie backedges, retained error causes and resealed restored-cursor discriminator
  - resource: /transfer/workspace/src/ignored.rs
    revision: 3860eccc
    title: Owner-native selector bounds, ordering, uniqueness and epsilon-closure validation
  - resource: /transfer/workspace/src/frozen.rs
    revision: 3860eccc
    title: Owner-native frozen ignore-chain reference and nesting validation
  - resource: "file:///home/rektide/src/diff-dirs/tree/compare/src/coordinated/mod.rs"
    title: Provider validates restored policy contexts before root or route access and preserves the context coordinate and cause
  - resource: "file:///home/rektide/src/diff-dirs/design/architecture/execution3.sol61x.md"
    revision: 1f1385de
    title: Final stable provider 0.6 and V3 grouped/cache/reporting code checkpoints and owner verification receipt, supplied after the receiving checks
verification:
  method: Fresh focused Nextest builds, strict receiving-owner Clippy, formatting and explicit rkt Markdown/provenance checking
  scope: Pure ordinary-context interpretation and existing bounded Linux clear-first N1/N2 behavior; no public resume, CLI cutover or provider delivery claim
---

# Restored ordinary residuals must be safely interpretable

**Code correction: `3860eccc`.** The preserved
[predecessor account](/design/coordinated-refill/execution1.sol61x.md#context-values-and-validation)
correctly calls the hashes nonsecret, but its Search gate checked only coordinate
and checksum consistency. A deliberately resealed residual could still contain
references that panic in the subsequent pure callbacks. This correction implements
the interpretation obligation in section 5 of the
[accepted execution design](/design/coordinated-refill/execution0.gpt6a.md).

## Exact correction and cost

- [`Patterns::validate_prefix`](/transfer/workspace/src/ignored.rs) checks every
  pattern index and inclusive terminal offset, strict state order/uniqueness and
  each recursive state's epsilon successor. It scans the residual once; it does
  not regenerate the residual from its path.
- [`FrozenIgnore::validate_state`](/transfer/workspace/src/frozen.rs) checks both
  lane chains against their own immutable rules: known keys, normal relative
  coordinates (empty rule-root allowed), ancestry of the current coordinate and
  strictly nested unique order. An ignored directory cannot include its own rule
  key: ignored descent never activates that key. No rule matching runs here.
- [`PathIndex::validate_reference`](/transfer/workspace/src/streaming.rs) checks
  native/link node bounds and binds every present index to the Search coordinate.
  One additional **`usize` parent backedge per frozen trie node** permits walking
  the referenced node's parents against reversed coordinate components and existing
  child maps. No duplicated full-path table or saved representation is needed.
  This is reference validation, not root-prefix policy replay; `None` contains no
  node reference to dereference.
- `SourceQualified::validate_context` runs these owner checks before checksum
  serialization or callbacks. Errors become `CheckError` with the owner's original
  cause retained, rather than stringified or caught panics. Provider restoration
  preserves the target and relative coordinate around that cause. The adjacent
  descent error mapping also stops flattening its original cause.

The checksum still detects integrity/binding differences, **not historical
authority**. Structural checks establish safely interpretable references; they do
not prove which earlier names or rules produced an otherwise well-formed residual,
the historical justification of an absent trie cursor or omitted rule key, or past
admission. Those remain under the existing caller-owned checkpoint contract. No
keys, signing, HMAC, new persistence format or public API were added. Policy/schema
versions and exact JW 0.9/provider 0.6/schema-3 receiving pins remain unchanged.

`SelectedMirror` is unchanged: no selector, ignore or native/link readmission below
Q. Incremental Search descent, byte-named admitted interiors, Unknown observations
and residual administrative/kind/device guards retain their existing owners.

## Discriminating evidence

The new restored-cursor test obtains an actual saved Search edge at `pkg/sub` and
builds an independently allocated equivalent policy. **All 29 malformed variants
are deliberately resealed and serialized/restored**: pattern index/offset,
duplicate/order/epsilon structure, out-of-range and wrong-coordinate native/link
indices, and unknown/nonancestor/duplicate/unordered/abnormal ignore keys in both
chains, plus the impossible ignored-current-key shape.

After S moves away, the valid independent context passes pure interpretation and
reaches the expected missing-source I/O error on bind. Each malformed context
instead returns the provider's typed context refusal at `pkg/sub`, with its nested
owner cause, no bound source session or route checks, and zero inspection/search/
admission callbacks. This discriminates structural validation from a checksum
refusal. The Git-oracle corpus now also validates each actual incremental chain;
the selector unit check accepts canonical recursive closures and empty residuals.

## Checks and remaining boundary

The successful receiving checks below rebuilt from the live provider 0.6 sources;
no provider file was edited and no stale-binary result was substituted. Commands ran from this checkout,
with `/usr/local/bin` first, serial tiny fixtures (one donor, at most two receivers,
within 256 ordinary entries/1 MiB), and no blanket `TMPDIR` override.

| Check | Result |
| --- | --- |
| Default transfer-owner unit tests plus actual command and retained N1 integration cases | **71 passed**, 2 isolated child entrypoints skipped; Nextest `3df3da00-45eb-4ce0-a421-02d97b7bf2c8`. |
| `native-dirent` contextual/ignore/selector and real command/failure cases | **23 passed**, 80 skipped by selection; Nextest `f3abc72d-9e89-4fdb-9973-eb389ce808d1`. |
| Strict transfer/engine Clippy, all targets/all features | Passed. |
| Transfer/engine formatting check | Passed. |
| Explicit `rkt check --okf` on this handoff | Passed, including this actual session's provenance. |
| Final six-test receiving rebuild after the provider's stable-receipt notification | **Blocked before tests:** then-current provider `tree/inspect/src/source.rs:309` called undefined `acquire_placeholder_metadata` (`E0425`). |

The default selection was `binary(command) | binary(refill_one) |
package(jj-waltz-transfer-workspace)` with transfer/engine/CLI `--lib --test command
--test refill_one`. The feature selection was `test(streaming::) |
test(residual_validation) | test(immutable_matcher) | test(real_command_) |
test(later_finish_)` with transfer/engine `--features native-dirent --lib --test
command`. Both used `cargo nextest run --locked --test-threads 1 --no-fail-fast`.
Lint used `cargo clippy --locked -p jj-waltz-transfer-workspace -p
jj-waltz-refill-engine --all-targets --all-features -- -D warnings`.

The final attempted six tests selected resealed/frozen context validation, paired
and mixed N2 admission counters, Unknown hints and byte-named routing. Cargo
recompiled provider owners and stopped on the concurrent declared-owner edit;
these six were **not run** and their earlier binaries were not substituted. The
exact tested receiving identity remains the unchanged JW **0.9.0**, all five
provider path products **0.6.0**, saved schema **3** and the existing `--locked`
lockfile. No future manifest/minor-version adoption was attempted.

The subsequently supplied
[stable provider receipt](file:///home/rektide/src/diff-dirs/design/architecture/execution3.sol61x.md)
records final code **`51046cc3`, `4f0c07c0`, `e3e6ef79`, `8d76f95e`**, with report
`1f1385de` and owner-reported 275 workspace / 178 no-default / 20 process passes
plus focused and strict checks. This closes the predecessor's uncommitted-provider
receipt gap; those are **provider receipts**, not relabeled receiving runs. The
new declared/manifest owner and later minor bump remain separate. Its in-progress
compile failure does not revise the stable receipt or establish a green final
receiving rebuild against the new source. Parent owns the next receiving gate.

This is not a full matrix, Rust-1.88/stable proof, performance result, public
CLI/resume cutover or listed-renewal implementation. Tickets, index and the other
[remaining execution boundaries](/design/coordinated-refill/execution1.sol61x.md#exact-remaining-owners)
stay with their existing owners; no provider grouped matrix was repeated here.
