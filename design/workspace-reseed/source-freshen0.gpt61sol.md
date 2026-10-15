---
type: Design
title: Freshen a reseed donor before distributing its tree
description: Explicit source fetch, baseline advancement, and configured dependency preparation, followed separately by opt-in default activation.
resource: /design/workspace-reseed/source-freshen0.gpt61sol.md
status: proposed
generated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
updated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
extensions:
  ticket: jjwaltz-reseed-freshen
  followup_ticket: jjwaltz-reseed-freshen-default
  epic: jjwaltz-reseed
  execution: explicit freshen implementation commissioned; default-on activation remains a separate follow-up
sources:
  - resource: "opencode://ses_ec37b01d2ffe9ytO0jRsdmoyXA"
    title: User confirmed freshening scope and separate default activation, then commissioned explicit freshen implementation with Sol medium
  - resource: /design/workspace-reseed/reseed0.gpt6a.md#prepare-the-source
    title: Existing manual fetch, JJ new, and frozen dependency installation workflow
  - resource: /docs/reseed.md
    title: Current donor precedence, explicit baseline assertion, target preservation, and frozen recovery
  - resource: /src/reseed/engine.rs
    title: Current planning, source validation, operation lock, and pending-record boundaries inspected before design
  - resource: /src/reseed.rs
    title: Current request API and selection diagnostics; freshen is not implemented
  - resource: /src/jj.rs
    title: Shell-free JJ adapter and frozen query boundary
---

# Freshen the donor, then reseed

## Situation and confirmed direction

The operator often wants to prepare `~/a/a/opencode` and distribute its tracked
tree and installed dependencies to existing workspaces in one invocation.
Today those are separate actions: fetch, select a clean source baseline, install
dependencies, then `jw reseed`.

The initial commission was **a design document followed by tickets, not execution**.
The human subsequently authorized Sol medium to implement the explicit feature.
Default-on configuration remains a separate follow-up. Confirmed freshening scope:

1. Fetch the chosen remote.
2. Advance the source to a fresh empty JJ child of the chosen baseline, preserving
   earlier tracked work in history.
3. Run a dependency preparation command declared in configuration.
4. Deliver explicit `--freshen` first. Default-on configuration lands separately,
   afterward.

The interfaces and recovery details below are recommendations, not claims that
these options already exist. No source checkout was fetched, reset, installed,
or reseeded to produce this document.

## Terms and boundaries

- **Source freshening:** fetch and prepare the donor for use. It changes the source
  checkout and may use the network; it is not merely status refresh/snapshotting.
- **Source goal:** the exact commit selected by `--source-at` or configured
  `source_at`, resolved in the source after fetching.
- **Preparation command:** a trusted, explicitly configured argv executed in the
  donor root, such as `bun install --frozen-lockfile`.
- **Preparation pending record:** write-ahead source intent and progress, separate
  from target refill records. An unfinished dependency installation is not a
  prepared donor even if its tracked files already equal the goal.

`--from` still chooses the donor: explicit path, configured source, then verified
primary checkout. Freshen acts on **that chosen donor only**. It does not always
modify a hard-coded opencode path or freshen both the primary and an override.

Source advancement and target placement remain separate. `--freshen` does not
imply target `--at`, a feature rebase, bookmark publication, or content wiping.

## Explicit interface: first delivery

Proposed use from a sibling workspace:

```sh
# Freshen the prepared donor but preserve ocw2's feature revision.
jw reseed ocw2 --from ~/a/a/opencode --source-at 'dev@origin' \
  --freshen --copy-ignored node_modules

# Also reset reusable worker slots onto the updated baseline.
jw reseed ocw1 ocw2 --from ~/a/a/opencode --source-at 'dev@origin' \
  --freshen --at 'dev@origin' --copy-ignored node_modules

# The same donor can be inferred when it is the verified primary checkout.
jw reseed ocw2 --source-at 'dev@origin' --freshen --dry-run
```

Normal reseeding remains offline without `--freshen`. In the first delivery,
declaring preparation settings alone must **not activate** source freshening.

Proposed extension to primary-owned `.jwreseed.local.toml`:

```toml
source = "/home/rektide/a/a/opencode"
source_at = "dev@origin"
copy_ignored = ["node_modules", "packages/*/node_modules"]

[prepare]
remote = "origin"
command = ["bun", "install", "--frozen-lockfile"]
```

Keep this machine-local policy ignored. `prepare` is a settings table, not an
activation switch. It allows generic projects to declare their package manager
without baking Bun or opencode paths into `jw`.

Recommended rules:

- Fetch only `prepare.remote`; use `origin` when absent and actually configured.
  Missing or invalid remote selection fails before source advancement. Do not
  guess a remote by parsing arbitrary revsets, fetch every remote, or silently
  continue after a failed fetch.
