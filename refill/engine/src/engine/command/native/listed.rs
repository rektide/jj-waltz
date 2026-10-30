//! One prepared tracked-source phase, shared across serial native receivers.
//! This adapter supplies native membership authority, never payload equality.
use super::*;
use diff_dirs::{CheckError, Demand, EntryFacts, Kind, Side, SideDemands};
use diff_dirs_inspect::DeclaredCandidate;
use diff_dirs_sync::{
    CopyPolicy, DesiredLeaf, EffectBinding, LeafExpectation, LeafKind, ListedCandidate, ListedGuard, ListedIntent, ListedTarget, OwnedLeafSet,
    PreparedLeafList, RetainedAttributes, reconcile_listed,
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
    pub fn prepare(membership: &Membership, source: &PreparedSource, saved: &[SavedReceiver], request: &ReseedRequest) -> Result<Self> {
        let desired = PreparedLeafList::from_specs(membership.get(&source.report.commit).leaves.iter().map(|e| DesiredLeaf {
            relative: e.path.clone(),
            expected: LeafExpectation::Exact {
                kind: if e.kind == "file" { LeafKind::RegularFile } else { LeafKind::Symlink },
                executable: (e.kind == "file").then_some(e.executable),
            },
        }))?;
        let intent = if request.renewal_route != RenewalRoute::ClearThenPopulate && !request.replace_content {
            match request.renewal_route {
                RenewalRoute::Refresh => ListedIntent::RefreshOwned(request.refresh_check),
                RenewalRoute::ReplaceAuthorized => ListedIntent::RenewOwned,
                RenewalRoute::ClearThenPopulate => unreachable!(),
            }
        } else {
            ListedIntent::CreateAbsent
        };
        let mut owned = BTreeMap::new();
        let mut owned_keys = BTreeMap::new();
        let mut bindings = BTreeMap::new();
        let mut guards = BTreeMap::new();
        for saved in saved {
            if intent != ListedIntent::CreateAbsent {
                let tree = membership.get(&saved.old_commit);
                if !owned.contains_key(&tree.key) {
                    owned.insert(tree.key.clone(), OwnedLeafSet::new(tree.leaves.iter().map(|e| &e.path))?);
                }
                owned_keys.insert(saved.old_commit.clone(), tree.key.clone());
            }
            bindings.insert(
                saved.binding.name.clone(),
                EffectBinding::new(&source.binding.root, &saved.binding.root, CopyPolicy::ReflinkRequired)?.with_postcheck(request.postcheck.into()),
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
        let owned = if self.intent != ListedIntent::CreateAbsent {
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
                retained_attributes: RetainedAttributes::ManifestExecutable,
            }],
        )?;
        let lane = result.outcomes.pop().context("provider returned no native lane")?;
        accept_lane(lane, &mut report.native)
    }
}

