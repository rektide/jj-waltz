---
type: PerformanceEvidence
title: Four actual OpenCode receivers — clear-first versus replacement
description: Full-scale live N4 clean and small-change route measurements, inode churn, native process counts, final preservation checks and the dominant clean payload traversal cost.
resource: /design/coordinated-refill/live-four0.gpt6a.md
status: measured-live-n4
generated: { by: "model:openai/gpt-6-astra#high", at: "2026-10-30" }
sources:
  - resource: "opencode://ses_e74f4db73ffebUUrvE3MbpCHWY"
    title: Human authorizes four live receivers and subsequent N1/N2 scaling; session measures both routes, churn, process counts and replacement scaling with preserved source and feature histories
  - resource: "opencode://ses_e74de64f4ffeWvWy0dGBaxfHlv"
    title: F discovers older managed receivers with covered work and records the limits of read-only cleanliness spot checks
  - resource: /design/coordinated-refill/policy-scale0.gpt6a.md
    title: Policy distinctions, initial OpenCode census and proposed measurement; execution now uses four authorized actual receivers
  - resource: /design/coordinated-refill/evidence0.sol61x.md
    title: Prior tiny route evidence and provenance of the reused release binary
  - resource: /.test-agent/policy-scale/live-four/README.md
    title: Local runner and metadata-inventory scope
  - resource: /.test-agent/policy-scale/live-four/manifest.json
    title: Exact executable, source and desired pins, initial receiver identities and native coordinates
  - resource: /.test-agent/policy-scale/live-four/summary.json
    title: All successful raw command outcomes, resources, phases, effects and separate count probes
  - resource: /.test-agent/policy-scale/live-four/final-verification.json
    title: Independent final recorded emptiness, conflict, parent, full-sparse, source and feature-bookmark checks
  - resource: /.test-agent/policy-scale/live-four/native-resolution.trace
    title: Post-campaign read-only exec trace identifies mise and rustup overhead before the same native JJ executable
  - resource: /.test-agent/policy-scale/live-four/scaling/summary.json
    title: Subsequent N1/N2 replacement commands, phase times, resources and source work beside the original clean N4 controls
verification:
  method: Actual public reseed commands, checked schema-one completion and coverage, metadata/inode inventories, sparse-byte restoration checks, separate native-call wrappers and final snapshot-free JJ queries
  scope: Four authorized live Linux Btrfs workspaces, default serial release build, warm sequential D=S repetitions; no fresh-donor performance extrapolation
---

# Result

**Replacement and clear-first were effectively tied on clean N4**, with paired
replacement deltas **+1.30% / -2.01%**. Replacement avoided recreating **3,656
tracked ancestor directories per command**. Both routes still replaced every
tracked regular inode. The dominant cost was **89–95 seconds in ordinary payload
traversal even when every ordinary effect was zero**.

All four receivers finish as full-sparse, conflict-free, empty children of pinned
source S. Their original feature bookmarks remain at their original tips.

## Actual workload and execution

Receivers, in stable order: `watchignore`, `cache-ttl`, `evented`, `cacheuse`.
Their canonical roots are `/home/rektide/src/opencode-<name>`. The user explicitly
authorized using these existing workspaces after F's discovery. There was no
new donor import, receiver copy or full-tree reset between samples.

Source: `/home/rektide/archive/anomalyco/opencode` (`~/a/a/opencode`).
S=D: `0ec385c0446ceb547f4e1a3154f65b65769c2c7b`.
Live source assertion: exact `161d4669663d1506b57b6e0950e90db91b04e456`, the
captured `dev@origin` commit, whose tree equals S. The configured payload patterns
remain `node_modules`, `**/node_modules`, `**/dist`. No prepare, fetch, build,
`--replace-content` or preservation bypass was used.

Each receiver after convergence contains the surveyed source shape:

| Scope | Regular files | Symlinks | Directories | Logical regular bytes |
| --- | ---: | ---: | ---: | ---: |
| Tracked | 8,892 | 43 | 960 ancestors | 143,066,283 |
| 70 outermost payload candidates | 201,221 | 10,383 | 26,263 | 11,613,540,197 |

Inventories cover these named roots and tracked membership, not arbitrary unrelated
local data. At the baseline, all **210,113 regular paths per receiver** differed
in inode from their donor counterpart, had link count one, and matched donor
size/mtime. This is a metadata/private-inode observation, not a full byte or
extent-sharing proof. Source inventoried metadata stayed unchanged through the
clean and sparse comparisons.

The candidate is the already accepted **JW 0.11 / provider 0.7**, default serial
release executable from [evidence0](evidence0.sol61x.md), SHA-256
`6718c01adc965e3b2823c11e3d8af19d7a858538242d2a790eed8cd5c5edb6d2`.
It includes the provider cache-charge fix. The original build sidecar is embedded
in the manifest. No production code or binary changed for this run.

