---
type: MeasurementReport
title: Integrated streaming reseed native controls
description: Immutable serial/P4 application builds and balanced synthetic correctness controls; primary opencode import remains a separate required campaign.
resource: /design/up-performance/integrated-controls0.sol61m.md
status: synthetic-controls-complete-primary-workload-pending
generated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb2158009ffeyJ0q8NuTd6g1RP"
    title: Implemented experimental shared payload execution and ran sequential native Rust controls from preserved release binaries
  - resource: /design/up-performance/streaming0.sol61m.md
    title: Implementation, review corrections, exact cleanup and correctness verification
  - resource: /tools/performance/README.md
    title: Accepted native adapter, balanced scenarios and candidate-only clean invariant
  - resource: /design/up-performance/diffdirs-review0.gpt6a.md
    title: Approved immutable production library revision after independent reassessment
---

# Integrated native controls

## Immutable inputs

All paths below are relative to this checkout. Builds use release profile with
Rust 1.101.0-nightly `822182db4` (2026-10-13), LLVM 23.1.3, Cargo
1.101.0-nightly `29c5daa1a`. Candidate source is
`0fca0c6c589e5b92f06d4d33369ae40dd0fa6490`; no uncommitted runtime source was
included. The user's unrelated handoff was not committed or edited.

| Input | Preserved executable | SHA-256 |
| --- | --- | --- |
| Before, source `7253ff6ddee639a98087e1729af58e0cbc546b6f` | `.test-agent/inline-policy/jw-before` | `03e0ada406426d9496c602e22c9dd0de452f788e424dab9b7069c07f812e715c` |
| Candidate serial, no features | `.test-agent/integrated-reseed/build-serial/jw` | `6ee5b43abde77cb63f1912e62b91cafc07c35236d0a723129c48974fa35d14e8` |
| Candidate capped-P4, `payload-parallel` | `.test-agent/integrated-reseed/build-p4/jw` | `8470f8a15b2cb435f2ee4d6ce82a0ce8c9250fd76491ad085d2a42167a637145` |

Each candidate directory retains `build.json`, Cargo manifest and lock; binaries
are mode 0555. Manifest hash is
`b77ddbf898f089a64556a073cb642b7a48e6419b77a07e0330dc0d8d46635f40`, lock hash
`385544d7c2fe926ef03815f307296181edaa0024b1e5206c15e81773cebe574d`.
Library approval is `8811c0561e90c84b29e22faa8b26c9a5cc6f76a1`, runtime
`243be8e2a28a796c61652161c9dd30c6b35fc5d0`. No library sources were edited.

The native Rust adapter/controller executable is `target/release/jw-perf`, SHA
`b6bf38c05f4730a9d688a4bf4d652289e845349976f95270e6344fbb66720f8d` at campaign
start. Every campaign preserves its adapter and JW executables, hashes, source
identities and exact argv in `campaign.json`. Native JJ resolves to
`/mnt/bitblaster/archive/jj-vcs/jj/target/release/jj`, SHA
`0b7462db466ffb0394cb30b0bb56480da3bce10a91f98b80b109302b0ec610d8`.

## Scope and reproduction

These are synthetic controls, **not the user's primary opencode workload**.
Each sample has 100 generated tracked regular files plus the fixture `.gitignore`,
four tracked links, 1,000 payload files
of 4,096 bytes plus a relative link, and two edited feature files per target.
Each baseline/candidate pair gets a NEW equivalent private fixture. Scenarios
are first fill (`init`), clean repeat, sparse delta and subtree transition; each
runs at N1/N2 for serial and capped-P4 builds. Two mirrored blocks per campaign,
seed 1701: two samples per executable, with both order positions represented.

```sh
PATH=/usr/local/bin:$PATH target/release/jw-perf run \
  .test-agent/integrated-reseed/control-SCENARIO-nN-MODE \
  --baseline .test-agent/inline-policy/jw-before \
  --baseline-provenance .test-agent/inline-policy/baseline.json \
  --candidate .test-agent/integrated-reseed/build-MODE/jw \
  --candidate-provenance .test-agent/integrated-reseed/build-MODE/build.json \
  --scenario SCENARIO --destinations N --feature-files 2 --files 100 \
  --symlinks 4 --payload-files 1000 --payload-bytes 4096 \
  --blocks 2 --seed 1701 --require-clean-label candidate
```

Campaigns run sequentially, with no competing agent campaign/build/test started
by this session. The first serial/N2 clean campaign preceded the loop over the
remaining 15. Raw `campaign.json`, `samples.jsonl`, `summary.json`, command logs,
private cases and immutable binaries remain under each named output.

