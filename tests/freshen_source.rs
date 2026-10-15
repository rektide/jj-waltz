//! Native source-line selection with disposable local Git remotes. These tests
//! exercise graph/reference semantics only, not fetching any operator repository.
use jj_waltz::freshen::history::{BookmarkNames, TargetHistory};
use jj_waltz::freshen::source::{SelectionKind, SourceSelection};
use jj_waltz::jj::JjClient;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

struct Fixture {
    temp: TempDir,
    remote: PathBuf,
    source: PathBuf,
    base: String,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let remote = temp.path().join("remote");
        let source = temp.path().join("donor with spaces");
        fs::create_dir(&remote).unwrap();
        let mut fixture = Self {
            temp,
            remote,
            source,
            base: String::new(),
        };
        fixture.git(&["init", "--initial-branch=dev"]);
        fs::write(fixture.remote.join("version"), "base\n").unwrap();
        fixture.git(&["add", "."]);
        fixture.git(&["commit", "-m", "published base"]);
        fixture.command(
            "jj",
            fixture.temp.path(),
            &[
                "git",
                "clone",
                "--no-colocate",
                fixture.remote.to_str().unwrap(),
                fixture.source.to_str().unwrap(),
            ],
        );
        fixture.base = fixture.id("dev@origin");
        fixture
    }

    fn command(&self, program: &str, cwd: &Path, args: &[&str]) -> String {
        let mut command = Command::new(program);
        command
            .current_dir(cwd)
            .env("JJ_CONFIG", "")
            .env("JJ_USER", "Source Test")
            .env("JJ_EMAIL", "source@example.invalid")
            .env("JJ_EDITOR", "true")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env(
                "GIT_CONFIG_GLOBAL",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            )
            .env("GIT_AUTHOR_NAME", "Source Test")
            .env("GIT_AUTHOR_EMAIL", "source@example.invalid")
            .env("GIT_COMMITTER_NAME", "Source Test")
            .env("GIT_COMMITTER_EMAIL", "source@example.invalid");
        if program == "jj" {
            command.args([
                "--config=git.colocate=false",
                "--config=fsmonitor.backend=none",
            ]);
        }
        let output = command.args(args).output().unwrap();
        assert!(
            output.status.success(),
            "{program} {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    fn git(&self, args: &[&str]) -> String {
        self.command("git", &self.remote, args)
    }

    fn jj(&self, args: &[&str]) -> String {
        self.command("jj", &self.source, args)
    }

    fn client(&self) -> JjClient {
        JjClient::new(&self.source)
    }

    fn id(&self, revision: &str) -> String {
        self.client().resolve_one(revision).unwrap().commit_id
    }

    fn operation(&self) -> String {
        self.jj(&["status"]);
        self.client().operation_id().unwrap()
    }

    fn select(
        &self,
        explicit: Option<&str>,
        remote: Option<&str>,
    ) -> anyhow::Result<SourceSelection> {
        let operation = self.operation();
        let result = SourceSelection::select(&self.client(), &operation, explicit, remote);
        assert_eq!(
            operation,
            self.client().operation_id().unwrap(),
            "selection must not mutate JJ"
        );
        result
    }

    fn advance(&self, version: &str) -> String {
        fs::write(self.remote.join("version"), version).unwrap();
        self.git(&["add", "."]);
        self.git(&["commit", "-m", version]);
        self.git(&["rev-parse", "HEAD"])
    }

    fn fetch(&self) {
        self.jj(&["git", "fetch", "--remote", "origin"]);
    }

    fn fetched(&self, selection: &SourceSelection) -> anyhow::Result<String> {
        selection.resolve_fetched(&self.client(), &self.client().operation_id().unwrap())
    }
}

#[test]
fn tracked_line_is_frozen_before_fetch_and_survives_journal_serialization() {
    let f = Fixture::new();
    let selected = f.select(None, None).unwrap();
    assert_eq!(selected.kind, SelectionKind::Tracked);
    assert_eq!(
        (&selected.name, &selected.remote, &selected.base),
        (&"dev".to_owned(), &"origin".to_owned(), &f.base)
    );
    let goal = f.advance("upstream advance");
    assert_eq!(
        f.fetched(&selected).unwrap(),
        f.base,
        "server changes do not change the local ref until fetch"
    );
    let frozen: SourceSelection =
        serde_json::from_slice(&serde_json::to_vec(&selected).unwrap()).unwrap();
    f.fetch();
    assert_eq!(f.fetched(&frozen).unwrap(), goal);
    assert_eq!(f.id("@"), selected.source.commit_id);
}

