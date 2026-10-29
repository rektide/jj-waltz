---
type: Guide
title: Reseed an existing workspace in place
description: Keep-history CoW reseeding, source-only upstream preparation, private payloads, and deliberate out-of-band recovery.
resource: /docs/reseed.md
status: maintained
generated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
updated:
  by: model:openai/gpt-6.1-sol#xhigh
  at: 2026-10-29
sources:
  - resource: "opencode://ses_e779f1a9affejTLMoRyE5Nv4xW"
    title: Cuts both public commands over to the reviewed two-route cohort, schema 1 and read-only PLANNED previews; retires the old driver and migrates public verification
  - resource: /design/coordinated-refill/public-contract0.gpt6a.md
    title: Accepted schema, outcome, timing and retirement contract
  - resource: /design/coordinated-refill/execution2.sol61x.md
    title: Reviewed two-route native implementation and reporting limits
  - resource: "opencode://ses_e8720e969ffeR1x2H40WZ36GY6"
    title: Implemented explicit reseed preparation, preparation-only lock scope, and honest prospective previews with native integration tests
  - resource: "opencode://ses_ead124fcaffedy7mtUij1h6iei"
    title: Integrated root-bound early/late visitors, shared seed/clear kernels and collision-aware preview without selecting a broader transfer default; tested existing-workspace donor-state convergence separately
  - resource: "opencode://ses_eb2158009ffeyJ0q8NuTd6g1RP"
    title: Integrated static checked streaming payload reconciliation, batch Linux topology guards and frozen persistent native ignore queries; revised refusal timing and convergence guarantees
  - resource: "opencode://ses_eb262a81effea1HgF0pPS3f1sy"
    title: Added navigation to the proposed inline-payload-guard design and its explicit preflight-timing tradeoff
  - resource: "opencode://ses_eb6ef3ef2ffe4155t844hpUnEL"
    title: Replaced the keep/reset reseed contract with donor-current sync and post-sync jj duplicate/rebase layering
  - resource: "opencode://ses_eb75a514cffe7jUOMWNZAIm3fX"
    title: Deleted recorded plans and progress recovery, retained live safeguards, and documented forward-only reseed and retry-from-start preparation
  - resource: "opencode://ses_eb867bfdaffefMVLpyaPUadbSp"
    title: Implemented and tested the reserved all group, donor-aware stable membership, unchanged readiness/bookmark guards, and literal recovery
  - resource: "opencode://ses_eb8801331ffeirQrF4GMFCxJHf"
    title: Implemented aggregate readiness, dirty keep/reset reporting, default-on bookmark coverage and opt-out, frozen proof rechecks, legacy recovery compatibility, and the fast journal/identity path
  - resource: "opencode://ses_eb899cdcdffecJnB7J1dSGs4ci"
    title: Replaced the combined up contract with source-only prepare, documented reseed-only worker semantics, legacy journal recovery, and the bounded tracked-tree verification split
  - resource: /design/up-performance/prepare0.gpt6a.md
    title: Command-split decision record — prepare is source-only, reseed keeps worker copying, performance first
  - resource: "opencode://ses_ec10d60dbffencx7PCZnh1dnjg"
    title: Integrated up with native source selection, source-only receipts, frozen preservation intent, legacy recovery, and native real-CoW verification
  - resource: "opencode://ses_ec1394461ffeEXBHM582vTs9VM"
    title: Implemented optional donor assertions, current-variant targeting, and phase-specific recovery with full preparation selection and independent frozen targets
  - resource: "opencode://ses_ec14d159effeiOep7U7f2iNQ7E"
    title: Connected ignored-copy policy documentation to delivered add/s payload integration and primary policy ownership
  - resource: "opencode://ses_ec1a8d43bffe02bXuEoA7hEW3l"
    title: Implemented local command/recovery foundations, then shared ignored-copy precedence, single-walk eligible selection, and frozen empty-selection compatibility
  - resource: "opencode://ses_ec37b01d2ffe9ytO0jRsdmoyXA"
    title: Implemented and verified reseeding, source defaults, and explicit donor freshening with recoverable preparation
  - resource: "opencode://ses_ec2e49e63ffe72f71k65378obu"
    title: Sol medium implemented explicit freshening and verified 23 native preparation/recovery cases on JJ 0.46
  - resource: "opencode://ses_ec2dd9386ffeDc0UNdIZu1bFbF"
    title: Independent source preparation, ownership, payload binding, receipt, and restoration review
  - resource: "opencode://ses_ec376c0e6ffewASuAvkLjXLTTU"
    title: Verified native sparse and symlink queries plus real-CoW integration across official JJ releases
  - resource: "opencode://ses_ec31c20c0ffeQNfMhRNezv4uuM"
    title: Implemented primary-source inference and verified 30 real-CoW integration cases on official JJ 0.46
  - resource: "opencode://ses_ec37a1a86ffeYXsS5j3A7Jng0t"
    title: Designed and implemented wildcard groups, strict or warn selection policy, and aggregate diagnostics
  - resource: /design/workspace-reseed/reseed0.gpt6a.md
    title: Prepared-source contract and stock-JJ sparse-prefill design
  - resource: /design/workspace-reseed/source-freshen0.gpt61sol.md
    title: Earlier fetch, source advancement, and dependency preparation design
  - resource: /CONTEXT.md
    title: Workspace identity, lifecycle metadata, and external ownership contracts
