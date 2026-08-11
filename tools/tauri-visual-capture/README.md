# Native Tauri visual capture

This is the strict macOS adapter for the shared differential scenario. It
loads `migration/freya/differential-scenarios.json` by default, or the file
passed with `--shared-scenario`, and emits the `schemaVersion: 1` manifest
consumed by `tools/freya-differential/orchestrate.mjs`.

```sh
node --test tools/tauri-visual-capture/tests/*.test.mjs
node tools/tauri-visual-capture/playwright-probe.mjs
node tools/tauri-visual-capture/index.mjs \
  --shared-scenario migration/freya/differential-scenarios.json \
  --output test-results/tauri-visual-capture/run
```

In orchestrated mode the adapter consumes `DIFFERENTIAL_SCENARIO_PATH`,
`DIFFERENTIAL_OUTPUT_DIR`, `DIFFERENTIAL_FIXTURE_ROOT`, and the other
provenance variables supplied by the shared orchestrator. It never substitutes
the old seven-step scenario. Every one of the 14 shared actions and every
checkpoint is emitted; an unavailable target is a `blocked` action and a hard
failure, not a skipped action or a successful bridge command.

The acceptance HTTP bridge is observation-only (`readDom`, `readState`, and
readiness checks). User input is dispatched through macOS `CGEvent` pointer,
keyboard, text, scroll, and drag events after resolving target bounds from the
real Tauri process's Accessibility tree. The physical request files and event
timestamps are retained. If profile isolation or target resolution blocks the
run, no DOM action fallback is attempted and the manifest says so.

The launcher may be `build/scripts/build_dev.sh`, but window selection proves
the selected owner is a unique descendant whose argv is the actual
`target/debug/Elephant`/packaged Tauri executable. A launcher-only match is
rejected.

Content is fail-closed. The harness measures the WebArea or native traffic
light bounds through macOS Accessibility/System Events. When native controls
are present, it crops using those measured bounds; it never removes a guessed
number of pixels. The manifest records the chrome proof, measured content
rectangle, crop, PNG dimensions, SHA-256, and capture errors. The current
`screencapture -l` fallback remains native evidence but is not comparison-ready
until Retina pixels are normalized to the shared viewport.
The exact PNG crop uses `ffmpeg` from `PATH` and fails closed if it is unavailable
or if the resulting dimensions do not match the measured rectangle.

The Playwright capability probe remains intentionally separate and honest:
`webkit.connectOverCDP` reports unsupported because Playwright CDP attachment
is Chromium-only. Official Tauri guidance for macOS requires
`@wdio/tauri-service` with `driverProvider: 'embedded'` and
`tauri-plugin-wdio-webdriver`. The dedicated implementation lives in
`tools/tauri-wdio-embedded`; this native CGEvent adapter remains a separate
diagnostic capture path and does not label its evidence as a WebDriver session.

The acceptance contract enables the smallest product hook through
`ELEPHANT_ACCEPTANCE_TAURI_PORT` plus the explicit absolute
`ELEPHANT_ACCEPTANCE_PROFILE_DIR`. That hook redirects only Tauri config and
user-data roots for the acceptance process; normal launches keep their normal
directories. The adapter preserves the inherited `HOME`, materializes the
shared fixture explicitly, and proves the active vault, config file, and
user-data marker against that temporary root before allowing `launch` to pass.
