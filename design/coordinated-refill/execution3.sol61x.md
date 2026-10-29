---
type: ImplementationHandoff
title: Public schema-one refill and retirement of the old driver
description: Actual shared public reseed and source-only prepare, captured typed failure projection, exact schema and bounded public subprocess verification.
resource: /design/coordinated-refill/execution3.sol61x.md
status: verified-public-cutover-for-parent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e779f1a9affejTLMoRyE5Nv4xW"
    title: Implements the accepted public contract, migrates native tests to real jw subprocesses, retires the old engine and records exact checks and remaining evidence
  - resource: "opencode://ses_e7a2771c3ffez3sv1oPgD5gPH4"
    title: Parent accepts both reviewed native routes and commissions public cutover while retaining provider documentation and later performance acceptance
  - resource: /design/coordinated-refill/public-contract0.gpt6a.md
    revision: 044b6471
    title: Accepted wire names, availability and status rules, CLI dispatch and retirement map
  - resource: /design/coordinated-refill/execution2.sol61x.md
    revision: 31d2e5ff
    title: Reviewed JW 0.10/provider 0.7 native protocols, resource ownership and reporting seam
  - resource: /app/cli/src/refill_report.rs
    title: Application-owned pure projection, canonical decimals, lossless paths and retained native Rust causes
  - resource: /docs/schema/refill-v1.schema.json
    title: JSON Schema 2020-12 with explicit nullable fields, u128 decimal bounds and receiver completion requirements
  - resource: /refill/engine/src/engine/command.rs
    title: Sole production public refill orchestration and command-scoped desired coordinate
  - resource: /refill/engine/src/engine/command/source.rs
    title: Recorded source context, actual S handoff, final-only tracked conformance and measured process prefixes
  - resource: /app/cli/tests/refill_one.rs
    title: Preserved native physical assertions migrated from temporary export to actual CLI, plus schema and failure boundary cases
  - resource: /app/cli/tests/prepare.rs
    title: Source-only native remotes, retry/lock guards, final repair and nonmutating public preview cases
verification:
  method: Real native jw subprocesses, required reflinks, schema fixtures, Nextest, strict workspace Clippy and explicit Markdown checks
  scope: Bounded Linux public integration; receipts below distinguish completed checks from outstanding work
---

# One public refill path

`jw reseed` now calls `ResolvedCommand::resolve` and either `preview()` or
`execute(&mut ContextualOrdinary, diagnostic)`. `jw prepare` calls only
`ResolvedSource::resolve` and `preview()`/`prepare_source()`. Both reviewed native
routes remain the same implementation; no old-driver fallback or new protocol is
introduced. `--prepare` produces S once and hands it directly to that command.

## Checkpoints and API

- `ea51595b` — public dispatch, initial schema-one DTO/schema/fixtures, engine data
  capture, 0.11 coordinated API marker, old-driver retirement and initial caller/test
  migration. This is a preserved implementation stage, not acceptance or a claim
  that every old test expectation was already migrated.
- `6540bae9` — stricter schema bounds/fixtures and runtime schema validation,
  captured bound-prefix/owned paths, final-conformance availability, one route
  authority, unsupported-platform refusal only, and migrated old execution tests.

Six coordinated JW packages are **0.11.0**, with exact internal pins and the root
receiving lock updated. Provider products remain exactly **0.7.0**. `cargo bump`
was attempted but its installed version panics on workspaces; the coordinated
workspace version/pins were therefore updated explicitly. No provider production
edits, installation, publication, deployment, push or squash occurred.

`ReseedRequest.renewal_route` is the single route authority. The separate route
argument is removed from `ResolvedCommand::resolve(cwd, request, diagnostic)`.
`PrepareRequest` and prepare help no longer accept ignored-copy fields/flags.
Resolver errors are typed `ResolutionFailure { facts, location, source }`:
available profile/source intent, successfully bound selections/origins and skips
survive without post-error discovery. `CommandReport.desired` records exact D
before the first receiver preservation attempt. `SourceContext` retains the
recorded-before-work source coordinate, effective assertion/result, selected
upstream and declarations; successful S remains exclusively the accepted handoff.

