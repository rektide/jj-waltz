//! Workspace mutation orchestration: creation with CoW materialization and
//! owned rollback, switching with recorded navigation state, and removal with
//! ownership-verified forget. Queries over the workspace inventory live in
//! the state crate.
use anyhow::{Context, Result, anyhow, bail};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use jj_waltz_native_jj::jj::{self, JjClient};
use jj_waltz_native_jj::paths;
use jj_waltz_transfer_workspace as cow;
use jj_waltz_workspace_state::inventory::{validate_workspace_name, workspace_bookmark_file, workspace_state_file};

pub use jj_waltz_workspace_state::inventory::WorkspaceInventory;
use jj_waltz_workspace_state::metadata::{ManagedWorkspaceMetadata, WorkspaceMetadataStore};
use jj_waltz_workspace_state::ownership;

const REMOVE_DIRECTORY_ATTEMPTS: usize = 4;
const REMOVE_DIRECTORY_RETRY_DELAY: Duration = Duration::from_millis(25);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchResult {
    pub workspace: String,
    pub path: PathBuf,
    pub created: bool,
    pub bookmark: Option<String>,
    pub relative_subdir: Option<PathBuf>,
    from_workspace: String,
    from_path: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct SwitchOptions {
    pub preserve_subdir: bool,
}

#[derive(Debug, Clone)]
pub struct AddOptions {
    /// Full commit ID resolved before any workspace mutation.
    pub base_commit_id: String,
    pub bookmark: Option<String>,
    /// Clone tracked files from this checkout root instead of letting JJ write them.
    pub clone_from: Option<PathBuf>,
    pub colocate: bool,
    pub destination: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddResult {
    pub workspace: String,
    pub path: PathBuf,
    pub bookmark: Option<String>,
    pub creation_operation_id: String,
    pub creation_base_commit_id: String,
    pub owned_git_worktree: Option<ownership::OwnedGitWorktree>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RemovalPlan {
    pub workspace: String,
    pub path: PathBuf,
    pub delete_dir: bool,
    pub bookmarks: Vec<String>,
    managed_metadata: Option<ManagedWorkspaceMetadata>,
    managed_store: Option<WorkspaceMetadataStore>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovalResult {
    pub workspace: String,
    pub path: PathBuf,
    pub deleted_dir: bool,
    pub deleted_bookmarks: Vec<String>,
}

pub fn switch_workspace(inventory: &WorkspaceInventory, target: &str, options: &SwitchOptions) -> Result<SwitchResult> {
    let current_name = inventory.current_name().to_owned();
    let current_root = inventory.current_root().to_path_buf();
    let current_dir = env::current_dir().context("failed to determine current directory")?;
    let relative_subdir = if options.preserve_subdir {
        current_dir.strip_prefix(&current_root).ok().map(Path::to_path_buf)
    } else {
        None
    };

    let resolved_name = inventory.resolve(target)?;
    if !inventory.contains(&resolved_name) {
        bail!("workspace does not exist: {resolved_name}")
    }
    let target_path = inventory.root(&resolved_name)?;

    Ok(SwitchResult {
        workspace: resolved_name,
        path: target_path,
        created: false,
        bookmark: None,
        relative_subdir,
        from_workspace: current_name,
        from_path: current_root,
    })
}

pub fn switch_to_created_workspace(inventory: &WorkspaceInventory, created: &AddResult, options: &SwitchOptions) -> Result<SwitchResult> {
    let current_name = inventory.current_name().to_owned();
    let current_root = inventory.current_root().to_path_buf();
    let current_dir = env::current_dir().context("failed to determine current directory")?;
    let relative_subdir = if options.preserve_subdir {
        current_dir.strip_prefix(&current_root).ok().map(Path::to_path_buf)
    } else {
        None
    };

    Ok(SwitchResult {
        workspace: created.workspace.clone(),
        path: created.path.clone(),
        created: true,
        bookmark: created.bookmark.clone(),
        relative_subdir,
        from_workspace: current_name,
        from_path: current_root,
    })
}

pub fn record_switch(result: &SwitchResult) -> Result<()> {
    remember_previous_workspace(&result.from_workspace, &result.from_path, &result.workspace, &result.path)
}

pub fn add_workspace(inventory: &WorkspaceInventory, target: &str, options: &AddOptions) -> Result<AddResult> {
    let name = inventory.resolve(target)?;
    if inventory.contains(&name) {
        bail!("workspace already exists: {name}")
    }
    add_workspace_by_name_with_inventory(&name, options, inventory)
}

pub fn plan_remove_workspace(inventory: &WorkspaceInventory, token: Option<&str>, delete_dir: bool) -> Result<RemovalPlan> {
    let name = match token {
        Some(value) => inventory.resolve(value)?,
        None => inventory.current_name().to_owned(),
    };

    if name == inventory.default_name()? {
        bail!("refusing to remove the default workspace")
    }

    let path = inventory.root(&name)?;
    if delete_dir && name == inventory.current_name() {
        bail!("cannot delete the current workspace directory; switch away first")
    }

    let store = metadata_store()?;
    let managed_metadata = store.get(&name)?;
    ownership::ensure_removal_allowed(&JjClient::current()?, &name, Some(&path), managed_metadata.as_ref())?;
    let bookmarks = match &managed_metadata {
        Some(metadata) => bookmarks_for_managed_workspace(metadata)?,
        None => bookmarks_for_workspace(&name, &path)?,
    };

    Ok(RemovalPlan {
        bookmarks,
        workspace: name,
        path,
        delete_dir,
        managed_store: managed_metadata.as_ref().map(|_| store),
        managed_metadata,
    })
}

pub fn execute_remove_workspace(plan: RemovalPlan, delete_bookmarks: bool) -> Result<RemovalResult> {
    let client = JjClient::current()?;
    let current_metadata = metadata_store()?.get(&plan.workspace)?;
    ownership::ensure_removal_allowed(&client, &plan.workspace, Some(&plan.path), current_metadata.as_ref())?;
    if current_metadata != plan.managed_metadata {
        bail!("workspace metadata changed during removal; review it again")
    }
    let deleted_dir = plan.delete_dir && plan.path.is_dir();
    let native_removal = deleted_dir && client.supports_workspace_removal()?;
    if native_removal {
        ownership::remove_workspace(&client, &plan.workspace, &plan.path, current_metadata.as_ref())?;
    } else {
        ownership::forget_workspace(&client, &plan.workspace, Some(&plan.path), current_metadata.as_ref())?;
    }

    let mut deleted_bookmarks = Vec::new();
    if delete_bookmarks && !plan.bookmarks.is_empty() {
        let mut args = vec!["bookmark".to_owned(), "delete".to_owned()];
        args.extend(plan.bookmarks.iter().cloned());
        JjClient::current()?.run(&args).with_context(|| {
            format!(
                "partial removal: workspace {} was forgotten, but associated bookmark deletion failed",
                plan.workspace
            )
        })?;
        deleted_bookmarks.clone_from(&plan.bookmarks);
    }

    if let Some(metadata) = &plan.managed_metadata {
        let store = plan.managed_store.as_ref().expect("managed metadata plan retains its store");
        let removed = store.remove_if_matches(metadata).with_context(|| {
            format!(
                "{}; managed workspace metadata cleanup failed",
                partial_removal_progress(&plan.workspace, &deleted_bookmarks)
            )
        })?;
        if !removed {
            bail!(
                "{}; workspace metadata changed during removal and was retained: {}",
                partial_removal_progress(&plan.workspace, &deleted_bookmarks),
                metadata.workspace_name
            )
        }
    }

    if native_removal {
        match fs::symlink_metadata(&plan.path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            outcome => {
                let detail = match outcome {
                    Err(error) => format!("; cannot verify directory deletion: {error}"),
                    Ok(_) => String::new(),
                };
                bail!(
                    "{}; workspace directory remains at {}{}",
                    partial_removal_progress(&plan.workspace, &deleted_bookmarks),
                    plan.path.display(),
                    detail
                )
            }
        }
    } else if deleted_dir {
        remove_workspace_directory(&plan.path).with_context(|| {
            format!(
                "{}; workspace directory remains at {}",
                partial_removal_progress(&plan.workspace, &deleted_bookmarks),
                plan.path.display()
            )
        })?;
    }

    Ok(RemovalResult {
        workspace: plan.workspace,
        path: plan.path,
        deleted_dir,
        deleted_bookmarks,
    })
}

fn partial_removal_progress(workspace: &str, deleted_bookmarks: &[String]) -> String {
    let mut progress = format!("partial removal: workspace {workspace} was forgotten");
    match deleted_bookmarks {
        [] => {}
        [bookmark] => progress.push_str(&format!(" and bookmark {bookmark} was deleted")),
        bookmarks => progress.push_str(&format!(" and bookmarks {} were deleted", bookmarks.join(", "))),
    }
    progress
}

fn remove_workspace_directory(path: &Path) -> io::Result<()> {
    remove_workspace_directory_with(path, |path| fs::remove_dir_all(path))
}

fn remove_workspace_directory_with(path: &Path, mut remove: impl FnMut(&Path) -> io::Result<()>) -> io::Result<()> {
    for attempt in 1..=REMOVE_DIRECTORY_ATTEMPTS {
        match remove(path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::DirectoryNotEmpty && attempt < REMOVE_DIRECTORY_ATTEMPTS => {
                thread::sleep(REMOVE_DIRECTORY_RETRY_DELAY);
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("directory removal loop always returns")
}

fn add_workspace_by_name_with_inventory(name: &str, options: &AddOptions, inventory: &WorkspaceInventory) -> Result<AddResult> {
    validate_workspace_name(name)?;
    let path = match &options.destination {
        Some(path) => path.clone(),
        None => workspace_dir_for_name(name, inventory)?,
    };
    require_absent_workspace_path(&path)?;

    let mut args = vec!["workspace".to_owned(), "add".to_owned(), "--name".to_owned(), name.to_owned()];

    args.push("--revision".to_owned());
    args.push(options.base_commit_id.clone());

    if options.clone_from.is_some() {
        // JJ writes no files; the clone below supplies them.
        args.push("--sparse-patterns".to_owned());
        args.push("empty".to_owned());
    }

    let command_path = if options.colocate {
        paths::external_command_path(&path)
    } else {
        std::borrow::Cow::Borrowed(path.as_path())
    };
    args.push(command_path.display().to_string());
    let client = JjClient::current()?;
    if options.colocate {
        client.require_workspace_colocation()?;
        args.push("--colocate".to_owned());
    } else if client.run(["workspace", "add", "--help"])?.stdout()?.contains("--no-colocate") {
        args.push("--no-colocate".to_owned());
    }
    let previous_registrations = if options.colocate {
        ownership::registration_paths(&client)?
    } else {
        Vec::new()
    };
    let added = client.run(&args);
    if added.is_err()
        && !client
            .workspace_names()
            .with_context(|| format!("cannot verify add outcome; checkout retained at {} for inspection", path.display()))?
            .iter()
            .any(|candidate| candidate == name)
    {
        if fs::symlink_metadata(&path).is_ok() {
            let error = added.expect_err("failed add checked above");
            bail!(
                "{error}; unregistered checkout retained at {} because creation ownership could not be verified",
                path.display()
            )
        }
        return added.map(|_| unreachable!());
    }
    let owned_git_worktree = if options.colocate {
        Some(
            ownership::record_created(&client, name, &path, &previous_registrations)
                .with_context(|| format!("cannot verify created Git topology at {}; checkout retained for inspection", path.display()))?,
        )
    } else {
        None
    };
    if let Err(error) = added {
        let cleanup = rollback_workspace_parts(name, &path, None, owned_git_worktree.as_ref()).err();
        if let Some(cleanup) = cleanup {
            bail!("{error}; cleanup also failed: {cleanup}")
        }
        return Err(error);
    }

    if let Some(source) = &options.clone_from {
        match materialize_clone(source, &path) {
            Ok(None) => {}
            Ok(Some(reason)) => {
                // Recreate through JJ's ordinary path. That inherits the source
                // sparse patterns verbatim, including paths with whitespace.
                rollback_workspace_parts(name, &path, None, None)?;
                let mut fallback = options.clone();
                fallback.clone_from = None;
                let warning = format!("copy-on-write is unavailable for workspace {name}, using ordinary JJ workspace creation: {reason}");
                let mut result = add_workspace_by_name_with_inventory(name, &fallback, inventory).with_context(|| warning.clone())?;
                result.warning = Some(warning);
                return Ok(result);
            }
            Err(error) => return Err(rollback_failed_add(error, name, &path, None)),
        }
    }

    // Capture provenance before bookmark creation records another JJ operation.
    let creation_operation_id = match client.operation_id() {
        Ok(operation_id) => operation_id,
        Err(error) => {
            let cleanup = rollback_workspace_parts(name, &path, None, owned_git_worktree.as_ref()).err();
            if let Some(cleanup) = cleanup {
                bail!("{error}; cleanup also failed: {cleanup}")
            }
            bail!(error)
        }
    };

    if let Some(bookmark) = &options.bookmark {
        if let Err(error) = JjClient::new(&path).run(["bookmark", "create", bookmark, "-r", "@"]) {
            let cleanup = rollback_workspace_parts(name, &path, None, owned_git_worktree.as_ref()).err();
            if let Some(cleanup) = cleanup {
                bail!("{error}; cleanup also failed: {cleanup}")
            }
            bail!(error)
        }
        if let Err(error) = fs::write(workspace_bookmark_file(&path), format!("{bookmark}\n")) {
            let cleanup = rollback_workspace_parts(name, &path, Some(bookmark), owned_git_worktree.as_ref()).err();
            if let Some(cleanup) = cleanup {
                bail!("failed to record workspace bookmark: {error}; cleanup also failed: {cleanup}")
            }
            bail!("failed to record workspace bookmark: {error}")
        }
    }

    Ok(AddResult {
        workspace: name.to_owned(),
        path,
        bookmark: options.bookmark.clone(),
        creation_operation_id,
        creation_base_commit_id: options.base_commit_id.clone(),
        owned_git_worktree,
        warning: None,
    })
}

/// Fill a sparse-empty workspace from a copy-on-write clone of another checkout.
///
/// Only the files JJ tracks in the source's working-copy commit are cloned, so
/// ignored and untracked files never reach the new workspace. JJ adopts the clone
/// through the source's working-copy state. Return a reason when the caller must
/// recreate through JJ's ordinary checkout path. Restoring `@` then rewrites every
/// file that differs from the creation base, so the result matches a full checkout.
fn materialize_clone(source_root: &Path, path: &Path) -> Result<Option<&'static str>> {
    // The shared clone kernel binds disjoint roots. Creation inside the donor
    // remains supported through the existing ordinary-JJ recreation fallback,
    // never through an unchecked overlapping-tree copy.
    if path.starts_with(source_root) || source_root.starts_with(path) {
        return Ok(Some("nested source/destination roots cannot use the shared clone kernel"));
    }
    let client = JjClient::new(path);
    if client.uses_watchman()? {
        return Ok(Some("the target uses Watchman"));
    }
    let files = JjClient::new(source_root).tracked_files()?;
    cow::clone_files(source_root, &files, path)
        .with_context(|| format!("failed to clone files from {}; retry with --no-cow for a full checkout", source_root.display()))?;
    if !adopt_source_tree_state(&client, source_root, path)? {
        return Ok(Some("JJ could not reuse the source working-copy state"));
    }
    client.run(["restore"])?;
    Ok(None)
}

/// Let the new workspace reuse the source checkout's working-copy state.
///
/// The clone preserves sizes and modification times, so JJ's snapshot only stats
/// files instead of hashing them. The copied state also carries the source's
/// sparse patterns, matching `jj workspace add`'s default. Returns `false` when
/// JJ rejects the copied state and must hash every file instead.
fn adopt_source_tree_state(client: &JjClient, source_root: &Path, path: &Path) -> Result<bool> {
    if client.uses_watchman()? {
        return Ok(false);
    }
    let previous = jj::copy_tree_state(source_root, path)?;
    if client.run(["status"]).is_ok() {
        return Ok(true);
    }
    jj::write_tree_state(path, &previous)?;
    Ok(false)
}

fn rollback_failed_add(error: impl Into<anyhow::Error>, name: &str, path: &Path, bookmark: Option<&str>) -> anyhow::Error {
    let error = error.into();
    match rollback_workspace_parts(name, path, bookmark, None) {
        Ok(()) => error,
        Err(cleanup) => error.context(format!("cleanup also failed: {cleanup}")),
    }
}

fn require_absent_workspace_path(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).context("cannot inspect creation destination"),
        Ok(_) => bail!("directory already exists: {}", path.display()),
    }
}

pub fn preflight_add_workspace(inventory: &WorkspaceInventory, name: &str, destination: Option<&Path>) -> Result<()> {
    validate_workspace_name(name)?;
    if inventory.contains(name) {
        bail!("workspace already exists: {name}")
    }
    let path = match destination {
        Some(path) => path.to_path_buf(),
        None => workspace_dir_for_name(name, inventory)?,
    };
    require_absent_workspace_path(&path)?;
    Ok(())
}

pub fn rollback_added_workspace(result: &AddResult) -> Result<()> {
    rollback_workspace_parts(&result.workspace, &result.path, result.bookmark.as_deref(), result.owned_git_worktree.as_ref())
}

pub fn rollback_added_workspace_with_payload(result: &AddResult, roots: &[PathBuf]) -> Result<()> {
    rollback_workspace_parts_with_payload(
        &result.workspace,
        &result.path,
        result.bookmark.as_deref(),
        result.owned_git_worktree.as_ref(),
        roots,
    )
}

fn rollback_workspace_parts(name: &str, path: &Path, bookmark: Option<&str>, owner: Option<&ownership::OwnedGitWorktree>) -> Result<()> {
    rollback_workspace_parts_with_payload(name, path, bookmark, owner, &[])
}

fn rollback_workspace_parts_with_payload(
    name: &str,
    path: &Path,
    bookmark: Option<&str>,
    owner: Option<&ownership::OwnedGitWorktree>,
    private_payload: &[PathBuf],
) -> Result<()> {
    let client = JjClient::current()?;
    // Do not delete the directory or bookmark if forgetting/registration cleanup fails.
    ownership::forget_created(&client, name, path, owner)?;
    let mut errors = Vec::new();
    // Only after creation ownership is verified may read-only cloned directories
    // be made removable. Never relax source, sibling, or arbitrary target content.
    for root in private_payload {
        #[cfg(unix)]
        cow::tracked::confined_parents(path, root)?;
        if let Err(error) = cow::remove_path(&path.join(root)) {
            errors.push(format!("remove private payload {}: {error:#}", root.display()));
        }
    }
    if let Some(bookmark) = bookmark
        && let Err(error) = JjClient::current()?.run(["bookmark", "delete", bookmark])
    {
        errors.push(format!("delete bookmark {bookmark}: {error}"));
    }
    if path.is_dir()
        && let Err(error) = fs::remove_dir_all(path)
    {
        errors.push(format!("delete directory {}: {error}", path.display()));
    }
    if errors.is_empty() { Ok(()) } else { bail!(errors.join("; ")) }
}

pub fn prune_missing_workspaces() -> Result<Vec<String>> {
    let mut removed = Vec::new();
    let store = metadata_store()?;

    for entry in WorkspaceInventory::load()?.entries().to_vec() {
        match &entry.root {
            Some(path) if path.is_dir() => {}
            _ => {
                let metadata = store.get(&entry.name)?;
                ownership::ensure_removal_allowed(&JjClient::current()?, &entry.name, entry.root.as_deref(), metadata.as_ref())?;
                ownership::forget_workspace(&JjClient::current()?, &entry.name, entry.root.as_deref(), metadata.as_ref())?;
                if let Some(metadata) = metadata
                    && !store.remove_if_matches(&metadata)?
                {
                    bail!("workspace metadata changed during prune and was retained: {}", metadata.workspace_name)
                }
                removed.push(entry.name);
            }
        }
    }

    Ok(removed)
}

fn remember_previous_workspace(from_name: &str, from_root: &Path, to_name: &str, to_root: &Path) -> Result<()> {
    if from_name == to_name {
        return Ok(());
    }

    let mut updates = vec![(workspace_state_file(from_root), format!("{to_name}\n").into_bytes())];
    let to_state_dir = to_root.join(".jj");
    if to_state_dir.is_dir() {
        updates.push((workspace_state_file(to_root), format!("{from_name}\n").into_bytes()));
    }

    let originals = updates.iter().map(|(path, _)| read_optional_file(path)).collect::<Result<Vec<_>>>()?;
    for (index, (path, contents)) in updates.iter().enumerate() {
        if let Err(error) = fs::write(path, contents) {
            let rollback_errors = updates[..=index]
                .iter()
                .zip(&originals[..=index])
                .rev()
                .filter_map(|((path, _), original)| restore_file(path, original.as_deref()).err())
                .map(|error| error.to_string())
                .collect::<Vec<_>>();
            if rollback_errors.is_empty() {
                return Err(error).with_context(|| format!("failed to record workspace state at {}", path.display()));
            }
            bail!(
                "failed to record workspace state at {}: {error}; state rollback also failed: {}",
                path.display(),
                rollback_errors.join("; ")
            )
        }
    }

    Ok(())
}

fn read_optional_file(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(contents) => Ok(Some(contents)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("failed to inspect workspace state {}", path.display())),
    }
}

fn restore_file(path: &Path, contents: Option<&[u8]>) -> Result<()> {
    match contents {
        Some(contents) => fs::write(path, contents).with_context(|| format!("failed to restore workspace state {}", path.display())),
        None => match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error).with_context(|| format!("failed to remove workspace state {}", path.display())),
        },
    }
}

pub fn workspace_dir_for_name(name: &str, inventory: &WorkspaceInventory) -> Result<PathBuf> {
    let default_root = inventory
        .primary_root()
        .map(std::path::Path::to_path_buf)
        .context("cannot choose a workspace destination without a verified primary checkout; use --path")?;
    if inventory.default() == Some(name) {
        Ok(default_root)
    } else {
        let parent = default_root.parent().ok_or_else(|| anyhow!("workspace root has no parent directory"))?;
        let base_name = default_root.file_name().ok_or_else(|| anyhow!("workspace root has no valid basename"))?;
        Ok(parent.join(format!("{}.{name}", base_name.to_string_lossy())))
    }
}

/// Resolve existing ancestors so aliases and symlinked parents cannot hide collisions.
/// Missing parent directories are left for JJ to create.
pub fn normalize_destination(path: &Path) -> Result<PathBuf> {
    let absolute = std::path::absolute(path).context("cannot resolve destination path")?;
    let mut ancestor = absolute.as_path();
    let mut missing = Vec::new();
    loop {
        match ancestor.canonicalize() {
            Ok(mut root) => {
                for component in missing.iter().rev() {
                    root.push(component);
                }
                return Ok(root);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                missing.push(ancestor.file_name().ok_or_else(|| anyhow!("invalid destination path"))?);
                ancestor = ancestor.parent().ok_or_else(|| anyhow!("invalid destination path"))?;
            }
            Err(error) => return Err(error).context("cannot resolve destination path"),
        }
    }
}

pub fn name_for_destination(inventory: &WorkspaceInventory, explicit_name: Option<&str>, destination: &Path) -> Result<String> {
    let registered = inventory.entries().iter().find(|entry| {
        entry
            .root
            .as_deref()
            .is_some_and(|root| normalize_destination(root).ok().as_deref() == Some(destination))
    });
    let name = match explicit_name {
        Some(name) => inventory.resolve(name)?,
        None => match registered {
            Some(entry) => entry.name.clone(),
            None => {
                let primary = inventory
                    .primary_root()
                    .context("cannot infer a workspace name without a verified primary checkout; provide NAME")?;
                let repo = primary
                    .file_name()
                    .and_then(|name| name.to_str())
                    .context("primary checkout has no valid basename")?;
                let basename = destination
                    .file_name()
                    .and_then(|name| name.to_str())
                    .context("destination has no valid basename")?;
                basename
                    .strip_prefix(&format!("{repo}-"))
                    .filter(|suffix| !suffix.is_empty())
                    .unwrap_or(basename)
                    .to_owned()
            }
        },
    };
    validate_workspace_name(&name)?;
    if let Some(entry) = registered
        && entry.name != name
    {
        bail!("destination is registered as workspace {}, not {name}", entry.name)
    }
    if inventory.contains(&name) {
        let root = normalize_destination(&inventory.root(&name)?)?;
        if root != destination {
            bail!("workspace {name} is already registered at {}, not {}", root.display(), destination.display())
        }
    }
    Ok(name)
}

pub fn workspace_base_root(current_root: &Path, current_name: &str) -> Result<PathBuf> {
    let parent = current_root.parent().ok_or_else(|| anyhow!("workspace root has no parent directory"))?;
    let mut base = current_root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("workspace root has no valid basename"))?
        .to_owned();

    let suffix = format!(".{current_name}");
    if current_name != "default" && base.ends_with(&suffix) {
        let new_len = base.len() - suffix.len();
        base.truncate(new_len);
    } else if current_name != "default"
        && base == current_name
        && base.contains('.')
        && let Some((prefix, _)) = base.rsplit_once('.')
    {
        base = prefix.to_owned();
    }

    Ok(parent.join(base))
}

