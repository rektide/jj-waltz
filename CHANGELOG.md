---
type: Changelog
title: jj-waltz changelog
description: Release history and unreleased workspace lifecycle changes.
resource: /CHANGELOG.md
status: maintained
updated:
  by: model:zai-coding-plan/glm-5.3
  at: 2026-10-16
sources:
  - resource: "opencode://ses_ebcf7a55dffeC3pTCPy2RUXq4B"
    title: Removed remove-command bookmark prompting and --keep-bookmark in favor of keep-by-default
  - resource: "opencode://ses_ec10d60dbffencx7PCZnh1dnjg"
    title: Delivered up integration, source-only receipts, native fetch pinning, frozen target preservation and legacy resume compatibility
  - resource: "opencode://ses_ec1394461ffeEXBHM582vTs9VM"
    title: Delivered donor assertions/current-variant selection, corrected phase-specific preparation and subset frozen-target recovery, and fixed nested payload preflight
  - resource: "opencode://ses_ec1a8d43bffe02bXuEoA7hEW3l"
    title: Added local command/recovery and shared ignored-copy policy/selection foundations with explicit legacy and current-exposure boundaries
  - resource: "opencode://ses_ec206881bffenBcwNsQxlvfx3J"
    title: Implemented and verified explicit add/s destinations and source selection, then CoW/JJ-only/current-source creation defaults
  - resource: "opencode://ses_ec37b01d2ffe9ytO0jRsdmoyXA"
    title: Added reseeding, primary donor defaults, and explicit recoverable source freshening
  - resource: "opencode://ses_ec37a1a86ffeYXsS5j3A7Jng0t"
    title: Implemented wildcard groups and configurable aggregate unmatched diagnostics
  - resource: /docs/reseed.md
    title: One-off and batch reseeding, replacement policy, and recovery
---

# Changelog

## Unreleased

- Remove `jw remove` bookmark prompting and the `--keep-bookmark` flag: associated
  bookmarks are kept by default with no interactive ask, and only an explicit
  `--delete-bookmark` ever deletes them. Scripted removals can no longer stall on
  or accidentally answer a bookmark prompt.
- Add `jw up`: native pre-fetch upstream selection, exact-name/remote fetch,
  sequential local commands, and private CoW refill on new empty target changes at
  the exact fetched goal. Bare up in a variant updates it; at primary it prepares
  only the donor with a real receipt. Explicit targets/groups suppress current.
  Preserve unpublished target work with frozen collision-safe freshen timestamp
  bookmarks; `--no-freshen-bookmark` opts out without moving feature bookmarks.
  Reject unpublished source work. V3 preparation journals freeze workflow and native
  selection. Remove `reseed --freshen`; legacy v1/v2 `reseed --resume` retains original
  keep-target/explicit-base intent. No new following state or default-main inference.
- Add optional add/s `--source-at` as a snapshotted donor tracked-tree assertion,
  independent of target `--at`. Equal trees may have different commit IDs; the
  default target parent remains donor `@`. Creation inherits only local ignored-copy
  policy, not reseed's donor/assertion/freshening fields; existing switches skip it.
- Ordinary reseed uses CLI `--source-at` > local `source_at` > freshly snapshotted
  donor `@`. Bare reseed selects only the current registered variant; primary
  invocation requires names/groups. Explicit selection suppresses implicit current,
  even when a group matches nothing. Keep target work by default. Resolve
  current/pending identity without snapshots; resume
  uses recorded IDs. Complete original target order is required only while source
  preparation has no frozen target plans; frozen targets/subsets resume independently
  without refetch/build, and bare resume selects only the current target.
- Add/s now copy selected ignored payloads privately with real CoW independently
  of tracked checkout mode, reject private-link overlap, and clean up owned read-only
  copies on failure. Actual payload selection rejects nested donor destinations
  during preflight; disabled/no-match selections retain normal nested placement.
- Add opt-in root global `copy_ignored` and shared CLI > local > global replacing
  policy, preserving missing versus explicit `[]` and `--no-copy-ignored`.
  Compile patterns together for one eligible-root selection walk, accept trailing
  slash, and collapse nested roots. Skip/report normal nonignored source matches
  while still finding eligible descendants; retain hard selected-root source and
  target guards and real private CoW. Missing matches succeed. Current integration
  includes reseed/preparation and add/s new-workspace payloads.
- Add repository-local `[freshen]` single `command` or sequential `commands`
  argv, rejecting both forms even when empty. Retain `[prepare]` as a warned
  alias and preserve v1 pending preparation recovery. New v2 journals freeze
  sequence/progress, skip completed steps, and explicitly retry interrupted
  in-flight commands when needed; arbitrary command execution is not exactly-once.
  No commands means no guessed build, while existing selected ignored payload
  may still be copied and is reported as existing data rather than freshly built.
