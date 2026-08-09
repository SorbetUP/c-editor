---
name: rust-tauri-runtime
description: >
  Rules for Rust, Cargo, Tauri command handlers, path safety, serde payloads,
  error propagation, tests, and desktop/mobile runtime boundaries.
argument-hint: '<rust-tauri-task>'
---

# Rust/Tauri Runtime

Use this skill when work touches `Elephant/backend/tauri`, Rust commands, Cargo manifests, Tauri config, capabilities, filesystem operations, sync backends, or mobile/desktop runtime behavior.

## Read first

- `Elephant/backend/tauri/Cargo.toml` and the closest Rust module.
- Command registration in the Tauri library entrypoint.
- Existing unit tests in the touched Rust module.
- Tauri capabilities/config when command access changes.
- JS bridge tests when a Rust command is called from the renderer.

## Implementation rules

- Return explicit errors with enough context for the UI/logs.
- Keep path validation centralized and conservative.
- Do not broaden capabilities to make one command work.
- Keep serialization stable for JS callers.
- Prefer small pure helpers around filesystem logic so tests can verify disk state.
- Avoid panics in command paths that can be reached from user input.

## Validation commands

Use the narrowest relevant command:

```bash
cargo check --manifest-path Elephant/backend/tauri/Cargo.toml --all-targets --no-default-features
cargo test --manifest-path Elephant/backend/tauri/Cargo.toml --lib --no-default-features
```

Add `pnpm tauri:web:build` or packaged-app checks when renderer/runtime integration is in scope.

## APEX integration

- Analyze: trace renderer bridge to Rust command to filesystem/runtime result.
- Plan: include Rust unit tests and JS bridge tests when both sides change.
- Execute: keep error handling explicit and path safety intact.
- Validate: run cargo check/tests and any required bridge tests.
- eXamine: check for unsafe path handling, fake success JSON, and widened capabilities.
