"""Image and motion metrics for deterministic, timestamp-aligned evidence."""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any

from models import FrameRef, Thresholds


class DependencyError(RuntimeError):
    """Raised with an actionable message when image dependencies are absent."""


def _libraries():
    try:
        import numpy as np
        from PIL import Image
        from skimage.metrics import structural_similarity
    except ImportError as exc:
        raise DependencyError(
            "visual comparison requires Pillow and scikit-image (with numpy); "
            "install them in the test environment, no network install is performed"
        ) from exc
    return np, Image, structural_similarity


def load_image(path: Path):
    np, Image, _ = _libraries()
    try:
        with Image.open(path) as image:
            rgb = image.convert("RGB")
            return rgb.copy(), np.asarray(rgb, dtype=np.uint8)
    except OSError as exc:
        raise ValueError(f"cannot read PNG {path}: {exc}") from exc


def image_size(path: Path) -> tuple[int, int]:
    _, Image, _ = _libraries()
    try:
        with Image.open(path) as image:
            return image.size
    except OSError as exc:
        raise ValueError(f"cannot read image {path}: {exc}") from exc


def image_metrics(left, right, thresholds: Thresholds) -> tuple[dict[str, Any], Any, Any]:
    np, _, structural_similarity = _libraries()
    if left.shape != right.shape:
        raise ValueError(f"image arrays differ in shape: {left.shape} vs {right.shape}")
    delta = np.abs(left.astype(np.int16) - right.astype(np.int16))
    per_pixel = delta.max(axis=2)
    squared = np.square(left.astype(np.float64) - right.astype(np.float64))
    mae = float(delta.mean())
    rmse = float(np.sqrt(squared.mean()))
    antialias_ratio = float((per_pixel > thresholds.antialias_pixel_threshold).mean())
    structure_ratio = float((per_pixel > thresholds.structure_pixel_threshold).mean())
    left_gray = left.astype(np.float64).mean(axis=2) / 255.0
    right_gray = right.astype(np.float64).mean(axis=2) / 255.0
    ssim = float(structural_similarity(left_gray, right_gray, data_range=1.0))
    return (
        {
            "mae": mae,
            "rmse": rmse,
            "ratio_pixels_over_antialias_threshold": antialias_ratio,
            "ratio_pixels_over_structure_threshold": structure_ratio,
            "ssim": ssim,
            "dimensions": {"width": int(left.shape[1]), "height": int(left.shape[0])},
            "thresholds": {
                "antialias_pixel_threshold": thresholds.antialias_pixel_threshold,
                "structure_pixel_threshold": thresholds.structure_pixel_threshold,
            },
        },
        delta,
        per_pixel,
    )


def visual_pass(metrics: dict[str, Any], thresholds: Thresholds) -> bool:
    return (
        metrics["mae"] <= thresholds.max_mae
        and metrics["rmse"] <= thresholds.max_rmse
        and metrics["ratio_pixels_over_antialias_threshold"] <= thresholds.max_antialias_ratio
        and metrics["ratio_pixels_over_structure_threshold"] <= thresholds.max_structure_ratio
        and metrics["ssim"] >= thresholds.min_ssim
    )


def frame_hash(frame: FrameRef) -> str:
    return hashlib.sha256(frame.path.read_bytes()).hexdigest()


def motion_profile(frames: tuple[FrameRef, ...]) -> dict[str, Any]:
    np, _, _ = _libraries()
    hashes = [frame_hash(frame) for frame in frames]
    energies: list[float] = []
    intervals: list[dict[str, Any]] = []
    arrays = [load_image(frame.path)[1] for frame in frames]
    for index in range(1, len(frames)):
        delta = np.abs(arrays[index].astype(np.int16) - arrays[index - 1].astype(np.int16))
        energy = float(delta.mean() / 255.0)
        energies.append(energy)
        intervals.append(
            {
                "fromIndex": index - 1,
                "toIndex": index,
                "fromMs": frames[index - 1].relative_ms,
                "toMs": frames[index].relative_ms,
                "energy": energy,
            }
        )
    changed_index = next((index for index in range(1, len(hashes)) if hashes[index] != hashes[index - 1]), None)
    return {
        "frameCount": len(frames),
        "distinctHashCount": len(set(hashes)),
        "hashes": hashes,
        "firstChangedFrameIndex": changed_index,
        "firstChangeMs": frames[changed_index].relative_ms if changed_index is not None else None,
        "motionEnergyCurve": intervals,
        "totalMotionEnergy": float(sum(energies)),
    }


