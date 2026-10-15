//! Snapshot-free group resolution. Membership is a registration fact, never an
//! inference from a directory basename or a reason to adopt another checkout.

use super::{ReseedRequest, ReseedSelection, ReseedSelectionMiss, UnmatchedPolicy};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum Group {
    Names(Vec<String>),
    Selectors(GroupSelectors),
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct GroupSelectors {
    names: Vec<String>,
    name_globs: Vec<String>,
    path_globs: Vec<String>,
    on_unmatched: UnmatchedPolicy,
}

/// All registrations, including unavailable roots. A missing root must not turn
/// a registered (but broken) target into a warn-and-skip candidate.
pub(super) type Registrations = BTreeMap<String, Option<PathBuf>>;

#[derive(Debug, Default)]
pub(super) struct SelectionPlan {
    pub selections: Vec<ReseedSelection>,
    pub skipped: Vec<ReseedSelectionMiss>,
    errors: Vec<String>,
}

impl SelectionPlan {
    pub fn warning(&self) -> Option<String> {
        (!self.skipped.is_empty()).then(|| {
            format!(
                "warning: skipped {} unmatched group candidates: {}",
                self.skipped.len(),
                describe_misses(&self.skipped)
            )
        })
    }

    pub fn validate(&self) -> Result<()> {
        if !self.errors.is_empty() {
            bail!("reseed selection failed: {}", self.errors.join("; "));
        }
        if self.selections.is_empty() {
            bail!("no workspaces selected; nothing reseeded");
        }
        Ok(())
    }
}

fn describe_misses(misses: &[ReseedSelectionMiss]) -> String {
    misses
        .iter()
        .map(|miss| format!("{} ({})", miss.candidate, miss.reason))
        .collect::<Vec<_>>()
        .join(", ")
}

struct Resolver<'a> {
    registrations: &'a Registrations,
    plan: SelectionPlan,
    // Strict wins when the same miss is encountered through several selectors.
    misses: BTreeMap<ReseedSelectionMiss, bool>,
}

impl Resolver<'_> {
    fn miss(&mut self, candidate: String, reason: &str, policy: UnmatchedPolicy) {
        let strict = self
            .misses
            .entry(ReseedSelectionMiss {
                candidate,
                reason: reason.to_owned(),
            })
            .or_default();
        *strict |= policy == UnmatchedPolicy::Error;
    }

    fn name(&mut self, name: &str, origin: &str, policy: UnmatchedPolicy) -> Result<()> {
        if name.is_empty() || name.contains('\0') {
            bail!("reseed requires a nonempty literal workspace name without NUL");
        }
        let Some(root) = self.registrations.get(name) else {
            self.miss(quote(name), "unknown workspace", policy);
            return Ok(());
        };
        // Root lookup errors are hard failures, regardless of group policy.
        let Some(root) = root else {
            self.plan.errors.push(format!(
                "workspace {} has no usable recorded root",
                quote(name)
            ));
            return Ok(());
        };
        if let Some(selected) = self.plan.selections.iter_mut().find(|s| s.name == name) {
            if !selected.origins.iter().any(|item| item == origin) {
                selected.origins.push(origin.to_owned());
            }
        } else {
            self.plan.selections.push(ReseedSelection {
                name: name.to_owned(),
                root: root.clone(),
                origins: vec![origin.to_owned()],
            });
        }
        Ok(())
    }

    fn paths(
        &mut self,
        pattern: &str,
        base: &Path,
        home: Option<&Path>,
        origin: &str,
        policy: UnmatchedPolicy,
    ) -> Result<()> {
        let pattern = path_pattern(pattern, base, home)?;
        // If a registration cannot be resolved, we cannot prove a candidate is
        // unrelated to it. Never downgrade incomplete inventory to a skip.
        if let Some((name, _)) = self.registrations.iter().find(|(_, root)| root.is_none()) {
            bail!(
                "cannot resolve registered workspace {} for path selection",
                quote(name)
            );
        }
        let mut candidates: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
        // Include matching recorded paths even when absent from disk. This makes
        // a stale registration a selected target that fails normal validation.
        for (name, root) in self.registrations {
            if root
                .as_ref()
                .is_some_and(|root| path_matches(&pattern, root))
            {
                candidates
                    .entry(root.as_ref().unwrap().clone())
                    .or_default()
                    .push(name.clone());
            }
        }
        for candidate in expand_paths(&pattern)? {
            candidates.entry(candidate).or_default();
        }
        if candidates.is_empty() {
            self.miss(quote_path(&pattern.absolute()), "no matches", policy);
        }
        for (candidate, names) in candidates {
            if !names.is_empty() {
                if let Some(obstruction) = non_directory_ancestor(&candidate)? {
                    self.plan.errors.push(format!(
                        "registered workspace path {} is not a real directory: {}",
                        quote_path(&candidate),
                        quote_path(&obstruction)
                    ));
                }
                for name in names {
                    self.name(&name, origin, policy)?;
                }
                continue;
            }
            let metadata = fs::symlink_metadata(&candidate).with_context(|| {
                format!("cannot inspect group candidate {}", quote_path(&candidate))
            })?;
            self.miss(
                quote_path(&candidate),
                if metadata.is_dir() {
                    "not a workspace of this repository"
                } else {
                    "not a real workspace directory"
                },
                policy,
            );
        }
        Ok(())
    }
}

