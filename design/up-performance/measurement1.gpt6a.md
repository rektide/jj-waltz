---
type: PerformanceAssessment
title: Fast prepare and reseed after removing repeated work
description: Delivered symlink-query batching, buffered journals, indexed obstruction checks, and inexpensive repository identity checks, with native end-to-end measurements.
resource: /design/up-performance/measurement1.gpt6a.md
status: verified
generated: { by: "model:openai/gpt-6-astra", at: 2026-10-17 }
sources:
  - resource: opencode://ses_eb2165cc8ffe4vOmOvDBe7aXQu
    title: Marked historical tooling commands and linked the Rust successor; measurements unchanged
  - resource: opencode://ses_eb8a7487affeE2smndZ914OQMS
    title: Integrated the delivered fixes, reviewed their boundaries, and recorded the resulting native timings
  - resource: opencode://ses_eb8801331ffeirQrF4GMFCxJHf
    title: Sol xhigh implemented buffered journals, indexed obstruction checks, and inexpensive identity checks; verified the fast path
  - resource: opencode://ses_eb89a214cffemn3L6w1Z9H78LR
    title: Identified JJ whole-tree copy detection and batched symlink capability proofs
  - resource: opencode://ses_eb87f7b4cffe23FlLmOW2DpE7o
    title: Sol medium delivered native timing mode and validated the maintained measurement tools
  - resource: /.test-agent/performance/files-8555-y57wtzlt/result.json
    title: Retained instrumented result after batching and journal buffering
  - resource: /.test-agent/performance/files-8555-ip_fz5u2/result.json
    title: Retained native result after repository identity optimization; all correctness checks passed
  - resource: /design/up-performance/measurement0.gpt6a.md
    title: Earlier constant-process-count but slow whole-operation baseline and 0.325-second raw-copy result
---

# Fast prepare and reseed after removing repeated work

Historical measurement/commands, not current Python tooling. The exact predecessor
is preserved at `a35da35d7635e14be70fb4aab37ea6edbd933ef8` (recover with
`git show COMMIT:tools/performance/measure.py`). Current [Rust controls](../../tools/performance/README.md)
are the supported workflow; instrumentation scopes are not interchangeable.

**Native end-to-end timings: prepare 3.17 s, preview 1.97 s, reseed two workers
6.64 s.** These are a synthetic fixture's measured times, not a promise for every
application's dependency tree or preparation commands.

## What changed

- `5f4ea0a9`: batch symlink capability proofs. JJ performed whole-tree copy
  detection before filtering paths: one symlink, all 43, and an empty fileset each
  cost roughly 0.7 s. Repeating the query per symlink repeated that whole-tree work.
- `70498975`: buffer journal serialization before the existing file-sync,
  atomic-replacement, and directory-sync sequence. Replace quadratic tracked-path
  obstruction searches with an indexed set.
- `08919e4a`: read native repository-pointer identity during repeated refill
  checks instead of rediscovering Git/JJ topology. Full topology and ownership
  checks remain at operation boundaries. Reuse symlink proofs for identical
  target commits and remove redundant tree enumeration.

The source-only `prepare`/worker-only `reseed` split and phase-aware recovery remain
as described in the [decision record](prepare0.gpt6a.md). Group reseeding uses the
same batch path. Feature rebasing remains a separate JJ operation.

## Measurements

Fixture: 8,555 regular data files, one tracked `.gitignore`, 43 symlinks, two
workers, and 100 ignored payload files of 4,096 bytes. Linux Btrfs, native JJ
`0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b`. Preparation executed once;
worker trees, symlinks, and copied payload passed correctness checks.

| Operation | Earlier instrumented | After batching/buffering, instrumented | Latest native |
| --- | ---: | ---: | ---: |
| Prepare | 43.88 s | 12.99 s | 3.17 s |
| Preview two workers | 68.65 s | 7.34 s | 1.97 s |
| Reseed two workers | 119.65 s | 18.22 s | 6.64 s |

Compare the two instrumented columns to assess that optimization stage. The
native column removes tracing overhead and includes the later identity
optimization; it is not an apples-to-apples speedup ratio against the first column.
No cold-cache or otherwise-idle-host claim is made. Raw reflink copying alone was
0.325 s; a complete refill still includes JJ sparse operations and validation.

The measured native binary SHA-256 is
`a393245bac388be85b66761e68cc9a7a0b354d4dc5163b305938374dc64011df`.
The final release was rebuilt after a lifetime-only Clippy correction; its hash is
`e22c05f18e5d3e501ecffc8944c51d881b0c6daf809f8789d0c5b73a35e8d542`.
The timing was not rerun for that non-behavioral correction.

Sol reported 56 focused engine/preparation tests passing with real CoW required,
plus Clippy and formatting. The tool suite passed 13 tests and native smoke checks.
An earlier broad run exposed one unrelated pre-existing deleted-checkout cleanup
failure on the installed JJ build; outside-repository discovery cases passed when
run outside this repository. This is not a new complete supported-JJ version matrix.

## Repeat

```sh
python3 tools/performance/measure.py run \
  --jw target/release/jw --jj /usr/local/bin/jj --timing-mode native \
  --files 8555 --symlinks 43 --workers 2 \
  --payload-files 100 --payload-bytes 4096 --timeout 1800
```

The [maintained tools](../../tools/performance/README.md) also provide instrumented
process counts, compatible-result comparisons, and the native reflink copying
baseline. Retained raw results above are ignored local evidence; this document
preserves the conditions, binary identities, findings, and measured values.
