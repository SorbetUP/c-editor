# Repository baseline

## Revision and working tree

- Migration branch: `nsb/freya-native-migration`
- Baseline `HEAD`: `62a606c145fe930682efbe491980c20ffeade14c`
- Product baseline branch: `develop` at the same commit
- New Freya work is uncommitted; the working tree contained pre-existing modifications and deletions before this task
- No reset, stash, cleanup, or deletion of unrelated user work was performed

The pre-existing tree includes removal of the historical Avalonia implementation, changes in Tauri/Vue/add-on paths, untracked `feature-incoming/` work and observability artifacts. Those changes are not attributed to the Freya slice.

## Provenance

| Surface | Current source | Historical source checked | Decision |
|---|---|---|---|
| shell/navigation | `Elephant/frontend/app/components/shell`, `Elephant/frontend/src/renderer/src` | current `develop` tree and runtime screenshot | keep as oracle; port contractually |
| vault visibility | `Elephant/backend/tauri/src/vault_layout.rs` | current Tauri backend | reuse predicate from the existing source |
| Rust editor domain | `Elephant/crates/muya-core` | `feature/rust-muya-complete@805041a75`, `feature/rust-muya-editor-core@b8d6c8fe9` | reuse candidate; not wired to Freya yet |
| native historical UI | deleted `Elephant/avalonia/` in working tree | `7b58d07f6`, `b04c29781`, `febd9d208`, `4e1ff499e` | audited as source material; not silently restored or rewritten |
| Freya | none in repository history | no Freya branch or commit found | new separate crate is required |

## Baseline runtime evidence

The Tauri/Vue oracle built and launched on macOS before the Freya slice. Unit tests passed (`3 238` passed, `171` skipped). The detailed commands and artifacts are in [`baseline/README.md`](baseline/README.md). The shell migration itself remains partial; no source deletion is justified by parity evidence.