pub(super) fn resolve(
    request: &ReseedRequest,
    groups: &BTreeMap<String, Group>,
    registrations: &Registrations,
    base: &Path,
    home: Option<&Path>,
) -> Result<SelectionPlan> {
    if request.names.is_empty() && request.groups.is_empty() {
        bail!("reseed requires literal workspace names or --group; never selects all implicitly");
    }
    let mut resolver = Resolver {
        registrations,
        plan: SelectionPlan::default(),
        misses: BTreeMap::new(),
    };
    for name in &request.names {
        resolver.name(name, "positional", UnmatchedPolicy::Error)?;
    }
    for group_name in &request.groups {
        let group = groups
            .get(group_name)
            .with_context(|| format!("unknown reseed group {}", quote(group_name)))?;
        let origin = format!("group {}", quote(group_name));
        let (names, name_globs, path_globs, default_policy) = match group {
            Group::Names(names) => (names.as_slice(), &[][..], &[][..], UnmatchedPolicy::Error),
            Group::Selectors(group) => (
                group.names.as_slice(),
                group.name_globs.as_slice(),
                group.path_globs.as_slice(),
                group.on_unmatched,
            ),
        };
        if names.is_empty() && name_globs.is_empty() && path_globs.is_empty() {
            bail!("{origin} requires at least one nonempty selector list");
        }
        let policy = request.on_unmatched.unwrap_or(default_policy);
        for name in names {
            resolver.name(name, &format!("{origin} names"), policy)?;
        }
        for pattern in name_globs {
            validate_glob(pattern)?;
            if pattern.contains('/') {
                bail!(
                    "name glob cannot contain a path separator: {}",
                    quote(pattern)
                );
            }
            let origin = format!("{origin} name_globs {}", quote(pattern));
            let mut matched = false;
            for name in registrations.keys() {
                if component_glob(pattern, name) {
                    matched = true;
                    resolver.name(name, &origin, policy)?;
                }
            }
            if !matched {
                resolver.miss(quote(pattern), "no matches", policy);
            }
        }
        for pattern in path_globs {
            resolver.paths(
                pattern,
                base,
                home,
                &format!("{origin} path_globs {}", quote(pattern)),
                policy,
            )?;
        }
    }
    for (miss, strict) in resolver.misses {
        if strict {
            resolver
                .plan
                .errors
                .push(format!("{} ({})", miss.candidate, miss.reason));
        } else {
            resolver.plan.skipped.push(miss);
        }
    }
    Ok(resolver.plan)
}

