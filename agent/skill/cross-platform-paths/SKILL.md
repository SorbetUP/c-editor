---
name: cross-platform-paths
description: >
  Verify path handling across macOS, Linux, Windows, Android, Docker, Electron,
  and Tauri. Use for vault, assets, dashboard, sync, import/export, and model
  library filesystem behavior.
argument-hint: '<path-or-filesystem-task>'
---

# Cross-Platform Paths

Use this skill whenever a task touches filesystem paths, vault trees, hidden folders, sync roots, model files, imports, exports, or drag/drop.

## Required cases

Test or review these cases when relevant:

- spaces in file and folder names;
- unicode names;
- nested folders;
- hidden folders such as `.assets`, `.dashboard`, `.elephantnote`, and `.config`;
- file-versus-directory mistakes;
- relative paths and normalized paths;
- Windows separators and drive-like paths;
- symlinks or aliases when the code explicitly supports them;
- permission-denied or missing-file errors surfaced to the user.

## ElephantNote path rules

- Never read a directory as a Markdown file.
- Hidden app folders must not show as normal notes in the main tree.
- Assets and Excalidraw files belong under `.assets` unless a feature explicitly says otherwise.
- Dashboard notes belong under `.dashboard`.
- Sync metadata belongs under `.elephantnote/sync` or the declared metadata path, not mixed into user notes.
- Path helpers must keep UI paths, vault-relative paths, and absolute OS paths clearly separated.

## Verification

Prefer tests that create a temporary vault and assert real disk state. Do not rely only on string snapshots when a path operation is supposed to touch the filesystem.

## APEX integration

- Analyze: locate every conversion between UI path, vault-relative path, and OS path.
- Plan: choose one shared helper instead of duplicating normalization rules.
- Execute: patch both read and write paths.
- Validate: run tests on the temporary vault contract.
- eXamine: check macOS/Linux/Windows naming assumptions and hidden-folder leakage.
