---
type: ImplementationAccount
title: Deferred routing containers and streaming wildcard adoption
description: Delivered upstream source-only conditional creation, public 0.2 marker, production preview and saved-decision integration, and JW adoption without payload discovery or empty skeletons.
resource: /design/coordinated-refill/implementation2.sol61x.md
status: implemented-slice2-awaiting-parent-review
generated: { by: "model:openai/gpt-6.1-sol#xhigh", at: "2026-10-28" }
sources:
  - resource: "opencode://ses_e7be6b24fffeDISpWdV1xNbqhD"
    title: Implements approved slice 2 after parent acceptance of slice 1; verifies conditional production effects and JW wildcard adoption, then returns before CLI replacement
  - resource: /design/coordinated-refill/implementation0.gpt6ax.md
    title: Source-qualified policy, concrete conditional-container gap and slice boundaries
  - resource: /design/coordinated-refill/implementation1.sol61x.md
    revision: ec7fe09f
    title: Approved native and authority checkpoint; its missing-route limitation is resolved by this slice
  - resource: "file:///home/rektide/src/diff-dirs/tree/inspect/src/visitor.rs"
    revision: 10361cf0
    title: Public Container addition, action legality and immediate potential-work counting
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/apply.rs"
    revision: 10361cf0
    title: Runtime pending ancestry, actual materialization, postorder modes and recursive preview
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/src/coordinated/decision.rs"
    revision: 10361cf0
    title: Metadata-free validation for current-group saved actions and coverage
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/tests/conditional.rs"
    revision: 10361cf0
    title: Focused production effect, refusal, permission, parallel and restoration discriminators
  - resource: "file:///home/rektide/src/diff-dirs/examples/reconcile/src/transport.rs"
    revision: 10361cf0
    title: Explicit action vocabulary and descent-preserving round-trip test
  - resource: "file:///home/rektide/src/diff-dirs/README.md"
    revision: 10361cf0
    title: Current specification, deliberate 0.2 public-interface marker and declared checks
  - resource: "file:///home/rektide/src/diff-dirs/transfer/sync/README.md"
    revision: 10361cf0
    title: Maintained conditional-container contract, accounting and verification
  - resource: /transfer/workspace/src/streaming.rs
    revision: cbf0a53c
    title: SourceQualified adoption and real required-reflink wildcard cursor tests
verification:
  method: Required-reflink upstream all-target/all-feature nextest, workspace doctests, independent example checks, and focused default/feature-enabled JW production-cursor cases
  result: 230 upstream workspace tests, 20 doctests and 15 reconciliation-example tests passed; two focused JW cases passed in each build profile; strict lint, format and Markdown checks passed
  scope: Linux and tiny owned fixtures; no CLI replacement, new cohort-sharing claim, cross-platform runtime certification or performance campaign
---

# Upstream public-effect integration checkpoint

**Slice 2 is implemented and committed.** The source-qualified visitor can now
discover selected ignored directories through absent unselected parents without
creating unmatched skeletons. The public CLI still uses the old route, as
commissioned. No native protocol, ordinary deletion breadth or empty-policy
meaning was changed. The two starting user drafts remain untouched.

| Repository / commit | Delivered change |
| --- | --- |
| diff-dirs `10361cf0` | Conditional container API/effect, validation, preview, continuation/transport coverage, native docs and coordinated production **0.2.0** marker. |
| jj-waltz `cbf0a53c` | Exact `=0.2.0` receiving dependencies/lock, streaming visitor adoption, and real wildcard-route cases. JW packages remain `0.8.0`. |

## Exact action and ownership

```rust,ignore
Action::Directory {
    container: Container::CreateIfNeeded,
    descend: true,
}
```

Legal only for a **source-only real directory with descent**. A paired directory,
target leaf collision, source leaf, or `descend: false` refuses the complete
immediate batch before writes. Eager `Create` retains its old meaning, including
creating an explicitly selected empty directory. Coupled replacement actions and
coordinated existing-root requirements are unchanged.

