"""Pair manifests by semantic ids and compare every aligned frame."""

from __future__ import annotations

from typing import Any

from metrics import (
    align_frames,
    geometry_metrics,
    image_metrics,
    load_image,
    motion_profile,
    region_metrics,
    visual_pass,
)
from models import ManifestModel, Thresholds, ValidationError


MOTION_WORDS = ("hover", "move", "scroll", "drag", "pan", "zoom", "motion", "animate")


def _is_motion(action_id: str) -> bool:
    lowered = action_id.lower()
    return any(word in lowered for word in MOTION_WORDS)


def _viewport_diff(left: dict[str, Any], right: dict[str, Any]) -> list[str]:
    differences = []
    for key in ("width", "height", "scaleFactor", "deviceScaleFactor"):
        if left.get(key) != right.get(key):
            differences.append(f"viewport.{key}: {left.get(key)!r} != {right.get(key)!r}")
    return differences


def _curve_value(profile: dict[str, Any], timestamp: float) -> float:
    curve = profile.get("motionEnergyCurve", [])
    if not curve:
        return 0.0
    if timestamp <= curve[0]["fromMs"]:
        return float(curve[0]["energy"])
    for interval in curve:
        if interval["fromMs"] <= timestamp <= interval["toMs"]:
            return float(interval["energy"])
    return float(curve[-1]["energy"])


def _motion_comparison(left_frames, right_frames, thresholds: Thresholds) -> dict[str, Any]:
    left = motion_profile(left_frames)
    right = motion_profile(right_frames)
    left_change = left["firstChangeMs"]
    right_change = right["firstChangeMs"]
    timestamps = sorted({frame.relative_ms for frame in left_frames} | {frame.relative_ms for frame in right_frames})
    curve_pairs = [
        {
            "timestampMs": timestamp,
            "leftEnergy": _curve_value(left, timestamp),
            "rightEnergy": _curve_value(right, timestamp),
        }
        for timestamp in timestamps
    ]
    curve_mae = (
        sum(abs(pair["leftEnergy"] - pair["rightEnergy"]) for pair in curve_pairs) / len(curve_pairs)
        if curve_pairs
        else 0.0
    )
    failures: list[str] = []
    if left["distinctHashCount"] < 2:
        failures.append("source motion frames are identical")
    if right["distinctHashCount"] < 2:
        failures.append("freya motion frames are identical")
    if left_change is None or right_change is None:
        failures.append("motion has no observable frame change")
    elif abs(left_change - right_change) > thresholds.max_change_moment_delta_ms:
        failures.append(
            f"first change differs by {abs(left_change - right_change):.3f}ms "
            f"> {thresholds.max_change_moment_delta_ms:.3f}ms"
        )
    if abs(left["distinctHashCount"] - right["distinctHashCount"]) > thresholds.max_distinct_hash_delta:
        failures.append("distinct motion hash counts differ beyond threshold")
    if curve_mae > thresholds.max_motion_curve_mae:
        failures.append(f"motion energy curve MAE {curve_mae:.6f} exceeds {thresholds.max_motion_curve_mae:.6f}")
    return {
        "source": left,
        "freya": right,
        "curvePairs": curve_pairs,
        "curveMae": curve_mae,
        "passed": not failures,
        "failures": failures,
    }


