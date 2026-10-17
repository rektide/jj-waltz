//! Named-reference coverage of frozen target work, without creating/moving refs.
use crate::jj::{BookmarkRef, CommitFacts, JjClient, commit_id_revset};
use anyhow::{Context, Result, bail};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Guard {
    AllowUnbookmarked,
    Required(Proof),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Proof {
    work_tip: String,
    name: String,
    remote: Option<String>,
    target: String,
}

impl Proof {
    fn label(&self) -> String {
        match &self.remote {
            Some(remote) => format!("{}@{remote}", self.name),
            None => self.name.clone(),
        }
    }
}

impl Guard {
    pub(super) fn recovery(&self, root: &std::path::Path, old_commit: &str) -> String {
        let (work, bookmark) = match self {
            Self::Required(proof) => (proof.work_tip.as_str(), proof.label()),
            Self::AllowUnbookmarked => (old_commit, old_commit.to_owned()),
        };
        let expression = match self {
            Self::Required(proof) => {
                let name = serde_json::to_string(&proof.name).expect("JSON string");
                match &proof.remote {
                    Some(remote) => format!(
                        "{name}@{}",
                        serde_json::to_string(remote).expect("JSON string")
                    ),
                    None => name,
                }
            }
            Self::AllowUnbookmarked => commit_id_revset(old_commit).expect("pinned commit ID"),
        };
        let revision = shlex::try_quote(&expression).expect("JSON revset has no NUL");
        match self {
            Self::Required(_) => format!(
                "your work tip {work} is covered by bookmark {bookmark}; recover with `jj new {revision}` and `jj sparse reset` in {}, then re-run reseed; jw will not repair it",
                root.display(),
            ),
            Self::AllowUnbookmarked => format!(
                "bookmark requirement was explicitly bypassed; your prior work is in JJ history at {work}; recover with `jj new {revision}` and `jj sparse reset` in {}, then re-run reseed; jw will not repair it",
                root.display(),
            ),
        }
    }

    pub(super) fn describe(&self) -> String {
        match self {
            Self::AllowUnbookmarked => {
                "bookmark requirement explicitly bypassed (--allow-unbookmarked)".into()
            }
            Self::Required(proof) => format!(
                "bookmark {:?} covers work tip {}",
                proof.label(),
                proof.work_tip
            ),
        }
    }

    /// A frozen required reference must still cover the ORIGINAL work tip. A
    /// forward move is fine; deletion, conflict or a move off that lineage is not.
    /// This is snapshot-free, including when a target is partially refilled.
    pub(super) fn verify(
        &self,
        client: &JjClient,
        operation: &str,
        references: &[BookmarkRef],
    ) -> Result<()> {
        let Self::Required(proof) = self else {
            return Ok(());
        };
        let reference = references
            .iter()
            .find(|reference| reference.name == proof.name && reference.remote == proof.remote);
        let target = reference.and_then(BookmarkRef::normal_target);
        if let Some(target) = target
            && (target == proof.target
                || client.is_ancestor_at(operation, &proof.work_tip, target)?)
        {
            return Ok(());
        }
        bail!(
            "required bookmark {:?} no longer covers frozen work tip {}; restore that reference to this tip or a descendant before retrying; no bookmark was created or moved",
            proof.label(),
            proof.work_tip
        )
    }
}

pub(super) struct Coverage {
    tips: BTreeMap<String, String>,
    proofs: BTreeMap<String, Proof>,
}

fn work_tip(target: &CommitFacts) -> &str {
    if target.ordinary_empty_child() {
        &target.parents[0]
    } else {
        &target.commit_id
    }
}

impl Coverage {
    pub(super) fn capture(
        client: &JjClient,
        operation: &str,
        commits: &BTreeSet<String>,
    ) -> Result<Self> {
        let mut capture = Self {
            tips: BTreeMap::new(),
            proofs: BTreeMap::new(),
        };
        if commits.is_empty() {
            return Ok(capture);
        }
        for chunk in commits.iter().cloned().collect::<Vec<_>>().chunks(128) {
            for facts in client.commit_facts_many_at(operation, &union(chunk)?)? {
                capture
                    .tips
                    .insert(facts.commit_id.clone(), work_tip(&facts).to_owned());
            }
        }
        if capture.tips.len() != commits.len() {
            bail!("bookmark preflight could not resolve every frozen target commit");
        }
        let mut references = client.bookmark_refs_at(operation)?;
        references
            .retain(|reference| !reference.name.is_empty() && reference.normal_target().is_some());
        references.sort_by(|a, b| {
            (a.remote.is_some(), &a.name, &a.remote).cmp(&(b.remote.is_some(), &b.name, &b.remote))
        });
        let mut targets = Vec::new();
        let mut seen = BTreeSet::new();
        for reference in references {
            let target = reference.normal_target().unwrap().to_owned();
            if seen.insert(target.clone()) {
                targets.push(Proof {
                    work_tip: String::new(),
                    name: reference.name,
                    remote: reference.remote,
                    target,
                });
            }
        }
        let mut remaining = capture.tips.values().cloned().collect::<BTreeSet<_>>();
        for proof in &targets {
            if remaining.remove(&proof.target) {
                let mut proof = proof.clone();
                proof.work_tip = proof.target.clone();
                capture.proofs.insert(proof.work_tip.clone(), proof);
            }
        }
        // One bounded ancestry matrix per group of bookmark targets and work
        // tips, not one process per file, target/ref pair, or duplicate work tip.
        // contained_in() takes a literal native revset; IDs are explicitly typed.
        for targets in targets.chunks(128) {
            if remaining.is_empty() {
                break;
            }
            let checks = targets
                .iter()
                .map(|proof| {
                    Ok(format!(
                        "json(self.contained_in({}))",
                        serde_json::to_string(&format!("::{}", commit_id_revset(&proof.target)?))?
                    ))
                })
                .collect::<Result<Vec<_>>>()?
                .join(" ++ \"\\t\" ++ ");
            let template = format!("json(commit_id) ++ \"\\t\" ++ {checks} ++ \"\\n\"");
            for tips in remaining.iter().cloned().collect::<Vec<_>>().chunks(128) {
                let output = client.run_at(
                    operation,
                    ["log", "--no-graph", "-r", &union(tips)?, "-T", &template],
                )?;
                for line in output.stdout()?.lines() {
                    let fields = line.split('\t').collect::<Vec<_>>();
                    if fields.len() != targets.len() + 1 {
                        bail!("malformed bookmark ancestry matrix");
                    }
                    let tip: String = serde_json::from_str(fields[0])?;
                    if !remaining.contains(&tip) {
                        bail!("unexpected work tip in bookmark ancestry matrix");
                    }
                    for (field, proof) in fields[1..].iter().zip(targets) {
                        if serde_json::from_str::<bool>(field)? {
                            let mut proof = proof.clone();
                            proof.work_tip = tip.clone();
                            capture.proofs.insert(tip.clone(), proof);
                            remaining.remove(&tip);
                            break;
                        }
                    }
                }
            }
        }
        Ok(capture)
    }

    pub(super) fn require(&self, commit: &str) -> Result<Guard> {
        let tip = self
            .tips
            .get(commit)
            .context("missing frozen work tip in bookmark preflight")?;
        match self.proofs.get(tip) {
            Some(proof) => Ok(Guard::Required(proof.clone())),
            None => bail!(
                "unbookmarked work tip {tip}; create a named local or remote bookmark at this tip or a descendant, or explicitly pass --allow-unbookmarked; a bookmark only on an older ancestor does not cover this work"
            ),
        }
    }
}

fn union(ids: &[String]) -> Result<String> {
    Ok(format!(
        "({})",
        ids.iter()
            .map(|id| commit_id_revset(id))
            .collect::<Result<Vec<_>>>()?
            .join(" | ")
    ))
}

pub(super) fn verify_batch<'a>(
    client: &JjClient,
    guards: impl IntoIterator<Item = &'a Guard>,
) -> Result<()> {
    let guards = guards
        .into_iter()
        .filter(|guard| matches!(guard, Guard::Required(_)))
        .collect::<Vec<_>>();
    if guards.is_empty() {
        return Ok(());
    }
    let operation = client.operation_id()?;
    let references = client.bookmark_refs_at(&operation)?;
    for guard in guards {
        guard.verify(client, &operation, &references)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deleted_conflicted_and_removed_reference_targets_never_satisfy_the_guard() {
        let tip = "1".repeat(40);
        let guard = Guard::Required(Proof {
            work_tip: tip.clone(),
            name: "saved".into(),
            remote: None,
            target: tip.clone(),
        });
        let client = JjClient::new("/nonexistent-no-jj-process-needed");
        let reference = BookmarkRef {
            name: "saved".into(),
            remote: None,
            tracked: false,
            present: true,
            conflicted: false,
            added: vec![tip],
            removed: Vec::new(),
        };
        guard
            .verify(&client, "unused", std::slice::from_ref(&reference))
            .unwrap();
        for invalid in [
            BookmarkRef {
                present: false,
                ..reference.clone()
            },
            BookmarkRef {
                conflicted: true,
                ..reference.clone()
            },
            BookmarkRef {
                removed: vec!["0".repeat(40)],
                ..reference.clone()
            },
            BookmarkRef {
                added: Vec::new(),
                ..reference.clone()
            },
        ] {
            assert!(
                guard
                    .verify(&client, "unused", &[invalid])
                    .unwrap_err()
                    .to_string()
                    .contains("no longer covers frozen work tip")
            );
        }
        assert!(guard.verify(&client, "unused", &[]).is_err());
    }
}
