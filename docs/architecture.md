---
type: Architecture
title: jj-waltz architecture
description: Module ownership and workspace lifecycle safety boundaries.
resource: /docs/architecture.md
status: maintained
updated:
  by: model:openai/gpt-6.1-sol#high
  at: 2026-10-17
sources:
  - resource: "opencode://ses_eb867bfdaffefMVLpyaPUadbSp"
    title: Added built-in all membership over registrations while retaining the shared readiness and recovery paths
  - resource: "opencode://ses_eb8a7487affeE2smndZ914OQMS"
    title: Reviewed command isolation and clarified legacy-only source recovery routing
  - resource: "opencode://ses_eb899cdcdffecJnB7J1dSGs4ci"
    title: Documented the prepare/reseed command split, v4 source-only journals, legacy recovery routing, and the bounded tracked-tree verification split
  - resource: /design/up-performance/prepare0.gpt6a.md
    title: Command-split decision record and first performance slice adopted by this architecture
  - resource: "opencode://ses_ec10d60dbffencx7PCZnh1dnjg"
    title: Integrated native source selection into shared preparation, v3 up mode and source-only receipts, and atomic target-history planning
  - resource: "opencode://ses_ec11dfc0dffeV1y2Am4AyutJlT"
    title: Implemented target-history planning, collision-safe timestamp allocation, and exact create-only bookmark recovery through existing pending and preparation intent
  - resource: "opencode://ses_ec1394461ffeEXBHM582vTs9VM"
    title: Fixed nested payload preflight and implemented source assertions/current-target borrowing with phase-specific preparation and frozen-target recovery
  - resource: "opencode://ses_ec14d159effeiOep7U7f2iNQ7E"
    title: Extracted shared frozen ignore/payload proof primitives and implemented add/s private payload planning, policy ownership, and read-only rollback
  - resource: "opencode://ses_ec1a8d43bffe02bXuEoA7hEW3l"
    title: Implemented native source, command/recovery, and shared ignored-copy policy/selection foundations with explicit frozen-policy compatibility
  - resource: "opencode://ses_ec206881bffenBcwNsQxlvfx3J"
    title: Implemented and verified shared add/s placement, snapshotted-current creation bases, explicit mode resolution, and visible fallback reporting
  - resource: "opencode://ses_ec37b01d2ffe9ytO0jRsdmoyXA"
    title: Integrated reseeding, verified recovery, primary-source selection, and preparation-batch ownership
  - resource: "opencode://ses_ec37a1a86ffeYXsS5j3A7Jng0t"
    title: Added snapshot-free wildcard group resolution and early structured selection diagnostics
  - resource: /design/workspace-reseed/reseed0.gpt6a.md
    title: Reseed planning, prefill, verification, and durable recovery design
---

# Architecture

`jj-waltz` keeps command syntax at adapters and puts behavior behind deep module
interfaces. Callers should not reproduce Jujutsu command grammar, lifecycle order,
link validation, or shell switching policy.

This document separates milestone-zero behavior from later publication and forge
work. The current command surface remains authoritative in `jw --help`.

## Product boundary

`jj`, `jw`, and forge clients solve different jobs:

| Owner | Responsibilities |
| --- | --- |
| `jj` | Revision creation and rewriting, conflicts, bookmarks, fetch/push, and operation-log recovery |
| `jw` | Workspace lifecycle, links, repository snapshots, managed intent, and cross-workspace safety |
| `gh` / `glab` | Pull or merge requests, forge checks, and CI state |

`jw` may coordinate a graph or publication step through `jj`, but it does not add
general wrappers for ordinary JJ verbs. Forge and network adapters are optional;
core creation, switching, inventory, local status, links, and removal stay
offline.

## Module map

- `cli` is the terminal adapter. It parses flags, asks questions, and renders
  outcomes.
- `lifecycle` owns creation policy, ordered add/switch workflows, adoption, and
  explicit metadata repair. It loads user config once, applies links, records
  successful switches, and rolls back failed creations.
- `jj` is the only production adapter that starts JJ. It owns frozen operation
  queries, version compatibility, process policy, and typed command errors.
- `observe` refreshes selected working copies, captures one final operation, and
  derives `list`/`status` snapshots without renderer subprocesses.
- `snapshot` defines schema-versioned JSON values and semantic status types.
- `metadata` persists repository-scoped managed-workspace intent.
- `doctor` runs independent read-only checks and always produces a complete report,
  including configured-link health for every managed workspace.
- `workspace` owns Jujutsu workspace discovery and mutation. Its inventory gives
  commands one consistent view; removal is planned before execution.
- `cow` owns copy-on-write file cloning and filesystem capability probing. The
  `workspace` module orchestrates JJ adoption of the clone. The `jj` adapter owns
  listing tracked files and reading and writing JJ's local working-copy state,
  which is not a public interface: every copied state is validated by JJ and
  replaced on rejection.
  Its `cow::ignored` submodule owns shared ignored-copy precedence and the compiled
  eligible-root selector; command adapters supply frozen source rules and tracked
  paths instead of implementing their own pattern traversal.
  `cow::payload` owns the shared native frozen-ignore adapter, operation-bound
  tree entries, filesystem stamps, confinement and logical payload digests.
- `reseed` owns existing-workspace batch reseeding: explicit selection and local
  configuration, frozen planning, sparse-prefill execution, ignored replacement,
  verification, and repository-scoped pending records and receipts. It uses
  public commands through `JjClient`, not creation's private `tree_state` copying.
  Filesystem cloning and validation remain in `cow`; configured links remain in
  `links`. Historical lifecycle metadata is not rewritten by reseeding.
- `freshen::source` owns operation-bound upstream selection and post-fetch graph
  proofs. This foundation does not fetch, advance, or refill workspaces itself;
  `prepare` owns those mutations and their journals, and legacy recovery replays
  recorded intent without reselecting.
