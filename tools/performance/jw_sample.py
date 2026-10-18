#!/usr/bin/env python3
"""Isolated single-operation reseed adapter. Setup/verification are separate commands.

No live JJ donors, fetches, preparation/install commands, or existing targets.
Use only state directories created by the shared engine's private reset.
"""
import argparse
import csv
import json
import os
from pathlib import Path
import subprocess
import sys
import time

from measure import isolated_env, native_binary, sha256


POLICY = ["node_modules", "**/node_modules", "**/dist"]


def write(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def inode_inventory(root):
    result = {}
    for directory, dirs, files in os.walk(root, followlinks=False):
        dirs[:] = [name for name in dirs if name not in (".git", ".jj")]
        for name in files:
            path = Path(directory) / name
            if not any(part in ("node_modules", "dist") for part in path.relative_to(root).parts):
                continue
            if path.is_symlink():
                continue
            info = path.stat()
            result[str(path.relative_to(root))] = [info.st_ino, info.st_size, info.st_mtime_ns]
    return result


def context(state):
    info = json.loads((state / "jw-case.json").read_text())
    if info["state"] != str(state) or not (state / "OWNED-JW-CASE").is_file():
        raise ValueError("adapter requires its original owned state path")
    env = isolated_env(state / "home")
    env["PATH"] = str(state / "bin") + os.pathsep + env.get("PATH", os.defpath)
    return info, env


def run_logged(argv, cwd, env, state):
    index = len(list(state.glob("adapter-*.json")))
    stem = state / f"adapter-{index:04}"
    start = time.perf_counter()
    # communicate's pipe wait avoids timeout polling quantization in the measured native subprocess.
    result = subprocess.run(argv, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    seconds = time.perf_counter() - start
    stem.with_suffix(".stdout").write_bytes(result.stdout)
    stem.with_suffix(".stderr").write_bytes(result.stderr)
    write(stem.with_suffix(".json"), {"argv": argv, "cwd": str(cwd), "seconds": seconds,
                                     "returncode": result.returncode})
    if result.returncode:
        raise ValueError(f"native command failed ({result.returncode}): see {stem}.stderr")
    return result.stdout.decode().strip(), seconds


def setup(args):
    state = Path(args.state).resolve()
    # Engine reset log is in the state parent; do not accept arbitrary source directories.
    reset = state.parent / "reset.json"
    if not reset.is_file() or not (state / "source").is_dir() or (state / "source/.jj").exists():
        raise ValueError("setup requires a fresh engine-created private reset")
    jj, git, jw = map(native_binary, (args.jj, args.git, args.jw))
    (state / "OWNED-JW-CASE").write_text("private synthetic/snapshot-derived JJ state\n")
    (state / "home").mkdir()
    (state / "bin").mkdir()
    for name, binary in (("jj", jj), ("git", git)):
        (state / "bin" / name).symlink_to(binary)
    source = state / "source"
    # NEVER execute donor-local freshen actions. Override local config before initializing JJ.
    (source / ".jwreseed.local.toml").write_text('copy_ignored = ["node_modules", "**/node_modules", "**/dist"]\n')
    ignore = source / ".gitignore"
    original = ignore.read_text() if ignore.exists() else ""
    ignore.write_text(original + "\nnode_modules/\ndist/\n.jwreseed.local.toml\n")
    info = {"state": str(state), "jw": {"path": jw, "sha256": sha256(jw)},
            "jj": {"path": jj, "sha256": sha256(jj)}, "git": {"path": git, "sha256": sha256(git)},
            "destinations": args.destinations, "policy": POLICY, "tracked_feature_files": args.feature_files}
    write(state / "jw-case.json", info)
    _, env = context(state)
    execute = lambda argv, cwd=source: run_logged(argv, cwd, env, state)[0]
    execute([jj, "git", "init", "--no-colocate", str(source)], state)
    execute([jj, "describe", "-m", "owned performance source"])
    goal = execute([jj, "log", "--no-graph", "-r", "@", "-T", "commit_id"])
    execute([jj, "bookmark", "create", "performance-source", "-r", goal])
    info.update(goal=goal, workers=[], targets={}, payload_inodes_before={})
    # Only ignored selections from reset targets are overlaid. Tracked files come from JJ's tree.
    for index in range(args.destinations):
        worker = state / f"worker-{index}"
        name = f"worker-{index}"
        execute([jj, "workspace", "add", "--name", name, "-r", goal, str(worker)])
        target = state / f"target-{index}"
        for directory, dirs, _ in os.walk(target, followlinks=False):
            dirs[:] = [entry for entry in sorted(dirs) if entry not in (".jj", ".git")]
            for entry in list(dirs):
                if entry in ("node_modules", "dist"):
                    old = Path(directory) / entry
                    new = worker / old.relative_to(target)
                    new.parent.mkdir(parents=True, exist_ok=True)
                    execute(["cp", "-a", "--reflink=always", "--", str(old), str(new)], state)
                    dirs.remove(entry)
        if args.feature_files:
            listed = execute([jj, "file", "list"], worker).splitlines()
            eligible = [worker / value for value in listed if value != ".gitignore" and
                        (worker / value).is_file() and not (worker / value).is_symlink()]
            for path in eligible[:args.feature_files]:
                with path.open("ab") as stream:
                    stream.write(f"\nfeature {index}\n".encode())
            execute([jj, "describe", "-m", f"owned feature {index}"], worker)
            execute([jj, "bookmark", "create", f"performance-feature-{index}", "-r", "@"], worker)
        info["workers"].append(name)
        info["targets"][name] = execute([jj, "log", "--no-graph", "-r", "@", "-T", "commit_id"], worker)
        info["payload_inodes_before"][name] = inode_inventory(worker)
    command = [jw, "reseed", *info["workers"], "--source-at", goal, "--from", str(source)]
    # Without --at, distinct feature targets preserve their own revisions.
    if not args.feature_files:
        command.extend(["--at", goal])
    info["command"] = command
    write(state / "jw-case.json", info)
    print(state / "jw-case.json")


def measure(args):
    state = Path(args.state).resolve()
    info, env = context(state)
    for tool in ("jw", "jj", "git"):
        if sha256(info[tool]["path"]) != info[tool]["sha256"]:
            raise ValueError(f"{tool} executable changed after setup")
    _, seconds = run_logged(info["command"], state / "source", env, state)
    writer = csv.writer(sys.stdout)
    writer.writerow(["phase", "seconds"])
    writer.writerow(["reseed-native-process", seconds])


def verify(args):
    state = Path(args.state).resolve()
    info, env = context(state)
    engine = Path(args.engine).resolve()
    sys.path.insert(0, str(engine))
    from experiments.verify_tree import inventory
    expected = inventory(state / "source")
    changes = {}
    for name in info["workers"]:
        worker = state / name
        actual = inventory(worker)
        # Tracked feature trees intentionally differ; compare ignored payload independently.
        def ignored(tree):
            return {path: value for path, value in tree.items()
                    if any(part in ("node_modules", "dist") for part in Path(path).parts)}
        if ignored(actual) != ignored(expected):
            raise ValueError(f"payload content/link/mode parity failed: {name}")
        diff, _ = run_logged([info["jj"]["path"], "diff", "--from", info["targets"][name], "--to", "@", "--summary"], worker, env, state)
        if diff:
            raise ValueError(f"tracked working copy differs from JJ target: {name}: {diff}")
        before, after = info["payload_inodes_before"][name], inode_inventory(worker)
        same = before.keys() & after.keys()
        changes[name] = {"common_files": len(same), "inode_changes": sum(before[p][0] != after[p][0] for p in same),
                         "mtime_changes": sum(before[p][2] != after[p][2] for p in same),
                         "added": len(after.keys() - before.keys()), "removed": len(before.keys() - after.keys())}
    write(state / "inode-changes.json", changes)
    print("payload byte/link/mode parity and tracked target checks passed; inode changes retained separately")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["setup", "run", "verify"])
    parser.add_argument("state")
    parser.add_argument("--jw")
    parser.add_argument("--jj", default="/usr/local/bin/jj")
    parser.add_argument("--git", default="/usr/bin/git")
    parser.add_argument("--destinations", type=int, default=1)
    parser.add_argument("--feature-files", type=int, default=0)
    parser.add_argument("--engine", default=str(Path(__file__).resolve().parents[3] / "diff-dirs/perf"))
    args = parser.parse_args()
    if args.destinations < 1 or args.feature_files < 0:
        parser.error("destinations positive; feature-files nonnegative")
    {"setup": setup, "run": measure, "verify": verify}[args.action](args)


if __name__ == "__main__":
    main()