Native timing includes process startup and full `jw reseed`, not merely library
API time. Setup/reset/hash/verification costs are retained but excluded. Private
setup and hashing warm caches; no cache drops or cold-cache claim. Background
host I/O remains uncontrolled and recorded pressure is descriptive only. This
small bounded run cannot establish significance or an exact checker/pool tax.
Serial/P4 use separate matched-baseline campaigns, not direct interleaved pool
comparison; do not rank them causally from this table.

## Completed results

**16/16 campaigns and 64/64 native samples passed**, with zero sample errors,
zero failed final fixture checks, and one unchanged controller hash across all
campaigns. Four scenarios × N1/N2 × two build variants, each two balanced pairs.
Tracked final goal/parent, old feature bookmarks/content, payload bytes/modes/file
mtimes/literal links, independent roots and frozen source payload checks passed.

Every clean candidate sample passed its hard regular-file inode/mtime invariant:
eight samples, twelve target observations, 12,000 common-file observations, zero
inode/mtime changes, additions or removals. The baseline changed all 1,000 regular
payload inodes per target and no mtimes; its historical full-copy behavior is
reported, not rejected. No syscall-level write trace was collected in these native
controls. The separate application regression additionally checks clean payload
directory/link/file inode, mtime and ctime preservation.

Descriptive native full-command medians, milliseconds; paired Δ and its two-pair
range are percentages (negative means candidate took less time):

| Scenario | Targets | Build | Before ms | Candidate ms | Paired Δ % | Pair range % |
| --- | ---: | --- | ---: | ---: | ---: | --- |
| init | 1 | serial | 1442.88 | 1281.60 | -11.15 | -12.05 … -10.24 |
| clean | 1 | serial | 1593.26 | 1504.98 | -6.01 | -9.48 … -2.54 |
| sparse | 1 | serial | 2083.11 | 1627.77 | -21.87 | -23.15 … -20.58 |
| subtree | 1 | serial | 1956.07 | 1475.30 | -23.72 | -34.75 … -12.69 |
| init | 2 | serial | 2395.36 | 2829.37 | **+18.04** | -8.09 … +44.17 |
| clean | 2 | serial | 3502.06 | 2218.23 | -33.68 | -47.55 … -19.81 |
| sparse | 2 | serial | 2778.38 | 2420.04 | -13.10 | -19.34 … -6.86 |
| subtree | 2 | serial | 2575.84 | 2753.72 | **+6.87** | -14.08 … +27.81 |
| init | 1 | capped-P4 | 1564.23 | 1433.66 | -7.59 | -23.84 … +8.67 |
| clean | 1 | capped-P4 | 2280.06 | 1475.31 | -27.10 | -53.27 … -0.93 |
| sparse | 1 | capped-P4 | 1902.02 | 1309.60 | -30.22 | -37.70 … -22.73 |
| subtree | 1 | capped-P4 | 1753.56 | 1408.98 | -19.71 | -20.95 … -18.47 |
| init | 2 | capped-P4 | 3011.23 | 2395.01 | -20.12 | -25.64 … -14.61 |
| clean | 2 | capped-P4 | 2905.21 | 2640.59 | -9.04 | -13.95 … -4.14 |
| sparse | 2 | capped-P4 | 3045.02 | 2513.69 | -17.45 | -18.46 … -16.45 |
| subtree | 2 | capped-P4 | 2905.91 | 2324.25 | -19.00 | -24.73 … -13.27 |

Two serial/N2 medians regress and several pair ranges cross zero. All attempts
remain retained; there were no repeat-until-fast rounds. These small controls do
not select a default or establish main-workload speedup. Capped-P4 identifies the
build's `min(available_parallelism, 4)` policy; per-sample worker count was not
separately logged.

Exact unrounded pairs and sample chronology are in each campaign's existing JSON.
The reporting-only `.test-agent/integrated-reseed/control-aggregate.json` collects
the sixteen summaries and clean identity observations. The raw campaign/sample/
summary hashes plus aggregate hash are bound by
`.test-agent/integrated-reseed/control-evidence.sha256`, SHA-256
`35ea264f205dcd6bcccc818720d75ce79fb7410705773363943738ebc23422d0`.
No new benchmark harness was written to aggregate these existing outputs.

The loop ran as shell job `sh_14e316a71001BBzUJOBqvPG9YI`, supervisor PID
`4136353`; the last explicitly observed child was runner PID `16118`. Completion
was received from the harness, and both recorded PIDs were absent afterward.
No campaign process from this session remains running. No instrumented count
campaign or new live-source campaign was started during this control completion.

## Outstanding primary-workload evidence

The accepted JW adapter initially supports synthetic repositories only. The
library adapter can import a single payload directory, but not a frozen JJ
tracked tree plus multiple outermost roots and feature histories. This concrete
missing read-only importer was reported to the parent/tool owner. No `jw reseed`,
dry-run, snapshot, fetch or installer was run against live opencode or its branches.
Do not substitute these controls for the required imported main-workload campaign.
Serial remains default; P4 remains an experimental Cargo feature.
