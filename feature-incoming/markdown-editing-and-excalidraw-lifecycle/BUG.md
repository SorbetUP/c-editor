# Markdown editing and Excalidraw lifecycle

- Request: node NodeId(10) is not in a supported editable structure; Markdown unusable; Excalidraw cannot be exited; test all flows automatically and log everything
- Level: critical
- Date: 2026-07-19
- Slug: markdown-editing-and-excalidraw-lifecycle

## Summary
- [x] Reproduced the reported Markdown/Excalidraw failure from the supplied Tauri log.
- [x] Isolated two independent paths: Rust structural editing and Tauri image URL loading.
- [ ] Complete a clean, repeatable Tauri desktop acceptance run before release.

## Repro Steps
- [x] Start the real Tauri app with `pnpm tauri:dev`.
- [x] Open a note containing Markdown Excalidraw previews under `.assets/`.
- [x] Observe `browser image element emitted an error` for existing PNG files served through `asset://`.
- [x] Exercise Enter at Markdown carets, including fenced code and table cells; the original failure was `node NodeId(10) is not in a supported editable structure`.
- [x] Open and leave the Excalidraw editor; lifecycle logs show the overlay unmount path.

## Environment
- [x] macOS arm64, Tauri desktop development runtime, Rust Muya editor, Node 22, pnpm 10.
- [x] Supplied evidence: `pasted-text-1.txt` and `test-results/observability/*tauri-dev*.log`.

## Observed vs Expected
- Observed:
- The original Rust command rejected valid Markdown carets inside code blocks/table cells.
- A Tauri `asset://` preview failed in the browser although the corresponding file existed and the Rust file bridge could read it.
- Existing tests covered the component and compatibility harness but did not prove the whole desktop Tauri editor interaction path.
- Expected:
- Every valid Markdown caret accepts editing and persists the document.
- Existing Excalidraw previews render in Tauri, and a browser image failure falls back to bytes read through the Tauri bridge.
- Close, Escape, navigation, save and initialization failures are observable and covered by automated tests.

## Hypotheses
- [x] `InsertParagraph` was routed to structural paragraph splitting for code blocks/table cells instead of literal newline insertion.
- [x] The Excalidraw image recovery handler repaired only `ag-image-fail` containers; direct `<img>` errors retried the same `asset://` URL.
- [ ] A dedicated Tauri desktop acceptance runner is still needed to exercise real renderer input without relying on Electron/CDP.

## Investigation Plan
- [x] Trace the error through Muya protocol dispatch, paragraph boundary matching and input selection recovery.
- [x] Trace Excalidraw preview resolution, `asset://` failures, Tauri file reads and overlay lifecycle logs.
- [x] Add structured renderer diagnostics with bounded retention.
- [ ] Run a clean Tauri acceptance fixture and retain its logs/artifacts.

## Fix Plan
- [x] Use literal newline insertion inside code blocks and table cells.
- [x] Recover authoritative selections after DOM refreshes.
- [x] Move Excalidraw UI into a stable shell zone and close it on explicit close/Escape/navigation.
- [x] On a direct browser image error, read the existing PNG through `window.fileUtils.readFile` and replace the source with a data URL.
- [ ] Keep all required Tauri acceptance checks required and green in CI.

## Regression Tests
- [x] Rust protocol regression iterating Markdown caret positions.
- [x] Rust core tests and formatting checks.
- [x] Vue Excalidraw close/Escape/initialization diagnostics tests.
- [x] Tauri Excalidraw image-loading tests, including browser-error fallback.
- [x] Full unit suite: 3200 passed, 171 skipped.
- [ ] Clean Tauri desktop acceptance test covering set/edit/save/reopen and Excalidraw close.

## Release Notes
- [x] The editor no longer treats code-block/table Enter as an unsupported structural edit.
- [x] Tauri Excalidraw previews recover from rejected `asset://` URLs using the Rust-owned file bridge.

## Risks
- [ ] Data URL conversion adds memory overhead for large images; keep it limited to failed/reload paths and monitor bounded logs.
- [ ] The desktop Tauri acceptance harness must avoid mutating the user vault.

## Rollout
- [ ] Merge the image fallback and test changes only after local and CI Tauri acceptance evidence is green.
