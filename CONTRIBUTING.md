---
type: Guide
title: Contributing to jj-waltz
description: Development checks, repeatable performance assessment, compatibility, and contribution boundaries.
resource: /CONTRIBUTING.md
status: maintained
updated: { by: "model:openai/gpt-6.1-sol#medium", at: 2026-10-18 }
sources:
  - resource: opencode://ses_eb2165cc8ffe4vOmOvDBe7aXQu
    title: Replaced current performance contribution commands with checked-in Rust tooling and native count controls
  - resource: opencode://ses_eb8947cfaffeQ1jxia03URCymY
    title: Added the checked-in performance harness and its contribution entry point
  - resource: opencode://ses_eb8a7487affeE2smndZ914OQMS
    title: Integrated preparation and performance verification; identified Nextest binary-path and outside-repository temporary-directory requirements
  - resource: /tools/performance/README.md
    title: Maintained measurement and process-count regression workflow
---

# Contributing

Thanks for contributing to `jj-waltz`.

## Development

```bash
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test

cargo fmt --manifest-path plugins/herdr/Cargo.toml --all --check
cargo clippy --locked --manifest-path plugins/herdr/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --locked --manifest-path plugins/herdr/Cargo.toml
```

The Herdr plugin is a separate Cargo workspace, so root Cargo commands do not
check it. CI runs root integration tests against pinned JJ 0.39.0, 0.44.0,
0.45.1, and 0.46.0 on Linux, macOS, and Windows. macOS tests require real
copy-on-write success. Update those explicit pins instead of following `latest`.

For Nextest, expose the built CLI to the older runtime `assert_cmd` helpers:

```sh
cargo build --locked --bin jw
CARGO_BIN_EXE_jw="$PWD/target/debug/jw" cargo nextest run --locked
```

Adjust the binary path for a custom target directory or profile. The full suite's
temporary directory must be **outside any repository**: some tests deliberately
exercise discovery with no enclosing checkout. To require real CoW throughout
the suite, use an outside-repository, reflink-capable `TMPDIR` and set
`JW_TEST_REQUIRE_COW=1`. The reseed, prepare, and ignored-copy integration fixtures
already allocate their own project-local CoW scratch directories.

## Performance assessment

Use the maintained [performance harness](tools/performance/README.md) for
prepare/reseed changes. Build a release binary, provide a real JJ executable,
and compare retained, verified fixtures on the same CoW-capable filesystem.
The Rust harness separates native process timings from instrumented JJ/Git counts;
small/large fixture comparisons and process-count budgets detect per-file
subprocess regressions without flaky wall-time gates.

```sh
cargo build --locked --release -p jj-waltz-perf
cargo nextest run --locked -p jj-waltz-perf
cargo clippy --locked -p jj-waltz-perf --all-targets -- -D warnings
```

The guide supplies verified artifact/build/import and N2 campaign commands. Native
fixtures are isolated explicitly; live donors are read-only. Retired Python
methods are historical evidence, not supported commands or runtime dependencies.

## Scope

Please prefer small, focused changes with tests where practical.

Keep product ownership clear:

- `jw` coordinates workspace-lifecycle jobs; ordinary revision manipulation stays
  in `jj`.
- Pull requests, merge requests, and CI belong to `gh`, `glab`, or another thin
  forge adapter.
- Local workspace management must work offline.
- A documented roadmap concept is not a shipped command. Update `README.md` only
  when `jw --help` and tests support the claim.

## Design principles

- Keep terminal, JSON, Herdr, and future views over one domain model.
- Capture one final JJ operation ID, then derive status without subprocesses.
- Put JJ process execution behind the central adapter; never assemble avoidable
  shell command strings.
- Keep static configuration separate from repository-scoped lifecycle metadata.
- Preserve JSON schema v1: optional additions are compatible; removals, renames,
  required-field changes, or changed enum meanings require a version bump.
- Keep shell integration native and errors actionable.

## JJ compatibility

JJ 0.39.0 is the minimum supported release. Root CI also tests pinned JJ 0.44.0,
0.45.1, and 0.46.0 as newer compatibility targets. Do not follow a floating `latest` in
tests. Raise the minimum only for a documented public capability, and keep JJ
output parsing and version differences inside the adapter. When changing
divergence recovery guidance, preserve the JJ 0.39-compatible manual remedy and
gate `jj converge --no-interactive` on JJ 0.45 or newer.