def compare_manifests(source: ManifestModel, freya: ManifestModel, thresholds: Thresholds) -> dict[str, Any]:
    """Return a complete raw result. Validation failures are never downgraded."""

    failures = _viewport_diff(source.viewport, freya.viewport)
    source_actions = set(source.action_ids)
    freya_actions = set(freya.action_ids)
    for action_id in sorted(source_actions - freya_actions):
        failures.append(f"action missing in Freya: {action_id}")
    for action_id in sorted(freya_actions - source_actions):
        failures.append(f"action missing in source: {action_id}")
    source_checkpoints = source.checkpoint_map
    freya_checkpoints = freya.checkpoint_map
    checkpoint_ids = sorted(set(source_checkpoints) | set(freya_checkpoints))
    for checkpoint_id in sorted(set(source_checkpoints) - set(freya_checkpoints)):
        failures.append(f"checkpoint missing in Freya: {checkpoint_id}")
    for checkpoint_id in sorted(set(freya_checkpoints) - set(source_checkpoints)):
        failures.append(f"checkpoint missing in source: {checkpoint_id}")

    checkpoint_results: list[dict[str, Any]] = []
    expected_width = source.viewport.get("width")
    expected_height = source.viewport.get("height")
    for checkpoint_id in checkpoint_ids:
        if checkpoint_id not in source_checkpoints or checkpoint_id not in freya_checkpoints:
            continue
        left_checkpoint = source_checkpoints[checkpoint_id]
        right_checkpoint = freya_checkpoints[checkpoint_id]
        checkpoint_failures: list[str] = []
        if left_checkpoint.after_action != right_checkpoint.after_action:
            checkpoint_failures.append(
                f"afterAction differs: {left_checkpoint.after_action!r} != {right_checkpoint.after_action!r}"
            )
        alignment = align_frames(left_checkpoint.frames, right_checkpoint.frames, thresholds.timestamp_tolerance_ms)
        if not alignment["alignable"]:
            checkpoint_failures.append(alignment["reason"] or "frame timelines are not alignable")
        frame_results: list[dict[str, Any]] = []
        worst: tuple[float, Any, Any] | None = None
        for pair in alignment["pairs"]:
            left_frame = left_checkpoint.frames[pair["leftIndex"]]
            right_frame = right_checkpoint.frames[pair["rightIndex"]]
            frame_result: dict[str, Any] = {
                "sourceIndex": left_frame.index,
                "freyaIndex": right_frame.index,
                "sourceMs": left_frame.relative_ms,
                "freyaMs": right_frame.relative_ms,
                "timestampDeltaMs": pair["deltaMs"],
            }
            try:
                left_image, left_array = load_image(left_frame.path)
                right_image, right_array = load_image(right_frame.path)
                left_size = left_image.size
                right_size = right_image.size
                frame_result["dimensions"] = {
                    "source": {"width": left_size[0], "height": left_size[1]},
                    "freya": {"width": right_size[0], "height": right_size[1]},
                }
                if (left_size[0], left_size[1]) != (expected_width, expected_height):
                    checkpoint_failures.append(
                        f"source frame {left_frame.index} dimensions {left_size} != viewport {(expected_width, expected_height)}"
                    )
                if (right_size[0], right_size[1]) != (expected_width, expected_height):
                    checkpoint_failures.append(
                        f"Freya frame {right_frame.index} dimensions {right_size} != viewport {(expected_width, expected_height)}"
                    )
                if left_size != right_size:
                    checkpoint_failures.append(f"frame dimensions differ: {left_size} != {right_size}")
                metrics, delta, per_pixel = image_metrics(left_array, right_array, thresholds)
                frame_result["metrics"] = metrics
                frame_result["passed"] = visual_pass(metrics, thresholds)
                if not frame_result["passed"]:
                    checkpoint_failures.append(f"frame {left_frame.index}->{right_frame.index} visual thresholds not met")
                score = (1.0 - metrics["ssim"]) + metrics["mae"] / 255.0 + metrics["ratio_pixels_over_structure_threshold"]
                if worst is None or score > worst[0]:
                    worst = (score, delta, per_pixel)
                regions = region_metrics(left_checkpoint.state, right_checkpoint.state, left_array, right_array, thresholds)
                if regions is not None:
                    frame_result["regions"] = regions
                    if not all(region.get("passed") is True for region in regions):
                        checkpoint_failures.append("one or more declared regions failed")
            except (OSError, ValueError) as exc:
                frame_result["passed"] = False
                frame_result["error"] = str(exc)
                checkpoint_failures.append(str(exc))
            frame_results.append(frame_result)

        geometry = geometry_metrics(left_checkpoint.state, right_checkpoint.state, thresholds)
        if geometry is not None and not geometry["passed"]:
            checkpoint_failures.append("declared geometry differs beyond threshold")
        motion = None
        if _is_motion(left_checkpoint.after_action):
            try:
                motion = _motion_comparison(left_checkpoint.frames, right_checkpoint.frames, thresholds)
                if not motion["passed"]:
                    checkpoint_failures.extend(motion["failures"])
            except (OSError, ValueError) as exc:
                checkpoint_failures.append(f"motion analysis failed: {exc}")
        checkpoint_results.append(
            {
                "id": checkpoint_id,
                "afterAction": left_checkpoint.after_action,
                "alignment": alignment,
                "frames": frame_results,
                "geometry": geometry,
                "motion": motion,
                "passed": not checkpoint_failures,
                "failures": checkpoint_failures,
                "artifactScore": None if worst is None else worst[0],
                "_artifactArrays": None if worst is None else {"delta": worst[1], "perPixel": worst[2]},
            }
        )
        if checkpoint_failures:
            failures.extend(f"{checkpoint_id}: {failure}" for failure in checkpoint_failures)

    for checkpoint in checkpoint_results:
        checkpoint.pop("_artifactArrays", None)
    return {
        "status": "passed" if not failures else "failed",
        "failures": failures,
        "thresholds": thresholds.as_dict(),
        "viewport": {"source": source.viewport, "freya": freya.viewport, "passed": not _viewport_diff(source.viewport, freya.viewport)},
        "actions": {"source": list(source.action_ids), "freya": list(freya.action_ids), "passed": source_actions == freya_actions},
        "checkpoints": checkpoint_results,
        "inputs": {
            "sourceManifest": str(source.path),
            "freyaManifest": str(freya.path),
            "sourceRuntime": source.raw.get("runtime"),
            "freyaRuntime": freya.raw.get("runtime"),
        },
    }


def validate_pair(source: ManifestModel, freya: ManifestModel) -> None:
    issues = _viewport_diff(source.viewport, freya.viewport)
    if issues:
        raise ValidationError(issues)


__all__ = ["compare_manifests", "validate_pair"]
