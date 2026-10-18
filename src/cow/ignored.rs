//! Shared ignored-payload policy and selection, independent of workspace command.
//!
//! One compiled pattern set carries all active prefixes through one selection
//! traversal. Selected outer roots prune that traversal, not subsequent frozen
//! ignore-rule, filesystem, payload-digest, or real-CoW validation/copy passes.
use crate::config::Config;
use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::fs;
use std::ops::Bound;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Origin {
    Disabled,
    Cli,
    Local,
    Global,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Policy {
    pub patterns: Vec<String>,
    pub origin: Origin,
}

impl Policy {
    /// Creation shares only the copy list, not reseed's source/freshening settings.
    pub fn local_patterns(primary: &Path) -> Result<Option<Vec<String>>> {
        #[derive(serde::Deserialize)]
        struct CopyPolicy {
            copy_ignored: Option<Vec<String>>,
        }
        let path = primary.join(".jwreseed.local.toml");
        match fs::read_to_string(&path) {
            Ok(text) => Ok(toml::from_str::<CopyPolicy>(&text)
                .with_context(|| format!("invalid {}", path.display()))?
                .copy_ignored),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error).with_context(|| format!("cannot read {}", path.display())),
        }
    }
    pub fn resolve(cli: Option<&[String]>, local: Option<&[String]>, global: Option<&[String]>, disabled: bool) -> Self {
        if disabled {
            return Self {
                patterns: Vec::new(),
                origin: Origin::Disabled,
            };
        }
        for (patterns, origin) in [(cli, Origin::Cli), (local, Origin::Local), (global, Origin::Global)] {
            if let Some(patterns) = patterns {
                return Self {
                    patterns: patterns.to_vec(),
                    origin,
                };
            }
        }
        Self {
            patterns: Vec::new(),
            origin: Origin::None,
        }
    }

    /// Read global defaults only when they are actually inherited. In particular,
    /// an explicit [] must not start reading global policy.
    pub fn load(cli: Option<&[String]>, local: Option<&[String]>, disabled: bool) -> Result<Self> {
        if disabled || cli.is_some() || local.is_some() {
            return Ok(Self::resolve(cli, local, None, disabled));
        }
        let config = Config::load()?;
        Ok(Self::resolve(None, None, config.copy_ignored.as_deref(), false))
    }

    pub fn describe(&self) -> String {
        format!("Ignored-copy policy {:?}: {:?}", self.origin, self.patterns)
    }
}

#[derive(Debug)]
enum Segment {
    Recursive,
    Component(Vec<char>),
}

#[derive(Debug)]
struct Pattern {
    display: String,
    segments: Vec<Segment>,
}

/// Compile every selector once; directories are never walked once per pattern.
#[derive(Debug)]
pub(crate) struct Patterns {
    patterns: Vec<Pattern>,
}

#[derive(Debug, Default)]
pub(crate) struct Selection {
    pub roots: Vec<PathBuf>,
    pub skipped: Vec<String>,
}

/// Revalidate an already frozen root set without rediscovering additional roots
/// under today's matching semantics. Recovery owns this exact payload scope.
pub(crate) fn validate_selected<'a>(
    source: &Path,
    roots: &[PathBuf],
    tracked: impl IntoIterator<Item = &'a Path>,
    mut is_ignored: impl FnMut(&Path) -> Result<bool>,
) -> Result<()> {
    validate_selected_with_topology(source, roots, tracked, &mut is_ignored, false)
}

pub(crate) fn validate_selected_with_topology<'a>(
    source: &Path,
    roots: &[PathBuf],
    tracked: impl IntoIterator<Item = &'a Path>,
    mut is_ignored: impl FnMut(&Path) -> Result<bool>,
    topology_checked: bool,
) -> Result<()> {
    if roots.is_empty() {
        return Ok(());
    }
    if !fs::symlink_metadata(source)?.is_dir() {
        bail!("ignored-copy source is not a real directory: {}", source.display());
    }
    let tracked = tracked.into_iter().collect();
    for root in roots {
        if root.as_os_str().is_empty()
            || root
                .components()
                .any(|part| !matches!(part, Component::Normal(name) if name != ".jj" && name != ".git"))
        {
            bail!("unsafe recorded ignored root: {}", root.display());
        }
        let mut parent = source.to_path_buf();
        for component in root.components() {
            let child = parent.join(component);
            if !fs::symlink_metadata(&child)?.is_dir() {
                bail!("recorded ignored root/parent is not a real directory: {}", child.display());
            }
            if !topology_checked {
                super::validate_same_mount(&parent, &child)?;
            }
            parent = child;
        }
        if !is_ignored(root)? {
            bail!("recorded ignored source root is no longer ignored: {}", root.display());
        }
        require_untracked(root, &tracked)?;
    }
    Ok(())
}

