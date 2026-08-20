# Note domain contract provenance

This crate is a vendored public-build compatibility slice of the private `SorbetUP/Note` repository.
It mirrors the note metadata/tag/task domain API consumed by Elephant without requiring private-submodule credentials during public CI.

Canonical editor development stays in `SorbetUP/Note`; the full Muya-compatible Rust engine is mirrored separately in `Elephant/crates/muya-core`.

- source repository: `SorbetUP/Note`
- source branch: `main`
- mirrored source files: `src/lib.rs`, `src/document.rs`, `src/markdown.rs`

Changes to these files must be authored in Note first and mirrored into Elephant in the same integration change.
