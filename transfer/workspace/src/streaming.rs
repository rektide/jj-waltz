//! Source-qualified ordinary authority, evaluated during production inspection.
//! No realized-root set, payload census, native commands or mutable admission map.
use crate::frozen::{FrozenIgnore, IgnoreState};
use crate::ignored::{Patterns, Prefix};
use anyhow::{Result, ensure};
use diff_dirs::{
    Action, CheckError, Choices, Container, ContextSite, Demand, FileKind, Interior, InteriorInput, InteriorRole, Kind, Names, Policy, SemanticId,
    ShapeChoices, SideDemands, SourceObservation, StatKey,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

#[derive(Debug, Default)]
struct Node {
    children: BTreeMap<OsString, usize>,
    leaf: bool,
}
#[derive(Debug)]
struct PathIndex(Vec<Node>);
impl PathIndex {
    fn new(paths: &BTreeSet<PathBuf>) -> Self {
        let mut nodes = vec![Node::default()];
        for path in paths {
            let mut at = 0;
            for part in path.components() {
                let name = part.as_os_str();
                at = match nodes[at].children.get(name) {
                    Some(i) => *i,
                    None => {
                        let i = nodes.len();
                        nodes.push(Node::default());
                        nodes[at].children.insert(name.to_owned(), i);
                        i
                    }
                };
            }
            nodes[at].leaf = true;
        }
        Self(nodes)
    }
    fn descend(&self, at: Option<usize>, name: &std::ffi::OsStr) -> Option<usize> {
        at.and_then(|i| self.0[i].children.get(name).copied())
    }
}

/// Shared immutable selector/native index. Lane excludes and active links do
/// not enter this shared owner.
#[derive(Debug)]
pub struct OrdinarySource {
    patterns: Arc<Patterns>,
    native: PathIndex,
    identity: [u8; 32],
}
impl OrdinarySource {
    pub fn new(patterns: Arc<Patterns>, native: &BTreeSet<PathBuf>) -> Self {
        let mut hash = Sha256::new();
        for p in patterns.declarations() {
            frame(&mut hash, p.as_bytes());
        }
        frame(&mut hash, b"native");
        for p in native {
            frame(&mut hash, p.as_os_str().as_encoded_bytes());
        }
        Self {
            patterns,
            native: PathIndex::new(native),
            identity: hash.finalize().into(),
        }
    }
}
fn frame(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

#[derive(Debug)]
pub struct SourceQualified {
    source: Arc<OrdinarySource>,
    source_ignore: Arc<FrozenIgnore>,
    desired_ignore: Arc<FrozenIgnore>,
    links: PathIndex,
    devices: (u64, u64),
    identity: SemanticId,
    #[cfg(test)]
    trace: Arc<std::sync::Mutex<Trace>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchState {
    at: PathBuf,
    selector: Prefix,
    source_ignore: IgnoreState,
    desired_ignore: IgnoreState,
    native: Option<usize>,
    links: Option<usize>,
    seal: [u8; 32],
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrdinaryRegion {
    Initial,
    Search(SearchState),
    SelectedMirror { root: PathBuf, admission: [u8; 32] },
}
static SCHEMA: LazyLock<SemanticId> = LazyLock::new(|| SemanticId::new("jw.ordinary-region", 1, vec![]));

impl SourceQualified {
    pub fn new(
        source: Arc<OrdinarySource>,
        source_ignore: Arc<FrozenIgnore>,
        desired_ignore: Arc<FrozenIgnore>,
        links: &BTreeSet<PathBuf>,
        devices: (u64, u64),
    ) -> Self {
        let mut hash = Sha256::new();
        hash.update(source.identity);
        hash.update(source_ignore.identity());
        hash.update(desired_ignore.identity());
        hash.update(devices.0.to_le_bytes());
        hash.update(devices.1.to_le_bytes());
        for p in links {
            frame(&mut hash, p.as_os_str().as_encoded_bytes());
        }
        Self {
            source,
            source_ignore,
            desired_ignore,
            links: PathIndex::new(links),
            devices,
            identity: SemanticId::new("jw.source-qualified", 3, hash.finalize().to_vec()),
            #[cfg(test)]
            trace: Default::default(),
        }
    }
    fn root(&self) -> SearchState {
        self.seal(SearchState {
            at: PathBuf::new(),
            selector: self.source.patterns.root(),
            source_ignore: self.source_ignore.root(),
            desired_ignore: self.desired_ignore.root(),
            native: Some(0),
            links: Some(0),
            seal: [0; 32],
        })
    }
    fn advance(&self, state: &SearchState, name: &std::ffi::OsStr) -> Result<SearchState> {
        let at = state.at.join(name);
        Ok(self.seal(SearchState {
            selector: self.source.patterns.descend(&state.selector, name)?,
            source_ignore: self.source_ignore.descend(&state.source_ignore, &at),
            desired_ignore: self.desired_ignore.descend(&state.desired_ignore, &at),
            native: self.source.native.descend(state.native, name),
            links: self.links.descend(state.links, name),
            at,
            seal: [0; 32],
        }))
    }
    fn seal(&self, mut state: SearchState) -> SearchState {
        state.seal = [0; 32];
        let mut hash = Sha256::new();
        hash.update(&self.identity.parameters);
        hash.update(serde_json::to_vec(&state).expect("UTF-8 search coordinates"));
        state.seal = hash.finalize().into();
        state
    }
    fn admission(&self, root: &Path) -> [u8; 32] {
        let mut hash = Sha256::new();
        hash.update(&self.identity.parameters);
        frame(&mut hash, root.as_os_str().as_encoded_bytes());
        hash.finalize().into()
    }
}
impl Policy for SourceQualified {
    type Context = OrdinaryRegion;
    fn identity(&self) -> &SemanticId {
        &self.identity
    }
    fn context_schema(&self) -> &SemanticId {
        &SCHEMA
    }
    fn initial_context(&self) -> OrdinaryRegion {
        OrdinaryRegion::Initial
    }
    fn validate_context(&self, site: ContextSite<'_>, context: &OrdinaryRegion) -> Result<(), CheckError> {
        let result = (|| {
            ensure!(
                site.relative.components().all(|c| matches!(c, std::path::Component::Normal(_))),
                "invalid ordinary coordinate"
            );
            match context {
                OrdinaryRegion::Initial => ensure!(
                    site.structural_root && site.role.is_none() && site.relative.as_os_str().is_empty(),
                    "initial ordinary context outside root admission"
                ),
                OrdinaryRegion::Search(state) => {
                    ensure!(state.at == site.relative, "search context coordinate differs");
                    ensure!(state.at.to_str().is_some(), "non-UTF-8 search coordinate");
                    // Frozen-value integrity, not selector/ignore/ownership replay.
                    ensure!(state == &self.seal(state.clone()), "search context differs from frozen inputs");
                }
                OrdinaryRegion::SelectedMirror { root, admission } => {
                    ensure!(
                        !root.as_os_str().is_empty() && site.relative.starts_with(root),
                        "selected region does not contain coordinate"
                    );
                    ensure!(
                        root.components()
                            .all(|c| matches!(c, std::path::Component::Normal(n) if n != ".jj" && n != ".git")),
                        "invalid selected root"
                    );
                    ensure!(*admission == self.admission(root), "selected admission differs from frozen inputs");
                }
            }
            Ok(())
        })();
        result.map_err(|e: anyhow::Error| e.to_string().into())
    }
    fn names(&self, context: &OrdinaryRegion, names: &mut Names) -> Result<(), CheckError> {
        if names.is_root() {
            return Ok(());
        }
        for name in names.entries_mut() {
            if name.name() == ".jj" || name.name() == ".git" {
                if matches!(context, OrdinaryRegion::SelectedMirror { .. }) {
                    return Err(name.refuse("administrative entry inside selected ordinary root"));
                }
                name.preserve_tree();
            } else if matches!(context, OrdinaryRegion::Search(_)) && !matches!(name.source().map(|f| f.kind()), Some(FileKind::Unknown | FileKind::Directory))
            {
                name.preserve_tree();
            }
        }
        Ok(())
    }
    fn shape(&self, context: &OrdinaryRegion, batch: &mut ShapeChoices<'_, OrdinaryRegion>) -> Result<(), CheckError> {
        let root = batch.info().structural_root;
        for mut entry in batch.entries_mut() {
            let shape = entry.shape();
            let mut routing = false;
            let next = if root {
                routing = true;
                OrdinaryRegion::Search(self.root())
            } else if let OrdinaryRegion::Search(state) = context {
                if shape.source != SourceObservation::Present(Kind::Directory) {
                    entry.preserve_tree();
                    continue;
                }
                let next = self.advance(state, entry.name()).map_err(|e| CheckError::from(e.to_string()))?;
                #[cfg(test)]
                {
                    self.trace.lock().unwrap().search_steps += 1;
                }
                if self.source.patterns.matched(&next.selector) && next.source_ignore.ignored() {
                    if next.native.is_some() {
                        return Err("selected ordinary root overlaps native tracked leaf".into());
                    }
                    if next.links.is_some() {
                        return Err("selected ordinary root overlaps active link".into());
                    }
                    if !next.desired_ignore.ignored() {
                        return Err("selected ordinary root is not ignored under frozen desired rules".into());
                    }
                    #[cfg(test)]
                    {
                        self.trace.lock().unwrap().admissions.push(next.at.clone());
                    }
                    OrdinaryRegion::SelectedMirror {
                        admission: self.admission(&next.at),
                        root: next.at,
                    }
                } else {
                    if next.links.is_some_and(|i| self.links.0[i].leaf) || !self.source.patterns.can_descend(&next.selector) {
                        entry.preserve_tree();
                        continue;
                    }
                    if next.native.is_some_and(|i| self.source.native.0[i].leaf) {
                        return Err("native leaf blocks ordinary selection route".into());
                    }
                    routing = true;
                    OrdinaryRegion::Search(next)
                }
            } else {
                context.clone()
            };
            if !shape.supported() {
                return Err("unsupported special entry".into());
            }
            if routing {
                if shape.target.is_some_and(|k| k != Kind::Directory) {
                    return Err("ordinary routing ancestor is not a real target directory".into());
                }
                entry.set_action(Action::Directory {
                    container: if shape.target.is_some() { Container::Keep } else { Container::CreateIfNeeded },
                    descend: true,
                });
            }
            entry.request(SideDemands {
                source: Demand {
                    attributes: true,
                    literal_link: false,
                },
                target: Demand {
                    attributes: true,
                    literal_link: false,
                },
            });
            let role = match (shape.source, shape.target) {
                (SourceObservation::Present(Kind::Directory), Some(Kind::Directory)) => Some(InteriorRole::Paired),
                (SourceObservation::Present(Kind::Directory), None) => Some(InteriorRole::SourceOnly),
                (SourceObservation::Present(Kind::Directory), Some(_)) => Some(InteriorRole::PrepareReplacement),
                (SourceObservation::Absent, Some(Kind::Directory)) => Some(InteriorRole::TargetOnly),
                (_, Some(Kind::Directory)) => Some(InteriorRole::RemoveForReplacement),
                _ => None,
            };
            if let Some(role) = role {
                entry.set_interior(Interior {
                    role,
                    context: next,
                    input: InteriorInput::PhysicalDirectory,
                });
            }
        }
        Ok(())
    }
    fn select(&self, _: &OrdinaryRegion, batch: &mut Choices<'_, OrdinaryRegion>) -> Result<(), CheckError> {
        #[cfg(test)]
        let parent = batch.info().relative.to_owned();
        for entry in batch.entries_mut() {
            if entry.is_terminal() {
                continue;
            }
            let facts = entry.facts();
            for (fact, device) in [(facts.source, self.devices.0), (facts.target, self.devices.1)] {
                if let Some(fact) = fact {
                    let metadata = fact.attributes.known().ok_or("device guard attributes unavailable")?;
                    if metadata.dev() != device {
                        return Err("mounted filesystem boundary".into());
                    }
                }
            }
            #[cfg(test)]
            if let Some(fact) = facts.source {
                self.trace
                    .lock()
                    .unwrap()
                    .source_attributes
                    .insert(parent.join(entry.name()), fact.attributes.known().unwrap() as *const _ as usize);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
struct Trace {
    search_steps: usize,
    admissions: Vec<PathBuf>,
    source_attributes: BTreeMap<PathBuf, usize>,
}

pub struct CohortLane {
    pub name: String,
    pub target: PathBuf,
    pub policy: SourceQualified,
}
#[derive(Debug)]
pub struct CohortFailure {
    pub lane: Option<usize>,
    pub reports: Vec<diff_dirs_sync::coordinated::LaneReport>,
    pub source: anyhow::Error,
}
impl std::fmt::Display for CohortFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.source.fmt(f)
    }
}
impl std::error::Error for CohortFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// One physical-union machine for N1 and N2. Inspect every lane before applying
/// any of the group; apply in stable target order and stop at first returned error.
pub fn sync_cohort(
    source: &Path,
    lanes: Vec<CohortLane>,
    resources: diff_dirs::ObservationResources,
) -> std::result::Result<Vec<diff_dirs_sync::coordinated::LaneReport>, CohortFailure> {
    use diff_dirs::Comparator;
    use diff_dirs::coordinated::{Bindings, Parts, TargetBinding, TargetId};
    use diff_dirs_sync::coordinated::{SyncCursor, SyncParts, SyncRuntime, SyncTarget};
    let bindings = Bindings {
        source: source.to_owned(),
        targets: lanes
            .iter()
            .map(|lane| TargetBinding {
                id: TargetId(lane.name.clone()),
                root: lane.target.clone(),
                policy: "jw-contextual-ordinary-v3".into(),
                policy_id: lane.policy.identity().clone(),
                context_schema: lane.policy.context_schema().clone(),
                comparator: StatKey.identity().clone(),
                initial_context: lane.policy.initial_context(),
            })
            .collect(),
    };
    let comparison = Parts::fresh(bindings).map_err(|source| CohortFailure {
        lane: None,
        reports: vec![],
        source: source.into(),
    })?;
    let mut cursor = SyncCursor::from_parts(
        SyncParts {
            version: 2,
            comparison,
            copy_policies: vec![diff_dirs_sync::CopyPolicy::ReflinkRequired; lanes.len()],
            lanes: vec![Default::default(); lanes.len()],
            current: None,
        },
        diff_dirs::Execution::Sequential,
    )
    .map_err(|source| CohortFailure {
        lane: None,
        reports: vec![],
        source: source.into(),
    })?;
    let mut failed_lane = None;
    let runtime = (|| {
        let mut targets = Vec::new();
        for (i, lane) in lanes.into_iter().enumerate() {
            failed_lane = Some(i);
            targets.push(SyncTarget {
                id: TargetId(lane.name),
                descriptor: "jw-contextual-ordinary-v3".into(),
                applier: diff_dirs_sync::Applier::new_policy(source, lane.target, diff_dirs_sync::CopyPolicy::ReflinkRequired, lane.policy, StatKey)?,
            });
        }
        Ok::<_, anyhow::Error>(SyncRuntime::new(targets))
    })()
    .map_err(|source| CohortFailure {
        lane: failed_lane,
        reports: cursor.reports(),
        source,
    })?;
    failed_lane = None;
    let session = Arc::new(diff_dirs::SourceSession::with_resources(source, resources));
    let result = (|| {
        cursor.bind_with_source(&runtime, session)?;
        while let Some(group) = cursor.next_group(&runtime)? {
            let mut first = None;
            for outcome in &group.outcomes {
                if let Err(error) = &outcome.outcome {
                    if first.is_none() {
                        failed_lane = cursor.reports().iter().position(|r| r.target == outcome.target);
                        first = Some(anyhow::Error::new(Arc::clone(error)));
                    }
                }
            }
            if let Some(error) = first {
                return Err(error);
            }
            for target in cursor.pending() {
                failed_lane = cursor.reports().iter().position(|r| r.target == target);
                cursor.apply(&runtime, &group.token, &target)?;
            }
            failed_lane = None;
            cursor.settle()?;
        }
        ensure!(cursor.reports().iter().all(|r| r.satisfied), "ordinary cohort did not satisfy all lanes");
        Ok::<_, anyhow::Error>(())
    })();
    let reports = cursor.reports();
    match result {
        Ok(()) => Ok(reports),
        Err(source) => Err(CohortFailure {
            lane: failed_lane,
            reports,
            source,
        }),
    }
}

/// Compatibility projection; command execution uses sync_cohort directly.
pub fn sync_one(source: &Path, target: &Path, name: &str, policy: SourceQualified) -> Result<Option<diff_dirs_sync::coordinated::LaneReport>> {
    if policy.source.patterns.is_empty() {
        return Ok(None);
    }
    Ok(sync_cohort(
        source,
        vec![CohortLane {
            name: name.into(),
            target: target.into(),
            policy,
        }],
        Default::default(),
    )?
    .pop())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::symlink;

    struct Fixture {
        _temp: tempfile::TempDir,
        source: PathBuf,
        target: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/streaming-tests");
            fs::create_dir_all(&scratch).unwrap();
            let temp = tempfile::tempdir_in(scratch.canonicalize().unwrap()).unwrap();
            let source = temp.path().join("source");
            let target = temp.path().join("target");
            fs::create_dir_all(source.join(".jj")).unwrap();
            fs::create_dir_all(target.join(".jj")).unwrap();
            Self { _temp: temp, source, target }
        }
        fn visitor(&self, patterns: &[&str], native: &[&str], links: &[&str]) -> SourceQualified {
            SourceQualified::new(
                Arc::new(OrdinarySource::new(
                    Arc::new(Patterns::compile(&patterns.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap()),
                    &native.iter().map(PathBuf::from).collect(),
                )),
                Arc::new(FrozenIgnore::test_rules(b"cache/\n**/node_modules/\nforced/\n")),
                Arc::new(FrozenIgnore::test_rules(b"cache/\n**/node_modules/\nforced/\n")),
                &links.iter().map(PathBuf::from).collect(),
                (fs::metadata(&self.source).unwrap().dev(), fs::metadata(&self.target).unwrap().dev()),
            )
        }
        fn run(&self, patterns: &[&str]) -> Result<Option<diff_dirs_sync::coordinated::LaneReport>> {
            sync_one(&self.source, &self.target, "test", self.visitor(patterns, &[], &[]))
        }
    }

    #[test]
    fn frozen_context_validation_is_value_based_and_does_not_read_source_or_readmit() {
        let f = Fixture::new();
        let v = f.visitor(&["cache"], &[], &[]);
        let search = OrdinaryRegion::Search(v.advance(&v.root(), std::ffi::OsStr::new("cache")).unwrap());
        let selected = OrdinaryRegion::SelectedMirror {
            root: "cache".into(),
            admission: v.admission(Path::new("cache")),
        };
        let restored: OrdinaryRegion = serde_json::from_slice(&serde_json::to_vec(&selected).unwrap()).unwrap();
        let independent = f.visitor(&["cache"], &[], &[]);
        let changed = f.visitor(&["forced"], &[], &[]);
        fs::rename(&f.source, f._temp.path().join("gone")).unwrap();
        let site = ContextSite {
            relative: Path::new("cache"),
            structural_root: false,
            role: Some(InteriorRole::Paired),
        };
        independent.validate_context(site, &search).unwrap();
        independent.validate_context(site, &restored).unwrap();
        assert!(changed.validate_context(site, &restored).is_err());
        assert!(v.trace.lock().unwrap().admissions.is_empty());
    }

    #[test]
    fn target_only_non_utf8_local_is_preserved_through_real_cursor() {
        let f = Fixture::new();
        let name = OsString::from_vec(vec![0xff]);
        fs::write(f.target.join(&name), "local").unwrap();
        let mut v = f.visitor(&["cache"], &[], &[]);
        v.source_ignore = Arc::new(FrozenIgnore::default());
        v.desired_ignore = Arc::new(FrozenIgnore::default());
        let done = sync_one(&f.source, &f.target, "byte-local", v).unwrap().unwrap();
        assert!(done.satisfied);
        assert_eq!(done.totals.completed, Default::default());
        assert_eq!(fs::read(f.target.join(&name)).unwrap(), b"local");
    }

    #[test]
    fn source_non_utf8_unselected_leaves_preserve_local_collisions_and_absence() {
        let f = Fixture::new();
        for (index, paired) in [(0xff, true), (0xfe, false)] {
            let file = OsString::from_vec(vec![index, b'f']);
            let link = OsString::from_vec(vec![index, b'l']);
            fs::write(f.source.join(&file), "donor").unwrap();
            symlink("donor-destination", f.source.join(&link)).unwrap();
            if paired {
                fs::write(f.target.join(&file), "local").unwrap();
                symlink("local-destination", f.target.join(&link)).unwrap();
            }
        }
        let done = f.run(&["cache"]).unwrap().unwrap();
        assert!(done.satisfied);
        assert_eq!(done.totals.completed, Default::default());
        assert_eq!(fs::read(f.target.join(OsString::from_vec(vec![0xff, b'f']))).unwrap(), b"local");
        assert_eq!(
            fs::read_link(f.target.join(OsString::from_vec(vec![0xff, b'l']))).unwrap(),
            Path::new("local-destination")
        );
        for suffix in *b"fl" {
            assert!(fs::symlink_metadata(f.target.join(OsString::from_vec(vec![0xfe, suffix]))).is_err());
        }
    }

    #[test]
    fn target_only_non_utf8_selected_interiors_are_removed_through_real_cursor() {
        let f = Fixture::new();
        let name = OsString::from_vec(vec![0xff]);
        fs::create_dir_all(f.source.join("cache")).unwrap();
        fs::create_dir_all(f.target.join("cache/stale").join(&name)).unwrap();
        fs::create_dir_all(f.target.join("cache").join(&name)).unwrap();
        fs::write(f.target.join("cache").join(OsString::from_vec(vec![0xfe])), "paired child").unwrap();
        fs::write(f.target.join("cache/stale").join(&name).join(&name), "opaque removal child").unwrap();
        fs::write(f.target.join("cache").join(&name).join(&name), "byte directory child").unwrap();
        let done = f.run(&["cache"]).unwrap().unwrap();
        assert!(done.satisfied);
        assert!(f.target.join("cache").is_dir());
        assert_eq!(fs::read_dir(f.target.join("cache")).unwrap().count(), 0);
        assert_eq!(done.totals.completed.files_removed, 3);
        assert_eq!(done.totals.completed.directories_removed, 3);
        assert_eq!(f.run(&["cache"]).unwrap().unwrap().totals.completed, Default::default());
    }

    #[test]
    fn source_qualified_mirror_and_target_only_root_retention_through_real_cursor() {
        let f = Fixture::new();
        crate::probe_between(&f.source, &f.target).unwrap();
        fs::create_dir_all(f.source.join("pkg/cache/new")).unwrap();
        fs::create_dir_all(f.target.join("pkg/cache/old/deep")).unwrap();
        fs::create_dir_all(f.target.join("vanished/cache")).unwrap();
        fs::create_dir_all(f.source.join("notignored/node_modules")).unwrap();
        fs::create_dir_all(f.target.join("notignored")).unwrap();
        fs::write(f.source.join("pkg/cache/new/file"), "new").unwrap();
        fs::write(f.source.join("notignored/node_modules/file"), "nested match").unwrap();
        fs::write(f.target.join("pkg/cache/old/deep/file"), "old").unwrap();
        fs::write(f.target.join("vanished/cache/local"), "retain").unwrap();
        fs::write(f.source.join(".env"), "donor").unwrap();
        fs::write(f.target.join(".env"), "local").unwrap();
        symlink("elsewhere", f.source.join("cache")).unwrap();
        fs::create_dir_all(f.target.join("cache")).unwrap();
        fs::write(f.target.join("cache/local"), "retain link candidate").unwrap();
        let patterns = ["**/cache", "notignored", "**/node_modules"];
        let report = f.run(&patterns).unwrap().unwrap();
        assert!(report.satisfied);
        assert_eq!(report.totals.completed.files_reflinked, 2);
        assert_eq!(report.totals.completed.files_copied, 0);
        assert!(!f.target.join("pkg/cache/old").exists());
        assert_eq!(fs::read(f.target.join("vanished/cache/local")).unwrap(), b"retain");
        assert_eq!(fs::read(f.target.join("cache/local")).unwrap(), b"retain link candidate");
        assert_eq!(fs::read(f.target.join(".env")).unwrap(), b"local");
        assert_eq!(f.run(&patterns).unwrap().unwrap().totals.completed, Default::default());
    }

    #[test]
    fn missing_unselected_route_without_matches_creates_no_skeleton_and_empty_policy_never_binds() {
        let f = Fixture::new();
        fs::create_dir_all(f.source.join("pkg/no-match")).unwrap();
        let done = f.run(&["**/node_modules"]).unwrap().unwrap();
        assert!(done.satisfied);
        assert_eq!(done.totals.completed, Default::default());
        assert!(!f.target.join("pkg").exists());
        // Nonexistent roots are fine only because empty policy does no runtime I/O.
        assert!(
            sync_one(
                Path::new("/nonexistent-source"),
                Path::new("/nonexistent-target"),
                "empty",
                f.visitor(&[], &[], &[])
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn wildcard_selection_discovers_through_absent_routes_without_unmatched_skeletons() {
        let f = Fixture::new();
        crate::probe_between(&f.source, &f.target).unwrap();
        fs::create_dir_all(f.source.join("packages/a/node_modules/empty")).unwrap();
        fs::create_dir_all(f.source.join("packages/a/src/unmatched")).unwrap();
        fs::create_dir_all(f.source.join("packages/b/src/unmatched")).unwrap();
        fs::write(f.source.join("packages/a/node_modules/file"), "opaque payload").unwrap();
        fs::create_dir_all(f.source.join("paired/nested/node_modules")).unwrap();
        fs::write(f.source.join("paired/nested/node_modules/file"), "paired payload").unwrap();
        fs::create_dir_all(f.target.join("paired/nested/node_modules/stale/deep")).unwrap();
        fs::write(f.target.join("paired/nested/node_modules/stale/deep/file"), "remove").unwrap();
        fs::create_dir_all(f.target.join("target-only/node_modules")).unwrap();
        fs::write(f.target.join("target-only/node_modules/local"), "retain absent source root").unwrap();
        let done = f.run(&["**/node_modules"]).unwrap().unwrap();
        assert!(done.satisfied);
        assert_eq!(done.totals.completed.files_reflinked, 2);
        assert_eq!(done.totals.completed.files_copied, 0);
        assert_eq!(done.totals.completed.directories_created, 4);
        assert!(f.target.join("packages/a/node_modules/empty").is_dir());
        assert!(!f.target.join("packages/a/src").exists());
        assert!(!f.target.join("packages/b").exists());
        assert!(!f.target.join("paired/nested/node_modules/stale").exists());
        assert_eq!(fs::read(f.target.join("target-only/node_modules/local")).unwrap(), b"retain absent source root");
        assert_eq!(f.run(&["**/node_modules"]).unwrap().unwrap().totals.completed, Default::default());
        fs::write(f.target.join("packages/a/node_modules/file"), "private").unwrap();
        assert_eq!(fs::read(f.source.join("packages/a/node_modules/file")).unwrap(), b"opaque payload");
    }

    #[test]
    fn selected_collision_opaque_copy_and_nested_administration_refusal() {
        let f = Fixture::new();
        crate::probe_between(&f.source, &f.target).unwrap();
        fs::create_dir_all(f.source.join("cache/new")).unwrap();
        fs::write(f.source.join("cache/new/file"), "data").unwrap();
        symlink(".jj", f.target.join("cache")).unwrap();
        fs::create_dir_all(f.source.join("cache/new/.git")).unwrap();
        assert!(format!("{:#}", f.run(&["cache"]).unwrap_err()).contains("administrative"));
        assert_eq!(fs::read_link(f.target.join("cache")).unwrap(), Path::new(".jj"));
        fs::remove_dir(f.source.join("cache/new/.git")).unwrap();
        let socket = std::os::unix::net::UnixListener::bind(f.source.join("cache/new/socket")).unwrap();
        assert!(format!("{:#}", f.run(&["cache"]).unwrap_err()).contains("special"));
        assert_eq!(fs::read_link(f.target.join("cache")).unwrap(), Path::new(".jj"));
        drop(socket);
        fs::remove_file(f.source.join("cache/new/socket")).unwrap();
        // Top-level native administration is excluded pair-wide before metadata.
        let _hidden = std::os::unix::net::UnixListener::bind(f.source.join(".jj/socket")).unwrap();
        assert_eq!(f.run(&["cache"]).unwrap().unwrap().totals.completed.files_reflinked, 1);
        fs::create_dir_all(f.source.join("cache/new/.git")).unwrap();
        assert!(format!("{:#}", f.run(&["cache"]).unwrap_err()).contains("administrative"));
        let v = f.visitor(&["cache"], &["cache/forced"], &[]);
        assert!(format!("{:#}", sync_one(&f.source, &f.target, "overlap", v).unwrap_err()).contains("overlaps native"));
    }
}