pub(crate) fn require_untracked(path: &Path, tracked: &BTreeSet<&Path>) -> Result<()> {
    let descendant = tracked.range::<Path, _>((Bound::Included(path), Bound::Unbounded)).next();
    if descendant.is_some_and(|tracked| tracked.starts_with(path)) || path.ancestors().any(|ancestor| tracked.contains(ancestor)) {
        bail!("ignored root overlaps tracked files: {}", path.display());
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct State {
    pattern: usize,
    offset: usize,
}

impl Patterns {
    pub fn compile(patterns: &[String]) -> Result<Self> {
        let mut seen = BTreeSet::new();
        let mut compiled = Vec::new();
        for pattern in patterns {
            let normalized = pattern.trim_end_matches('/');
            if normalized.is_empty()
                || pattern.contains(['[', ']', '\\', '\0'])
                || normalized.split('/').any(|part| part.is_empty() || matches!(part, "." | ".." | ".jj" | ".git"))
                || Path::new(normalized).components().any(|part| !matches!(part, Component::Normal(_)))
            {
                bail!("unsafe ignored directory selection {pattern:?}; use relative literal components, *, ?, and ** (optional trailing /)");
            }
            if !seen.insert(normalized.to_owned()) {
                continue;
            }
            compiled.push(Pattern {
                display: pattern.clone(),
                segments: normalized
                    .split('/')
                    .map(|part| {
                        if part == "**" {
                            Segment::Recursive
                        } else {
                            Segment::Component(part.chars().collect())
                        }
                    })
                    .collect(),
            });
        }
        Ok(Self { patterns: compiled })
    }

    /// `is_ignored` must query the caller's frozen desired source rules. Native
    /// query failures are errors, not nonignored skips. Frozen tracked paths are
    /// checked separately so --no-index ignore probes cannot admit forced-tracked
    /// payload. Selected roots are not cloned or recursively validated here.
    pub fn select<'a>(
        &self,
        source: &'a Path,
        tracked: impl IntoIterator<Item = &'a Path>,
        is_ignored: impl FnMut(&Path) -> Result<bool>,
    ) -> Result<Selection> {
        self.select_with_topology(source, tracked, is_ignored, false)
    }

    /// Linux reseed has already rejected nested mounts for the whole batch.
    pub fn select_with_topology<'a>(
        &self,
        source: &'a Path,
        tracked: impl IntoIterator<Item = &'a Path>,
        is_ignored: impl FnMut(&Path) -> Result<bool>,
        topology_checked: bool,
    ) -> Result<Selection> {
        if self.patterns.is_empty() {
            return Ok(Selection::default());
        }
        if !fs::symlink_metadata(source)?.is_dir() {
            bail!("ignored-copy source is not a real directory: {}", source.display());
        }
        let states = self.close(self.patterns.iter().enumerate().map(|(pattern, _)| State { pattern, offset: 0 }));
        let mut walk = Walk {
            source,
            patterns: self,
            tracked: tracked.into_iter().collect(),
            is_ignored,
            selected: Selection::default(),
            observed: vec![false; self.patterns.len()],
            covered: vec![false; self.patterns.len()],
            topology_checked,
        };
        walk.visit(Path::new(""), &states)?;
        for (index, pattern) in self.patterns.iter().enumerate() {
            if !walk.observed[index] && !walk.covered[index] {
                walk.selected
                    .skipped
                    .push(format!("ignored selection {:?} matched no source directories", pattern.display));
            }
        }
        walk.selected.roots.sort();
        walk.selected.skipped.sort();
        Ok(walk.selected)
    }

    /// Epsilon transitions let ** consume zero components, without another walk.
    fn close(&self, states: impl IntoIterator<Item = State>) -> Vec<State> {
        let mut closed = BTreeSet::new();
        for mut state in states {
            loop {
                closed.insert(state);
                if matches!(self.patterns[state.pattern].segments.get(state.offset), Some(Segment::Recursive)) {
                    state.offset += 1;
                } else {
                    break;
                }
            }
        }
        closed.into_iter().collect()
    }

    fn advance(&self, states: &[State], name: &str) -> Vec<State> {
        let name = name.chars().collect::<Vec<_>>();
        self.close(states.iter().filter_map(|state| match self.patterns[state.pattern].segments.get(state.offset) {
            Some(Segment::Recursive) => Some(*state),
            Some(Segment::Component(pattern)) if component_matches(pattern, &name) => Some(State {
                offset: state.offset + 1,
                ..*state
            }),
            _ => None,
        }))
    }

    fn can_descend(&self, states: &[State]) -> bool {
        states.iter().any(|state| state.offset < self.patterns[state.pattern].segments.len())
    }
}

