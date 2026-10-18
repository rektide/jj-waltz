---
type: ToolGuide
title: Rust-owned preparation and reseed performance controls
description: Checked-in native fixtures, balanced variants, single-reseed samples, Rust JJ/Git process counting and retained correctness evidence.
resource: /tools/performance/README.md
status: maintained-rust-only
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-17 }
updated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2165cc8ffe4vOmOvDBe7aXQu"
    title: Replaced supported performance commands with native Rust, diagnosed the old feature fixture expectation and proved native/counting cases against preserved BEFORE
  - resource: "opencode://ses_eb75a514cffe7jUOMWNZAIm3fX"
    title: Historical matched native journal-deletion measurements and process-count controls
  - resource: "opencode://ses_eb87f7b4cffe23FlLmOW2DpE7o"
    title: Historical separation of native and instrumented timing
  - resource: "opencode://ses_eb8a7487affeE2smndZ914OQMS"
    title: Historical strict GNU cp baseline
  - resource: "opencode://ses_eb8947cfaffeQ1jxia03URCymY"
    title: Original performance harness, fixtures, subprocess counts and compatibility checks
  - resource: /src/reseed/prepare.rs
    title: Source-only preparation contract measured by native phases
  - resource: /tests/prepare.rs
    title: Isolated local remote and configuration precedent
  - resource: /design/up-performance/handoff0.glm53m.md
    title: Historical motivation for per-file process-count regression controls
---

# Native performance tooling

`jw-perf` is a **development-only Cargo member**, not a production dependency.
It shares the Rust artifact/schedule/execution/report primitives in the sibling
`diff-dirs/perf/runner` checkout. No Python runtime, import, shell wrapper or
package installation is needed by supported commands. Linux, native JJ/Git,
GNU cp and a required-reflink filesystem are needed. Use configured Rust 1.101
nightly; this does not change JW's production compiler policy or release profile.

## Run

```sh
cargo build --locked --release -p jj-waltz-perf
target/release/jw-perf run .test-agent/native-clean-NEW \
  --baseline .test-agent/inline-policy/jw-before \
  --baseline-provenance .test-agent/inline-policy/baseline.json \
  --scenario clean --destinations 2 --feature-files 2 --blocks 8 --seed 1701
```

Every output must be NEW. Without `--candidate` / `--candidate-provenance`, both
labels intentionally use the SAME saved baseline: this is a plumbing/noise
control, not an optimization. Add an independently built candidate and its JSON
build provenance (original binary `sha256`/`binary_sha256` required) for real A/B.
Only a named incremental candidate should get `--require-clean-label candidate`.
The historical baseline rewrites clean payload inodes; that is recorded behavior,
not failed baseline latency/correctness. A mislabeled incremental candidate is
rejected with its measured phases and rewrite counts retained.

Scenarios: `init`, `clean`, `sparse`, `subtree`. All use equivalent NEW private
payload resets, source guards and N independent destination workspaces.
`--files`, `--symlinks`, `--feature-files`, `--payload-files`, `--payload-bytes`
control dimensions; excessive feature requests fail rather than being truncated.
Without `--fixture`, the generated native fixture is synthetic. Use the imported
fixture below for the actual donor's tracked shape and ALL selected payload roots.
Neither tiny synthetic nor imported plumbing smokes establish a performance win.

## Actual opencode import and N2 campaign

From this checkout, with the sibling diff-dirs checkout present:

```sh
cargo build --locked --release -p jj-waltz-perf
target/release/jw-perf snapshot "$HOME/a/a/opencode" .test-agent/opencode-import
target/release/jw-perf build . .test-agent/opencode-serial --variant serial
target/release/jw-perf build . .test-agent/opencode-p4 --variant payload-parallel
target/release/jw-perf run .test-agent/opencode-clean-serial \
  --fixture .test-agent/opencode-import/fixture \
  --baseline .test-agent/inline-policy/jw-before \
  --baseline-provenance .test-agent/inline-policy/baseline.json \
  --candidate .test-agent/opencode-serial/jw \
  --candidate-provenance .test-agent/opencode-serial/provenance.json \
  --scenario clean --destinations 2 --feature-files 2 --blocks 8 --seed 1701 \
  --require-clean-label candidate --timeout-seconds 1800
```

