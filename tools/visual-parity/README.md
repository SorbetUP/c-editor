# Elephant visual parity comparator

This CLI compares two real capture manifests without changing either
manifest or updating a baseline. It aligns checkpoint ids, `afterAction` ids,
viewport/DPI and frame timestamps. It scores every timestamp-aligned frame;
the PNG selected for display is only the worst observed pair and never changes
the pass/fail result.

## Usage

```bash
python3 tools/visual-parity/visual_parity.py compare \
  --source /path/to/source/manifest.json \
  --freya /path/to/freya/manifest.json \
  --output /private/tmp/elephant-visual-parity-run
```

The command writes `comparison.json`, `summary.md`, and one `diff.png` plus
one `heatmap.png` under `checkpoints/<checkpoint-id>/`. The default output is
a fresh `/private/tmp/elephant-visual-parity-*` directory. It exits 0 only
when all required evidence and thresholds pass, 2 for evidence or parity
failure, and 3 when Pillow/scikit-image/numpy are unavailable.

## Contract checks

Before image work, both manifests must be schema version 1, declare a passed
run, pass all runtime validation flags, contain passed actions and complete
frame paths. The validator rejects explicit `mustChange:false`, missing or
unchanged scroll evidence, unchanged rail order after a drag, and motion
actions with fewer than two or identical frames. Viewport width, height,
scaleFactor and deviceScaleFactor must match exactly; every PNG must have the
declared viewport dimensions.

The default thresholds are deliberately explicit and strict:

```bash
python3 tools/visual-parity/visual_parity.py defaults --pretty
```

They separately measure antialiasing (`>8` pixel values) and structural
changes (`>32`), as well as raw 8-bit MAE/RMSE, SSIM, declared geometry,
timestamp alignment, distinct motion hashes, first-change time and motion
energy curve error. A JSON object passed with `--config` can override only
named threshold keys; no baseline or threshold is updated automatically.

The image metrics require Pillow, numpy and scikit-image. The tool does not
download dependencies. `validate` remains available for contract-only checks:

```bash
python3 tools/visual-parity/visual_parity.py validate /path/to/manifest.json
```
