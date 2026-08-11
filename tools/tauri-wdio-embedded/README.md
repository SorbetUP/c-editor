# Elephant Tauri embedded WebDriver

This is the official macOS/WKWebView acceptance runner. It launches the real
Tauri binary through `@wdio/tauri-service` with `driverProvider: 'embedded'`;
the application is built with the test-only `acceptance-wdio` Cargo feature.
The runner uses the shared fourteen-action differential scenario and real
WebDriver selectors, keyboard, wheel, pointer and drag gestures. It does not
use `browser.execute`, `evaluate`, IPC mocks or injected success values.

Install the dedicated tool with network access:

```bash
pnpm --dir tools/tauri-wdio-embedded --ignore-workspace install
```

Build the acceptance binary with the inline test capability from
`Elephant/backend/tauri/tauri.wdio.conf.json` and run:

```bash
pnpm --dir tools/tauri-wdio-embedded --ignore-workspace exec wdio run wdio.conf.mjs
```

Set `ELEPHANT_TAURI_ACCEPTANCE_BINARY`, `DIFFERENTIAL_OUTPUT_DIR` and
`DIFFERENTIAL_FIXTURE_ROOT` to control the binary and fresh evidence folder.
The resulting `manifest.json` identifies `runtime=tauri-wdio-embedded` and
records measured postconditions and frame hashes.