#[test]
fn target_publication_is_compared_to_selected_fetched_goal_not_the_old_remote() {
    let f = Fixture::new();
    let root = f.temp.path().join("worker");
    f.jj(&[
        "workspace",
        "add",
        "--name",
        "worker",
        "-r",
        &f.base,
        root.to_str().unwrap(),
    ]);
    fs::write(root.join("local-work"), "work being published\n").unwrap();
    f.command("jj", &root, &["describe", "-m", "feature"]);
    let target = JjClient::new(&root);
    let work = target.resolve_one("@").unwrap().commit_id;
    f.command("jj", &root, &["new"]);
    let selected = f.select(None, None).unwrap();
    let before = target.operation_id().unwrap();
    let scratch = target.resolve_one_at(&before, "@").unwrap().commit_id;
    let mut names = BookmarkNames::at(&target, &before, jiff::Timestamp::now()).unwrap();
    assert!(
        TargetHistory::plan(&target, &before, &scratch, &f.base, false, &mut names)
            .unwrap()
            .bookmark
            .is_some()
    );
    // Transfer disposable fixture objects directly, not a push to a live remote.
    f.git(&[
        "fetch",
        f.source.join(".jj/repo/store/git").to_str().unwrap(),
        &work,
    ]);
    f.git(&["update-ref", "refs/heads/dev", &work]);
    f.client()
        .fetch_bookmark(&selected.remote, &selected.name)
        .unwrap();
    let fetched = f.fetched(&selected).unwrap();
    assert_eq!(fetched, work);
    let operation = target.operation_id().unwrap();
    let plan =
        TargetHistory::plan(&target, &operation, &scratch, &fetched, false, &mut names).unwrap();
    assert_eq!(plan.work_tip, work);
    assert!(plan.published && plan.bookmark.is_none());
    assert_eq!(target.operation_id().unwrap(), operation);
}

#[test]
fn prior_fetch_uses_the_unique_descendant_not_a_default_branch() {
    let f = Fixture::new();
    let goal = f.advance("already fetched");
    f.fetch();
    let selected = f.select(None, None).unwrap();
    assert_eq!(selected.kind, SelectionKind::Tracked);
    assert_eq!(selected.base, f.base);
    assert_eq!(selected.old_remote.as_deref(), Some(goal.as_str()));
    assert_eq!(f.fetched(&selected).unwrap(), goal);
}

#[test]
fn untracked_exact_remote_ignores_an_unrelated_same_named_local() {
    let f = Fixture::new();
    f.jj(&["bookmark", "untrack", "dev@origin"]);
    f.jj(&["new", "-r", &f.base, "-m", "local dev work"]);
    fs::write(f.source.join("local"), "unpublished").unwrap();
    f.jj(&["bookmark", "set", "dev", "-r", "@"]);
    f.jj(&["new", "-r", &f.base]);
    let local = f.id("dev");
    let selected = f.select(None, None).unwrap();
    assert_eq!(selected.kind, SelectionKind::ExactRemote);
    assert_eq!(selected.tracked, Some(false));
    assert!(selected.old_local.is_none());
    let goal = f.advance("remote only");
    f.fetch();
    assert_eq!(f.fetched(&selected).unwrap(), goal);
    assert_eq!(
        f.id("dev"),
        local,
        "untracked local must not be moved/repaired"
    );
}

#[test]
fn exact_tracked_match_does_not_hide_another_applicable_descendant() {
    let f = Fixture::new();
    f.git(&["branch", "main"]);
    f.fetch();
    f.jj(&["bookmark", "track", "main@origin"]);
    f.advance("dev ahead of main");
    f.fetch();
    let error = f.select(None, None).unwrap_err().to_string();
    assert!(error.contains("ambiguous tracked"), "{error}");
    assert!(error.contains("dev") && error.contains("main"), "{error}");
    let explicit = f.select(Some("dev@origin"), None).unwrap();
    assert_eq!(explicit.kind, SelectionKind::Explicit);
    assert_eq!(explicit.name, "dev");
}

#[test]
fn unrelated_tracked_local_descendant_does_not_relabel_the_source() {
    let f = Fixture::new();
    // The remote side of other predates B. Moving only its local bookmark to a
    // descendant of B is not evidence that this donor follows other@origin.
    f.git(&["branch", "other"]);
    f.advance("source base B");
    f.fetch();
    f.jj(&["bookmark", "track", "other@origin"]);
    f.jj(&["new", "dev@origin", "-m", "unpublished other"]);
    f.jj(&["bookmark", "set", "other", "-r", "@"]);
    f.jj(&["new", "dev@origin"]);
    let selected = f.select(None, None).unwrap();
    assert_eq!(selected.name, "dev");
    assert_eq!(selected.kind, SelectionKind::Tracked);
}