- `freshen::history` owns target work-tip/publication planning, timestamp bookmark
  allocation, and create-only verification of a previously journaled name/tip.
  No new command plans preservation: `prepare` never allocates history bookmarks.
  The retained semantics serve legacy v3 journals, whose recorded intent the
  `reseed --resume` executor checks create-only before clearing or advancing.
- `links` owns config merging, path confinement, link classification, preflight,
  and filesystem changes.
- `shell` owns shared shell behavior. Each shell adapter varies syntax, not policy.
- `manager` owns the interactive terminal adapter. `manager::ui` owns the Ratatui
  0.30.2 event loop, rendering, keyboard/mouse input, search, and multi-selection;
  it uses Ratatui's crossterm re-export and remains compatible with Rust 1.88.
  `manager::status` derives display state from completed snapshots, while
  `manager::removal` plans and executes eligible removals and
  `manager::preferences` persists the user-local bookmark policy.
- `plugins/herdr` is a separate Cargo workspace and a UI adapter over `jw`. Its
  removal workflow closes Herdr before deleting provenance.

```mermaid
flowchart LR
    interfaces["CLI / JSON / Herdr / shell views"] --> app["Lifecycle services"]
    app --> domain["Snapshot, metadata, plans, safety"]
    app --> ports["Application port traits"]
    adapters["JJ / filesystem / optional forge adapters"] --> ports
    adapters --> domain
```

Arrows show allowed source dependencies. Domain values and policies have no
dependency on Clap, terminal formatting, shell syntax, `gh`, or process
execution. Lifecycle services own the port contracts; concrete adapters implement
them and translate external results back into domain values. A separate-process
view such as Herdr crosses through the CLI/JSON boundary. Renderers consume
completed snapshots or plans and never run their own JJ queries.

The interactive manager is another renderer and action adapter at this boundary.
It does not introduce a second repository model or a second JSON contract. Search
and selection stay in manager state; repository services receive named targets and
return completed per-target outcomes.

Do not create one source directory per conceptual noun. Existing deep modules can
own these responsibilities until a split reduces coupling.

## Snapshot boundary

Any status or plan that compares workspaces uses this sequence:

```text
discover repository and selected workspaces
refresh selected working copies
capture final JJ operation ID
query repository and graph facts at that operation
derive statuses in memory
render once
```

Refresh may be current, selected/all, or none. Skipping it trades freshness for
speed and must be represented in output. No status derivation may trigger another
implicit JJ operation after capture.

All JJ process execution belongs behind the central `jj::JjClient` adapter. It
owns safe argument passing, color/pager policy, parsed version and capability
checks, operation-ID capture, template parsing, errors, diagnostics, and test
fakes.

### Creation-base contract

Creation resolves one exact base commit before creating destination workspaces.
With no `--at`, `resolve_creation_base` first runs source `jj status`, then captures
the operation and resolves `@` at that operation. This refresh is necessary because
`operation_id()` and frozen revision queries deliberately do not snapshot. Dirty
tracked changes and normal JJ auto-tracking therefore enter the implicit base.
An empty merge working copy is itself one valid commit, not an ambiguous parent set.

Explicit `--at REVSET` retains the existing frozen-operation resolution branch,
without the implicit refresh unless a source assertion is supplied. It must resolve
to exactly one commit; no policy substitutes source `@` for that selection. JJ
creates an empty child of the exact base, which is also recorded in lifecycle
metadata. Creation, switching, provenance,
and rollback share this contract; reseed's separate keep/reset semantics are unchanged.

CLI `--source-at` snapshots the donor even alongside explicit `--at`. Donor `@`,
assertion, and target base resolve at one operation; tree equality, not commit
identity, authorizes borrowing. `CreationBase` retains the independent target
revision and asserted donor ID. Payload planning retains its own fresh snapshot
and requires that donor ID, rather than silently reasserting a moving revset.
No assertion means unchanged creation behavior. Local reseed donor/assertion/
freshening fields do not enter creation policy, and existing-only switches skip it.

### Source, destination, and creation mode

The CLI resolves an explicit destination against the caller's directory before
entering optional `--from SOURCE`. Existing lifecycle/JJ helpers then share that
source command context. Without `--from`, the current checkout is the source.

`WorkspaceInventory` validates the source's JJ repository pointer and identifies
the owning `.jj/repo` directory for placement. This ownership is distinct from Git
colocation eligibility or a registration named `default`; it remains useful when
the primary registration has been forgotten. Native workspace registrations remain
authoritative for existing names and paths.

For one `--path`, shared resolution uses a matching registration first. Otherwise
it strips a clear primary-basename-plus-`-` prefix (`opencode-fix` becomes `fix`),
or keeps the destination basename (`myfix` stays `myfix`). An explicit `NAME` can
override inference for a new path, but cannot contradict an existing registration.
Destination paths flow through creation preflight, CoW probing, and `AddOptions`,
including ordinary-checkout fallback. Name-only placement remains `repo.NAME`
beside the owning checkout, not an automatic `~/src` or hyphen-based destination.

