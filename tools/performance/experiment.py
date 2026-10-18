#!/usr/bin/env python3
"""Consumer entrypoint for diff-dirs' shared experiment engine (no vendored copy)."""
import os
from pathlib import Path
import sys


def main():
    # An explicit location wins; sibling checkouts are the convenient local default.
    project = Path(__file__).resolve().parents[2]
    engine = Path(os.environ.get("DIFF_DIRS_PERF", project.parent / "diff-dirs/perf")).resolve()
    if not (engine / "experiments/cli.py").is_file():
        raise SystemExit("Shared engine missing: set DIFF_DIRS_PERF=/path/to/diff-dirs/perf")
    sys.path.insert(0, str(engine))
    from experiments.cli import main as shared_main
    return shared_main()


if __name__ == "__main__":
    raise SystemExit(main())