fn validate_glob(pattern: &str) -> Result<()> {
    if pattern.is_empty()
        || pattern.contains(['\0', '[', ']', '{', '}', '\\'])
        || pattern.contains("**")
    {
        bail!(
            "group patterns support nonempty literal components, * and ? only: {}",
            quote(pattern)
        );
    }
    Ok(())
}

struct PathPattern {
    prefix: PathBuf,
    pattern: PathBuf,
}

impl PathPattern {
    fn absolute(&self) -> PathBuf {
        self.prefix.join(&self.pattern)
    }
}

fn path_pattern(pattern: &str, base: &Path, home: Option<&Path>) -> Result<PathPattern> {
    validate_glob(pattern)?;
    let (prefix, relative) = if let Some(relative) = pattern.strip_prefix("~/") {
        (
            home.context("cannot expand ~/ without a home directory")?,
            Path::new(relative),
        )
    } else {
        if pattern.starts_with('~') {
            bail!("group paths support ~/ but not ~user: {}", quote(pattern));
        }
        let path = Path::new(pattern);
        if path.is_absolute() {
            (Path::new("/"), path.strip_prefix("/")?)
        } else {
            (base, path)
        }
    };
    if !prefix.is_absolute()
        || prefix
            .components()
            .chain(relative.components())
            .any(|component| component == Component::ParentDir)
    {
        bail!(
            "group paths cannot contain .. components: {}",
            quote(pattern)
        );
    }
    Ok(PathPattern {
        prefix: prefix.to_path_buf(),
        pattern: relative
            .components()
            .filter(|c| *c != Component::CurDir)
            .collect(),
    })
}

fn path_matches(pattern: &PathPattern, path: &Path) -> bool {
    let Ok(path) = path.strip_prefix(&pattern.prefix) else {
        return false;
    };
    let pattern = pattern.pattern.components().collect::<Vec<_>>();
    let path = path.components().collect::<Vec<_>>();
    pattern.len() == path.len()
        && pattern
            .iter()
            .zip(path)
            .all(|(pattern, value)| os_glob(pattern.as_os_str(), value.as_os_str()))
}

/// Bounded component traversal. Preserve non-UTF-8 candidates and never follow a
/// symlink, including one in the literal prefix of a configured pattern.
fn expand_paths(pattern: &PathPattern) -> Result<Vec<PathBuf>> {
    if let Some(obstruction) = non_directory_ancestor(&pattern.prefix)? {
        return Ok(vec![obstruction]);
    }
    match fs::symlink_metadata(&pattern.prefix) {
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("cannot inspect {}", quote_path(&pattern.prefix)));
        }
    }
    let mut paths = vec![pattern.prefix.clone()];
    for component in pattern.pattern.components() {
        let Component::Normal(pattern) = component else {
            continue;
        };
        let mut next = Vec::new();
        for parent in paths {
            let metadata = match fs::symlink_metadata(&parent) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == ErrorKind::NotFound => continue,
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("cannot inspect {}", quote_path(&parent)));
                }
            };
            if !metadata.is_dir() {
                // Report the obstruction rather than silently traversing it.
                next.push(parent);
                continue;
            }
            if pattern
                .as_bytes()
                .iter()
                .any(|byte| matches!(byte, b'*' | b'?'))
            {
                for child in fs::read_dir(&parent)
                    .with_context(|| format!("cannot enumerate {}", quote_path(&parent)))?
                {
                    let child = child?;
                    if os_glob(pattern, &child.file_name()) {
                        next.push(child.path());
                    }
                }
            } else {
                let child = parent.join(pattern);
                match fs::symlink_metadata(&child) {
                    Ok(_) => next.push(child),
                    Err(error) if error.kind() == ErrorKind::NotFound => {}
                    Err(error) => {
                        return Err(error)
                            .with_context(|| format!("cannot inspect {}", quote_path(&child)));
                    }
                }
            }
        }
        paths = next;
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn non_directory_ancestor(path: &Path) -> Result<Option<PathBuf>> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if !metadata.is_dir() => return Ok(Some(current)),
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("cannot inspect {}", quote_path(&current)));
            }
        }
    }
    Ok(None)
}

