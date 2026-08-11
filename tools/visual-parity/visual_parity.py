#!/usr/bin/env python3
"""CLI for strict source/Tauri/Freya visual and motion parity evidence."""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path
from typing import Any

TOOL_DIR = Path(__file__).resolve().parent
if str(TOOL_DIR) not in sys.path:
    sys.path.insert(0, str(TOOL_DIR))

from comparison import compare_manifests  # noqa: E402
from metrics import DependencyError  # noqa: E402
from models import Thresholds, ValidationError  # noqa: E402
from reporting import write_artifacts, write_failure_artifacts  # noqa: E402
from validation import load_manifest, validate_manifest  # noqa: E402


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Compare two deterministic Elephant visual/motion manifests")
    subparsers = parser.add_subparsers(dest="command", required=True)
    compare = subparsers.add_parser("compare", help="validate and compare two manifests")
    compare.add_argument("--source", required=True, type=Path, help="source/Tauri manifest")
    compare.add_argument("--freya", required=True, type=Path, help="Freya manifest")
    compare.add_argument("--output", type=Path, help="artifact directory (defaults to /private/tmp/elephant-visual-parity-*)")
    compare.add_argument("--config", type=Path, help="JSON object overriding explicit thresholds")
    validate = subparsers.add_parser("validate", help="validate one manifest without image comparison")
    validate.add_argument("manifest", type=Path)
    defaults = subparsers.add_parser("defaults", help="print strict default thresholds")
    defaults.add_argument("--pretty", action="store_true")
    return parser


def _output_path(value: Path | None) -> Path:
    if value is not None:
        return value.expanduser().resolve()
    return Path(tempfile.mkdtemp(prefix="elephant-visual-parity-"))


def _load_config(path: Path | None) -> Thresholds:
    if path is None:
        return Thresholds()
    try:
        values = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ValidationError(f"cannot read threshold config {path}: {exc}") from exc
    if not isinstance(values, dict):
        raise ValidationError("threshold config must be a JSON object")
    try:
        return Thresholds.from_mapping(values)
    except ValueError as exc:
        raise ValidationError(str(exc)) from exc


def _compare(args: argparse.Namespace) -> int:
    output = _output_path(args.output)
    failures: list[str] = []
    loaded: dict[str, Any] = {}
    for label, path in (("source", args.source), ("freya", args.freya)):
        try:
            loaded[label] = load_manifest(path)
        except ValidationError as exc:
            failures.extend(f"{label}: {issue}" for issue in exc.issues)
    try:
        thresholds = _load_config(args.config)
    except ValidationError as exc:
        failures.extend(exc.issues)
        thresholds = Thresholds()
    if failures:
        write_failure_artifacts(output, failures, {"sourceManifest": str(args.source), "freyaManifest": str(args.freya)})
        print(f"FAILED: {output / 'summary.md'}")
        return 2
    try:
        result = compare_manifests(loaded["source"], loaded["freya"], thresholds)
        write_artifacts(result, output, loaded["source"], loaded["freya"])
    except (DependencyError, OSError, ValueError) as exc:
        write_failure_artifacts(output, [str(exc)], {"sourceManifest": str(args.source), "freyaManifest": str(args.freya)})
        print(f"FAILED: {output / 'summary.md'}")
        return 3 if isinstance(exc, DependencyError) else 2
    print(f"{result['status'].upper()}: {output / 'summary.md'}")
    return 0 if result["status"] == "passed" else 2


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    if args.command == "defaults":
        print(json.dumps(Thresholds().as_dict(), indent=2 if args.pretty else None, sort_keys=True))
        return 0
    if args.command == "validate":
        try:
            model = load_manifest(args.manifest)
        except ValidationError as exc:
            print("FAILED")
            for issue in exc.issues:
                print(f"- {issue}")
            return 2
        print(f"PASSED: {model.path}")
        return 0
    return _compare(args)


if __name__ == "__main__":
    raise SystemExit(main())


__all__ = ["Thresholds", "ValidationError", "main", "validate_manifest"]
