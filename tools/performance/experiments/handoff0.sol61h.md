---
type: ImplementationAccount
title: JW matched-reseed adapter prototype — stopped for assessment
description: Shared-engine adapter, successful N1 scenario smoke, failed feature/N2 verification, preserved artifacts and an unexecuted integration recipe.
resource: /tools/performance/experiments/handoff0.sol61h.md
status: incomplete-stopped-by-user
generated: { by: "model:openai/gpt-6.1-sol#high", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb2165cc8ffe4vOmOvDBe7aXQu
    title: Marked retired commands historical and linked the supported Rust successor
  - resource: opencode://ses_eb224880fffeZxuFkeb0yDaCeN
    title: Built independent JW adapter, retained all smoke evidence and stopped expansion at human direction before repairing feature/N2 semantics
  - resource: /tools/performance/README.md
    title: Existing first-fill native and instrumented timing contracts
  - resource: /.test-agent/inline-policy/integration-handoff.md
    title: Preserved native baseline, actual nested payload policy and read-only live donor restriction
  - resource: file:///home/rektide/src/diff-dirs/perf/experiments/tooling0.sol61h.md
    title: Shared engine architecture, final checks, exact frozen library variants and limitations
---

# Stopped JW tooling prototype

Historical stopped prototype. Its active Python files and JSON recipe are retired;
use the [Rust guide](../README.md). Exact predecessor source is preserved at
`a35da35d7635e14be70fb4aab37ea6edbd933ef8` (`git show COMMIT:PATH`). Commands below
are historical transcripts/research recipes, not supported launchers.

The human stopped expansion for independent assessment. **Do not treat this as an
accepted base or validated feature-variant benchmark.** Production source/tests,
existing measure/fixture scripts and G's implementation paths were not edited.

`experiment.py` is a 20-line shared-engine launcher. Set `DIFF_DIRS_PERF` if
`../diff-dirs/perf` is not the engine location. `jw_sample.py` is a 189-line native
reseed adapter (209 new Python lines here; engine has 1,051 including tests).
`experiments/reseed.json` is the editable registration/job example.

Setup initializes ONLY a fresh private reset using native JJ/Git, isolated HOME
and policy `[node_modules, **/node_modules, **/dist]`. It overrides donor-local
freshen config: no fetch, install, live reseed or live dry-run. A CSV timer wraps
only the native `jw reseed` subprocess, including its startup. Fixture setup,
private reflinks and verification are outside that timer. Clean inode/mtime
changes are retained as diagnostics, not hidden or mixed with latency.

## Actual checks / failures / locations

The two-label baseline control uses the SAME preserved binary on both sides:
`/home/rektide/src/jj-waltz/.test-agent/inline-policy/jw-before`, SHA-256
`03e0ada406426d9496c602e22c9dd0de452f788e424dab9b7069c07f812e715c`, source
`7253ff6ddee639a98087e1729af58e0cbc546b6f`. Original `baseline.json` and binary
were not modified. Native JJ resolves from `/usr/local/bin/jj` to
`/mnt/bitblaster/archive/jj-vcs/jj/target/release/jj`.

Retained `.test-agent/matched-reseed-smoke/` contains:

- `fixture/`: two 16-byte ignored files plus tiny tracked data, no live content.
- `manifest.json`, `store/registry.json`: both labels import jw-before; immutable
  preserved copies and hashes are inside `store/`.
- `run/`: four successful unchanged/N1 native command and verification attempts.
- `matrix.json`, `matrix-run/`: **16/20 passed**, covering unchanged,
  initialization, sparse and subtree at N1 with two AB/BA blocks. All four
  N2/feature attempts reached verification but failed with
  `tracked working copy differs from JJ target: worker-0: M tracked.txt`.
  This is an **unresolved fixture/target semantic mismatch**, not a claimed
  production regression. No repair or further benchmark round followed the stop.

Actual invocations (after creating/editing the retained manifests):

```sh
python3 tools/performance/experiment.py fixture .test-agent/matched-reseed-smoke/fixture --files 2 --bytes 16
python3 tools/performance/experiment.py register .test-agent/matched-reseed-smoke/manifest.json .test-agent/matched-reseed-smoke/store
python3 tools/performance/experiment.py run .test-agent/matched-reseed-smoke/manifest.json .test-agent/matched-reseed-smoke/store/registry.json .test-agent/matched-reseed-smoke/run --blocks 2 --seed 17 --warmups 0
python3 tools/performance/experiment.py run .test-agent/matched-reseed-smoke/matrix.json .test-agent/matched-reseed-smoke/store/registry.json .test-agent/matched-reseed-smoke/matrix-run --blocks 2 --seed 17 --warmups 0
```

Shared-engine final checks: 12 unittest tests passed; see the linked engine
account. No new dependency, real large campaign or speedup claim.

## Historical retired before/after recipe — not executed here

Copy `experiments/reseed.json` into a NEW owned scratch directory. Replace the
candidate revision, worktree and artifact path with the actual integrated build;
keep the original release/compiler/profile and the baseline identity input.
Register the old binary separately from a candidate build:

```sh
python3 tools/performance/experiment.py register /path/to/manifest.json /path/to/new-store baseline
python3 tools/performance/experiment.py register /path/to/manifest.json /path/to/new-store candidate --build
python3 tools/performance/experiment.py run /path/to/manifest.json /path/to/new-store/registry.json /path/to/new-run --blocks 12 --seed 1701 --warmups 1
python3 tools/performance/experiment.py report /path/to/new-run/result.json
```

For an already separately built candidate omit `--build` but retain exact source
and compiler provenance. Build mode checks the declared source identity and
clean state; do not bypass this by calling an unrelated revision the candidate.

Create an owned realistic fixture with the engine's `fixture --donor ...
--tracked-json ...` command, using an explicit donor-relative tracked-path JSON
array and default nested policy. It copies private reflinks without `.jj/.git`;
the live donor remains read-only. No donor preparation command runs. Donor
mutation during copying and formerly forced-tracked ignored paths are not solved.

Duplicate the example job for `scenario: initialization|unchanged|sparse|subtree`.
Use separate IDs and `destinations: 1` or `2` (also pass `{destinations}` to
adapter setup). Each job starts from the matching scenario template for BOTH
binaries, not a cold first-fill baseline versus clean candidate. Setup and reset
are excluded; whole native reseed is the CSV sample. Keep native and instrumented
process-count controls separate. **N2 with distinct tracked features is unfinished;
repair/assess that mismatch before drawing conclusions or treating it as a gate.**

No full doc-pass/index update or framework reduction was undertaken after the
user's stop. All campaign scratch, including failures, remains for FX assessment.
