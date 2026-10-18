---
type: DesignProposal
title: "Transfer a workspace by authority, not by ignoredness"
description: A reusable scoped filesystem reconciler in diffdirs, with JJ retaining revision authority, broad donor updates with exclusions, explicit deletion regions, and shared tracked-prefill primitives.
resource: /design/sync-scope/design0.gpt6a.md
status: proposed-for-human-selection
generated: { by: "model:openai/gpt-6-astra#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eaf6d43a9ffeoStzj7zyWNVCWh"
    title: Independent Astra xhigh architectural follow-up to the human's synchronization-boundary questions; reviewed targeted source and retained evidence, designed authority/exclusion semantics and bounded verification
  - resource: /design/sync-scope/recovery0.gpt6a.md
    revision: 6931c142
    title: Complete recovered human brief, including the reopened design direction and cancelled benchmark
  - resource: /README.md
    title: Current donor-current reseed default, explicit --at override, creation behavior, copying policy and feature layering
  - resource: /docs/reseed.md
    title: Maintained selected-root, ignoredness, replacement, tracked-verification and recovery contracts
  - resource: /src/reseed/engine.rs
    revision: 0fca0c6c
    title: Reviewed source/desired/old trees, tracked prefills, source rechecks, selective cleanup and checked payload mirrors
  - resource: /src/reseed/payload.rs
    title: Current static guard, retained-region walks and borrowed execution pool
  - resource: /src/reseed/topology.rs
    title: Existing command-local mountinfo front guard and its alias/autofs limitations
  - resource: /src/cow.rs
    title: Remaining JW-owned file-copy, recursive-copy and removal kernels
  - resource: /src/cow/ignored.rs
    title: Source-only eligible-root discovery and current absent-match behavior
  - resource: /src/cow/payload.rs
    title: Frozen native ignore oracle, tracked membership and confined-parent helpers
  - resource: /src/workspace.rs
    title: Creation's tracked-prefill and JJ working-copy-state adoption, distinct from reseed
  - resource: /tests/reseed.rs
    title: Existing donor-sync, private-payload, tracked-type/mode and retained-obstruction precedents, read without execution
  - resource: /design/up-performance/inline-policy0.gpt6a.md
    title: Earlier scope partition and inline-guard proposal; useful antecedent with a narrower organizing model
  - resource: /design/up-performance/streaming0.sol61m.md
    title: Delivered integration, runtime coordinates and the reported verification envelope
  - resource: /design/up-performance/primary0.sol61m.md
    title: Actual workload census, five completed samples, cancellation, setup/verification costs and compact retention
  - resource: "file:///home/rektide/src/diff-dirs/README.md"
    title: Native comparison, opaque subtrees, scope of Check, frontier semantics and prospective source-to-many direction
  - resource: "file:///home/rektide/src/diff-dirs/sync/README.md"
    title: Native sync ownership, required reflinks, staged transitions, metadata semantics and partial-progress reports
  - resource: "file:///home/rektide/src/diff-dirs/src/check.rs"
    title: Actual statically dispatched accept/refuse observation interface
  - resource: "file:///home/rektide/src/diff-dirs/src/pair.rs"
    title: Immediate listings, metadata inspection, name merging and opaque one-sided classification
  - resource: "file:///home/rektide/src/diff-dirs/src/walk.rs"
    title: Directory-local traversal and committed summaries
  - resource: "file:///home/rektide/src/diff-dirs/src/frontier.rs"
    title: Speculative child admission and divergence seals that exclusions must precede
  - resource: "file:///home/rektide/src/diff-dirs/sync/src/lib.rs"
    title: Root binding and checked compare/apply orchestration
  - resource: "file:///home/rektide/src/diff-dirs/sync/src/apply.rs"
    revision: 243be8e2
    title: Shared recursive kernels, descriptor-based cloning, target staging and deletion behavior
  - resource: "file:///home/rektide/src/diff-dirs/sync/src/paths.rs"
    title: Structural root and public-record validation
  - resource: "file:///home/rektide/src/diff-dirs/sync/tests/checked.rs"
    title: Recursive refusal, staged-root, partial directory-removal and clean-repeat precedents, read without execution
  - resource: "file:///home/rektide/src/diff-dirs/design/fit/policy0.gpt6a.md"
    title: Earlier pair-wide preservation argument and explicitly deferred preserve-capable recursive application
  - resource: "file:///home/rektide/src/diff-dirs/design/performance/reassessment0.sol61x.md"
    title: Separate 64-sample warm-clean library evidence and substantial untimed verification cost
---

# Transfer a workspace by authority, not by ignoredness

## Recommendation

**Make diffdirs own scoped filesystem reconciliation, including the recursive
copy/removal work and the listed-leaf copying JW still implements. Make JW own
which revision is wanted and which filesystem regions the donor may change.**

For the reopened JW design, I recommend **broad donor copy/update with a small
preserve list, plus explicitly named mirror regions for disposable data**.
`node_modules` and `dist` are natural mirror regions: stale destination entries
must disappear there. Outside those regions, copy donor entries and update
same-name entries, but retain destination-only data. Git ignoredness does not
grant either permission.

This genuinely retires `copy_ignored` as the organizing concept. The new model is:

> Choose a JJ revision; transfer the donor's remaining filesystem contents;
> preserve named local regions; delete extras only where the donor is explicitly
> authoritative for absence.

It is a proposed behavioral change, especially for same-name local files. It
needs the human selections in [Decisions](#decisions-for-the-human), rather than
being inferred from the question. The [recovered brief](recovery0.gpt6a.md#human-questions-to-answer)
is an active human design reopening. Existing useful implementation and evidence
are inputs to this proposal, not a reason to constrain the answer to their shape.

### Three independent questions

| Question | Owner | What it answers |
| --- | --- | --- |
| Is this path recorded, or eligible for a future snapshot? | JJ and repository ignore/snapshot configuration | Repository meaning. An ignored file can still be recorded. An unrecorded file need not be ignored. |
| May donor presence overwrite this path? May donor absence remove it? | The caller's transfer policy | Mutation authority. Copy/update and mirror differ on absence. |
| Must this name and its subtree be left alone? | The caller's preserve scope | Filesystem exclusion on **both** sides, including target-only names. |

The current selected ignored roots answer all three at once. That is the
architectural problem: the useful intent “refresh these dependencies” is encoded
as “discover source directories that match patterns and are ignored in two
repository contexts, then grant wholesale mirror authority inside them.”

## What JJ contributes, and what it does not

The native [reseed guide](../../docs/reseed.md#one-off-use), current
`engine.rs::run_inner/execute`, and
`plain_sync_replaces_feature_tree_types_modes_and_literal_paths` agree:

* Normal reseed snapshots the old work and creates a fresh empty child of the
  **donor's current commit**. It does not preserve the feature tree in place.
* `--at` can select a different desired tree. `--source-at` asserts the donor
  tree; it does not select the destination tree.
* Feature duplication/rebasing afterward belongs to JJ.

Let **S** be the pinned donor tree, **D** the desired tracked tree, and **O** the
old target tree. The filesystem donor is not the entire desired workspace when
`D != S`. For example, a tracked file deleted in D must not reappear because it
still exists on the donor's disk. Mode-only tracked differences matter even
though diffdirs deliberately does not compare mode-only changes.

Today JW computes tracked paths equal between S and D, empties the sparse
selection, removes residual recorded leaves, creates the new JJ child, copies
those equal paths itself, and lets `jj sparse reset` materialize D. JJ supplies
revision correctness; JW supplies donor-to-target reflinks. Plain filesystem
mirroring does not establish JJ working-copy state or revision identity.

**The tracked prefill is an optimization with an explicit product purpose:
renewing CoW sharing.** Replacing it with ordinary incremental `update` would
not renew equal target files: stat equality says nothing about shared extents.
Replacing it with ordinary JJ checkout would establish D, but would give up the
current donor-sharing guarantee. Neither is an equivalent simplification.

Creation has a related but different adapter: `workspace.rs::materialize_clone`
copies tracked files, adopts source working-copy state, then uses JJ restore.
That state adoption remains JW code. A generic copier must never copy `.jj` to
simulate it.

## Ownership and control flow

### Native responsibilities

| Owner | Responsibilities after this change |
| --- | --- |
| **JJ** | Snapshots, commits, bookmarks, sparse state, tracked-tree comparison/materialization and working-copy bookkeeping. |
| **JW adapter** | Resolve S/D/O; choose workspaces; capture identities/topology; define donor authority, tracked exclusions, local preservation and link-owned paths; orchestrate JJ and filesystem phases; report recovery coordinates. |
| **diff-dirs core** | Sorted directory-local inspection, stat/literal-link comparison, pair-wide scope exclusion, checked observations, deterministic traversal and coverage. No Git/JJ parser. |
| **diff-dirs-sync** | Copy/update, explicitly deletion-authoritative mirror, retained ancestors, staged leaf/type transitions, required reflinks, listed-leaf installation and recursive removal. |
| **JW links** | Deliberate shared-link installation after copying. Those names are excluded from donor transfer. |

Scope decisions are compiled Rust implementations or closures over frozen caller
facts. Config lists can be compiled once to prefix/glob matchers in JW. No
interpreted action-rule vector, per-entry process, persistent filesystem catalog,
or source-provider runtime is required in either library.

### One command, two necessary filesystem operations

1. **Capture the semantic inputs.** Resolve donor/targets, snapshot and pin S/D/O,
   establish existing ownership/bookmark/obstruction conditions, and capture
   mount topology once. Build indexes from the native tracked lists already
   required; do not enumerate a second filesystem tree to build a work map.
2. **Establish D and renew tracked sharing.** Keep the verified sparse-prefill/JJ
   protocol. JW supplies the S/D-equal tracked leaf list to a generic
   create-only leaf installer in diff-dirs-sync. JJ supplies all other tracked
   contents. Old tracked leaves that become auxiliary data are cleared by this
   phase, then become eligible for the next one.
3. **Transfer auxiliary contents from the workspace root.** Run one scoped
   compare/apply traversal. Skip the leaves/subtrees owned by S or D, native
   administrative regions, configured link names and explicit preserved regions.
   Update everything else, with deletion enabled only in declared mirror regions.
4. **Apply links and verify the result through JJ.** Check the desired tracked
   tree, full sparse state and exact parent relationship. Successful filesystem
   reconciliation establishes its own scoped result without a second payload
   walk. Complete peers remain complete if another target fails.

There are two sources of truth, not two copy implementations. D supplies the
tracked answer; donor disk supplies the auxiliary answer. Stating that explicitly
is smaller than introducing a virtual union filesystem that pretends both are a
single physical source.

### Tracked boundaries need ancestor handling

After JJ materialization, exclude every leaf in `tracked(S) ∪ tracked(D)` and any
subtree at that exact relative name. Excluding only D would copy back donor
tracked paths intentionally absent in D. Excluding only S would expose
destination-only feature paths to deletion.

**Do not exclude their ancestor directories wholesale.** A `packages/cli`
directory can contain both tracked source and unrecorded `dist`. Descend its
paired directories normally. If a donor auxiliary leaf conflicts with an
ancestor required to contain D's tracked files, report that shape conflict;
never replace the tracked container. These ancestor constraints come from the
tracked index and use observed types, not another walk or JJ invocation.

Explicit preservation is a command-wide promise. An existing preserved region
that tracked clearing/materialization would change is an obstruction to the
requested operation, checked before clearing. A preserve entry is not a way to
silently select a different tracked tree. The existing retained-secret and
retained-directory [tests](../../tests/reseed.rs) are the precedent.

## A small generic API, with separate absence authority

Illustrative signatures, not a proposed second framework:

```rust,ignore
// diff-dirs: relative to the bound LOGICAL roots, never a staging directory.
enum Visit { Reconcile, PreserveTree }

trait Scope: Sync {
    fn at(&self, relative: &Path) -> Visit;
}

// diff-dirs-sync: called only for real target entries with no source counterpart.
// Keeping a directory name does not imply keeping all of its children.
trait Absence: Sync {
    fn remove(&self, relative: &Path) -> bool;
}

// No exclusions / no deletions / all deletions have no-op or constant
// specializations, like today's NoCheck. Check retains accept/refuse semantics.
let report = Transfer::bind(source, target, ReflinkRequired)?
    .scope(&workspace_scope)
    .check(&entry_guard)
    .absence(&disposable_regions)
    .run()?;

// Ordinary conveniences select constant absence policies explicitly:
update(source, target, copy_policy, scope, check)?; // NeverRemove
mirror(source, target, copy_policy, scope, check)?; // RemoveAll

// No comparison or destination-only removal. Every named leaf is installed
// into an absent destination; real directories are created only as needed.
seed_leaves(source, target, equal_tracked_paths, ReflinkRequired, check)?;
```

If JW retains its distinct `--replace-content` clearing operation, expose an
explicit root-bound `clear(target, scope, check)` over the same recursive remover.
Its input is a deletion request, not a missing donor or a forged diff record;
the bound container and excluded administrative/local names remain. This lets
JW express that existing command without maintaining another removal kernel.
Whether explicit local preserves also survive this deliberately destructive
mode must be stated in its revised contract; recommend that they do, with a
separate policy change needed to relinquish them.

`Scope` is a symmetric path-space decision: a preserved directory ends descent,
and descendants cannot be re-included. The policy must be stable for the command.
`Absence` is independent: a custom implementation can return true under
`**/node_modules` and `**/dist`, false elsewhere. Both are statically dispatched;
the library does not know what those names mean. A caller needing only one whole
mirror uses the simpler `mirror` entry point.

Constant `NeverRemove` can retain target-only subtrees without descending them.
A custom mixed policy must inspect target-only directories if descendants might
have removal authority. In particular, a vanished `packages/old` can still
contain a configured `packages/old/dist` mirror. Skipping the parent would miss
that request. Prefix pruning is valid only when the caller can establish that
the entire branch is retained; arbitrary `**` selection sometimes requires the
walk. This is an honest cost of broad, destination-aware scope.

`seed_leaves` is the missing reusable primitive, not a fake diff. It consumes
JW's existing path list, uses the current staged copy/link and metadata kernel,
groups work by parent where useful, checks real ancestors, and never scans
unlisted donor directories. It must distinguish missing source, existing target
and unsupported type rather than silently deciding JJ fallback. JW chooses
whether a creation failure triggers its existing ordinary-checkout fallback;
reseed continues requiring actual reflinks. Known relative leaves are explicit
input, not a whole-tree filesystem plan invented by the library.

## Scope must reach every operation

The current [`Check`](file:///home/rektide/src/diff-dirs/src/check.rs) is useful
and stays: it accepts or refuses observed entries. Returning success cannot mean
“preserve”; returning an error cannot mean “excluded but successful.”

### Name selection before observation and descent

In `pair.rs`, merge the sorted **union of names** before deciding which names
need metadata. Ask `Scope` once for the logical relative name, applying its
answer to both sides. `PreserveTree` means no stat/readlink/child traversal for
that entry after its name is known. Root/ancestor binding still performs its
necessary structural checks. The same name-stage is used by recursive copy,
recursive deletion and explicit read-only inspection.

This makes a fixed `.jj` or private-data exclusion cheap. A source entry removed
from consideration is never converted to source absence. Permission failures
inside a preserved tree are irrelevant because that tree is not opened.

Run the existing observed-entry `Check` on all participating present entries,
including equal leaves. Unsupported specials still fail there. Scope/check
decisions run before `frontier.rs` exposes any speculative child jobs. Retain
the existing pure/deterministic/`Sync` callback contract and joined returns.

### Recursive outcomes

| Situation | Required behavior |
| --- | --- |
| Paired included directories | Always descend. Directory mtime/size never proves equality. |
| Paired included regular leaves | Existing type/size/exact-mtime comparison; stage changes. Mode-only repair remains outside this contract. |
| Symlink leaves | Compare/copy literal link text; never follow it. |
| Preserved name, either or both sides present | Skip both sides, including copying, deletion and descendant inspection. |
| Source-only included directory | Copy recursively with scope applied at every name; excluded descendants stay absent. Included empty directories are still copied. |
| Target-only included leaf | Remove only if `Absence::remove` permits that exact name. Otherwise retain it. |
| Target-only included directory | Reconcile descendants under the same policies; remove the directory only if its name is deletion-authorized **and** no retained descendant remains. |
| Target-only directory containing a preserved child | Keep that child and the necessary real ancestor directories. Removable siblings can disappear. |
| Absent target root | Initialize the scoped contents under the existing binding/confinement contract. No synthetic clone probe. |
| Missing source root | Error. Never interpret a missing donor as an empty tree authorizing deletion. |
| Existing target file/link root | Require the existing explicit replacing-root binding. The workspace-root adapter itself never authorizes replacing its root. |

Recursive removal returns an outcome such as `Removed` or `Retained`, plus
completed work. It cannot call unconditional `remove_dir_all` on caller data
after filtering a few immediate entries. Retention propagates upward, restoring
any temporarily relaxed surviving directory modes. An already-satisfied run
with retained containers must do zero writes, including avoiding gratuitous
permission relaxation just to discover that everything will be retained.

Scope operates on **logical final paths even during private staging**. Passing
the physical temporary pathname to a prefix matcher would make exclusions
disappear during leaf-to-directory replacement. Carry logical relative paths
separately from physical source/output locations in the existing kernel.
Owned staging cleanup remains independent of caller preservation policy.

### Type transitions: resolve the actual conflict

* **Leaf → directory:** construct the scoped source directory privately first;
  then replace the permitted target leaf. A copied subtree can omit preserved
  descendants that are absent on the target. A preparation/refusal/reflink
  failure leaves the old leaf in place.
* **Directory → leaf:** replacing the name would erase all descendants. Any
  preserved descendant, or any descendant retained by absence policy, blocks
  that transition. `Update` therefore cannot replace a nonempty target directory
  with a leaf; it can replace an empty directory. A mirror can replace the
  nonempty directory only if every existing descendant is removable.

**Recommendation for this one destructive shape case:** before changing that
directory's contents, perform a local read-only removability check of the
affected destination subtree. If blocked, return the conflicting path and leave
that subtree intact. If permitted, prepare the source leaf, then use the shared
recursive remover and install it. No full-tree plan is retained. Successful
directory-to-leaf replacement pays another traversal of that affected subtree;
ordinary paired directories, leaf updates and target-only pruning do not.

This is a deliberate improvement over today's checked-transition precedent,
which allows earlier descendant removals before a deep refusal. It is not a
whole-command transaction: previous records may already have changed, and I/O
failure during an authorized removal remains partial progress. The cheaper
alternative is streaming deletion until a retained descendant blocks replacement;
it preserves the descendant but can empty its siblings before failing. That
tradeoff should be selected explicitly, not emerge accidentally from `rmdir`.

### Public records and reports

Public `DirDiff` remains stat-free. For a scoped caller, an opaque removed-dir
record means **source-absent subtree to reconcile under this binding's policy**,
not proof that the entire directory will disappear. The applier must enforce its
own bound scope/absence policy on public records, including all their ancestors;
a fabricated record naming a child below a preserved parent must not bypass it.
Unscoped callers retain the existing all-entries meaning directly.

Keep three results distinct:

1. **Observed comparison:** the pre-application delta in visited space. Opaque
   records do not count every recursively encountered child.
2. **Policy result:** exhausted/satisfied, failed or stopped, including retained
   extras, excluded boundaries and retained structural containers.
3. **Completed work:** actual installed/reflinked/removed entries and restoration
   failures, including partial work on failure.

An intentionally retained container can appear again in a raw comparison while
requiring zero further operations. Report `policy_satisfied`, not “full roots
equal.” Existing full-coverage cleanliness must not turn true merely because
scope excluded something; give scoped completion an explicit meaning. Boundary
counts and bounded diagnostic examples are sufficient by default; do not create
a retained per-file policy catalog just to print a report.

Dry-run uses the same scope/check/absence decisions and covers participating
one-sided interiors. Excluded subtrees are reported as excluded, not audited.
Ordinary execution does not run that whole audit first. Native errors retain
path/cause and completed progress; retry is a fresh comparison, with existing
out-of-band JJ recovery if the tracked phase was interrupted.

## Worked opencode workspace

Assume a prepared opencode donor S and a target with feature work O. The target
has changed `packages/cli/src/feature.ts`, deleted a formerly tracked source file,
and changed a tracked executable bit. Both workspaces have dependencies/build
output. The target also has `.env`, `.test-agent/notes`, and a local experiment
database under `packages/cli/dist/local-db`.

An illustrative **new**, explicitly selected policy is:

```toml
[transfer]
base = "update"
preserve = [".env", ".test-agent", "packages/cli/dist/local-db"]
mirror = ["**/node_modules", "**/dist"]
```

This is intent syntax, not a request to implement these field names now. The
mirror patterns grant deletion authority whether a matching region is present
on the donor or only on the destination. Preserve wins inside a mirror.

| Path / event | Authority and outcome |
| --- | --- |
| Tracked feature file, deletion and executable bit | D controls them through JJ. Normal reseed chooses D=S and leaves O in history; `--at <feature-tip>` instead chooses that feature tree. |
| Donor tracked file omitted from feature D | Excluded from auxiliary transfer by the S/D tracked union; it cannot be resurrected by broad copying. |
| `node_modules` and nested package dependencies | Donor presence updates; donor absence removes extras. Unchanged files keep inode/mtime. Newly installed files are private reflinks. |
| `packages/cli/dist/local-db` | Explicitly preserved even if donor has another database, has none, or removes its whole `dist` directory. Necessary `packages/cli/dist` ancestors survive. |
| Other stale files under that `dist` | Removed under mirror authority; preserving the database does not preserve unrelated stale siblings. |
| Target-only `scratch-result.txt` outside mirrors | Retained by update policy. If the donor also has that name, donor content wins unless the path is explicitly preserved. |
| Donor-only unrecorded `notes.txt` outside mirrors | Copied under broad update, irrespective of Git ignoredness. |
| Root `.jj`, permitted donor `.git` | Owned by repository tooling; never copied, diffed recursively or deleted by auxiliary transfer. Root identity remains. |
| JW's local control files and configured link-source paths | Excluded as native caller-owned names; links are applied by their owner afterward. The policy does not treat every dotfile as administrative. |
| Nested repository in participating data | Existing JW guard refuses it. To keep an entire nested checkout local, preserve that checkout's subtree, rather than silently copying it without its metadata. |

Two less obvious outcomes matter:

* If the donor changes `packages/cli/dist` from a directory to a symlink, the
  target's preserved `local-db` blocks the transition. The link target is not
  followed, and the database is not displaced into an invented backup directory.
* Dependencies describe S's installed state even with `--at` selecting a feature
  D with different manifests. This is already the product's constraint: the
  feature's normal install/build can be needed. Generic copying cannot infer
  package-manager validity from either ignores or filenames.

### The real consequence of decoupling ignores

Broad transfer can copy a **nonignored, unrecorded** file. JW's own verification
continues with auto-tracking disabled, so it does not silently turn that file
into D. A later ordinary JJ snapshot may track it according to normal repository
configuration. Similarly, D can stop ignoring a dependency directory that S
ignored.

I recommend allowing this ordinary behavior and dropping the two-sided
ignoredness eligibility requirement for the new transfer model. Do not secretly
edit `.gitignore`, write excludes, or persist `snapshot.auto-track=none()` to hide
it. If the product instead promises that **every transferred auxiliary path
remains outside future snapshots**, that is a separate selected requirement:
keep a destination snapshot-policy admission step in JW, with its real rule/query
cost. It still must not define overwrite/deletion authority. Merely checking
the source's ignore rules would not establish that stronger promise.

## Cost model and concrete work removed

Let T be tracked leaves already obtained from JJ, V the included paired-tree
entries and structural routing directories, U visited one-sided entries, and
W actual installed/removed leaves. Excluded interiors are outside V/U. A mixed
absence policy can require target-only traversal that a uniform update avoids.

| Work | Current integrated path | Proposed path |
| --- | --- | --- |
| Payload selection | Source-only `Patterns::select`; outer roots discovered before sync. | Broad scoped union traversal makes source-root discovery unnecessary. Mirror/preserve matchers classify names during that traversal. |
| Ignore admission for transfer | Freeze tracked ignore files; construct/reuse Git oracle contexts; query candidate roots in source/desired contexts. | None in the recommended broad-transfer model. Rules still needed by another actual caller, such as the existing link contract, remain with that caller. |
| “Outside selected payload” validation | `payload::outside` walks retained/tracked regions during planning and again before refill; replacement adds broad passes. | Remove whole retained-region validation. Inspect tracked obstructions where JJ will act and check participating auxiliary entries in the actual traversal. Preserved interiors are genuinely untouched. |
| Paired payload comparison | Incremental, always-descending diffdirs; clean entries incur metadata work but no writes. | Same essential O(V) work. No directory-stat shortcut and no full payload hash. |
| One-sided copy/delete | Checked recursive sync already visits interiors. | Same O(U) traversal, adding scope and retained outcomes. No preliminary full payload pass. |
| Tracked prefill | JW-owned `clone_files` for S/D-equal leaves. | Same necessary clone work to renew sharing, using diffdirs' single staged kernel. No filesystem discovery for T's known leaves. |
| Large type replacement | Stage plus recursive removal; deep refusal may follow deletions. | Local removability scan adds one affected-subtree pass for directory→leaf, in exchange for preservation-conflict refusal before dismantling it. |
| Mount checks / reflink feasibility | Linux upfront mountinfo; actual staged clone errors. | Retain that model. No per-entry mount-table matching, mount syscall or synthetic clone probe. |

Memory stays current directory listings, active DFS/frontier state, staging and
small reports, plus JW's O(T) native tracked indexes and compiled configuration.
It is width-sensitive, like today's walk; it is not a promise of depth-only
memory. There is no O(all payload entries) operation map.

A mixed scope still pays name/prefix classification; monomorphization does not
make a real matcher free. Use cheap native-name cases and compiled prefix
indexes, rather than comparing every payload entry against every tracked path
or configuration string. A uniform included subtree can avoid those lookups
only when the concrete policy can establish uniformity; it must still perform
all paired-directory comparisons and required entry checks.

Tracked verification is another substantial cost, independent of payload scope.
Current `execute` calls `validate_source` four times per target; each includes a
live tracked-tree verification. Under the existing paused-donor contract,
recommend one verified donor capture for the batch and one final donor check,
with cheap identity/revision comparisons at intermediate barriers. No command
between them is supposed to mutate the donor. Keep the target pre-clear
bookmark/work-tip check and one final tracked-tree verification after JJ,
auxiliary transfer and links. A separate post-materialization snapshot can be
removed once the explicit S/D exclusion proof is established. This changes
when an unexpected writer is detected; it does not establish concurrent-writer
support. It removes repeated work instead of moving it into another preflight.
If the final shared donor verification fails, report the batch as unverified
with its completed target work; do not certify donor consistency retroactively
or pretend to roll those targets back.

The new broad default may visit/copy **more** data than today's narrow selectors.
If the donor has a large unselected cache, broad update includes it unless
excluded. Reduced passes do not guarantee a faster command under a larger
contract. Keep scope and requested outcome equal when making performance claims.

For N destinations, this first design still reads source metadata N times. The
library's [source-to-many direction](file:///home/rektide/src/diff-dirs/README.md#coordinated-multiple-destinations)
could later share one directory observation while retaining per-target authority
and failures. It is independent of fixing scope; a persistent source catalog or
a new scheduler is not a prerequisite.

### What the measurements actually establish

The [cancelled primary report](../up-performance/primary0.sol61m.md#cancelled-campaign-retained-observations-not-completed-matrix)
records 8,557 tracked paths and 68 selected payload roots: 171,567 regular payload
files, 9,561 links and 11,356,860,904 logical bytes. Five of twenty reduced samples
finished. Serial N2 clean took 17.64/21.15 seconds versus paired baselines
99.11/108.31 seconds. P4 has one unpaired 8.81-second observation. There is no
completed actual mutable/scaling matrix or evidence selecting P4 as JW's default.

The failure was the experimental design too: roughly seven full payload copies
per N2 sample, repeated hashes and retained cases. Setup cost 336–460 seconds
and verification 91–176 seconds per completed sample; the filesystem reached
99% utilization with a copy waiting for Btrfs metadata reservation. The parent
commissioned and accepted that harness and shares responsibility. Global disk
use is not attributable solely to it.

The separate [64-sample library assessment](file:///home/rektide/src/diff-dirs/design/performance/reassessment0.sol61x.md)
supports checked warm-clean library behavior within its own scope; the reported
64 tiny synthetic JW samples cover different questions. Neither validates this
new design. All of those useful observations survive cancellation; none licenses
recreating the deleted fixtures.

## Credible alternatives

| Alternative | Strong reason to choose it | Cost / semantic tradeoff |
| --- | --- | --- |
| **Recommended: broad update + preserves + explicit mirrors** | Matches source→destination reasoning while making absence authority explicit. No copy-ignored selector needed. Useful generic library boundary. | Same-name donor auxiliary files now overwrite unless preserved; extra source data transfers; mixed policies can traverse target-only regions. |
| **Whole auxiliary-root mirror + preserves** | Simplest exact donor replica, particularly for disposable workers. One deletion authority everywhere outside exclusions. | Every unpreserved target-only local file disappears. This must be an explicit selected mode, never the accidental effect of omitting `copy_ignored`. |
| **Explicit transfer roots + preserves** | The donor contains much more than should be propagated, or users want opt-in data movement. | Root discovery/configuration remains. Name these roots by transfer purpose and select update versus mirror explicitly; ignoredness still need not be their authority source. This is the credible conservative alternative, not just a new spelling for the old eligibility pipeline. |
| **Ordinary JJ checkout, generic auxiliary sync** | The tracked CoW-renewal outcome is no longer important; the smallest JW adapter is preferred. | Drop sparse-prefill/state-copy complexity at the price of no donor-sharing promise for tracked files. Choose this as a product change, not an equivalent optimization. |
| **One generic virtual desired tree spanning JJ objects and donor disk** | Could be worthwhile if diffdirs later has several independent non-filesystem source users. | Requires revision/object/mode/link sourcing, JJ bookkeeping integration and new equality semantics now. The existing verified JJ materializer already solves that part; insufficient evidence for this larger abstraction. |

Merely teaching `Check` to return “skip” and filtering `DirDiff` is not an
alternative: it leaves recursive one-sided copy/removal, retained parents and
type transitions incorrect. Merely renaming `copy_ignored` keeps the disputed
authority coupling intact.

## Decisions for the human

1. **Breadth:** select broad auxiliary update with exclusions (recommended),
   opt-in transfer roots, or explicit full auxiliary mirror. Broad update
   preserves target-only extras but still overwrites unpreserved same-name data.
2. **Deletion regions and disappearance:** should configured `node_modules` /
   `dist` mirrors be authoritative even when a whole matching donor directory
   disappears? Recommend yes. Today a source-only selector never selects an
   absent root, so that case is preserved instead. This is a real change.
3. **Snapshot eligibility:** allow nonignored auxiliary transfer with ordinary
   future JJ snapshot behavior (recommended), or require destination policy to
   keep every transferred path unrecorded? Do not conflate this with #1/#2.
4. **Tracked CoW renewal:** retain the present outcome and share its installer
   (recommended), or simplify to ordinary JJ checkout? Desired revision selection
   remains the current donor-default/explicit-`--at` contract in either case.
5. **Preserved type-transition conflict:** locally check removability before
   dismantling a directory (recommended), or accept streaming partial sibling
   deletions before discovering the conflict? This choice affects only that
   exceptional shape path, not a compulsory full preflight.

The whole-root deletion mode should have a distinct explicit name. Existing
`--replace-content` means clearing unrecorded target contents **without** copying
arbitrary source dotfiles; it is not equivalent to the proposed full auxiliary
mirror. Do not silently alias the two.

## Coherent implementation order, after selection

1. **Settle the semantic table in the native specs.** diffdirs owns generic
   preservation/absence/report semantics; JW owns the selected transfer default,
   ignore/snapshot consequences and tracked boundary. Name the actual contract
   changes before changing behavior.
2. **Complete generic scoped sync end to end.** Add pair-wide scope before
   speculative descent, cover recursive actions and initialization, retained
   containers, explicit update versus mirror, and the selected type-transition
   behavior. Keep static specialization, existing staging and borrowed pools.
3. **Expose the existing leaf kernel for explicit seeds.** Move JW tracked
   prefills onto it, preserving JJ ownership, real-reflink requirements and the
   distinct creation fallback. Replace recursive JW clone/remove call sites
   only as their explicit authority is represented. `remove_recorded_file` must
   remain leaf-only; a generic recursive remove there would erase local children.
4. **Integrate the selected JW transfer model.** Replace source-only ignored-root
   discovery with root-scoped transfer and S/D exclusions. Preserve native admin,
   link-owned and local regions through tracked refill. Remove redundant
   retained-region scans and transfer-only ignore machinery from that route.
   Keep the upfront topology guard; initially its existing conservative footprint
   can remain. Supporting mounts inside excluded local trees needs a separate
   exact footprint argument, not per-entry lookups.
5. **Consolidate verified coordination and update native consumers.** Reduce
   repeated donor/target scans as described above. Update `add`/new `s`, `reseed`,
   `prepare`'s source validation, user/primary-local config, help/completions,
   README/skill and relevant expectations together. An existing `s` still does
   not refill. `prepare` does not gain worker selection or transfer work.

Below 1.0 a clean config/API break is reasonable. Actual consumers include the
local/global `copy_ignored` lists, their empty/disable precedence, creation and
prepare, `--replace-content`, and test/performance callers of current library
constructors. An old `copy_ignored=[]` cannot silently become “copy everything.”
Either explicitly migrate the user's configuration to the selected model or
reject the obsolete field with the required new meaning. No permanent dual
policy engine or invented compatibility layer is necessary.

## Minimal future verification, with bounded physical work

The first verification of this design should be **semantic**, using the existing
Rust test infrastructure after implementation. No real opencode import or new
campaign is needed to distinguish its critical behaviors.

* One owned source/target pair at a time; at most **256 payload entries and 1 MiB
  of payload bytes per pair**, including any staged replacement data. A native
  JW case uses one repository and its donor/target workspaces, not extra control,
  snapshot or probe copies. No input-directory/import option in this small suite.
* Reuse that bounded pair for source-only exclusion, target-only preservation,
  retained ancestors, both type-transition directions, absent target/missing
  source roots, literal links and private CoW writes. Add a tiny native JJ case
  with S different from D: tracked deletion, mode change, a new feature path and
  unrecorded siblings in the same directory. Assert preserved bytes/identities
  and exact JJ tree/parent behavior.
* Explicitly exercise a vanished mirror root, public records below preserved
  ancestors, and scope matching through staging. These distinguish a complete
  scoped reconciler from an output filter. Keep the current recursive-refusal,
  mount and staged-reflink precedents in their existing appropriate suites.
* For a clean-repeat observation, use the same pair before and after; no resets
  or byte copies between clean runs. Verify zero writes and selected inode/mtime
  invariants. For a mutable case, reset only the small named changed paths from
  literal expected data. Full byte checking of this tiny bounded pair is cheap;
  never turn it into repeated whole-workload hashing.
* If an initial performance smoke comparison is desired, fix it to one semantic
  scope, one target and one four-run balanced sequence over that same bounded
  pair. Record setup, operation and verification separately. Such a result is a
  smoke observation, not a real-workload speedup or concurrency-default decision.
* Print/flush phase boundaries and periodic directory/work counters without a
  pre-counting walk. Cancellation stops admission, joins started work and emits
  the partial report; a second interruption may terminate a blocked process.
  Kernel I/O can delay cooperative cancellation. Do not auto-reset or start the
  next case after cancellation. Retain compact logs, coordinates and failed-case
  descriptions; the bounded pair is the maximum failure artifact.

Those fixed input, residency and byte limits make the earlier multi-copy fixture
explosion impossible in this first verification. A later actual-workload study
would need its own explicitly selected question and resident/reset/hash budget;
the cancelled matrix is not a backlog to resume automatically. Reusing Rust
tooling does not by itself make a repeated-full-fixture experiment proportional.

## Relationship to the existing work

* The [human recovery brief](recovery0.gpt6a.md) owns the commission and cancelled
  experiment boundary. This document answers it with a new authority model.
* [JW's current operator guide](../../docs/reseed.md#ignored-data) owns present
  behavior. Its opt-in, two-sided-ignore and replace-content guarantees are
  exactly the contracts human selection would revise.
* The [inline-policy proposal](../up-performance/inline-policy0.gpt6a.md) and
  [generic policy antecedent](file:///home/rektide/src/diff-dirs/design/fit/policy0.gpt6a.md#4-cover-recursive-actions-not-just-comparison)
  supplied the useful streaming/check/topology seam. This proposal extends their
  deferred recursive preservation and challenges their selected-ignored-root
  organizing model.
* The [delivered integration](../up-performance/streaming0.sol61m.md) is evidence
  that checked incremental sync, preserved payloads and native-context reuse
  already work within a narrower contract. Reuse those kernels and native
  precedents; architectural simplification does not require discarding them.
* The native [comparison specification](file:///home/rektide/src/diff-dirs/README.md#model)
  and [sync specification](file:///home/rektide/src/diff-dirs/sync/README.md#replacement-permissions-and-metadata)
  own equality and filesystem-operation semantics. This proposal preserves their
  stat/literal-link model and staged-copy discipline while deliberately extending
  scope, absence authority and coverage reporting.

**Bottom line:** JW needs a revision adapter and a transfer policy. It does not
need its own filesystem synchronization implementation, and ignoredness does
not need to be the gate through which all auxiliary copying is expressed.
