---
type: Guide
title: Reseed an existing workspace in place
description: One-off and batch copy-on-write reseeding, explicit replacement policy, and interrupted-operation recovery.
resource: /docs/reseed.md
status: maintained
generated:
  by: model:openai/gpt-6.1-sol
  at: 2026-10-15
sources:
  - resource: "opencode://ses_ec37b01d2ffe9ytO0jRsdmoyXA"
    title: Implemented and verified reseeding, clarified safe snapshots, and documented primary-source inference
  - resource: "opencode://ses_ec376c0e6ffewASuAvkLjXLTTU"
    title: Verified native sparse and symlink queries plus real-CoW integration across official JJ releases
  - resource: "opencode://ses_ec31c20c0ffeQNfMhRNezv4uuM"
    title: Implemented primary-source inference and verified 30 real-CoW integration cases on official JJ 0.46
  - resource: "opencode://ses_ec37a1a86ffeYXsS5j3A7Jng0t"
    title: Designed and implemented wildcard groups, strict or warn selection policy, and aggregate diagnostics
  - resource: /design/workspace-reseed/reseed0.gpt6a.md
    title: Prepared-source contract and stock-JJ sparse-prefill design
  - resource: /CONTEXT.md
    title: Workspace identity, lifecycle metadata, and external ownership contracts
---

# Reseed an existing workspace

`jw reseed` replenishes copy-on-write sharing in existing JJ workspaces. It keeps
the workspace root, `.jj`, registration, name, bookmarks, and lifecycle metadata.
It neither fetches upstream changes nor installs dependencies.

## One-off use

**No groups or configuration are required.** Prepare the primary checkout—the
repository's original checkout—at your chosen baseline, then run from any sibling
workspace and name the workspace to refill:

```sh
jw reseed my-feature --source-at 'dev@origin'
```

`--from` defaults to the verified primary checkout of the current JJ repository,
not the current feature workspace or an arbitrary directory under `~/archive`.
An explicit `--from` or configured `source` overrides that default. For a separate
prepared source in the same repository:

```sh
jw reseed my-feature --from /path/to/prepared-source --source-at 'dev@origin'
```

By default, the target keeps its snapshotted working-copy revision, including
feature differences. Unchanged paths are reflinked from the prepared source;
JJ materializes paths that differ. A repeated run renews sharing even when file
contents have not changed.

Source and target revisions do **not** need to be identical: each unchanged path
can be reflinked independently. `--source-at` is a different check: it asserts the
baseline you prepared in the source. If that assertion does not match, reseeding
stops before clearing a target rather than silently ignoring it.

To put a reusable workspace onto a new empty child of an explicit base instead:

```sh
jw reseed my-worker --from /path/to/prepared-source \
  --source-at 'dev@origin' --at 'dev@origin'
```

`--at` is explicit reset behavior, not a rebase or bookmark move. Existing tracked
work is snapshotted first. It remains in JJ history rather than being copied into
the new child. An already empty child of the same base can be reused.

Multiple positional names select a batch. Selection is always explicit; an empty
selection never means all workspaces. Names are literal, not navigation tokens.

## Ignored data

By default, other ignored or untracked contents stay in the target. To replace
specific ignored dependency directories with private CoW copies:

```sh
jw reseed my-feature --from /path/to/prepared-source --source-at 'dev@origin' \
  --copy-ignored node_modules --copy-ignored 'packages/*/node_modules'
```

Quote patterns so your shell does not expand them. Selections must be ignored in
the source and target and cannot overlap tracked files or administrative paths.
Directory patterns support literal components, `*`, `?`, and recursive `**`;
they never traverse symlinked directories.
Missing matches are reported as skipped. Selected roots are **replaced**, so old
entries disappear. They are not hard links or shared-directory symlinks: edits in
one checkout do not alter the other.

Dependencies reflect the prepared source's installed state. If a feature changes
package manifests, run that project's frozen install in the feature workspace.
Symlinks are copied without traversal; source-bound absolute links are reported
because they are not a self-contained dependency snapshot. `jw` does not rewrite
package-manager links.

