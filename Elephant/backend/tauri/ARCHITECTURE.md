# Tauri backend architecture

Goal: one shared renderer, one modular Rust backend, no separate native Java clone.

Active modules:

- lib_min.rs: temporary Tauri command registry.
- vault/: active modular vault backend.
- vault/types.rs: vault descriptors and config domain types.
- vault/config.rs: app-level vault registry read/write and active vault selection.
- vault/metadata.rs: hidden vault metadata initialization.
- vault/entries.rs: visible vault content operations.
- vault/commands.rs: Tauri command wrappers for vault operations.
- vault_layout.rs: canonical hidden/visible vault layout contract.
- markdown/: active Rust Markdown engine replacing the deterministic parts of Muya.
- markdown/types.rs: parsed document model.
- markdown/parser.rs: frontmatter, blocks, outline, links, images and tasks parser.
- markdown/renderer.rs: safe HTML and plain-text renderer.
- markdown/commands.rs: Tauri command wrappers for Markdown operations.
- markdown_engine.rs: old simple Markdown metadata helper kept temporarily.
- path_utils.rs: generic path helpers.
- note_domain.rs: note naming and Markdown creation.
- folder_domain.rs: folder path primitives.
- drawing_domain.rs: drawing scene primitives.
- media_domain.rs: media detection primitives.
- model_domain.rs: model selection primitives.
- search_logic.rs: search query and scoring primitives.

Compatibility modules: none. Dead duplicate glue (`lib.rs`, `vault_backend.rs`, `vault_min.rs`, `chat_runtime_local.rs`, `model_safety.rs`, `sync_safety.rs`, `vault_lib.rs`) was deleted during the Rust-only migration.

Rule: new behavior goes into a small domain module with tests first. Tauri commands should only call those modules.

Future folders: core, notes, folders, drawings, media, models, search, sync, commands.

Coverage rule: domain modules must keep at least 90 percent line coverage. Temporary command glue is excluded from the first coverage gate until it is split into smaller testable files.
