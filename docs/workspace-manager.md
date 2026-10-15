---
type: Guide
title: Interactive workspace manager
description: Navigation, selection, status, and deliberate cleanup in the terminal workspace manager.
resource: /docs/workspace-manager.md
status: maintained
generated:
  by: model:openai/gpt-6-astra
  at: 2026-10-15
sources:
  - resource: "opencode://ses_ebf068824ffeRuaEQy26Krx5u6"
    title: Preserved the existing manager reference while synthesizing the canonical README and skill
  - resource: /README.md
    title: Existing interactive manager instructions before the three-take synthesis
---

# Interactive workspace manager

In a terminal, bare `jw` opens the Ratatui workspace manager by default. Use
`jw ui` to select it explicitly. The top-level user configuration can instead
select the legacy list or help view:

```toml
default_command = "ui" # "ui", "list", or "help"
```

A bare invocation outside a terminal prints help; explicit `jw ui` requires a
terminal. [Shell integration](../README.md#shell-setup) lets the manager change
your shell's directory when you select a workspace. Re-source it after upgrading.

## Navigate and select

| Key | Action |
| --- | --- |
| Arrows or `j` / `k` | Move between rows |
| `Space` | Mark a row |
| `a` / `c` | Select visible rows / clear selection |
| `/` | Search |
| `Enter` | Switch to the highlighted workspace and exit |
| `f` | Cycle filters |
| `1`–`4` | Show all, integrated bookmarks, unfinished work, or problems |
| `i` | Refresh the highlighted workspace's file state |
| `n` | Create from an editable trunk, highlighted-workspace, or custom revision |
| `h` | Open the read-only health report |
| `y` | Request copying the path through the terminal clipboard protocol |
| `Tab` | Open details on narrow terminals |
| `m` | Toggle mouse controls outside text-entry fields |
| `?` | Show help |

Selections survive searches and filters; the header counts hidden selections too.
With mouse controls enabled, click a checkbox to select. Toggle them off to drag
and select terminal text; keyboard controls still work. The footer shows the mode.
Clipboard support depends on the terminal; the path also appears in the notice.

Colors follow your terminal palette: cyan identifies focus and shortcuts, magenta
bookmarks, green integrated work and success, yellow unfinished work and warnings,
and red conflicts, errors, and destructive actions. Text labels accompany colors.

## Read status

Bookmark integration and workspace work are separate columns: a merged bookmark
can still have newer workspace work. “In trunk” means local graph ancestry, not
squash-merge equivalence or GitHub merge status. The resolved trunk appears above
the table. File state starts as unchecked; `i` refreshes the highlighted workspace,
and removal refreshes its selected targets.

## Remove or prune

`d` previews removal of marked rows, or the highlighted row when none are marked.
`p` opens **Prune missing workspaces**, which forgets registrations whose
directories are already gone. Remove also permanently deletes existing directories.
The manager cannot remove current or default workspaces.

Risky rows are excluded initially; `r` explicitly includes them. The preview
reports how many rows will be removed, skipped, or blocked and explains why.
If eligible workspaces contain files not recorded by JJ, choose `i` to delete the
listed files or `s` to skip those workspaces. These can include ignored caches,
build output, and private files; JJ cannot restore them after deletion. Enter waits
for this choice and does nothing if no workspace is eligible. JJ's operation log
is not a directory backup.

In the preview, `b` toggles keeping or deleting eligible associated **local**
bookmarks. The last confirmed choice is remembered across repositories in
`$XDG_STATE_HOME/jj-waltz/ui.json`, falling back to
`~/.local/state/jj-waltz/ui.json`. The initial choice is keep. Bookmarks associated
with surviving managed workspaces are preserved; remote bookmarks are untouched.
Unmanaged workspaces never have inferred bookmarks deleted. If saving the choice
fails, removal still runs and reports a separate preference warning.

Each target is revalidated before mutation. Changed targets require another
review. Independent rows continue after a failure; results identify partial
progress, and failed rows remain selected for an explicit retry. No batch rollback
or automatic repair is performed.

See [lifecycle safeguards](../skills/jj-waltz/references/lifecycle.md#remove) for
CLI cleanup, native-app ownership, and preservation options.
