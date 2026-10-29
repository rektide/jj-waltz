---
type: Investigation
title: Renewal policies and representative OpenCode measurement
description: Explains the four-call replacement delta, distinguishes native route from payload comparison and inode publication, and grounds the next measurements in a read-only live workload census.
resource: /design/coordinated-refill/policy-scale0.gpt6a.md
status: investigation-and-proposed-measurement
generated: { by: "model:openai/gpt-6-astra#high", at: "2026-10-30" }
sources:
  - resource: "opencode://ses_e74f4db73ffebUUrvE3MbpCHWY"
    title: Human challenges tiny route evidence and metadata churn; session traces production policies, inventories OpenCode read-only and proposes representative comparisons
  - resource: /design/coordinated-refill/evidence0.sol61x.md
    title: Exact tiny fixture, full-route timings, native-call delta and effect counters
  - resource: /design/coordinated-refill/execution0.gpt6a.md
    title: Same-tree child protocol, full tracked renewal and native parent convergence
  - resource: /refill/engine/src/engine/command/native.rs
    title: Actual native route transitions, replacement reparent checks and commit-identity restore condition
  - resource: /refill/engine/src/engine/command/native/listed.rs
    title: Both tracked routes require full source installation and required reflinks
  - resource: /refill/engine/src/engine/command/binding.rs
    title: Repeated workspace-name and root verification; configured source assertions
  - resource: /transfer/workspace/src/streaming.rs
    title: Both native routes use the same source-qualified incremental ordinary cohort
  - resource: "file:///home/rektide/src/diff-dirs/tree/inspect/src/comparison.rs"
    title: StatKey regular-file size and mtime comparison and literal symlink comparison
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/kernel.rs"
    title: Fresh staging files, required clone preparation and rename publication
  - resource: /tools/performance/README.md
    title: Current tiny retained-fixture adapter and older imported-workload adapter have different capabilities
  - resource: /tools/performance/src/snapshot.rs
    title: Existing read-only membership and private payload import mechanism
  - resource: /design/up-performance/primary0.sol61m.md
    title: Earlier actual-shape census, completed historical clean observations and cancelled campaign overhead
  - resource: /.test-agent/policy-scale/census.json
    title: Current recorded membership and live metadata census; candidate roots are not frozen ignore admission
  - resource: "file:///home/rektide/a/a/opencode/.jwreseed.local.toml"
    title: Actual source assertion, payload patterns and configured worker group
---

# What the current result establishes

The final [route evidence](evidence0.sol61x.md) used seven tracked regulars
totaling **104 bytes**, one tracked symlink, and a largest aggregate fixture bound
of **84 entries / 1,018 bytes** including staging reserve. Listed installation
was 0–1 rounded ms. Its replacement slowdown measures native-command-dominated
controls; it cannot choose the faster policy for the operator's OpenCode workload.

## Three independent policy choices

| Axis | Current behavior | Question worth testing |
| --- | --- | --- |
| Native tracked route | ClearThenPopulate clears tracked paths through JJ; ReplaceAuthorized preserves containers and replaces owned leaves | Is avoided clearing worth the extra native transition on the actual tracked tree? |
| Ordinary payload work | Both routes stat-compare admitted node_modules/dist; regular equality is size + mtime, links compare literal targets | When does unconditional renewal beat incremental comparison, if ever? |
| Leaf publication | Both routes prepare a fresh private regular inode, reflink into it, preserve metadata and rename into place | Would inode-preserving clone-overwrite materially reduce churn under a deliberately different contract? |

**Tracked renewal already means clone every source leaf even when equal.**
`TrackedPhase` uses `CreateAbsent` after clear, or `RenewOwned` against saved O;
both install the complete source manifest with `ReflinkRequired`. There is no
tracked equality shortcut to remove by asking for a simpler reflink policy.

Replacement avoids a separate whole-tree clear and keeps compatible directories.
It still creates/retires file inodes. Its rename removes the old directory entry
as it publishes the new one, so it is not zero metadata churn. Per-directory
staging also has a cost. Provider removal counters do not count all native or
rename-induced churn; an all-zero removal report is not proof of no deletions.

For ordinary data, unconditional renewal would avoid comparison work but still
need discovery, authority/device/kind checks and attributes for installation.
The current guard itself requests both sides' metadata. Skipping the equality
predicate alone therefore does not automatically eliminate the target stat pass.
Clean incremental payload keeps file identities and issues no clone effects.

An inode-preserving FICLONE path is a separate design possibility, not a synonym
for replacement. It must settle destination ownership, hardlinks, exact final
length/metadata, open-file visibility and failure behavior. Current fresh-private
publication guarantees cannot simply be attributed to that path. Prototype it on
owned data if selected; measure it rather than assuming ioctl count implies speed.

## Why four additional JJ calls

Current replacement protocol:

```text
preserve old O -> new child C of O (same tree, no wholesale checkout)
              -> renew from S -> adopt S state -> optional restore D
              -> rebase only C onto D -> final verification
```

Starting directly with `new D` would let JJ materialize D before the reflink
renewal. The current alternative uses the old tree to avoid that checkout, then
pays to establish the desired parent afterward. The reparent phase verifies C's
identity/parent and absence of outside descendants before the rebase.

Observed net inventory delta **per receiver**, from the actual trace table:

| Command | Replacement minus clear-first |
| --- | ---: |
| log | +2 |
| workspace list | +1 |
| workspace root | +1 |
| rebase | +1 |
| sparse set --clear | -1 |
| **Total** | **+4** |