---

# Reseed an existing workspace

`jw reseed` renews copy-on-write sharing without replacing the workspace root,
`.jj`, registration, name, bookmarks, or lifecycle identity. Plain reseed uses the
existing donor without fetching or building. `jw prepare` fetches and prepares
only its donor and reports the exact prepared commit; `jw reseed --prepare`
explicitly runs that preparation once before planning and refilling targets.
The old `up` command and `--freshen` flag remain removed.

The update flow has three steps, in this order:

1. **Update the seed** — `jw prepare` fetches and builds in the donor.
2. **Sync a workspace to the seed** — `jw reseed NAME` makes the workspace's
   content be the donor's content: a fresh empty child of the donor's current
   commit, the whole tracked tree reflinked from the donor, and selected ignored
   payload copied. No feature logic runs inside reseed.
3. **Layer the feature back** — in jj, after the sync: `jj duplicate -r
   <bookmark> --onto <base>` keeps the old feature line and its dated bookmarks
   while copying it onto the synced base, or `jj rebase` moves commits and drags
   their bookmarks. Resolve conflicts in jj. This step is not jw's.

## One-off use

No groups, configuration, or donor assertion are required:

```sh
jw reseed --dry-run            # current registered variant
jw reseed my-feature           # literal target; primary requires explicit selection
jw reseed worker-a worker-b --from /path/to/prepared-source --dry-run
```

Remove `--dry-run` when you intend execution. Execution snapshots receiving work,
not the plain donor, and syncs onto actual donor WC commit **S** (or exact desired
parent **D** selected by `--at`). Every S tracked leaf is installed, including
equal paths, before native state adoption. Selected ignored
payload is reconciled incrementally; unchanged payload is not recopied.

Donor precedence is CLI `--from` → primary-local `source` → verified primary
checkout of the same JJ repository. CLI relative paths start at the caller;
configured relative paths start at the primary. The donor must name a workspace
root, not an arbitrary subdirectory. Caller identity is independent of donor
selection; `--from` does not change the process cwd or caller role.

The optional donor **tracked-tree assertion** is CLI `--source-at` → local
`source_at` → no live assertion. Plain input is assumed prepared; it is not an
implicit donor snapshot or payload verification. A supplied assertion resolves to
one exact commit whose tracked tree matches the donor. Equal trees need not have
equal commit IDs. A mismatch stops before target snapshots or mutations. The
assertion never changes the sync target.

`--at REVSET` is an optional explicit override of the target commit; without it
the target is the donor's current commit. `--at` creates a fresh empty child of
the selected base. It is **not a rebase** or bookmark move. Old tracked work
remains in JJ history and must be covered by a bookmark unless explicitly opted
out below. Names/groups suppress implicit current selection, even if they match
nothing. Empty selection never means all; maintenance names are literal, not
navigation tokens.