`--replace-content` explicitly authorizes deleting other unrecorded target
contents, including `.env`, except `.jj`, before refilling. It does not copy
arbitrary source dotfiles. There is no additional confirmation prompt.

## Optional shortcuts

To avoid repeating source arguments, put `.jwreseed.local.toml` in the default
workspace and keep it ignored:

```toml
source = "/path/to/prepared-source"
source_at = "dev@origin"
copy_ignored = ["node_modules", "packages/*/node_modules"]

[groups]
workers = ["worker-a", "worker-b"]
```

This permits either individual or group use:

```sh
jw reseed my-feature
jw reseed --group workers
jw reseed --group workers --at 'dev@origin' --dry-run
```

Array groups remain lists of literal workspace names; even `"worker-*"` in an
array is literal. Positional names and groups can be combined; duplicates are
removed. CLI source arguments override configuration. Repeated `--copy-ignored`
flags replace the configured selection;
`--no-copy-ignored` disables copying it. Reset behavior cannot be configured:
`--at` must be supplied for that run.

### Wildcard groups

Use a group table to match workspace names or filesystem locations. For example,
when `~/src/opencode-*` also contains unrelated projects:

```toml
[groups.opencode]
path_globs = ["~/src/opencode-*"]
on_unmatched = "warn"
```

```sh
jw reseed --group opencode --dry-run
jw reseed --group opencode

# Require every group candidate to match for this run:
jw reseed --group opencode --on-unmatched error --dry-run
```

Only exact registered workspace roots **in this JJ repository** are selected;
directory basenames need not equal JJ workspace names. Plain directories,
independent JJ repositories, and unrelated Git checkouts are not adopted or
modified. With `warn`, all skipped candidates appear together in one escaped
stderr line, without truncation:

```text
warning: skipped 2 unmatched group candidates: "/home/me/src/opencode-notes" (not a workspace of this repository), "/home/me/src/opencode-tools" (not a workspace of this repository)
```

Group tables support `names` (literal names), `name_globs` (registered workspace
names), and `path_globs` (root locations). Lists are unioned; repeated groups and
selectors run each workspace only once. For instance:

```toml
[groups.workers]
names = ["special-worker"]
name_globs = ["ocw-*"]
```

Patterns are case-sensitive whole-name or whole-path matches. `*` and `?` match
within one path component. Recursive `**`, character classes, braces, backslash
escapes, and `..` path components are rejected. A leading `~/` expands the home
directory; other relative paths start at the configuration's default/primary
workspace, regardless of the caller's workspace or subdirectory. There is no
environment-variable or shell expansion. Symlinks are not traversed, including
in a pattern's prefix. Files and symlinks are reported as non-directory misses.
This group syntax is separate from `copy_ignored`, which still supports `**`.

`on_unmatched` defaults to `"error"`. A group can opt into `"warn"`, or
`--on-unmatched error|warn` overrides all selected groups, including legacy
arrays. The policy covers unknown group-member names, unmatched expressions,
and paths that are not workspace roots of this repository. Direct positional
names are **always strict**. Unknown/empty groups and malformed patterns always
fail. Strict selection misses abort before source or target snapshots; warning
skips are reported before later source/target preflight, even if it fails.

A broken registered workspace is not an unrelated checkout. Missing/stale roots,
inconsistent identity, primary/source selection, external ownership, unsupported
colocation/sparsity/monitoring, pending recovery, I/O errors, and CoW failures
remain errors under `warn`. Path selection fails closed if a registration's root
cannot be resolved at all. If no targets remain, the command fails with
`no workspaces selected; nothing reseeded` rather than selecting everything or
claiming success. Dry-run shows selected names, paths, and selector origins.
Resume requires literal names and rejects groups or `--on-unmatched` overrides.

## Safety and recovery

Pause agents, builds, editors, and independent JJ processes that write the source
or targets. The repository-scoped advisory lock serializes reseed callers only;
it does not isolate arbitrary writers.

The first version requires full, conflict-free checkouts and JJ-only targets.
The primary checkout, prepared source, colocated/external targets, nested
repositories, mounted children, unsupported special files, and unverified
identities are rejected. A colocated source is permitted; its metadata is never
copied. Unsupported reflinking fails before clearing files rather than silently
falling back to an ordinary copy. Watchman-enabled mutation is not supported.
Resolve stale targets explicitly with JJ before retrying.