### Initialization and retained unsuccessful attempt

Initial clear-first convergence succeeded in **291.263 seconds**, including
**243.251 seconds ordinary**. Ordinary regular clones per receiver were
40,135 / 40,135 / 40,338 / 12,678. This heterogeneous catch-up is recorded
separately, not paired against replacement or included in clean timing.

The initial local report parser used the wrong finish-field spelling; the actual
command exited zero and its raw report was retained. Correcting the parser to
`finish.state == complete` validated that same result without rerunning reseed.

The first clean attempt (`clean-0`) then stopped at the history guard in 12.587 s:
S is an empty donor working commit without its own covering bookmark. No receiver
native establishment began. The operator-authorized benchmark explicitly created
`reseed-benchmark-source-20261030` at exact S, retaining the normal guard. New
sample labels preserve the refused attempt. The four original feature bookmarks
were not moved. This also exposes a repeat-reseed usability issue in the current
work-tip coverage rule; it was not patched during measurement.

## Native latency samples

Fixed order for each shape: **C, R, R, C**, two samples per route. C means
clear-then-populate; R means replace-authorized. All eight samples exited zero,
returned complete reports, installed the full 8,935-leaf manifest per receiver,
and completed ordinary and finish phases. All source regular installation was
required reflink, with zero byte-copy fallback.

The small-change shape edits one different small ignored regular per receiver,
flipping one byte and changing mtime while keeping length. Original bytes are
retained in `sparse-inputs.json`; each application checks private destination
ownership and donor bytes. Each reseed restored all four files byte-for-byte.
Each lane reports exactly one ordinary clone, totaling 5,109 bytes per command.

| Sample | Route | Whole command seconds | Pre-native seconds | Ordinary seconds | Native receiver spans, seconds | wait4 max RSS KiB |
| --- | --- | ---: | ---: | ---: | --- | ---: |
| clean-1 | C | 132.889 | 14.382 | 89.094 | 7.011 / 6.200 / 6.366 / 7.046 | 75,204 |
| clean-2 | R | 134.614 | 14.489 | 90.357 | 7.244 / 6.628 / 6.549 / 6.558 | 75,188 |
| clean-3 | R | 137.573 | 15.066 | 90.657 | 7.716 / 6.809 / 6.884 / 7.716 | 75,052 |
| clean-4 | C | 140.395 | 15.231 | 94.589 | 7.298 / 6.504 / 6.402 / 7.794 | 75,272 |
| sparse-0 | C | 137.707 | 15.127 | 91.150 | 7.099 / 6.570 / 8.162 / 6.847 | 75,240 |
| sparse-1 | R | 141.606 | 17.709 | 91.013 | 8.886 / 7.920 / 6.788 / 6.512 | 75,004 |
| sparse-2 | R | 135.545 | 15.065 | 89.189 | 7.895 / 6.351 / 6.609 / 7.738 | 75,020 |
| sparse-3 | C | 204.130 | 33.093 | 106.962 | 25.219 / 19.944 / 8.004 / 8.069 | 75,240 |

The last C sample is a large slowdown across multiple phases, including common
pre-native work. Its cause is unmeasured; keep it, and do not claim the sparse
midpoint (-15.38%) as a reliable replacement improvement. The first sparse pair
was +2.83% for R, the second -33.60%. Clean's descriptive midpoint is -0.36%.
With two pairs on a busy live filesystem, these are observations, not confidence
intervals or a demonstrated universal winner.

These are actual PATH-environment uninstrumented command latencies. Inventories,
scenario edits and validation are outside each timer. Metadata inventories warm
caches. One read-only baseline-inventory analysis ran concurrently with the early
clean campaign; host activity was not isolated. Btrfs reported 99% utilization
before execution. Neither that fact nor the outlier proves a specific bottleneck.
The returned wait4 maxima are child-resource observations (~73 MiB), not summed
process-tree peaks, provider allocator measurements or evidence of a memory
improvement over an earlier release.

## Measured churn and source work

Every clean command, per receiver:

- **8,892 tracked regular inodes and 43 tracked link inodes changed**, both routes.
- Clear-first changed **911** tracked-directory inodes in watchignore and **915**
  in each other receiver. Replacement changed **zero** such directory inodes.
- Replacement still changed metadata of **890** tracked ancestor directories per
  receiver. Keeping the inode is not zero metadata work.
- Every ordinary effect was zero. Inventories show **no payload metadata/inode
  changes** across all clean samples, including all 201,221 regulars per receiver.
- Across the sparse block, each receiver's only payload inode change is its one
  deliberately edited file, with metadata changes in its containing directory.

Native source diagnostics are identical between routes: **39,580 declared
metadata acquisitions and 172 link acquisitions per command**, 3,812 cache
evictions. That is four times the source's 9,895 leaf/ancestor entries and 43
links: the bounded 32-batch cache does not retain the entire tracked source
through serial receiver transitions.

