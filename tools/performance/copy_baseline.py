#!/usr/bin/env python3
"""Measure GNU cp's strict-reflink baseline on a retained, verified fixture."""

import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

from measure import load_result, sha256


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("result", type=Path, help="successful measure.py result.json")
    args = parser.parse_args()
    evidence = args.result.resolve()
    source = load_result(evidence)
    fixture = Path(source["fixture_path"]).resolve()
    if fixture != evidence.parent:
        raise ValueError("run against the original retained fixture result")
    root = fixture / "repo"
    destination = Path(tempfile.mkdtemp(prefix="copy-baseline-", dir=fixture))
    cp = shutil.which("cp")
    if cp is None:
        raise ValueError("GNU cp with --reflink=always is required")
    paths = [root / name for name in (".gitignore", "files", "links", "cache")
             if (root / name).exists()]
    command = [cp, "-a", "--reflink=always", "--", *map(str, paths), str(destination)]
    started = time.perf_counter()
    process = subprocess.run(command, capture_output=True, text=True, timeout=180)
    seconds = time.perf_counter() - started
    expected = (source["fixture"]["files"] + source["fixture"]["symlinks"]
                + source["fixture"]["payload_files"] + 2)
    count = sum(path.is_symlink() or path.is_file() for path in destination.rglob("*"))
    first = Path("files/file-00000")
    private = ((root / first).stat().st_dev, (root / first).stat().st_ino) != (
        (destination / first).stat().st_dev, (destination / first).stat().st_ino
    ) if process.returncode == 0 else False
    result = {"schema_version": 1, "method": "gnu-cp-strict-reflink-baseline",
              "source_result": str(evidence), "source_result_sha256": sha256(evidence),
              "fixture": source["fixture"], "destination": str(destination),
              "argv": command, "seconds": seconds, "returncode": process.returncode,
              "stderr": process.stderr, "copied_files_and_symlinks": count,
              "expected_files_and_symlinks": expected, "first_tracked_file_private": private,
              "scope": "fresh copy of tracked baseline and payload; excludes JJ metadata, feature application, clearing, and verification"}
    result["status"] = "passed" if process.returncode == 0 and count == expected and private else "failed"
    result_path = fixture / f"{destination.name}.json"
    result_path.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    print(f"Evidence: {result_path}")
    return int(result["status"] != "passed")


if __name__ == "__main__":
    raise SystemExit(main())