`--from` names a workspace root. A relative CLI path is resolved from the calling
directory; a relative configured `source` is resolved from the default workspace.
Source selection is CLI `--from`, then configured `source`, then the verified
primary checkout. Missing or inconsistent primary registrations fail closed:
repair the repository/workspace identity before retrying. An explicit source
chooses another donor; it does not bypass those identity checks. `source_at` must
still be supplied on the command line or through local configuration.

The Git executable is used for isolated ignore-rule probes, including nested
`.gitignore` files and backend/global excludes. These probes do not create Git
metadata in a target workspace.
Symlinked `.gitignore` files are rejected when their rules cannot be verified.
On Linux, mount-ID verification requires `statx` support (kernel 5.8 or newer);
unavailable or blocked topology queries fail closed.

`--dry-run` validates and reports the plan without clearing or refilling target
contents. Planning can snapshot working copies to freeze their tracked work.

Normal snapshots are intentional: they preserve your real tracked work before
replacement. Recovery avoids recording only the temporary half-filled tree;
after reconstructing and verifying the recorded tree, normal JJ snapshots are
safe again. There is no general `--skip-snapshot` flag in this implementation.

Interrupted operations retain repository-scoped pending records outside the
replaceable contents. **Do not run ordinary snapshotting JJ commands in a
half-filled target.** Resume the recorded plan instead:

```sh
jw reseed my-feature --resume
```

Resume revalidates the recorded source, root, registration, revision, and policy;
it does not accept replacement flags. A changed source or target stops recovery
and retains the pending record. Restore the frozen source before retrying. A
successful tracked refill alone is not success if ignored copying or link
application failed. Batch output distinguishes completed, failed, and pending
targets and exits nonzero for partial failure. Recovery never uses repository-wide
`jj undo`.

## Related material

- [CoW creation](../README.md#copy-on-write-workspaces) creates a new workspace;
  reseeding refreshes an existing one without private working-copy-state copying.
- [Design and acceptance checks](../design/workspace-reseed/reseed0.gpt6a.md)
  explain why public sparse-prefill works and the verification obligations.
- [Architecture](architecture.md#module-map) separates the command adapter,
  reseed coordination, JJ operations, filesystem cloning, links, and metadata.
- [Wildcard-group design](../design/workspace-reseed/groups0.gpt61sol.md) preserves
  the selection rationale; this guide describes the implemented interface.

## Verification and limits

The [native integration cases](../tests/reseed.rs) were verified on Linux Btrfs
with actual reflinks required, including FIEMAP shared-extents checks. The original
22-case official JJ 0.39.0, 0.44.0, 0.45.1, and 0.46.0 release matrix covered one-off and batch use,
source restoration, ignored replacement, symlink bytes, and interruption recovery.
The final optimized build passed all 22 cases on both 0.39.0 and 0.46.0;
0.44.0 and 0.45.1 were fully verified before the source-proof reuse optimization.
Primary-source inference and final-operation revalidation expanded the suite to
30 cases, all passing on official JJ 0.46.0 with real CoW required.
Focused engine tests cover every write-ahead boundary and safe tracked-only type
transitions. Existing integration regressions, Clippy, and formatting also passed.

Wildcard groups at implementation checkpoint `9bd3a1c1` add six native cases
(36 total) and nine selection unit cases. All 353 regression tests passed with
official JJ 0.46.0 using an external temporary directory and explicit nextest
binary lookup. Separately, all 81 CoW/reseed tests passed on Linux Btrfs with
`JW_TEST_REQUIRE_COW=1` on official JJ 0.39.0 and 0.46.0, so unsupported-cloning
early returns in general temporary-directory fixtures are not counted as
real-CoW evidence.
The wildcard build has not yet been rerun on 0.44.0 or 0.45.1.

This is not a representative opencode throughput benchmark or proof of live
Watchman behavior. Watchman-enabled mutation, sparse targets, and colocated targets
remain unsupported. No live operator source or target was reseeded during these
checks.
