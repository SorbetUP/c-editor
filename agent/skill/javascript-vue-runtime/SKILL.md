---
name: javascript-vue-runtime
description: >
  Rules for JavaScript, Vue, Vite, Vitest, Playwright, frontend runtime bridges,
  state stores, async UI flows, and browser/Electron/Tauri renderer behavior.
argument-hint: '<js-vue-task>'
---

# JavaScript/Vue Runtime

Use this skill when work touches JavaScript, Vue components, Vite config, Vitest tests, Playwright flows, renderer bridges, stores, or browser-like runtime behavior.

## Read first

- `package.json` scripts and dependency versions.
- The closest component, store, or service with similar behavior.
- Existing Vitest or Playwright tests for the touched surface.
- Runtime bridge code when a component calls Electron or Tauri APIs.

## Implementation rules

- Keep renderer code deterministic with explicit state transitions, clear async boundaries, and visible error states.
- Do not hide backend failure behind optimistic UI success.
- Prefer existing composables, stores, helpers, and domain clients.
- Keep Vue component changes focused; move pure logic into helper functions only when tests benefit.
- Avoid adding dependencies for simple state, parsing, or formatting work.

## Testing rules

- Pure helpers: Vitest unit tests.
- Component behavior: test user-visible state and emitted or called domain action.
- Bridge calls: verify command or action names and payloads.
- UI workflows: use Playwright or runtime evidence when unit tests cannot prove the behavior.

## APEX integration

- Analyze: trace component to store/service to bridge/backend.
- Plan: choose unit, component, or workflow proof.
- Execute: keep async state honest and errors visible.
- Validate: run the narrow Vitest or Playwright command.
- eXamine: check for fake success states and stale loading loops.
