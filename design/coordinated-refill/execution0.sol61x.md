---
type: ImplementationHandoff
title: Shared native command, source producer and PLANNED boundary
description: Native slice A checkpoint, final-only producer and read-only planning implementation, verification receipts, and exact pending provider and public-cutover interfaces.
resource: /design/coordinated-refill/execution0.sol61x.md
status: implementation-checkpoint-awaiting-provider-integration
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e7a14b1afffe7eRp0y0L0JssYC"
    title: Implements shared native bindings, serial native vector execution, structured forward results, final-only source production and read-only planning; discovers and tests version-query config migration; records the provider build blocker and integration handoff
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent commissions this independent native/input/producer/preview slice and later authorizes the completed provider design
  - resource: /design/coordinated-refill/execution0.gpt6a.md
    title: Selected native ownership, installed-build profile, clear-first control, source producer, preview and forward-error contracts
  - resource: /design/coordinated-refill/verification0.sol61x.md
    title: Existing tooling, isolated test environment, bounded fixture limits and later two-route/cohort evidence requirements
  - resource: "file:///home/rektide/src/diff-dirs/design/architecture/annealing0.gpt6ax.md"
    title: Four ownership lifetimes and whole-command architecture
  - resource: "file:///home/rektide/src/diff-dirs/design/architecture/execution0.gpt6a.md#9-reusable-prepared-leaf-lists-and-proactive-inputs"
    title: Parent-authorized desired/owned listed interface, source-session ownership, complete manifest gate and staged delivery
  - resource: "file:///home/rektide/archive/jj-vcs/jj/lib/src/secure_config.rs"
    title: Installed-build startup migration, config IDs, metadata protobuf, copied/moved owner handling and secure-config construction
  - resource: "file:///home/rektide/archive/jj-vcs/jj/cli/src/config.rs"
    title: Secure-config root selection and configuration loading during native startup
verification:
  method: Focused Nextest subprocess fixtures and native/state owner tests; formatting; strict owner Clippy; explicit Markdown/provenance check
  scope: Clear-first native control and independent producer/planning work only; final engine rebuild blocked by concurrent provider observation changes; no replacement/cohort performance or public CLI claim
---

# Architectural checkpoint

**One vector-ready command boundary now owns native execution.** The original
`refill_one_from` remains only a compatibility projection plus the old one-lane
ordinary adapter. It no longer owns another native orchestration. CLI dispatch is
unchanged, as commissioned. This is not completion of the two-route/cohort delivery.

## Commits and files

| Commit | Coherent stage |
| --- | --- |
| `d44dbfee` | Exact native build classification, read-only registration/config primitives, snapshot-free ownership input and structured actual link finish effects. |
| `70e36580` | Shared command/native/producer/preview implementation, old prototype reduction, source-only registry view and seven bounded integration tests. |
| `b9f0fc80` | Preservation and donor-coordinate discriminators, exact history evidence, pending preview meaning, source identity checks, raw profile/timing reports and native startup migration correction. |

Main implementation:

- [`native/jj/src/refill.rs`](/native/jj/src/refill.rs): `NativeProfile`,
  `RegistrySnapshot`, source-only registration view and secure-config startup guard.
  [`jj.rs`](/native/jj/src/jj.rs) adds frozen current-workspace names.
- [`command.rs`](/refill/engine/src/engine/command.rs): one `ResolvedCommand`,
  serial preservation/native transitions, one ordinary cohort-shaped call,
  loaded serial finish, shared capture and phase-aware failure.
- [`binding.rs`](/refill/engine/src/engine/command/binding.rs): stable
  `RepositoryBinding`/`BoundWorkspace`, separate from changing `CommitFacts`.
- [`native.rs`](/refill/engine/src/engine/command/native.rs): `NativeTree`, shared
  membership/indexes, native-only inputs, extracted clear-first operation,
  D-addition admission and snapshot-free finish checks.
- [`source.rs`](/refill/engine/src/engine/command/source.rs): `ResolvedSource`,
  `PreparedSource`, source evidence, producer steps/failure and final conformance.
- [`report.rs`](/refill/engine/src/engine/command/report.rs): actual phase outcomes,
  original causal errors, exact saved O/history coverage and partial receiver state.
- [`replacement.rs`](/refill/engine/src/engine/replacement.rs): compatibility
  projection and narrow `LegacyOrdinary`; no independent native driver.
- [`links.rs`](/workspace/state/src/links.rs): `LinkFinishFailure` retains actual
  link/directory successes, failed path and original error. Planned link counts no
  longer masquerade as completed counts. Reversible lifecycle semantics remain.
