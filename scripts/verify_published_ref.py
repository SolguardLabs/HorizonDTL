#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--expected-sha", required=True)
    args = parser.parse_args()

    refs = {
        "main": git("rev-parse", "origin/main^{commit}"),
        "production": git("rev-parse", "origin/production^{commit}"),
        "v1.0.0": git("rev-parse", "refs/tags/v1.0.0^{commit}"),
    }
    tag_type = git("cat-file", "-t", "refs/tags/v1.0.0")
    expected = args.expected_sha.lower()
    mismatches = {name: sha for name, sha in refs.items() if sha.lower() != expected}
    if tag_type != "tag":
        print(f"ERROR: v1.0.0 must be annotated, observed {tag_type}", file=sys.stderr)
        return 1
    if mismatches:
        print(
            f"ERROR: published references differ from {expected}: {mismatches}",
            file=sys.stderr,
        )
        return 1
    print(f"published references converge at {expected}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
