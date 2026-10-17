#!/usr/bin/env python3
"""Retained, isolated prepare/reseed experiments and process-count comparisons."""

import argparse
from collections import Counter
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import traceback

from fixture import payload


SCHEMA_VERSION = 1
PHASES = ("prepare", "reseed-preview", "reseed")
HERE = Path(__file__).resolve().parent
PROJECT = HERE.parent.parent


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def native_binary(value):
    found = shutil.which(value)
    if found is None:
        raise ValueError(f"executable not found: {value}")
    path = Path(found).resolve()
    with path.open("rb") as stream:
        magic = stream.read(4)
    if magic not in (b"\x7fELF", b"\xfe\xed\xfa\xce", b"\xce\xfa\xed\xfe",
                     b"\xfe\xed\xfa\xcf", b"\xcf\xfa\xed\xfe", b"\xca\xfe\xba\xbe",
                     b"\xbe\xba\xfe\xca", b"\xca\xfe\xba\xbf", b"\xbf\xba\xfe\xca"):
        raise ValueError(f"use the actual native executable, not a script/tool-manager shim: {path}")
    return str(path)


def jj_command(argv):
    """Classify command prefixes, skipping JJ global options with their values."""
    valued = {"--repository", "-R", "--at-operation", "--color", "--config",
              "--config-file", "--config-toml"}
    index = 0
    while index < len(argv) and argv[index].startswith("-"):
        arg = argv[index]
        index += 2 if arg in valued else 1
    if index == len(argv):
        return "(global)"
    command = argv[index]
    if command == "diff" and "--git" in argv[index + 1:]:
        return "diff --git"
    if command in {"file", "workspace", "git", "bookmark", "config", "operation", "op", "debug"}:
        if index + 1 < len(argv) and not argv[index + 1].startswith("-"):
            command += " " + argv[index + 1]
    return command


def git_command(argv):
    index = 0
    while index < len(argv) and argv[index].startswith("-"):
        index += 2 if argv[index] in {"-C", "-c", "--git-dir", "--work-tree", "--namespace"} else 1
    return argv[index] if index < len(argv) else "(global)"


def read_calls(path):
    calls = []
    for line in path.read_text().splitlines():
        call = json.loads(line)
        if not isinstance(call, dict) or call.get("tool") not in ("jj", "git") or not isinstance(call.get("argv"), list) or not all(
                isinstance(arg, str) for arg in call["argv"]):
            raise ValueError(f"invalid argv record in {path}")
        calls.append(call)
    return calls


def call_stats(calls):
    grouped = dict(sorted(Counter(jj_command(call["argv"]) for call in calls if call["tool"] == "jj").items()))
    git = dict(sorted(Counter(git_command(call["argv"]) for call in calls if call["tool"] == "git").items()))
    return {"jj_calls": sum(grouped.values()), "file_show_calls": grouped.get("file show", 0),
            "jj_by_command": grouped, "git_calls": sum(git.values()), "git_by_command": git}


def isolated_env(home):
    env = {key: value for key, value in os.environ.items()
           if not key.startswith(("JJ_", "GIT_", "JW_", "XDG_", "PYTHON"))}
    env.update({"HOME": str(home), "XDG_CONFIG_HOME": str(home / "config"),
                "XDG_CACHE_HOME": str(home / "cache"), "XDG_STATE_HOME": str(home / "state"),
                "XDG_DATA_HOME": str(home / "data"), "JJ_CONFIG": "",
                "JJ_USER": "Performance Fixture", "JJ_EMAIL": "performance@example.invalid",
                "JJ_EDITOR": "true", "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null",
                "GIT_AUTHOR_NAME": "Performance Fixture", "GIT_AUTHOR_EMAIL": "performance@example.invalid",
                "GIT_COMMITTER_NAME": "Performance Fixture", "GIT_COMMITTER_EMAIL": "performance@example.invalid",
                "GIT_TERMINAL_PROMPT": "0", "LC_ALL": "C", "TERM": "dumb", "NO_COLOR": "1"})
    return env


