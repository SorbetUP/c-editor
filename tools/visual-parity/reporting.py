"""Persist raw comparison data and human-readable PNG/Markdown artifacts."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from comparison import compare_manifests
from metrics import align_frames, load_image, image_metrics
from models import ManifestModel, Thresholds


def _write_pngs(output: Path, checkpoint_result: dict[str, Any], source, freya, thresholds: Thresholds) -> None:
    checkpoint_dir = output / "checkpoints" / checkpoint_result["id"]
    checkpoint_dir.mkdir(parents=True, exist_ok=True)
    alignment = align_frames(source.frames, freya.frames, thresholds.timestamp_tolerance_ms)
    worst: tuple[float, Any, Any] | None = None
    for pair in alignment["pairs"]:
        left_array = load_image(source.frames[pair["leftIndex"]].path)[1]
        right_array = load_image(freya.frames[pair["rightIndex"]].path)[1]
        metrics, delta, per_pixel = image_metrics(left_array, right_array, thresholds)
        score = (1.0 - metrics["ssim"]) + metrics["mae"] / 255.0 + metrics["ratio_pixels_over_structure_threshold"]
        if worst is None or score > worst[0]:
            worst = (score, delta, per_pixel)
    if worst is None:
        return
    np, Image, _ = _image_modules()
    delta = np.asarray(worst[1])
    per_pixel = np.asarray(worst[2], dtype=np.float64)
    diff = np.clip(delta * 4, 0, 255).astype(np.uint8)
    heat_value = np.clip(per_pixel * 4, 0, 255).astype(np.uint8)
    heatmap = np.stack(
        [
            heat_value,
            np.clip(heat_value.astype(np.int16) * 2, 0, 255).astype(np.uint8),
            np.clip(255 - heat_value.astype(np.int16), 0, 255).astype(np.uint8),
        ],
        axis=2,
    )
    Image.fromarray(diff, mode="RGB").save(checkpoint_dir / "diff.png")
    Image.fromarray(heatmap, mode="RGB").save(checkpoint_dir / "heatmap.png")
    checkpoint_result["artifacts"] = {
        "diff": str((checkpoint_dir / "diff.png").relative_to(output)),
        "heatmap": str((checkpoint_dir / "heatmap.png").relative_to(output)),
    }


def _image_modules():
    try:
        import numpy as np
        from PIL import Image
    except ImportError as exc:
        raise RuntimeError("PNG artifact generation requires Pillow and numpy") from exc
    return np, Image, None


def _markdown(result: dict[str, Any], output: Path) -> str:
    lines = [
        f"# Visual parity: `{result['status'].upper()}`",
        "",
        "The comparator aligns semantic checkpoint/action ids and frame timestamps; every aligned frame is scored. No baseline was updated.",
        "",
        f"- Source manifest: `{result.get('inputs', {}).get('sourceManifest', 'unavailable')}`",
        f"- Freya manifest: `{result.get('inputs', {}).get('freyaManifest', 'unavailable')}`",
        f"- Artifact root: `{output}`",
        "",
        "## Thresholds",
        "",
        "```json",
        json.dumps(result.get("thresholds", {}), indent=2, sort_keys=True),
        "```",
        "",
        "## Checkpoints",
        "",
        "| Checkpoint | Status | Frames | SSIM (worst) | MAE (worst) | Motion |",
        "| --- | --- | ---: | ---: | ---: | --- |",
    ]
    for checkpoint in result.get("checkpoints", []):
        frames = checkpoint.get("frames", [])
        metrics = [frame.get("metrics", {}) for frame in frames if frame.get("metrics")]
        worst_ssim = min((metric.get("ssim", 0.0) for metric in metrics), default=None)
        worst_mae = max((metric.get("mae", 0.0) for metric in metrics), default=None)
        motion = checkpoint.get("motion")
        motion_status = "n/a" if motion is None else ("pass" if motion.get("passed") else "fail")
        lines.append(
            f"| `{checkpoint.get('id')}` | {'PASS' if checkpoint.get('passed') else 'FAIL'} | "
            f"{len(frames)} | {worst_ssim if worst_ssim is not None else 'n/a'} | "
            f"{worst_mae if worst_mae is not None else 'n/a'} | {motion_status} |"
        )
    lines.extend(["", "## Failures", ""])
    failures = result.get("failures", [])
    if failures:
        lines.extend(f"- {failure}" for failure in failures)
    else:
        lines.append("- None")
    lines.extend(["", "## PNG artifacts", ""])
    artifacts = [checkpoint.get("artifacts", {}) for checkpoint in result.get("checkpoints", [])]
    artifacts = [item for item in artifacts if item]
    if artifacts:
        for item in artifacts:
            lines.append(f"- diff: `{item['diff']}`; heatmap: `{item['heatmap']}`")
    else:
        lines.append("- None (validation failed before frame comparison or no frame was available)")
    return "\n".join(lines) + "\n"


def write_artifacts(result: dict[str, Any], output: str | Path, source: ManifestModel | None = None, freya: ManifestModel | None = None) -> Path:
    output_path = Path(output).expanduser().resolve()
    output_path.mkdir(parents=True, exist_ok=True)
    if source is not None and freya is not None:
        thresholds = Thresholds.from_mapping(result.get("thresholds"))
        source_map = source.checkpoint_map
        freya_map = freya.checkpoint_map
        for checkpoint_result in result.get("checkpoints", []):
            checkpoint_id = checkpoint_result["id"]
            if checkpoint_id in source_map and checkpoint_id in freya_map:
                _write_pngs(output_path, checkpoint_result, source_map[checkpoint_id], freya_map[checkpoint_id], thresholds)
    (output_path / "comparison.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    (output_path / "summary.md").write_text(_markdown(result, output_path), encoding="utf-8")
    return output_path


def write_failure_artifacts(output: str | Path, failures: list[str], inputs: dict[str, Any] | None = None) -> Path:
    result = {
        "status": "failed",
        "failures": failures,
        "thresholds": Thresholds().as_dict(),
        "inputs": inputs or {},
        "checkpoints": [],
    }
    return write_artifacts(result, output)


__all__ = ["write_artifacts", "write_failure_artifacts"]
