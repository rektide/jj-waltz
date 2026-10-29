---
type: VerificationHandoff
title: Bounded native renewal verification preparation
description: Reuses existing performance tooling, adds standalone native trace counts, and identifies the exact remaining adapter work for two-route command verification on one reused owned fixture.
resource: /design/coordinated-refill/verification0.sol61x.md
status: preparation-only
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: 2026-10-29 }
sources:
  - resource: "opencode://ses_e7a2227e7ffeTQH5TWAkgiaxOz"
    title: Independently inspects existing tooling and command boundaries, implements and checks standalone trace counting, and prepares bounded integrated verification without a refill campaign
  - resource: "file:///home/rektide/src/diff-dirs/design/architecture/annealing0.gpt6ax.md"
    title: Commissioned two renewal routes, whole-command attribution and bounded semantic evidence
  - resource: /tools/performance/README.md
    title: Existing artifact preservation, native fixtures, balanced campaigns and exec-wrapper contracts
  - resource: /tools/performance/src/build.rs
    title: Actual application and library source guards and compiler, feature and artifact provenance
  - resource: /tools/performance/src/workload.rs
    title: Current fresh-per-sample workload, fixed D=S arguments and phase measurement boundary
  - resource: /tools/performance/src/context.rs
    title: Isolated command environment, process records and wrapper injection
  - resource: /tools/performance/src/instrumentation.rs
    title: Wrapper records, command classification and standalone count implementation
  - resource: /app/cli/src/cli.rs
    title: Public reseed still calls the old driver and exposes no selected renewal-route argument
  - resource: /refill/engine/src/engine/replacement.rs
    title: Callable N1 phase timers, grouped native work and exact current profile restriction
  - resource: /app/cli/tests/refill_one.rs
    title: Existing N1 native outcomes, D-not-S shapes, obstruction, private-clone and finish-failure precedents; not rerun here
  - resource: "file:///home/rektide/src/diff-dirs/perf/runner/src/scheduling.rs"
    title: Existing seeded mirrored order with a minimum two blocks
  - resource: "file:///home/rektide/src/diff-dirs/perf/runner/src/execution.rs"
    title: Reusable process-versus-setup accounting and retained failures
verification:
  method: Three focused Nextest cases, package formatting and strict package Clippy; read-only real-JJ version wrapper probe; source and command-help inspection
  scope: Tooling only. No semantic refill, candidate production build, native protocol proof or performance result.
---

# Verification preparation, not a route result

**Reuse the existing Rust tooling; do not start another campaign framework.**
Tooling commit `36b1a38056402b7439d43d223b4f5b9a4fe1c943` adds
`jw-perf count TRACE` and distinguishes `sparse list` from `sparse set` in count
summaries. Existing total-count budgets are unchanged; raw argv still distinguishes
`sparse set --clear` from other sparse updates. No production code changed.

## What is ready and what is not

| Existing capability | Verified boundary / remaining work |
| --- | --- |
| `jw-perf build` | Pins immutable JW/library inputs, rejects mismatching production sources before/after build, and retains compiler, release profile, features, environment, input hashes and binary SHA. Reuse unchanged. |
| Balanced schedule and process runner | Two blocks give four commands in mirrored A/B order. Reuse `scheduling::schedule` and `process::run`; setup/reset/verification remain outside command timing. |
| Native/instrumented separation | Existing Rust JJ/Git symlink wrappers preserve literal argv, cwd, PID and start chronology. `count` now works without a campaign. Wrapper timings are not native latency; absolute child paths can bypass them. |
| Existing `run` workload | **Not the requested reused-fixture harness.** Every sample creates another donor, local remote and N receivers. Do not run it unchanged for this commission. |
| Existing scenarios | `clean`, `init`, `sparse`, `subtree` apply the same shape to every lane. No mixed missing/paired N2. `--feature-files` changes O; reseed argv still uses the same `goal` for S and D, so it does not exercise D≠S. |
| Actual command | Public `run_refill` still calls `run_with_diagnostics`, not `refill_one_from`. The latter refuses N2, preparation and preview. No route selector is wired. **`--replace-content` is ordinary-content deletion, not authorized tracked replacement.** |