/// Shared success/failure projection and the adoption gate. Coverage is copied
/// before inspecting the lane result; neither outer Ok nor installation counts
/// can hide cleanup failure. No post-failure filesystem census is performed.
fn accept_lane(lane: diff_dirs_sync::ListedOutcome, native: &mut NativeOutcome) -> Result<()> {
    native.manifest = Some(lane.coverage);
    native.retained_permissions_set = Some(lane.retained_permissions_set);
    match lane.outcome {
        Ok(done) => {
            native.completed = Some(done.completed);
            native.policy = Some(done.policy);
        }
        Err(failure) => {
            native.completed = Some(failure.completed.clone());
            native.policy = Some(*failure.policy.clone());
            return Err(failure.into());
        }
    }
    ensure!(
        lane.coverage.complete,
        "provider native lane succeeded without complete manifest coverage; S adoption refused"
    );
    Ok(())
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
            let supported = match fact.kind {
                Kind::Directory => metadata.is_dir(),
                Kind::File => metadata.is_file(),
                Kind::Symlink => metadata.is_symlink(),
                Kind::Other => false,
            };
            if !supported {
                return Err("native physical attributes disagree with supported kind".into());
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use diff_dirs::{BatchInfo, Coordinate, Shape, SourceObservation};
    use diff_dirs_inspect::{DeclaredEntryCandidate, DeclaredRouteCandidate};
    use diff_dirs_sync::{ListedMode, ListedRelation, ManifestCoverage, Progress};

    fn candidate(observed: DeclaredCandidate<'_>) -> ListedCandidate<'_> {
        ListedCandidate {
            observed,
            intent: ListedIntent::RenewOwned,
            mode: ListedMode::Declared,
            relation: ListedRelation::default(),
        }
    }
    #[test]
    fn native_guard_requests_only_actual_route_side_and_never_o_only_source_facts() {
        let guard = NativeGuard {
            source: "/s".into(),
            target: "/t".into(),
            source_identity: (1, 2),
            target_identity: (1, 3),
        };
        let info = BatchInfo {
            source_root: Path::new("/s"),
            target_root: Path::new("/t"),
            relative: Path::new(""),
            structural_root: false,
            purpose: diff_dirs::Purpose::Copy,
        };
        for side in [Side::Original, Side::Target] {
            let demand = guard.demand(candidate(DeclaredCandidate::Route(DeclaredRouteCandidate {
                info,
                side,
                relative: Path::new(""),
                kind: Kind::Directory,
            })));
            assert_eq!(demand.source.attributes, side == Side::Original);
            assert_eq!(demand.target.attributes, side == Side::Target);
            assert!(!demand.source.literal_link && !demand.target.literal_link);
        }
        for source in [
            SourceObservation::NotRequested,
            SourceObservation::Absent,
            SourceObservation::Present(Kind::File),
        ] {
            let demand = guard.demand(candidate(DeclaredCandidate::Entry(DeclaredEntryCandidate {
                info,
                domain: diff_dirs::SelectionDomain::PhysicalUnion,
                coordinate: Coordinate {
                    parent: Path::new(""),
                    name: std::ffi::OsStr::new("old"),
                    structural_root: false,
                },
                desired: None,
                comparison: diff_dirs::Comparison::NotAssessed,
                shape: Shape {
                    source,
                    target: Some(Kind::File),
                },
            })));
            assert_eq!(demand.source.attributes, source.kind().is_some());
            assert!(demand.target.attributes);
        }
        assert!(safe(Path::new("route/.jj/leaf")).is_err());
        assert!(safe(Path::new(".git")).is_err());
        assert!(safe(Path::new("/absolute")).is_err());
        assert!(safe(Path::new("")).is_ok(), "root route is valid");
    }

    #[test]
    fn incomplete_success_and_completed_installations_with_cleanup_failure_never_pass_adoption_gate() {
        let coverage = ManifestCoverage {
            desired: 3,
            installed: 3,
            retained: 0,
            owned_obsolete: 1,
            owned_retired: 1,
            complete: false,
        };
        let progress = Progress {
            files_reflinked: 3,
            files_removed: 1,
            directory_permissions_set: 2,
            ..Default::default()
        };
        let mut native = NativeOutcome::default();
        let error = accept_lane(
            diff_dirs_sync::ListedOutcome {
                id: "target".into(),
                coverage,
                retained_permissions_set: 0,
                outcome: Ok(diff_dirs_sync::Report {
                    planned: 4,
                    completed: progress.clone(),
                    policy: Default::default(),
                    observed: None,
                }),
            },
            &mut native,
        )
        .unwrap_err();
        assert!(error.to_string().contains("without complete manifest coverage"));
        assert_eq!(native.manifest, Some(coverage));
        assert_eq!(native.completed, Some(progress.clone()));
        let error = accept_lane(
            diff_dirs_sync::ListedOutcome {
                id: "target".into(),
                coverage,
                retained_permissions_set: 0,
                outcome: Err(diff_dirs_sync::Failure {
                    planned: 4,
                    policy: Box::default(),
                    primary: Box::new(diff_dirs_sync::Issue {
                        operation: "restore directory permissions",
                        path: "/target".into(),
                        cause: Box::new(std::io::Error::new(ErrorKind::PermissionDenied, "cleanup failed after all installs")),
                    }),
                    secondary: Box::default(),
                    completed: progress.clone(),
                    observed: None,
                }),
            },
            &mut native,
        )
        .unwrap_err();
        assert_eq!(error.downcast_ref::<diff_dirs_sync::Failure>().unwrap().completed, progress);
        assert!(
            error
                .chain()
                .any(|e| e.downcast_ref::<std::io::Error>().is_some_and(|e| e.kind() == ErrorKind::PermissionDenied))
        );
        assert_eq!(native.manifest, Some(coverage));
        assert_eq!(native.completed, Some(progress));
        assert!(native.attempted.is_empty());
        assert_eq!(native.state, Establishment::Unstarted);
    }

    #[test]
    fn actual_retained_metadata_failure_prefix_cannot_reach_native_adoption() {
        use std::os::unix::fs::PermissionsExt;
        struct Fault(PathBuf);
        impl ListedGuard for Fault {
            fn check(&self, at: ListedCandidate<'_>, _: EntryFacts<'_>) -> Result<(), CheckError> {
                // Deliberately impure TEST fault after the provider acquired real
                // attributes. Production NativeGuard remains strictly pure.
                if let DeclaredCandidate::Entry(e) = at.observed
                    && e.coordinate.to_path_buf() == Path::new("z/file")
                {
                    fs::remove_file(self.0.join("z/file")).unwrap();
                }
                Ok(())
            }
        }
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/native-attributes");
        fs::create_dir_all(&scratch).unwrap();
        let temp = tempfile::tempdir_in(scratch.canonicalize().unwrap()).unwrap();
        let source = temp.path().join("source");
        let target = temp.path().join("target");
        for root in [&source, &target] {
            fs::create_dir_all(root.join("z")).unwrap();
        }
        for name in ["a", "z/file"] {
            fs::write(source.join(name), "same").unwrap();
            fs::write(target.join(name), "same").unwrap();
            fs::File::options()
                .write(true)
                .open(target.join(name))
                .unwrap()
                .set_modified(fs::metadata(source.join(name)).unwrap().modified().unwrap())
                .unwrap();
            fs::set_permissions(source.join(name), fs::Permissions::from_mode(0o755)).unwrap();
            fs::set_permissions(target.join(name), fs::Permissions::from_mode(0o640)).unwrap();
        }
        let desired = PreparedLeafList::from_specs(["a", "z/file"].map(|p| DesiredLeaf {
            relative: p.into(),
            expected: LeafExpectation::Exact {
                kind: LeafKind::RegularFile,
                executable: Some(true),
            },
        }))
        .unwrap();
        let owned = OwnedLeafSet::new(["a", "z/file"]).unwrap();
        let binding = EffectBinding::new(&source, &target, CopyPolicy::ReflinkRequired).unwrap();
        let id = "failure".into();
        let guard = Fault(target);
        let lane = reconcile_listed(
            &diff_dirs::SourceSession::new(&source),
            &desired,
            &[ListedTarget {
                id: &id,
                binding: &binding,
                owned: &owned,
                intent: ListedIntent::RefreshOwned(Default::default()),
                guard: &guard,
                retained_attributes: RetainedAttributes::ManifestExecutable,
            }],
        )
        .unwrap()
        .outcomes
        .remove(0);
        let mut native = NativeOutcome {
            state: Establishment::InProgress,
            source_state_adopted: Some(false),
            ..Default::default()
        };
        let error = accept_lane(lane, &mut native).unwrap_err();
        let failure = error.downcast_ref::<diff_dirs_sync::Failure>().unwrap();
        assert_eq!(failure.primary.operation, "conform retained executable");
        assert_eq!(failure.completed.files_reflinked, 0);
        assert_eq!(native.retained_permissions_set, Some(1));
        let coverage = native.manifest.unwrap();
        assert_eq!((coverage.installed, coverage.retained, coverage.complete), (0, 1, false));
        assert_eq!(native.source_state_adopted, Some(false));
        assert_eq!(native.state, Establishment::InProgress);
        assert!(native.attempted.is_empty());
    }
}
