# Bug: duplicate AI chat surface and addon/core visual drift

## Symptoms

- The AI chat panel is rendered twice side-by-side when the Chat addon is enabled.
- The addon panel uses a different header, empty state and composer from the former core chat interface.

## Reproduction

1. Enable `elephant.ai-chat`.
2. Open the AI chat sidebar.
3. Observe two identical chat panels.

## Root cause

The addon contribution registry accepted duplicate contribution IDs. The shell then rendered every `shell.right` entry, so a duplicate `elephant.ai-chat.sidebar` contribution mounted the same physical chat component twice.

## Fix checklist

- [x] Ignore duplicate contribution IDs within the same addon and extension point.
- [x] Deduplicate shell-right rendering defensively.
- [x] Align the addon chat markup and styling with `ChatView.vue`.
- [x] Add a regression unit test for duplicate contribution registration.

## Verification

`pnpm test:unit:raw` passed: 168 files passed, 3212 tests passed, 27 files and 171 tests skipped.

`pnpm tauri:web:build` passed, including addon synchronization, Muya WASM generation and the Vite renderer build.

The addon JavaScript entrypoints also pass `node --check`.
