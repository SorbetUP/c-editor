"""Manifest contract validation. This module never changes a baseline."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any, Iterable

from models import CheckpointRef, FrameRef, ManifestModel, ValidationError


REQUIRED_VIEWPORT_KEYS = ("width", "height", "scaleFactor", "deviceScaleFactor")
MOTION_WORDS = ("hover", "move", "scroll", "drag", "pan", "zoom", "motion", "animate")


def _walk(value: Any, path: str = "") -> Iterable[tuple[str, Any]]:
    yield path, value
    if isinstance(value, dict):
        for key, child in value.items():
            child_path = f"{path}.{key}" if path else str(key)
            yield from _walk(child, child_path)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from _walk(child, f"{path}[{index}]")


def _as_number(value: Any, label: str, issues: list[str]) -> float | None:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        issues.append(f"{label} must be numeric")
        return None
    return float(value)


def _frame_path(frame: dict[str, Any], artifact_root: Path) -> tuple[Path | None, str]:
    source_path = frame.get("sourcePath")
    relative_path = frame.get("path")
    candidates: list[tuple[Path, str]] = []
    if isinstance(source_path, str) and source_path:
        source = Path(source_path).expanduser()
        candidates.append((source, source_path))
    if isinstance(relative_path, str) and relative_path:
        candidates.append((artifact_root / relative_path, relative_path))
    for candidate, display in candidates:
        if candidate.is_file():
            return candidate.resolve(), display
    if candidates:
        return candidates[0][0], candidates[0][1]
    return None, "<missing path>"


def _frames_for_checkpoint(
    checkpoint: dict[str, Any],
    top_level_frames: dict[str, list[dict[str, Any]]],
) -> list[dict[str, Any]]:
    frames = checkpoint.get("frames")
    if isinstance(frames, list):
        return [frame for frame in frames if isinstance(frame, dict)]
    return top_level_frames.get(str(checkpoint.get("id")), [])


def _top_level_frame_index(raw: dict[str, Any], issues: list[str]) -> dict[str, list[dict[str, Any]]]:
    grouped: dict[str, list[dict[str, Any]]] = {}
    frames = raw.get("frames", [])
    if frames is None:
        return grouped
    if not isinstance(frames, list):
        issues.append("frames must be an array")
        return grouped
    for index, frame in enumerate(frames):
        if not isinstance(frame, dict):
            issues.append(f"frames[{index}] must be an object")
            continue
        checkpoint_id = frame.get("checkpoint")
        if not isinstance(checkpoint_id, str) or not checkpoint_id:
            issues.append(f"frames[{index}] is missing checkpoint")
            continue
        grouped.setdefault(checkpoint_id, []).append(frame)
    return grouped


def _raw_hash(path: Path) -> str | None:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError:
        return None


def _motion_checkpoint(action_id: str) -> bool:
    lowered = action_id.lower()
    return any(word in lowered for word in MOTION_WORDS)


def validate_manifest(raw: dict[str, Any], path: str | Path = "<memory>") -> ManifestModel:
    """Validate semantic evidence and resolve every referenced frame on disk."""

    issues: list[str] = []
    manifest_path = Path(path).expanduser().resolve() if path != "<memory>" else Path("<memory>")
    if not isinstance(raw, dict):
        raise ValidationError("manifest root must be an object")
    if raw.get("schemaVersion") != 1:
        issues.append("schemaVersion must be 1")
    if raw.get("status") != "passed":
        issues.append(f"manifest status must be passed, got {raw.get('status')!r}")
    validation = raw.get("validation")
    if not isinstance(validation, dict):
        issues.append("validation object is missing")
    else:
        for key in ("allActionsExecuted", "allTargetsPlaywrightResolved", "stateAndVaultHashesCaptured"):
            if validation.get(key) is not True:
                issues.append(f"validation.{key} must be true")
    viewport = raw.get("viewport")
    if not isinstance(viewport, dict):
        issues.append("viewport object is missing")
        viewport = {}
    for key in REQUIRED_VIEWPORT_KEYS:
        if key not in viewport:
            issues.append(f"viewport.{key} is missing")
        else:
            _as_number(viewport[key], f"viewport.{key}", issues)
    for key in ("width", "height"):
        if isinstance(viewport.get(key), (int, float)) and viewport[key] <= 0:
            issues.append(f"viewport.{key} must be positive")

    actions_raw = raw.get("actions")
    if not isinstance(actions_raw, list) or not actions_raw:
        issues.append("actions must be a non-empty array")
        actions_raw = []
    action_ids: list[str] = []
    for index, action in enumerate(actions_raw):
        if not isinstance(action, dict):
            issues.append(f"actions[{index}] must be an object")
            continue
        action_id = action.get("id")
        if not isinstance(action_id, str) or not action_id:
            issues.append(f"actions[{index}].id is missing")
            continue
        if action_id in action_ids:
            issues.append(f"duplicate action id: {action_id}")
        action_ids.append(action_id)
        if action.get("status") != "passed":
            issues.append(f"action {action_id} status must be passed")

    for location, value in _walk(raw):
        if location.endswith("mustChange") and value is False:
            issues.append(f"{location} explicitly declares mustChange:false")

    provenance = raw.get("provenance")
    artifact_root_value = provenance.get("artifactRoot") if isinstance(provenance, dict) else None
    artifact_root = Path(artifact_root_value).expanduser() if isinstance(artifact_root_value, str) else Path()
    if not artifact_root.is_absolute() and manifest_path != Path("<memory>"):
        artifact_root = manifest_path.parent / artifact_root
    artifact_root = artifact_root.resolve()

    top_level_frames = _top_level_frame_index(raw, issues)
    checkpoints_raw = raw.get("checkpoints")
    if not isinstance(checkpoints_raw, list) or not checkpoints_raw:
        issues.append("checkpoints must be a non-empty array")
        checkpoints_raw = []
    checkpoints: list[CheckpointRef] = []
    seen_checkpoints: set[str] = set()
    previous_rail: list[Any] | None = None
    action_id_set = set(action_ids)
    for index, checkpoint in enumerate(checkpoints_raw):
        if not isinstance(checkpoint, dict):
            issues.append(f"checkpoints[{index}] must be an object")
            continue
        checkpoint_id = checkpoint.get("id")
        after_action = checkpoint.get("afterAction")
        if not isinstance(checkpoint_id, str) or not checkpoint_id:
            issues.append(f"checkpoints[{index}].id is missing")
            continue
        if checkpoint_id in seen_checkpoints:
            issues.append(f"duplicate checkpoint id: {checkpoint_id}")
        seen_checkpoints.add(checkpoint_id)
        if not isinstance(after_action, str) or after_action not in action_id_set:
            issues.append(f"checkpoint {checkpoint_id} references unknown afterAction {after_action!r}")
            after_action = str(after_action or "")
        state = checkpoint.get("state")
        if not isinstance(state, dict):
            issues.append(f"checkpoint {checkpoint_id} state must be an object")
            state = {}
        raw_frames = _frames_for_checkpoint(checkpoint, top_level_frames)
        if not raw_frames:
            issues.append(f"checkpoint {checkpoint_id} has no frames")
        frames: list[FrameRef] = []
        for frame_index, frame in enumerate(raw_frames):
            relative_ms = frame.get("relativeMs", frame_index)
            if isinstance(relative_ms, bool) or not isinstance(relative_ms, (int, float)):
                issues.append(f"checkpoint {checkpoint_id} frame {frame_index} relativeMs must be numeric")
                relative_ms = frame_index
            resolved, display_path = _frame_path(frame, artifact_root)
            if resolved is None or not resolved.is_file():
                issues.append(f"checkpoint {checkpoint_id} frame {frame_index} missing: {display_path}")
                resolved = Path(display_path)
            expected_hash = frame.get("sha256")
            if resolved.is_file() and isinstance(expected_hash, str):
                actual_hash = _raw_hash(resolved)
                if actual_hash != expected_hash:
                    issues.append(f"checkpoint {checkpoint_id} frame {frame_index} sha256 mismatch")
            frames.append(FrameRef(frame_index, float(relative_ms), resolved, display_path, frame))
        checkpoint_ref = CheckpointRef(checkpoint_id, after_action, state, checkpoint, tuple(frames))
        checkpoints.append(checkpoint_ref)

        lowered_action = after_action.lower()
        scroll = state.get("scroll")
        if "scroll" in lowered_action:
            if not isinstance(scroll, dict):
                issues.append(f"scroll action {after_action} has no scroll state")
            elif scroll.get("mustChange") is not True:
                issues.append(f"scroll action {after_action} must prove scroll.mustChange:true")
            elif len({hashlib.sha256(frame.path.read_bytes()).hexdigest() for frame in frames if frame.path.is_file()}) < 2:
                issues.append(f"scroll action {after_action} has no changing frame")

        rail = state.get("railOrder")
        if "drag" in lowered_action:
            if not isinstance(rail, list):
                issues.append(f"drag action {after_action} has no railOrder")
            elif previous_rail is None:
                issues.append(f"drag action {after_action} has no prior railOrder")
            elif rail == previous_rail:
                issues.append(f"drag action {after_action} left railOrder unchanged")
        if isinstance(rail, list):
            previous_rail = rail

        if _motion_checkpoint(after_action):
            if len(frames) < 2:
                issues.append(f"motion action {after_action} needs at least two timeline frames")
            else:
                hashes = {hashlib.sha256(frame.path.read_bytes()).hexdigest() for frame in frames if frame.path.is_file()}
                if len(hashes) < 2:
                    issues.append(f"motion action {after_action} has identical frames")

    if issues:
        raise ValidationError(issues)
    return ManifestModel(raw, manifest_path, dict(viewport), tuple(actions_raw), tuple(checkpoints), artifact_root)


def load_manifest(path: str | Path) -> ManifestModel:
    manifest_path = Path(path).expanduser().resolve()
    try:
        raw = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ValidationError(f"cannot read manifest {manifest_path}: {exc}") from exc
    return validate_manifest(raw, manifest_path)


__all__ = ["load_manifest", "validate_manifest", "ValidationError"]
