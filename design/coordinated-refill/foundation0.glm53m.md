---
type: ImplementationReport
title: Six-crate foundation restructure completed
description: Delivered crate split, dependency-direction and test ownership mapping, repaired diff-dirs coordinates, verification results including one inherited colocate failure with root cause, and remaining hooks for the streaming rewrite.
resource: /design/coordinated-refill/foundation0.glm53m.md
status: implemented-foundation
generated: { by: "model:zai-coding-plan/glm-5.3#max", at: "2026-10-28" }
sources:
  - resource: "opencode://ses_e7c479171ffeGjjWpkruJHEmUO"
    title: Commissioned structural implementation of the approved six-crate shape with test moves, coordinate repair, and this source-grounded completion report
  - resource: /design/coordinated-refill/architecture0.gpt6a.md
    title: Approved target architecture whose owner boundaries the crate split mirrors
  - resource: /design/coordinated-refill/current0.gpt6a.md
    title: Current-implementation census and stale-manifest evidence this work consumed
  - resource: /design/coordinated-refill/library0.gpt6a.md
    title: Verified diff-dirs package coordinates and alias seam adopted here
  - resource: /Cargo.toml
    title: Virtual workspace manifest with six production members plus the dev crate
  - resource: /workspace/state/src/inventory.rs
    title: Shared inventory and query facts extracted from the former workspace.rs
  - resource: /transfer/workspace/src/frozen.rs
    title: Frozen-ignore adapter split out of the former cow/payload.rs mixture
  - resource: /transfer/workspace/src/tracked.rs
    title: Native tracked-leaf queries and path primitives split out of cow/payload.rs
  - resource: /workspace/lifecycle/src/workspace.rs
    title: Creation, switching, and removal mutation orchestration retained by lifecycle
  - resource: /refill/engine/src/engine.rs
    title: Reseed orchestration with global policy layer now loaded by the engine and passed explicitly
  - resource: /app/cli/src/cli.rs
    title: App-owned UnmatchedPolicyArg conversion replacing the engine's clap ValueEnum coupling
  - resource: /docs/architecture.md
    title: Maintained architecture doc with source links updated to the new layout
verification:
  method: cargo check/clippy/fmt across the workspace, cargo test for every member with real /usr/local/bin/jj ahead of mise shims, release build with payload-parallel forwarded through app/cli
  result: All checks pass; full test suite green except one pre-existing colocate failure reproduced identically before the restructure and root-caused to a jj 0.45.1 error-wording change
  scope: Linux x86_64 with real CoW filesystem; musl target not installed locally so the CI musl job is unverified here
---

# Foundation restructure: what shipped

## Delivered shape

The approved six-crate structure is implemented as a virtual workspace. Every
production crate sits in a two-level domain directory and carries the matching
`jj-waltz-*` package name at shared version `0.8.0` with exact internal pins;
`publish = false` keeps internal crates out of distribution. `app/cli` retains
package name `jj-waltz` and binary `jw`, so every existing `-p jj-waltz`,
`--bin jw`, dist, and `cargo metadata` reference keeps working.

| Crate | Package | Content moved in |
| --- | --- | --- |
| native/jj | `jj-waltz-native-jj` | `src/jj.rs` (client, revision/tree queries, working-copy state), `src/context.rs` (checkout discovery), `src/paths.rs` (external-command path helpers). Mechanisms only. |
| workspace/state | `jj-waltz-workspace-state` | `config.rs`, `metadata.rs`, `ownership.rs`, `snapshot.rs`, `links.rs`, and the inventory/query half of `workspace.rs` as new `inventory.rs`. The metadata↔ownership mutual references stay together here as permitted. |
| workspace/lifecycle | `jj-waltz-workspace-lifecycle` | `lifecycle.rs` (now lib.rs) with `ignored.rs` creation planning, the mutation half of `workspace.rs` as `workspace.rs`, plus `observe.rs` and `doctor.rs`. Re-exports `WorkspaceInventory` through `workspace` for its own call sites. |
| transfer/workspace | `jj-waltz-transfer-workspace` | `cow.rs` as lib.rs (capability helpers), `topology.rs`, `transfer.rs` (diff-dirs visitor), `ignored.rs` (path policy), and `cow/payload.rs` split by ownership into `frozen.rs` + `tracked.rs`. |
| refill/engine | `jj-waltz-refill-engine` | `reseed.rs` (now lib.rs) with `engine/` submodules (engine, payload, selection, bookmarks, prepare) and `freshen/source.rs`. |
| app/cli | `jj-waltz` (bin `jw`) | `main.rs`, `cli.rs`, `shell.rs`, `manager/` (mod, preferences, removal, status, ui). |

