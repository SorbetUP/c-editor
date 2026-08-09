# Acceptance test bridge

An explicitly instrumented Tauri process exposes `window.__ELEPHANT_ACCEPTANCE_TEST__` after the renderer is mounted. The bridge is not installed in a normal desktop build: it is enabled only when `ELEPHANT_ACCEPTANCE_TAURI_PORT` is set. Every command writes a structured entry to `window.__ELEPHANT_DEBUG_LOGS__` and to the terminal console.

```js
const test = window.__ELEPHANT_ACCEPTANCE_TEST__
test.capabilities()
await test.invokeTauri('tauri_markdown_render_html', { markdown: '# Probe' })
test.readDom('[data-testid="muya-rust-runtime-editor"]')
test.click('[aria-label="Search"]')
test.fill('[role="searchbox"]', 'Project Alpha')
test.press('[role="searchbox"]', 'Enter')
test.selectText('.en-editor-host', 0, 24)
test.contextClick('[data-elephant-citation-buffer-item]')
test.executeCommand('view.toggle-sidebar')
await test.waitFor('.en-search-results')
await test.selectVault('/absolute/path/to/test-vault')
await test.openNote('Projects/Beta.md')
test.setMarkdown('# Changed\n\nBody')
test.appendMarkdown('\n\nMore')
await test.save()
test.readState()     // Markdown, path, saved state and active vault
test.readDisplayed() // Markdown plus visible editor text/HTML
test.addonState()    // Registered addons, resources, resource methods and actions
await test.enableAddon('elephant.example')
await test.disableAddon('elephant.example')
await test.runAddonAction('elephant.example.action', { value: 1 })
await test.installOfficialAddon('elephant.dashboard')
await test.invokeAddonResource('sites.provider', 'status')
await test.addonNativeStatus('elephant.ai-ocr', 'sidecar')
await test.addonNativeStatus('elephant.sync', 'service')
await test.addonNativeCall('elephant.sync', 'sync.status', {}, { service: true })
await test.addonNativeCall('elephant.knowledge', 'knowledge.status', {}, { service: true })
await test.openExcalidraw('acceptance.excalidraw.png')
test.readExcalidraw()
test.closeExcalidraw()
test.logs()          // Complete structured acceptance log (not the capped UI diagnostics buffer)
```

The bridge is intended for functional automation, not visual assertions. It operates through the same Pinia stores, editor bus and save IPC used by the application. `logs()` reads the unbounded acceptance archive; the separate UI diagnostics buffer may remain bounded for production memory safety.

Desktop acceptance runs use `pnpm test:desktop:acceptance`. The runner launches the real Tauri desktop shell and talks to it through a loopback-only Tauri acceptance transport. It does not launch Electron, use CDP, or rely on visual screenshots. Every transport request, renderer command, result, and error is logged. After building a desktop package, `pnpm test:desktop:acceptance:packaged` detects and tests the release binary automatically.

By default the runner launches the development Tauri shell. Set `ELEPHANT_ACCEPTANCE_APP_PATH` to the packaged Tauri executable (for example `Elephant/backend/tauri/target/release/Elephant`) to replay the same scenario against the installed desktop binary. The fixture vault, temporary user profile and logs remain isolated.

Tauri production builds stage the official addon catalogue and the platform-native sidecars under the application resources. The release acceptance log records `official-addon-catalog source=bundled`; this prevents a clean installation from depending on a development checkout or on a source-only remote catalogue.

The last structured result is written to `test-results/acceptance/latest.json`; the complete Tauri stdout/stderr is written to `test-results/acceptance/latest-tauri.log`. The scenario starts from an isolated empty profile and verifies the first-run vault picker, selects a fixture vault, installs and enables all 16 official addons, probes every service-backed native addon, probes their resources, runs addon actions/views, executes Python Code Execution, and exercises Dashboard, Google Keep, Sites and Sync actions/resources alongside citation selection/paste/context-menu deletion and note/UI/file/drawing flows. It restarts Tauri and rereads the vault and edited note. It also includes deliberate unknown-command, invalid-path and missing-resource failures and requires those failures to appear in the structured renderer and Tauri logs, so a green run checks both success and error paths.

The workflow `.github/workflows/tauri-desktop-acceptance.yml` runs the packaged command on Linux x86_64, Windows x86_64, macOS Intel and macOS arm64, and uploads the JSON/Tauri logs even when a build or scenario fails.

The transport is disabled unless `ELEPHANT_ACCEPTANCE_TAURI_PORT` is set. It binds only to `127.0.0.1`, waits for the renderer `ready` signal, and exposes `GET /health` plus `POST /command` with `{ "command": "...", "args": [] }`. Each command-start entry stores the complete argument array in the structured archive; terminal output remains compacted for readability.