These are net counts, not just four extra checks. Other operations replace each
other; in particular the operation-log totals match. N1 totals are **86 vs 90**;
N2 totals **131 vs 139**. No extra JJ process is launched per payload file.

The common discovery/verification budget deserves investigation too. For example,
`BoundWorkspace::verify_at` queries workspace names and registered root separately,
and is called across multiple barriers. Look for a single operation-bound query
that returns the required facts and reuse within that operation; do not assume
facts from before native mutation remain current. The four-call difference is
an implementation result, not an intrinsic filesystem cost of replacement.

Another discriminating native case is **different S/D commits with equal trees**.
The current restore branch tests commit inequality. Prepared S can be an empty
child of upstream D, making this relevant even without content differences.
Measure this separately from truly different trees before selecting an optimization.

## Current OpenCode scale

Read-only census at `2026-10-30T04:24:32Z`, recorded commit
`0ec385c0446ceb547f4e1a3154f65b65769c2c7b`:

| Scope | Regular files | Symlinks | Directories | Logical regular bytes |
| --- | ---: | ---: | ---: | ---: |
| Recorded tracked membership, live metadata | 8,892 | 43 | not inventoried | 143,066,283 |
| 70 outermost named payload candidates | 201,221 | 10,383 | 26,263 | 11,613,540,197 |

The payload census selects outermost real directories named node_modules/dist,
excludes .git/.jj, and never follows symlinks. These are **candidate roots**, not
the production engine's frozen-ignore admission result. Live sizes are not a
quiescent snapshot, content verification or exclusive physical disk usage. The
recorded membership has 8,935 paths; none was missing during this census.
185,910 payload regular paths have link count greater than one in the donor;
that says nothing about existing receivers' private-inode status.

The actual local configuration selects node_modules, nested node_modules and
nested dist, with `source_at = "dev@origin"`. Its normal configured use therefore
can include a live source assertion even when the CLI omits `--source-at`.
The benchmark should match the effective configuration rather than describe all
plain reseeds as assumed-prepared. Retain exact source/desired identities.

## Existing representative evidence and its limits

The [older imported campaign](../up-performance/primary0.sol61m.md#cancelled-campaign-retained-observations-not-completed-matrix)
recorded two clean N2 incremental samples at **21.15 / 17.64 seconds**, paired
with an older rewrite-all baseline at **108.31 / 99.11 seconds**. All 171,567
common regular payload inodes survived per incremental receiver; the old baseline
rewrote them. These historical implementations differ from today's route pair.
The measurements support retaining incremental-clean as an important control,
not a current speedup claim or proof that comparator cost is always negligible.

That campaign was cancelled because repeated full fixture copies and full-byte
verification dominated useful work. Do not repeat that construction pattern.
The current tiny adapter successfully reuses roots, but hardcodes eight-leaf
coverage/effect expectations and a 256-entry / 1 MiB ceiling. It needs a real
manifest-driven large-workload mode, not merely a larger numeric limit. The older
importer supplies useful machinery, not an immediately compatible route campaign.

## Proposed next measurement

1. **Reuse one isolated, full-shape donor and two receivers.** Import the actual
   tracked tree and all qualifying payload roots once, with private required
   reflinks and literal symlinks. Record one full input manifest and native graph
   provenance. Reset only declared differences between samples; account setup
   and verification separately. Retain independent output checks, including exact
   native state and private-inode/write-isolation controls. Avoid full multi-tree
   content hashing on every clean sample; perform full content acceptance once
   and targeted checks plus membership/metadata/effect checks for subsequent
   declared perturbations, with that narrower verification scope explicit.
2. **Measure existing routes first.** Full-scale clean and small distinct receiver
   deltas, N1 and N2, same release binary, effective source-assertion profile and
   serial setting. Use a fixed balanced schedule (four matched pairs per chosen
   case), retain every result and report dispersion. Include equal-tree/different-
   commit S/D and a real feature-tree difference as named cases. Missing payload
   provides a first-fill control; creation itself is a separate command workload.
3. **Attribute cost.** Whole-command wall/CPU/RSS plus existing native/install/
   ordinary phase spans; separate instrumented probes for JJ calls, metadata
   lookups, clones, rename/unlink/mkdir/rmdir and actual inode churn. Trace counts
   from JW and its native children together. Instrumented wall is not native
   latency. Report entry counts and directories as well as logical bytes: reflink
   work is strongly affected by file/extent shape, not just payload GiB.
4. **Then test ordinary policy and publication independently.** If commissioned,
   compare incremental against unconditional staged renewal on clean, sparse and
   broadly changed payloads, keeping native route fixed. An inode-preserving
   clone prototype is a further separate alternative with explicit semantics.
   This reveals whether time goes into decisions, filesystem effects or native
   orchestration without multiplying all policy dimensions at once.
5. **Check native repository representativeness.** Importing files into a fresh JJ
   repository loses the real history and workspace registry size. Record that
   limitation and use a controlled actual-history/registry specimen or selected
   disposable actual-repository receivers for a subsequent native-cost check.
   File-shape results alone cannot establish live pool command latency.

This session performed source investigation and the read-only census only.
Current full-scale route latency, memory peak and a crossover remain unmeasured.
The priority is a usable full-scale retained-fixture comparison, followed by
optimization where those measurements locate the cost.