struct Walk<'a, 'p, F> {
    source: &'a Path,
    patterns: &'p Patterns,
    tracked: BTreeSet<&'a Path>,
    is_ignored: F,
    selected: Selection,
    observed: Vec<bool>,
    /// Potential matches under an already selected ancestor need not be visited
    /// or spuriously reported as missing: that ancestor's payload includes them.
    covered: Vec<bool>,
    topology_checked: bool,
}

impl<F: FnMut(&Path) -> Result<bool>> Walk<'_, '_, F> {
    fn visit(&mut self, relative: &Path, states: &[State]) -> Result<()> {
        let directory = self.source.join(relative);
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let name = entry.file_name();
            if name == ".jj" || name == ".git" || !entry.file_type()?.is_dir() {
                continue; // Never follow directory symlinks or enter metadata.
            }
            let next = self.patterns.advance(states, name.to_str().context("directory selection entry not UTF-8")?);
            if next.is_empty() {
                continue;
            }
            if !self.topology_checked {
                super::validate_same_mount(&directory, &entry.path())?;
            }
            let path = relative.join(name);
            let mut matched = false;
            for state in &next {
                if state.offset == self.patterns.patterns[state.pattern].segments.len() {
                    self.observed[state.pattern] = true;
                    matched = true;
                }
            }
            if matched {
                if (self.is_ignored)(&path)? {
                    require_untracked(&path, &self.tracked)?;
                    self.selected.roots.push(path);
                    for state in &next {
                        if state.offset < self.patterns.patterns[state.pattern].segments.len() {
                            self.covered[state.pattern] = true;
                        }
                    }
                    continue; // Outermost eligible root already includes descendants.
                }
                self.selected.skipped.push(format!(
                    "ignored selection skipped source directory {path:?}: not ignored under frozen source rules"
                ));
            }
            if self.patterns.can_descend(&next) {
                self.visit(&path, &next)?;
            }
        }
        Ok(())
    }
}