- [`ownership.rs`](/workspace/state/src/ownership.rs): supplied frozen registration
  inventory and refill-only ownership check, without repeated primary discovery.
- [`command tests`](/refill/engine/tests/command.rs): isolated real-native fixture
  and argv recorder. The existing [N1 tests](/app/cli/tests/refill_one.rs) exercise
  the new engine through the retained compatibility projection, unchanged.

No production file in `transfer/workspace`, `app/cli` or diff-dirs was edited.
The two pre-existing unrelated drafts remain uncommitted and untouched.

## Delivered ownership and semantics

### Native command and lifetimes

`ResolvedCommand::resolve(cwd, request, route, diagnostic)` loads configuration,
selection, source/receiver bindings, topology, effective patterns and loaded link
declarations once. Repository reads use explicit captured operations. Ownership
checks use one frozen Git registration inventory, not full context rediscovery per
receiver. The exact raw tested build is retained separately from its parsed version.

Execution probes actual donor/receiver routes before destructive refill. Plain
reseed performs no live donor snapshot; a named assertion performs one live diff.
An in-process prepared handoff is consumed without repeating the producer or its
tracked conformance proof. Explicit D resolves after production/assertion; omitted
D is actual S, not the upstream goal.

Receivers are snapshotted serially and each exact O is recorded immediately.
Bookmark coverage is checked from a subsequent shared view. The final barrier
checks **both source S and every saved O** before clearing any receiver. A later
receiver snapshot can rewrite an earlier receiver or the donor through shared
history; this is refused, not silently repinned.

K(S), K(D) and K(O) are captured once per distinct *proven* tree. Empty single-parent
native facts provide the existing bounded equality proof. Each tree gets one path
index. S=D reuses that index directly; the ordinary adapter borrows its shared path
set rather than cloning it. D-addition paths are computed once. `FrozenIgnores`
shares extracted bytes by equivalent tree and rules by tree plus effective
excludes; it is not replaced with a second ignore implementation.

Complete native transitions run serially. `NativeInputs`, membership tables and
the extraction cache are explicitly dropped before ordinary transfer. Saved
receivers retain O coordinates/history and links, **not O leaf lists or donor
`tree_state` bytes**. Ordinary keeps only source/bindings, patterns, S/D exclusion
paths and needed frozen rule objects.

### Producer

`ResolvedSource::resolve` discovers primary/source registrations only; it does not
load receiver link plans, validate receiver sparse state, enumerate tracked leaves,
select ordinary roots or inspect payload. An ignored socket and malformed unrelated
link declarations do not turn source-only preparation into a payload census.

`prepare_source(&resolved, diagnostic)` retains the preparation-only lock, initial
source snapshot, pinned upstream/publication/fast-forward guards and ordered argv.
Each successful argv receives stable root/registration/change/parent checks, **not
a live tracked diff**. One final live diff establishes tracked conformance. Thus a
successful intermediate edit followed by repair is allowed, while process failure
stops later commands and final divergence fails distinctly. Native change identity
changes still refuse. No repeated `verify_disk`, payload selection or recursive
payload audit enters this producer.

`PreparedSource.report.commit` is actual resulting S; `upstream_goal` is separate.
Step statuses and final-conformance state survive failure. A command failure retains
that preparation report and the original `PreparationFailure` cause. The handoff
does not claim that existing ignored data was freshly built.

### Read-only PLANNED boundary

`preview()` returns before probes, locks, source production/assertion, snapshots,
membership extraction, transfer/runtime construction or payload inspection. It
reports pending preservation, unprobed cloning and uninspected obstructions. Source
assertion results and future prepared S remain unknown; explicit post-prepare D is
a pending revset, not a fabricated current result. Effective pattern declarations
are reported without a root census.

**Native startup was an important discovered write path.** The installed JJ can
migrate legacy secure config even when invoked as `jj --version` in a checkout.
Build identification now runs at `/`, outside repository discovery. The actual
checkout is inspected in-process for legacy/missing/misbound secure-config metadata
before any workspace JJ process. Read-only discovery refuses migration/repair/copy
conditions rather than triggering them. The guard checks this exact build's
20-hex-character IDs and canonical `metadata.binpb` owner encoding; it is deliberately
not a general future-JJ config decoder. Metadata-store construction remains read-only.

Divergent operation heads are also refused without reconciliation. The tests compare
repository/WC bytes and metadata paths, not atime.

## Verification and current blocker

