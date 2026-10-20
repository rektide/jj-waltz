---
type: DesignAssessment
title: Continuation cursors, inherited traversal modes, and extensible actions
description: Human-directed continuation semantics and a source-grounded assessment of work avoidance, generated entries, action dispatch, and the narrow JJ adapter boundary.
resource: /design/sync-scope/cursor-actions0.gpt6a.md
status: proposal-with-commissioned-cursor-demo
generated: { by: "model:openai/gpt-6-astra#high", at: 2026-10-20 }
sources:
  - resource: "opencode://ses_ea82cf969ffeAXqmUxHChTJar2"
    title: Human rejects recovery promises, commissions AM cursor design and Sol xhigh demo, and explores subtree shortcuts and generated parameterized actions
  - resource: "file:///home/rektide/src/diff-dirs/src/visitor.rs"
    title: Actual early facts, late observations, fixed actions, existing-slot constraints, and pure speculative visitor contract
  - resource: "file:///home/rektide/src/diff-dirs/src/pair.rs"
    title: Immediate paired inspection and disposal of clean plans
  - resource: "file:///home/rektide/src/diff-dirs/src/walk.rs"
    title: Existing DFS frames and finalized divergent-batch yields
  - resource: "file:///home/rektide/src/diff-dirs/sync/src/apply.rs"
    title: Action dispatch, source-only recursion, staged replacements and public versus internal plan application
  - resource: "file:///home/rektide/src/diff-dirs/sync/src/visitor.rs"
    title: Existing Seed, Update, Prune and compatibility steering
  - resource: "file:///home/rektide/src/diff-dirs/design/execution/flow0.gpt6a.md"
    title: Maintained execution walkthrough used to navigate source
  - resource: /src/cow/transfer.rs
    title: JW's current bounded auxiliary visitor and administrative-name treatment
  - resource: /design/sync-scope/visitor0.sol61xh.md
    title: Existing compatibility integration and native donor-state experiment
  - resource: "file:///home/rektide/src/diff-dirs/design/cursor/cursor0.gpt6am.md"
    revision: 00600ed9
    title: Reviewed AM design selecting immediate inspection, explicit DFS DTOs and kopacetic-rkyv snapshots for the commissioned demo
  - resource: "file:///home/rektide/src/kopacetic/crates/kopacetic-rkyv/src/lib.rs"
    title: Snapshot projection, checked decoding, explicit schema matching and owned Facet reconstruction
  - resource: "file:///home/rektide/src/kopacetic/crates/kopacetic-rkyv/README.md"
    title: Reflected archive format and its cost and type-adapter boundaries
  - resource: "file:///home/rektide/src/diff-dirs/design/performance/opportunities0.gpt6am.md"
    title: Prior proposal to obtain comparison keys only where they affect classification; read A03 and the prepared-pipeline discussion
  - resource: "file:///home/rektide/src/diff-dirs/design/performance/opportunities0.sol61x.md#c04"
    title: Earlier distinction between expanded one-sided iteration and whole-subtree replacement; read C04 and M01
---

# Continuation and action design

## Direction and standing

The human's failure model is forward iteration: work happens, and a failure
promises no rollback, recovery boundary, or effects history. The useful optional
facility is a serializable cursor retaining current input tips, traversal position
and implementation state, so another invocation can continue, retry or skip bad
work. Concurrent writers and locking are out-of-band concerns. Assume the donor
checkout is good; do not make repeated donor proof the premise of the design.

The immediate commission is an AM design assessing kopacetic-rkyv, followed by a
Sol xhigh Rust CLI demo which assesses N units and exports a cursor for the next
invocation. The broader action forms below are proposals, not new production JW
defaults. Existing copying and removal contracts have not changed.

## Reviewed demo direction

