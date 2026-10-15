//! New-target ignored payload phase. Existing switches never construct this plan.
use super::PlannedWorkspace;
use crate::cow::ignored::{Patterns, Policy};
use crate::jj::ResolvedRevision;
use crate::workspace::WorkspaceInventory;
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub(super) struct IgnoredPlan {
    pub diagnostics: Vec<String>,
    #[cfg(unix)]
    proof: Option<unix::Proof>,
}

impl IgnoredPlan {
    pub fn roots(&self) -> &[PathBuf] {
        #[cfg(unix)]
        if let Some(proof) = &self.proof {
            return &proof.roots;
        }
        &[]
    }
    pub fn prepare(
        inventory: &WorkspaceInventory,
        base: &ResolvedRevision,
        policy: &Policy,
        plans: &[PlannedWorkspace],
        asserted_source: Option<&str>,
    ) -> Result<Self> {
        let patterns = Patterns::compile(&policy.patterns)?;
        if policy.patterns.is_empty() {
            return Ok(Self::default());
        }
        #[cfg(unix)]
        {
            unix::prepare(inventory, base, patterns, plans, asserted_source)
        }
        #[cfg(not(unix))]
        {
            let _ = (base, plans, asserted_source);
            let selection = patterns.select(inventory.current_root(), [], |_| {
                anyhow::bail!(
                    "ignored payload copying requires Unix filesystem identity and real CoW support"
                )
            })?;
            Ok(Self {
                diagnostics: selection.skipped,
            })
        }
    }

