---
name: elephantnote-editor-assets
description: >
  Rules for Markdown editing, image insertion, Excalidraw sidecars, asset paths,
  Muya/editor state, and disk round-trips.
argument-hint: '<editor-assets-task>'
---

# ElephantNote Editor Assets

Use this skill for Markdown editor behavior, images, pasted/uploaded files, Excalidraw, preview rendering, autosave, reload, and `.assets` handling.

## Invariants

- Images, Excalidraw files, and generated previews live in the hidden `.assets` folder, not at the visible vault root.
- The user-facing explorer must hide `.assets`.
- Markdown references must resolve after reload and across Electron/Tauri.
- Do not read binary assets as text notes.
- Do not overwrite a fresh insert/save with stale editor state.
- Excalidraw sidecar and PNG preview stay paired and move together when expected.

## Read first

- `Elephant/frontend/app/components/editor/**`.
- `Elephant/frontend/app/services/excalidraw.js`.
- `Elephant/shared/excalidrawAssets.js`, `imageSource.js`, `markdownDocument.js`.
- Vault/file commands in Electron or Tauri for actual disk writes.
- Existing tests around `excalidraw`, `imageSource`, `noteEditorHostImageLinks`, and markdown documents.

## Verification

- Insert or generate an asset.
- Confirm disk path is `.assets/...`.
- Confirm Markdown contains a stable relative reference.
- Save, reload the note, and verify preview still renders.
- Confirm hidden assets do not appear as notes in the explorer or recently edited list.

## Anti-slop

No alternate Markdown renderer just to hide broken note rendering. Fix the shared markdown/asset path flow.