[AM's design](file:///home/rektide/src/diff-dirs/design/cursor/cursor0.gpt6am.md)
was reviewed and handed to Sol xhigh for implementation. The example belongs in
diff-dirs `examples/cursor`, with a separate workspace and lockfile. Heavy Facet
and kopacetic dependencies stay out of the comparison core.

One unit is an attempted immediate-directory assessment, including failed attempts.
The driver calls existing public `inspect_directory`, serializes explicit DFS
frames and accumulators, and emits post-order completion records. It generates
virtual summary-file bytes through a parameterized handler, without modifying the
input trees. A blocked directory can be retried or its scheduled subtree skipped.

Kopacetic fits: `Snapshot::from_value` archives a Facet cursor under explicit schema
and identity; checked decoding followed by `reconstruct` restores an owned cursor.
This is a generic logical-path-row archive, not native `Archived<Cursor>`. Its
projection and reconstruction allocate, and application `PathBuf` needs an explicit
native-path adapter. The demo uses Unix bytes and binary cursor files. Direct rkyv
could reduce representation work, but no size/speed superiority has been measured.
The cursor demonstration will establish integration, not a codec performance win.

## What exists now

The observations/disposition separation is implemented, although `disposition`
is an explanatory term rather than a distinct public type:

- `Names` exposes actual union names, presence and cached early facts. Early
  preservation avoids child observation and descent.
- `Operation` contains actual source/target facts, comparison, baseline action
  and selected action. `DirDiff` remains the add/change/remove projection.
- `DirectoryPlan` finalizes those actions. `sync::Context::execute_entry` is
  already their dispatcher; finalized actions also steer descent.
- `Action` is a fixed enum: preserve/keep/install/remove, directory
  container-plus-descent, and type replacements. It has some parameters already.
- Public slots come only from filesystem inspection. There is no generated-entry
  insertion, path remapping, custom action payload, or portable cursor API.
- `Visitor` is explicitly pure: frozen inputs, `&self + Sync`, possibly repeated
  or speculative calls. Ordered accumulated mappings cannot be hidden in those
  callbacks without changing their contract.
- `PlanWalk` yields divergent immediate plans, not completed subtree entities.
  Equal batches still run steering but are not retained as public plans.

The new stateful driver should own ordered state and continuation. Existing
inspection is a reusable component, not itself a suitable stateful callback loop.

## Work avoidance: specialize the remaining question

| Known condition | Work that can disappear | Remaining work |
| --- | --- | --- |
| Entire subtree retained | All child observation, comparison and mutation | Parent enumeration; already implemented early preservation |
| Source directory, absent destination | Destination listing and paired comparison throughout the new subtree | Source enumeration, required metadata, file installation |
| All descendants use the same copy rule | Repeated general policy selection | Simple source-kind handling and copying |
| Target-only subtree retained by update policy | Recursive inspection/removal | Recognize the boundary; already supported |
| Target-only subtree wholly selected for removal | Source comparison and repeated general policy | Removal work; optional descendant callbacks require enumeration |
| Known file list supplied by caller | Discovery and sorting of unrelated names | Install listed leaves; current `seed_leaves` is precedent |
| Same leaf under the chosen comparison | Content transfer | Existing stat-key/literal-link comparison; not byte equality |

The current source-only copier already supplies `target = None` for descendant
inspection. It still constructs normal observation/action batches and calls the
visitor. Thus the next useful shortcut is **inherited assessment mode**, not
another way to discover that the destination is absent.

A recurse tip could carry source origin, destination mapping, and a mode such as
`Compare`, `CopySource`, `RemoveTarget`, or `Preserve`. The policy chooses the mode
once at a suitable boundary. If child-specific exclusions or generation remain,
carry only those residual rules. "Most files" does not imply "all descendants";
the shortcut must retain the exceptions the caller actually supplied.

`CopyTree` can be an action descriptor whose handler uses this mode. A filesystem
tree still normally requires one enumeration and creation per entry; reflinking
a file is not a constant-time directory clone. Treat an opaque whole-tree backend
operation as a separate optional capability. An action counted as one item need
not be bounded work: resumable handlers should expose inner traversal steps when
N-per-invocation control is meant to reach inside a large directory.

Assessment should request the facts its decision or selected handler actually
needs. Source-only classification does not need a target mtime; copying with
permission/mtime preservation still needs the source attributes. Enumeration
hints can answer kind-only questions without a stat on supporting backends. Thus
"less assessment" and "no metadata work" are different achievable savings.

Earlier [M01](file:///home/rektide/src/diff-dirs/design/performance/opportunities0.sol61x.md#m01)
and [C04](file:///home/rektide/src/diff-dirs/design/performance/opportunities0.sol61x.md#c04)
already distinguish selective observation, expanded traversal and wholesale
replacement. Their historical implementation descriptions predate the current
visitor pipeline. Whole-subtree replacement of an existing paired tree is a
different operation from skipping needless comparison in a genuinely new tree;
it can change otherwise untouched leaves and is not selected automatically here.

## Generated entries and path mappings

Make generated content an explicit logical input or action output:

```text
input origin: physical source | generated bytes | generator descriptor
destination: logical relative path, possibly remapped
attributes: domain-owned facts used by assessment
selected action: handler identifier + schema version + parameters
```

A generated entry does not claim to have appeared in `read_dir`. A projection can
merge physical names with virtual entries, or an action can emit additional tasks.
Either mechanism needs an explicit same-destination rule. That is useful even with
no concurrent writers: a generated config and copied config can target one name.

For small generated files, bytes can travel in the cursor. For larger/derived
files, a serializable generator identifier and parameters can retain the recipe.
Process-local closures, open descriptors and function pointers are not the
portable continuation representation. If generation depends on children, store
the accumulator in the active frame and emit the file during finalization.

This is a genuine extension to the current physical-name-only action model.
Adding a KV field to today's enum alone cannot manufacture a new slot or path.

## Extensible descriptors and dispatch

Lean toward a small descriptor envelope with a stable handler ID, payload version,
typed parameters, and optional application attributes. An application's typed
enum remains a convenient initial payload. A serialized descriptor must identify
which registered handler understands it; a KV map alone supplies no behavior.

Useful candidate handlers include copy-leaf, copy-tree with inherited mode,
write-generated, map-path, preserve, remove and finalize-directory. These are not
all new primitive filesystem operations: several are compositions which expand
into child tasks. Keep traversal mode and content origin explicit rather than
inventing a separate enum variant for every combination.

The dispatcher resolves a descriptor to executable code. A sequential driver
owns the active frames, accumulators and cursor. Pure observation remains usable
independently, including the old add/change/remove display. This boundary also
allows the existing filesystem dispatcher to serve as one handler family.

Do not gate this driver on a nonempty physical `DirDiff`: generated output or a
transformation can require work when the physical pair compares equal. Today's
paired walker discards clean plans after steering, which is consistent with its
current constrained actions but is not a sufficient publication rule for new
handlers. Likewise, `disposition` need not become another mandatory type: selected
action plus child traversal mode already expresses the useful decision.

## Completion and continuation

Post-order results are a useful option: enter entity, process children, finalize,
yield the completed entity. Execution can still create a container before its
children and remove/finalize it afterward. Returning a completed parent need not
retain every child result; an accumulator can suffice.

A cursor needs unfinished frames and any accumulators/mappings necessary for
their continuation, not a record of all completed mutations. Count assessment
steps separately from completed results: one step may yield no completed parent,
and finishing a child may complete several ancestor frames.

The demo should make its unit explicit. Serialization describes a saved position;
it is not an exactly-once execution promise. A stale cursor can revisit work.
State evolution should stay in the driver, outside speculative pure visitors.

## Narrow JJ boundary

Ordinary directory copying needs no JJ behavior. JW contributes policy and any
metadata-specific setup:

- For an existing workspace, preserving the target's `.jj` and suppressing the
  donor's `.jj` keeps target identity in place. Early pair-wide preservation is
  already a suitable mechanism. This alone does not move its existing working
  revision to the donor's revision; plain copying and revision advancement are
  different requested outcomes.
- For a new workspace, no target `.jj` exists to preserve. Native initialization
  or an explicit JJ adapter supplies the binding; generic copying does not know
  how to invent it.
- Mapping donor `.jj` to another name produces a copy at that other name; it does
  not itself create the intended target working-copy identity.
- If desired tracked contents differ from the donor, some owner must supply that
  delta. Generic copying can consume it; it need not discover it through JJ.

The cache experiment remains interesting as **reuse of already-computed work**:
clone donor S, convey its useful native tree knowledge, apply only S→D. Whether
JJ accepts that knowledge without rereading affects performance. It need not
expand into a donor-validity or concurrent-mutation research program.

## Connections

- [Existing integration](visitor0.sol61xh.md) records the actual production slice;
  this proposal extends its immutable-slot and pure-visitor boundaries.
- [Earlier scope design](design0.gpt6a.md) supplies separation of retained regions
  and deletion authority. Inherited modes should preserve those decisions while
  avoiding repeated assessment.
- [Prior implementation checkpoint](implementation0.gpt6a.md) retains the open
  broad-transfer policy question. The cursor demo does not choose that default.
- [Native-state review](jj-state-review0.gpt6a.md) distinguishes working-copy
  identity from reusable file-state knowledge. The human's present assumption of
  a good donor narrows the next investigation to useful transfer mechanics.
