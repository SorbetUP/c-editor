# Elephant Freya shell

This crate is the first native migration slice. It is intentionally a separate
binary while the Tauri/Vue application remains the behavioral oracle.

Run it with a real vault:

```bash
ELEPHANT_FREYA_VAULT=/absolute/path/to/vault cargo run --manifest-path Elephant/freya/Cargo.toml
```

The loader reuses the existing Tauri vault visibility rules from
`Elephant/backend/tauri/src/vault_layout.rs`. It reads real Markdown files and
surfaces filesystem failures in the native UI. It does not claim parity for the
editor, graph, settings persistence, addons, sync, or mobile runtime yet.
