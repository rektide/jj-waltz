---
type: ImplementationBrief
title: "Supersede ignored-copy selection with scoped transfer"
description: Selected implementation direction and dependency order following the human's request to surge toward implementation against the changed checkout.
resource: /design/sync-scope/implementation0.gpt6a.md
status: implementing
generated: { by: "model:openai/gpt-6-astra#xhigh", at: 2026-10-18 }
sources:
  - resource: "opencode://ses_eb262a81effea1HgF0pPS3f1sy"
    title: Human authorized implementation superseding copy_ignored; parent inspected underfoot changes and selected a sequential library-first landing
  - resource: /design/sync-scope/design0.gpt6a.md
    title: Authority-based transfer model, generic scope/absence interfaces and listed-leaf installation
  - resource: /design/sync-scope/jj-state-review0.gpt6a.md
    title: Corrected JJ identity/cache mechanics; state transplantation and move protocols remain distinct work
  - resource: /src/reseed/payload.rs
    revision: dba9765f653bacc5139398447a8a80f9565236b5
    title: New fused ignored-root discovery/validation inspected before superseding the mechanism
  - resource: /src/cow/ignored.rs
    revision: dba9765f653bacc5139398447a8a80f9565236b5
    title: Existing caller-driven selector, pattern semantics and old empty/disabled policy behavior
---

# Implementation direction

The human requested implementation after the research, explicitly superseding
the way `copy_ignored` operated. This is a new implementation commission; the
cancelled benchmark is not revived.

## Outcomes

- **Presence:** broad auxiliary updates from the donor. Unpreserved same-name
  entries are donor-owned; destination-only entries are retained by default.
- **Absence:** deletion only in explicitly configured mirror regions, including
  matching target-only regions whose source root has disappeared.
- **Preservation:** symmetric source/target exclusions, applied before metadata
  and descent. Protected descendants retain necessary directory ancestors.
- **Tracking:** JJ owns the desired tracked tree. The auxiliary scope excludes
  source/desired tracked leaves and their subtrees; required tracked-container
  directories remain protected from incompatible leaf replacement. Git ignoredness
  does not authorize transfer or deletion. Nonignored transferred data follows
  ordinary future JJ snapshot policy; the command does not rewrite ignore files.
- **Native ownership:** `.jj`, root Git metadata, JW control paths and configured
  link-owned names are excluded. Local preservation is explicit transfer policy.
- **CoW:** retain tracked sharing renewal, using a generic listed-leaf installer;
  keep the existing JJ sparse/materialization protocol for this implementation.

`copy_ignored` config/flags must not be silently reinterpreted, especially `[]`.
Retire the old transfer selector and provide a direct migration diagnostic for
obsolete inputs. New transfer controls expose preserve/mirror and a complete
auxiliary-transfer opt-out. No permanent second ignored-copy execution engine.
Ignore helpers may remain for independent link/snapshot contracts that actually
use them; they cease to be the auxiliary synchronization gate.

Broad transfer is a real outcome change, not merely a faster implementation of
the old selection. Describe it in CLI/config/operator documentation and tests.
No live user configuration is rewritten as part of this commission.

## Dependency and ownership order

1. Implement and verify generic scope, absence authority, recursive retained
   outcomes, type-transition preservation, scoped audit, explicit clear and
   listed-leaf installation in diffdirs. Existing ordinary APIs retain their
   behavior; use compiled traits/closures and shared operation kernels.
2. Parent reviews that concrete API/semantic checkpoint.
3. Integrate JW against it: retire selector/outside-payload walks, use the generic
   kernels, migrate creation/reseed/prepare policy plumbing and maintained callers.
   Preserve source/target/desired distinctions and native working-copy identity.
4. Verify the integrated behavior on tiny semantic cases, default and experimental
   parallel builds. Update native Rust performance callers for the new config/API
   without running a campaign.

The underfoot `dba9765f` fusion is not reverted. Its repeated-pass reduction is
useful precedent; the new root-scoped traversal supersedes the fused selector as
a whole. The existing human handoff and unrelated workspaces remain untouched.

## Limits

Directory-to-leaf replacement locally checks removability before dismantling a
protected subtree. There is no whole-tree operation plan or normal pre-audit.
Keep upfront mount topology and real operation-time reflink failure.

No `.jj` transplantation, stash/restore protocol, new journal, automatic cleanup,
whole-workspace swap, new Python tooling or benchmark framework in this slice.
Those state optimizations need their own verified protocol; scoped preservation
already avoids walking or copying the protected directories.

Fixtures remain small (at most 256 payload entries / 1 MiB per case). Native JJ
fixtures must be isolated; external-discovery tests must genuinely live outside
an enclosing JJ workspace. No real opencode import, live reseed, large retained
copies or repeat-until-fast measurement. Completed evidence must distinguish the
new semantics from earlier tests/results of the old mechanism.