| Owner | Implementation |
| --- | --- |
| `tree/inspect/src/visitor.rs` | Public variant, shape/descent validation and immediate potential-work classification. |
| `transfer/sync/src/apply.rs` | Source-only opaque recursion, pending ancestry/materialization, normal entry-kernel dispatch and preview. |
| `transfer/sync/src/coordinated/decision.rs` | Same legality/work rules for metadata-free saved decisions; lowering retains the actual action. |
| `examples/reconcile/src/transport.rs` | Explicit `directory/create-if-needed/descend` vocabulary; descent round-trips. Invalid stopped actions still fail production decision validation. |
| JW `transfer/workspace/src/streaming.rs` | Missing unselected real-directory routes select this action instead of refusing. Target leaf/link route collisions still refuse. |

### Pending ancestry and effects

Each opaque call owns a depth-sized `Vec<PendingDirectory>`: physical output
path, source permissions, and `needed`/`created`/`relaxed` state. Source-only
inspection requires no target directory. Final logical coordinates remain in
the actual directory plans and visitor operations, even under replacement staging.

On an admitted descendant namespace action, `materialize_pending` marks the
required chain, obtains the existing external parent's normal writable lease and
exclusively creates missing frames in order. It records creation ownership before
fallible metadata/permission work. It then dispatches the selected entry through
the existing production copy/removal/staging kernel, not a new copier. Subsequent
siblings reuse materialized frames.

Frames finalize in **postorder**, after descendants and siblings return, on success
or failure. Only created paths receive source permissions and one completed
directory-creation count. Modes are not restored after the first leaf while later
siblings still need their parent. Uncreated frames are neither chmodded nor
finalized. The external parent's existing lease remains owned by its enclosing
batch. Temporary staging cleanup and joined permission restoration retain their
normal responsibilities; visible earlier effects are not rolled back.

The mutable pending recursion is sequential within one opaque call; independent
outer entries can still execute in a parallel batch. This adds neither a scheduler
nor a source catalog. Current directory plans/listings remain width-sensitive;
only the additional pending ancestry is depth-sized.

## Accounting, preview and continuation

- Immediate inspection and saved comparison coverage count a conditional **potential
  obligation**. They do not know whether its opaque interior has an admitted effect.
- Recursive application and preview resolve that obligation. `Report.planned` and
  `Failure.planned` count a routing container once a descendant needs it, including
  an attempted materialization that fails. `Progress.directories_created` counts
  only successful actual creations, once per frame.
- A no-match conditional branch therefore has zero recursive planned/completed
  effects despite a possible immediate observed obligation. A selected empty
  directory materializes its pending chain through ordinary eager `Create`.
- Preview performs the same recursive source-only assessment without creating,
  staging, chmodding or pruning anything. Existing nonconditional preview outcomes
  are preserved by the regression suite.
- `into_parts`/`from_parts` save the current group's action and exact decisions,
  not pending physical ancestors or in-flight recursion. Refresh validates the new
  action normally. The focused restore test checks invalid stopped descent,
  refreshed application and normal token replacement before effects.
- Created one-sided interiors do **not** join the paired frontier. Sharing still
  covers paired immediate groups; absent routing interiors are inspected/copied
  separately per lane. No new whole-command source-once claim follows.

The example transport retains its existing explicit version-one DTO layout while
adding the new action spelling. Older readers reject unknown vocabulary; this is
not an in-flight recovery format or an automatic migration promise. The example's
stock preserve/refuse CLI does not itself select conditional routing.

## Version and dependency marker

The five production libraries and CLI move together from `0.1.0` to **`0.2.0`**.
Their internal dependencies are exact; dev-only performance tools retain `0.0.0`
and pin their actual internal consumers. All three standalone examples adopt exact
library pins and receiving locks; cursor/walk checks confirm their real consumption.
JW's three receiving manifests and root lock agree on exact `=0.2.0` aliases.

The installed `cargo-bump` rejects workspace manifests, and `cargo set-version`
is unavailable. The explicit coordinated manifest edits were therefore applied
directly, with normal targeted Cargo lock updates. No tool was installed. The
minor marker makes the public enum/source-interface addition visible; version
equality is not a broader compatibility guarantee.

## Verified discriminators

