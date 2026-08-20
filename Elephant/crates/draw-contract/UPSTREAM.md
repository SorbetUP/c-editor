# Draw contract provenance

This crate is a vendored compatibility slice of the private `SorbetUP/Draw` repository. It exists so Elephant/Freya can compile and test without requiring cross-repository credentials at build time while canonical development stays in Draw.

- source repository: `SorbetUP/Draw`
- source commit: `0bb1a17aee152eb7fbd983a9901b1823288b0d73`
- mirrored source files: `src/binding.rs`, `src/document.rs`, `src/geometry.rs`, `src/history.rs`, `src/scene_ops.rs`, `src/selection.rs`, `src/tool.rs`
- mirrored symbol: `create_element` from `src/editor.rs`, kept in `src/factory.rs` here to avoid vendoring the full editor runtime

The mirror is deliberately small. Freya owns native widget/event adaptation while canonical Draw owns scene mutations, Excalidraw binding metadata, selection geometry and newly-created element defaults.

Any change to a mirrored contract must be authored in `SorbetUP/Draw` first, then copied here with this source commit updated in the same Elephant branch. The contract tests must continue to assert the Excalidraw reconciliation metadata, bindings, selection geometry and history semantics relied on by Freya.
