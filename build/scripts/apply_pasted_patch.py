#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def read_patch_text() -> str:
    if sys.stdin.isatty():
        print("Paste the patch, then press Ctrl-D on a blank line when finished.", file=sys.stderr)
    return sys.stdin.read()


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Read a patch from stdin, write it to a .patch file, and apply it with git apply."
    )
    parser.add_argument(
        "-o",
        "--output",
        default="pasted.patch",
        help="Path to the patch file to write (default: pasted.patch).",
    )
    parser.add_argument(
        "--repo",
        default=".",
        help="Repository root where git apply should run (default: current directory).",
    )
    args = parser.parse_args()

    patch_text = read_patch_text()
    if not patch_text.strip():
        print("No patch text received on stdin.", file=sys.stderr)
        return 1

    repo_root = Path(args.repo).resolve()
    patch_path = Path(args.output)
    if not patch_path.is_absolute():
        patch_path = (repo_root / patch_path).resolve()

    patch_path.write_text(patch_text, encoding="utf-8")
    print(f"Wrote patch to {patch_path}", file=sys.stderr)

    result = subprocess.run(
        ["git", "apply", str(patch_path)],
        cwd=repo_root,
        check=False,
    )

    if result.returncode != 0:
        print("git apply failed.", file=sys.stderr)
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
