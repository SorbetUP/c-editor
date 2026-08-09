---
name: tauri-ci-verifier
description: >
  Verify Tauri builds, Rust commands, packaged app launch, bridge contracts,
  capabilities, icons, and cross-platform runtime behavior.
argument-hint: '<tauri-ci-task>'
---

# Tauri CI Verifier

Use this skill when a task touches Tauri, Rust commands, packaging, app launch, bridge code, filesystem persistence, or desktop/mobile parity.

## Required checks

- Run the Tauri web build when renderer/runtime code changes.
- Run `cargo check` and Rust tests for `Elephant/backend/tauri` when Rust commands or config change.
- Use the packaged macOS smoke command when the task is about whether the built app opens a visible window.
- Verify bridge serialization with a JS contract test when frontend calls Rust commands.
- Verify real disk state when the task touches vault files, assets, dashboard files, or sync state.

## ElephantNote invariants

- Tauri must not silently fall back to legacy behavior that only pretends to work.
- Bridge methods must return explicit unavailable errors when the runtime is missing.
- File writes must use the guarded backend path.
- Hidden app folders such as `.assets`, `.dashboard`, and `.elephantnote` must stay out of the normal note tree.
- A successful web build alone is not proof that the packaged app opens.

## APEX integration

- Analyze: map the visible flow to JS bridge, Rust command, config, and test.
- Plan: add one real assertion at each boundary touched.
- Execute: patch implementation and test together.
- Validate: run the exact Tauri command set or report the missing platform runner.
- eXamine: check the diff for fake fallbacks, broadened runtime access, and missing disk-state assertions.