- `command` is a nonempty argv array, not an implicitly shell-evaluated string.
  Run it in the canonical donor root with ordinary project environment and visible
  diagnostics. An explicitly requested shell can still be an argv program, but
  no shell is inserted automatically.
- A preparation command is required when freshening will distribute selected
  ignored dependency roots. Without one, fail before freshening rather than
  describe old installed contents as refreshed. With no ignored payload selected,
  a missing command means tracked-source preparation only, reported explicitly.
- Configuration is trusted local execution policy. Fetching a project and running
  its installer may execute project/package scripts; `jw` is not a sandbox.
- An explicit `--source-at` is a preparation goal under `--freshen`, rather than
  an assertion that the **old** donor already has those files. After preparation,
  the usual exact tracked-tree assertion applies again.
- Without `--freshen`, the current strict source assertion is unchanged. Making
  `source_at` optional for ordinary borrowing is a separate discussion, not
  included in these two work items.

## Preparation and freeze order

The source changes before the final immutable reseed plan, not behind that plan's
back. Proposed sequence:

1. Read source-selection and preparation policy once. Discover explicit/group
   selection and repository identities. No eligible targets means no freshening.
2. Acquire the existing repository-scoped reseed lock. Validate identities,
   caller/source/target overlap, ownership, pending records, monitor restrictions,
   full-checkout support, and actual CoW capability before freshening. Require
   quiescent writers in donor and targets; the lock does not isolate editors or JJ.
3. Snapshot real source and selected target work before fetch can affect repository
   graph state. Preserve the source's old WC commit/operation and root/admin
   identity in preparation intent. Do not inspect a half-filled pending target
   using ordinary snapshotting commands.
4. Fetch the declared remote through `JjClient`. Resolve the source goal exactly
   once afterward. A failed fetch never substitutes a cached/stale ref.
5. Record the exact goal before selecting a new source WC. Use ordinary JJ `new`
   at that goal; keep root and `.jj`. An existing empty single-parent child of
   that exact goal may be reused, but matching tracked bytes do not skip fetch or
   dependency preparation.
6. Execute the configured preparation argv in the donor, with write-ahead progress
   recorded before starting it. No `jw`-initiated ignored-data prune or filesystem
   replacement is hidden in this step; the declared installer owns its effects.
7. Snapshot and verify that the source tracked tree still equals the exact goal.
   Successful installer exit is not enough: lockfile/manifest edits, conflicts,
   failed sparse materialization, or unexpected source changes fail preparation.
8. Validate the now-current shared links and final ignored-root rules. Capture
   one final operation for all donor and target reseed facts, exact IDs, source
   payload proof, and target graph positions. Fail closed if fetching/preparation
   made a target stale or changed it unexpectedly; do not silently update-stale.
9. Execute the existing reseed backend from that frozen plan. Keep or explicitly
   reset each target according to `--at`; retain current per-path clone safety,
   metadata, ignored replacement policy, and target recovery semantics.

Donor/target tree differences are normal and still handled per path. Freshening
does not require every target feature tree to become the donor tree.

Resolve target `--at` at final capture, after source preparation; it does not
implicitly become the source goal. Prefer explicit named baselines in recipes:
when the caller is itself the donor, `--at @` observes its post-freshen WC.

JJ owns fetch's normal remote-ref and tracked-bookmark effects. `jw` must not add
bookmark moves, rebases, publishing, or repository-wide undo. Preserve lifecycle
creation fields and associations; a preparation receipt records this action
separately from historical creation metadata.

Freshening is a donor mutation, not just borrowing files. First delivery should
refuse app-owned linked Git donors unless their mutation ownership is explicitly
supported; ordinary colocated primary donors remain useful. Do not convert or
assume ownership of native-app checkouts as part of this feature.

## Dry-run, errors, and interrupted preparation

`--freshen --dry-run` shows donor, remote, prospective source goal, preparation argv
and cwd, selected targets, keep/reset intent, and ignored replacement scope.
It must not fetch, switch the source WC, install dependencies, or refill files.
Existing real-work snapshot behavior must be disclosed. A remote-dependent goal
after the unperformed fetch is **prospective**, not a verified post-fetch commit.

Before target filesystem mutation, preparation is a **batch prerequisite**:
fetch, source-selection, installer, and tracked-tree failures stop distribution.
Report completed source actions and intact targets accurately; do not claim the
source was rolled back. Previously snapshotted work remains in JJ history.

Source preparation needs its own recoverable progress:

- Persist source identity, old WC, remote, exact goal once resolved, command argv,
  relevant configuration, and phases such as `fetching`, `selecting`, `preparing`,
  and `verified`, outside source/target replaceable contents.
- Block ordinary reseeding from a donor with unfinished preparation, even when
  its tracked tree is clean. An interrupted installer can leave ignored data
  incomplete without affecting tracked-tree equality.
- Proposed `jw reseed NAME... --resume` resumes recorded preparation before target
  refill. It uses the exact stored goal once available, not today's newly advanced
  remote. Before a goal was frozen, a successful re-fetch may establish it;
  afterward, do not fetch again and silently change the recovery target.