`CreationMode::resolve` is the single mode policy: CoW/JJ-only by default; positive
CLI choices override config; otherwise CLI opt-outs overlay their matching config
fields. Config preserves unset/true/false intent, allowing `colocate=true` alone
while rejecting explicitly configured CoW plus colocation. Explicit CLI `--cow`
plus `--colocate` also conflicts. No primary-topology or JJ-setting inheritance
selects colocation. See the [operator precedence table](../README.md#creation-mode-precedence).

## Repository-scoped metadata

Dynamic workspace metadata is separate from static user configuration. Resolve
the store from public JJ configuration plumbing:

```text
repo_config = jj config path --repo
store       = parent(repo_config)/jj-waltz/

jj-waltz/
  manifest.json
  workspaces/
    workspace-<stable 128-bit id>.json
```

`manifest.json` records schema version 2 (version 1 remains readable) and a deterministic `repo-<id>` initially
derived from the normalized canonical repository-config path. Once created, the
manifest is authoritative. JJ's secure per-repository config path is stable when
the repository moves, and a moved config directory plus its adjacent `jj-waltz`
store also keeps the persisted identity.
Each schema-versioned workspace record can be written, repaired, or removed
independently and contains only `jw` lifecycle intent such as creation time,
creation operation ID, recorded creation base, associated bookmark, and intended
remote. Writes use a unique same-directory temporary file followed by atomic
replacement. Parse or identity errors are diagnostics, never permission to
silently reset the store. No secrets or forge tokens belong here.

This location is repository-scoped, shared by sibling workspaces, uncommitted,
and avoids undocumented `.jj` layouts. Moving only a checkout while leaving its
repository config or copying a detached metadata store is not automatic
migration; `jw` never searches arbitrary paths or silently adopts an old store.
On an older repository that has never used per-repository configuration, JJ's
documented `config path --repo` query may initialize an empty secure-config
directory. Snapshot and doctor commands still leave the JJ operation and working
copy unchanged.

### In-place reseeding

`reseed::selection` resolves legacy literal groups and table groups with explicit
name/path patterns against a frozen, snapshot-disabled registration inventory.
It retains missing-root registrations rather than classifying broken targets as
unrelated checkouts. Selection never initializes or adopts matched directories.
The optional unmatched policy affects group misses only, not positional names or
target safety. Its aggregate warning is emitted through `run_with_diagnostics`
before planning (and survives later errors); successful reports retain structured
skips and selected roots/origins. Empty remaining selections fail. Resume never
re-expands groups. See [operator syntax](reseed.md#wildcard-groups).

Reserved built-in `all` expands the sorted registration inventory, excluding only
the verified primary and effective donor by canonical root identity. Donor path
resolution is shared with source preflight, not repeated per target. Unusable
registrations become selection issues; all other members enter the existing
readiness and bookmark checks. A local `all` definition is rejected. See
[all registered workers](reseed.md#all-registered-workers).

Reseeding records pending intent and successful receipts separately from creation
metadata, beside the repository-scoped store. A reseed advisory lock serializes
reseed callers, not editors, independent JJ commands, or build processes. These
writers must be paused by the operator.

The authoritative repository manifest is initialized before nested reseed storage
is created. This also works for unmanaged native JJ workspaces: initializing the
repository identity does not adopt a workspace or create lifecycle records.
Existing corrupt or nonempty manifest-less stores are rejected rather than reset.

Plans snapshot the prepared source and named targets before capturing exact
revisions. A full, conflict-free prepared source must match the selected source
tree. Full JJ-only targets retain root and `.jj` identity and registration. The
primary checkout and source are never eligible targets. CoW probing, ignored-root
validation, and filesystem/ownership checks precede destructive work.

The donor path uses explicit `--from`, then configured `source`, then a verified
primary checkout of the current repository. Discovery follows JJ identity rather
than searching archive directories or substituting the current feature checkout.
Ordinary donor baseline selection is CLI `--source-at`, then configured `source_at`,
otherwise the freshly snapshotted donor `@`. Asserted tree identity is checked at
the same captured source operation before target snapshots. A donor/target tree
difference is handled per path; an explicit/configured mismatch remains an error.
Up uses native source selection instead of ordinary reseed's tree assertion.

Snapshot-free `Inventory` retains the caller's verified registered name. With no
names/groups, `run` inserts that one non-primary literal target before pending
lookup or source/target snapshots and reports origin `current`. At the primary it
requires explicit selection for reseed. `prepare` is separate and source-only
from any registered workspace. Explicit names/groups suppress this default even for
an empty group result. No navigation-token semantics or implicit-all selection is
introduced. Resume uses recorded source IDs and ignores newly configured defaults.
While legacy source preparation has no frozen target plans, recovery requires the
complete recorded target order. After plans freeze, each selected target/receipt is
recovered independently against its frozen identity, without refetching or
rebuilding. Bare current selection never expands to the rest of the batch in
either phase.

`prepare` snapshots the source, freezes native upstream selection, fetches only
that bookmark/remote through `JjClient::fetch_bookmark`, validates/pins the source
goal, selects its empty child, and runs trusted configured argv in the donor. It
then records an explicitly empty target plan (journal v4), finalizes the
completion receipt, and reports the exact frozen source commit. It never enters
target planning and creates no worker-preservation bookmarks; it mutates only its
selected donor — which may be the caller's own checkout — leaving other
workspaces untouched. Preparation settings alone never activate it. The
source-keyed
journal stays active until its receipt and blocks new borrowing or preparation
that would invalidate unfinished work, including older unselected target records
depending on the donor.

Verified preparation binds the source change/goal, root identities, selected
ignored roots, and logical payload digest. Normal JJ restoration may change a
commit ID; same-change/exact-parent/tree/payload proofs permit recovery without
accepting unrelated tree-equivalent changes. Source ownership and trusted policy
are rechecked before resumed mutation or installer execution. Shared baseline-owned
link configuration is captured after preparation, not mistaken for policy tampering.

Legacy batches store all target plans atomically in the preparation batch before
individual pending records are installed. Preparation IDs and immutable plan
comparisons associate pending records/receipts, while mutable runtime link
diagnostics are separate from planned warnings. Lost handoff recreates only
missing recorded intent; completed receipts prevent rerunning finished targets or
preparation. Target checks refuse changed/stale work rather than snapshotting
installer changes into it. No second preparation/refill engine was introduced.
New prepare journals are v4 and freeze `SourceSelection` before commands. Legacy
v3 up journals, v2, and v1 records have their recorded selection/semantics and
are resume-only through `reseed --resume`, using their recorded
remote/revset/base and target plans rather than acquiring prepare's source-only
shape.

Caller inventory is resolved before donor selection; `--from` never changes
process cwd. Reseed requires a registered caller; `prepare` accepts any
registered workspace with the same source-only meaning. A new source-only
journal uses empty names/targets and an explicitly verified empty `plans` array,
not an empty group or fake target. Its receipt records the same final
source/payload proof. `prepare --resume` finds the unique active v4 journal
before any snapshot, even for an alternate donor. Bare `reseed --resume` at the
primary recovers legacy source-only v3 journals; it does not resume v4 preparation.
Legacy target batches require their recorded names. A second unfinished
source-only operation is refused. Frozen source-only completion revalidates the
verified donor before finalizing its receipt.

Execution clears sparsity, optionally creates a fresh child at a frozen base,
prefills tree-identical paths, expands sparsity, and verifies the target against
its desired revision. Configured ignored roots are copied only after tracked
verification. An interrupted target is resumed from its pending record before
ordinary snapshotting; partial prefill must never become intended user history.
Batch recovery does not use repository-wide undo.

### Tracked-tree verification split

Normal-path verification of an intact, full-sparse workspace
(`engine::verify_disk`) is a bounded JJ tracked-tree proof: one snapshot with
`fsmonitor.backend=none`, `snapshot.auto-track=none()`, and
`snapshot.auto-update-stale=false`, then a single
`jj diff --from @ --to <frozen commit> --summary` whose empty output proves
tracked-tree equality under JJ's semantics, including its normal
[mtime/size stat cache](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/docs/technical/architecture.md#treestate).
This deliberately replaces the former per-file byte-extraction audit. It is not
an unconditional raw-byte promise, and because the snapshot runs first, a
mismatch can be reported after a snapshot was taken, possibly recording a
rejected edit; the earlier "no snapshot performed" rejection promise is retired
on this path. Auto-tracking stays off, so a snapshot never adds files, and
automatic stale-tree updates are disabled; JJ may still refresh metadata-only
stale state when the tree identity is unchanged.

Recovery keeps the strict path. A target that crossed the `Prepared` boundary —
or any sparse-incomplete tree — cannot be trusted to an empty diff, so
`engine::verify_reconstructed_disk` retains per-file byte/type/mode verification
against pinned immutable data before any snapshot-capable command. Fresh
execution is decided by the recorded target mutation phase
(`reconstruct = phase != Prepared`), not by preparation-ID presence, so an
untouched preparation handoff no longer enters phantom reconstruction while a
half-filled target still reconstructs before snapshots.

Stock JJ rejects sparse mutation with `--ignore-working-copy`. Recovery therefore
reconstructs and verifies the entire recorded desired tracked tree before a normal
sparse-clear command can snapshot it. Identical files come from the frozen source;
other recovery bytes come from frozen public JJ file queries. Sparse expansion
runs normally while sparse-empty, where staged prefills are outside the snapshot
selection. Scoped `snapshot.auto-track=none()` prevents unrelated retained files
from entering the desired change. This is a public-CLI recovery protocol, not an
import of JJ's private working-copy state.

See the [operator guide](reseed.md) and
[design](../design/workspace-reseed/reseed0.gpt6a.md) for policy and limitations.

### Shared ignored-copy policy and selection

[`cow::ignored`](../src/cow/ignored.rs) is command-independent. `Policy::load`
resolves optional CLI/local lists and root global `copy_ignored`, preserving
missing versus explicit empty intent. `--no-copy-ignored` wins; otherwise lists
replace in CLI > local > global order. Global config is read only when inherited,
and never written. `Policy::recorded` represents frozen recovery selection instead
of reading current defaults. Current command exposure includes add/s creation,
reseed, prepare payload verification, and legacy source preparation recovery.

`Patterns::compile` normalizes trailing slash, rejects unsafe relative paths and
unsupported syntax, deduplicates equivalent selectors, and compiles components.
`Patterns::select(source, tracked_paths, is_ignored)` carries all active pattern
prefixes through **one directory-selection traversal**. Recursive `**` has
zero/one-component transitions; it does not launch another subtree walk. Prefixes
that cannot match are pruned. Selected ignored outer roots prune the selection
walk and include nested matches once, so covered descendants are not separately
enumerated or spuriously reported missing. Administrative directories, directory
symlinks, and mounted-child boundaries are not traversed.

Creation and reseed supply `is_ignored` from the shared isolated native Git probe over
frozen desired `.gitignore` content and excludes. A nonignored source match is
skipped/reported while viable descendant patterns continue. Native probe/I/O
failures are errors, not silently converted to skips. An ignored candidate is
checked against frozen source tracked paths before selection; forced-tracked
overlap is a hard failure. Selected roots still face target ignored/tracked,
confinement, obstruction, and actual private-CoW checks. Unselected target data
is not automatically pruned; `--replace-content` remains its explicit destructive
exception. See [operator policy examples](reseed.md#ignored-copy-policy-layers).

After preparation is verified, its recorded root set is authoritative.
`validate_selected` checks those exact roots, parent confinement, source ignore
rules and tracked overlap without discovering additional matches. Target plans
use that set, not a fresh expansion of the original selectors. This preserves
legacy verified payload scope across matcher fixes (including formerly unmatched
trailing-slash patterns). New matches outside the frozen roots are neither adopted
nor pruned; changed bytes within selected roots still fail the frozen digest proof.
Selection diagnostics are optional proof metadata; older records default to none.

This optimizes matching, not every filesystem operation. Frozen ignore assembly,
source/target stamps, recursive validation, payload digest, capability probes and
copying retain independent correctness passes. New/unverified preparation compiles
syntax before fetch and selects against the exact frozen goal before commands and
at source-proof capture. Verified recovery validates frozen roots instead. It
does not reject a missing literal directory
or require a builder merely because global patterns exist.

The typed reseed request now uses an optional selection; CLI absence and local
absence remain distinct from explicit `[]`. Preparation always serializes an
effective array in the frozen request. Existing v1/v2 arrays deserialize to
`Some(array)` with the same wire representation, including frozen empty lists;
missing frozen selection is rejected instead of inheriting new policy. Target
pending roots and journal schema versions are unchanged by this stage.

### Creation ignored payload

[`lifecycle::ignored`](../src/lifecycle/ignored.rs) prepares a single frozen plan
only when at least one workspace will be created. `WorkspaceInventory` identifies
the policy owner by the real JJ repository directory, independently of a workspace
named `default`. Creation reads only `copy_ignored` from that owner's existing
`.jwreseed.local.toml`; local source/freshening settings are not implicit creation
overrides. Explicit CLI selection/disable bypasses local-policy loading.

The donor is snapshotted independently of the exact target base. Shared
`cow::payload` primitives bind its current commit, filesystem stamp, frozen source
ignore rules and selected payload digest. Selected roots also require the exact
base's frozen ignore rules and no overlap with its tracked paths. An empty
selection returns before clone capability is required. Nonempty plans recheck
source identity/content around copying, compare the new target revision with its
frozen base, and reject target obstructions and parent symlinks.

Nonempty payload plans preflight every prospective destination against the
canonical donor root, resolving existing parent aliases. A nested destination is
rejected before any workspace/bookmark is created: creation itself would change
the whole-donor stamp, and a destination inside a selected root risks recursive
copying. The proof does not exempt destination subtrees or ancestor metadata.
Disabled copying and no-match selections retain ordinary nested placement.

Tracked creation remains free to use ordinary checkout or its established warned
fallback. The separate ignored phase requires actual reflinks and has no fallback.
It runs after tracked materialization and metadata insertion, before links.
Non-self link sources overlapping private roots (including absent descendants)
fail; a deliberate source-equals-target override is allowed. Lifecycle failure
rollback verifies creation ownership before removing selected private roots with
the strict CoW removal helper, which handles read-only directory modes without
touching the donor or unrelated content. The ordinary owned-checkout rollback
then removes the new directory, bookmark and metadata; it does not undo the repo.

Existing-only switches never load, compile or execute ignored-copy policy. Their
existing explicit link behavior remains unchanged. Native creation tests exercise
policy ownership/layers, exact base rules, nested matches and nonignored skips,
required real Btrfs sharing/private inodes, independent writes, tmpfs no-match and
strict-CoW failure, private-link conflicts, and post-payload read-only rollback.
Nested-destination checks cover direct paths, symlinked-parent aliases, payload
overlap, unchanged JJ operation IDs on rejection, and no-copy/no-match success.

### Local freshening commands and recovery

The existing [`reseed::prepare`](../src/reseed/prepare.rs) engine now accepts local
`[freshen]` policy with either one `command` argv or ordered `commands` argv arrays.
Presence of both is invalid even when empty; no global command policy exists.
The local loader retains `[prepare]` as a deprecated, warned alias and rejects
both table names together. This preserves frozen legacy policy bytes during
recovery instead of silently ignoring old settings or forcing a mid-flight edit.

Preparation journals v2/v3/v4 freeze a `command_sequence` containing `argv`, `completed`,
and `in_flight`. Before spawning a command, the engine durably marks it in flight.
After successful exit, it checks source ownership/policy, the same selected source
change and exact goal parent, the unchanged tracked tree (including new tracked
paths), disk materialization, and frozen targets before acknowledging completion.
Commands execute sequentially in donor cwd without an implicit shell, and command
stdout is forwarded to stderr so jw's own stdout stays reserved for its report.
Any failed command/proof stops before later commands or target refill. Final
source/payload proof capture remains the prerequisite for target planning, handoff,
or, for `prepare`, receipt finalization.

Resume skips durable completed steps; an in-flight step may retry after failure
or interruption, including a lost acknowledgement after successful side effects.
There is no exactly-once claim. During preparation, ignored outputs can be partial
and later steps may alter earlier outputs; the current step must tolerate that
state or the operator must repair it. Borrowing stays blocked until final proof
capture. Once verified, the existing frozen payload digest/restoration contract
continues to apply, without rerunning commands to adopt changed data.

V1 preparation records load compatibly into one-step sequences. Only `Verified`
proves legacy completion; other legacy progress is unknown and may retry. Migration
preserves frozen identities, policy bytes, source proof, and target plans. Lookup
and preview only migrate in memory; real progress writes v2. Unsupported versions,
malformed completion positions, or conflicting legacy/v2 command fields fail
closed. Target pending/receipt schemas are unchanged.

No commands means fetch/advance only, not a guessed build. Existing selected
ignored payload may still be verified, explicitly reported as existing rather than
freshly built. There is no automatic payload pruning. `prepare` reuses this engine
and ends at its source proof; `reseed --freshen` is removed, while legacy
preparation and removed-up batches use `reseed --resume` and retain their
original target policy. See
[operator configuration and retry guidance](reseed.md#prepare-the-source).

### Target history preservation foundation

[`TargetHistory`](../src/freshen/history.rs) implements the target-preservation
foundation of the [removed combined up](../design/workspace-reseed/up0.gpt6a.md#unpublished-target-preservation).
No command activates its planning anymore: `prepare` never allocates history
bookmarks, and ordinary reseed keeps its existing history behavior. The retained
semantics serve legacy v3 up journals, whose recorded `history` intent the
`reseed --resume` executor verifies create-only before clearing or advancing.

After snapshotting target work and proving its disk bytes, a removed-up
coordinator called
`TargetHistory::plan(client, operation, target_commit, fetched_goal,
no_freshen_bookmark, names)`. IDs are queried with explicit `commit_id(...)`
revsets, not symbols which could be shadowed by bookmarks. The effective work
tip is the sole parent of an ordinary undescribed, unconflicted, nondivergent
empty change; otherwise it is literal target `@`. Described empty changes and
empty merges remain work. Publication is ancestry/equality against the **selected
fetched goal**, not source `@`, the old remote tip, or equal file trees. Opt-out
still records this history assessment; it only suppresses the bookmark.

`BookmarkNames::at(client, operation, jiff::Timestamp::now())` supplies one shared
allocator per batch. Names use UTC `freshen-YYYYMMDDTHHMMSS.nnnnnnnnnZ`, with
`-2`, `-3`, ... on collision, including identical timestamps/clock repeats.
Existing local/remote names and deleted tracked relationships are reserved.
Before planning a new batch, the coordinator also called `reserve(name)` for names
in other active pending/preparation intents: those bookmarks may not exist yet.
Do not rebuild an allocator or replan when recovering frozen targets.

The existing target pending/receipt schema has an optional `history` field;
legacy absence stays absent on serialization. It holds target facts/operation,
work tip, exact upstream goal, publication assessment, and the chosen bookmark
name/exact tip. Preparation's atomic `plans` and immutable handoff comparison
carry the same field without adding permanent workspace tracking. The removed up
planner populated it **before** the batch plans were saved/handed off.
The executor verifies it matches the frozen old commit and new source/base goal,
durably saves intent, then ensures the pair before sparse clearing or advancement.
This also applies on later-phase recovery, before partial-tree reconstruction.
Source preparation can retry target planning before plans freeze, but cannot
create preservation bookmarks during that unfrozen stage.

Creation uses snapshot-disabled native **`bookmark create`**, never set/move.
Absent name creates at the exact stored tip; an already identical pair is a no-op
(including lost command success); another target, conflict, or remote collision
stops without reallocation or overwrite. Native create checks its live view too,
so a collision inserted between precheck and create is refused. A postcheck
requires the exact pair before allowing target mutation. JW's existing repository
operation lock serializes JW operations, **not arbitrary concurrent JJ writers**:
native concurrent operations can still create conflicts/multiple operation heads,
which stop the operation rather than being repaired. Pause writers; this is not
a cross-process compare-and-swap or a guarantee against later external movement.

No existing feature bookmark is moved. Legacy advancement recovery keeps its new
empty change on the frozen prepared baseline, not a rebase; even an already empty
old target gets a new empty change (recovery recognizes a completed new without
repeating it). A recorded opt-out leaves the old JJ history intact.
Focused native tests cover scratch/meaningful empty/merge publication, exact-ID
shadowing, deterministic collisions and reserved uncreated intents, a precheck/
create collision, serialization/replay, lost success, immutable handoff identity,
later-phase mismatch before repair, and opt-out history. The real-CoW executor
checks use disposable Btrfs repositories; the selected-fetched-goal case uses
only a disposable local Git remote in [`tests/freshen_source.rs`](../tests/freshen_source.rs).
Ten new focused cases and six relevant existing recovery regressions passed on
installed JJ `0.45.1-33d3a595`, with real CoW required for executor/preparation
checks; root format and all-target/all-feature Clippy checks passed. This is not
a rerun of the full JJ release matrix or unrelated suites.

### Native freshening source foundation

[`SourceSelection`](../src/freshen/source.rs) implements native upstream selection
for the `prepare` workflow, called by the shared coordinator before creating a
prepare journal. Only legacy v1/v2/v3 resume retains the earlier stored
remote/revset fetch behavior.

The caller snapshots real source work, verifies filesystem bytes, and supplies one
captured operation. Selection and validation use only frozen read-only JJ queries:

1. Derive source base `B` from the sole parent of ordinary undescribed empty `@`;
   otherwise use `@` itself. Described empty changes and merges are not disposable
   scratch. A directly published nonempty source is valid; an unpublished source
   or empty child above unpublished history is not.
2. An explicit source revset wins. Named remote symbols retain their identity even
   when other refs share the same tip. A tracked local symbol can identify its
   remote; other expressions must resolve to a unique recorded remote tip.
   An explicit not-yet-imported branch uses the narrower form described below.
   A configured remote constrains selection; it never silently redirects an
   explicitly named remote.
3. Otherwise collect all applicable tracked pairs. Both the remote and some local
   target alternative must contain `B`. A deleted local relationship or relevant
   remote conflict is retained as a blocker, not silently discarded. Exact and
   descendant local targets compete in the same tier; any ambiguity stops.
   Only when this tier is empty may a unique exact remote target at `B` win,
   including **untracked** refs. Synthetic `@git` refs never identify an upstream.
4. For a recorded remote, require the base, explicit goal, and selected tracked
   local target to be published there before fetch. An unrelated tracked branch
   whose local target descends from `B`, but whose remote does not contain `B`,
   is inapplicable.
   An applicable tracked pair with unpublished local work blocks rather than
   falling through. An untracked remote does not give its unrelated same-named
   local bookmark any authority, and that local bookmark remains untouched.

The serializable selection freezes names, tracking state, old target IDs, source
facts, and operation in the preparation journal. After the coordinator
fetches, validation rereads **the same ref**, requires old remote ancestry in the
fetched remote (not merely source-base ancestry), and rejects deletion, conflicts,
tracking changes, local unpublished work, or sideways/rewound goals. A tracked
local may lag the remote if it remains published; no repair is performed.
Explicit revsets are reevaluated as goals on that pinned line; a fixed ID can
deliberately stay fixed rather than implicitly becoming a bookmark-following rule.

For an **explicit not-yet-imported branch**, the supported form is unquoted
`branch@remote`, for example `introduced-later@origin` or
`feature/new-line@origin`. Branch components contain ASCII letters, digits, `_`,
internal `-` and `.`, separated by `/`; a component cannot begin/end with `.` or
`-`, contain `..`, or end in `.lock`. The remote uses one such component and must
already be configured in native JJ Git configuration. Synthetic `git` is excluded.
This deliberately narrow interface does not reinterpret quoted names, patterns,
operators, arbitrary revsets, or nonexistent local bookmark names as new branches.
Use explicit `branch@remote` rather than asking a missing local name to imply a
remote; pre-import unusually named branches with native JJ before selecting them.

Selection first asks native JJ whether the expression resolves. Syntax/query
errors and multiple results are not treated as absence. Existing unresolved or
deleted remote records remain blockers, including native tracked placeholders;
they are not reclassified as new branches. Existing known-ref conflict and
fast-forward checks remain unchanged.

Missing-ref intent serializes old remote ID, old goal ID, and tracking state as
**unknown (`null`)**, alongside the exact requested name/remote and full source
snapshot. Preview does not fetch or claim the goal is prepared. The adapter's
`fetch_bookmark` fetches that one exact branch from that one remote, not all
remotes or guessed/default branches. Native Git remote configuration is live,
not part of the frozen JJ operation; the coordinator must retain its existing
policy/identity revalidation boundary before mutation.

Publication may only become knowable after this fetch. Until then, potentially
local source history is preserved, not discarded as scratch. Afterward the same
source/base and fetched-goal publication checks must pass before any source
advancement or configured command. Still-absent refs, divergent source history,
and unpublished source work stop there. Native auto-tracking may import the new
line; any resulting tracked local conflict/unpublished work also blocks. With no
old remote ID there is **no prior fast-forward proof**; validation establishes
publication on the newly imported goal without inventing an old remote baseline.

The source must remain the same revision. One safe normalization is allowed:
native JJ can create an ordinary empty child above a directly published source
when that checkout becomes immutable during fetch. The original tree and history
are unchanged. Filesystem proof, ownership checks, and the exact accepted source
revision still belong to the coordinator; graph queries do not prove disk bytes.
Once the fetched goal is frozen, recovery must use it rather than selecting or
fetching again. There is no permanent following state, operation-log inference,
default-branch heuristic, or automatic bookmark movement.

[`tests/freshen_source.rs`](../tests/freshen_source.rs) exercises these boundaries
using disposable local Git remotes, including the exact-main/advanced-dev
ambiguity, untracked remote with unrelated local dev, unpublished empty-parent
history, conflicts, old-remote rewind, and native immutable-checkout normalization.
The original 14 cases passed on pinned JJ 0.39.0, 0.44.0, 0.45.1, and 0.46.0.
The missing-ref extension adds six focused cases: all 20 source cases pass on the
installed JJ 0.45.1 development build, with the six missing-ref cases separately
verified on official JJ 0.46.0. This extension did not rerun the full release
matrix or unrelated suites. No live source was mutated.
Those foundation checks concern graph/reference semantics. End-to-end combined-up
and reflink verification are recorded historically in the
[operator guide](reseed.md#combined-up-verification-historical); the prepare/reseed
split itself had not been runtime-verified at this documentation's baseline
(installed JJ `0.45.1-33d3a595`, Linux Btrfs).

### Metadata repair

`jw repair NAME --base REVSET (--bookmark BOOKMARK | --no-bookmark)` is the explicit
correction path for an existing readable managed record. `NAME` is literal and must
identify a workspace registered with JJ at one captured validation operation. The
replacement base resolves to exactly one revision at that operation; a requested
bookmark must already exist locally. The checkout path need not be readable or
present.

Repair replaces only `creation_base_commit_id` and `associated_bookmark`. It preserves
creation time, creation operation ID, and intended remote. The record is replaced
atomically only if it still matches the validated old record; validation failure,
write failure, or concurrent change leaves the old record intact. Repair does not
create a JJ operation, change commits or bookmarks, refresh a working copy, or
create a usable checkout. Missing records remain an adoption case, and corrupt
records remain a restore/manual-repair case.

## Workspace-link health

The default workspace owns `.jwlinks.toml` and `.jwlinks.local.toml`. The local file
overrides a shared entry with the same `source`. Each configured entry is evaluated
once per managed workspace. Its `source` is relative to, and confined within, that
receiving workspace; a relative `target` is resolved from the same receiver. Doctor
does not inspect unmanaged workspaces because they have no `jw` link intent.

The link classifier is shared with link application and recognizes four outcomes:

| Outcome | Meaning | Doctor result |
| --- | --- | --- |
| Satisfied | Source resolves canonically to target, including an ordinary path | `PASS` |
| Missing | Source is absent while its target exists, or a required target is absent | `FAIL` |
| Skipped | Optional target is absent and source is absent or the correct dangling link | `WARN`/`SKIP` |
| Conflicting | Source is occupied by an ordinary private path or a link to another target | `FAIL` |

An existing managed record whose workspace path is stale or missing still appears in
doctor's workspace checks; link inspection for that record is `SKIP` because the
receiving root cannot be read. This keeps the workspace failure visible without
guessing at paths.

## Interactive manager boundaries

The manager compares recorded bookmark targets and workspace publish tips with
resolved trunk independently. Only local ancestry establishes “in trunk”; a squash
merge is not inferred. Unmanaged workspaces have no authoritative association.
The existing JSON schema and CLI status contracts remain unchanged.

The UI renders immediately, then receives completed snapshots from one background
worker. Search, selection, and rendering do not start JJ processes. Explicit
inspection and removal preparation refresh named working copies; browsing does
not. Rendering is event-driven so idle frames do not disturb native terminal text
selection. The mouse-mode toggle releases terminal mouse capture without changing
keyboard controls. Generation IDs reject obsolete results. Repository-changing work is serialized
and must finish before switching or exiting; terminal state is restored on failure.

Removal previews list every target and its directory, associated local bookmark,
risks, and ignored content. Default/current workspaces and unsafe or unreadable
identities are blocked. Risky rows require explicit inclusion. For eligible rows
with unrecorded content, the preview requires a batch choice to delete that content
or skip those workspaces before execution. The final check compares filesystem identity,
workspace revision, metadata, bookmark targets, and ignored-file inventory with
the reviewed plan. Changed targets require another review. Independent targets
continue after failure, and outcomes describe any partial progress. A directory
removal is permanent; JJ history cannot restore unrecorded files.

Bookmarks associated with surviving managed workspaces are retained. The last
confirmed keep/delete choice is user-global state in
`$XDG_STATE_HOME/jj-waltz/ui.json`, falling back to
`~/.local/state/jj-waltz/ui.json`. It is separate from static configuration and uses
the existing atomic file writer. A preference-write failure is reported separately
and does not block a confirmed removal. Network and remote bookmark operations are absent.

Shell adapters reserve stdout for a selected destination through `--ui-path`;
the interface and diagnostics use stderr. Cancellation emits no destination.
The initial screen clear also uses stderr: Ratatui's cursor-preserving clear
queries the cursor through stdout and cannot be used on the captured path channel.

Keep Ratatui and its crossterm re-export inside the terminal adapter. Lifecycle
services own creation and switching; doctor supplies read-only health results.

## JSON schema version 1

Snapshot JSON from `list` and `status` uses a versioned envelope and writes only
JSON to stdout. Diagnostics and warnings go to stderr. `prepare --format=json`
currently serializes its source-only report directly, including `source_commit_id`,
without this snapshot envelope or a schema-version field.

```json
{
  "schema_version": 1,
  "command": "<command>",
  "repository": {
    "operation_id": "<operation id>"
  },
  "workspaces": [],
  "warnings": []
}
```

Within schema version 1, producers may add optional fields. Known optional values,
including a missing bookmark or creation base, remain explicit `null` fields.
Consumers must ignore unknown optional fields and treat unknown enum values as
unsupported data rather than guessing. Removing or renaming a field, making an
optional field required, or changing an existing enum value's meaning requires a
new schema version. Human output is not governed by this machine contract.

Mutation plans use the same versioned-envelope rule. ANSI escapes never appear
in JSON.

`doctor` deliberately uses a separate schema-versioned envelope. A repository
snapshot requires one resolved trunk, while doctor must serialize useful checks
when trunk resolves to zero or multiple revisions, metadata is corrupt, or a
workspace path is missing. CLI rendering completes before an unhealthy doctor
returns a failing exit status.

Doctor's `workspace-link` diagnostic code is additive within schema version 1. The
human renderer uses `PASS`, `WARN`, `FAIL`, and `SKIP`; machine output retains the
existing `passed`, `failed`, and `skipped` states and uses severity to distinguish
optional warnings from informational skips and errors. Consumers must ignore
unknown additive diagnostic codes and fields. `jw status` remains a single-workspace
snapshot and does not inspect link health.

Metadata repair is a human lifecycle command. It does not add a JSON schema or link
health field to `jw status`.

## Copy-on-write creation

Copy-on-write is the default creation mode. Creation probes the destination
filesystem before destination mutation and falls back to ordinary JJ workspace
creation with a warning if it cannot clone.
Otherwise it adds the workspace with empty sparse patterns, clones the files JJ
tracks in the source workspace's working-copy commit, and copies that source's
working-copy state. Ignored and untracked files are never cloned; the implicit
base refresh may first auto-track files according to ordinary JJ rules.
A JJ snapshot validates the copied state; if JJ rejects it, the previous state is
restored, the attempted clone workspace is rolled back, and JJ recreates it
through its ordinary workspace-add path, preserving the source sparse patterns.
On successful state adoption, `jj restore` then makes `@` match the creation base exactly before provenance is
captured, so the recorded creation operation describes a complete checkout.

Watchman or rejected copied state also trigger the existing ordinary-checkout
fallback. Each materialization fallback carries a reason in `AddResult`; lifecycle
outcomes collect it alongside filesystem-preflight warnings. The CLI renders these
on stderr, preserving shell path stdout, and the manager displays a warning report.
Ordinary-checkout fallback is not reported as CoW success. No strict-mode flag or
fallback-policy change is introduced here.

## Failure order

Creation preflights what it can, creates the workspace, applies required setup,
then records switching state. Failure removes any workspace, bookmark, links, and
directories created by that operation; cleanup failures are attached to the
original error.

Removal builds a plan before mutation. CLI callers choose whether associated
bookmarks should be deleted. Herdr removal keeps its provenance marker until the
container close reports success; a stale marker is safer than lost recovery data.

## Test surfaces

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked

cd plugins/herdr
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
```

JJ 0.39.0 is the oldest supported release. CI runs root integration tests against
0.39.0 and the newer pinned compatibility targets, 0.44.0, 0.45.1, and 0.46.0,
on Linux, macOS, and Windows. macOS tests require real copy-on-write success. The adapter
keeps version-specific behavior at the boundary: `jw doctor` recommends
`jj converge --no-interactive` for divergence on JJ 0.45 and newer, while older
supported versions receive manual recovery guidance. A minimum-version bump
requires a documented public JJ capability that cannot reasonably be adapted.
Newer JJ versions may work, but are not part of the declared window until the pin
is advanced. Herdr is checked separately because it is a separate Cargo
workspace.


### External checkout lifecycle

Workspace metadata schema 2 adds an optional `external_owner` with the actual linked
Git checkout root, Git admin directory, and common directory. Schema 1 manifests and
records remain readable. Adoption and registration inspect actual topology; repair
preserves ownership. Older or absent records still receive live topology checks.

CLI removal, manager preview and execution, pruning, and rollback revalidate ownership
before native `workspace remove` or `workspace forget`. An externally owned linked
Git checkout or remaining external Git registration is
ineligible for destructive cleanup; uncertainty fails closed. A native app owns its
checkout even after JJ adoption. Creation uses the shared
[mode resolver](#source-destination-and-creation-mode), defaulting to CoW/JJ-only.
Explicit `--colocate` or configured colocation requests an ordinary Git worktree
checkout; `--colocate` alone overrides default/configured CoW.
New colocated workspaces have jw ownership recorded through the same explicit
creation path; existing workspaces are never converted by a default change.
This fork's current defaults supersede v0.8.0's automatic colocation inheritance.
Explicit colocation still requires advertised JJ support and a Git backend.

Existing-directory removal probes `workspace remove --help` and uses native
snapshotting/removal when advertised (JJ 0.46+); minimum support remains JJ 0.39.
Keep-directory, missing-directory, rollback, and older-JJ cleanup retain guarded
forgetting. Both commands share the owned Git-link detachment and exact-registration
cleanup boundary, preventing JJ 0.46's global prune from touching unrelated missing
registrations. Workspace absence is verified even after command failure; native
removal never retries through custom deletion. A remaining directory is reported as
partial progress after bookmark/metadata cleanup, including JJ's warning-only failures.

`reconcile-external` is separate from automatic pruning. It verifies that the recorded
checkout and Git admin directory are absent and the live registration is gone,
rechecks immediately before forgetting any remaining JJ workspace, and removes only
the matching metadata. It neither deletes files or Git registration nor rewrites
commits or removes bookmarks. Doctor reports external ownership and routes cleanup
through the owning app before explicit reconciliation.
