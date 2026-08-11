# Drawing entry and library toolbar clickability

- Request: Opening a saved drawing must open the drawing editor directly, not a note editor; Excalidraw assets with repeatedly encoded names must load; library sort and view controls must remain clickable when floating over cards.
- Level: high
- Date: 2026-08-10
- Slug: drawing-entry-toolbar-clickability

## Summary
- [x] A saved drawing is represented/opened through the note editor instead of opening the Excalidraw drawing flow directly.
- [x] A drawing preview can retain repeated percent-encoding and fail with `local-file-read-error` when opened.
- [x] The floating sort/view controls can overlap the first library card and are reported as not clickable.

## Repro Steps
- [x] Start the current `develop` worktree with a clean vault containing a Markdown entry whose front matter is `type: "drawing"` and whose Excalidraw preview is under `.assets/`.
- [x] Click the drawing card in the library. The current path falls back to `openNote` when the backend reports the entry as a Markdown note; the expected result is the Excalidraw dialog with no Muya note editor.
- [x] Open a preview whose persisted source contains repeated `%25` encoding, as in the supplied screenshot. The observed error is `Failed to load image ... Reason: local-file-read-error`.
- [x] With several library cards visible, click the sort and view buttons positioned at the top-right of the library. The controls must receive the pointer event and change their state even when a card is underneath.

## Environment
- [x] ElephantNote 0.1.0, current `develop` worktree, macOS desktop/Electron UI harness.
- [x] User screenshot: `codex-clipboard-b987d35b-2dbe-4596-bf51-b4278863c3ad.png`.
- [x] User screenshot: `codex-clipboard-11b0e4e8-274e-4ecd-aa80-86248a2be0fa.png`.
- [x] Existing regression evidence: `/tmp/elephant-asset-encoding-red.log` records the repeated `%25` asset failure before the bounded decoder; the current direct library-opening path is not covered by a test.

## Observed vs Expected
- Observed: `LibraryGrid.openEntry` routes Markdown paths to `vaultStore.openNote`, while the saved drawing note is written with note metadata; direct opening can therefore mount Muya and the image resolver can receive a multiply encoded path. Sort/view controls are absolutely positioned over the first card with no reserved toolbar layer contract.
- Expected: drawing entries are classified as `drawing`, clicking one emits the existing production `open-excalidraw-from-image` path, repeated encoding is decoded with a bounded routine, and sort/view controls remain above the card hit-test layer and update state.

## Hypotheses
- [x] The backend directory summary always returns `type: "note"` for every Markdown file instead of preserving drawing front matter.
- [x] The drawing save path writes `type: "note"` and `LibraryGrid` has no explicit drawing branch.
- [x] The Excalidraw overlay uses a single `decodeURI` pass while older persisted paths may be encoded repeatedly.
- [x] Toolbar/card stacking and pointer-event ownership are not proven by an element hit-test assertion.

## Investigation Plan
- [x] Inspect branch, dirty worktree, drawing save/open path, backend directory metadata, image resolver, toolbar CSS and existing tests.
- [x] Run focused pre-fix UI/unit tests and record failures or missing coverage.
- [x] Verify the real user path after the smallest production fix.

## Fix Plan
- [x] Preserve `drawing` in directory metadata, write new drawing notes with drawing metadata, and recognize the narrow legacy drawing-note shape.
- [x] Route Markdown drawing cards and direct `.excalidraw` cards directly to the existing Excalidraw bus action and keep note cards on the note editor path.
- [x] Decode persisted image sources with a bounded, idempotent routine in the production overlay and Muya loader paths.
- [x] Load drawing previews in library cards through the vault file API instead of an unusable relative or `file://` browser URL.
- [x] Make toolbar hit-testing explicit and test clicks with a card underneath.

## Regression Tests
- [x] UI: create/name/save drawing, click its library card, assert Excalidraw dialog visible and Muya editor absent.
- [x] UI: direct `.excalidraw` card opens Excalidraw without mounting Muya.
- [x] Unit: repeatedly encoded `file://` preview resolves to the real local path before `readFile`.
- [x] UI: sort cycle and grid/list cycle receive clicks in the reserved toolbar hit area and change their data attributes.
- [x] Visual UI: capture and inspect the library toolbar, Markdown drawing preview, direct Excalidraw dialog, and Markdown Excalidraw dialog.
- [x] Rust/unit: directory summaries preserve `type: "drawing"` from front matter.
- [x] Rust/unit: legacy drawing notes with only a heading and Excalidraw image are classified as drawings.
- [x] Rust + Electron UI: folder summaries expose a sorted, capped `childrenPreview`, the rendered folder card shows it, and its normal grid height matches a standard note card.
- [x] Visual UI: Excalidraw close/save actions are on the same row as `Library` and remain to its left without overlap.

## Evidence