class Runner:
    def __init__(self, directory, env, timeout):
        self.directory, self.env, self.timeout = directory, env, timeout
        self.sequence = 0

    def run(self, argv, cwd, env=None):
        self.sequence += 1
        stem = f"commands/{self.sequence:04}"
        stdout, stderr = (self.directory / f"{stem}.{suffix}" for suffix in ("stdout", "stderr"))
        record = {"argv": list(map(str, argv)), "cwd": str(cwd), "stdout": str(stdout.relative_to(self.directory)),
                  "stderr": str(stderr.relative_to(self.directory))}
        start = time.perf_counter()
        with stdout.open("wb") as out, stderr.open("wb") as err:
            process = subprocess.Popen(argv, cwd=cwd, env=env or self.env, stdout=out, stderr=err,
                                       start_new_session=True)
            try:
                record["returncode"] = process.wait(timeout=self.timeout)
            except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                process.wait()
                record.update(returncode=process.returncode,
                              timed_out=isinstance(error, subprocess.TimeoutExpired),
                              interrupted=isinstance(error, KeyboardInterrupt))
        record["seconds"] = time.perf_counter() - start
        with (self.directory / "commands.jsonl").open("a") as log:
            log.write(json.dumps(record) + "\n")
        if record["returncode"]:
            raise RuntimeError(f"command failed ({record['returncode']}): {argv!r}; see {stderr}")
        return stdout.read_text().strip(), record


def filesystem(path):
    info = {"device": path.stat().st_dev, "type": None, "mount": None, "source": None, "options": None}
    mountinfo = Path("/proc/self/mountinfo")
    if mountinfo.exists():
        def unescape(value):
            return re.sub(r"\\([0-7]{3})", lambda match: chr(int(match[1], 8)), value)
        matches = []
        for line in mountinfo.read_text().splitlines():
            fields, tail = line.split(" - ", 1)
            fields, tail = fields.split(), tail.split()
            mount = Path(unescape(fields[4]))
            if path == mount or mount in path.parents:
                matches.append((len(mount.parts), mount, fields, tail))
        if matches:
            _, mount, fields, tail = max(matches, key=lambda entry: entry[0])
            info.update(type=tail[0], mount=str(mount), source=unescape(tail[1]),
                        options=fields[5] + ";" + tail[2])
    return info


def install_shim(directory, python, jj, git):
    script = directory / "process-trace.py"
    script.write_text(
        "import json, os, sys, time\n"
        "tool, argv = sys.argv[1], sys.argv[2:]\n"
        "record = {'tool': tool, 'argv': argv, 'cwd': os.getcwd(), 'pid': os.getpid(), 'time_ns': time.time_ns()}\n"
        "fd = os.open(os.environ['JW_PERF_TRACE'], os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)\n"
        "data = (json.dumps(record) + '\\n').encode()\n"
        "if os.write(fd, data) != len(data): raise RuntimeError('short trace write')\n"
        "os.close(fd)\n"
        f"binary = {dict(jj=jj, git=git)!r}[tool]\n"
        "os.execv(binary, [binary, *argv])\n")
    (directory / "bin").mkdir()
    for tool in ("jj", "git"):
        shim = directory / "bin" / tool
        shim.write_text(f"#!/bin/sh\nexec {shlex.quote(python)} {shlex.quote(str(script))} {tool} \"$@\"\n")
        shim.chmod(0o755)
    return directory / "bin"


def tracked_bytes(index, feature=False):
    return f"{'feature' if feature else 'baseline'} {index}\n".encode()