def align_frames(left: tuple[FrameRef, ...], right: tuple[FrameRef, ...], tolerance_ms: float) -> dict[str, Any]:
    if not left or not right:
        return {"pairs": [], "maxTimestampDeltaMs": None, "alignable": False, "reason": "empty frame timeline"}
    pairs: list[dict[str, Any]] = []
    max_delta = 0.0
    for left_index, left_frame in enumerate(left):
        right_index = min(
            range(len(right)),
            key=lambda candidate: (abs(right[candidate].relative_ms - left_frame.relative_ms), candidate),
        )
        delta = abs(right[right_index].relative_ms - left_frame.relative_ms)
        max_delta = max(max_delta, delta)
        pairs.append({"leftIndex": left_index, "rightIndex": right_index, "deltaMs": delta})
    reverse_max = max(
        abs(right_frame.relative_ms - min((left_frame.relative_ms for left_frame in left), key=lambda value: abs(value - right_frame.relative_ms)))
        for right_frame in right
    )
    max_delta = max(max_delta, reverse_max)
    return {
        "pairs": pairs,
        "maxTimestampDeltaMs": max_delta,
        "alignable": max_delta <= tolerance_ms,
        "reason": None if max_delta <= tolerance_ms else f"timestamp delta {max_delta:.3f}ms exceeds {tolerance_ms:.3f}ms",
    }


def _rectangles(value: Any) -> dict[str, list[dict[str, float]]]:
    if not isinstance(value, dict):
        return {}
    result: dict[str, list[dict[str, float]]] = {}
    for label, entries in value.items():
        candidates = entries if isinstance(entries, list) else [entries]
        rects: list[dict[str, float]] = []
        for entry in candidates:
            if not isinstance(entry, dict):
                continue
            keys = ("x", "y", "width", "height")
            if all(isinstance(entry.get(key), (int, float)) for key in keys):
                rects.append({key: float(entry[key]) for key in keys})
        if rects:
            result[str(label)] = rects
    return result


def geometry_metrics(left_state: dict[str, Any], right_state: dict[str, Any], thresholds: Thresholds) -> dict[str, Any] | None:
    left = _rectangles(left_state.get("geometry"))
    right = _rectangles(right_state.get("geometry"))
    if not left and not right:
        return None
    missing_left = sorted(set(right) - set(left))
    missing_right = sorted(set(left) - set(right))
    deltas: list[float] = []
    mismatched_counts: list[str] = []
    for label in sorted(set(left) & set(right)):
        if len(left[label]) != len(right[label]):
            mismatched_counts.append(label)
            continue
        for left_rect, right_rect in zip(left[label], right[label]):
            deltas.extend(abs(left_rect[key] - right_rect[key]) for key in ("x", "y", "width", "height"))
    max_delta = max(deltas, default=0.0)
    passed = not missing_left and not missing_right and not mismatched_counts and max_delta <= thresholds.max_geometry_delta_px
    return {
        "labelsCompared": len(set(left) & set(right)),
        "missingInLeft": missing_left,
        "missingInRight": missing_right,
        "mismatchedCounts": mismatched_counts,
        "maxDeltaPx": max_delta,
        "thresholdPx": thresholds.max_geometry_delta_px,
        "passed": passed,
    }


def region_metrics(left_state: dict[str, Any], right_state: dict[str, Any], left, right, thresholds: Thresholds) -> list[dict[str, Any]] | None:
    left_regions = left_state.get("regions")
    right_regions = right_state.get("regions")
    if left_regions is None and right_regions is None:
        return None
    if not isinstance(left_regions, dict) or not isinstance(right_regions, dict):
        return [{"passed": False, "error": "regions must be matching objects"}]
    np, _, _ = _libraries()
    results = []
    for name in sorted(set(left_regions) | set(right_regions)):
        l = left_regions.get(name)
        r = right_regions.get(name)
        if not isinstance(l, dict) or not isinstance(r, dict):
            results.append({"name": name, "passed": False, "error": "region missing"})
            continue
        box_keys = ("x", "y", "width", "height")
        if not all(isinstance(l.get(key), (int, float)) and isinstance(r.get(key), (int, float)) for key in box_keys):
            results.append({"name": name, "passed": False, "error": "region geometry incomplete"})
            continue
        lx, ly, lw, lh = (int(round(l[key])) for key in box_keys)
        rx, ry, rw, rh = (int(round(r[key])) for key in box_keys)
        if (lw, lh) != (rw, rh) or lw <= 0 or lh <= 0:
            results.append({"name": name, "passed": False, "error": "region dimensions differ"})
            continue
        if min(lx, ly, rx, ry) < 0 or lx + lw > left.shape[1] or ly + lh > left.shape[0] or rx + rw > right.shape[1] or ry + rh > right.shape[0]:
            results.append({"name": name, "passed": False, "error": "region outside image"})
            continue
        metrics, _, _ = image_metrics(left[ly : ly + lh, lx : lx + lw], right[ry : ry + rh, rx : rx + rw], thresholds)
        results.append({"name": name, "metrics": metrics, "passed": visual_pass(metrics, thresholds)})
    return results


__all__ = [
    "DependencyError",
    "align_frames",
    "frame_hash",
    "geometry_metrics",
    "image_metrics",
    "image_size",
    "load_image",
    "motion_profile",
    "region_metrics",
    "visual_pass",
]
