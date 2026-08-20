# Draw contract provenance

This crate is a vendored public-build compatibility slice of the private `SorbetUP/Draw` repository.
It contains the exact `DrawingScene`, `DrawingElement`, `Viewport`, geometry and color contract consumed by Elephant/Freya.

Canonical development stays in `SorbetUP/Draw`.

- source repository: `SorbetUP/Draw`
- source commit: `7a802d4e8f1d38557c63a0f282ddf6cc9676a746`
- mirrored source files: `src/document.rs`, `src/geometry.rs`

The full editor/history/SVG implementation is intentionally not duplicated here because Freya does not import those symbols through this dependency. Changes to the shared scene/geometry contract must be authored in Draw first and mirrored here in the same Elephant commit that consumes them.