Preparation steps retain selected/started/succeeded process state and actual
nullable process durations. A later successful argv may repair tracked edits;
there is one final tracked conformance check. An identity/handoff failure after
that check does not rewrite its successful outcome. Preparation failure never
projects an accepted S. Source phase time includes the handoff once; producer and
process spans are nested actual measurements, not subtracted approximations.

## Projection and availability

[`refill_report.rs`](/app/cli/src/refill_report.rs) owns serde, the outcome envelope,
pure borrowed-report projection and compact plain rendering. Engine/provider
types and original error chains remain native and downcastable. The CLI buffers
serialization before writing once; successful/failed parsed JSON outcomes include
early resolution failures. Builder stdout and diagnostics go to stderr. Already
handled outcomes bypass generic `context::add_workspace_hint`; no recovery-time
rediscovery, text parsing or filesystem census is used to fill a report.

Counts, step indexes and integer-ms durations use `Decimal(u128)` serialized as
canonical strings. Schema bounds include u128 overflow, not merely JavaScript's
safe-integer boundary. Unavailable facts are null. Unix paths retain exact OsStr
bytes in lowercase hex alongside display text. Actual provider failure coordinates
can identify a containing checked directory rather than a child mentioned only
inside a diagnostic; reporting preserves that real coordinate, never parses a
more specific path out of text.

Native child coordinates on failure are **last observed facts**, not promised
fresh after a failed native command. Full sparse patterns do not prove an
interrupted replacement finished. Returned coverage/effects, successful adoption,
native conformance, ordinary satisfaction and loaded finish remain separate facts.
Complete peers remain complete on later/shared failures; started unfinished peers
are partial; untouched rows are not_started. Ordinary retention and deviation are
projected independently. Source diagnostics are emitted once per native/ordinary
source phase and are not raw syscall counts or physical-sharing evidence.

### Field-contract deviations for parent review

1. **Explicitly permitted additive capture keys:** `desired_manifests` and
   `owned_manifests`, both canonical decimals. The latter counts actual required
   distinct O indexes; clear-first/explicit-clear do not count the provider's
   implicit empty ownership. These map actual capture counters, not reconstructed
   leaf/work metrics.
2. **Specified spelling where the design left a string:** observed comparator
   identity renders `family@version:lowercase-hex-parameters`. It is an opaque
   diagnostic identity, not Rust Debug text or a counter.
3. Non-Unix builds retain a refusal-only schema-one boundary; they do not attempt
   a second engine, fallback, path transcoding or effects. No cross-platform runtime
   certification was performed.
4. No other intentional wire-key/status/count/path-contract deviation. Optional
   process timings are now measured; ordinary per-lane application time remains
   unavailable/null. Unsupported/unresolved native profiles remain null.

## Retirement and retained dependencies

Removed the legacy `run_refill`/`print_refill_report`, `ReseedReport`/Outcome/Status,
source-only old run routing, Plan/execute/readiness, repeated `verify_disk`, payload
audit/realization, FutureContent/FutureNode, old preparation producer and temporary
`refill_one_from` projection. Its meaningful native N1 assertions now invoke actual
`jw` and derive their test-only convenience aliases from schema 1.

Kept root identity/config/PreparationLock/snapshot, bookmark/history/selection,
native upstream selection, and creation's actually used IgnoreRules, tracked,
confinement, Auxiliary and link rollback helpers. Removed only unused old reseed
FrozenRules caching, not the shared transfer module. Default ordinary execution
stays serial; feature forwarding still uses the same command-owned min4 borrowed
pool and native-dirent backend. Native transitions stay serial, peer apply fails
fast, and disabled selection has no ordinary runtime/pool.

## Verification receipts

The native checkpoint's 0.10 protocol evidence remains valid; these checks test
the integrated public boundary and retirement. New owned fixtures use one donor,
at most two receivers, ≤256 ordinary entries and ≤1 MiB. Native commands use
`/usr/local/bin` first and no blanket TMPDIR. No workload import or performance
campaign ran.

- `81231050-532b-40a1-ad30-e5c2ebecef2d`: **139 passed, 2 failed, 1 skipped** across
  public prepare/bookmark/N1, CLI/engine units and callable command tests. The two
  new test expectations were corrected: a raw native command can itself be the
  primary message (not an inner cause), and the provider's actual checked path
  for an unsupported opaque child is its containing directory. Neither correction
  changes production error semantics or invents a leaf coordinate.