All outputs must be NEW. For the parallel campaign change the output name and
candidate paths to `.test-agent/opencode-p4/{jw,provenance.json}`. For first-fill
and sparse use `--scenario init` / `sparse` with NEW outputs and omit the
clean-only invariant flag. Each campaign interleaves preserved BEFORE and that
one candidate. Build everything before timing; never run campaigns concurrently.

`snapshot` reads native JJ root/operation/commit/file inventory with
`--ignore-working-copy`; it NEVER invokes live status, prepare, reseed, dry-run,
fetch, install or builds. The pinned JJ coordinate defines tracked MEMBERSHIP;
copied bytes are those observed on disk, not an atomic commit export. An isolated
Git index built from copied tracked files (including forced-tracked ignored files)
checks the copied `.gitignore` rules. Only outermost eligible `node_modules` and
`**/dist` directories are selected; ** consumes zero components too. Source/target
roots are disjoint, `.jj`/`.git` are excluded, unrelated ignored caches/secrets
are not copied, and every copy requires private reflinks. The resulting owned
fixture is fingerprinted and verified before a campaign. Root lists, tracked
inventory, provenance and source/target signatures are retained in its JSON.

Source tracked files, directories and literal links retain their real shape.
Owned Git/JJ fixture setup force-adds the recorded inventory and appends only
fixture-private ignore entries; donor freshen actions are replaced, never run.
The same root list is reset, overlaid and independently verified for EVERY
destination, not silently reduced to a single node_modules tree. The retained
integration smoke proves four roots, a native link, forced-tracked binary content,
N2 feature histories and unchanged donor JJ operation. Missing files/submodules,
root symlinks, administrative entries inside selected payloads, unsafe paths and
tracked/payload overlap are explicit refusals, not silently omitted work.
The opencode adapter requires the donor's tracked root `.gitignore`; it will not
invent an extra tracked file to isolate its private configuration.

`--phase prepare|preview|reseed` selects one native phase. `--phase all --scenario
init` measures preparation, preview, and first-fill reseed separately and retains
their sum. Preparation uses only a freshly created OWNED local Git remote and a
Rust fixture installer; no network/live-donor fetch or user install is invoked.
The installer counter must be exactly one; preview/preparation must preserve
worker commits and payloads. Setup explicitly creates `.jj` repositories and
checks every root/repository link before adding or measuring any workspace.

**Feature contract:** reseed creates an empty child of the explicit donor goal;
omitting `--at` does NOT retain the old worker feature tree. The old Python N2
failure expected the wrong final tree. Rust checks the requested final goal/tree,
actual root and parent, exact feature-file delta, and preserved old feature
bookmark IDs/content separately. Payload bytes, links, modes and regular-file
mtimes are checked outside native timing. Clean inode/mtime changes are streamed
to disk and counted without a full-tree runtime map.

## Native versus instrumented counts

### Integrated build variants

The Cargo owner wired non-default `payload-parallel` to optional `rayon`,
`diff-dirs/parallel` and `diff-dirs-sync/parallel`. The default remains serial.
Application pool code is owned separately; enabling the feature is not itself
evidence of a speedup. Equivalent direct Cargo builds are:

```sh
cargo build --locked --release -p jj-waltz --target-dir target/perf-serial
cargo build --locked --release -p jj-waltz --features payload-parallel --target-dir target/perf-p4
```

Prefer `jw-perf build` above: it pins the current committed application and
library coordinates, rejects mismatching production source/manifests, records
actual compiler/profile/features/environment/input hashes, builds into a unique
target directory and preserves a mode-0555 `jw` plus `provenance.json`. Optional
`--revision` / `--library-revision` accept exact full IDs for repeat experiments
from matching prepared checkouts; the tool never switches or resets a checkout.
No fake revision/path placeholder is silently accepted as a ready artifact.