- Recognize a lost successful `jj new` result without creating duplicate changes.
  A failed/interrupted preparation command is rerun only against the recorded
  source goal and policy. Frozen install commands must be restartable; arbitrary
  external effects of a user command are not transactional or exactly-once.
- `--resume` never adds `--freshen` to an existing frozen target refill. Once
  target records exist, reuse the completed source preparation they reference.
- Failed target refills retain existing per-target pending records; successful
  independent targets may still complete. Never rerun source preparation halfway
  through an already frozen batch or use global JJ undo for recovery.

Detailed journal schema and multi-batch receipt linkage are implementation
choices to review before coding. The invariant is firm: partially prepared ignored
dependencies must never be presented as a verified donor.

### Accepted implementation checkpoint

The implementation uses a source-keyed preparation **batch** journal. It stores
all frozen target plans atomically before handing them to individual target
pending records; preparation IDs link those records and their completion receipts.
This keeps later targets' frozen intent recoverable after a partial handoff.

New freshening must not invalidate any unfinished target that depends on that
donor, including an ordinary older reseed pending record outside the selected
batch. Existing records or receipts from another preparation are never overwritten
or mistaken for completion. Preparation policy is frozen before source mutation;
tracked shared-link configuration is read after source checkout, because the
chosen baseline may legitimately change it.

## Default-on configuration: second delivery only

After explicit `--freshen` is implemented and verified, a separate change may add
this **root-level** local policy:

```toml
source_at = "dev@origin"
freshen = true
copy_ignored = ["node_modules", "packages/*/node_modules"]

[prepare]
remote = "origin"
command = ["bun", "install", "--frozen-lockfile"]
```

Default-on source mutation is explicit persistent opt-in, not the global default.
Proposed companion `--no-freshen` bypasses it for one ordinary reseed invocation.
The effective policy is explicit `--freshen`/`--no-freshen` > configured `freshen`
> false, with mutually exclusive CLI flags and real presence tracking. A parser's
default false must not accidentally override configured true.

Neither activation setting may imply `--at` or `--replace-content`. Configuration
must not make resume fetch or re-prepare an already frozen source. Dry-run exposes
that config would activate network/preparation work. Old configurations remain
non-freshening unless the new property is deliberately set.

The default-on ticket depends on completion of the explicit-feature ticket.
Settings for the installer can land in the first; **automatic activation cannot**.

## Acceptance and delivery anchors

### `jjwaltz-reseed-freshen`

- Explicit flag executes one fetch/advance/preparation sequence per selected batch;
  no flag means no new network or source mutation.
- Explicit/configured/inferred donors work from sibling and subdirectory callers;
  source-only invocation, empty/all-skipped selection, source targets, unsafe
  ownership, pending state, and unsupported CoW fail before freshening.
- Native fixtures with a disposable local remote demonstrate a moved baseline,
  preserved old source/target work, exact new source parent/tree, unchanged roots,
  and keep-mode feature differences versus explicit target reset.
- A fake declared installer demonstrates cwd/argv preservation, ignored payload
  readiness, failures, tracked lockfile mutation, and absence of shell interpolation.
  No network fetch or package install runs against the operator's real opencode.
- Dry-run invokes no fetch/new/install/refill. Failure at each preparation boundary,
  including lost `new` results and partial ignored installs, remains recoverable
  without snapshotting partial targets or borrowing unverified dependencies.
- Freeze/journal consistency, changed config/source, stale targets, real reflinks,
  and supported JJ versions are tested. Help and docs distinguish source
  preparation from target reset; no throughput or exactly-once command claim.

### `jjwaltz-reseed-freshen-default`

- Begins only after the explicit flag and preparation recovery are verified.
- Default remains false; configured true activates preparation with no CLI flag.
- CLI opt-in/opt-out precedence and mutual exclusion are covered, including
  configured false, missing configuration, `--dry-run`, and frozen `--resume`.
- Configured activation does not change target placement, replacement scope,
  ordinary source assertions, source selection, or unrelated workspace ownership.
- Documentation explicitly states the newly opted-in network/source/installer
  effects and shows the one-run opt-out.

## Related work

- [Original reseed proposal](reseed0.gpt6a.md#prepare-the-source): this adds an
  opt-in coordinator for the source preparation that was deliberately manual.
- [Current operator guide](../../docs/reseed.md): still authoritative for shipped
  behavior; it must not claim `--freshen` exists until implementation lands.
- [Architecture](../../docs/architecture.md#in-place-reseeding): keep CLI thin,
  route all JJ through `JjClient`, and keep source preparation outside the already
  frozen target materialization plan.
- [Wildcard groups](groups0.gpt61sol.md): selection and one-line unmatched policy
  are independent work. Freshening must not run when selection has no eligible
  targets; do not couple these deliveries or weaken positional-name strictness.
