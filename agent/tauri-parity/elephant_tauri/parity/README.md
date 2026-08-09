# Tauri parity tests

This folder stores behavior fixtures used to compare the Electron implementation with the Tauri/Rust implementation.

## Markdown parity

`markdown_cases.json` is the first contract file. It describes Markdown inputs and the expected behavior that both engines must preserve:

- frontmatter extraction;
- outline extraction;
- task extraction;
- link extraction;
- GFM rendering;
- code block rendering.

## Migration rule

Before replacing an Electron feature with a Rust/Tauri feature, add a fixture here that describes the expected behavior.

The current Rust unit tests validate the Rust side. The next step is a small Node/Vitest harness that runs the Electron-side implementation against the same fixtures, then compares it with the Rust command output.