All native integration fixtures use required clones, tiny files, one donor and at
most two receivers; Nextest is explicitly serial. No blanket `TMPDIR`, imported
workload, deployment or performance campaign was used. Earlier fixture setup errors
(`git push --allow-new` and undescribed published work) were corrected to the actual
installed command behavior.

Verified during this slice:

- Seven new command tests passed together at the shared-command checkpoint: vector
  native reuse/failure, incomplete desired installation, final-only two-step source
  success/direct handoff, stopped/final-divergent producer failure, read-only preview,
  source-only scope and divergent-operation preview refusal.
- A later combined native run passed **17/18** cases: all nine existing N1 native
  cases plus eight new command cases. The remaining new secure-config migration
  test exposed the version-query write above. After correction that test passed
  independently. This is not being represented as a final all-tests pass.
- Final native/state owner selection: **4 passed, 42 skipped**, including exact-build
  refusal, config noninitialization, structured forward link failure and reversible
  link rollback. Strict native/state Clippy passed.
- Strict native/state/engine Clippy passed at the earlier shared-command checkpoint.
  Final owned-package formatting passed. This handoff passed Rekon's explicit
  `check --okf` (structure, destinations, anchors and session provenance).

**Final engine rebuild is blocked by concurrent provider work**, not repaired here:
`diff-dirs/tree/observe/src/names.rs:62` uses a `#[cfg]` expression in a position this
compiler rejects, then the Unix build references absent non-Unix `NameTable.names`
fields. Two final focused Nextest build attempts stopped there before executing any
test. Later native/report refinements and the added donor-rewrite and later-finish
tests consequently still need the final engine build/run after that owner settles.
Do not count an old test binary as their verification.

Final integration commands once the provider compiles:

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

The new suite now includes explicit receiver-O and donor-S rewrite barriers,
change-identity refusal during preparation, pending post-prepare D and a later link
finish failure retaining an earlier complete receiver and its actual link count.

## Exact remaining integration interfaces

1. **Listed native effects:** use the accepted provider
   [`PreparedLeafList`/`OwnedLeafSet` contract](file:///home/rektide/src/diff-dirs/design/architecture/execution0.gpt6a.md#91-prepared-inputs-and-exact-provider-api).
   Prepare exact desired S specs once and owned O sets once per distinct tree.
   `EffectBinding` owns roots/copy policy, not an ordinary visitor. Current clear-first
   still calls existing `tracked::seed`; do not invent a replacement API or local
   copier around missing production interfaces.
2. **Manifest adoption gate:** replace the current strict-seed success projection
   `native.manifest_complete` with actual returned `ManifestCoverage`. Successful
   complete desired/owned coverage, not clone counts or physical name-union
   exhaustion, gates cache adoption. Preserve returned prefix coverage/effects on
   failure. A missing listed leaf is already a tested failure in the control.
3. **ReplaceAuthorized:** the enum is accepted by the shared command/plan/report,
   but execution refuses it **before probes or mutation**, with no fallback.
   Add the selected same-tree-O child, `RenewOwned`, full manifest gate, optional
   S-to-D restore and rebase-only-new-C sequence inside the existing native owner.
   Do not remove sparse clear and call that replacement support.
4. **One ordinary runtime:** implement `OrdinaryTransfer` against the forthcoming
   contextual `sync_cohort` adapter. Its `admit` runs before native mutation;
   `transfer(&OrdinaryCohort, &mut [OrdinaryOutcome])` runs exactly once after all
   native receivers succeed. It receives frozen lane-specific rules/links and
   shared native exclusions, but no old membership/cache bytes. Keep one
   `SourceSession` generation shared by compare/sync; runtime resources are not
   policy or saved-state meaning. JW continues ordered fail-fast apply-one.
5. **Actual partial ordinary errors:** the old N1 compatibility adapter cannot
   extract structured reports from `sync_one` on failure. It honestly reports
   `OrdinaryState::Partial` with `report: None` while retaining that original cause.
   The new adapter must assign actual cursor reports before returning
   `OrdinaryFailure { lane, source }`; no diagnostic-string parsing or fake zeros.
6. **Public cutover:** dispatch reseed to this shared command and standalone prepare
   to the producer, then remove the compatibility projection/old orchestration.
   CLI `--renewal-route`, clear-first default, JSON envelope/lossless large counts,
   help/docs/schema and coordinated version/pin changes remain parent-owned later
   work. Current public CLI still has the old preview/prepare behavior. Resolution
   errors precede `CommandReport`; the later envelope must allow zero resolved rows.

The accepted provider design's v2 contexts, comparators, staged facts, observation
resources/cache and desired/owned execution are prerequisites to that integration,
not features this native checkpoint claims to have emulated or delivered.
