#!/usr/bin/env python3
"""Deterministic ignored payload installer used by the performance fixture."""

import hashlib
from pathlib import Path
import sys


def payload(index, size):
    block = hashlib.sha256(f"jw-performance-payload-{index}".encode()).digest()
    return (block * ((size + len(block) - 1) // len(block)))[:size]


def main():
    count, size = map(int, sys.argv[1:])
    root = Path("cache")
    root.mkdir(exist_ok=True)
    counter = root / "runs"
    counter.write_text(str(int(counter.read_text()) + 1 if counter.exists() else 1))
    for index in range(count):
        directory = root / f"group-{index // 100:05}"
        directory.mkdir(exist_ok=True)
        (directory / f"payload-{index:05}").write_bytes(payload(index, size))


if __name__ == "__main__":
    main()
