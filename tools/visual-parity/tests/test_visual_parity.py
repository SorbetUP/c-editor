import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).parents[1] / "visual_parity.py"


def load_module():
    spec = importlib.util.spec_from_file_location("visual_parity", MODULE_PATH)
    if spec is None or spec.loader is None:
        raise AssertionError(f"visual parity module missing: {MODULE_PATH}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class VisualParityTests(unittest.TestCase):
    def test_rejects_self_declared_pass_with_unchanged_scroll_and_rail_drag(self):
        manifest_path = Path("/private/tmp/elephant-source-playwright.GtMMxG/output/manifest.json")
        self.assertTrue(manifest_path.is_file(), manifest_path)
        module = load_module()
        with self.assertRaises(module.ValidationError):
            module.validate_manifest(json.loads(manifest_path.read_text()))

    def _manifest(self, root: Path, image_name: str, geometry_x: int | None = None):
        module = load_module()
        image_path = root / image_name
        digest = __import__("hashlib").sha256(image_path.read_bytes()).hexdigest()
        state = {}
        if geometry_x is not None:
            state["geometry"] = {"All notes": [{"x": geometry_x, "y": 0, "width": 4, "height": 4}]}
        raw = {
            "schemaVersion": 1,
            "scenarioId": "minimal-identical",
            "runtime": "test",
            "status": "passed",
            "viewport": {"width": 8, "height": 8, "scaleFactor": 1, "deviceScaleFactor": 1, "fullPage": False},
            "provenance": {"artifactRoot": str(root)},
            "actions": [{"index": 0, "id": "launch", "status": "passed"}],
            "checkpoints": [{
                "id": "startup",
                "afterAction": "launch",
                "state": state,
                "frames": [{"index": 0, "relativeMs": 0, "path": image_name, "sha256": digest}],
            }],
            "validation": {
                "allActionsExecuted": True,
                "allTargetsPlaywrightResolved": True,
                "stateAndVaultHashesCaptured": True,
            },
        }
        return module.validate_manifest(raw)

    def test_identical_minimal_pair_passes_and_writes_raw_markdown_png_artifacts(self):
        module = load_module()
        with tempfile.TemporaryDirectory(prefix="visual-parity-test-") as temporary:
            root = Path(temporary)
            from PIL import Image

            image = Image.new("RGB", (8, 8), (20, 40, 60))
            image.save(root / "frame.png")
            source = self._manifest(root, "frame.png")
            freya = self._manifest(root, "frame.png")
            result = module.compare_manifests(source, freya, module.Thresholds())
            output = root / "artifacts"
            module.write_artifacts(result, output, source, freya)
            self.assertEqual(result["status"], "passed")
            self.assertTrue((output / "comparison.json").is_file())
            self.assertTrue((output / "summary.md").is_file())
            self.assertTrue((output / "checkpoints/startup/diff.png").is_file())
            self.assertTrue((output / "checkpoints/startup/heatmap.png").is_file())
            self.assertNotIn("<html", (output / "summary.md").read_text().lower())

    def test_large_declared_geometry_difference_fails_even_when_pixels_match(self):
        module = load_module()
        with tempfile.TemporaryDirectory(prefix="visual-parity-test-") as temporary:
            root = Path(temporary)
            from PIL import Image

            Image.new("RGB", (8, 8), (20, 40, 60)).save(root / "frame.png")
            source = self._manifest(root, "frame.png", geometry_x=0)
            freya = self._manifest(root, "frame.png", geometry_x=12)
            result = module.compare_manifests(source, freya, module.Thresholds())
            self.assertEqual(result["status"], "failed")
            geometry = result["checkpoints"][0]["geometry"]
            self.assertFalse(geometry["passed"])
            self.assertGreater(geometry["maxDeltaPx"], module.Thresholds().max_geometry_delta_px)

    def test_motion_contract_rejects_identical_frames(self):
        module = load_module()
        with tempfile.TemporaryDirectory(prefix="visual-parity-test-") as temporary:
            root = Path(temporary)
            from PIL import Image

            Image.new("RGB", (8, 8), (20, 40, 60)).save(root / "frame.png")
            image_digest = __import__("hashlib").sha256((root / "frame.png").read_bytes()).hexdigest()
            raw = {
                "schemaVersion": 1,
                "status": "passed",
                "viewport": {"width": 8, "height": 8, "scaleFactor": 1, "deviceScaleFactor": 1},
                "provenance": {"artifactRoot": str(root)},
                "actions": [{"id": "scroll-note", "status": "passed"}],
                "checkpoints": [{
                    "id": "note-scrolled",
                    "afterAction": "scroll-note",
                    "state": {"scroll": {"mustChange": True}},
                    "frames": [
                        {"index": 0, "relativeMs": 0, "path": "frame.png", "sha256": image_digest},
                        {"index": 1, "relativeMs": 50, "path": "frame.png", "sha256": image_digest},
                    ],
                }],
                "validation": {
                    "allActionsExecuted": True,
                    "allTargetsPlaywrightResolved": True,
                    "stateAndVaultHashesCaptured": True,
                },
            }
            with self.assertRaises(module.ValidationError):
                module.validate_manifest(raw)


if __name__ == "__main__":
    unittest.main()
