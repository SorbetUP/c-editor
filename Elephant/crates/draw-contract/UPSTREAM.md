# Draw contract provenance

This crate is a vendored compatibility slice of the private `SorbetUP/Draw` repository. It exists so Elephant/Freya can compile and test without requiring cross-repository credentials at build time while canonical development stays in Draw.

- source repository: `SorbetUP/Draw`
- source commit: `b1d3ee8dae3c4f6ec7eb26c5cce9f003a7974cf6`
- mirrored source files: `src/document.rs`, `src/geometry.rs`, `src/history.rs`, `src/tool.rs`
- mirrored symbol: `create_element` from `src/editor.rs`, kept in `src/factory.rs` here to avoid vendoring the full editor runtime

The mirror is deliberately small. Freya owns native widget/event adaptation while scene semantics and newly-created element defaults remain sourced from Draw.

Any change to a mirrored contract must be authored in `SorbetUP/Draw` first, then copied here with this source commit updated in the same Elephant branch. The contract tests must continue to assert the Excalidraw reconciliation metadata and history semantics relied on by Freya.