Consequently, no current `jw-perf run` invocation establishes either forthcoming
route's correctness or speed. Route spelling and JSON/phase-report parsing remain
pending the reviewed interfaces. Do not benchmark the callable test subprocess
and describe it as the public command.

### Attribution available today

`RefillOneReport` exposes `elapsed_ms`, `native_ms`, `ordinary_ms`, `finish_ms`,
plus `native_install`, `explicit_clear`, ordinary lane progress and link results.
These are callable N1 fields, not public command JSON:

- `native_ms` starts after initial configuration/topology/CoW admission. It includes
  capture/index/rule work, clearing, tracked installation, adoption, optional restore
  and native conformance. It is **not native-subprocess time alone**.
- Tracked installation has progress counts but no separate duration. Do not infer
  its time by subtraction or from wrapper start timestamps.
- Finish includes link work **and final native verification**. The three phase
  timers do not account for all of `elapsed_ms`; expose ingress/unattributed work
  separately rather than claiming exact decomposition.

The later command probe should retain its external process timer, reported phases,
route, successful required-clone/copy counts and partial progress independently.
Missing phase fields mean unavailable, not zero. On failure retain raw process
records and the original cause before attempting optional report parsing.

## Small later integration slice

Extend the existing workload adapter only after command review:

1. Initialize **one owned donor and at most two receivers once**; reuse their roots
   sequentially. Reset matching start signatures outside timing. Do not copy `.jj`
   to fake reset, import a real workload, or use ordinary-content deletion as reset
   authority. Construct D in an already-owned receiver during setup, not a third
   `desired` workspace as the old N1 test helper does.
2. Accept per-route literal argv arrays and independent pinned S/D. Use the **same
   preserved JW binary/features** for both routes; route is not a serial/parallel
   build variant. Every measured invocation must execute the actual command once
   for the selected N1/N2 cohort.
3. Inventory all fixture copies and staging before semantic work. Keep the stated
   aggregate ceiling: **256 ordinary entries and 1 MiB including staging**. Tiny
   tracked leaves and a few 16-byte ordinary leaves suffice. Require real reflinks;
   never turn failure into a byte-copy fallback. Store artifacts outside the fixture.
4. Do the semantic matrix below first, then a short native A/B/B/A (or its seeded
   mirror) on reused matched starts. Run wrapper-count probes separately. No full
   suite, warmup campaign, imported donor or repeated benchmark is needed now.

| Case, for both selected routes | Required discriminator |
| --- | --- |
| Clean N1, D=S | Tracked renewal still executes on equality; required-clone successes and zero byte copies. Ordinary equal file inode/mtime and zero-write progress remain unchanged. Private receiver file objects and write isolation; do not mistake inode reuse for proof of replacement. |
| Paired N2 | Both selected regions exist, with different small deltas. One actual cohort command; stale removal only inside admitted regions; unrelated locals remain. |
| Mixed N2 | A lacks selected Q; B has Q with a stale descendant. A populates through opaque sync; B compares/removes through paired traversal. Count opaque source work separately from paired sharing. |
| D≠S | Tiny modified/added/removed leaves, a directory↔leaf transition, literal symlink and executable mode. Final tracked tree and empty-child parent equal D, registration/root/`.jj` and full sparse state are correct. Do not claim donor sharing for D-only or subsequently native-rewritten leaves. |
| Failure / authority | An unrelated S/D obstruction or retained descendant is not silently adopted/deleted. A required-link finish failure retains known native/ordinary effects and the original cause; no switch to the other route. |