```sh
target/release/jw-perf run .test-agent/native-counts-NEW \
  --baseline .test-agent/inline-policy/jw-before \
  --baseline-provenance .test-agent/inline-policy/baseline.json \
  --phase all --scenario init --destinations 1 --timing instrumented
target/release/jw-perf check .test-agent/native-counts-NEW --max-jj-calls total=500
target/release/jw-perf compare SMALL LARGE --scaling \
  --max-jj-growth 0 --max-file-show-growth 0 --max-git-growth 0
```

Native mode runs the real tools directly. Instrumented mode uses Rust symlink/
exec wrappers to log argv boundaries, cwd, PID and chronology for JJ/Git calls;
wrapper overhead is included and **must not be called native latency**. Count
gates require instrumented mode and successful complete campaigns. Budgets use
`PHASE=N` for `prepare`, `preview`, `reseed`, `total`; file-show and Git budgets
have corresponding flags. Scaling allows only increased regular-file count,
requires identical JW binaries, tool/adapter hashes and Rust engine identity,
and gates maximum observed phase counts. Comparisons never silently pool runs.
Absolute native child paths/private Git exec helpers can bypass PATH wrappers:
these are observed tool calls, not system-wide process accounting.

`campaign.json`, append-only `samples.jsonl`, `summary.json`, immutable artifact
copies, per-command stdout/stderr/process JSON and raw `*.processes.jsonl` all
remain under the owned output. Mirrored seeded blocks balance chronology;
external I/O load is not eliminated. API timing is never substituted for JW
native-process timing. Setup/reset/hash/verification costs are retained and
excluded. Native command records include CPU, Linux peak RSS and timeout status;
Command timeouts (180s default, `--timeout-seconds` to change) kill ordinary
descendant process groups. No global cache
drops, automatic cleanup, significance test or causal load correction exists.

Raw copy control (owned source from a retained sample, never a live workspace):

```sh
target/release/jw-perf copy-baseline OWNED_CASE/repo .test-agent/native-copy-NEW
```

This uses strict GNU cp on tracked/payload selections, excluding JJ administration
and fixture config, with independent payload verification. It is not whole reseed.

## Verified checkpoint and development

Preserved BEFORE SHA: `03e0ada406426d9496c602e22c9dd0de452f788e424dab9b7069c07f812e715c`.
Actual native N2/two-feature clean, N2/two-feature full first-fill phases, N1 sparse
and N2 subtree smokes passed. Clean BEFORE rewrote four regular payload inodes per
target with zero mtime changes, correctly retained as historical behavior.
Instrumented full phases passed at 4 and 100 tracked files; zero JJ/file-show/Git
process growth gates passed. Those tiny controls are NOT real performance wins.

Evidence is retained at `.test-agent/rust-native-*`. Earlier failed first-fill
attempts are retained too: preparation correctly refused a local-only goal, then
a described unpublished source child. The corrected isolated local-remote,
undescribed-empty-WC fixture passed. The deliberate
`rust-native-reject-legacy-as-incremental` campaign fails only the explicit
candidate invariant, preserving baseline correctness and all raw results.

```sh
cargo nextest run -p jj-waltz-perf
cargo clippy -p jj-waltz-perf --all-targets -- -D warnings
cargo fmt -p jj-waltz-perf -- --check
```

Seven default tests cover literal Rust exec-wrapper argv, command classification, native
count requirements, exact feature dimension refusal and streaming inode order.
The explicit retained integration test runs with:
`cargo nextest run --locked -p jj-waltz-perf --test import --run-ignored ignored-only --no-capture`.
It requires the preserved BEFORE artifact, so is opt-in on fresh checkouts.
Shared Rust primitives and actual library scenario instruments have their own
tests in diff-dirs. Integrated candidate/large-workload timing is separate work.

The six active Python files and old JSON experiment recipe have been retired
after native parity; there is no Python dependency or supported launcher.
Historical methods/results remain in exact
committed history, raw fixtures and frozen evidence. Recover the earlier guide
with `git show a35da35d7635e14be70fb4aab37ea6edbd933ef8:tools/performance/README.md`;
the same revision preserves its old `measure.py`, fixtures and test sources.
