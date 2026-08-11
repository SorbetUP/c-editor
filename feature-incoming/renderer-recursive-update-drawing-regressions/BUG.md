# renderer recursive update and drawing regressions

- Request: Fix recursive renderer updates, broken checkboxes, saved drawing navigation failure, Excalidraw control placement, white scroll border, and real grid/list modes; add UI regression coverage.
- Level: high
- Date: 2026-08-09
- Slug: renderer-recursive-update-drawing-regressions

## Summary
- [x] Renderer reports repeated `Maximum recursive updates exceeded` rejections while the editor/library UI is active.
- [x] Task checkboxes render as a broken/native-looking control and do not reliably persist their state.
- [x] A drawing created, named, saved, and then opened from the library is not a stable user path.
- [x] Excalidraw header actions overlap the Library area and the workspace shows a white page-like strip while scrolling.
- [x] Library grid/list modes are currently only a class variation and need distinct Google Keep-style layouts.

## Repro Steps
- [x] Start the app with a clean test vault and open a note containing a task list.
- [x] Toggle a task checkbox and inspect the rendered control and saved Markdown.
- [x] Use Create → Drawing, enter a name, save, return to the library, and click the resulting drawing card.
- [x] Scroll the library/editor shell and inspect the top edge for a white border/page background.
- [x] Toggle the library view mode and compare card structure, density, and metadata placement.

## Environment
- [x] ElephantNote 0.1.0 worktree on macOS/Electron E2E runtime.
- [x] Renderer log supplied by the user: `Maximum recursive updates exceeded` and `uncaught non-error event`.
- [x] Fresh reproduction: `/tmp/elephant-drawing-flow-current.log` records React errors 153/168; the diagnostic overlay intercepts the name prompt's Save click and the E2E test times out.
- [x] Required proof: real UI path plus focused unit/E2E regression tests.

## Observed vs Expected
- Observed: Vue emits unhandled recursive-update rejections; task controls are visually broken; saved drawings can fail on the library-to-editor transition; Excalidraw controls overlap; scrolling exposes a white strip; grid/list lacks structural distinction.
- Expected: no unhandled renderer rejection, square accessible checkboxes with persisted toggling, a saved drawing opens reliably and exposes its edit action, controls stay inside the Excalidraw surface, all shell edges use the application background, and grid/list have clear independent layouts.

## Hypotheses
- [x] A watcher or render callback mutates one of its own reactive dependencies, possibly around editor/Excalidraw lifecycle or a layout synchronization watcher.
- [x] Checkbox DOM ownership and delegated event handling are split between Rust Muya and legacy Muya styles/handlers.
- [x] The saved drawing path/type/preview metadata is not normalized consistently between creation, directory refresh, card click, and Excalidraw opening.
- [x] A global body margin or shell surface border is leaking through during scroll.

## Investigation Plan
- [x] Reproduce each path in the existing Electron/Tauri acceptance harness and capture renderer logs.
- [x] Search watchers, lifecycle hooks, drawing path normalization, checkbox DOM contracts, and layout CSS.
- [x] Add tests that fail against the observed defect before changing production code.

## Fix Plan
- [x] Make reactive synchronization idempotent and guard lifecycle callbacks against re-entrant updates.
- [x] Give task checkboxes one DOM/style/event contract and persist the exact checked state.
- [x] Normalize drawing note metadata and open the saved asset through the real Excalidraw path.
- [x] Constrain Excalidraw actions to its header, reset shell edge backgrounds/borders, and implement explicit grid/list layout rules.

## Regression Tests
- [x] Unit test for the reactive guard and checkbox DOM/event contract.
- [x] UI test for checkbox toggle plus persisted Markdown.
- [x] UI usage test for create → name → save drawing → library card → open/edit.
- [x] UI contract/visual geometry test for no white top scroll edge and distinct grid/list layouts.

## Release Notes
- [x] macOS/Electron real UI paths and focused suites pass on the final state; packaged and other-platform proof remains separate.

## Risks
- [ ] Existing dirty worktree contains unrelated user changes; do not reset or broad-format it.
- [ ] Native Windows/Linux/macOS/Android packaged proof remains separate from Electron UI proof.

## Rollout
- [x] Run focused red/green tests first, then unit, E2E, lint, renderer build, and the relevant desktop acceptance scenario.

## Evidence

- Red drawing flow: `/tmp/elephant-drawing-flow-current.log` (React errors 153/168; diagnostic overlay intercepted Save).
- Red recursive rewrite loop: `/tmp/elephant-drawing-flow-after-excalidraw-guard.log` (repeated root-asset rewrite events).
- Green drawing flow: `/tmp/elephant-drawing-flow-final.log` and `/tmp/elephant-critical-ui-final.log` (`2 passed` for drawing plus checkbox after the final renderer build).
- Green full UI regression suite: `/tmp/elephant-ui-regressions-final.log` (`19 passed (44.7s)`).
- Green product check: `/tmp/elephant-prod-check-current-final-2.log` (`62 passed (2.6m)`, observable E2E exit code `0`, licence groups valid, unpack build completed).
- Green focused Vitest: `48 passed` across five files.
- Green renderer build: `/tmp/elephant-tauri-web-build-final-2.log`.
- Green focused lint: no project-file warnings/errors; ESLint still emits its existing module-type startup warning. Full lint reports `0 errors` but retains 915 legacy warnings across the dirty repository.

## Follow-up evidence (2026-08-09)

- Red persisted-asset regression: `/tmp/elephant-asset-encoding-red.log` (`5 tests | 1 failed`) showed an Excalidraw asset path retaining repeated `%25` encoding instead of resolving to the vault file path.
- Green persisted-asset regression: `/tmp/elephant-asset-encoding-green-2.log` (`5 passed`) after bounded decoding and `asset:` URL normalization in `excalidrawImageRuntimeFixes.js`.
- Red complete UI suite: `/tmp/elephant-ui-feature-regressions-current.log` (`19 passed, 1 failed`); the remaining failure was the rename test targeting a pointer-transparent floating toolbar as if it were interactive.
- Green complete UI suite: `/tmp/elephant-ui-feature-regressions-final-current.log` (`20 passed (42.8s)`) after exercising outside-click cancellation through the library surface.
- Green library/shell contracts: `/tmp/elephant-library-shell-final-current.log` (`3 passed (4.8s)`).
- Green citation/add-on/rail audit: `/tmp/elephant-ui-request-coverage-final-current.log` (`5 passed (11.7s)`).

## Reopened verification (2026-08-09)

- [x] The recursive-update path was narrowed to repeated editor listeners and a global Excalidraw image observer; both now install/queue idempotently.
- [x] The Excalidraw image observer now ignores unrelated DOM/class mutations instead of scanning every image after every renderer mutation.
- [x] The Excalidraw close/save controls use Lucide icons and are offset from Excalidraw's Library control.
- [x] Citation E2E coverage now records renderer `pageerror` events and asserts that the selected cross-note citation path produces none.
- [ ] A fresh packaged Tauri run of the complete drawing + citation path on every target platform is still not proven from this macOS worktree.
- [ ] The pre-existing full E2E suite still has unrelated/known failures in mobile artifact handling and moving a note to `All notes`; those must not be reported as fixed by this bug work.

Current source evidence from the user-provided logs remains retained at:

- `/Users/sorbet/.codex/attachments/4125c451-49b9-4cf8-88c1-d3c8c41f4748/pasted-text.txt`
- `/Users/sorbet/.codex/attachments/5b07d4e4-e419-4f46-81c7-ffce74f8976f/pasted-text.txt`
