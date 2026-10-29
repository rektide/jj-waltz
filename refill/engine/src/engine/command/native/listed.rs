//! One prepared tracked-source phase, shared across serial native receivers.
//! This adapter supplies native membership authority, never payload equality.
use super::*;
use diff_dirs::{CheckError, Demand, EntryFacts, Kind, Side, SideDemands};
use diff_dirs_inspect::DeclaredCandidate;
use diff_dirs_sync::{
    CopyPolicy, DesiredLeaf, EffectBinding, LeafExpectation, LeafKind, ListedCandidate, ListedGuard, ListedIntent, ListedTarget, OwnedLeafSet,
    PreparedLeafList, reconcile_listed,
};
use std::os::unix::fs::MetadataExt;

pub(in crate::engine::command) struct TrackedPhase {
    desired: PreparedLeafList,
    owned: BTreeMap<String, OwnedLeafSet>,
    owned_keys: BTreeMap<String, String>,
    empty: OwnedLeafSet,
    bindings: BTreeMap<String, EffectBinding>,
    guards: BTreeMap<String, NativeGuard>,
    intent: ListedIntent,
    pub source: diff_dirs::SourceSession,
}
impl TrackedPhase {
    pub fn prepare(membership: &Membership, source: &PreparedSource, saved: &[SavedReceiver], route: RenewalRoute, replace_content: bool) -> Result<Self> {
        let desired = PreparedLeafList::from_specs(membership.get(&source.report.commit).leaves.iter().map(|e| DesiredLeaf {
            relative: e.path.clone(),
            expected: LeafExpectation::Exact {
                kind: if e.kind == "file" { LeafKind::RegularFile } else { LeafKind::Symlink },
                executable: (e.kind == "file").then_some(e.executable),
            },
        }))?;
        let intent = if route == RenewalRoute::ReplaceAuthorized && !replace_content {
            ListedIntent::RenewOwned
        } else {
            ListedIntent::CreateAbsent
        };
        let mut owned = BTreeMap::new();
        let mut owned_keys = BTreeMap::new();
        let mut bindings = BTreeMap::new();
        let mut guards = BTreeMap::new();
        for saved in saved {
            if intent == ListedIntent::RenewOwned {
                let tree = membership.get(&saved.old_commit);
                if !owned.contains_key(&tree.key) {
                    owned.insert(tree.key.clone(), OwnedLeafSet::new(tree.leaves.iter().map(|e| &e.path))?);
                }
                owned_keys.insert(saved.old_commit.clone(), tree.key.clone());
            }
            bindings.insert(
                saved.binding.name.clone(),
                EffectBinding::new(&source.binding.root, &saved.binding.root, CopyPolicy::ReflinkRequired)?,
            );
            guards.insert(
                saved.binding.name.clone(),
                NativeGuard {
                    source: source.binding.root.clone(),
                    target: saved.binding.root.clone(),
                    source_identity: source.binding.root_identity(),
                    target_identity: saved.binding.root_identity(),
                },
            );
        }
        let session = diff_dirs::SourceSession::new(bindings.values().next().context("tracked phase requires receivers")?.source());
        Ok(Self {
            desired,
            owned,
            owned_keys,
            empty: OwnedLeafSet::new(std::iter::empty::<&Path>())?,
            bindings,
            guards,
            intent,
            source: session,
        })
    }
    pub fn owned_count(&self) -> usize {
        self.owned.len()
    }
    pub fn install(&self, saved: &SavedReceiver, report: &mut ReceiverReport) -> Result<()> {
        let id = diff_dirs::coordinated::TargetId(saved.binding.name.clone());
        let owned = if self.intent == ListedIntent::RenewOwned {
            &self.owned[&self.owned_keys[&saved.old_commit]]
        } else {
            &self.empty
        };
        let mut result = reconcile_listed(
            &self.source,
            &self.desired,
            &[ListedTarget {
                id: &id,
                binding: &self.bindings[&saved.binding.name],
                owned,
                intent: self.intent,
                guard: &self.guards[&saved.binding.name],
            }],
        )?;
        let lane = result.outcomes.pop().context("provider returned no native lane")?;
        report.native.manifest = Some(lane.coverage);
        match lane.outcome {
            Ok(done) => report.native.completed = done.completed,
            Err(failure) => {
                report.native.completed = failure.completed.clone();
                return Err(failure.into());
            }
        }
        ensure!(
            lane.coverage.complete,
            "provider native lane succeeded without complete manifest coverage; S adoption refused"
        );
        Ok(())
    }
}

struct NativeGuard {
    source: PathBuf,
    target: PathBuf,
    source_identity: (u64, u64),
    target_identity: (u64, u64),
}
impl ListedGuard for NativeGuard {
    fn demand(&self, candidate: ListedCandidate<'_>) -> SideDemands {
        let attributes = Demand {
            attributes: true,
            literal_link: false,
        };
        match candidate.observed {
            DeclaredCandidate::Route(route) => {
                if route.side == Side::Original {
                    SideDemands {
                        source: attributes,
                        target: Demand::NONE,
                    }
                } else {
                    SideDemands {
                        source: Demand::NONE,
                        target: attributes,
                    }
                }
            }
            DeclaredCandidate::Entry(entry) => SideDemands {
                source: if entry.shape.source.kind().is_some() { attributes } else { Demand::NONE },
                target: if entry.shape.target.is_some() { attributes } else { Demand::NONE },
            },
        }
    }
    fn check(&self, candidate: ListedCandidate<'_>, facts: EntryFacts<'_>) -> Result<(), CheckError> {
        let (info, root_side) = match candidate.observed {
            DeclaredCandidate::Route(route) => {
                safe(route.relative)?;
                if route.kind != Kind::Directory {
                    return Err("native route is not a real directory".into());
                }
                (route.info, route.relative.as_os_str().is_empty().then_some(route.side))
            }
            DeclaredCandidate::Entry(entry) => {
                safe(&entry.coordinate.to_path_buf())?;
                (entry.info, None)
            }
        };
        if info.source_root != self.source || info.target_root != self.target {
            return Err("native guard roots differ from bound phase".into());
        }
        for (side, fact, identity) in [
            (Side::Original, facts.source, self.source_identity),
            (Side::Target, facts.target, self.target_identity),
        ] {
            let Some(fact) = fact else { continue };
            if fact.kind == Kind::Other {
                return Err("unsupported native physical kind".into());
            }
            let metadata = fact.attributes.known().ok_or("native guard requires actual side attributes")?;
            if metadata.dev() != identity.0 {
                return Err("native manifest crossed a device boundary".into());
            }
            if root_side == Some(side) && (!metadata.is_dir() || metadata.ino() != identity.1) {
                return Err("native bound root identity changed".into());
            }
        }
        Ok(())
    }
}
fn safe(path: &Path) -> Result<(), CheckError> {
    if path
        .components()
        .any(|c| !matches!(c, std::path::Component::Normal(n) if n != ".jj" && n != ".git"))
    {
        return Err("native manifest reached administration or abnormal coordinate".into());
    }
    Ok(())
}