Also check an empty ordinary selection and a vanished qualifying source root:
neither grants target deletion. Use the [existing N1 cases](/app/cli/tests/refill_one.rs)
as outcome precedents, not as a wholesale fixture or a new native protocol proof.
Native cache/protocol approval remains the native architect's gate.

## Commands and receipts

From `/home/rektide/src/jj-waltz`, after matching production inputs are committed:

```sh
PATH=/usr/local/bin:$PATH target/debug/jw-perf build . \
  .test-agent/annealing-evidence/candidate-NEW --library ../diff-dirs --variant serial
target/debug/jw-perf count OWNED_TRACE.processes.jsonl
```

The build command is a **future recipe, not executed here**. Output must be new;
retain `jw` and `provenance.json` together. Do not relabel an old binary as the
candidate. For the later real command, the argument vector is:

```sh
"$JW" reseed worker-a worker-b --from "$OWNED_SOURCE" \
  --source-at "$S" --at "$D" "${ROUTE_ARGS[@]}"
```

Run from the owned donor under `Context`'s isolated environment; omit `worker-b`
for N1. `ROUTE_ARGS` must be filled from the reviewed explicit selector, not guessed
or silently left empty. Use the existing native `process::run` timer for native
samples. For **count-only** samples prepend a directory containing `jj` and `git`
symlinks to the preserved `jw-perf`, set `JW_PERF_REAL_JJ`, `JW_PERF_REAL_GIT` and
`JW_PERF_TRACE` to the explicit tools/new empty JSONL file, and summarize that file
with `count`. Keep setup/verification calls outside that trace window.

Actual checks in this slice:

```sh
PATH=/usr/local/bin:$PATH cargo nextest run --locked -p jj-waltz-perf \
  --bin jw-perf --test wrapper \
  -E 'binary(wrapper) | test(classify_respects_global_values_and_argv_boundaries)'
PATH=/usr/local/bin:$PATH cargo fmt -p jj-waltz-perf -- --check
PATH=/usr/local/bin:$PATH cargo clippy --locked -p jj-waltz-perf --all-targets -- -D warnings
```

**3 passed, 6 skipped; formatting and strict Clippy passed.** The existing
redundant-homepage Cargo manifest warning remains. Initial local borrow/format
errors were corrected; their first logs are preserved. No baseline suite ran.
A real read-only `jj --version` through the preserved Rust wrapper produced one
JJ `(global)` call and zero Git calls. This proves wrapper execution, not refill.

Retained scratch: [`.test-agent/annealing-evidence/`](../../.test-agent/annealing-evidence/README.md).
It contains help, source-match receipts, compiler identity, checks and raw/count
version-probe records. JW production inspected at
`22a8224efbebb57e3da5577dae2dc66432baa616`; shared runner at diff-dirs
`bac8b33a9ce1d90d47e9db143588a522a392824a` (runner source match passed).

| Preserved preparation artifact | SHA-256 |
| --- | --- |
| `.test-agent/annealing-evidence/jw-perf` | `3cb8e34810b4297e6f58df56d940d41b37bd3f16405a668708972b0da8a8450b` |
| `/usr/local/bin/jj` | `0b7462db466ffb0394cb30b0bb56480da3bce10a91f98b80b109302b0ec610d8` |
| `/usr/bin/git` | `07a55b56936312b25c6d6a9fa1d5e8d5672fb588ac1bb26e8dd0b9b61a7bac7b` |

The preparation driver is default-feature **dev/test profile**, not a release JW
candidate. Actual compiler: `rustc 1.101.0-nightly`, commit
`822182db4c4a1827256881d8c7aa685f0bcf2b11`, host `x86_64-unknown-linux-gnu`,
LLVM 23.1.3. JJ reports `0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`
and resolves to `/mnt/bitblaster/archive/jj-vcs/jj/target/release/jj`.
Future production artifacts must obtain their own build receipts; these identities
cannot stand in for them. **No performance or two-route semantic claim is made.**
