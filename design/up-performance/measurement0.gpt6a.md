---
type: PerformanceAssessment
title: Bounded subprocess verification still leaves tree-size latency
description: First checked-in-harness measurements after the prepare split and bounded tracked-tree verification; constant process counts but substantial remaining large-tree work.
resource: /design/up-performance/measurement0.gpt6a.md
status: investigated
generated: { by: "model:openai/gpt-6-astra", at: 2026-10-17 }
sources:
  - resource: opencode://ses_eb8a7487affeE2smndZ914OQMS
    title: Ran the maintained harness at 100 and 8555 regular files, compared process growth, and commissioned investigation of the remaining latency
  - resource: /tools/performance/README.md
    title: Measurement method, instrumentation costs, fixture dimensions, and compatibility checks
  - resource: /design/up-performance/prepare0.gpt6a.md
    title: Source-only preparation and shared-engine performance direction
  - resource: /.test-agent/performance/files-100-ccks0sng/result.json
    title: Retained small-fixture result; ignored local evidence, not a portable archive
  - resource: /.test-agent/performance/files-8555-h9l8h81r/result.json
    title: Retained large-fixture result and sibling argv logs; ignored local evidence, not a portable archive
  - resource: /.test-agent/performance/files-8555-h9l8h81r/copy-baseline-gobytzkf.json
    title: Strict native reflink copy of all 8700 fixture files and links completed in 0.325 seconds
---

# Bounded subprocess verification still leaves tree-size latency

The first optimization removes subprocess growth with regular tracked-file count.
It does **not** yet establish a fast large-tree refill. The maintained harness
exposed substantial remaining latency, including in dry-run.

## Conditions

- Release `jw 0.8.0`, SHA-256
  `f1b83bec0e91387e1da2318efeecc725448d0f08851255be42325330f7758051`.
  Production code includes the prepare split and verification optimization from
  `bed02439`, the shim-safe process test from `25710456`, and recovery guidance
  from `9bf436c5`.
- JJ `0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`, Git 2.55.0,
  Python 3.14.6, Linux Btrfs.
- 100 versus 8,555 regular data files; both also have one tracked `.gitignore`,
  43 tracked symlinks, two workers, and 100 ignored payload files of 4,096 bytes.
  No feature delta in these two runs.
- Timings include the harness's shell/Python process instrumentation. They are
  not uninstrumented production latency. Other session verification work occurred
  during this assessment, so wall-clock comparisons also include possible host
  contention. Process counts are the stable regression measure.

## Results

All fixture correctness checks passed: prepare ran once, prepare/preview kept
worker revisions, preview copied no payload, reseed did not prepare again, and
both workers' tracked trees, symlinks, and payload bytes/counts matched.

| Operation | JJ calls, both sizes | `file show`, both sizes | Git calls, both sizes | 100 files, instrumented s | 8,555 files, instrumented s |
| --- | ---: | ---: | ---: | ---: | ---: |
| Prepare | 299 | 3 | 187 | 11.193 | 43.881 |
| Reseed preview, two workers | 212 | 3 | 110 | 7.482 | 68.648 |
| Reseed, two workers | 428 | 13 | 246 | 17.071 | 119.646 |
| Total | 939 | 19 | 543 | 35.746 | 232.176 |

The zero-growth comparison passed for every phase and the total: 8,455 additional
regular files introduced **zero additional JJ or Git invocations**. The remaining
`file show` queries read ignore rules, not each tracked data file.

## Native copying baseline

The maintained `tools/performance/copy_baseline.py` ran GNU
`cp -a --reflink=always` against the retained large donor. Copying all **8,700
files and symlinks completed in 0.325 seconds**, with successful strict reflinks,
the expected output count, and a distinct inode for the checked tracked file.
This timing has no JJ/Git instrumentation. It excludes `.jj`, target deletion,
feature application, and verification. The fixture and filesystem were warm.

This establishes a fast copying lower bound on the same filesystem. It does not
measure every part of `jw`'s own copy implementation, but it rules out accepting
minutes as the intrinsic cost of reflinking this fixture.

## Reproduce

```sh
cargo build --locked --release --bin jw
python3 tools/performance/measure.py run \
  --jw target/release/jw --jj /usr/local/bin/jj \
  --files 100 --symlinks 43 --timeout 1800
python3 tools/performance/measure.py run \
  --jw target/release/jw --jj /usr/local/bin/jj \
  --files 8555 --symlinks 43 --timeout 1800
python3 tools/performance/measure.py compare SMALL/result.json LARGE/result.json \
  --scaling --max-jj-growth 0 --max-file-show-growth 0 --max-git-growth 0
```

Use the retained paths printed by each run. The comparison verifies fixture,
environment, harness, and binary compatibility. `/usr/local/bin/jj` is this
machine's native-binary symlink; select the actual JJ executable elsewhere.

## Next investigation

The fixed 43 symlinks still cause 43 root-to-commit Git-diff extraction queries
during preparation and 86 in each two-target preview/refill. Their individual
cost may grow with the complete tree. Repeated traversal and linear membership
searches are other candidates. These are hypotheses, not established attribution;
native query timing and code inspection should select the next optimization.

The [incident handoff](handoff0.glm53m.md) supplies the earlier hours-scale
failure. The [command-split decision](prepare0.gpt6a.md) explains why the core
engine is being improved independently of `up` ergonomics. The
[performance harness](../../tools/performance/README.md) is the maintained tool
for repeating and extending this assessment.