Dependency direction is exactly the approved graph, with no cycles:

```
app/cli ──────────────┬───────────────► lifecycle ─┬──────────────► state ──► native/jj
  │                   │                           └─► transfer ──────────────┬─► native/jj
  └─► refill ─┬───────┴─► state, transfer, native/jj                          └─► diff-dirs-tree-api
             └─► diff-dirs-tree-api + diff-dirs-transfer-sync (aliased diff-dirs / diff-dirs-sync)
transfer/workspace ─► native/jj + diff-dirs-tree-api + diff-dirs-transfer-sync
```

`tools/performance` remains a dev member (its `diff-dirs-perf-runner` coordinates
at `../../../diff-dirs/perf/runner` still resolve from its own manifest depth),
and `plugins/herdr` remains a separate Cargo workspace, untouched.

## Ownership decisions worth review

1. **`cow/payload.rs` was split, not moved.** `frozen.rs` owns the frozen-ignore
   adapter (`FrozenRules`, `IgnoreRules`, the persistent Git oracle, `Excludes`,
   `checked_git`); `tracked.rs` owns native tracked-leaf queries (`Entry`,
   `entries`, `file_bytes`) and the path primitives (`relative`,
   `confined_parents`, `directory_identity`). `frozen` calls into `tracked` for
   rule extraction; nothing else overlaps.
2. **`workspace.rs` split along query/mutation lines.** `WorkspaceInventory`,
   name/root resolution, previous-workspace and bookmark record reads,
   `path_for_workspace`, and completion candidates live in state's `inventory`.
   Creation with materialize/rollback, switching with navigation recording,
   removal planning/execution, and destination normalization stay in lifecycle.
   `record_created` now takes `(name, root)` instead of the lifecycle-owned
   `AddResult`, keeping state free of lifecycle types; new accessors
   (`previous`, `default`, `primary_root`) replaced direct field access.
3. **Transfer receives resolved policy.** `Policy::load` (which read the
   application global config inside the transfer crate) is gone. `Policy::resolve`
   takes cli/local/global layers explicitly; the refill engine loads the global
   config once through workspace-state and passes its layer. Lifecycle already
   resolved all three layers itself. No application configuration loading remains
   under `transfer/`.
4. **Clap decoupled from engine types.** `reseed::UnmatchedPolicy` no longer
   derives `clap::ValueEnum`; `cli.rs` owns `UnmatchedPolicyArg` with the parse
   and a `From` conversion. Clap and ratatui are app/cli-only dependencies.
5. **`payload-parallel` lives where the code is.** The optional rayon dep, the
   pool `cfg`, and the diff-dirs feature forwarding moved to transfer/workspace;
   lifecycle and refill forward it, and app/cli forwards through refill, so
   `cargo build -p jj-waltz --features payload-parallel` (used by the perf
   harness) still works.
6. **Promoted visibility is demand-driven.** Only items with real cross-crate
   consumers became `pub` (for example `Execution`, `Auxiliary`, `Topology`,
   `LinkApplication`, `write_json_atomic`, `upsert`, `root`, capture methods).
   `metadata::upsert`/`remove`/`root` lost their `#[cfg(test)]` gates because
   doctor and manager tests in *other crates* legitimately seed stores; they are
   ordinary store operations, and the gates were a single-crate compilation
   artifact.

## Dependency-coordinate repair

`diff-dirs` now aliases `diff-dirs-tree-api` at
`../../../diff-dirs/tree/api`, and `diff-dirs-sync` aliases
`diff-dirs-transfer-sync` at `../../../diff-dirs/transfer/sync` (three directory
ups from two-level crate directories — the single root-level dependency used to
need only two). The alias keeps every existing `diff_dirs`/`diff_dirs_sync`
import name working, exactly as the [library assessment](library0.gpt6a.md)
proposed. Verified library versions remain `0.1.0`. `tools/performance` needed
no manifest change. No diff-dirs production file was touched.

## Tests and fixtures

- Binary integration tests (`cli`, `colocate`, `context`, `context_routing`,
  `creation_ignored`, `external_owner`, `prepare`, `reseed`, `reseed_bookmarks`,
  `shell_ui`, `ui_cli`) moved to `app/cli/tests` so `CARGO_BIN_EXE_jw` and
  `cargo_bin("jw")` resolve in the defining package.