- Add shared `add`/`s` `--path DESTINATION` and optional `--from SOURCE`, defaulting
  to the current checkout. Honor registered names/paths; otherwise infer `fix` from
  `opencode-fix` under repository `opencode`, or keep a basename such as `myfix`.
  Name-only placement remains `repo.NAME` beside the verified JJ storage owner.
- Default new workspaces to CoW/JJ-only and an empty child of the source's current
  `@`, snapshotting before freezing the implicit base. Preserve explicit `--at`
  resolution. This supersedes v0.8.0's automatic colocation inheritance.
  Explicit CLI `--colocate` opts out of default/configured CoW; `--cow` overrides
  configured colocation. Both explicit positive CLI modes conflict; config
  `colocate=true` alone works, while both explicitly true settings conflict unless
  overridden. Ordinary-checkout fallbacks remain and now all report warnings,
  including rejected copied state and Watchman, in CLI and manager output.
- Add recoverable donor preparation (now activated through `up`): snapshot real
  work, fetch upstream, select a post-fetch source baseline, and run trusted local
  argv before distribution. Add prospective/no-action
  previews, source ownership checks, verified dependency payload binding, and
  recoverable atomic batch handoff with completion receipts. Source preparation
  blocks unsafe borrowing after failure; settings alone never activate it.
- Extend reseed groups with `names`, `name_globs`, and `path_globs`. Match exact
  registrations in the current repository, with optional `on_unmatched = "warn"`
  and `--on-unmatched error|warn` overrides. List all skipped unrelated candidates
  in one escaped warning; keep positional names and target safety failures strict.
  Preserve literal array groups and show selected roots/origins in dry-run output.
- Add `jw reseed NAME...` for repeatable in-place CoW refills of full JJ-only
  workspaces. Individual one-offs need no groups or configuration. Preserve
  feature revisions by default, or explicitly start children with `--at`;
  optionally replace named ignored dependency roots or all unrecorded contents.
  Include local source/group shortcuts, dry-run plans, verified public-JJ sparse
  prefill, and repository-scoped interruption recovery without private JJ state
  copying. CoW capability and ownership checks fail closed before clearing files.
- When `--from` and configured `source` are absent, reseed discovers the verified
  primary checkout of the current JJ repository as its donor. Explicit and local
  source settings retain precedence; optional tree assertions follow the precedence above.
- Prefer native `jj workspace remove` when the executable advertises it (JJ 0.46+),
  snapshotting dirty target changes before deletion while retaining ownership,
  manager review gates, unrelated Git registrations, and older-JJ fallback.
  Keep-directory and missing-directory cleanup continue to use guarded forgetting.
  Verify native partial progress, including warning-only directory deletion failures,
  without retrying through custom deletion. Minimum JJ remains 0.39.
- Explicit unsupported workspace colocation advises upgrading to JJ 0.46+
  or using `--no-colocate`; default creation is JJ-only.

## Version 0.8.0 (2026-10-09)

**Breaking change: new workspace colocation defaults.**

### Breaking change and migration

- When no explicit CLI or jw configuration override is present, new workspaces
  inherit colocation from a verified colocated primary workspace when its effective
  JJ `git.colocate` setting is true and JJ supports workspace colocation. Previously,
  new workspaces defaulted to JJ-only. Creation from registered JJ-only or colocated
  siblings now uses the same primary policy.
- To retain the previous JJ-only creation behavior, set `workspace.colocate = false`
  in jw configuration or pass `--no-colocate` for one command. Explicit CLI
  true/false overrides explicit jw configuration; omitted configuration inherits.
  Existing workspaces are not converted. Automatic CoW creation remains JJ-only;
  explicitly combining CoW and colocation remains an actionable error.
- Older supported JJ versions and unverified or non-Git primary topology continue
  to create JJ-only workspaces. No repositories are migrated or adopted automatically.

### Features and fixes

- Call Linux's exclusive rename syscall directly instead of requiring the
  `renameat2` libc wrapper, preserving atomic refusal to overwrite occupied
  destinations on both GNU and static musl builds.