- Pre-fix direct drawing test: `/tmp/elephant-direct-drawing-red.log`, exit 1; the card was visible but `excalidraw-dialog` never opened.
- Pre-fix repeated-encoding reproduction from the image-loader audit: `/tmp/elephant-asset-encoding-red.log`; `readFile` received an encoded path instead of the decoded vault path.
- Direct-file card visual regression turned red at `/tmp/elephant-direct-card-visual-final-2.log`: the rendered title still contained `.excalidraw`; the green rerun is `/tmp/elephant-direct-card-visual-final-3.log`.
- Final focused UI run: `/tmp/elephant-drawing-toolbar-final-all.log`, 4 passed in 8.2s. It covers drawing preview/open, create-name-save-reopen, direct `.excalidraw` open, and sort/view hit testing.
- Visual regression run: `/tmp/elephant-visual-regressions-final-4.log`, 2 passed. Captures are in `/tmp/elephant-visual-evidence/`: `library-toolbar-hit-area.png`, `markdown-drawing-library-preview.png`, `direct-drawing-excalidraw-dialog.png`, and `markdown-drawing-excalidraw-dialog.png`; each was inspected after the run.
- Latest visual regression run after direct-file card cleanup: `/tmp/elephant-direct-card-visual-final-3.log`, 2 passed in 7.0s. The additional `/tmp/elephant-visual-evidence/direct-drawing-library-preview.png` capture proves that a direct `.excalidraw` card has a drawing icon, a clean title, and a loaded sidecar PNG instead of raw JSON.
- The first visual run exposed the broken relative preview (`/tmp/elephant-visual-regressions.log`); the final capture shows the real image after routing the preview through `fileUtils.readFile` and an object URL.
- Final direct-drawing proof: `/tmp/elephant-direct-drawing-green.log`, 1 passed; the log contains `open existing drawing:success` and confirms Muya was absent.
- Final focused unit run: `/tmp/elephant-drawing-unit-final.log`, 4 files and 20 tests passed.
- Final full unit run: `/tmp/elephant-unit-full-final.log`, 173 files passed, 27 skipped; 3236 tests passed, 171 skipped.
- Latest full unit run after direct-file card cleanup: `/tmp/elephant-full-unit-after-direct-drawing.log`, 173 files passed, 27 skipped; 3237 tests passed, 171 skipped.
- Final Rust proof: `/tmp/elephant-drawing-rust-final-2.log`, 4 drawing-related tests passed and 1220 filtered out.
- Latest folder-preview Rust proof: `rtk cargo test --manifest-path Elephant/backend/tauri/Cargo.toml includes_a_small_sorted_preview_for_folder_cards`, 1 passed and 1224 filtered out.
- Latest visual proof: `ELEPHANT_E2E_HIDE_WINDOW=1 pnpm exec playwright test tests/app/e2e/drawing-visual-regressions.spec.js --workers=1`, 2 passed. The inspected captures are `/tmp/elephant-visual-evidence/library-toolbar-hit-area.png`, `/tmp/elephant-visual-evidence/direct-drawing-library-preview.png`, `/tmp/elephant-visual-evidence/direct-drawing-excalidraw-dialog.png`, `/tmp/elephant-visual-evidence/markdown-drawing-library-preview.png`, and `/tmp/elephant-visual-evidence/markdown-drawing-excalidraw-dialog.png`.
- Latest library-shell proof: `ELEPHANT_E2E_HIDE_WINDOW=1 pnpm exec playwright test tests/app/e2e/library-shell-regressions.spec.js --workers=1`, 3 passed.
- Latest full unit proof: `/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-10T16-03-44-292Z-vitest-unit-33084.log`, 173 files passed, 27 skipped; 3237 tests passed, 171 skipped.
- Final full unit proof after the folder preview type and Excalidraw extension consistency changes: `/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-10T16-08-39-657Z-vitest-unit-36890.log`, 173 files passed, 27 skipped; 3237 tests passed, 171 skipped.
- Final targeted Rust proof after classifying `.excalidraw` folder children as drawings: `rtk cargo test --manifest-path Elephant/backend/tauri/Cargo.toml includes_a_small_sorted_preview_for_folder_cards`, 1 passed and 1224 filtered out.
- Final visual proof after the same changes: `ELEPHANT_E2E_HIDE_WINDOW=1 pnpm exec playwright test tests/app/e2e/drawing-visual-regressions.spec.js --workers=1`, 2 passed; the Electron windows were hidden and closed by the test harness.
- Final library-shell proof after the same changes: `ELEPHANT_E2E_HIDE_WINDOW=1 pnpm exec playwright test tests/app/e2e/library-shell-regressions.spec.js --workers=1`, 3 passed; the Electron windows were hidden and closed by the test harness.
- Legacy migration red/green proof: `/tmp/elephant-legacy-drawing-red.log` failed before the classifier change; `/tmp/elephant-legacy-drawing-green.log` passed after it.
- Final renderer build: `/tmp/elephant-tauri-web-build-final-2.log`, build succeeded; existing Muya dead-code and large-chunk warnings remain visible in the build output.
- Targeted ESLint passed with `--max-warnings=0`; Node emitted the repository's existing module-type warning.

## Release Notes
- [ ] Not released; no commit or push requested for this bug fix.

## Risks
- [ ] Existing dirty worktree contains unrelated user changes; only the relevant production files and regression tests may be changed.
- [ ] Packaged Tauri and non-macOS platform proof remains separate.

## Rollout
- [x] Run red focused tests, apply the smallest fix, rerun green focused tests, then relevant unit/E2E/lint checks. Close every test process.
