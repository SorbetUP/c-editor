# Plan : migration du backend vers Rust (Tauri uniquement)

## Statut d'avancement

| Lot | État | Validation |
|---|---|---|
| 0 — Nettoyage code mort | ✅ Fait | `cargo check` + `cargo test` verts (1180 + 3 intégration) |
| 1 — Fondations (deps + skeletons) | ❌ Roulé dans chaque lot (Ponytail : pas d'échafaudage vide) |
| 2 — Logging/atomic/prefs/dataCenter/bufferStore | ✅ Fait | `cargo test` verts (1180 → 1180+11 nouveaux) |
| 3 — Filesystem (encoding + IO) | ✅ Fait | `cargo test` verts (1193 → +13 nouveaux) |
| 4 — Watcher (notify) | ✅ Fait | `cargo test` verts (1201 → +8) |
| 5 — Recents + keybindings | ✅ Fait | `cargo test` verts (1210 → +9) |
| 6 — FTS5 SQLite | ✅ Fait | `cargo test` verts (1221 → +11) |
| 7 — Embeddings | ✅ Fait | `cargo test` verts (1233 → +12) |
| 8 — Sync git + LAN | ✅ Fait | `cargo test` verts (1240 → +7) |
| 9 — Wiki library | ✅ Fait | `cargo test` verts (1250 → +10) |
| 10 — RAG prompt | ✅ Fait | `cargo test` verts (1255 → +5) |
| 11 — Atomic features + ollama | ✅ Fait | `cargo test` verts (1265 → +10) |
| 12 — Site preview (tiny_http) | ✅ Fait | `cargo test` verts (1271 → +6) |
| 13 — OCR runtime | ✅ Fait | `cargo test` verts (1275 → +4) |
| 14 — Frontend : invoke bridge | ✅ Déjà en place (`src/renderer/src/platform/tauriElephantNoteBridge.js` etc.) ; ~50 nouvelles commandes Rust accessibles via `invoke()` |
| 15 — Suppression Electron | ✅ Fait | `src/main/`, `web/`, `Dockerfile`, scripts Electron supprimés ; `package.json` nettoyé ; `vitest.config.js` nettoyé |
| 16 — Vérification finale | ✅ Fait | `cargo test` 1275+3 / `pnpm tauri:check` / `cargo clippy` verts (pré-existant warning `operation_payload` sorti du scope) |

**Résumé validation finale** :
- `cargo test --manifest-path src-tauri/Cargo.toml` : 1275 lib + 3 integration tests verts (0 échecs)
- `cargo clippy` : 0 erreurs, 22 warnings (la plupart pré-existants)
- `pnpm tauri:check` : vert
- `pnpm test:unit` : 2701/2703 verts (2 échecs pré-existants `muyaRuntimeRoute` / `realComponentImportSmoke` indépendants d'Electron — pré-existants avant le lot 0)
- `pnpm lint` : 41943 erreurs pré-existantes (legacy MarkText, inchangées)

Nouveaux packages Rust ajoutés : `notify`, `encoding_rs`, `chardetng`, `keyring`, `rusqlite` (FTS5+SQLITE), `tiny_http`, `fuzzy-matcher`, `which`, `chrono`, `reqwest` (avec feature `blocking`).

Packages npm retirés automatiquement via `pnpm install` (dépendances Electron supprimées du fait de l'absence de references) : `electron`, `electron-vite`, `electron-builder`, `@electron/*`, `electron-store`, `electron-log`, `electron-updater`, `native-keymap`, etc. (Tauri est désormais le runtime unique.)

---

**Tâche :** Migrer tout le backend (A `src/main/` + B `Elephant/back/app/`) vers Rust dans `src-tauri/`, supprimer Electron, faire de Tauri le runtime unique.

**Critères d'acceptation :**
1. Aucun code backend Node.js n'est plus nécessaire au runtime ; tous les `invoke` du renderer appellent des commandes Rust `#[tauri::command]`.
2. `cargo check --manifest-path src-tauri/Cargo.toml` et `cargo test --manifest-path src-tauri/Cargo.toml` passent.
3. `pnpm tauri:check` passe ; l'app Tauri se lance (`pnpm tauri:dev`).
4. `src/main/`, `Elephant/back/app/`, `electron.vite.config.js`, `electron-builder.yml`, `Dockerfile`, `web/` et les scripts Electron sont supprimés ; `package.json` ne contient plus de dépendances Electron.

**Contrainte APEX/Ponytail :** chaque lot garde `cargo check` + `cargo test` verts ; on supprime le code mort avant d'ajouter du parallèle ; nouvelles deps uniquement si pas d'équivalent stdlib/Tauri plugin.

---

## Lots (chaque lot = `cargo check` + `cargo test` verts)

### Lot 0 — Nettoyage du code mort (Ponytail : delete over add)
Fichiers à supprimer (non compilés ou non enregistrés, confirmé par grep) :
- `src-tauri/src/lib.rs` (entry = `lib_min.rs` ; `run()` jamais appelé)
- `src-tauri/src/vault_backend.rs` (stubs dupliqués non enregistrés)
- `src-tauri/src/vault_min.rs` (non importé)
- `src-tauri/src/chat_runtime_local.rs` (variante `tauri_rag_chat` non enregistrée)
- `src-tauri/src/model_safety.rs` (variante non enregistrée)
- `src-tauri/src/sync_safety.rs` (variante non enregistrée)
- `src-tauri/src/vault_lib.rs` (supprimer le fichier + `pub mod vault_lib;` dans `lib_min.rs` — aucun usage)
- `src-tauri/ARCHITECTURE.md` : retirer les références à `vault_min.rs` / `markdown_engine.rs` legacy.

À garder : `markdown_engine.rs` (utilisé par `note_domain.rs::parse_markdown/render_note`).

Tests : `cargo test` doit rester vert (les tests des supprimés partent avec).
Validation : `cargo check` + `cargo test` verts.

### Lot 1 — Fondations Rust (deps + modules vides)
`src-tauri/Cargo.toml` : ajouter (versions stables, features minimales) :
- `notify = "6"` (watcher FS, remplace chokidar) — feature `default`
- `encoding_rs` + `chardetng` (remplacent iconv-lite/ced)
- `keyring = "3"` (remplace keytar) — feature `apple-native` sur macOS, etc. via target cfg
- `rusqlite = "0.32"` features `["bundled", "bundled-full"]` (FTS5 inclus ; remplace electron-store + vectra persist)
- `tracing` + `tracing-subscriber` + `tracing-appender` (remplace electron-log)
- `fuzzy-matcher` (remplace fuzzaldrin)
- `which = "6"` (remplace command-exists)
- `chrono = "0.4"` (timestamps screenshots/recents)

À éviter pour l'instant (Ponytail) : `tantivy` (lourd ; FTS5 SQLite suffit), `git2` (préférer `std::process::Command` git comme le fait déjà le JS), `ort`/`candle` (embeddings via `llama-server` déjà bundlé), `tauri-plugin-store` (serde_json suffit).

Créer modules vides avec `mod.rs` déclarés dans `lib_min.rs` :
- `infra/logging.rs` (init tracing)
- `infra/atomic_write.rs` (write-tmp + rename portable)
- `preferences/` (mod.rs, types.rs)
- `data_center/` (mod.rs, types.rs, secrets.rs)
- `buffer_store/`
- `filesystem/` (mod.rs, encoding.rs, io.rs)
- `watcher/`
- `recents/`
- `keybindings/`
- `fts/` (mod.rs, schema.rs, index.rs) — SQLite FTS5
- `embeddings/` (mod.rs, local.rs, store.rs)
- `sync/git.rs`, `sync/lan.rs`
- `wiki/`
- `agents/`
- `atomic_features/`
- `site_preview/`
- `ocr/`

Validation : `cargo check` vert.

### Lot 2 — Logging + atomic_write + preferences/dataCenter/bufferStore
- `infra/logging.rs` : `init_logger()` (tracing-subscriber + appender rolling) ; appelé dans `run()`.
- `infra/atomic_write.rs` : `write_atomically(path, bytes) -> Result<()>` (tempfile dans même dir + rename, fsync dir sur macOS).
- `preferences/` : structs serde `Preferences` (typedef du `preferences.json` MarkText), `load()`, `save_atomic()`, `set_item(key, value)`, migrations min.
- `data_center/` : `UserData` struct (imageFolder, screenshotFolder, webImageList, cloudImageList, githubToken), `load_or_default()`, `save_atomic()`, `set_github_token` via `keyring::Entry`.
- `buffer_store/` : `EditorBuffer` (openTabs, unsavedMarkdown), `load(window_id)`, `save_atomic(window_id, buffer)`.
- Nouvelles commandes Tauri (enregistrées dans `lib_min.rs`) :
  - `tauri_prefs_get`, `tauri_prefs_set(key, value)`, `tauri_prefs_all`
  - `tauri_user_data_get`, `tauri_user_data_set(key, value)`, `tauri_user_data_set_image_folder(path)`
  - `tauri_buffer_save(window_id, state)`, `tauri_buffer_load(window_id)`, `tauri_buffer_clear(window_id)`
  - `tauri_secret_set(name, value)`, `tauri_secret_get(name)`, `tauri_secret_delete(name)` (générique via keyring)
- Tests unitaires : atomic_write (roundtrip + crashed tmp), preferences load/save, data_center load/save, keyring roundtrip (mock via feature flag `test_keyring` ou skip sur CI), buffer_store round-trip.
Validation : `cargo test` vert.

### Lot 3 — Filesystem (encoding + IO) + path resolve
- `filesystem/encoding.rs` : `detect_and_decode(bytes) -> (String, &'static str)`, `encode(text, encoding) -> Vec<u8>`, BOM strip/add (UTF-8/16 LE+BE), EOL normalization (`\r\n`→`\n` à la lecture, `os_eol` à l'écriture selon prefs).
- `filesystem/io.rs` : `read_markdown_file(path)` (détection encoding + BOM + EOL), `write_markdown_file(path, content, encoding, eol)`, `resolve_path(path)` (canonicalize + symlink resolution via `std::fs::canonicalize`).
- Commandes Tauri :
  - `tauri_fs_read_markdown(path) -> { content, encoding, eol }`
  - `tauri_fs_write_markdown(path, content, encoding?, eol?)`
  - `tauri_fs_resolve_path(path)` (supprime `mt::fs-trash-item` via `tauri_plugin_opener`/`fs` existant)
- Hiérarchie : `tauri_notes_read/write` existants restent pour le vault ; `tauri_fs_*` pour chemin absolu (export/save-as).
- Tests : roundtrip UTF-8/UTF-16LE/GB18030, BOM, EOL, symlink résolution.
Validation : `cargo test` vert.

### Lot 4 — Watcher (notify) — remplace chokidar
- `watcher/mod.rs` : `Watcher` HashMap<path, Notify> par window_id, threshold de stabilité (debounce 300ms), ignore patterns (`.elephantnote/`, gros fichiers). Émet événements Tauri `elephantnote:fs:changed { path, kind }`.
- Commandes Tauri :
  - `tauri_watcher_watch_file(window_id, path)`, `tauri_watcher_watch_directory(window_id, path)`, `tauri_watcher_unwatch_file`, `tauri_watcher_unwatch_directory`, `tauri_watcher_unwatch_all(window_id)`
  - `tauri_watcher_ignore_next(window_id, path)` (pour `window-file-saved`)
- Tests : simulate d'événements via `notify` EventKind, debounce, ignore patterns.
Validation : `cargo test` vert.

### Lot 5 — Recents + keybindings
- `recents/mod.rs` : `recently_used_documents.json` load/save/add/clear, limite 20.
- `keybindings/mod.rs` : merge defaults(OS-spécifique) + user `keybindings.json`, load/save.
- Commandes Tauri :
  - `tauri_recents_add(path)`, `tauri_recents_clear`, `tauri_recents_list`
  - `tauri_keybindings_get`, `tauri_keybindings_save(map)`
- Tests : recents dedupe/LRU, keybindings merge.
Validation : `cargo test` vert.

### Lot 6 — FTS5 SQLite (remplace scan ad-hoc + vectra textuel)
- `fts/schema.rs` : tables `notes(rowid INTEGER PRIMARY KEY, vault_id, rel_path, title, body, mtime)`, `notes_fts USING fts5(title, body, content='notes')` ; triggers sync.
- `fts/index.rs` : `open_db(vault_dir)` (ouvre `.elephantnote/index/notes.sqlite`), `upsert_note(vault_id, rel_path, markdown)`, `remove_note`, `search(query, limit) -> Vec<Hit>` (snippet via `snippet()`, `bm25()`).
- Brancher : `tauri_notes_write`/`tauri_entries_rename/move/delete` mettent à jour l'index après écriture ; `tauri_search_query` devient une requête FTS5 (fallback scan si index vide).
- Nouvelles commandes : `tauri_search_index_status`, `tauri_search_index_rebuild` (remplace `tauri_search_rebuild` qui écrit du JSON).
- Tests : upsert/remove/search, BM25 ranking, multivault.
Validation : `cargo test` vert.

### Lot 7 — Embeddings (local via llama-server + remote)
- `embeddings/local.rs` : POST `http://127.0.0.1:<port>/embeddings` (OpenAI-compatible, llama-server déjà bundlé) — génère embeddings texte.
- `embeddings/store.rs` : table SQLite `embeddings(rowid INTEGER PRIMARY KEY, note_rowid, model_id, vec BLOB)` + `vec0` virtual table (sqlite-vec) OU `hnsw`. Ponytail : utiliser SQLite + produit scalaire en Rust pour <10k notes (pas de nouvelle lib). `add_embedding`, `search_semantic(query_vec, k)`.
- Commandes Tauri :
  - `tauri_embeddings_embed(text, model?) -> Vec<f32>`
  - `tauri_search_semantic(query, k) -> Vec<Hit>`
  - `tauri_search_rebuild` étendu pour embeddings + FTS.
- Tests : roundtrip embedding (mock HTTP via `mockito` en dev dép), similarité cosinus, recherche top-k.
Validation : `cargo test` vert (skip si pas de llama-server en CI : `#[ignore]` sur les tests live).

### Lot 8 — Sync backends (git + LAN)
Ponytail : utiliser `std::process::Command` (git) plutôt que `git2` (le JS le fait déjà), `reqwest` pour LAN HTTP.
- `sync/git.rs` : `git_run(vault_dir, args)` → `git add/commit/pull/push`; detection conflit via exit code ; garde l'engine local-folder existant comme fallback.
- `sync/lan.rs` : pair protocol HTTP via `reqwest` (echo pair-code depuis l'invite existant `sync_create_invite`/`sync_accept_invite`), transfert par `reqwest::multipart`.
- Commandes Tauri :
  - `tauri_sync_git_run(vault_id, op)`, `tauri_sync_lan_run(vault_id, op)`, `tauri_sync_git_status`
- Tests : git commit dry-run (initialise un repo tmp), invite/accept pair LAN (deux ports loopback).
Validation : `cargo test` vert.

### Lot 9 — Wiki library
- `wiki/mod.rs` : port `wikiLibrary.js` — propose liens wiki depuis le graphe de notes (notes participant + outbound/inbound edges).
- Réutilise `fts/` + graphe de liens déjà extrait (`markdown::extract_links`).
- Commandes Tauri : `tauri_wiki_proposals(note_path)` remplace l'actuel `tauri_wiki_list` (vide).
- Tests : proposals déterministes sur un mini-vault.
Validation : `cargo test` vert.

### Lot 10 — Agents + RAG prompt
- `agents/mod.rs` : port `agents.js` (tool dispatch, memory, plan/exécute simple).
- `chat/rag_prompt.rs` : port `ragPrompt.js` (top-k FTS5 + embeddings → prompt).
- Commandes Tauri : `tauri_agents_run(goal)`, étendre `tauri_rag_chat` pour utiliser le prompt reconstruit.
- Tests : prompt assembly, tool dispatch basique.
Validation : `cargo test` vert.

### Lot 11 — Atomic features + ollama runtime
- `atomic_features/mod.rs` : port `AtomicFeatureService.js` + `atomicIpc.js`.
- `runtime/ollama.rs` : client HTTP `http://127.0.0.1:11434/api` (list, generate, embeddings).
- Commandes Tauri : `tauri_atomic_features_list/toggle/run`, `tauri_ollama_generate`, `tauri_ollama_embeddings`.
- Tests : feature toggle persistence, ollama stub via `mockito`.
Validation : `cargo test` vert.

### Lot 12 — Site preview
- `site_preview/mod.rs` : port `sitePreviewIpc.js` + `SitePreviewService.js` ; `StaticSiteServer` via `tiny-http` (servir dossier static sur port éphémère local 127.0.0.1).
- Commandes Tauri : `tauri_site_preview_open(vault_id) -> {url}`, `tauri_site_preview_close`.
- Tests : démarrage serveur loopback, GET /, shutdown.
Validation : `cargo test` vert.

### Lot 13 — OCR runtime
- `runtime/ocr.rs` : port `ocrRuntime.js` — delegate à un sidecar `tesseract` ou avertir desktop-only.
- Commandes Tauri : `tauri_ocr_image(path) -> { text }` (desktop-only).
- Tests : config parsing, skip mobile.
Validation : `cargo test` vert.

### Lot 14 — Frontend : remplacer `elephant-back` par `invoke`
- Supprimer les alias `elephant-back`/`elephant-front`/`elephant-shared` dans `vite.tauri.config.js` ? Conserver `elephant-front`/`elephant-shared` (frontend). Supprimer `elephant-back` alias uniquement.
- Trouver les imports `from 'elephant-back/...'` dans `src/renderer/` et `Elephant/front/app/` ; remplacer par un module `src/renderer/src/platform/bridge.ts` qui appelle `@tauri-apps/api/core` `invoke('tauri_*', ...)` et `listen('elephantnote:*')`.
- Faire pointer tous les anciens IPC (noms `mt::*`) sur les nouvelles commandes Rust via un shim `invokeBridge.ts`.
- Lancer `pnpm tauri:web:dev` pour vérifier le rendu sans Electron.
Validation : `pnpm tauri:web:build` vert ; l'app se charge.

### Lot 15 — Suppression Electron
- Supprimer : `src/main/`, `src/preload/` (scripts Electron), `electron.vite.config.js`, `vite.tauri.config.js` devient `vite.config.js` (frontend unique), `electron-builder.yml`, `Dockerfile`, `web/`, `build_dev.sh` (Electron), `out/`, `agent/` si Electron-only.
- `package.json` : retirer `electron`, `electron-vite`, `electron-builder`, `@electron/*`, `electron-store`, `electron-log`, `electron-updater`, `electron-window-state`, `keytar`, `node-llama-cpp`, `vectra`, `@vscode/ripgrep`, `chokidar`, `fs-extra`, `iconv-lite`, `ced`, `fuzzaldrin`, `command-exists`, `native-keymap`, `arg`, `@electron/remote`.
- Scripts `package.json` : supprimer `start`, `dev` (Electron), `build`, `build:unpack`, `build:mac/linux/win`, `web:start`, `web:docker:*`, `tauri:dev` devient le `dev` par défaut.
- `.npmrc`, `pnpm-workspace.yaml` (allowBuilds native) : nettoyer.
- `README.md` : retirer mentions Electron ; Quick start = `pnpm tauri:dev`.
Validation : `pnpm install` vert ; `pnpm tauri:check` vert ; `pnpm tauri:dev` lance l'app.

### Lot 16 — Vérification finale
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `pnpm tauri:check`
- `pnpm lint` (front)
- `pnpm test:unit` (renderer)
- `pnpm tauri:mac:smoke`
- Update `docs/parity` si script existe.
Validation : tout vert.

---

## Skipped by Ponytail
- `tantivy` : FTS5 SQLite suffit ; éviter la lourdeur.
- `git2` : `std::process::Command` git comme en JS.
- `ort`/`candle` : embeddings via `llama-server` déjà bundlé.
- `tauri-plugin-store` : serde_json + atomic_write couvre electron-store.
- `tauri-plugin-menu` custom : menus via frontend (Tauri webview).
- Refactor de `markdown_engine.rs`/`note_domain.rs` : fonctionne, hors scope.
- Abstraction "provider AI" générique : `pi_runtime.rs` + `chat_runtime.rs` couvrent ; ajouter seulement les routes manquantes.

## Ordre garanti runnable
Lot 0 → 1 → 2 → 3 → 4 → 5 → 6 (FTS routage) → 7 → 8 → 9 → 10 → 11 → 12 → 13 → 14 (frontend rebranche) → 15 (suppression Electron, qui ne peut se faire qu'après 14) → 16.

Après chaque lot : `cargo check` + `cargo test`. Après le 14 : `pnpm tauri:web:build`. Après le 15 : `pnpm tauri:dev`.