- Build and exercise the static musl binary in PR CI before publication.
- Add opt-in copy-on-write workspace creation with `jw add --cow`,
  `jw switch --cow`, or `workspace.copy_on_write = true`. New workspaces clone
  the current workspace's tracked files on APFS, Btrfs, or XFS and match a full
  checkout of the creation base within the source workspace's sparse patterns.
  Unsupported filesystems, Watchman, and rejected copied working-copy state fall
  back to ordinary JJ workspace creation while preserving those patterns;
  `--no-cow` opts out for one command. Use a private, exclusively created directory
  for the reflink capability probe. Thanks to @gergesh for the contribution (#48).
- Add opt-in Git worktree creation with `jw add --colocate`, `jw switch --colocate`,
  and `workspace.colocate = true` on JJ 0.46+. `--no-colocate` overrides config.
  Track jw-created Git worktree ownership and clean only its registration; external
  checkouts and unrelated stale registrations remain protected.
- Reject combining copy-on-write and Git colocation, including when enabled in
  config; use `--no-cow` or `--no-colocate` to override one option.
- Protect externally owned Git checkouts and registrations during workspace
  removal, pruning, and rollback (#49).
- Explain whole-directory link conflicts with existing directories, including
  tracked files, and suggest a local self-target override or sharing only absent
  untracked paths while preserving refusal and workspace rollback.
- Correct Windows exclusive ownership operations, metadata initialization, path
  alias handling, and PowerShell switching and command execution.
- Test JJ 0.39.0, 0.44.0, 0.45.1, and 0.46.0 on Linux, macOS, and Windows,
  requiring real copy-on-write success on macOS. Update root and Herdr dependencies
  while retaining Rust 1.88 as the minimum supported version.

## Version 0.7.0 (2026-09-25)

- Add a persistent workspace manager with `jw ui` and bare `jw` in a terminal,
  with search, filters, multi-selection, workspace creation, and shell switching.
- Show bookmark integration separately from workspace work using local trunk
  ancestry, with explicit unknown, conflict, and unchecked states.
- Preview bulk removal and missing-workspace pruning, protect current/default
  workspaces, and require a choice before deleting files not recorded by JJ.
- Keep unmanaged bookmarks and bookmarks shared with surviving workspaces;
  report partial failures without blocking independent removals or retries.
- Add semantic terminal colors and a mouse/text-selection toggle (`m`), with
  readable default foreground text across terminal themes.
- Support configurable bare-command behavior and retain existing CLI/JSON commands.
- Raise the minimum Rust version to 1.88.

## Version 0.6.1 (2026-09-07)

- Preserve whitespace in checkout and workspace paths reported by `jw context`.
- Ignore inherited Git discovery ceilings when inspecting checkout paths.
- Skip the JJ routing-hint test when JJ is unavailable.

## Version 0.6.0 (2026-09-07)

- Add read-only `jw context [PATH] --format human|json` checkout discovery and
  verified Git-worktree-to-JJ-primary association, with Codex routing guidance.
- Exercise the compatibility matrix against JJ 0.45.1 and teach `jw doctor` to
  recommend non-interactive `jj converge` for divergent changes on JJ 0.45 and
  newer while retaining the manual recovery guidance for older supported JJ.

## Version 0.5.0 (2026-08-21)

- Make `jw doctor` inspect every configured workspace link for every managed
  workspace, reporting satisfied, missing, skipped, conflicting, and unreadable
  rules without hiding later diagnostics.
- Add `jw repair NAME --base REVSET (--bookmark BOOKMARK | --no-bookmark)` for
  safe, atomic repair of an existing workspace's managed base and bookmark
  metadata without changing JJ commits, bookmarks, operations, or checkouts.

## Version 0.4.1 (2026-08-15)

- Keep `jw doctor` machine-readable when jj-waltz configuration cannot be read
  or parsed, with a failed configuration diagnostic and skipped trunk check.
- Let `jw adopt --no-bookmark` ignore stale legacy bookmark markers while
  preserving the existing explicit bookmark association behavior.
- Correct architecture documentation to name JJ 0.44.0 as the newer tested
  compatibility target.

## Version 0.4.0 (2026-08-15)

- Add semantic `list --format=json`, `status`, `doctor`, and `adopt` commands
  backed by frozen, schema-versioned snapshots and repository-scoped lifecycle
  metadata.
- Diagnose corrupt, stale, and missing workspace state without silently replacing
  metadata or changing JJ history.
- Make workspace removal report partial progress and clean managed metadata even
  when directory deletion cannot finish immediately.
- Support legacy JJ repositories whose default workspace has no recorded path,
  restoring `jw list`, workspace creation, and the Herdr integration (#30).
- Test the compatibility window against JJ 0.39.0 and JJ 0.44.0.

## Version 0.3.1 (2026-07-12)

- Add a static x86_64 Linux release for older-glibc systems such as CerpacNAS.

## Version 0.3.0 (2026-07-12)

- Make workspace creation and switching transactional, including rollback of
  workspaces, bookmarks, links, directories, and switch state after failures.
- Plan removals before mutation and safely prompt for associated bookmark deletion,
  with explicit flags for scripts.
- Read link rules from the default workspace, reject escaping sources, and preflight
  every rule before changing the filesystem.
- Share one switching and completion policy across Bash, Elvish, Fish, PowerShell,
  and Zsh.
- Harden Herdr removal ordering and validate root, Herdr, and multi-version JJ builds
  independently in CI.

## Version 0.2.1 (2026-06-22)

- Automatically ignore machine-local workspace link configuration.
