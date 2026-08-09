---
name: elephantnote-tauri
description: >
  Tauri-specific rules for Rust commands, filesystem safety, window behavior,
  frontend bridge calls, and macOS/dev build verification.
argument-hint: '<tauri-task>'
---

# ElephantNote Tauri

Use this skill for `Elephant/backend/tauri`, Tauri invokes, Rust command registration, Tauri window behavior, permissions, and Tauri dev/build failures.

## Read first

- `Elephant/backend/tauri/src/**/*.rs` touched by the command.
- `Elephant/backend/tauri/tauri.conf.json` and capabilities when permissions are involved.
- Frontend caller in `Elephant/frontend/app/services/elephantnoteClient*` or component/store using the command.
- Existing Rust and JS tests for the command family.

## Rules

- Register every exposed command in the Tauri command handler and test that it is reachable.
- Validate vault paths with canonical root containment before reading, writing, moving, deleting, or scanning.
- Do not broaden filesystem scopes to make a test pass.
- Keep binary files out of text reads; directories must not be opened as notes.
- Preserve macOS window behavior: draggable top bar, visible window after build/dev, stable app icon.
- Keep Rust warnings clean when the fix is local and safe.

## Verification

- Rust tests for path safety and command helpers.
- JS/TS contract test for invoke payload/response when bridge shape changes.
- `pnpm tauri:dev` or the narrowest build command when the task is window/bootstrap related.
- Logs from Rust/backend when a bug is a loading loop or command silently fails.

## Anti-slop

No fallback that only makes the UI stop crashing while disk state is wrong. No fake `ok: true` bridge path.
