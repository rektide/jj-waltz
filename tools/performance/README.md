---
type: ToolGuide
title: Repeatable prepare and reseed performance assessment
description: Retained synthetic fixtures, per-tool subprocess counts, instrumented timings, correctness checks, and deterministic regression budgets.
resource: /tools/performance/README.md
status: maintained
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-17 }
sources:
  - resource: opencode://ses_eb8947cfaffeQ1jxia03URCymY
    title: Formalized the performance harness, isolated interpreter execution, symlink and payload dimensions, JJ/git counting, and comparison checks
  - resource: /design/up-performance/handoff0.glm53m.md
    title: Incident context motivating per-file process scaling checks and mise-safe executable selection
  - resource: /src/reseed/prepare.rs
    title: Source-only preparation contract and freshen argv configuration
  - resource: /tests/prepare.rs
    title: Local-remote fixture and isolated configuration precedent
---

# Performance assessment

`measure.py` measures source-only preparation, exact-commit reseed preview, and
actual reseed in an isolated synthetic repository. **Subprocess counts are the
primary regression metric. Timings include instrumentation overhead.** Python 3.9+
stdlib, POSIX, Git, JJ, and a built `jw` are required.

## Run

Build the binary before measuring; prefer the same release profile for both sides:

```sh
cargo build --release --locked
python3 tools/performance/measure.py run \
  --jw target/release/jw --jj /absolute/path/to/real/jj \
  --files 100 --workers 2 --payload-files 100 --payload-bytes 4096
```

Use actual native executables: script/tool-manager shims are rejected, and
symlinks resolve to their native binary. For mise installations, obtain the real
JJ path with `mise which jj` in your normal environment. The installer uses the
running interpreter's resolved `sys.executable`, never `which python3` under the
isolated HOME. Logging wrappers execute the resolved binaries directly.

Each invocation prints its retained directory to stderr and `result.json` path
to stdout. Default output is ignored `.test-agent/performance/files-N-<unique>/`.
`--output-dir` selects another retained location on the **same filesystem as this
checkout**; a device mismatch fails. All donor/worker roots share that filesystem.
Use a filesystem supporting the CoW operation required by `jw`; unsupported CoW
is a failed experiment, not a speed result. No existing repository is reseeded.
Fixtures and failures are retained until deliberately removed by the operator.

Dimensions:

| Option | Default | Meaning |
| --- | ---: | --- |
| `--files` | 1000 | Regular tracked data files; try 100, 1000, and 8555 |
| `--symlinks` | 0 | Additional tracked relative symlinks pointing to data files |
| `--workers` | 2 | Workspace refill targets |
| `--payload-files` | 100 | Ignored payload files in groups of 100 |
| `--payload-bytes` | 4096 | Bytes per payload file, with deterministic contents |
| `--feature-files` | 0 | Data files modified in a common feature commit used by targets |
| `--timeout` | 180 | Per-command timeout in seconds; increase explicitly for slow baselines |

There is also one tracked `.gitignore` and one ignored preparation counter.
For the incident's 8512-regular-file/43-symlink shape, use `--files 8512
--symlinks 43` (plus the harness's `.gitignore`). This reproduces the shape, not
the original application's contents. Payload data is repetitive, so filesystem
compression and cache behavior differ from real dependency trees. Feature mode
tests a shared modified target tree; it does not simulate arbitrary merge conflicts.

The preparation command runs exactly once, generates the ignored payload, and
increments a counter. Its reported immutable source commit is passed unchanged
to both preview and actual reseed. The tool checks that preparation and preview
preserve target commit IDs, preview copies no payload, reseed never reruns the
installer, and every target has the expected tracked tree, data, symlinks, and
payload bytes/count. Verification commands are excluded from phase measurements.

To measure an older binary with the previous **source-only** `up` interface:

```sh
python3 tools/performance/measure.py run \
  --jw /absolute/path/to/preserved-before-jw --jj /absolute/path/to/real/jj \
  --prepare-command up --files 100 --timeout 1800