## Ignored data

Ignored/untracked target contents stay unless selected or explicitly erased:

```sh
jw reseed feature --copy-ignored node_modules --copy-ignored 'packages/*/node_modules'
```

Quote patterns. Components support `*`, `?`, recursive `**`, and trailing `/`.
Missing matches succeed and are reported; matching nonignored source directories
are skipped while eligible descendants remain searchable. Selected outer roots
include nested matches once. Selection never follows directory symlinks or enters
administrative directories. A selected root must be ignored under frozen donor
and desired rules and pass confinement/topology checks. Donor/desired tracked
overlap is an error; old target tracked files becoming payload are supported.

Selected roots are **mirrored incrementally** with private reflinks, not hard
links or shared directory symlinks. Target-only entries inside those roots are
deleted; unrelated contents outside them are not pruned. Symlinks are copied as links;
absolute links retain their literal spelling and may remain source-bound; they are
not rewritten. Dependencies reflect
the installed donor state. If feature manifests differ, run the project's frozen
install in the feature workspace.

Creation and reseed share bounded ignored-root selection semantics. Reseed uses
one contextual source-qualified cohort, not repeated single-target transfers: names outside the
selected regions (including native metadata and unrelated local data) are skipped
before child metadata/readlink; late decisions retain routing directories and
refuse routing file/link obstructions. The same finalized actions control writes
and descent. The existing `copy_ignored` inputs are a **bounded compatibility
adapter**, not permission to copy the whole donor. Empty lists and disabling
flags still copy no auxiliary data. No broader mirror/update default or new
preserve/mirror configuration is selected by this implementation.
These transfer exclusions do not change JJ's initial snapshot/auto-tracking
policy: a nonignored local file that JJ records becomes tracked work saved in
history, not an implicitly preserved untracked exception.

Comparison uses file type, size and exact mtime, or literal symlink text, not
content hashes. Clean payload preserves inode/mtime and requires no writes.
Mode-only changes and byte edits retaining size/mtime are **not repaired**;
this is not an archive-metadata/ownership/ACL/xattr sweep. Newly copied files
preserve permissions and mtime. Absent roots and selected target file/link roots
are supported; final target symlinks are replaced, never followed.

The default build reconciles payload serially. Experimental Cargo feature
`payload-parallel` uses one command-owned borrowed pool across all selected roots
and targets, capped at four available workers, with matching directory admission
and 256 speculative directories. JJ operations remain serial. The feature is a
measurement variant, not a new runtime option or the selected default.

`--replace-content` explicitly permits deleting all non-administrative target
contents, including `.env`, except `.jj`, before refill. It does not copy arbitrary
source dotfiles and has no further confirmation prompt.

### Ignored-copy policy layers

The root of `$XDG_CONFIG_HOME/jj-waltz/config.toml` (otherwise
`~/.config/jj-waltz/config.toml`) may contain:

```toml
copy_ignored = ["**/target", "**/node_modules", "**/dist/", "**/build/"]
```

Lists replace in **CLI explicit list → primary-local list → global list → none**
order. Missing local policy inherits; local `copy_ignored = []` does not.
`--no-copy-ignored` wins even over an explicit CLI list. `jw` never writes this
policy for you. There are no global build commands.

