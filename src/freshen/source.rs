//! Select one upstream identity before fetch; never rediscover a line afterward.
use crate::jj::{BookmarkRef, CommitFacts, JjClient, commit_id_revset};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionKind {
    Explicit,
    Tracked,
    ExactRemote,
}

/// Transient operation intent, not a permanent workspace-following association.
/// Persist this with the preparation journal before fetching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSelection {
    pub kind: SelectionKind,
    pub operation: String,
    pub source: CommitFacts,
    pub base: String,
    pub name: String,
    pub remote: String,
    pub old_remote: String,
    pub old_local: Option<String>,
    pub tracked: bool,
    /// An explicit revset remains an explicit goal, including a fixed commit ID.
    pub source_at: Option<String>,
    pub old_goal: String,
}

impl SourceSelection {
    /// The caller must first snapshot real source work and verify disk bytes.
    /// All queries here are read-only against `operation`, including previews.
    pub fn select(
        client: &JjClient,
        operation: &str,
        source_at: Option<&str>,
        remote: Option<&str>,
    ) -> Result<Self> {
        if remote == Some("") {
            bail!("freshen.remote must not be empty");
        }
        let source = client.commit_facts_at(operation, "@")?;
        require_sound(&source, "source")?;
        let base = if source.ordinary_empty_child() {
            source.parents[0].clone()
        } else {
            source.commit_id.clone()
        };
        let refs = client.bookmark_refs_at(operation)?;
        let remotes = refs
            .iter()
            .filter(|r| {
                r.remote.as_deref().is_some_and(|r| r != "git")
                    && remote.is_none_or(|remote| r.remote.as_deref() == Some(remote))
            })
            .collect::<Vec<_>>();

        let (kind, selected, old_goal) = if let Some(expression) = source_at {
            let goal = client
                .resolve_one_at(operation, expression)
                .context("cannot resolve --source-at before fetch")?
                .commit_id;
            // Preserve the name in an explicit remote/local symbol even if
            // several refs share its tip. Arbitrary revsets can identify a
            // unique remote tip, but equal tips alone do not disambiguate names.
            let named_remote = refs
                .iter()
                .filter(|r| r.remote.as_deref().is_some_and(|r| r != "git"))
                .filter(|r| {
                    let remote = r.remote.as_deref().expect("remote ref");
                    remote_symbol_matches(expression, &r.name, remote)
                })
                .collect::<Vec<_>>();
            if !named_remote.is_empty()
                && named_remote
                    .iter()
                    .any(|r| remote.is_some_and(|remote| r.remote.as_deref() != Some(remote)))
            {
                bail!("explicit --source-at selects a different remote than freshen.remote");
            }
            let named = if named_remote.is_empty() {
                remotes
                    .iter()
                    .copied()
                    .filter(|r| r.tracked && local_symbol_matches(expression, &r.name))
                    .collect::<Vec<_>>()
            } else {
                named_remote
            };
            let candidates = if named.is_empty() {
                remotes
                    .iter()
                    .copied()
                    .filter(|r| r.normal_target() == Some(goal.as_str()))
                    .collect()
            } else {
                named
            };
            (
                SelectionKind::Explicit,
                unique(candidates, "explicit source goal")?,
                goal,
            )
        } else {
            let mut tracked = Vec::new();
            for r in remotes.iter().copied().filter(|r| r.tracked) {
                // A local branch descending from B does not make an unrelated
                // remote a source line. The remote must also contain B. Keep
                // related conflicts as blockers rather than hiding them.
                if !has_descendant_target(client, operation, r, &base)? {
                    continue;
                }
                let local = local_ref(&refs, &r.name);
                if local.is_none_or(|r| !r.present)
                    || has_descendant_target(
                        client,
                        operation,
                        local.expect("present local"),
                        &base,
                    )?
                    || r.conflicted
                {
                    tracked.push(r);
                }
            }
            if !tracked.is_empty() {
                let selected = unique(tracked, "tracked source relationships")?;
                let goal = normal(selected, "selected remote")?.to_owned();
                (SelectionKind::Tracked, selected, goal)
            } else {
                // This deliberately includes UNTRACKED remote references.
                let exact = remotes
                    .iter()
                    .copied()
                    .filter(|r| r.added.iter().chain(&r.removed).any(|id| id == &base))
                    .collect();
                let selected = unique(exact, "remote bookmarks at the source base")?;
                let goal = normal(selected, "selected remote")?.to_owned();
                (SelectionKind::ExactRemote, selected, goal)
            }
        };

        let old_remote = normal(selected, "selected remote")?.to_owned();
        require_ancestor(
            client,
            operation,
            &base,
            &old_remote,
            "source contains unpublished work",
        )?;
        require_ancestor(
            client,
            operation,
            &base,
            &old_goal,
            "source goal would leave source work behind",
        )?;
        require_ancestor(
            client,
            operation,
            &old_goal,
            &old_remote,
            "source goal is not published on the selected remote",
        )?;
        // Only a tracked relationship gives the same-named local bookmark any
        // authority here. Untracked remote refs can coexist with unrelated locals.
        let old_local = if selected.tracked {
            let local = local_ref(&refs, &selected.name)
                .context("selected tracked local bookmark is deleted; restore it explicitly")?;
            let target = normal(local, "selected tracked local bookmark")?;
            require_ancestor(
                client,
                operation,
                target,
                &old_remote,
                "tracked local bookmark contains unpublished work",
            )?;
            Some(target.to_owned())
        } else {
            None
        };
        require_sound(
            &client.commit_facts_at(operation, &commit_id_revset(&old_goal)?)?,
            "source goal",
        )?;
        Ok(Self {
            kind,
            operation: operation.to_owned(),
            source,
            base,
            name: selected.name.clone(),
            remote: selected.remote.clone().expect("remote ref"),
            old_remote,
            old_local,
            tracked: selected.tracked,
            source_at: source_at.map(str::to_owned),
            old_goal,
        })
    }