```

This explicit measurement adapter parses that old command's source ID; it does
not enable `up` in the production CLI or support arbitrary older workflows.

## Compare and gate process counts

Keep fixture options and environment identical when comparing before/after:

```sh
python3 tools/performance/measure.py compare BEFORE/result.json AFTER/result.json
python3 tools/performance/measure.py check AFTER/result.json \
  --max-file-show-calls total=0 --max-jj-calls reseed=500
```

Budgets above are examples to choose deliberately, not established guarantees.
`run` accepts the same repeated `--max-jj-calls`, `--max-file-show-calls`, and
`--max-git-calls PHASE=N` options. `PHASE` is `prepare`, `reseed-preview`, `reseed`,
or `total`. A correctness or budget failure makes `run` return 1 and retain its
failed result. `check` and `compare` reject failed or malformed results.

For a deterministic per-file subprocess regression check, run **the same binary**
with 100 and 8555 files, holding all other options fixed:

```sh
python3 tools/performance/measure.py compare SMALL/result.json LARGE/result.json \
  --scaling --max-jj-growth 0 --max-file-show-growth 0 --max-git-growth 0
```

Growth limits apply to each phase and the total; zero detects any additional
processes as tracked-file count grows. Adjust the accepted fixed growth only
with an explained reason. Normal comparison also accepts these growth limits.
No wall-time CI gate is supplied.

Comparison requires schema v1, successful correctness checks, identical harness
hash and recorded environment (JJ/Git/Python identities, host/platform, CPU count,
filesystem identity/options), and matching fixture configuration. `--scaling`
allows only increased regular-file count, and also requires the same `jw` hash
and preparation command. Ordinary before/after comparison permits a changed
`jw` binary and `up` → `prepare` measurement adapter. A mismatched comparison
returns 2 rather than presenting a misleading delta.

## Evidence and limits

`result.json` records schema version, binary path/hash/version, harness hash,
environment, fixture shape/path, exact source/target commit IDs, phase argv,
instrumented seconds, per-tool call counts and command histograms, verification,
and raw log references. Each `*.processes.jsonl` record preserves the tool, full
argument vector, cwd, PID, and timestamp. `commands.jsonl` references stdout and
stderr for setup, measured commands, and verification; `failure.txt` retains
exception details. Paths in log references are relative to the retained directory.

JJ histograms distinguish `file show`, `diff --git`, and other `diff` calls; Git
histograms expose `check-ignore` separately. PATH wrappers count nested JJ/Git
invocations too. Absolute-path child execution and Git's private exec-path
helpers can bypass wrappers; this is a count of observed tool invocations, not
a system-wide process profiler. Raw argv logs let a reviewer inspect grouping.

Timings are **instrumented warm-cache observations**, including a shell and
Python startup per observed invocation. That overhead can dominate small JJ/Git
calls and magnify a per-file subprocess bottleneck. Do not report these values
as raw production latency or infer a universal latency promise. Setup/cloning,
binary hashing, and correctness checks are excluded; the configured payload
installer is included in the preparation phase. Setup warms caches, preview
warms reseed, and this harness does not drop caches. Repeat fresh experiments
under comparable host load when interpreting elapsed deltas. Keep the same
interpreter and instrumentation on both sides.

External commands have bounded timeouts. On timeout the harness kills the POSIX
process group, including ordinary descendants; independently detached processes
cannot be guaranteed covered. A failed phase keeps its partial process log and
all prior evidence. There is no automatic pruning or reconciliation.

## Maintain

```sh
python3 -m unittest discover -s tools/performance -p 'test_*.py' -v
```

Tests cover argv boundaries, grouping, malformed/partial results, compatibility,
scaling restrictions, and deterministic budgets. After changing the harness,
rerun both sides: harness hashes intentionally prevent mixing methodologies.
Keep schema changes explicit and update the validator/tests together.

See [contributing](../../CONTRIBUTING.md#performance-assessment) for the development
entry point, [the incident handoff](../../design/up-performance/handoff0.glm53m.md)
for the per-file verification failure that motivates these checks, and
[source-only preparation](../../src/reseed/prepare.rs) for the measured contract.