    pub fn copy_to(&self, target: &Path, config_root: Option<&Path>) -> Result<()> {
        #[cfg(unix)]
        if let Some(proof) = &self.proof {
            proof.copy_to(target, config_root)?;
        }
        #[cfg(not(unix))]
        let _ = (target, config_root);
        Ok(())
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use crate::cow::ignored::{require_untracked, validate_selected};
    use crate::cow::payload::{self, Entry, IgnoreRules, Stamp};
    use crate::{cow, jj::JjClient, links};
    use anyhow::{Context, bail};
    use std::collections::BTreeSet;
    use std::fs;
    use std::io::ErrorKind;
    use std::path::PathBuf;

    #[derive(Debug)]
    pub(super) struct Proof {
        source: PathBuf,
        source_identity: (u64, u64),
        source_jj_identity: (u64, u64),
        repository: PathBuf,
        source_commit: String,
        source_entries: Vec<Entry>,
        source_rules: IgnoreRules,
        target_rules: IgnoreRules,
        pub(super) roots: Vec<PathBuf>,
        stamp: Vec<Stamp>,
        digest: String,
        base: String,
    }

    pub(super) fn prepare(
        inventory: &WorkspaceInventory,
        base: &ResolvedRevision,
        patterns: Patterns,
        plans: &[PlannedWorkspace],
        asserted_source: Option<&str>,
    ) -> Result<IgnoredPlan> {
        let source = inventory.current_root();
        let source_identity = payload::directory_identity(source)?;
        let source_jj_identity = payload::directory_identity(&source.join(".jj"))?;
        let repository = crate::context::repository_path(source).map_err(anyhow::Error::msg)?;
        let client = JjClient::new(source);
        // Explicit --at freezes target intent independently; payload authorization
        // still comes from a fresh snapshot of the actual donor checkout.
        client.run(["status"])?;
        let operation = client.operation_id()?;
        let source_commit = client.resolve_one_at(&operation, "@")?.commit_id;
        if asserted_source.is_some_and(|commit| commit != source_commit) {
            bail!("source changed since --source-at was asserted; pause writers and retry");
        }
        let source_entries = payload::entries(&client, &operation, &source_commit)?;
        let source_rules =
            IgnoreRules::new(&client, &operation, &source_commit, &source_entries, true)?;
        let selected = patterns.select(
            source,
            source_entries.iter().map(|entry| entry.path.as_path()),
            |path| source_rules.is_ignored_directory(path),
        )?;
        let mut plan = IgnoredPlan {
            diagnostics: selected.skipped,
            proof: None,
        };
        // Missing matches are success, without requiring a clone-capable filesystem.
        if selected.roots.is_empty() {
            return Ok(plan);
        }
        // Creation inside the donor changes its whole-checkout stamp even when
        // the destination is ignored. Do not exempt ancestors from that proof or
        // risk recursively copying a destination inside a selected payload root.
        // Check the whole batch before creating any workspace, including siblings.
        for target in plans {
            let destination = match &target.destination {
                Some(path) => path.clone(),
                None => crate::workspace::workspace_dir_for_name(&target.name, inventory)?,
            };
            if crate::workspace::normalize_destination(&destination)?.starts_with(source) {
                bail!(
                    "cannot copy ignored payload into a workspace nested within its source: {}; choose an external --path or use --no-copy-ignored",
                    destination.display()
                );
            }
        }
        let target_entries = payload::entries(&client, &operation, &base.commit_id)?;
        let target_rules =
            IgnoreRules::new(&client, &operation, &base.commit_id, &target_entries, false)?;
        let tracked = target_entries
            .iter()
            .map(|entry| entry.path.as_path())
            .collect::<BTreeSet<_>>();
        for path in &selected.roots {
            require_untracked(path, &tracked)
                .context("ignored payload overlaps the desired target tree")?;
            target_rules
                .require_ignored(path)
                .context("ignored payload is not ignored in the desired target tree")?;
        }
        let digest = payload::payload_digest(source, &selected.roots)?;
        let stamp = payload::stamp(source)?;
        plan.proof = Some(Proof {
            source: source.to_owned(),
            source_identity,
            source_jj_identity,
            repository,
            source_commit,
            source_entries,
            source_rules,
            target_rules,
            roots: selected.roots,
            stamp,
            digest,
            base: base.commit_id.clone(),
        });
        Ok(plan)
    }

    impl Proof {
        fn validate_source(&self) -> Result<()> {
            if payload::directory_identity(&self.source)? != self.source_identity
                || payload::directory_identity(&self.source.join(".jj"))? != self.source_jj_identity
                || crate::context::repository_path(&self.source).map_err(anyhow::Error::msg)?
                    != self.repository
            {
                bail!("source directory or repository identity changed since the creation plan");
            }
            let client = JjClient::new(&self.source);
            let operation = client.operation_id()?;
            if client.resolve_one_at(&operation, "@")?.commit_id != self.source_commit
                || payload::stamp(&self.source)? != self.stamp
            {
                bail!(
                    "source changed since the ignored payload was frozen; pause writers and retry"
                );
            }
            validate_selected(
                &self.source,
                &self.roots,
                self.source_entries.iter().map(|entry| entry.path.as_path()),
                |path| self.source_rules.is_ignored_directory(path),
            )?;
            if payload::payload_digest(&self.source, &self.roots)? != self.digest {
                bail!("source ignored payload changed since the creation plan");
            }
            Ok(())
        }

        pub(super) fn copy_to(&self, target: &Path, config_root: Option<&Path>) -> Result<()> {
            self.validate_source()?;
            payload::directory_identity(target)?;
            if crate::context::repository_path(target).map_err(anyhow::Error::msg)?
                != self.repository
            {
                bail!("new workspace no longer belongs to the frozen source repository");
            }
            let client = JjClient::new(target);
            let operation = client.operation_id()?;
            if !client
                .run_at(
                    &operation,
                    ["diff", "--from", &self.base, "--to", "@", "--summary"],
                )?
                .stdout()?
                .is_empty()
            {
                bail!("new workspace no longer matches its frozen creation base");
            }
            if let Some(config_root) = config_root
                && let Some(config) = links::load_link_config(config_root)?
            {
                for check in links::inspect_loaded_workspace_links(&config, target) {
                    if check.source != check.target
                        && self.roots.iter().any(|root| {
                            let root = target.join(root);
                            check.source.starts_with(&root) || root.starts_with(&check.source)
                        })
                    {
                        bail!(
                            "configured link overlaps private ignored payload: {}; use a source-equals-target local override to keep it private",
                            check.source.display()
                        );
                    }
                }
            }
            for path in &self.roots {
                self.target_rules.require_ignored(path)?;
                payload::confined_parents(target, path)?;
                match fs::symlink_metadata(target.join(path)) {
                    Err(error) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => return Err(error.into()),
                    Ok(_) => bail!(
                        "ignored payload target is obstructed: {}",
                        target.join(path).display()
                    ),
                }
            }
            // Independent of the tracked checkout mode: no bytes/hardlinks/shared
            // directory fallback, even when creation used --no-cow or --colocate.
            cow::probe_between(&self.source, target)?;
            for path in &self.roots {
                let destination = target.join(path);
                fs::create_dir_all(destination.parent().context("ignored root has no parent")?)?;
                payload::confined_parents(target, path)?;
                cow::clone_tree(&self.source.join(path), &destination)?;
            }
            self.validate_source()?;
            if payload::payload_digest(target, &self.roots)? != self.digest {
                bail!("copied ignored payload differs from the frozen source");
            }
            Ok(())
        }
    }
}
