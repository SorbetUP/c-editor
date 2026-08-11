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
is Chromium-only. Official Tauri guidance for macOS now requires
`@wdio/tauri-service` with `driverProvider: 'embedded'` and
`tauri-plugin-wdio-webdriver`; neither is present in this repository, so this
write set does not label the native CGEvent adapter as a WebDriver session.

The current product source also does not consume the requested
`ELEPHANTNOTE_CONFIG_DIR`, `ELEPHANTNOTE_USER_DATA_DIR`, or
`ELEPHANTNOTE_PROFILE_DIR` overrides when resolving Tauri `app_config_dir` and
`app_data_dir`. The adapter preserves the inherited `HOME`, materializes the
shared fixture explicitly, records those app-specific variables, and marks the
real run `NOT PROVEN`/blocked until the product exposes a supported profile
override or the embedded WDIO route is installed.