The same selector and policy apply to new `add`/`s` workspaces and reseed, not prepare.
Creation inherits only the local copy list, not donor or preparation settings.
Existing switches never refill payloads. Neither path hashes or persists a
payload inventory. Rules come from pinned tracked `.gitignore` files plus frozen
effective backend/global excludes. Reseed reuses compiled in-process frozen ignore
rules; copying/comparison validates entries inline. Creation retains its native
ignore oracle. See
[creation's private dependencies](../README.md#private-dependencies-on-creation).

## Prepare the source

```sh
jw prepare --dry-run
jw prepare --source-at 'dev@origin'
jw prepare --from /path/to/donor --format=json
jw reseed worker --copy-ignored node_modules
jw reseed worker --prepare --copy-ignored node_modules  # prepare once, then refill
```

Prepare is source-only from any registered workspace. It selects/fetches a pinned
upstream, selects an empty donor child of the exact goal, runs local commands,
performs one final tracked conformance check after the ordered argv, then reports
actual S independently of the fetched goal. It performs no payload selection,
census or verification. `--copy-ignored` and `--no-copy-ignored` are rejected on
prepare, not silently ignored. Installer stdout goes to stderr. It never selects,
snapshots, refills, or advances other workers, nor creates preservation bookmarks.
Native fetch may update refs. Reuse the reported commit without preparing again;
feature layering after a sync stays an ordinary JJ operation (`jj duplicate` keeps
the old line and its dated bookmarks; `jj rebase` drags bookmarks).

`reseed --prepare` uses the same effective donor path, upstream selection, local
commands as standalone preparation. `--source-at` is then an upstream goal; actual
S is handed directly to the same reseed command without an extra live assertion.
`--at` still independently overrides D. Target
selection and early structural checks precede preparation; the ordinary donor,
bookmark and native-input guards precede destructive refill. Payload/link checks
run in their execution phases. If preparation fails, no
target is snapshotted or refilled, and donor progress is left for deliberate repair.

With `reseed --prepare --dry-run`, no fetch, donor advancement, installer, target
snapshot, lock creation, metadata initialization or refill runs. The report says
**PLANNED — payload/path obstructions not inspected**, keeps S null, and records
prospective upstream selection separately. Receiver preservation and live assertions
remain pending. Static blockers still fail; a plan is never a readiness certificate.

### Native upstream selection

CLI `--source-at`, then local `source_at`, overrides automatic selection. Otherwise
pin one applicable native tracked pair before fetch; if none, one remote bookmark
exactly at the clean donor base, including untracked refs. Ambiguity asks for a
one-run override. There is no guessed main/origin, operation-log inference, or new
following preference. `[freshen].remote` constrains selection, never redirects an
explicitly selected remote. Fetch addresses only that pinned name/remote.

Unpublished donor work blocks, including local commits beneath empty scratch.
The selected remote must fast-forward. A not-yet-imported goal may use simple
`branch@configured-remote`; dry-run leaves its exact commit unknown. Fetch must
prove publication before source advancement. Prepare's `--source-at`, also used by
`reseed --prepare`, is an upstream **goal**, unlike plain reseed's tracked-tree assertion.

### Local commands

Keep `.jwreseed.local.toml` ignored in the verified primary:

```toml
source_at = "dev@origin"  # optional native-selection override
copy_ignored = ["node_modules", "packages/*/node_modules"]

[freshen]
remote = "origin"       # optional constraint
commands = [
  ["bun", "install", "--frozen-lockfile"],
  ["bun", "run", "build"],
]
```

For one program use `command = ["cargo", "build", "--locked"]`. `command` and
`commands` cannot both be present, even when empty. Every command needs a nonempty
argv/program; `commands = []` means no commands. Execution is sequential in donor
cwd, without an implicit shell, and stops on first failure. Later successful
steps may repair earlier tracked edits. Only the final tracked conformance check
must pass; identity guards still run between steps.
These are trusted programs, not sandboxed code.

**Commands must tolerate retry.** If prepare fails, fix the donor and run prepare
again. All commands may repeat, including earlier successful steps; upstream is
selected and fetched anew. Tracked installer edits require deliberate restoration.
No source progress is automatically rolled back, and no durable completion or
partial-install health is recorded. Do not borrow incomplete dependencies.

With no commands, only fetch/source advancement and tracked conformance run.
Ignored contents are **existing data, not freshly built or verified**. There is
no guessed installer or automatic pruning. `[prepare]` remains a deprecated warned
alias for `[freshen]`; both together are an error. Settings alone never activate
preparation. Prepare accepts no target names, groups, `--at`, or `--replace-content`.

## Optional shortcuts

```toml
source = "/path/to/prepared-source"
source_at = "dev@origin"
copy_ignored = ["node_modules"]

[groups]
workers = ["worker-a", "worker-b"]
```

```sh
jw reseed feature
jw reseed --group workers --dry-run
```

Array groups contain literal names, including strings such as `"worker-*"`.
Names and repeated groups can combine; selected workspaces are deduplicated.
The sync target is never configured: supply `--at` to override it for that run.

### All registered workers

```sh
jw reseed --group all --dry-run
jw reseed --group all --from /path/to/prepared-source
```

Built-in `all` needs no configuration and works from a worker subdirectory too.
It expands sorted registrations, excluding only the verified primary and effective
donor by canonical root identity, not the name `default`. Unsafe/missing/external/
colocated/sparse/conflicted/unbookmarked members remain BLOCKED, even under
`--on-unmatched warn`; any blocker prevents all refills. It neither adopts nor
prunes registrations. Local `all` definitions and empty worker membership fail.
Positional `jw reseed all` still means the literal workspace `all`.

### Wildcard groups

```toml
[groups.opencode]
path_globs = ["~/src/opencode-*"]
on_unmatched = "warn"

[groups.workers]
names = ["special-worker"]
name_globs = ["ocw-*"]
```

```sh
jw reseed --group opencode --dry-run
jw reseed --group opencode --on-unmatched error --dry-run
```

Only exact registered roots in this JJ repository match. Plain directories,
independent repos, and unrelated Git checkouts are not adopted or modified.
Table groups union `names`, `name_globs`, and `path_globs`. Patterns are case-sensitive
whole-name/path matches with `*` and `?` within a component. Recursive `**`, classes,
braces, backslash escapes, and `..` are rejected for groups. Leading `~/` expands
HOME; other relative paths start at the primary, with no environment/shell expansion
or symlink traversal. This differs from ignored-copy patterns, which allow `**`.

`on_unmatched` defaults to `error`; `warn` emits all escaped skipped candidates in
one stderr warning, even if later preflight fails. CLI overrides selected groups'
miss policy. Direct positional names and target safety failures stay strict.
Unknown/empty groups and invalid patterns always fail. Broken registrations are
blockers, not unrelated candidates to skip. No remaining targets is an error.

## Safety and recovery

Pause agents, builds, editors, monitors, and independent JJ writers on donor and
targets, including before previewing. The repository `operation.lock` serializes
only preparation (`jw prepare` and the preparation portion of `reseed --prepare`),
never preview, and is released on completion or error. Reseed planning
and refill hold no operation lock; plain reseed does not acquire it. The caller
must keep shared donors and targets quiescent across these operations. Full, conflict-free checkouts and JJ-only
targets are required. Primary/source/overlapping targets, external ownership,
nested repositories, mounted children, special files, and unverifiable identities
are rejected. Colocated donors are allowed; linked Git/app-owned donors cannot be
prepared. Real reflinks are required; there is no ordinary-copy fallback. Watchman
mutation is unsupported. Resolve stale state explicitly with JJ.

Linux reads `/proc/self/mountinfo` once per worker batch before recursive
selection/traversal, rejecting nested mounts including same-device directory/file
bind mounts and selected-root mounts. A mount at the workspace root is allowed;
obvious overlapping filesystem-relative bind aliases, covered/stacked ancestors,
and applicable autofs roots/ancestors are refused. Autofs traversal can change
topology without another writer, so it cannot use this fixed snapshot. Only root `.jj` and permitted donor `.git` regions, which are never
copied/deleted, are excluded. The guard conservatively covers retained regions
too, and fails closed when procfs topology is unavailable. Other platforms keep
their existing native boundary checks. Paths and mount topology must stay stable;
this is not protection against hostile concurrent path/mount replacement.

### Check readiness before refilling

`--dry-run` is read-only declaration planning: **PLANNED**, not READY. It resolves
profile, config, selection, bindings and recorded intent, but performs no live diff,
snapshot, fetch, builder, membership enumeration, clone probe, payload inspection,
transfer or lock/metadata write. Source results stay null. Static refusal is
`blocked`/nonzero; known bound rows stay planned unless an offending row is attributed.
Unresolved names never become synthetic failed receiver rows. History coverage
is checked after execution's preservation snapshots, not promised by a preview.

Normal execution does **not** audit payload interiors first. The checked streaming
comparison/copier/remover rejects administrative names and special entries even
when equal, but deep refusal or actual required-reflink failure can follow tracked
changes, earlier payload progress, or completed peers. The offending entry is not
acted on; there is no rollback. `--replace-content` is a separate returned broad
clear with its own progress, not a preview guarantee. Resolve the violation and use
the recovery instructions below before retrying.

Execution can snapshot real receiving work under JJ's ordinary auto-tracking
policy. Sparse/mid-refill targets are refused before preservation.
Unrelated data stays unless selected or erased
with explicit `--replace-content`.

#### Bookmarked work is required by default

A present, unconflicted named local or remote bookmark must cover the snapshotted
work tip or a descendant, in the default sync and `--at` modes alike. An older
ancestor/base,
equal tree, tag, or arbitrary snapshot does not count. Remote coverage uses JJ's
recorded view without fetching. An ordinary undescribed empty single-parent
scratch uses its parent; described empty changes and merges count themselves.
Execution records the saved O, work tip and covering bookmark; preview leaves them pending.

```sh
jj bookmark create my-feature -r <reported-work-tip>
jw reseed worker --dry-run
jw reseed worker --allow-unbookmarked --dry-run  # deliberate exception only
```

`jw` never creates or moves bookmarks for this guard. It captures coverage after
snapshots, checks the whole batch before execution, and checks each target again
before clearing. Forward bookmark movement on the same lineage is allowed;
deletion, conflict, or movement off it stops mutation. `--allow-unbookmarked`
bypasses only coverage, not other safety checks. Prepare is unaffected.

### How intact workspaces are verified

Verification is a bounded JJ tracked-tree check, not a per-file raw-byte audit:
snapshot with `fsmonitor.backend=none`, `snapshot.auto-track=none()`, and
`snapshot.auto-update-stale=false`, then require empty
`jj diff --from @ --to <pinned commit> --summary`. Equality uses JJ's normal
[tree-state/stat-cache semantics](https://github.com/jj-vcs/jj/blob/803ef00a0a6ee4ddc275d3138d5f9e7326399906/docs/technical/architecture.md#treestate).
A mismatch can be reported after recording a rejected edit. No additional files
are tracked; automatic stale-tree updates are disabled.

### Interrupted refills

Execution is one shared forward pass: resolve, source handoff, preserve all O/history,
capture shared S/D/O inputs, establish serial native receivers, run one ordinary
cohort, then loaded-link/native-identity finish. Both native routes use the same
reviewed provider and require successful **complete full-S coverage before adoption**:

- `--renewal-route=clear-then-populate` (default): sparse clear, unchanged-O and
  residual guards, new exact-D child, strict create-absent S install, D admission,
  captured-S state adoption, optional S→D restore and conformance.
- `--renewal-route=replace-authorized`: new same-O child with its own change identity,
  owned O→S renewal, D admission, S adoption, optional D restore, then receiving-only
  reparent to exact D and conformance. No clear-first fallback.

`--replace-content` independently authorizes broad non-admin clearing on either
route; installation after that clear is strict create-absent. D≠S may remove S
installs and materialize D leaves from the store. S install counters do **not**
prove final-D donor sharing. Commit-ID checks remain in memory. There is no durable
intermediate state and **jw never repairs a partial workspace**.

Only captured S `tree_state` is adopted after full installation; receiving checkout,
repository pointer, root and `.jj` identities are retained. Native clear/restore/
reparent effects are unenumerated, not fabricated clone/remove counts. Both routes
require the exact `jj 0.45.1-33d3a595429b7e3b54a8ea769560247913bc8a1b` profile.

Failures retain saved O/history/root, last entered phase and actual returned partial
effects. Native child coordinates are **last observed facts**, not a freshness
promise after a failed native process; reporting performs no post-error census.
An interrupted replacement can have full sparse patterns but incomplete contents.
Inspect the saved history and choose deliberate out-of-band recovery, for example:

```sh
# In the affected workspace, using the recorded covering bookmark:
jj new <bookmark>
jj sparse reset
# Then re-run the intended jw reseed command.
```

Do not run ordinary snapshotting commands on a half-filled target first. The next
reseed refuses non-full sparse selection without repairing or snapshotting it.
Full sparse alone does not prove recovery. If coverage was deliberately bypassed, use
the printed prior commit or recover through JJ's operation log instead. Successful
peers stay complete; the batch exits nonzero on any failure. A successful tracked
refill is not complete if ignored copying or links fail.

There are no journals, receipts, legacy readers, migrations, or `--resume` flags.
Leftover old files are ignored and left untouched, never automatically pruned.
Each new prepare/reseed resolves its own policy and source selection.

## Structured outcomes

Both commands accept `--format=plain|json` (plain default). Schema 1 lives in
[`schema/refill-v1.schema.json`](schema/refill-v1.schema.json); illustrative fixtures
live in [`app/cli/tests/fixtures/refill`](../app/cli/tests/fixtures/refill/).
Every normally returned parsed JSON command emits exactly one complete stdout
document, including early resolution failure, then exits zero/nonzero. Diagnostics,
builders and the original error chain go to stderr. Clap syntax/help and broken
stdout/process termination are outside this outcome guarantee.

- Dispatch on numeric `schema_version:1` and `command`; ignore additive fields.
  All counts, durations (integer milliseconds) and step indexes are canonical
  decimal **strings**; parse checked u128, never JavaScript floating-point numbers.
  Unavailable/unstarted values are explicit null, not successful zero reports.
- Paths contain `display` plus lowercase `unix_bytes_hex` of exact Unix bytes.
  Decode hex for identity. Display text and diagnostic messages are not reversible
  identifiers or machine classifiers.
- `.source.source_commit_id` is accepted actual S; `.source.upstream_goal_commit_id`
  is fetched goal. The old unversioned top-level `source_commit_id` meant goal and
  is removed. Default D is actual S, not goal. Recorded input, effective assertion
  and prospective upstream selection remain independently visible.
- Complete receivers require native establishment, ordinary disabled/satisfied,
  and finish complete. On failure, complete peers remain complete, started unfinished
  peers are partial, untouched peers not_started. A shared final donor error can
  coexist with every receiver complete. Preview status is planned or blocked.
- Effects are actual returned visible provider work, not native JJ effects or a
  post-effect census. Ordinary deviation and retention are independent totals.
  `observed.comparator` renders `family@version:hex-parameters`; observed data is the
  committed comparison prefix, not final equality or physical sharing proof.
- `capture.desired_manifests` counts prepared full-S manifests;
  `capture.owned_manifests` counts actual required distinct O indexes, not the
  provider's implicit empty ownership. Diagnostics appear once per native/ordinary
  source phase, not per receiver; retained gauges are return-boundary ownership,
  not peak RSS or raw syscall counts.
- `command_ms` excludes rendering; `engine_ms` spans resolution through final barrier;
  `pre_native_ms` includes source/preservation/capture; `ordinary_ms` is one cohort
  wall span. Native install/step, source/process and finish spans are measured at
  their own boundaries; overlapping spans must not be summed. Per-lane ordinary
  application time is unavailable (null), not derived by subtraction.

## Related material

- [Reviewed native checkpoint](../design/coordinated-refill/execution2.sol61x.md):
  both routes, coverage/adoption, optional borrowed pool and bounded native evidence.
- [Public contract](../design/coordinated-refill/public-contract0.gpt6a.md): exact
  field and status rules plus the performance adapter's separate acceptance binding.
- [CoW creation](../README.md#copy-on-write-workspaces): new workspaces, rather than refilling existing ones.
- [Architecture](architecture.md#in-place-reseeding): ownership and live safety boundaries.
- [Performance assessment](../tools/performance/README.md): retained fixtures and process-count gates.
- [Command-split decision](../design/up-performance/prepare0.gpt6a.md): why preparation and worker copying are separate.
- [Inline payload-guard design](../design/up-performance/inline-policy0.gpt6a.md):
  rationale and alternatives; the streaming integration now follows the
  [reviewed library seam](../design/up-performance/diffdirs-review0.gpt6a.md).
- [Streaming implementation](../design/up-performance/streaming0.sol61m.md):
  integration boundaries, native correctness checks and outstanding measurements.
- [Integrated native controls](../design/up-performance/integrated-controls0.sol61m.md):
  preserved serial/P4 build identities and bounded synthetic campaign evidence;
  not a claim about the primary opencode workload.
- [Actual-workload observations](../design/up-performance/primary0.sol61m.md):
  real imported root census and partial clean measurements; the operator cancelled
  the rest because of excessive fixture overhead and disk pressure.
- [Incident handoff](../design/up-performance/handoff0.glm53m.md): preserved measurement evidence.
- [Earlier reseed design](../design/workspace-reseed/reseed0.gpt6a.md) and [source design](../design/workspace-reseed/source-freshen0.gpt61sol.md): historical rationale, not the current recovery contract.
- [Wildcard-group design](../design/workspace-reseed/groups0.gpt61sol.md): selection rationale.

## Verification and limits

Current native tests use disposable local repositories/remotes and fake installers, never
operator workspaces. Real-CoW runs require `JW_TEST_REQUIRE_COW=1` on a reflink-capable
filesystem and check shared extents, private inodes, independent writes, ignored
replacement, symlink boundaries, read-only planning, bookmark guards, and partial
outcomes. [The native checkpoint](../design/coordinated-refill/execution2.sol61x.md#bounded-verification-receipts)
and [public execution account](../design/coordinated-refill/execution3.sol61x.md)
separate current evidence from historical measurements below.

### Historical checkpoints (not schema 1 or current latency evidence)

The explicit `--prepare` / preparation-only lock checkpoint covered 266 library,
preparation, reseed, and bookmark cases with real CoW required on native JJ
`0.45.1-33d3a595`. All passed across the main run and a focused rerun correcting
missing-store fixture assumptions and test-environment path/shim constraints.
The [preparation tests](../app/cli/tests/prepare.rs) check contention, lock release before
refill and after errors, single preparation, prospective previews, and source/policy
propagation. Formatting, all-target/all-feature Clippy (`-D warnings`), and maintained
document checks passed. Build verification used byte-identical source in scratch
with `diff-dirs` revision `3db9293`, its last pre-relocation layout: the shared
checkout's newer crate relocation no longer matches this manifest's existing paths.
That newer dependency layout was not adopted or verified by this checkpoint.

The journal-deletion checkpoint passed 258 native cases (library, reseed,
prepare, bookmark guards, and upstream selection) on Linux Btrfs with actual CoW
required. The separate CLI/colocation run passed 112 of 113 cases; the
`owned_cleanup_removes_only_target_and_keep_dir_preserves_files` failure also
reproduces with the preserved pre-cut executable (`workspace not found: missing`)
and is outside this change. Formatting and all-target/all-feature Clippy with
`-D warnings` passed; the existing Cargo manifest homepage warning remains.

The requested native synthetic fixture (8,555 files, 43 symlinks, two workers,
100 × 4,096-byte payload files) measured **5.93s** for reseed after journal
deletion, versus the supplied 6.64s baseline. A matched preserved-binary run
measured 7.46s before / 5.93s after; prepare measured 5.39s / 2.78s.
Instrumented 100 → 8,555-file scaling passed with zero JJ, Git, and
`file show` call growth. See
[historical guide recovery](../tools/performance/README.md#verified-checkpoint-and-development)
for the earlier measurement evidence.

A follow-up pass removed payload hashing and stat-stamping entirely (creation
digest/stamp walks, the untracked-`.gitignore` tree hunt, double-walk cloning,
and the per-target symlink scan): 269 native cases passed under the same
requirements, `jw add --copy-ignored` dropped from 3.56s to 1.53s on a
30,000-file/123 MB payload, and the same fixture measured **5.08s** reseed and
2.42s prepare. That pass responds to `jw add`/`reseed --dry-run` timeouts
against a large real `node_modules` donor.

Historical release-matrix results predate this deletion; they are not a new matrix
verification. Watchman mutation, sparse targets, and colocated targets remain
unsupported. Native synthetic timings are measurements of that fixture, not a
representative opencode throughput guarantee.
