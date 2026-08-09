---
name: elephantnote-ui-parity
description: >
  Rules for matching Electron/Tauri/mobile UI behavior, layout, window chrome,
  theme, scroll, graph/chat/search/editor surfaces, and screenshot verification.
argument-hint: '<ui-task>'
---

# ElephantNote UI Parity

Use this skill for UI behavior, visual parity, layout bugs, theme issues, graph/search/chat/editor pages, and Electron/Tauri differences.

## Invariants

- Electron and Tauri should feel like the same app unless a platform difference is intentional.
- Top bars, draggable areas, app icon, sidebar toggle, search bar height, and scroll behavior must be checked visually when touched.
- Fix layout at the shared component/style layer when possible.
- Do not hide a broken state behind a loading spinner.
- Mobile/desktop differences must be explicit and testable.

## Read first

- Component touched plus parent shell/layout components.
- Shared CSS in `Elephant/frontend/app/styles/**`.
- Store/service driving the UI state.
- Existing render/helper tests.

## Verification

- Component/unit test for stateful behavior when possible.
- Runtime screenshot or manual visual confirmation for layout/window chrome changes.
- Electron/Tauri parity check when a bug exists only in one runtime.
- Confirm scrolling and resizing behavior, not only initial render.

## Anti-slop

No pixel patch in one component when a shared layout wrapper is wrong. No infinite loading state as an error handler.
