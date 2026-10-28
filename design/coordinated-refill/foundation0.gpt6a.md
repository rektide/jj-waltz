---
type: ImplementationBrief
title: Approved domain-grouped crate foundation
description: Human-approved six-crate decomposition, dependency boundaries and foundation commission preceding the streaming reseed implementation.
resource: /design/coordinated-refill/foundation0.gpt6a.md
status: approved-shape-implementation-commissioned
generated: { by: "model:openai/gpt-6-astra#high", at: "2026-10-28" }
sources:
  - resource: "opencode://ses_e7c8f24ebffelGRFC0RXsQvf61"
    title: Parent proposes source-informed six-crate decomposition; human approves; parent commissions regular GLM 5.3 max to implement the foundation
  - resource: "opencode://ses_e7c479171ffeGjjWpkruJHEmUO"
    title: Commissioned foundation implementation session; selected and reported model verified as zai-coding-plan/glm-5.3#max
  - resource: /design/coordinated-refill/architecture0.gpt6a.md
    title: Streaming reseed architecture the foundation must enable
  - resource: /design/coordinated-refill/current0.gpt6a.md
    title: Current ownership, native workflows and source dependency evidence
  - resource: /design/coordinated-refill/library0.gpt6a.md
    title: Dependency relocation and actual shared-transfer API boundary
---

# Approved crate foundation

## Decision and scope

The human approved the six-production-crate shape below and requested G to first
rearrange/restructure the current implementation into a usable foundation. The
parent commissioned **regular GLM 5.3 max**, explicitly selecting the model; both
the selected and provider-reported model identify that variant.

The approved boundary is structural implementation: real code extraction, small
interface corrections, repaired dependencies, relocated tests and working build/
release/install paths. The [streaming reseed architecture](architecture0.gpt6a.md)
defines the subsequent behavior redesign. Current operational behavior supplies
relocation checks; its rollback/fallback conventions do not acquire new authority
over the future implementation merely because those tests still run here.

## Layout

```text
Cargo.toml                    virtual workspace
native/
  jj/                         native adapter
workspace/
  state/                      shared bindings and local state
  lifecycle/                  ordinary lifecycle operations
transfer/
  workspace/                  JW-specific transfer integration
refill/
  engine/                     source preparation and reseed
app/
  cli/                        distributable jj-waltz package, jw binary
tools/
  performance/                existing development-only tool crate
plugins/
  herdr/                      existing separate integration workspace
```

Each production crate directory contains its own manifest and source directory.
Internal packages follow their paths: `jj-waltz-native-jj`,
`jj-waltz-workspace-state`, `jj-waltz-workspace-lifecycle`,
`jj-waltz-transfer-workspace`, and `jj-waltz-refill-engine`. The executable's
distributable package retains `jj-waltz`, with binary `jw`.

### Responsibilities

| Crate | Owned capability |
| --- | --- |
| `native/jj` | JJ client/command grammar, revisions/tree queries, native working-copy state and checkout discovery, with related Git topology support. No high-level lifecycle policy. |
| `workspace/state` | Shared workspace identity/inventory, configuration, ownership records, metadata storage, snapshot/report types and configured links. These can perform owner-specific I/O; this is not a pure-data-only crate. |
| `workspace/lifecycle` | Create, switch, remove, adopt and repair workflows; observation/status and doctor. |
| `transfer/workspace` | JW path-policy and copy adapters, CoW/topology/capability helpers, listed-leaf integration and native frozen-ignore support needed by current callers. Generic filesystem kernels remain in diff-dirs. |
| `refill/engine` | Prepare/reseed request and result types, source selection, native phase orchestration, transfer invocation and diagnostics. Main home for the next implementation. |
| `app/cli` | CLI parsing/rendering, shell integration and current TUI manager. Owns Clap/Ratatui coupling and conversions to library request types. |

## Dependency rules

```text
app/cli
  -> workspace/lifecycle
  -> refill/engine
  -> shared state/native APIs where the adapter actually needs them

workspace/lifecycle -> workspace/state, transfer/workspace, native/jj
refill/engine       -> workspace/state, transfer/workspace, native/jj
workspace/state     -> native/jj
transfer/workspace  -> native/jj, diff-dirs
```

Lifecycle and refill are peers. Shared transfer sits below both; refill does not
depend on the creation workflow just to reach a copier or inventory. Lower crates
do not import application code. Transfer receives resolved policy values rather
than loading global application configuration. State and transfer do not depend
on one another merely to share configuration parsing.

Extract actual interfaces rather than making all existing private modules public.
Keep related implementation details as modules within their owner; there is no
additional generic common/utils crate or mandatory trait layer.

## Concrete extraction concerns

- `workspace.rs` currently combines inventory queries and lifecycle mutation.
  Shared inventory/bindings belong in state; mutation workflows belong in lifecycle.
- Metadata and ownership currently refer to each other. They can remain related
  modules within state rather than creating a cross-crate cycle.
- `cow/payload` mixes native queries, policy and filesystem concerns. Split its
  responsibilities at actual caller needs. The native adapter remains the owner
  of JJ command mechanics; transfer can consume that adapter.
- `cow/ignored` currently reads global configuration. Callers must project resolved
  config into the transfer policy, preserving explicit-empty/disabled meanings
  during this structural stage.
- The engine's `UnmatchedPolicy` currently derives Clap `ValueEnum`. Move CLI
  parsing/conversion to the app so the library's request types are adapter-neutral.
- The existing manager stays an app module. Its size does not alone justify another
  crate before there is an independent consumer/interface.
- Root-relative path assumptions in tests and performance/build helpers need actual
  review. Moving manifests changes `CARGO_MANIFEST_DIR` and sibling dependency paths.

## Foundation deliverable

1. Populated crates and an acyclic graph with working public boundaries.
2. Current diff-dirs package coordinates and receiving lockfile/feature wiring.
3. Owner-local library tests and executable integration tests under the app crate.
4. Updated release/CI/installation commands and native development tooling paths.
5. Appropriate formatting, lint and test evidence with native runtime/environment
   limits identified.
6. A G-authored implementation account and coherent explicit-path commits.

No placeholder-only crate scaffold satisfies this commission. Completion means
the current application builds through the new boundaries and the next reseed work
has an actual engine and transfer API to change.

Native checks use a real JJ executable ahead of mise shims. CoW tests must genuinely
exercise required reflinks where relevant. Discovery fixtures must remain outside
an enclosing checkout when testing external discovery. A global project-local
TMPDIR override would invalidate those tests. Tests operate on owned fixtures;
the operator's actual workspaces are not structural-refactor test subjects.

## Parent review

Review the actual crate graph and public APIs, especially:

- Can root/native resolution be used without importing a refill or creation engine?
- Are global configuration reads outside transfer policy callbacks/adapters?
- Do app libraries avoid leaking Clap/Ratatui into lower domain crates?
- Does the shared transfer API serve both creation and refill without a cycle?
- Are moved test fixtures, packaged artifacts and performance-tool assumptions real?
- Is current machinery located clearly enough to replace rather than duplicated
  behind new wrappers?

Trust reported passing checks; additional verification should answer a concrete
review concern. Material changes to the approved decomposition return for decision.
The subsequent streaming/lifecycle rewrite remains a separate implementation step.