#[test]
fn multiple_exact_untracked_refs_require_an_explicit_identity() {
    let f = Fixture::new();
    f.git(&["branch", "other"]);
    f.fetch();
    f.jj(&["bookmark", "untrack", "dev@origin"]);
    let error = f.select(None, None).unwrap_err().to_string();
    assert!(error.contains("ambiguous remote bookmarks"), "{error}");
    assert_eq!(f.select(Some("other@origin"), None).unwrap().name, "other");
}

#[test]
fn snapshot_and_history_both_guard_against_source_work() {
    let f = Fixture::new();
    fs::write(f.source.join("local"), "work").unwrap();
    assert!(
        f.select(Some("dev@origin"), None)
            .unwrap_err()
            .to_string()
            .contains("unpublished work")
    );
    f.jj(&["new", "-m", "intentional empty"]);
    // An empty @ above unpublished history must not be treated as clean.
    assert!(f.select(None, None).is_err());
    f.jj(&["new"]);
    assert!(
        f.select(Some("dev@origin"), None)
            .unwrap_err()
            .to_string()
            .contains("unpublished work")
    );
}

#[test]
fn meaningful_empty_and_merge_sources_are_not_disposable_scratch() {
    let f = Fixture::new();
    f.jj(&["new", "-r", &f.base, "-m", "intentional empty"]);
    let left = f.id("@");
    assert!(
        f.select(Some("dev@origin"), None)
            .unwrap_err()
            .to_string()
            .contains("unpublished work")
    );
    f.jj(&["new", "-r", &f.base, "-m", "other empty"]);
    let right = f.id("@");
    f.jj(&["new", &left, &right]);
    assert!(
        f.client()
            .commit_facts_at(&f.operation(), "@")
            .unwrap()
            .empty
    );
    assert!(
        f.select(Some("dev@origin"), None)
            .unwrap_err()
            .to_string()
            .contains("unpublished work")
    );
}

#[test]
fn direct_at_published_nonempty_source_is_not_mistaken_for_local_work() {
    let f = Fixture::new();
    fs::write(f.source.join("published-file"), "published work").unwrap();
    f.jj(&["describe", "-m", "published current commit"]);
    f.jj(&["bookmark", "set", "dev", "-r", "@"]);
    let published = f.id("@");
    // Transfer only disposable fixture objects; no push or live-repo mutation.
    f.jj(&["git", "export"]);
    f.git(&[
        "fetch",
        f.source.join(".jj/repo/store/git").to_str().unwrap(),
        &published,
    ]);
    f.git(&["update-ref", "refs/heads/dev", &published]);
    f.jj(&[
        r#"--config=revset-aliases."immutable_heads()"="none()""#,
        "--ignore-working-copy",
        "git",
        "fetch",
        "--remote",
        "origin",
    ]);
    let operation = f.client().operation_id().unwrap();
    assert_eq!(f.id("@"), published);
    let selected = SourceSelection::select(&f.client(), &operation, None, None).unwrap();
    assert_eq!(selected.base, published);
    assert!(!selected.source.empty);
    assert_eq!(f.fetched(&selected).unwrap(), published);
    let goal = f.advance("next upstream");
    f.fetch();
    assert_ne!(
        f.id("@"),
        published,
        "native JJ normalizes an immutable checkout"
    );
    assert_eq!(f.fetched(&selected).unwrap(), goal);
}

#[test]
fn applicable_unpublished_local_bookmark_blocks_instead_of_falling_back() {
    let f = Fixture::new();
    f.jj(&["new", "-r", &f.base, "-m", "local branch work"]);
    fs::write(f.source.join("local"), "unpublished").unwrap();
    f.jj(&["bookmark", "set", "dev", "-r", "@"]);
    f.jj(&["new", "-r", &f.base]);
    let error = f.select(None, None).unwrap_err().to_string();
    assert!(
        error.contains("tracked local bookmark contains unpublished work"),
        "{error}"
    );
}

#[test]
fn conflicted_selected_local_is_not_filtered_out_of_the_tracking_tier() {
    let f = Fixture::new();
    f.jj(&["new", "-r", &f.base, "-m", "local branch work"]);
    fs::write(f.source.join("local"), "unpublished").unwrap();
    f.jj(&["bookmark", "set", "dev", "-r", "@"]);
    f.jj(&["new", "-r", &f.base]);
    f.advance("remote divergence");
    f.fetch();
    let error = f.select(None, None).unwrap_err().to_string();
    assert!(error.contains("conflicted or deleted"), "{error}");
}

