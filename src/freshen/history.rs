//! Freeze unpublished-target preservation before any bookmark/checkout mutation.
use crate::jj::{CommitFacts, JjClient, commit_id_revset};
use anyhow::{Context, Result, bail};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// One allocator per batch, under the coordinator's repository operation lock.
/// Reserve names from OTHER active pending/batch intents too: those bookmarks
/// may not exist in JJ yet. Completed receipts need no reservation.
pub struct BookmarkNames {
    stem: String,
    reserved: BTreeSet<String>,
}

impl BookmarkNames {
    /// Read-only; the timestamp is supplied once so previews/tests can pin it.
    /// UTC names have nanosecond precision; occupied names gain -2, -3, ... .
    pub fn at(client: &JjClient, operation: &str, timestamp: Timestamp) -> Result<Self> {
        Ok(Self {
            stem: format!("freshen-{}", timestamp.strftime("%Y%m%dT%H%M%S%.9fZ")),
            // Include remote names and deleted tracked relationships, avoiding
            // native bookmark creation's auto-tracking/recreation side effects.
            reserved: client
                .bookmark_refs_at(operation)?
                .into_iter()
                .map(|r| r.name)
                .collect(),
        })
    }

    pub fn reserve(&mut self, name: &str) {
        self.reserved.insert(name.to_owned());
    }

