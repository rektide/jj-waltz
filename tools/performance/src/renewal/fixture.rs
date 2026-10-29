use super::Case;
use crate::{context::Context, instrumentation};
use diff_dirs_perf_runner::{
    Result,
    artifacts::{self, Control},
    write_json,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    fs::FileTimes,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const MARKER: &str = "jw-perf renewal owned fixture v1\n";
const FILES: [&str; 7] = [".gitignore", "steady", "changed", "source-only", "collapse/leaf", "expand", "mode"];
const PAYLOAD: [&str; 3] = ["Q/alpha", "Q/sub/beta", "Q/link"];
const FIXED_MTIME: u64 = 1_700_000_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Node {
    kind: String,
    mode: u32,
    bytes: u64,
    content: String,
    mtime: (i64, i64),
    dev: u64,
    ino: u64,
}
impl Node {
    fn signature(&self, times: bool) -> Value {
        json!({"kind":self.kind,"mode":self.mode,"bytes":self.bytes,"content":self.content,
            "mtime":if times { Some(self.mtime) } else { None }})
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(super) struct Binding {
    root: PathBuf,
    root_id: (u64, u64),
    jj_id: (u64, u64),
    repo: PathBuf,
    repo_id: (u64, u64),
    pointer: Option<String>,
}
impl Binding {
    fn read(root: &Path) -> Result<Self> {
        let identity = |p: &Path| -> Result<(u64, u64)> {
            let m = fs::symlink_metadata(p)?;
            if !m.is_dir() {
                return Err("owned native root/administration must remain real directories".into());
            }
            Ok((m.dev(), m.ino()))
        };
        let root = fs::canonicalize(root)?;
        let jj = root.join(".jj");
        let ptr = jj.join("repo");
        let pointer = if fs::symlink_metadata(&ptr)?.is_file() {
            Some(fs::read_to_string(&ptr)?)
        } else {
            None
        };
        let repo = if let Some(pointer) = &pointer {
            fs::canonicalize(jj.join(pointer.trim()))?
        } else {
            fs::canonicalize(ptr)?
        };
        Ok(Self {
            root_id: identity(&root)?,
            jj_id: identity(&jj)?,
            repo_id: identity(&repo)?,
            root,
            repo,
            pointer,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Fixture {
    pub root: PathBuf,
    pub control: Control,
    pub jj: PathBuf,
    pub git: PathBuf,
    tools: Value,
    pub s: String,
    d: String,
    old: [String; 2],
    bindings: Vec<Binding>,
    donor: BTreeMap<PathBuf, Node>,
    source_tree: Value,
    desired_tree: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Start {
    pub signature: Value,
    old: Vec<String>,
    changes: Vec<String>,
    bookmarks: Vec<String>,
    ordinary: Vec<BTreeMap<PathBuf, Node>>,
}

pub(super) fn exact(id: &str) -> Result<String> {
    if ![40, 64].contains(&id.len()) || !id.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("full exact native commit ID required".into());
    }
    Ok(format!("commit_id(\"{id}\")"))
}

impl Fixture {
    pub fn source(&self) -> PathBuf {
        self.root.join("trees/source")
    }
    fn worker(&self, index: usize) -> PathBuf {
        self.root.join(if index == 0 { "trees/worker-a" } else { "trees/worker-b" })
    }
    pub fn desired(&self, case: Case) -> &str {
        if case == Case::DesiredN2 { &self.d } else { &self.s }
    }

    pub fn create(root: &Path, control: Control, ctx: &mut Context) -> Result<Self> {
        let started = std::time::Instant::now();
        fs::create_dir(root.join("trees"))?;
        fs::write(root.join(".owned-renewal"), MARKER)?;
        let source = root.join("trees/source");
        ctx.jj(root, &["git", "init", "--no-colocate", utf8(&source)?])?;
        ctx.root_check(&source)?;
        fs::create_dir(source.join("collapse"))?;
        fs::write(source.join(".gitignore"), "Q/\nvanished/\n.env\n.jwreseed.local.toml\n")?;
        for path in &FILES[1..] {
            fs::write(source.join(path), format!("S {path}\n"))?;
        }
        fs::set_permissions(source.join("mode"), fs::Permissions::from_mode(0o755))?;
        std::os::unix::fs::symlink("steady", source.join("link"))?;
        stable_times(&source)?;
        ctx.jj(&source, &["describe", "-m", "owned pinned S"])?;
        let s = coordinate(ctx, &source, "@", "commit_id")?;
        ctx.jj(&source, &["bookmark", "create", "perf-source", "-r", &exact(&s)?])?;
        fs::write(source.join(".jwreseed.local.toml"), "copy_ignored = [\"Q\", \"vanished\"]\n")?;
        make_payload(&source)?;
        stable_times(&source)?;
        for name in ["worker-a", "worker-b"] {
            let path = root.join("trees").join(name);
            ctx.jj(&source, &["workspace", "add", "--no-colocate", "--name", name, "-r", &exact(&s)?, utf8(&path)?])?;
            ctx.root_check(&path)?;
        }
        // Build D IN the already-owned A root. Never create a third receiver.
        let a = root.join("trees/worker-a");
        ctx.jj(&a, &["new", &exact(&s)?, "-m", "owned D distinct from S"])?;
        fs::write(a.join("changed"), "D modified\n")?;
        fs::remove_file(a.join("source-only"))?;
        fs::write(a.join("desired-only"), "D addition\n")?;
        fs::remove_file(a.join("collapse/leaf"))?;
        fs::remove_dir(a.join("collapse"))?;
        fs::write(a.join("collapse"), "D collapse\n")?;
        fs::remove_file(a.join("expand"))?;
        fs::create_dir(a.join("expand"))?;
        fs::write(a.join("expand/child"), "D expansion\n")?;
        fs::remove_file(a.join("link"))?;
        std::os::unix::fs::symlink("desired-only", a.join("link"))?;
        fs::set_permissions(a.join("mode"), fs::Permissions::from_mode(0o644))?;
        ctx.jj(&a, &["status"])?;
        let d = coordinate(ctx, &a, "@", "commit_id")?;
        ctx.jj(&a, &["bookmark", "create", "perf-desired", "-r", &exact(&d)?])?;
        let desired_tree = signature(&inventory(&a)?, false);
        let mut old = [String::new(), String::new()];
        for (index, pin) in old.iter_mut().enumerate() {
            let worker = root.join("trees").join(if index == 0 { "worker-a" } else { "worker-b" });
            ctx.jj(&worker, &["new", &exact(&s)?, "-m", &format!("owned old lane {index}")])?;
            fs::write(worker.join("changed"), format!("O lane {index}\n"))?;
            ctx.jj(&worker, &["status"])?;
            *pin = coordinate(ctx, &worker, "@", "commit_id")?;
            ctx.jj(&worker, &["bookmark", "create", &format!("perf-old-{index}"), "-r", &exact(pin)?])?;
            fs::write(worker.join(".env"), format!("retained local lane {index}\n"))?;
            fs::create_dir(worker.join("vanished"))?;
            fs::write(worker.join("vanished/keep"), "source root vanished; not deletion authority\n")?;
        }
        let bindings = [source.clone(), root.join("trees/worker-a"), root.join("trees/worker-b")]
            .iter()
            .map(|p| Binding::read(p))
            .collect::<Result<Vec<_>>>()?;
        let donor = inventory(&source)?;
        let source_tree = tracked_signature(&donor);
        let f = Self {
            root: root.into(),
            control,
            jj: ctx.jj.clone(),
            git: ctx.git.clone(),
            tools: json!({"jj":instrumentation::identity(&ctx.jj)?,"git":instrumentation::identity(&ctx.git)?,
                "jj_version":ctx.jj(root, &["--version"])?}),
            s,
            d,
            old,
            bindings,
            donor,
            source_tree,
            desired_tree,
        };
        f.verify(ctx)?;
        write_json(
            &root.join("setup/fixture-setup.json"),
            &json!({"elapsed_ns":started.elapsed().as_nanos(),"budget":f.budget()?,
            "tracked_regular_source":FILES.len(),"scope":"native setup only, no public renewal or performance"}),
        )?;
        Ok(f)
    }

    pub fn open(root: &Path) -> Result<Self> {
        let root = fs::canonicalize(root)?;
        if fs::read_to_string(root.join(".owned-renewal"))? != MARKER {
            return Err("not an owned renewal fixture".into());
        }
        let f: Self = serde_json::from_slice(&fs::read(root.join("fixture.json"))?)?;
        if f.root != root {
            return Err("fixture moved; native roots are bound to their original locations".into());
        }
        super::check_control(&f.control)?;
        f.verify_bindings()?;
        Ok(f)
    }

    pub fn verify_bindings(&self) -> Result<()> {
        if self.bindings.len() != 3 {
            return Err("exactly one donor/two receiver roots required".into());
        }
        for (binding, path) in self.bindings.iter().zip([self.source(), self.worker(0), self.worker(1)]) {
            if *binding != Binding::read(&path)? {
                return Err("native root/.jj/repository identity changed".into());
            }
        }
        let names = fs::read_dir(self.root.join("trees"))?
            .map(|e| Ok(e?.file_name().to_string_lossy().into_owned()))
            .collect::<Result<Vec<_>>>()?;
        if names.len() != 3 || names.iter().any(|n| !["source", "worker-a", "worker-b"].contains(&n.as_str())) {
            return Err("unexpected additional ordinary fixture root".into());
        }
        Ok(())
    }
    pub fn verify(&self, ctx: &mut Context) -> Result<()> {
        self.verify_bindings()?;
        super::check_control(&self.control)?;
        if self.tools["jj"] != instrumentation::identity(&ctx.jj)? || self.tools["git"] != instrumentation::identity(&ctx.git)? {
            return Err("native tools changed after fixture creation".into());
        }
        if self.donor != inventory(&self.source())? || coordinate(ctx, &self.source(), "@", "commit_id")? != self.s {
            return Err("owned pinned donor changed".into());
        }
        for (bookmark, pin) in [
            ("perf-source", &self.s),
            ("perf-desired", &self.d),
            ("perf-old-0", &self.old[0]),
            ("perf-old-1", &self.old[1]),
        ] {
            if coordinate(ctx, &self.source(), bookmark, "commit_id")? != *pin {
                return Err("pinned native history changed".into());
            }
        }
        let registrations = ctx.jj(
            &self.source(),
            &["--at-operation", "@", "workspace", "list", "-T", "name ++ \"\\t\" ++ root ++ \"\\n\""],
        )?;
        for (name, binding) in ["default", "worker-a", "worker-b"].iter().zip(&self.bindings) {
            if !registrations
                .lines()
                .any(|l| l.split_once('\t').is_some_and(|(n, p)| n == *name && p == binding.root.to_string_lossy()))
            {
                return Err("native registration/name/root binding changed".into());
            }
        }
        for path in [self.source(), self.worker(0), self.worker(1)] {
            ctx.root_check(&path)?;
            if ctx.jj(&path, &["--at-operation", "@", "sparse", "list"])? != "." {
                return Err("fixture interrupted/sparse; recover deliberately, no automatic native reconciliation".into());
            }
        }
        self.budget()?;
        Ok(())
    }

    pub fn budget(&self) -> Result<Value> {
        let roots = [self.source(), self.worker(0), self.worker(1)];
        let counts = roots.iter().map(|p| inventory(p).map(|v| usage(&v))).collect::<Result<Vec<_>>>()?;
        // Reserve TWO entire source copies, not just its selected payload. This
        // conservatively covers simultaneous source-leaf/payload staging in N2.
        let (reserve, total) = aggregate_bound(&counts)?;
        Ok(json!({"live":counts,"staging_reserve":reserve,"aggregate_bound":total,"ceiling":[256,1048576],
            "scope":"all ordinary checkout entries including dirs/config/links; excludes native administration and artifacts outside trees"}))
    }

    pub fn reset(&self, ctx: &mut Context, case: Case) -> Result<Start> {
        self.verify(ctx)?;
        let mut start = Start {
            signature: Value::Null,
            old: Vec::new(),
            changes: Vec::new(),
            bookmarks: Vec::new(),
            ordinary: Vec::new(),
        };
        let mut signatures = Vec::new();
        // Both roots are reset even for N1; no unmeasured third destination and
        // no case creates or imports another donor. Only named Q deltas are removed.
        for index in 0..2 {
            let worker = self.worker(index);
            clear_payload(&worker)?;
            let pin = if case == Case::CleanN1 { &self.s } else { &self.old[index] };
            ctx.jj(&worker, &["new", &exact(pin)?, "-m", "owned matched renewal start"])?;
            if !(case == Case::MixedN2 && index == 0) {
                clone_payload(ctx, &self.source(), &worker)?;
                if case != Case::CleanN1 {
                    if index == 0 {
                        fs::write(worker.join("Q/sub/beta"), "small changed A\n")?;
                    } else {
                        fs::write(worker.join("Q/alpha"), "small changed B\n")?;
                    }
                    fs::write(worker.join(if index == 0 { "Q/stale-a" } else { "Q/stale-b" }), "stale selected leaf\n")?;
                }
            }
            stable_times(&worker)?;
            // Named preservation snapshot happens OUTSIDE the measured command.
            ctx.jj(&worker, &["status"])?;
            let old = coordinate(ctx, &worker, "@", "commit_id")?;
            let bookmark = format!("perf-saved-{index}-{old}");
            ctx.jj(&worker, &["bookmark", "create", &bookmark, "-r", &exact(&old)?])?;
            if index < case.receivers() {
                start.old.push(old);
                start.changes.push(coordinate(ctx, &worker, "@", "change_id")?);
                start.bookmarks.push(bookmark);
                let now = inventory(&worker)?;
                start.ordinary.push(payload_nodes(&now));
                signatures.push(signature(&now, true));
            }
        }
        start.signature = json!({"case":case,"S":self.s,"D":self.desired(case),"lanes":signatures});
        self.budget()?;
        Ok(start)
    }

    pub fn verify_result(&self, ctx: &mut Context, case: Case, start: &Start) -> Result<Value> {
        self.verify(ctx)?;
        let mut lanes = Vec::new();
        for index in 0..case.receivers() {
            let worker = self.worker(index);
            let desired = self.desired(case);
            if !ctx
                .jj(
                    &worker,
                    &["--config", "snapshot.auto-track=none()", "diff", "--from", &exact(desired)?, "--summary"],
                )?
                .is_empty()
            {
                return Err("live tracked receiver tree differs from exact D".into());
            }
            let facts = coordinate(
                ctx,
                &worker,
                "@",
                "parents.map(|p| p.commit_id()).join(\"\\n\") ++ \"\\n\" ++ empty ++ \"\\n\" ++ conflict",
            )?;
            if facts != format!("{desired}\ntrue\nfalse") || coordinate(ctx, &worker, "@", "change_id")? == start.changes[index] {
                return Err("receiver is not its own new conflict-free empty single-parent child of D".into());
            }
            let now = inventory(&worker)?;
            let expected_tree = if case == Case::DesiredN2 { &self.desired_tree } else { &self.source_tree };
            if tracked_signature(&now) != *expected_tree {
                return Err("physical D files/types/literal links/executable mode differ".into());
            }
            let payload = payload_nodes(&now);
            if signature(&payload, true) != signature(&payload_nodes(&self.donor), true) {
                return Err("ordinary Q did not converge (including stale removal)".into());
            }
            if fs::read_to_string(worker.join(".env"))? != format!("retained local lane {index}\n")
                || fs::read_to_string(worker.join("vanished/keep"))? != "source root vanished; not deletion authority\n"
            {
                return Err("unrelated local or vanished source root was erased".into());
            }
            if case == Case::CleanN1 && payload != start.ordinary[index] {
                return Err("clean ordinary Q inode/mtime/content changed".into());
            }
            if coordinate(ctx, &worker, &start.bookmarks[index], "commit_id")? != start.old[index] {
                return Err("preserved O bookmark/history moved".into());
            }
            // Private objects, NOT extent sharing. Equality/inode churn do not
            // prove required renewal; actual successful clone counts gate separately.
            for (path, node) in &now {
                if node.kind == "file"
                    && let Some(source) = self.donor.get(path)
                    && (node.dev, node.ino) == (source.dev, source.ino)
                {
                    return Err("receiver regular file aliases donor inode".into());
                }
            }
            let before = fs::read(self.source().join("steady"))?;
            let receiver = fs::read(worker.join("steady"))?;
            fs::write(worker.join("steady"), "bounded private-write probe\n")?;
            let isolated = fs::read(self.source().join("steady"))? == before;
            fs::write(worker.join("steady"), receiver)?;
            if !isolated {
                return Err("receiver write changed donor".into());
            }
            lanes.push(json!({"root":worker,"old":start.old[index],"parent":desired,"private_write":true,
                "final_D_sharing":"not claimed; D-only/changed/native-restored leaves may come from native store"}));
        }
        Ok(
            json!({"lanes":lanes,"budget":self.budget()?,"source_install_regular_obligations_per_receiver":FILES.len(),
            "storage_scope":"inode/write isolation only; no extent-range measurement or successful clone inference"}),
        )
    }

    #[cfg(test)]
    pub(super) fn fake(root: &Path, control: Control) -> Self {
        Self {
            root: root.into(),
            control,
            jj: "/usr/bin/true".into(),
            git: "/usr/bin/true".into(),
            tools: Value::Null,
            s: "a".repeat(40),
            d: "b".repeat(40),
            old: ["c".repeat(40), "d".repeat(40)],
            bindings: Vec::new(),
            donor: BTreeMap::new(),
            source_tree: Value::Null,
            desired_tree: Value::Null,
        }
    }
}

fn utf8(path: &Path) -> Result<&str> {
    path.to_str().ok_or_else(|| "fixture path must be UTF-8".into())
}
fn coordinate(ctx: &mut Context, root: &Path, revision: &str, template: &str) -> Result<String> {
    ctx.jj(root, &["--at-operation", "@", "log", "--no-graph", "-r", revision, "-T", template])
}

fn inventory(root: &Path) -> Result<BTreeMap<PathBuf, Node>> {
    fn visit(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, Node>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if dir == root && entry.file_name() == ".jj" {
                continue;
            }
            if entry.file_name() == ".jj" || entry.file_name() == ".git" {
                return Err("nested administration in ordinary fixture refused".into());
            }
            let path = entry.path();
            let m = fs::symlink_metadata(&path)?;
            let (kind, content, bytes) = if m.is_dir() {
                ("dir", String::new(), 0)
            } else if m.is_file() {
                if m.len() > 1_048_576 {
                    return Err("oversized ordinary fixture file".into());
                }
                ("file", artifacts::sha256(&path)?, m.len())
            } else if m.is_symlink() {
                ("link", fs::read_link(&path)?.to_str().ok_or("non-UTF8 fixture link")?.into(), m.len())
            } else {
                return Err("special ordinary fixture entry refused".into());
            };
            out.insert(
                path.strip_prefix(root)?.into(),
                Node {
                    kind: kind.into(),
                    content,
                    bytes,
                    mode: m.mode() & 0o777,
                    mtime: if m.is_file() { (m.mtime(), m.mtime_nsec()) } else { (0, 0) },
                    dev: m.dev(),
                    ino: m.ino(),
                },
            );
            enforce_budget(usage(out))?;
            if m.is_dir() {
                visit(root, &path, out)?;
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out)?;
    Ok(out)
}
fn usage(nodes: &BTreeMap<PathBuf, Node>) -> (u64, u64) {
    (nodes.len() as u64, nodes.values().map(|n| n.bytes).sum())
}
fn enforce_budget((entries, bytes): (u64, u64)) -> Result<()> {
    if entries > 256 || bytes > 1_048_576 {
        return Err("aggregate ordinary fixture exceeds 256 entries / 1 MiB including staging reserve".into());
    }
    Ok(())
}
fn aggregate_bound(counts: &[(u64, u64)]) -> Result<((u64, u64), (u64, u64))> {
    if counts.len() != 3 {
        return Err("budget requires one donor and exactly two reused receivers".into());
    }
    for &count in counts {
        enforce_budget(count)?;
    }
    let reserve = (counts[0].0 * 2, counts[0].1 * 2);
    let total = counts.iter().fold((reserve.0 + 3, reserve.1), |a, b| (a.0 + b.0, a.1 + b.1));
    enforce_budget(total)?;
    Ok((reserve, total))
}
fn signature(nodes: &BTreeMap<PathBuf, Node>, times: bool) -> Value {
    json!(nodes.iter().map(|(p, n)| (p.clone(), n.signature(times))).collect::<BTreeMap<_, _>>())
}
fn tracked_signature(nodes: &BTreeMap<PathBuf, Node>) -> Value {
    signature(
        &nodes
            .iter()
            .filter(|(p, _)| !p.starts_with("Q") && !p.starts_with("vanished") && **p != Path::new(".env") && **p != Path::new(".jwreseed.local.toml"))
            .map(|(p, n)| (p.clone(), n.clone()))
            .collect(),
        false,
    )
}
fn payload_nodes(nodes: &BTreeMap<PathBuf, Node>) -> BTreeMap<PathBuf, Node> {
    nodes.iter().filter(|(p, _)| p.starts_with("Q")).map(|(p, n)| (p.clone(), n.clone())).collect()
}
fn make_payload(root: &Path) -> Result<()> {
    fs::create_dir(root.join("Q"))?;
    fs::create_dir(root.join("Q/sub"))?;
    fs::write(root.join("Q/alpha"), "tiny source alpha\n")?;
    fs::write(root.join("Q/sub/beta"), "tiny source beta\n")?;
    std::os::unix::fs::symlink("alpha", root.join("Q/link"))?;
    Ok(())
}
fn clear_payload(root: &Path) -> Result<()> {
    // Never remove arbitrary descendants, even in scratch. Unexpected content
    // leaves the root intact and asks the operator to recover deliberately.
    let nodes = inventory(root)?;
    for (path, node) in payload_nodes(&nodes) {
        if !["Q", "Q/sub", "Q/stale-a", "Q/stale-b"].contains(&path.to_str().unwrap_or("")) && !PAYLOAD.iter().any(|p| path == Path::new(p)) {
            return Err("unexpected Q reset entry; refusing cleanup outside named fixture deltas".into());
        }
        let expected = match path.to_str() {
            Some("Q" | "Q/sub") => "dir",
            Some("Q/link") => "link",
            _ => "file",
        };
        if node.kind != expected {
            return Err("named ordinary reset route/leaf changed kind; refusing before deletion".into());
        }
    }
    for path in ["Q/stale-a", "Q/stale-b", "Q/link", "Q/sub/beta", "Q/alpha"] {
        match fs::symlink_metadata(root.join(path)) {
            Ok(m) if m.is_file() || m.is_symlink() => fs::remove_file(root.join(path))?,
            Ok(_) => return Err("named ordinary reset leaf changed kind".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    for path in ["Q/sub", "Q"] {
        match fs::remove_dir(root.join(path)) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
fn clone_payload(ctx: &mut Context, source: &Path, target: &Path) -> Result<()> {
    // Same strict native cp primitive as existing fixture::clone_tree, but with
    // full process/stdout/stderr receipts through Context. No byte-copy fallback.
    ctx.run(
        Path::new("/usr/bin/cp"),
        &[
            "-a".into(),
            "--reflink=always".into(),
            "--".into(),
            utf8(&source.join("Q"))?.into(),
            utf8(&target.join("Q"))?.into(),
        ],
        target,
        None,
    )?;
    Ok(())
}
fn stable_times(root: &Path) -> Result<()> {
    let at: SystemTime = UNIX_EPOCH + Duration::from_secs(FIXED_MTIME);
    for (path, node) in inventory(root)? {
        if node.kind == "file" {
            fs::File::options()
                .write(true)
                .open(root.join(path))?
                .set_times(FileTimes::new().set_modified(at))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_ids_and_staging_budget_fail_closed() {
        assert!(exact("@-").is_err());
        assert!(exact("deadbeef").is_err());
        assert_eq!(exact(&"a".repeat(40)).unwrap(), format!("commit_id(\"{}\")", "a".repeat(40)));
        assert!(enforce_budget((256, 1048576)).is_ok());
        assert!(enforce_budget((257, 1)).is_err());
        assert!(enforce_budget((1, 1048577)).is_err());
        assert_eq!(aggregate_bound(&[(50, 100_000); 3]).unwrap(), ((100, 200_000), (253, 500_000)));
        assert!(
            aggregate_bound(&[(51, 1_000); 3]).is_err(),
            "staging pushes aggregate entries over ceiling although each root fits"
        );
        assert!(
            aggregate_bound(&[(1, 210_000); 3]).is_err(),
            "staging pushes aggregate bytes over ceiling although each root fits"
        );
        assert!(aggregate_bound(&[(1, 1); 4]).is_err(), "no additional fixture root");
    }
    #[test]
    fn named_reset_refuses_unexpected_content_and_never_follows_links() {
        let own = super::super::tests::scratch();
        make_payload(own.path()).unwrap();
        fs::write(own.path().join("Q/foreign"), "retained").unwrap();
        assert!(clear_payload(own.path()).is_err());
        assert!(own.path().join("Q/alpha").is_file(), "preflight refusal before any deletion");
        fs::remove_file(own.path().join("Q/foreign")).unwrap();
        clear_payload(own.path()).unwrap();
        assert!(!own.path().join("Q").exists());
        let external = own.path().join("external");
        fs::create_dir(&external).unwrap();
        fs::write(external.join("keep"), "keep").unwrap();
        fs::write(external.join("alpha"), "must not follow Q to this known leaf name").unwrap();
        std::os::unix::fs::symlink(&external, own.path().join("Q")).unwrap();
        assert!(clear_payload(own.path()).is_err());
        assert_eq!(fs::read(external.join("keep")).unwrap(), b"keep");
        assert_eq!(fs::read(external.join("alpha")).unwrap(), b"must not follow Q to this known leaf name");
    }

    #[test]
    #[ignore = "explicit bounded native fixture setup/reset smoke; does NOT invoke a renewal route"]
    fn one_native_fixture_reuses_all_tiny_shapes_without_running_candidate() {
        let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.test-agent/annealing-evidence");
        fs::create_dir_all(&scratch).unwrap();
        let own = tempfile::tempdir_in(scratch.canonicalize().unwrap()).unwrap();
        let root = own.path();
        let logs = root.join("setup");
        fs::create_dir(&logs).unwrap();
        let c = super::super::tests::control(root, Path::new("/usr/bin/true"));
        let mut ctx = super::super::context(&logs, Path::new("/usr/local/bin/jj"), Path::new("/usr/bin/git")).unwrap();
        let f = Fixture::create(root, c, &mut ctx).unwrap();
        write_json(&root.join("fixture.json"), &f).unwrap();
        let f = Fixture::open(root).unwrap();
        let mut peak = (0, 0);
        let mut starts = Vec::new();
        for case in Case::ALL {
            let a = f.reset(&mut ctx, case).unwrap();
            let b = f.reset(&mut ctx, case).unwrap();
            assert_eq!(a.signature, b.signature, "unmatched {case:?} start");
            if case == Case::MixedN2 {
                assert!(!f.worker(0).join("Q").exists());
                assert!(f.worker(1).join("Q/stale-b").is_file());
            }
            let budget = f.budget().unwrap();
            peak.0 = peak.0.max(budget["aggregate_bound"][0].as_u64().unwrap());
            peak.1 = peak.1.max(budget["aggregate_bound"][1].as_u64().unwrap());
            starts.push(a.signature);
        }
        assert_ne!(f.s, f.d);
        assert_eq!(f.bindings.len(), 3);
        let receipt = scratch.join(format!(
            "adapter-setup-smoke-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        write_json(
            &logs.join("smoke.json"),
            &json!({"passed":true,"peak_including_staging_reserve":peak,"starts":starts,
            "S":f.s,"D":f.d,"tools":f.tools,"scope":"fixture setup/reset only; /usr/bin/true is a test placeholder, never a route result"}),
        )
        .unwrap();
        // Retain every native setup/reset argv/stdout/stderr/process record, but
        // release this test's owned native trees so reruns do not accumulate donors.
        fs::rename(&logs, &receipt).unwrap();
        println!("fixture setup/reset receipt: {}", receipt.display());
    }
}
