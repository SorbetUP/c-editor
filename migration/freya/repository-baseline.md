# Freya migration repository baseline

Baseline source is `develop` at `62a606c145fe930682efbe491980c20ffeade14c`. The active worktree already contained unrelated user changes before the migration branch was created; those changes remain untouched.

The old Vue/Tauri renderer remains the behavior oracle. Relevant source paths are:

- `Elephant/frontend/app/components/shell/AppShell.vue`
- `Elephant/frontend/app/components/navigation/IconRail.vue`
- `Elephant/frontend/app/components/navigation/SidebarNav.vue`
- `Elephant/frontend/app/components/library/LibraryToolbar.vue`
- `Elephant/frontend/app/components/library/LibraryGrid.vue`
- `Elephant/frontend/app/components/library/NoteCard.vue`
- `Elephant/frontend/app/components/library/CreateEntryMenu.vue`
- `Elephant/frontend/app/components/shell/NoteEditorHost.vue`
- `Elephant/backend/tauri/src/vault/entries.rs`
- `Elephant/crates/muya-core`

No pre-existing valid `Elephant/freya` implementation was found in branch history. A first manual shell attempt was reverted after review because it was not a source conversion. The current files are a new adapter layer only where the source contract and production Rust path are explicit.

## Current evidence

| Surface | Evidence | State |
| --- | --- | --- |
| Freya crate | `cargo check` | PROVEN to compile on macOS arm64 |
| Rust contracts | 49 focused unit tests | PROVEN at contract/adapter level |
| Native shell | `freya-testing` semantic click Create → Note → filesystem | PARTIALLY PROVEN |
| visual parity | no before/after screenshot pair yet | NOT PROVEN |
| packaged app | no packaged Freya target yet | NOT PROVEN |
| Android | no Freya Android target yet | NOT PROVEN |

## Non-goal of this baseline

The old Vue code is not deleted, hidden, or replaced by a WebView wrapper. It remains runnable until each surface has its own runtime and visual proof.
