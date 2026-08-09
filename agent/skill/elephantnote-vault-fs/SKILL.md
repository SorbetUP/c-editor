---
name: elephantnote-vault-fs
description: >
  Rules for vault tree operations, hidden folders, file safety, create/rename/move/delete,
  drag-and-drop, and disk/UI consistency.
argument-hint: '<vault-filesystem-task>'
---

# ElephantNote Vault Filesystem

Use this skill for the vault explorer, folder tree, note/folder creation, rename, move, delete, drag-and-drop, trash/archive, hidden internal folders, and file safety.

## Invariants

- `.dashboard`, `.assets`, `.embeddings`, `.wiki`, and similar internal folders are hidden from normal navigation.
- Dashboard notes belong in `.dashboard`, not at the vault root.
- Folder and note operations update real disk state and UI state.
- Directory entries are never read as text files.
- Drag/drop move must either complete on disk or report a visible/logged failure.
- Rename/delete via context menu must work for folders and notes where supported.

## Read first

- Vault backend: `Elephant/backend/js/vaults.js` and Tauri vault commands.
- Frontend stores/components: `vaultStore`, `navigationStore`, `SidebarTreeEntry`, drag/drop utilities.
- Shared path/workspace helpers in `Elephant/shared/**`.
- Existing file-safety and vault tests.

## Verification

- Create folder, create note inside it, rename both, move both, delete both.
- Confirm disk state and UI state match after reload.
- Confirm hidden folders exist on disk when needed but never show in the explorer.
- Confirm unsafe paths are rejected instead of silently rewritten.

## Anti-slop

No UI-only rename/move/delete. No silent path rewriting that pretends an unsafe input was valid.
