"""Small data structures shared by the visual parity comparator."""

from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class Thresholds:
    antialias_pixel_threshold: int = 8
    structure_pixel_threshold: int = 32
    max_mae: float = 1.5
    max_rmse: float = 4.0
    max_antialias_ratio: float = 0.01
    max_structure_ratio: float = 0.0025
    min_ssim: float = 0.985
    max_geometry_delta_px: float = 2.0
    timestamp_tolerance_ms: float = 100.0
    max_motion_curve_mae: float = 0.05
    max_change_moment_delta_ms: float = 100.0
    max_distinct_hash_delta: int = 3

    def as_dict(self) -> dict[str, Any]:
        return {
            "antialias_pixel_threshold": self.antialias_pixel_threshold,
            "structure_pixel_threshold": self.structure_pixel_threshold,
            "max_mae": self.max_mae,
            "max_rmse": self.max_rmse,
            "max_antialias_ratio": self.max_antialias_ratio,
            "max_structure_ratio": self.max_structure_ratio,
            "min_ssim": self.min_ssim,
            "max_geometry_delta_px": self.max_geometry_delta_px,
            "timestamp_tolerance_ms": self.timestamp_tolerance_ms,
            "max_motion_curve_mae": self.max_motion_curve_mae,
            "max_change_moment_delta_ms": self.max_change_moment_delta_ms,
            "max_distinct_hash_delta": self.max_distinct_hash_delta,
        }

    @classmethod
    def from_mapping(cls, values: dict[str, Any] | None) -> "Thresholds":
        if not values:
            return cls()
        defaults = cls().as_dict()
        unknown = sorted(set(values) - set(defaults))
        if unknown:
            raise ValueError(f"unknown threshold keys: {', '.join(unknown)}")
        merged = {**defaults, **values}
        return cls(**merged)


@dataclass(frozen=True)
class FrameRef:
    index: int
    relative_ms: float
    path: Path
    manifest_path: str
    raw: dict[str, Any]


@dataclass(frozen=True)
class CheckpointRef:
    checkpoint_id: str
    after_action: str
    state: dict[str, Any]
    raw: dict[str, Any]
    frames: tuple[FrameRef, ...]


@dataclass(frozen=True)
class ManifestModel:
    raw: dict[str, Any]
    path: Path
    viewport: dict[str, Any]
    actions: tuple[dict[str, Any], ...]
    checkpoints: tuple[CheckpointRef, ...]
    artifact_root: Path

    @property
    def action_ids(self) -> tuple[str, ...]:
        return tuple(str(action["id"]) for action in self.actions)

    @property
    def checkpoint_map(self) -> dict[str, CheckpointRef]:
        return {checkpoint.checkpoint_id: checkpoint for checkpoint in self.checkpoints}


class ValidationError(Exception):
    """Raised when an evidence manifest cannot be trusted for comparison."""

    def __init__(self, issues: list[str] | str):
        self.issues = [issues] if isinstance(issues, str) else list(issues)
        super().__init__("; ".join(self.issues))