- `b841da4a-ea39-42c8-9560-1a45fa78e504`: native failure/no-post-error-query case
  passed; byte-path fixture still expected the unowned child coordinate and was
  subsequently changed to make the actual checked directory byte-named.
- `acee600d-64eb-48e2-b7f7-490d10c8e297`: **23 passed, 1 failed**, including all
  **12 creation ignored-copy guards** after retirement, plus public preview,
  preparation, native, ordinary and schema cases. The new retention discriminator
  initially chose a fixture whose actual totals were equal; it was changed to
  routing into a nested selected root, not by reconstructing either count.
  `e9edb135-a694-429b-a0ab-6acaac30f60f` then passed the real public unequal-total
  discriminator in default serial mode.
- `25060783-910f-4c36-a9b3-911d43602a97`: **74 passed, 16 failed** during the old
  reseed test migration. These were old diagnostic/preview/unchanged-before-work
  expectations, not attributed to an unrelated baseline. Meaningful filesystem
  assertions were retained: guards deferred to execution may follow native work;
  authorized clearing can precede required-link failure; explicit recovery precedes
  retrying an interrupted sparse checkout. No readiness engine was restored.
- `ac8b74a9-eec0-4b8b-849c-9e76bc99ef5b`: **35 passed, 2 failed, 113 skipped**, with
  both `native-dirent,payload-parallel`. It covers public N1/N2/both routes/default,
  assertions, exact D, source/ordinary/final failure, byte paths and migrated old
  guards. Remaining failures were two over-specific old test assumptions: a
  symlink obstruction can refuse before native work, and a moved primary may
  surface the original typed JJ failure rather than the resolver's owner message.
  Neither production behavior was altered to make those assertions pass.
- `dec883d5-f349-4bb4-8441-6624498be36b`: **8 passed, 143 skipped**, same two
  features. Rechecks both corrected tests and adds real PUBLIC N2 finish failure
  on both routes: the earlier receiver stays complete with its actual created
  link; both ordinary lanes retain returned clones; the offending receiver has
  partial finish and actual returned zero link work. Also checks structured
  preparation failure after a successful earlier argv, step index and successors.
- `9dae3359-226f-4b6b-82a6-9976388eb159`: final **5 application-report tests passed**,
  including compact saved O/bookmark/work-tip/root rendering, original typed
  primary/secondary causes, explicit fixtures, missing/null required effects,
  incomplete coverage, numeric/noncanonical/overflow counts and wide u128 totals.
- Final strict workspace `cargo clippy --workspace --all-targets --all-features
  -- -D warnings` passed. CLI/engine/transfer all-target Clippy also passed with
  `--no-default-features`. The existing Cargo manifest homepage warning remains.
  Rustfmt passed; explicit `rkt check --okf` passed all six revised knowledge docs.
- The first feature test attempt accidentally replaced the environment's pinned
  nightly shim path with rustup stable; native-dirent correctly failed to compile
  its nightly-only std API. Re-running with `/usr/local/bin:$PATH` retained the
  configured nightly and produced the successful feature receipts above. No
  provider workaround or toolchain configuration change was made.
- An earlier broad migration run hit its tool timeout after 47/185 tests; it is
  not a completed suite or baseline attribution. Old preview/output/payload-flag
  assumptions were migrated rather than restored to production.

There are **no known remaining failures in the affected selections**: each surfaced
migration failure has a named successful follow-up. This is not a single final
whole-workspace Nextest run, cross-platform/stable-toolchain certification or a
representative-latency claim. New default/feature receipts are integration proofs,
not performance samples. Native-dirent/payload-parallel use the existing actual
command-owned pool/native protocol; their prior worker/empty/fail-fast evidence
in execution2 is not relabeled as new instrumentation here.

## Next owner

Parent reviews this public contract and its receipts before commissioning the
performance adapter's final schema binding/acceptance. That parser/campaign is
not implemented here. Its existing explicit `--source-at` profile remains
**live-asserted-source**, not plain prepared-source latency. Provider maintained
docs, tracker/index and cross-project consumer accounts remain with the parent.
