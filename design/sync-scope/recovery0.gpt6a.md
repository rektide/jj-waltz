---
type: DesignBrief
title: "Recovering the synchronization-boundary questions"
description: Human-directed follow-up from the cancelled actual-workload campaign: revisit copy_ignored, reusable synchronization scope, JJ ownership, and proportional measurement before further implementation.
resource: /design/sync-scope/recovery0.gpt6a.md
status: commissioned-design
generated: { by: "model:openai/gpt-6-astra", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb262a81effea1HgF0pPS3f1sy"
    title: Recovered the child-session discussion after the human returned, separated implementation/evidence/design questions, and commissioned the blocked Astra xhigh follow-up
  - resource: "opencode://ses_eb2158009ffeyJ0q8NuTd6g1RP"
    title: Human cancellation and cleanup direction, zoom-out, ignore-rule questions, synchronization-boundary challenge, and Astra xhigh design request at 16:44–19:31 UTC
  - resource: /design/up-performance/primary0.sol61m.md
    title: Actual imported census, five completed samples, verification/copy overhead, cancellation and condensed-evidence retention
  - resource: /design/up-performance/streaming0.sol61m.md
    title: Implemented streaming integration, correctness evidence, and changed contracts
  - resource: /design/up-performance/integrated-controls0.sol61m.md
    title: Completed synthetic controls and frozen executable identities
  - resource: /design/up-performance/inline-policy0.gpt6a.md
    title: Earlier caller integration design; useful history, not a constraint against reconsidering the organizing concept
  - resource: /src/reseed/engine.rs
    title: Separate JJ orchestration, tracked prefills, ignored-root selection and checked mirrors
  - resource: /src/cow/ignored.rs
    title: Selection patterns and outermost eligible payload-root discovery
  - resource: /src/cow/payload.rs
    title: Frozen Git-ignore inputs, persistent native oracle and separate tracked membership
  - resource: "file:///home/rektide/src/diff-dirs/src/check.rs"
    title: Current static accept/refuse interface, without preserve/exclude decisions
  - resource: "file:///home/rektide/src/diff-dirs/sync/src/apply.rs"
    title: Current recursive copy/remove kernel and staged transitions
  - resource: "file:///home/rektide/src/diff-dirs/design/performance/reassessment0.sol61x.md"
    title: Bounded warm-clean library results, distinct from full JW results
---

# Recovering the synchronization-boundary questions

## Present direction

The human continued directly in the integration child session after the parent's
last handoff. That discussion is new direction to carry forward, not simply an
agent departure to undo. The human now asks the parent to recover and follow up
the questions and exploration.

The latest concrete commission in that discussion was **an independent Astra
xhigh improved design**. The child tried to delegate but nested delegation was
blocked. This brief restores that commission at the parent.

**Design only.** The actual-workload benchmark was cancelled. Do not restart it,
recreate its fixtures, run new campaigns, change production code, or initiate
filesystem maintenance as part of this follow-up. Existing code and compact
evidence remain available for reasoning. Preserve the user's unrelated uncommitted
`design/up-performance/handoff0.glm53m.md`.

## Human questions to answer

### 1. Is the synchronization boundary backwards?

At 19:26 UTC the human asked whether JW must replicate synchronization work, or
whether diffdirs can be a generally usable system accepting JW's requirements.
They explicitly questioned whether `copy_ignored` is necessary, and whether the
extra policy should instead describe what is **not** copied.

Representative exact phrases:

> "i'd like to try to push to diffdirs a general usable system"
>
> "i'm not sure that we really need the copy_ignored stuff?"
>
> "it's the ignores the things we dont do that are extra work right?"

The design must genuinely reconsider the organizing concept, not merely rename
`copy_ignored` or defend the existing implementation because it passed tests.
Equally, a question is not acceptance of a new default that overwrites every
untracked destination path. Compare plausible authority/selection models and
make their consequences concrete.

### 2. What does JJ actually do, and why does JW still copy?

The user's mental model was roughly source → destination through JJ. The current
code has several distinct actions:

1. JJ supplies pinned tracked-tree/revision facts and owns working-copy state.
2. JW clears/refills tracked state, performs its own equal-path reflink prefill,
   and lets JJ materialize the required tracked differences.
3. JW separately discovers selected ignored directories and checks eligibility.
4. diff-dirs-sync reconciles those selected filesystem roots.

Explain which boundaries are necessary for JJ correctness (including a desired
tree different from the donor), which are optimizations, and which are historical
duplication. Consider whether tracked prefills can use generic primitives, whether
they should disappear, or whether a different source/authority interface makes
the whole path simpler. Do not silently collapse revision ownership into copying.

### 3. What are the three kinds of "ignore"?

The 18:21 UTC discussion recovered this current implementation:

- `copy_ignored` patterns select candidate roots. They are not Git ignore rules.
- Frozen `.gitignore`, backend `info/exclude`, and effective global excludes are
  interpreted by a persistent native Git `check-ignore` oracle in JW.
- Tracked membership is checked separately; `--no-index` Git ignore answers do
  not establish whether JJ tracks the path.

Frozen rule contexts and memoized oracle answers remove repeated extraction and
process work, but do not settle whether Git-ignore eligibility should define
sync authority. Dependency payload and private local data can both be ignored;
the current opt-in roots also carry permission to overwrite and delete.

New design should distinguish:

1. repository tracking/snapshot policy;
2. source authority and allowed destination mutations;
3. excluded/preserved filesystem regions.

Choose useful terminology and interfaces without introducing an interpreted rule
engine. The human prefers coded implementations, static specialization or
callbacks, with very little hot-loop work.

### 4. How general can diffdirs become?

Current `Check` is compiled accept/refuse. It already covers clean/equal entries
and recursive one-sided copy/removal. It does **not** express preservation or
exclusion. There remains JW-owned copying for tracked prefills.

Examine a reusable scope mechanism with explicit behavior for:

- paired directories and equal entries;
- source-only and target-only subtrees;
- protected target-only entries and their necessary ancestor directories;
- file/directory transitions containing preserved descendants;
- absent roots, symlink leaves and root replacement;
- copy/update versus deletion-authoritative mirror;
- partial progress, refusal, excluded coverage and convergence reports.

Scope must govern the recursive applier as well as the comparison. Hiding a source
entry must not accidentally authorize deleting its target counterpart. Keep Git/JJ
meaning in an adapter unless a concrete alternative earns that coupling.

No full-tree operation map should be assumed. The previous useful implementation
is directory-at-a-time streaming; bigger metadata catalogs/runtimes require an
actual argument. Upfront mount topology, no synthetic clone probe, operation-time
required reflink failure, and static checks remain strong human performance
preferences, not excuses for redefining destination ownership silently.

## Measurement and operations questions

The human asked for progress visibility at 16:44 UTC, then cancelled the run and
requested bulky-fixture cleanup while preserving condensed results. This cancels
the parent's earlier requirement to finish the primary matrix.

The failure was substantive, not merely bad luck with host load:

- The actual dataset was 8,557 tracked paths plus 68 selected roots, with 171,567
  payload regular files, 9,561 links and about 11.36 GB of logical regular bytes.
- A two-target sample made roughly seven payload-tree copies, repeatedly hashed
  large trees, and retained generated cases. Samples spent 336–460 seconds in
  untimed setup and 91–176 seconds in verification in the five retained attempts.
- Native reseed itself was much smaller: the two serial candidate samples took
  17.64 and 21.15 seconds, against paired baseline 99.11 and 108.31 seconds. One
  unpaired P4 sample took 8.81 seconds. These are partial warm-clean observations,
  not a completed matrix or proof of a P4 default.
- At cancellation the filesystem was 99% full and a copy waited on Btrfs metadata
  reservation. Benchmark retention contributed burden; total disk usage cannot
  honestly be attributed to this session alone.

The parent commissioned and accepted the fixture-heavy experiment and shares
responsibility. Retained raw results do not justify repeating this operational
shape. A future measurement design needs proportional verification, bounded
fixture residency/reuse, observable phases/progress, and a deliberately bounded
initial run. Preserve compact artifacts, failures and provenance without retaining
every full data tree. This is a follow-up design requirement, not authorization
to implement another harness immediately.

The Btrfs filtered-balance versus scrub question was answered as operational
advice in the child. It did not authorize this parent to run maintenance commands.

## Current facts and standing

- Production streaming integration and the checked diffdirs kernel are committed;
  serial remains default and `payload-parallel` remains experimental.
- Earlier focused correctness, synthetic controls, and library clean reassessment
  remain valid within their documented scope. The known unrelated missing-workspace
  cleanup test failure remains separately recorded.
- The primary campaign is **cancelled**: five of its reduced twenty samples
  completed. P4 has no completed pair; actual mutable cases and final instrumented
  scaling were not completed.
- Cleanup was human-authorized. Compact records and canonical executables were
  retained; generated case/import trees and selected build caches were removed.
  Do not treat their old paths in historic commands as available fixtures.
- At parent recovery, JW's only working-copy change is the pre-existing human
  handoff; diffdirs is clean. Later JW commits record plan/census/cancellation,
  not a new runtime rewrite. No matching benchmark process was found in the
  parent's targeted process check.

## Astra xhigh design return

Produce one independent model-suffixed design, grounded in the current code and
the human questions above. Include:

1. a recommended reusable diffdirs/JW boundary, with a small concrete API sketch;
2. credible alternatives, particularly opt-in roots versus broader sync with
   exclusions, and separate copy/update versus mirror authority;
3. a worked example of an opencode workspace with tracked feature changes,
   dependency/build artifacts, administrative state, and locally retained data;
4. how the design reduces actual walks, metadata work, subprocesses and duplicate
   filesystem implementations without hiding required work in a preflight;
5. exact semantic decisions needing human selection and a coherent landing order;
6. a minimal future verification/measurement method that cannot recreate the
   fixture explosion described above.

Read source and retained evidence. Preserve useful existing work while questioning
its architecture. No production edits, benchmarks, large scans, fixture creation,
cleanup or further delegated agents in this design commission.