fn os_glob(pattern: &OsStr, text: &OsStr) -> bool {
    if let (Some(pattern), Some(text)) = (pattern.to_str(), text.to_str()) {
        component_glob(pattern, text)
    } else {
        glob_items(pattern.as_bytes(), text.as_bytes(), b'*', b'?')
    }
}

pub(super) fn component_glob(pattern: &str, text: &str) -> bool {
    glob_items(
        &pattern.chars().collect::<Vec<_>>(),
        &text.chars().collect::<Vec<_>>(),
        '*',
        '?',
    )
}

fn glob_items<T: PartialEq>(pattern: &[T], text: &[T], star: T, question: T) -> bool {
    let mut matches = vec![false; text.len() + 1];
    matches[0] = true;
    for c in pattern {
        let mut next = vec![false; text.len() + 1];
        if c == &star {
            next[0] = matches[0];
        }
        for i in 1..=text.len() {
            next[i] = if c == &star {
                matches[i] || next[i - 1]
            } else {
                matches[i - 1] && (c == &question || c == &text[i - 1])
            };
        }
        matches = next;
    }
    matches[text.len()]
}

fn quote(text: &str) -> String {
    // JSON handles C0 controls; also escape terminal controls and Unicode line
    // separators that JSON leaves literal, keeping diagnostics one safe line.
    serde_json::to_string(text)
        .expect("string serialization cannot fail")
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') {
                format!("\\u{:04x}", c as u32)
            } else {
                c.to_string()
            }
        })
        .collect()
}