#[test]
fn rewind_is_rejected_even_when_both_source_and_local_lagged_the_old_remote() {
    let f = Fixture::new();
    let old_remote = f.advance("old remote ahead");
    f.fetch();
    f.jj(&["bookmark", "set", "dev", "-r", &f.base, "--allow-backwards"]);
    let selected = f.select(Some("dev@origin"), None).unwrap();
    assert_eq!(selected.old_remote.as_deref(), Some(old_remote.as_str()));
    assert_eq!(selected.old_local.as_deref(), Some(f.base.as_str()));
    f.git(&["update-ref", "refs/heads/dev", &f.base]);
    f.fetch();
    let error = f.fetched(&selected).unwrap_err().to_string();
    assert!(error.contains("did not fast-forward"), "{error}");
}

#[test]
fn post_fetch_deletion_and_source_change_do_not_trigger_reselection() {
    let f = Fixture::new();
    let selected = f.select(None, None).unwrap();
    f.git(&["update-ref", "-d", "refs/heads/dev"]);
    f.fetch();
    assert!(f.fetched(&selected).is_err());

    let f = Fixture::new();
    let selected = f.select(None, None).unwrap();
    f.jj(&["describe", "-m", "source changed"]);
    let error = f.fetched(&selected).unwrap_err().to_string();
    assert!(error.contains("source changed during fetch"), "{error}");
}