- Library tests moved to owners: `freshen_source.rs` → `refill/engine/tests`
  (imports `jj_waltz_refill_engine`/`jj_waltz_native_jj`), `links_stress.rs` →
  `workspace/state/tests`. Inline unit tests stayed with their modules.
- `CARGO_MANIFEST_DIR` assumptions updated: `reseed_bookmarks` and the transfer
  visitor fixture now join `../../.test-agent/...` from their two-level crate
  directories. The visitor fixture additionally canonicalizes its scratch path:
  the deepened manifest path plus `..` components otherwise pushed its
  unix-socket path past `SUN_LEN`, which was the one test failure this
  restructure introduced and fixed.
- `terminal_ui.py` resolves `jw` via `parents[2]` now; scratch/discovery
  conventions (project `.test-agent`, outside-repo TMPDIR for discovery tests,
  real CoW under `JW_TEST_REQUIRE_COW`) are unchanged.
- Doc references to moved sources updated in `docs/architecture.md`,
  `docs/reseed.md`, and `docs/workspace-manager.md`; historical OKF `resource:`
  fields in `tools/performance/README.md` were left as records of where files
  lived when read.

## Verification

Run with the real `/usr/local/bin/jj` (0.45.1) ahead of mise shims to avoid the
documented exec recursion:

- `cargo check --locked`, `cargo clippy --workspace --all-targets
  --all-features`, `cargo fmt --all --check` — all clean. The one remaining
  warning is the pre-existing `package.homepage` manifest lint, present before
  the restructure.
- `cargo test --locked --workspace --no-fail-fast` — **every suite green except
  `colocate::owned_cleanup_removes_only_target_and_keep_dir_preserves_files`**,
  which fails identically on the pre-restructure parent (`61fe94d6`) and is
  therefore inherited, not introduced.
- `cargo build --locked --release -p jj-waltz --features payload-parallel` —
  forwarded feature compiles; `jw --version` reports `0.8.0`; the CI
  `cargo metadata` version probe still resolves.
- Not verifiable here: the musl CI job (target not installed locally).

### The inherited colocate failure, root-caused

The test deletes a colocated workspace's directory and expects `jw remove` to
forget it. On jj 0.45.1, `jj workspace root --name <gone>` now fails with
`Error: No such workspace: gone` — wording that the missing-checkout classifier
in `workspace/state/src/inventory.rs` does not match, so the inventory records
`root: None` and removal errors with `workspace not found`. I prototyped an
additive match for the new wording and it does **not** repair the test: the
remove flow needs the recorded (dangling) path to proceed, meaning older jj
returned the deleted checkout's path successfully. This is a jj-version
behavioral gap the current route encodes, not a structural defect; deciding how
removal should treat registered-but-unresolvable checkouts (and which jj
versions to match) belongs to the parent, so no behavior change shipped here.

## Limits and hooks for the real rewrite

- `cargo publish -p jj-waltz` is no longer possible as-is: the app now has
  internal path dependencies. Release distribution remains dist/homebrew (which
  never published to crates.io); if crates.io publishing is ever wanted, the
  internal crates must be versioned and published or the split reconsidered.
- The engine still loads the application global config itself (through
  workspace-state) to fill the policy's global layer; when the reseed command
  moves to the app-owned request assembly the parent may prefer to hoist that
  load into the CLI.
- Reseed orchestration, rollback/fallback routes, and all current behavior are
  deliberately unchanged — this stage moved code, it did not select streaming
  policy, failure contracts, or any item from the architecture's selection
  points. The one behavior-adjacent repair (jj wording) was reverted rather
  than smuggled in.
- The `1 + 8N` verification structure, `Auxiliary::legacy` realized-root
  constructor, and target-at-a-time executor all still exist where the
  [census](current0.gpt6a.md) left them, now addressable as
  `jj-waltz-refill-engine` and `jj-waltz-transfer-workspace` seams.

## Connections

- Implements the structural prerequisite of [the streaming reseed
  architecture](architecture0.gpt6a.md); its phases 4 and 10 own the behavior
  changes this stage explicitly did not make.
- [Library assessment](library0.gpt6a.md) proposed the package-alias seam
  adopted for the coordinate repair.
- The [current census](current0.gpt6a.md) stage-0 row (repair the dependency
  boundary) and the workspace-splitting groundwork for its stages 1–2 are
  complete; those stages remain unimplemented and unselected.