fn quote_path(path: &Path) -> String {
    let mut result = String::from("\"");
    for chunk in path.as_os_str().as_bytes().utf8_chunks() {
        let escaped = quote(chunk.valid());
        result.push_str(&escaped[1..escaped.len() - 1]);
        for byte in chunk.invalid() {
            use std::fmt::Write;
            write!(result, "\\x{byte:02x}").expect("string write cannot fail");
        }
    }
    result.push('"');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::symlink;

    #[derive(Deserialize)]
    struct Config {
        groups: BTreeMap<String, Group>,
    }

    fn tempdir() -> tempfile::TempDir {
        // macOS temporary-directory prefixes can themselves be symlink aliases.
        // Use the real root so these fixtures test the configured path, not an
        // incidental system alias that production selection correctly refuses.
        tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap()
    }

    fn plan(
        config: &str,
        request: ReseedRequest,
        registered: &Registrations,
        base: &Path,
    ) -> Result<SelectionPlan> {
        let config: Config = toml::from_str(config)?;
        resolve(&request, &config.groups, registered, base, Some(base))
    }

    fn request(groups: &[&str]) -> ReseedRequest {
        ReseedRequest {
            groups: groups.iter().map(|s| (*s).into()).collect(),
            ..Default::default()
        }
    }

    fn registrations(base: &Path, names: &[&str]) -> Registrations {
        names
            .iter()
            .map(|name| ((*name).into(), Some(base.join(name))))
            .collect()
    }

    #[test]
    fn literal_arrays_remain_literal_and_deduplicate_without_selectors() {
        let registered = registrations(
            Path::new("/repo"),
            &["a", "b", "one-off", "a*", "@", "^", "-"],
        );
        let mut req = request(&["workers"]);
        req.names = vec!["b".into(), "one-off".into()];
        let result = plan(
            "[groups]\nworkers=['a','b','a','a*','@','^','-']",
            req,
            &registered,
            Path::new("/repo"),
        )
        .unwrap();
        result.validate().unwrap();
        assert_eq!(
            result
                .selections
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            ["b", "one-off", "a", "a*", "@", "^", "-"]
        );
        assert_eq!(result.selections[0].origins.len(), 2);
        assert!(
            plan(
                "[groups]",
                ReseedRequest::default(),
                &registered,
                Path::new("/repo")
            )
            .is_err()
        );
    }

    #[test]
    fn group_and_cli_policy_never_weaken_positional_names() {
        let config =
            "[groups.warn]\nnames=['absent','a']\non_unmatched='warn'\n[groups]\nstrict=['absent']";
        let registered = registrations(Path::new("/repo"), &["a"]);
        let result = plan(config, request(&["warn"]), &registered, Path::new("/repo")).unwrap();
        result.validate().unwrap();
        assert_eq!(result.skipped.len(), 1);
        let result = plan(
            config,
            request(&["warn", "strict"]),
            &registered,
            Path::new("/repo"),
        )
        .unwrap();
        assert!(result.validate().is_err());
        assert!(result.warning().is_none(), "strict wins for duplicate miss");
        let mut req = request(&["warn", "strict"]);
        req.on_unmatched = Some(UnmatchedPolicy::Warn);
        let result = plan(config, req.clone(), &registered, Path::new("/repo")).unwrap();
        result.validate().unwrap();
        assert_eq!(result.skipped.len(), 1);
        req.names.push("absent".into());
        let result = plan(config, req, &registered, Path::new("/repo")).unwrap();
        assert!(result.validate().is_err());
        assert!(result.skipped.is_empty());
        let mut req = request(&["warn"]);
        req.on_unmatched = Some(UnmatchedPolicy::Error);
        assert!(
            plan(config, req, &registered, Path::new("/repo"))
                .unwrap()
                .validate()
                .is_err()
        );
    }

    #[test]
    fn name_patterns_include_unavailable_registrations() {
        let mut registered = registrations(Path::new("/repo"), &["ocw-a", "other"]);
        registered.insert("ocw-stale".into(), None);
        let result = plan(
            "[groups.w]\nname_globs=['ocw-*']\non_unmatched='warn'",
            request(&["w"]),
            &registered,
            Path::new("/repo"),
        )
        .unwrap();
        assert!(
            result
                .validate()
                .unwrap_err()
                .to_string()
                .contains("ocw-stale")
        );
        assert!(result.skipped.is_empty());
    }

    #[test]
    fn paths_use_registration_not_basename_and_preserve_path_order() {
        let temp = tempdir();
        let base = temp.path();
        for name in ["opencode-a", "opencode-b", "opencode-notes", "unrelated"] {
            fs::create_dir(base.join(name)).unwrap();
        }
        let registered = BTreeMap::from([
            ("z-name".into(), Some(base.join("opencode-a"))),
            ("a-name".into(), Some(base.join("opencode-b"))),
        ]);
        for pattern in ["opencode-*", "~/opencode-*"] {
            let config = format!("[groups.w]\npath_globs=[{pattern:?}]\non_unmatched='warn'");
            let result = plan(&config, request(&["w"]), &registered, base).unwrap();
            result.validate().unwrap();
            assert_eq!(
                result
                    .selections
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>(),
                ["z-name", "a-name"]
            );
            assert_eq!(result.skipped.len(), 1);
            assert!(result.warning().unwrap().contains("opencode-notes"));
            assert!(!result.warning().unwrap().contains("unrelated"));
        }
    }

    #[test]
    fn missing_recorded_paths_are_not_silently_skipped() {
        let temp = tempdir();
        let registered = registrations(temp.path(), &["opencode-missing"]);
        let result = plan(
            "[groups.w]\npath_globs=['opencode-*']\non_unmatched='warn'",
            request(&["w"]),
            &registered,
            temp.path(),
        )
        .unwrap();
        assert_eq!(result.selections[0].name, "opencode-missing");
        assert!(result.skipped.is_empty()); // Engine root validation must fail.
    }

    #[test]
    fn misses_are_aggregated_escaped_and_never_lossy() {
        let temp = tempdir();
        for name in [
            b"opencode-\nquote\"".to_vec(),
            b"opencode-\xff".to_vec(),
            b"opencode-\xfe".to_vec(),
        ] {
            fs::create_dir(temp.path().join(std::ffi::OsString::from_vec(name))).unwrap();
        }
        let result = plan(
            "[groups.w]\npath_globs=['opencode-*','opencode-*']\non_unmatched='warn'",
            request(&["w"]),
            &Registrations::new(),
            temp.path(),
        )
        .unwrap();
        let warning = result.warning().unwrap();
        assert_eq!(result.skipped.len(), 3);
        assert_eq!(warning.lines().count(), 1);
        for text in ["\\nquote\\\"", "\\xff", "\\xfe"] {
            assert!(warning.contains(text), "{warning}");
        }
        assert!(
            result
                .validate()
                .unwrap_err()
                .to_string()
                .contains("no workspaces selected")
        );
    }

    #[test]
    fn zero_matches_report_expression_and_invalid_groups_fail() {
        let temp = tempdir();
        let result = plan(
            "[groups.w]\npath_globs=['missing-*']\nname_globs=['missing-*']\non_unmatched='warn'",
            request(&["w"]),
            &Registrations::new(),
            temp.path(),
        )
        .unwrap();
        assert_eq!(result.skipped.len(), 2);
        assert!(result.skipped.iter().all(|s| s.reason == "no matches"));
        for config in [
            "[groups]\nw=[]",
            "[groups.w]",
            "[groups.w]\nname_globs=['']",
            "[groups.w]\nunknown=[]",
            "[groups.w]\npath_globs=['**']",
            "[groups.w]\npath_globs=['../other']",
            "[groups.w]\npath_globs=['~other/path']",
            "[groups.w]\nname_globs=['[ab]']",
            "[groups.w]\non_unmatched='ignore'",
        ] {
            assert!(
                plan(config, request(&["w"]), &Registrations::new(), temp.path()).is_err(),
                "{config}"
            );
        }
        assert!(
            plan(
                "[groups]",
                request(&["missing"]),
                &Registrations::new(),
                temp.path()
            )
            .is_err()
        );
    }

    #[test]
    fn symlink_prefixes_are_not_traversed_and_registered_obstructions_are_errors() {
        let temp = tempdir();
        fs::create_dir(temp.path().join("real")).unwrap();
        fs::create_dir(temp.path().join("real/child")).unwrap();
        symlink("real", temp.path().join("alias")).unwrap();
        let config = "[groups.w]\npath_globs=['alias/*']\non_unmatched='warn'";
        let result = plan(config, request(&["w"]), &Registrations::new(), temp.path()).unwrap();
        assert_eq!(result.skipped.len(), 1);
        assert!(result.skipped[0].candidate.ends_with("alias\""));
        assert!(!result.warning().unwrap().contains("child"));
        let registered = BTreeMap::from([("broken".into(), Some(temp.path().join("alias/child")))]);
        let result = plan(config, request(&["w"]), &registered, temp.path()).unwrap();
        assert!(
            result
                .validate()
                .unwrap_err()
                .to_string()
                .contains("not a real directory")
        );
    }

    #[test]
    fn relative_pattern_prefix_is_literal_even_with_glob_characters() {
        let temp = tempdir();
        let base = temp.path().join("literal*prefix");
        fs::create_dir(&base).unwrap();
        fs::create_dir(base.join("target")).unwrap();
        let registered = registrations(&base, &["target"]);
        let result = plan(
            "[groups.w]\npath_globs=['target']",
            request(&["w"]),
            &registered,
            &base,
        )
        .unwrap();
        result.validate().unwrap();
        assert_eq!(result.selections.len(), 1);
        assert!(component_glob("package-?", "package-λ"));
    }
}
