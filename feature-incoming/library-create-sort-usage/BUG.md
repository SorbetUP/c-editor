# Library create and sort usage paths

- Request: Enlarge the floating create square, verify all sort and view modes, add automatic UI usage coverage, then commit and merge the fix.
- Level: high
- Date: 2026-08-10
- Slug: library-create-sort-usage

## Summary
- [x] The create control was only 44px square and was too small for the requested floating action.
- [x] Existing usage coverage checked sort state attributes but did not verify the resulting card order.
- [x] The E2E bridge did not implement `tauri_folders_create`, so the real folder creation path could not be exercised.

## Repro Steps
- [x] Launch the hidden Electron application with the seeded vault.
- [x] Measure the `Create` button: the pre-fix result was 44px, below the new 56px minimum.
- [x] Cycle the sort control through newest, oldest, title A-Z and title Z-A, then switch grid/list.
- [x] Select Folder from Create: the pre-fix E2E path failed with an unhandled `tauri_folders_create` invoke.

## Environment
- [x] ElephantNote 0.1.0, macOS Electron UI harness, hidden-window mode.
- [x] Final source branch: `develop` (the reviewed library changes were already present on this branch before the bounded E2E bridge/test commit).

## Observed vs Expected
- Observed: the create target was 44px; sort coverage could pass without proving card order; folder creation was unavailable in the automatic usage bridge.
- Expected: a 56px square create target, four sort modes with correct visible order, two working view modes, and real note/folder creation through the production UI path.

## Hypotheses
- [x] The create dimensions were still the previous compact-toolbar values.
- [x] The test asserted state labels instead of the user-visible ordering.
- [x] The preload switch lacked the production folder-create command.

## Investigation Plan
- [x] Inspect the current branch, dirty worktree, toolbar, store sorting and existing UI tests.
- [x] Reproduce the old 44px assertion and the missing folder-create invoke.
- [x] Run the corrected real Electron usage paths after the fix.

## Fix Plan
- [x] Increase the create control from 44px to 56px and enlarge its icon proportionally.
- [x] Assert actual card order for all four sort states and distinct grid/list geometry.
- [x] Add the missing E2E folder-create bridge using the same unique-path and directory semantics as Tauri.
- [x] Add a real UI usage path for folder creation followed by note creation.

## Regression Tests
- [x] Create menu exposes Note, Folder and Drawing with the real Excalidraw asset.
- [x] Create button is square and at least 56px in the real UI.
- [x] Folder and note creation persist through the production UI path.
- [x] Sort order is verified for updated-newest, updated-oldest, title A-Z and title Z-A.
- [x] Grid and list modes are verified through their rendered layout, not only a data attribute.

## Release Notes
- [x] Focused UI, visual, shell, unit and Rust validation completed on the final source state; commit and merge are recorded below.

## Risks
- [x] The worktree contains unrelated existing changes; only the bounded toolbar, E2E bridge, regression test and bug report files will be staged.
- [ ] Packaged Tauri and non-macOS platform proof remains separate.

## Rollout
- [x] Run the final focused UI tests, visual UI tests, shell tests, full unit suite, build, lint and diff checks.
- [x] Commit the bounded changes and confirm the reviewed commit is already on `develop`.

## Final Evidence
- [x] Focused usage UI: 5 passed, including the 56px square create control, folder+note creation, all four sort orders, grid/list rendering, and drawing usage.
- [x] Full library usage UI: 22 passed.
- [x] Visual drawing/library regressions: 2 passed; screenshots retained under `/tmp/elephant-visual-evidence/`.
- [x] Library shell regressions: 3 passed.
- [x] Unit suite after aligning the stale addon expectation with the current Rust editor runtime: 173 files passed, 27 skipped; 3237 passed, 171 skipped. Log: `test-results-observability/2026-08-10T16-37-23-563Z-vitest-unit-60552.log`.
- [x] Folder preview Rust regression: 1 passed, 1224 filtered out.
- [x] Renderer build and focused ESLint completed successfully; existing Vite/vendor-size warnings remain outside this bounded change.
- [x] Final branch: `develop` at the reviewed commit; no unrelated dirty files were staged.
