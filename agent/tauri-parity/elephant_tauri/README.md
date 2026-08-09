# Elephant Tauri workspace

This folder is the migration workspace for the Tauri/Rust version of ElephantNote.

The Electron application remains the reference implementation during the migration. The Tauri implementation must be built and tested here before replacing Electron behavior.

## Goals

- Keep the Electron application stable.
- Move deterministic application logic to Rust modules.
- Compare Electron behavior and Tauri behavior with parity fixtures.
- Avoid touching Electron renderer code unless a Tauri bridge API is ready and tested.

## Current source map

The active Tauri app still uses the root-level Tauri files for compatibility with the existing scripts:

- `src-tauri/`: active Rust/Tauri crate.
- `src/renderer/src/platform/tauriElephantNoteBridge.js`: active Tauri renderer bridge.
- `elephant_tauri/parity/`: parity fixtures and expected behavior contracts.

Once the Tauri version is stable, the active crate can be moved under this folder in a dedicated follow-up migration.

## Rule

Every feature migrated from Electron to Tauri should get a parity fixture here first. The Rust result must match the intended Electron behavior before the UI is switched to the Rust API.