fn bookmarks_for_workspace(name: &str, root: &Path) -> Result<Vec<String>> {
    let bookmark_path = workspace_bookmark_file(root);
    let recorded = match fs::read_to_string(&bookmark_path) {
        Ok(contents) => {
            let names = contents.lines().map(str::trim).filter(|line| !line.is_empty()).collect::<Vec<_>>();
            if names.len() != 1 {
                bail!("workspace bookmark record is invalid: {}", bookmark_path.display())
            }
            Some(names[0].to_owned())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read workspace bookmark {}", bookmark_path.display()));
        }
    };
    let client = JjClient::current()?;
    let operation_id = client.operation_id()?;
    let candidates = match &recorded {
        Some(_) => client.local_bookmark_names_at(&operation_id)?,
        None => client.local_bookmark_names_for_workspace_at(&operation_id, name)?,
    };
    if let Some(recorded) = recorded {
        return Ok(candidates.into_iter().filter(|candidate| candidate == &recorded).collect());
    }
    let suffix = format!("/{name}");
    Ok(candidates
        .into_iter()
        .filter(|candidate| candidate == name || candidate.ends_with(&suffix))
        .collect())
}

fn bookmarks_for_managed_workspace(metadata: &ManagedWorkspaceMetadata) -> Result<Vec<String>> {
    let Some(recorded) = &metadata.associated_bookmark else {
        return Ok(Vec::new());
    };
    let client = JjClient::current()?;
    let operation_id = client.operation_id()?;
    Ok(client
        .local_bookmark_names_at(&operation_id)?
        .into_iter()
        .filter(|candidate| candidate == recorded)
        .collect())
}

fn metadata_store() -> Result<WorkspaceMetadataStore> {
    let client = JjClient::current()?;
    WorkspaceMetadataStore::from_repo_config_path(client.repo_config_path()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retries_directory_not_empty_during_workspace_cleanup() {
        let tempdir = tempfile::tempdir().expect("create temporary directory");
        let workspace = tempdir.path().join("workspace");
        fs::create_dir(&workspace).expect("create workspace directory");
        let mut attempts = 0;

        remove_workspace_directory_with(&workspace, |path| {
            attempts += 1;
            if attempts == 1 {
                Err(io::Error::from(io::ErrorKind::DirectoryNotEmpty))
            } else {
                fs::remove_dir_all(path)
            }
        })
        .expect("retry directory removal");

        assert_eq!(attempts, 2);
        assert!(!workspace.exists());
    }
}