    /// After fetching the pinned remote, validate the SAME bookmark and donor.
    /// Returns the exact goal to persist before `jj new` or configured commands.
    /// The coordinator must also revalidate source disk bytes before mutation.
    pub fn resolve_fetched(&self, client: &JjClient, operation: &str) -> Result<String> {
        self.verify_source_at(client, operation)?;
        let refs = client.bookmark_refs_at(operation)?;
        let selected = refs
            .iter()
            .find(|r| r.name == self.name && r.remote.as_deref() == Some(&self.remote))
            .context("selected remote bookmark disappeared during fetch")?;
        if selected.tracked != self.tracked {
            bail!("selected remote tracking relationship changed during fetch");
        }
        let fetched = normal(selected, "fetched remote bookmark")?;
        // Checking B/L alone would allow a rewind when old R was ahead of both.
        require_ancestor(
            client,
            operation,
            &self.old_remote,
            fetched,
            "selected remote did not fast-forward (rewind or sideways move)",
        )?;
        require_ancestor(
            client,
            operation,
            &self.base,
            fetched,
            "source base is not published on the fetched remote",
        )?;
        if let Some(old_local) = &self.old_local {
            let local = normal(
                local_ref(&refs, &self.name)
                    .context("tracked local bookmark disappeared during fetch")?,
                "fetched tracked local bookmark",
            )?;
            require_ancestor(
                client,
                operation,
                old_local,
                local,
                "tracked local bookmark moved backwards or sideways during fetch",
            )?;
            require_ancestor(
                client,
                operation,
                local,
                fetched,
                "tracked local bookmark contains unpublished work after fetch",
            )?;
        }
        let goal = match &self.source_at {
            Some(expression) => client.resolve_one_at(operation, expression)?.commit_id,
            None => fetched.to_owned(),
        };
        require_ancestor(
            client,
            operation,
            &self.old_goal,
            &goal,
            "explicit source goal moved backwards or sideways during fetch",
        )?;
        require_ancestor(
            client,
            operation,
            &self.base,
            &goal,
            "source goal would leave source work behind",
        )?;
        require_ancestor(
            client,
            operation,
            &goal,
            fetched,
            "source goal is not published on the fetched remote",
        )?;
        require_sound(
            &client.commit_facts_at(operation, &commit_id_revset(&goal)?)?,
            "fetched source goal",
        )?;
        Ok(goal)
    }

    /// Native JJ may normalize a directly published/immutable checkout to a new
    /// empty child while fetching. That leaves the exact old tree/history intact;
    /// any other changed source revision is work, not a disposable normalization.
    /// Disk bytes still need the coordinator's separate filesystem proof.
    pub fn verify_source_at(&self, client: &JjClient, operation: &str) -> Result<CommitFacts> {
        let current = client.commit_facts_at(operation, "@")?;
        let normalized_published = self.base == self.source.commit_id
            && current.ordinary_empty_child()
            && current.parents[0] == self.source.commit_id;
        if current != self.source && !normalized_published {
            bail!("source changed during fetch; source advancement refused");
        }
        Ok(current)
    }
}

fn require_sound(facts: &CommitFacts, role: &str) -> Result<()> {
    if facts.conflicted || facts.divergent {
        bail!("{role} is conflicted or divergent; resolve it explicitly before freshening");
    }
    Ok(())
}

fn local_ref<'a>(refs: &'a [BookmarkRef], name: &str) -> Option<&'a BookmarkRef> {
    refs.iter().find(|r| r.remote.is_none() && r.name == name)
}

fn normal<'a>(reference: &'a BookmarkRef, role: &str) -> Result<&'a str> {
    reference.normal_target().with_context(|| {
        format!(
            "{role} {:?} is conflicted or deleted; resolve it explicitly",
            reference.name
        )
    })
}

fn unique<'a>(candidates: Vec<&'a BookmarkRef>, tier: &str) -> Result<&'a BookmarkRef> {
    match candidates.as_slice() {
        [selected] => Ok(selected),
        [] => bail!("no unambiguous {tier}; supply a one-run --source-at <bookmark>@<remote>"),
        _ => bail!(
            "ambiguous {tier}: {}; supply a one-run --source-at <bookmark>@<remote>",
            candidates
                .iter()
                .map(|r| format!("{:?}@{:?}", r.name, r.remote.as_deref().unwrap_or_default()))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn has_descendant_target(
    client: &JjClient,
    operation: &str,
    reference: &BookmarkRef,
    base: &str,
) -> Result<bool> {
    for id in reference.added.iter().chain(&reference.removed) {
        if client.is_ancestor_at(operation, base, id)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn require_ancestor(
    client: &JjClient,
    operation: &str,
    ancestor: &str,
    descendant: &str,
    reason: &str,
) -> Result<()> {
    if !client.is_ancestor_at(operation, ancestor, descendant)? {
        bail!("{reason}: {ancestor} is not an ancestor of {descendant}");
    }
    Ok(())
}

fn local_symbol_matches(expression: &str, name: &str) -> bool {
    expression == name || serde_json::to_string(name).is_ok_and(|quoted| expression == quoted)
}

fn remote_symbol_matches(expression: &str, name: &str, remote: &str) -> bool {
    [
        name.to_owned(),
        serde_json::to_string(name).expect("string JSON"),
    ]
    .iter()
    .any(|name| {
        [
            remote.to_owned(),
            serde_json::to_string(remote).expect("string JSON"),
        ]
        .iter()
        .any(|remote| expression == format!("{name}@{remote}"))
    })
}