def run_experiment(args):
    if os.name != "posix":
        raise ValueError("this harness requires POSIX process groups and executable shims")
    output = Path(args.output_dir).resolve()
    output.mkdir(parents=True, exist_ok=True)
    directory = Path(tempfile.mkdtemp(prefix=f"files-{args.files}-", dir=output))
    (directory / "commands").mkdir()
    print(f"Retained experiment: {directory}", file=sys.stderr, flush=True)
    result = {"schema_version": SCHEMA_VERSION, "status": "running", "fixture_path": str(directory),
              "started_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
              "fixture": {"files": args.files, "symlinks": args.symlinks, "workers": args.workers, "payload_files": args.payload_files,
                          "payload_bytes": args.payload_bytes, "feature_files": args.feature_files},
              "prepare_command": args.prepare_command, "timeout_seconds": args.timeout,
              "phases": [], "verification": {}, "command_log": "commands.jsonl"}
    try:
        if directory.stat().st_dev != PROJECT.stat().st_dev:
            raise ValueError("output directory must be on the same filesystem as this checkout")
        if args.feature_files > args.files:
            raise ValueError("--feature-files must not exceed --files")
        jj, jw, git = map(native_binary, (args.jj, args.jw, "git"))
        python = str(Path(sys.executable).resolve())
        home = directory / "home"
        home.mkdir()
        runner = Runner(directory, isolated_env(home), args.timeout)
        result["binary"] = {"path": jw, "sha256": sha256(jw)}
        version, _ = runner.run([jj, "--version"], directory)
        jw_version, _ = runner.run([jw, "--version"], directory)
        git_version, _ = runner.run([git, "--version"], directory)
        for executable, actual, expected in ((jj, version, "jj "), (jw, jw_version, "jw "),
                                             (git, git_version, "git version ")):
            if not actual.startswith(expected):
                raise ValueError(f"{executable} identifies as {actual!r}, expected {expected.strip()!r}; "
                                 "use the actual tool executable, not a tool-manager shim")
        result["binary"]["version"] = jw_version
        result["environment"] = {"jj": {"path": jj, "sha256": sha256(jj), "version": version},
                                 "git": {"path": git, "sha256": sha256(git), "version": git_version},
                                 "python": {"path": python, "version": platform.python_version(), "sha256": sha256(python)},
                                 "platform": platform.platform(), "machine": platform.machine(),
                                 "host": platform.node(), "cpu_count": os.cpu_count(),
                                 "filesystem": filesystem(directory)}
        result["harness_sha256"] = hashlib.sha256(
            Path(__file__).read_bytes() + (HERE / "fixture.py").read_bytes()).hexdigest()
        result["instrumentation"] = "shell -> Python argv logger -> exec native JJ/git; instrumentation included in timings"
        remote, root = directory / "remote", directory / "repo"
        runner.run([git, "init", "--quiet", "--initial-branch=dev", str(remote)], directory)
        (remote / ".gitignore").write_text(".jwreseed.local.toml\ncache/\n")
        (remote / "files").mkdir()
        for index in range(args.files):
            (remote / "files" / f"file-{index:05}").write_bytes(tracked_bytes(index))
        if args.symlinks:
            (remote / "links").mkdir()
            for index in range(args.symlinks):
                (remote / "links" / f"link-{index:05}").symlink_to(f"../files/file-{index % args.files:05}")
        runner.run([git, "add", "."], remote)
        runner.run([git, "commit", "--quiet", "-m", "performance baseline"], remote)
        runner.run([jj, "git", "clone", "--no-colocate", str(remote), str(root)], directory)
        workers = [f"worker-{index + 1}" for index in range(args.workers)]
        feature = None
        for name in workers:
            worker = directory / name
            runner.run([jj, "workspace", "add", "--name", name, "-r", feature or "dev@origin", str(worker)], root)
            if args.feature_files and feature is None:
                for index in range(args.feature_files):
                    (worker / "files" / f"file-{index:05}").write_bytes(tracked_bytes(index, True))
                runner.run([jj, "describe", "-m", "performance feature delta"], worker)
                feature, _ = runner.run([jj, "log", "--no-graph", "-r", "@", "-T", "commit_id"], worker)
        fixture_script = directory / "prepare-fixture.py"
        shutil.copyfile(HERE / "fixture.py", fixture_script)
        (root / ".jwreseed.local.toml").write_text(
            'source_at = "dev@origin"\ncopy_ignored = ["cache"]\n[freshen]\ncommand = '
            + json.dumps([python, str(fixture_script), str(args.payload_files), str(args.payload_bytes)]) + "\n")
        shim = install_shim(directory, python, jj, git)
        measured_env = runner.env | {"PATH": str(shim) + os.pathsep + runner.env.get("PATH", os.defpath)}

        def measure(label, argv):
            trace = directory / f"{label}.processes.jsonl"
            trace.touch()
            phase = {"name": label, "argv": [jw, *argv], "process_log": trace.name}
            result["phases"].append(phase)
            start = time.perf_counter()
            try:
                text, record = runner.run([jw, *argv], root, measured_env | {"JW_PERF_TRACE": str(trace)})
                phase.update(stdout=record["stdout"], stderr=record["stderr"])
                return text
            finally:
                phase["seconds"] = time.perf_counter() - start
                phase.update(call_stats(read_calls(trace)))

        def require(name, condition):
            result["verification"][name] = bool(condition)
            if not condition:
                raise ValueError(f"verification failed: {name}")

        def worker_ids():
            return [runner.run([jj, "log", "--no-graph", "-r", "@", "-T", "commit_id"], directory / name)[0]
                    for name in workers]

        before = worker_ids()
        text = measure("prepare", [args.prepare_command, *(["--format=json"] if args.prepare_command == "prepare" else [])])
        if args.prepare_command == "prepare":
            goal = json.loads(text)["source_commit_id"]
        else:
            match = re.search(r"^source: .* \(([0-9a-f]+)\)$", text, re.MULTILINE)
            if match is None:
                raise ValueError("legacy source-only up output lacks source commit ID")
            goal = match[1]
        if not isinstance(goal, str) or not re.fullmatch(r"[0-9a-f]{40,64}", goal):
            raise ValueError(f"expected full immutable source commit ID, got {goal!r}")
        result["source_commit_id"] = goal
        require("prepare_ran_once", (root / "cache/runs").read_text() == "1")
        require("prepare_kept_targets", worker_ids() == before)
        target = feature or goal
        result["target_commit_id"] = target
        command = ["reseed", *workers, "--source-at", goal, "--at", target]
        measure("reseed-preview", [*command, "--dry-run"])
        require("preview_did_not_prepare", (root / "cache/runs").read_text() == "1")
        require("preview_kept_targets", worker_ids() == before)
        require("preview_did_not_copy_payload", all(not (directory / name / "cache").exists() for name in workers))
        measure("reseed", command)
        require("reseed_did_not_prepare", (root / "cache/runs").read_text() == "1")
        for name in workers:
            worker = directory / name
            diff, _ = runner.run([jj, "diff", "--from", "@", "--to", target, "--summary"], worker)
            require(f"{name}_tree_matches_target", not diff)
            require(f"{name}_tracked_bytes", all(
                (worker / "files" / f"file-{index:05}").read_bytes() == tracked_bytes(index, index < args.feature_files)
                for index in range(args.files)))
            require(f"{name}_symlinks", all(
                os.readlink(worker / "links" / f"link-{index:05}") == f"../files/file-{index % args.files:05}"
                for index in range(args.symlinks)))
            require(f"{name}_payload_counter", (worker / "cache/runs").read_text() == "1")
            require(f"{name}_payload_bytes", all(
                (worker / "cache" / f"group-{index // 100:05}" / f"payload-{index:05}").read_bytes() == payload(index, args.payload_bytes)
                for index in range(args.payload_files)))
            require(f"{name}_payload_file_count", sum(p.is_file() for p in (worker / "cache").rglob("*")) == args.payload_files + 1)
        result["status"] = "passed"
        result["budget_failures"] = budget_failures(result, args)
        if result["budget_failures"]:
            result["status"] = "failed"
    except Exception as error:
        result.update(status="failed", error=f"{type(error).__name__}: {error}")
        (directory / "failure.txt").write_text(traceback.format_exc())
        print(result["error"], file=sys.stderr)
    finally:
        (directory / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(directory / "result.json")
    return 0 if result["status"] == "passed" else 1


def nonnegative(value):
    parsed = int(value)
    if parsed < 0:
        raise argparse.ArgumentTypeError("must be nonnegative")
    return parsed


def positive(value):
    parsed = nonnegative(value)
    if parsed == 0:
        raise argparse.ArgumentTypeError("must be positive")
    return parsed


def budget(value):
    try:
        phase, count = value.split("=")
        if phase not in (*PHASES, "total"):
            raise ValueError()
        return phase, nonnegative(count)
    except (ValueError, argparse.ArgumentTypeError):
        raise argparse.ArgumentTypeError("expected prepare|reseed-preview|reseed|total=NONNEGATIVE_INTEGER") from None


def counts(result, metric):
    values = {phase["name"]: phase[metric] for phase in result["phases"]}
    values["total"] = sum(values.values())
    return values


def budget_failures(result, args):
    failures = []
    for metric, limits in (("jj_calls", args.max_jj_calls), ("file_show_calls", args.max_file_show_calls),
                           ("git_calls", args.max_git_calls)):
        values = counts(result, metric)
        for phase, maximum in limits:
            if values[phase] > maximum:
                failures.append(f"{phase}: {metric} {values[phase]} exceeds {maximum}")
    return failures


def validate_result(result):
    """Reject partial/failed/unknown result formats before computing comparisons."""
    if not isinstance(result, dict) or type(result.get("schema_version")) is not int or result["schema_version"] != SCHEMA_VERSION:
        raise ValueError("unsupported result schema_version")
    if result.get("status") != "passed":
        raise ValueError("comparison/check requires a passed, verified experiment")
    for key in ("environment", "binary", "fixture", "verification"):
        if not isinstance(result.get(key), dict) or not result[key]:
            raise ValueError(f"missing/invalid {key}")
    for key in ("files", "symlinks", "workers", "payload_files", "payload_bytes", "feature_files"):
        value = result["fixture"].get(key)
        if type(value) is not int or value < (1 if key in ("files", "workers") else 0):
            raise ValueError(f"invalid fixture.{key}")
    if result["fixture"]["feature_files"] > result["fixture"]["files"]:
        raise ValueError("feature_files exceeds files")
    for key in ("prepare_ran_once", "prepare_kept_targets", "preview_did_not_prepare", "preview_kept_targets",
                "preview_did_not_copy_payload", "reseed_did_not_prepare"):
        if result["verification"].get(key) is not True:
            raise ValueError(f"missing verification: {key}")
    for index in range(result["fixture"]["workers"]):
        for suffix in ("tree_matches_target", "tracked_bytes", "symlinks", "payload_counter", "payload_bytes", "payload_file_count"):
            if result["verification"].get(f"worker-{index + 1}_{suffix}") is not True:
                raise ValueError(f"missing worker verification: worker-{index + 1}_{suffix}")
    if any(value is not True for value in result["verification"].values()):
        raise ValueError("failed verification")
    for key in ("harness_sha256",):
        if not isinstance(result.get(key), str) or not re.fullmatch(r"[0-9a-f]{64}", result[key]):
            raise ValueError(f"invalid {key}")
    for key in ("path", "version", "sha256"):
        if not isinstance(result["binary"].get(key), str) or not result["binary"][key]:
            raise ValueError(f"missing binary.{key}")
    if not re.fullmatch(r"[0-9a-f]{64}", result["binary"]["sha256"]):
        raise ValueError("invalid binary.sha256")
    if result.get("prepare_command") not in ("prepare", "up"):
        raise ValueError("invalid prepare_command")
    for key in ("jj", "git", "python", "platform", "machine", "host", "cpu_count", "filesystem"):
        if key not in result["environment"]:
            raise ValueError(f"missing environment.{key}")
    for executable in ("jj", "git", "python"):
        item = result["environment"][executable]
        if not isinstance(item, dict) or not all(isinstance(item.get(key), str) and item[key] for key in ("path", "sha256")):
            raise ValueError(f"invalid environment.{executable}")
        if not re.fullmatch(r"[0-9a-f]{64}", item["sha256"]):
            raise ValueError(f"invalid environment.{executable}.sha256")
    for executable in ("jj", "python"):
        if not isinstance(result["environment"][executable].get("version"), str):
            raise ValueError(f"missing environment.{executable}.version")
    fs = result["environment"]["filesystem"]
    if not isinstance(fs, dict) or type(fs.get("device")) is not int or not all(
            key in fs and (fs[key] is None or isinstance(fs[key], str)) for key in ("type", "mount", "source", "options")):
        raise ValueError("invalid environment.filesystem")
    phases = result.get("phases")
    if not isinstance(phases, list) or [phase.get("name") for phase in phases if isinstance(phase, dict)] != list(PHASES):
        raise ValueError("missing, duplicated, or unordered phases")
    for phase in phases:
        seconds = phase.get("seconds")
        if type(seconds) not in (int, float) or not math.isfinite(seconds) or seconds < 0:
            raise ValueError("invalid phase seconds")
        for metric in ("jj_calls", "file_show_calls", "git_calls"):
            if type(phase.get(metric)) is not int or phase[metric] < 0:
                raise ValueError(f"invalid {metric}")
        grouped = phase.get("jj_by_command")
        if not isinstance(grouped, dict) or not all(isinstance(key, str) and type(value) is int and value >= 0 for key, value in grouped.items()):
            raise ValueError("invalid jj_by_command")
        if sum(grouped.values()) != phase["jj_calls"] or grouped.get("file show", 0) != phase["file_show_calls"]:
            raise ValueError("inconsistent JJ process counts")
        grouped_git = phase.get("git_by_command")
        if not isinstance(grouped_git, dict) or not all(isinstance(key, str) and type(value) is int and value >= 0 for key, value in grouped_git.items()):
            raise ValueError("invalid git_by_command")
        if sum(grouped_git.values()) != phase["git_calls"]:
            raise ValueError("inconsistent git process counts")
        if not isinstance(phase.get("process_log"), str):
            raise ValueError("missing raw process log reference")
    return result


def load_result(path):
    return validate_result(json.loads(Path(path).read_text()))


def compatibility(before, after, scaling=False):
    differences = []

    def compare_field(old, new, path):
        if isinstance(old, dict) and isinstance(new, dict):
            for key in sorted(old.keys() | new.keys()):
                compare_field(old.get(key), new.get(key), f"{path}.{key}")
        elif old != new:
            differences.append(f"{path} differs")

    for key in ("harness_sha256", "environment"):
        compare_field(before[key], after[key], key)
    for key in before["fixture"].keys() | after["fixture"].keys():
        if scaling and key == "files":
            continue
        if before["fixture"].get(key) != after["fixture"].get(key):
            differences.append(f"fixture.{key} differs")
    if scaling:
        if after["fixture"]["files"] <= before["fixture"]["files"]:
            differences.append("scaling requires AFTER.files > BEFORE.files")
        if before["binary"]["sha256"] != after["binary"]["sha256"] or before["prepare_command"] != after["prepare_command"]:
            differences.append("scaling requires the same binary and preparation command")
    return differences


def compare(args):
    before, after = load_result(args.before), load_result(args.after)
    differences = compatibility(before, after, args.scaling)
    if differences:
        raise ValueError("incompatible results: " + "; ".join(differences))
    print("Compatible fixture/environment" + (" (tracked-file scaling)" if args.scaling else ""))
    print(f"Preparation: {before['prepare_command']} -> {after['prepare_command']}")
    print("phase             JJ calls (delta)    file show (delta)     git calls (delta)    seconds (delta)")
    failures = []
    for name in (*PHASES, "total"):
        values = []
        for metric in ("jj_calls", "file_show_calls", "git_calls", "seconds"):
            old, new = counts(before, metric)[name], counts(after, metric)[name]
            delta = new - old
            values.append(f"{old:.3f}->{new:.3f} ({delta:+.3f})" if metric == "seconds" else f"{old}->{new} ({delta:+})")
            maximum = {"jj_calls": args.max_jj_growth, "file_show_calls": args.max_file_show_growth,
                       "git_calls": args.max_git_growth}.get(metric)
            if maximum is not None and delta > maximum:
                failures.append(f"{name}: {metric} grew by {delta}, maximum {maximum}")
        print(f"{name:18} {values[0]:19} {values[1]:21} {values[2]:19} {values[3]}")
    if args.scaling:
        extra_files = after["fixture"]["files"] - before["fixture"]["files"]
        growth = counts(after, "jj_calls")["total"] - counts(before, "jj_calls")["total"]
        print(f"Additional tracked files: {extra_files}; additional JJ calls/file: {growth / extra_files:.6f}")
    for failure in failures:
        print(failure, file=sys.stderr)
    return int(bool(failures))


def add_budgets(parser):
    parser.add_argument("--max-jj-calls", action="append", type=budget, default=[], metavar="PHASE=N")
    parser.add_argument("--max-file-show-calls", action="append", type=budget, default=[], metavar="PHASE=N")
    parser.add_argument("--max-git-calls", action="append", type=budget, default=[], metavar="PHASE=N")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    run = commands.add_parser("run", help="retain an isolated synthetic experiment")
    run.add_argument("--jw", required=True, help="actual native jw binary (prefer release)")
    run.add_argument("--jj", required=True, help="actual native JJ binary, not a mise shim")
    run.add_argument("--files", type=positive, default=1000)
    run.add_argument("--symlinks", type=nonnegative, default=0, help="additional tracked relative symlinks")
    run.add_argument("--workers", type=positive, default=2)
    run.add_argument("--payload-files", type=nonnegative, default=100)
    run.add_argument("--payload-bytes", type=nonnegative, default=4096, help="bytes per ignored payload file")
    run.add_argument("--feature-files", type=nonnegative, default=0, help="tracked files modified in a shared target feature commit")
    run.add_argument("--prepare-command", choices=("prepare", "up"), default="prepare", help="up is explicit old-binary measurement compatibility only")
    run.add_argument("--output-dir", default=str(PROJECT / ".test-agent/performance"))
    run.add_argument("--timeout", type=positive, default=180, help="seconds per external command, including descendants on timeout")
    add_budgets(run)
    comparison = commands.add_parser("compare", help="compare verified compatible results")
    comparison.add_argument("before")
    comparison.add_argument("after")
    comparison.add_argument("--scaling", action="store_true", help="permit tracked-file growth, require identical jw binary")
    comparison.add_argument("--max-jj-growth", type=nonnegative)
    comparison.add_argument("--max-file-show-growth", type=nonnegative)
    comparison.add_argument("--max-git-growth", type=nonnegative)
    check = commands.add_parser("check", help="apply deterministic process-count budgets")
    check.add_argument("result")
    add_budgets(check)
    args = parser.parse_args()
    try:
        if args.command == "run":
            return run_experiment(args)
        if args.command == "compare":
            return compare(args)
        failures = budget_failures(load_result(args.result), args)
        print("\n".join(failures) if failures else "Process-count budgets passed")
        return int(bool(failures))
    except (ValueError, OSError, KeyError, TypeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