| Case | Actual result |
| --- | --- |
| No selected descendant | Zero recursive planned/effect counters, no target children; existing read-only root inode/mode/mtime/ctime retained. |
| Deep selected empty directory | Exactly its required three-directory chain created; unmatched sibling branch absent. |
| First leaf then sibling | Two routing containers created once, two files copied; read-only source modes installed postorder and one external-parent lease restored. |
| Conditional recursion under staged leaf→directory transition | Correct final logical policy paths, selected contents installed, excluded branch absent. |
| Private preparation refusal | Old collision leaf retained, staging cleaned, no private work reported as installed effects. |
| Failure after visible work | One installed file/two created directories retained with honest progress; owned routing/external-parent modes finalized. |
| Illegal shape/stopped descent | Whole immediate batch refuses, including an otherwise admitted earlier sibling. |
| Parallel outer siblings | Independent pending chains and one shared parent lease; correct creation/permission counts. |
| Saved current group | New action and coverage validate/refresh, opaque interior completes, only the original paired root is examined. |
| JW `**/node_modules` | Missing `packages/a` routes created only for selected contents/empty dir; `packages/a/src` and `packages/b` remain absent. Paired selected stale descendants removed; target-only candidate root retained. Two actual required reflinks, zero ordinary copies, clean-repeat zero effects and private writes. |

Checks actually run:

- Upstream `DIFF_DIRS_SYNC_REQUIRE_REFLINK=1 cargo nextest run --locked --workspace
  --all-targets --all-features`: **230 passed, none skipped**, including nine new
  conditional cases. The strengthened no-effect fingerprint case also passed in
  the default build.
- `cargo test --locked --workspace --doc`: **20 passed**.
- Reconcile all-feature nextest: **15 passed**, including the new vocabulary case;
  strict all-target/all-feature Clippy passed after moving its test module to the
  normal end-of-file position.
- Cursor all-target and walk all-target/all-feature checks passed with their
  updated standalone locks. Upstream strict workspace all-target/all-feature
  Clippy and root/example formatting passed.
- JW's two focused route cases passed with `JW_TEST_REQUIRE_COW=1`, `/usr/local/bin`
  first, in both default and `payload-parallel` builds. New N1 runtime remains
  sequential; feature compilation is not a new parallel-execution claim.
- JW strict transfer-crate all-target/all-feature Clippy and workspace
  all-target/all-feature check passed. Slice-1 native tests were not rerun merely
  for confirmation. No new workload import, install or benchmark campaign occurred.
- Native upstream Markdown and this account passed Rekon link/provenance checks.

No unresolved semantic blocker was demonstrated in this scope. Runtime evidence
is Linux-only; ordinary I/O/race hazards and required-reflink failure remain normal
forward failures under the existing stable-tree contract.

## Remaining 3–4 handoff

After parent review, replace the actual CLI driver rather than preserving parallel
old/new modes. Extract batch-owned resolution/native inputs from the callable N1
seam: snapshot receivers once for preservation, recheck donor coordinate once,
share memberships/control bytes/cache, execute serial native prefixes, then bind
one real N2 runtime. **Do not implement N2 by repeatedly invoking N1.** Keep source
preparation a separate producer and consume its handoff without another donor proof.

Land honest nonmutating root/native dry-run, help/operator hazards and removal of
legacy reseed discovery/proof/prediction machinery with that replacement. Only
policy-satisfied, link-finished receivers are complete; native-ready alone is not
success. A surfaced failure stops with known partial effects, not mandatory peer
completion or a fallback algorithm.

The current integration concern is **report meaning**, not another capability gap:
immediate conditional obligations, recursively resolved plans, actual completed
effects and native correctness are different results. CLI rendering and bounded
N2/fault/process-count evidence must keep them distinct. Account for per-lane
opaque source-only work rather than labeling all source observations shared.
Use a small equal-policy performance comparison only after the real CLI works.

This extends the [approved N1 account](implementation1.sol61x.md#next-coherent-slice-after-review)
and realizes [slice 2 of the commission](implementation0.gpt6ax.md#slice-2--finish-streaming-missing-route-semantics).
The [upstream maintained contract](file:///home/rektide/src/diff-dirs/transfer/sync/README.md#conditional-routing-containers)
owns the generic effect; this account owns its JW adoption and next handoff.