/// Compiled component glob, with Unicode-aware ? and iterative * backtracking.
fn component_matches(pattern: &[char], text: &[char]) -> bool {
    let (mut p, mut t) = (0, 0);
    let mut star = None;
    while t < text.len() {
        match pattern.get(p) {
            Some('*') => {
                star = Some((p, t));
                p += 1;
            }
            Some(c) if *c == '?' || *c == text[t] => {
                p += 1;
                t += 1;
            }
            _ => match star {
                Some((s, consumed)) if consumed < text.len() => {
                    star = Some((s, consumed + 1));
                    p = s + 1;
                    t = consumed + 1;
                }
                _ => return false,
            },
        }
    }
    pattern[p..].iter().all(|c| *c == '*')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_empty_policy_does_not_inherit_lower_layers() {
        let cli = vec!["cli".into()];
        let local = vec!["local".into()];
        let global = vec!["**/target".into()];
        assert_eq!(Policy::resolve(Some(&cli), Some(&local), Some(&global), false).patterns, cli);
        assert_eq!(
            Policy::resolve(None, Some(&[]), Some(&global), false),
            Policy {
                patterns: vec![],
                origin: Origin::Local
            }
        );
        assert_eq!(Policy::resolve(Some(&[]), Some(&local), Some(&global), false).origin, Origin::Cli);
        assert_eq!(Policy::resolve(None, None, Some(&global), false).patterns, global);
        assert_eq!(Policy::resolve(Some(&cli), Some(&local), Some(&global), true).origin, Origin::Disabled);
        assert_eq!(Policy::resolve(None, None, None, false).origin, Origin::None);
    }

    #[cfg(unix)]
    #[test]
    fn all_patterns_share_a_walk_and_nonignored_matches_do_not_hide_descendants() {
        let temp = tempfile::tempdir().unwrap();
        for path in [
            "build/tracked",
            "build/packages/a/node_modules",
            "packages/a/node_modules/deep/target",
            "packages/b/target",
            "packages/λ/dist",
            ".jj/node_modules",
            "normal/.git/target",
        ] {
            fs::create_dir_all(temp.path().join(path)).unwrap();
        }
        std::os::unix::fs::symlink(temp.path().join("packages/a"), temp.path().join("linked")).unwrap();
        let patterns = Patterns::compile(&[
            "**/node_modules/".into(),
            "**/target".into(),
            "**/build/".into(),
            "packages/?/dist/".into(),
            "packages/a/node_modules/deep/target".into(),
            "absent/".into(),
        ])
        .unwrap();
        let tracked = [PathBuf::from("build/tracked/file")];
        let mut checked = Vec::new();
        let selection = patterns
            .select(temp.path(), tracked.iter().map(PathBuf::as_path), |path| {
                checked.push(path.to_owned());
                Ok(path != Path::new("build"))
            })
            .unwrap();
        assert_eq!(
            selection.roots,
            [
                "build/packages/a/node_modules",
                "packages/a/node_modules",
                "packages/b/target",
                "packages/λ/dist"
            ]
            .map(PathBuf::from)
        );
        assert!(!checked.iter().any(|path| path.starts_with("linked")
            || path.starts_with(".jj")
            || path.starts_with("normal/.git")
            || path == Path::new("packages/a/node_modules/deep/target")));
        assert_eq!(selection.skipped.len(), 2);
        assert!(selection.skipped.iter().any(|message| message.contains("not ignored")));
        assert!(selection.skipped.iter().any(|message| message.contains("absent/")));
    }

    #[cfg(unix)]
    #[test]
    fn selected_ignored_roots_cannot_hide_tracked_overlap_and_missing_is_success() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("cache/nested")).unwrap();
        let tracked = [PathBuf::from("cache/nested/forced-tracked")];
        let patterns = Patterns::compile(&["cache/".into(), "cache/nested".into()]).unwrap();
        assert!(
            patterns
                .select(temp.path(), tracked.iter().map(PathBuf::as_path), |_| Ok(true))
                .unwrap_err()
                .to_string()
                .contains("overlaps tracked")
        );
        let selection = patterns.select(temp.path(), [], |_| Ok(true)).unwrap();
        assert_eq!(selection.roots, [PathBuf::from("cache")]);
        assert!(validate_selected(temp.path(), &selection.roots, tracked.iter().map(PathBuf::as_path), |_| Ok(true)).is_err());
        assert!(validate_selected(temp.path(), &selection.roots, [], |_| Ok(false)).is_err());
        std::os::unix::fs::symlink(temp.path().join("cache"), temp.path().join("linked")).unwrap();
        assert!(
            validate_selected(temp.path(), &[PathBuf::from("linked/nested")], [], |_| Ok(true))
                .unwrap_err()
                .to_string()
                .contains("not a real directory")
        );
        let empty = Patterns::compile(&["no-such-root/".into()])
            .unwrap()
            .select(temp.path(), [], |_| Ok(true))
            .unwrap();
        assert!(empty.roots.is_empty());
        for pattern in ["../*", "**/.jj", "/cache", "packages//cache", "cache/[a]", "cache\\nested"] {
            assert!(Patterns::compile(&[pattern.into()]).is_err(), "{pattern}");
        }
    }
}