#[test]
fn remote_constraint_cannot_silently_redirect_an_explicit_line() {
    let f = Fixture::new();
    f.jj(&["git", "remote", "add", "other", f.remote.to_str().unwrap()]);
    f.jj(&["git", "fetch", "--remote", "other"]);
    let error = f
        .select(Some("dev@other"), Some("origin"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("different remote"), "{error}");
    assert_eq!(
        f.select(Some("dev@other"), Some("other")).unwrap().remote,
        "other"
    );
    f.jj(&["bookmark", "track", "dev@other"]);
    assert_eq!(
        f.select(Some("dev"), Some("origin")).unwrap().remote,
        "origin"
    );
}

#[test]
fn missing_ref_preview_freezes_unknown_ids_then_fetches_only_the_requested_branch() {
    let f = Fixture::new();
    let goal = f.advance("new branch tip");
    f.git(&["branch", "introduced-later"]);
    f.advance("unrequested dev advance");
    let selected = f.select(Some("introduced-later@origin"), None).unwrap();
    assert_eq!(selected.kind, SelectionKind::Explicit);
    assert_eq!(selected.name, "introduced-later");
    assert_eq!(selected.remote, "origin");
    let json = serde_json::to_value(&selected).unwrap();
    assert!(json["old_remote"].is_null());
    assert!(json["old_goal"].is_null());
    assert!(json["tracked"].is_null());
    assert_eq!(f.id("dev@origin"), f.base, "preview must not fetch");
    assert!(
        f.fetched(&selected)
            .unwrap_err()
            .to_string()
            .contains("still absent after fetch")
    );
    let frozen: SourceSelection = serde_json::from_value(json).unwrap();
    f.client()
        .fetch_bookmark(&frozen.remote, &frozen.name)
        .unwrap();
    assert_eq!(f.fetched(&frozen).unwrap(), goal);
    assert_eq!(
        f.id("dev@origin"),
        f.base,
        "fetch must not advance an unrequested branch"
    );
    assert_eq!(
        f.id("@"),
        frozen.source.commit_id,
        "resolver must not advance the source"
    );
}

#[test]
fn missing_ref_requires_the_supported_literal_form_and_a_configured_remote() {
    let f = Fixture::new();
    for expression in [
        "introduced-later",
        "\"introduced-later\"@origin",
        "introduced-later@origin | another@origin",
        "introduced-later@origin-",
        "introduced-later@git",
    ] {
        let error = f.select(Some(expression), None).unwrap_err().to_string();
        assert!(
            error.contains("missing refs require a simple unquoted --source-at"),
            "{expression}: {error}"
        );
    }
    let error = f
        .select(Some("introduced-later@unknown"), None)
        .unwrap_err()
        .to_string();
    assert!(error.contains("unconfigured remote"), "{error}");
    let error = f
        .select(Some("introduced-later@origin"), Some("other"))
        .unwrap_err()
        .to_string();
    assert!(error.contains("different remote"), "{error}");
    // Invalid native syntax is not reinterpreted as a branch to fetch.
    assert!(f.select(Some("introduced-later@origin("), None).is_err());
    assert_eq!(f.id("dev@origin"), f.base);
}

#[test]
fn missing_ref_that_stays_absent_does_not_advance_or_invent_a_goal() {
    let f = Fixture::new();
    let selected = f
        .select(Some("does-not-exist@origin"), Some("origin"))
        .unwrap();
    f.client()
        .fetch_bookmark(&selected.remote, &selected.name)
        .unwrap();
    let error = f.fetched(&selected).unwrap_err().to_string();
    assert!(error.contains("still absent after fetch"), "{error}");
    assert!(selected.old_remote.is_none() && selected.old_goal.is_none());
    assert_eq!(f.id("@"), selected.source.commit_id);
}

#[test]
fn missing_ref_fetch_cannot_discard_dirty_source_or_unpublished_parent_history() {
    let f = Fixture::new();
    f.git(&["branch", "future-dirty"]);
    f.git(&["branch", "future-empty"]);
    fs::write(f.source.join("local-work"), "keep this work").unwrap();
    for name in ["future-dirty", "future-empty"] {
        let selected = f.select(Some(&format!("{name}@origin")), None).unwrap();
        assert!(selected.old_goal.is_none());
        f.client()
            .fetch_bookmark(&selected.remote, &selected.name)
            .unwrap();
        let error = f.fetched(&selected).unwrap_err().to_string();
        assert!(error.contains("source base is not published"), "{error}");
        assert_eq!(f.id("@"), selected.source.commit_id);
        assert_eq!(
            fs::read_to_string(f.source.join("local-work")).unwrap(),
            "keep this work"
        );
        // The second pass has an ordinary empty @ above the same local work.
        f.jj(&["new"]);
    }
}

#[test]
fn missing_ref_can_establish_that_previously_unimported_source_history_was_published() {
    let f = Fixture::new();
    fs::write(
        f.source.join("published-later"),
        "work published on new branch",
    )
    .unwrap();
    f.jj(&["describe", "-m", "published on absent branch"]);
    let published = f.id("@");
    f.jj(&["new"]);
    f.jj(&["git", "export"]);
    f.git(&[
        "fetch",
        f.source.join(".jj/repo/store/git").to_str().unwrap(),
        &published,
    ]);
    f.git(&["update-ref", "refs/heads/introduced-later", &published]);
    let selected = f.select(Some("introduced-later@origin"), None).unwrap();
    assert_eq!(selected.base, published);
    assert!(selected.source.ordinary_empty_child());
    assert!(
        !f.client()
            .is_ancestor_at(&selected.operation, &selected.base, &f.base)
            .unwrap()
    );
    f.client()
        .fetch_bookmark(&selected.remote, &selected.name)
        .unwrap();
    assert_eq!(f.fetched(&selected).unwrap(), published);
    assert_eq!(f.id("@"), selected.source.commit_id);
}

#[test]
fn missing_ref_native_auto_tracking_cannot_hide_local_bookmark_work() {
    let f = Fixture::new();
    f.jj(&[
        "config",
        "set",
        "--repo",
        "remotes.origin.auto-track-bookmarks",
        "glob:*",
    ]);
    f.jj(&["new", "-r", &f.base, "-m", "unpublished future branch"]);
    fs::write(f.source.join("local-work"), "unpublished future").unwrap();
    f.jj(&["status"]);
    let local = f.id("@");
    f.jj(&["new", "-r", &f.base]);
    f.advance("remote future");
    f.git(&["branch", "future"]);
    let selected = f.select(Some("future@origin"), None).unwrap();
    assert!(selected.tracked.is_none());
    // A local bookmark appears after intent capture. Native auto-tracking on
    // import must not turn that unpublished history into permission to advance.
    f.jj(&["bookmark", "create", "future", "-r", &local]);
    let recorded = f
        .select(Some("future@origin"), None)
        .unwrap_err()
        .to_string();
    assert!(
        recorded.contains("already recorded but unresolved or deleted"),
        "{recorded}"
    );
    f.client()
        .fetch_bookmark(&selected.remote, &selected.name)
        .unwrap();
    let operation = f.client().operation_id().unwrap();
    assert!(
        f.client()
            .bookmark_refs_at(&operation)
            .unwrap()
            .iter()
            .any(|r| r.name == "future" && r.remote.as_deref() == Some("origin") && r.tracked)
    );
    let error = f.fetched(&selected).unwrap_err().to_string();
    assert!(
        error.contains("conflicted or deleted") || error.contains("unpublished work after fetch"),
        "{error}"
    );
    assert_eq!(f.id("@"), selected.source.commit_id);
}
