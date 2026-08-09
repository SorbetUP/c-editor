---
name: elephantnote-electron
description: >
  Electron-specific rules for legacy IPC, preload contracts, backend services,
  and parity with the Tauri runtime.
argument-hint: '<electron-task>'
---

# ElephantNote Electron

Use this skill for `Elephant/back`, Electron preload/main processes, legacy IPC, local shortcuts, and behavior that must remain compatible with Tauri.

## Read first

- Backend service file in `Elephant/backend/js/**`.
- IPC registration and preload/API contract.
- Frontend caller in `Elephant/frontend/app/services/**`.
- Shared contract in `Elephant/shared/**` when present.
- Matching Tauri command when the behavior exists in both runtimes.

## Rules

- Do not keep a privileged Electron-only path if Tauri uses a safer shared contract.
- Prefer shared validation and serialization helpers over duplicate runtime-specific copies.
- Keep filesystem behavior consistent with vault path safety rules.
- Do not use Electron as a hidden workaround for a broken Tauri implementation.
- Preserve existing local-shortcut/window behavior unless the task explicitly changes it.

## Verification

- Unit tests for service functions.
- IPC/preload serialization tests for bridge changes.
- Parity check against the Tauri client contract when both paths exist.

## Anti-slop

No silent catch that returns an empty result for a real backend failure. Surface useful errors and keep debug logs actionable.