Ordinary diagnostics are also identical across the clean/sparse native samples:
paired **27,236 listings / 253,863 metadata acquisitions**, opaque **2,707 /
21,617**, with 29,911 cache evictions. These are source acquisition counters,
not all filesystem syscalls, target stats or an attribution of 90 seconds to
one specific comparator. CPU profiling is needed to locate that traversal cost.

## Separate native-call probes and launcher discovery

One additional complete clean probe per route used the preserved Rust JJ/Git
exec wrappers. Count-probe wall times are not mixed into the native sample table.

| Command | C | R |
| --- | ---: | ---: |
| All observed JJ | 275 | 291 |
| log | 44 | 52 |
| workspace list | 45 | 49 |
| workspace root | 80 | 84 |
| rebase | 0 | 4 |
| sparse set --clear | 4 | 0 |
| file show | 17 | 17 |
| All observed Git | 6 | 6 |

The original **+4 JJ calls per receiver** remains exactly true at N4. There is
substantial common overhead too: 45 workspace lists and 80 workspace-root calls
even on clear-first. Count wrappers observe PATH-launched native commands, not
every subprocess inside launchers.

**Important environment distinction:** ordinary PATH resolves `jj` through
`~/.local/share/mise/shims/jj`. A post-campaign read-only `jj --version` exec trace
shows four rustup subprocesses before it execs `/usr/local/bin/jj`. The wrappers
instead exec that native JJ directly. Both report the same
`0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b` profile. This explains why probe
timings are not a clean instrumentation-only comparison: they change launcher
behavior too. No precise shim-time saving is claimed from the probe wall times.
Resolving the native executable once is an additional concrete optimization lead.

## Replacement scaling follow-up: one and two receivers

At the human's request, four additional clean replacement commands ran in fixed
**N1, N2, N2, N1** order. N1 selects watchignore; N2 adds cache-ttl. They reuse
the exact candidate, S/D/assertion, normal mise-containing PATH and existing
clean receivers. PATH equality with the prior campaign is checked before running.
There are no new fixtures, payload changes, builds, count wrappers or source
preparation. All four commands completed with full tracked coverage, zero ordinary
effects and finished receivers. No concurrent parent inventory analysis ran during
this follow-up.

| Receivers | Whole-command seconds, both runs | Ordinary payload seconds, both runs | Pre-native seconds, both runs |
| --- | --- | --- | --- |
| 1 | 68.107 / 48.149 | 35.147 / 24.531 | 11.804 / 13.039 |
| 2 | 161.411 / 87.313 | 66.307 / 49.776 | 12.038 / 14.023 |
| 4, earlier controls | 134.614 / 137.573 | 90.357 / 90.657 | 14.489 / 15.066 |

N2's first run includes **66.665 seconds in cache-ttl's reparent phase**, versus
0.937 seconds there on the second N2 run. That phase contains checks and native
rebase; the exact cause of the spike is not established. N1's first tracked
installation also took 16.891 seconds versus 5.069 on its second run. Keep both
runs rather than deriving a latency curve by discarding or subtracting slow phases.
N4 controls preceded this follow-up, not an interleaved N1/N2/N4 experiment.

The source-work counters give a clearer scaling observation than noisy wall time:

| Receivers | Ordinary source listings, paired + opaque | Ordinary source metadata acquisitions | Native source metadata acquisitions |
| --- | ---: | ---: | ---: |
| 1 | 28,018 | 259,222 | 9,895 |
| 2 | 28,663 | 268,652 | 19,790 |
| 4 | 29,943 | 275,480 | 39,580 |

Both repetitions at each size have identical counters. Ordinary source metadata
acquisition grows only **6.3% from N1 to N4**, and source link acquisitions remain
10,383: source sharing is real. Native source metadata acquisition grows exactly
linearly with receiver count under the serial transition / bounded-cache behavior.
Ordinary elapsed time still grows substantially with destinations; target-side
observation and per-lane processing remain. These counts do not identify which
part of that processing dominates CPU or I/O. The data support profiling it,
not an assertion that the source is independently walked four complete times.

## Landing and next decision

All four final receivers are independently verified as empty, conflict-free,
full-sparse children of exact S. Source recorded S is unchanged. The original
covering bookmarks still identify their original feature tips. The benchmark
source bookmark remains as explicit preservation. Receivers now contain the
donor baseline; feature histories were preserved, not reapplied.

**Prefer replacement when avoiding directory teardown is the priority:** this
campaign shows that physical benefit without a consistent clean latency penalty.
The largest next performance target is the ~90-second clean ordinary traversal;
profile that before adding an unconditional payload rewrite. Inode-preserving
reflink publication remains a separate design, since both current routes churn
all tracked file inodes. Batch/reuse native queries and avoid repeated shim setup
alongside that work. No default or production implementation changed here.
