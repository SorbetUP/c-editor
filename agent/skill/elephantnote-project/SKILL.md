---
name: elephantnote-project
description: >
  Default project rules for ElephantNote. Use for every task in this repository
  before choosing more specific skills.
argument-hint: '<task>'
---

# ElephantNote Project

ElephantNote is a local-first Markdown notes app with Electron and Tauri runtimes, vault storage, search/graph/wiki, sync, model library, and optional AI assistance.

## Always preserve

- Local-first behavior: notes and assets are real files in the vault.
- Hidden internal folders stay hidden from the user-facing explorer: `.assets`, `.dashboard`, `.embeddings`, `.wiki`, and internal sync/index folders.
- Electron/Tauri parity: do not fix one runtime by breaking or silently bypassing the other.
- Real implementation only: no fake provider, fake sync, fake search, fake graph, fake write success, or smoke-only UI.
- Tests before or with non-trivial code changes.

## Skill routing

- Tauri/Rust/window command change → `elephantnote-tauri`.
- Electron/legacy IPC/preload change → `elephantnote-electron`.
- Markdown/assets/Excalidraw/image change → `elephantnote-editor-assets`.
- Vault explorer/tree/path operation → `elephantnote-vault-fs`.
- Sync/mobile/desktop pairing → `elephantnote-sync`.
- Search/embedding/wiki/graph → `elephantnote-search-wiki-graph`.
- AI/model provider/runtime/OCR → `elephantnote-ai-runtime`.
- UI parity/layout/window chrome → `elephantnote-ui-parity`.
- CI/build/test workflows → `elephantnote-ci`.
- Loading loop/logging/async race → `elephantnote-debugging`.
- Addons/plugins → `elephantnote-addons`.
- Android/mobile behavior → `elephantnote-mobile`.
- Clean code/maintainability → `clean-code-ponytail`.
- Verification/proof → `real-verification`.

## Done means

- The user-visible behavior works.
- The disk/runtime state matches the UI state.
- Logs or tests prove the real path was used.
- No regression is hidden behind skips, loose assertions, or placeholder code.