    fn allocate(&mut self) -> Result<String> {
        let mut name = self.stem.clone();
        let mut suffix = 1_u64;
        while !self.reserved.insert(name.clone()) {
            suffix = suffix
                .checked_add(1)
                .context("freshen name space exhausted")?;
            name = format!("{}-{suffix}", self.stem);
        }
        Ok(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreservationBookmark {
    pub name: String,
    pub tip: String,
}

/// Transient immutable operation intent, not workspace-following metadata.
/// Persist the WHOLE plan in the existing target/batch journal before calling
/// ensure_bookmark. Recovery must deserialize it, never allocate a new name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetHistory {
    pub operation: String,
    pub target: CommitFacts,
    pub work_tip: String,
    pub upstream_goal: String,
    pub published: bool,
    pub bookmark: Option<PreservationBookmark>,
}

impl TargetHistory {
    /// The coordinator first snapshots actual target work and verifies its disk
    /// bytes. Pass the selected FETCHED goal's exact ID, not source @ or a revset
    /// that could move on retry. These queries do not snapshot or mutate anything.
    pub fn plan(
        client: &JjClient,
        operation: &str,
        target_commit: &str,
        fetched_goal: &str,
        no_freshen_bookmark: bool,
        names: &mut BookmarkNames,
    ) -> Result<Self> {
        let target = client.commit_facts_at(operation, &commit_id_revset(target_commit)?)?;
        let upstream_goal = client
            .resolve_one_at(operation, commit_id_revset(fetched_goal)?)?
            .commit_id;
        let work_tip = work_tip(&target).to_owned();
        let published = client.is_ancestor_at(operation, &work_tip, &upstream_goal)?;
        let bookmark = if published || no_freshen_bookmark {
            None
        } else {
            Some(PreservationBookmark {
                name: names.allocate()?,
                tip: work_tip.clone(),
            })
        };
        Ok(Self {
            operation: operation.to_owned(),
            target,
            work_tip,
            upstream_goal,
            published,
            bookmark,
        })
    }

    /// Reconcile ONLY this recorded pair. Missing -> native create; exact -> no
    /// action; any other target/conflict/remote collision -> stop. Never set/move.
    /// Call under the existing operation lock before clearing/advancing, including
    /// on recovery from later phases (where snapshotting would corrupt history).
    ///
    /// JJ create checks its live view too, closing the precheck/create overwrite
    /// window. Concurrent independent JJ operations can still conflict: the
    /// postcheck refuses advancement, not repairs them. Writers must be paused;
    /// this is not a cross-process CAS or a lock on arbitrary JJ commands.
    pub fn ensure_bookmark(&self, client: &JjClient) -> Result<()> {
        self.validate()?;
        let Some(bookmark) = &self.bookmark else {
            return Ok(());
        };
        if self.bookmark_exists(client)? {
            return Ok(());
        }
        client
            .create_bookmark(&bookmark.name, &bookmark.tip)
            .with_context(|| {
                format!(
                    "could not create recorded freshen bookmark {:?} at {}; pending retained; resume reuses this exact pair",
                    bookmark.name, bookmark.tip
                )
            })?;
        if !self.bookmark_exists(client)? {
            bail!("freshen bookmark disappeared after creation; advancement refused");
        }
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        if self.work_tip != work_tip(&self.target)
            || self.bookmark.as_ref().is_some_and(|b| {
                self.published || b.tip != self.work_tip || !b.name.starts_with("freshen-")
            })
        {
            bail!("invalid frozen target history intent");
        }
        Ok(())
    }

    fn bookmark_exists(&self, client: &JjClient) -> Result<bool> {
        let bookmark = self.bookmark.as_ref().context("missing bookmark intent")?;
        let operation = client.operation_id()?;
        let refs = client.bookmark_refs_at(&operation)?;
        let matching = refs
            .iter()
            .filter(|r| r.name == bookmark.name)
            .collect::<Vec<_>>();
        match matching.as_slice() {
            [] => Ok(false),
            [reference]
                if reference.remote.is_none()
                    && reference.normal_target() == Some(bookmark.tip.as_str()) =>
            {
                Ok(true)
            }
            _ => bail!(
                "recorded freshen bookmark {:?} has another target, a conflict, or a remote collision; expected exact tip {}; advancement refused without overwriting it",
                bookmark.name,
                bookmark.tip
            ),
        }
    }

    pub fn describe(&self, planned: bool) -> String {
        if let Some(bookmark) = &self.bookmark {
            format!(
                "{} freshen bookmark {:?} at {} (unpublished target work)",
                if planned {
                    "would preserve with"
                } else {
                    "preserved with"
                },
                bookmark.name,
                bookmark.tip
            )
        } else if self.published {
            format!(
                "target work {} is upstream; no freshen bookmark",
                self.work_tip
            )
        } else {
            format!(
                "freshen bookmark disabled for {}; JJ history still preserved",
                self.work_tip
            )
        }
    }
}

pub(crate) fn work_tip(target: &CommitFacts) -> &str {
    if target.ordinary_empty_child() {
        &target.parents[0]
    } else {
        &target.commit_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, process::Command};

    struct Repo {
        _temp: tempfile::TempDir,
        client: JjClient,
        base: String,
    }

    impl Repo {
        fn new() -> Self {
            let temp = tempfile::tempdir().unwrap();
            let output = Command::new("jj")
                .args(["git", "init", "--no-colocate"])
                .arg(temp.path())
                .output()
                .unwrap();
            assert!(output.status.success(), "{:?}", output);
            let client = JjClient::new(temp.path());
            client
                .run(["config", "set", "--repo", "fsmonitor.backend", "none"])
                .unwrap();
            fs::write(temp.path().join("file"), "base\n").unwrap();
            client.run(["describe", "-m", "base"]).unwrap();
            let base = client.resolve_one("@").unwrap().commit_id;
            client.run(["new"]).unwrap();
            Self {
                _temp: temp,
                client,
                base,
            }
        }

        fn names(&self) -> BookmarkNames {
            BookmarkNames::at(
                &self.client,
                &self.client.operation_id().unwrap(),
                "2026-10-15T12:34:56.123456789Z".parse().unwrap(),
            )
            .unwrap()
        }

        fn plan(&self, goal: &str, disabled: bool, names: &mut BookmarkNames) -> TargetHistory {
            self.client
                .run(["--config=fsmonitor.backend=none", "status"])
                .unwrap();
            let operation = self.client.operation_id().unwrap();
            let target = self
                .client
                .resolve_one_at(&operation, "@")
                .unwrap()
                .commit_id;
            let plan =
                TargetHistory::plan(&self.client, &operation, &target, goal, disabled, names)
                    .unwrap();
            assert_eq!(
                self.client.operation_id().unwrap(),
                operation,
                "planning is read-only"
            );
            plan
        }

        fn tip(&self, bookmark: &PreservationBookmark) -> String {
            self.client
                .bookmark_refs_at(&self.client.operation_id().unwrap())
                .unwrap()
                .into_iter()
                .find(|r| r.remote.is_none() && r.name == bookmark.name)
                .unwrap()
                .normal_target()
                .unwrap()
                .to_owned()
        }
    }

    #[test]
    fn publication_uses_ancestry_and_only_discards_ordinary_scratch() {
        let f = Repo::new();
        let mut names = f.names();
        let scratch = f.plan(&f.base, false, &mut names);
        assert!(scratch.published && scratch.bookmark.is_none());
        assert_eq!(scratch.work_tip, f.base);

        f.client
            .run(["describe", "-m", "meaningful empty"])
            .unwrap();
        let meaningful = f.plan(&f.base, false, &mut names);
        assert!(meaningful.target.empty && !meaningful.published);
        assert_eq!(meaningful.work_tip, meaningful.target.commit_id);
        assert!(
            meaningful.bookmark.is_some(),
            "equal trees are not publication"
        );
        f.client.run(["new"]).unwrap();
        let above_local = f.plan(&f.base, false, &mut names);
        assert_eq!(above_local.work_tip, meaningful.work_tip);
        assert!(!above_local.published);

        f.client
            .run(["new", "-r", &f.base, "-m", "other empty"])
            .unwrap();
        let other = f.client.resolve_one("@").unwrap().commit_id;
        f.client.run(["new", &meaningful.work_tip, &other]).unwrap();
        let merge = f.plan(&f.base, false, &mut names);
        assert!(merge.target.empty && merge.target.parents.len() == 2);
        assert_eq!(merge.work_tip, merge.target.commit_id);
        assert!(merge.bookmark.is_some());

        // The selected goal can contain work that an older recorded goal did not.
        let published_merge = merge.target.commit_id;
        f.client.run(["new"]).unwrap();
        let contained = f.plan(&published_merge, false, &mut names);
        assert!(contained.published && contained.bookmark.is_none());
        assert_eq!(contained.work_tip, published_merge);
    }

    #[test]
    fn dirty_work_is_exact_and_replay_neither_duplicates_nor_snapshots() {
        let f = Repo::new();
        fs::write(f.client.cwd().join("file"), "unpublished work\n").unwrap();
        let plan = f.plan(&f.base, false, &mut f.names());
        let bookmark = plan.bookmark.as_ref().unwrap();
        assert_eq!(plan.work_tip, plan.target.commit_id);
        f.client.create_bookmark("feature", &plan.work_tip).unwrap();
        // An ID-shaped bookmark must not redirect the exact preservation target.
        f.client.create_bookmark(&plan.work_tip, &f.base).unwrap();
        let serialized = serde_json::to_vec(&plan).unwrap();
        let frozen: TargetHistory = serde_json::from_slice(&serialized).unwrap();
        // Simulate half-filled target bytes: replay must not snapshot them.
        fs::write(f.client.cwd().join("file"), "partial materialization\n").unwrap();
        frozen.ensure_bookmark(&f.client).unwrap();
        assert_eq!(f.tip(bookmark), plan.work_tip);
        let after_creation = f.client.operation_id().unwrap();
        frozen.ensure_bookmark(&f.client).unwrap();
        assert_eq!(f.client.operation_id().unwrap(), after_creation);
        assert_eq!(
            f.client.resolve_one("@").unwrap().commit_id,
            plan.target.commit_id
        );
        assert_eq!(
            f.client.resolve_one("feature").unwrap().commit_id,
            plan.work_tip
        );
        assert_eq!(serde_json::to_vec(&frozen).unwrap(), serialized);
        assert_eq!(
            fs::read_to_string(f.client.cwd().join("file")).unwrap(),
            "partial materialization\n"
        );
    }

    #[test]
    fn batch_repeats_and_uncreated_intents_reserve_collision_safe_names() {
        let f = Repo::new();
        f.client
            .run(["describe", "-m", "meaningful empty"])
            .unwrap();
        let first = f.plan(&f.base, false, &mut f.names());
        let first_name = &first.bookmark.as_ref().unwrap().name;
        assert_eq!(first_name, "freshen-20261015T123456.123456789Z");
        f.client.create_bookmark(first_name, &f.base).unwrap();
        let mut names = f.names();
        let second = f.plan(&f.base, false, &mut names);
        let third = f.plan(&f.base, false, &mut names);
        assert_eq!(
            second.bookmark.as_ref().unwrap().name,
            format!("{first_name}-2")
        );
        assert_eq!(
            third.bookmark.as_ref().unwrap().name,
            format!("{first_name}-3")
        );
        let mut next_batch = f.names();
        next_batch.reserve(&second.bookmark.as_ref().unwrap().name);
        next_batch.reserve(&third.bookmark.as_ref().unwrap().name);
        let fourth = f.plan(&f.base, false, &mut next_batch);
        assert_eq!(
            fourth.bookmark.as_ref().unwrap().name,
            format!("{first_name}-4")
        );
        assert_eq!(f.tip(first.bookmark.as_ref().unwrap()), f.base);
    }

    #[test]
    fn collision_after_freeze_stops_without_reallocating_or_repointing() {
        let f = Repo::new();
        f.client
            .run(["describe", "-m", "meaningful empty"])
            .unwrap();
        let plan = f.plan(&f.base, false, &mut f.names());
        let bookmark = plan.bookmark.as_ref().unwrap();
        f.client.create_bookmark(&bookmark.name, &f.base).unwrap();
        let operation = f.client.operation_id().unwrap();
        let error = plan.ensure_bookmark(&f.client).unwrap_err().to_string();
        assert!(error.contains("another target"), "{error}");
        assert_eq!(f.client.operation_id().unwrap(), operation);
        assert_eq!(f.tip(bookmark), f.base);
        // Native create is itself create-only, not a precheck followed by `set`.
        assert!(
            f.client
                .create_bookmark(&bookmark.name, &plan.work_tip)
                .is_err()
        );
        assert_eq!(f.tip(bookmark), f.base);
    }

    #[cfg(unix)]
    #[test]
    fn native_create_refuses_a_collision_inserted_after_the_precheck() {
        use std::os::unix::fs::PermissionsExt;

        if let Some(path) = std::env::var_os("JW_HISTORY_RACE_PLAN") {
            let plan: TargetHistory = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            let client = JjClient::new(std::env::var_os("JW_HISTORY_RACE_ROOT").unwrap());
            let error = plan.ensure_bookmark(&client).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("could not create recorded freshen bookmark"),
                "{error:#}"
            );
            return;
        }
        let f = Repo::new();
        f.client
            .run(["describe", "-m", "meaningful empty"])
            .unwrap();
        let plan = f.plan(&f.base, false, &mut f.names());
        let bookmark = plan.bookmark.as_ref().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let plan_path = scratch.path().join("plan.json");
        fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
        let real_jj = std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|p| p.join("jj"))
            .find(|p| p.is_file())
            .unwrap();
        let wrapper = scratch.path().join("jj");
        // Insert a foreign bookmark after the read-only precheck, immediately
        // before the real create command. The actual native create must refuse.
        // Restore PATH before delegation: a version-manager shim may otherwise
        // rediscover this wrapper instead of its actual JJ executable.
        let original_path = std::env::var("PATH").unwrap();
        fs::write(&wrapper, format!(
            "#!/bin/sh\nset -eu\nPATH={}; export PATH\nfor arg do\nif [ \"$arg\" = create ]; then\n{:?} --ignore-working-copy bookmark create -r {} -- {}\nfi\ndone\nexec {:?} \"$@\"\n",
            shlex::try_quote(&original_path).unwrap(), real_jj, f.base, bookmark.name, real_jj,
        )).unwrap();
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "freshen::history::tests::native_create_refuses_a_collision_inserted_after_the_precheck", "--nocapture"])
            .env("JW_HISTORY_RACE_PLAN", &plan_path)
            .env("JW_HISTORY_RACE_ROOT", f.client.cwd())
            .env("PATH", std::env::join_paths(std::iter::once(scratch.path().to_path_buf()).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap()))).unwrap())
            .output().unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(f.tip(bookmark), f.base);
        assert_eq!(
            f.client.resolve_one("@").unwrap().commit_id,
            plan.target.commit_id
        );
        assert_eq!(
            f.client
                .local_bookmark_names_at(f.client.operation_id().unwrap())
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn opt_out_records_unpublished_history_without_bookmark_mutation() {
        let f = Repo::new();
        f.client
            .run(["describe", "-m", "intentional empty"])
            .unwrap();
        let plan = f.plan(&f.base, true, &mut f.names());
        assert!(!plan.published && plan.bookmark.is_none());
        let operation = f.client.operation_id().unwrap();
        plan.ensure_bookmark(&f.client).unwrap();
        assert_eq!(f.client.operation_id().unwrap(), operation);
        assert!(plan.describe(false).contains("disabled"));
    }
}
