# Tauri manual save writes nothing

- Request: Corriger le bug où CmdOrCtrl+S marque la note comme sauvegardée sans écrire le fichier sur desktop Tauri, avec logs et test réel sur disque.
- Level: high
- Date: 2026-07-19
- Slug: tauri-manual-save-writes-nothing

## Summary
- [x] Desktop Tauri manual save now writes the active Markdown file through `tauri_marktext_write_file`.
- [x] Success and failure paths remain logged, including the file path, content length and acknowledgment event.

## Repro Steps
- [x] Start `pnpm tauri:dev` or the Tauri package.
- [x] Open a Markdown note, edit it, then trigger the `file.save` command (`CmdOrCtrl+S`).
- [x] Before the fix, `mt::response-file-save` was received, the tab became saved, but no write log or disk update followed.
- [x] Reproduce through `pnpm test:desktop:acceptance`; the runner now rereads the fixture file from disk after `executeCommand file.save`.

## Environment
- [x] macOS arm64, Tauri desktop, renderer Rust/Muya.
- [x] Node.js 22, Rust/Cargo, ElephantNote 0.18.9.
- [x] Reproduced in the real Tauri acceptance process with a temporary fixture vault.

## Observed vs Expected
- Observed: `tauriMarkTextSaveBridge` deliberately ignored `mt::response-file-save` and only sent `mt::tab-saved`; manual save could report success while the file stayed unchanged.
- Expected: the same command must call the Rust filesystem command, report a failure when it rejects, and only acknowledge the tab after the write succeeds.

## Hypotheses
- [x] The compatibility handler incorrectly assumed a separate NoteEditorHost autosave existed for the Rust editor path.
- [x] The Rust/Muya runtime updates Pinia state, but manual `file.save` still relies on the IPC compatibility event.

## Investigation Plan
- [x] Correlate the supplied `tauri-dev` log with the save bridge and editor command path.
- [x] Trace `file.save` → `mt::response-file-save` → Tauri filesystem invoke.
- [x] Add structured acceptance coverage that verifies the persisted file, not only `isSaved`.

## Fix Plan
- [x] Route `mt::response-file-save` through the existing logged `writeRecord` implementation.
- [x] Preserve `mt::tab-save-failure` on Rust write errors and prevent false success acknowledgments.
- [x] Expose `file.save` through the acceptance command surface for functional testing.
- [x] Make the desktop runner poll the file on disk after the manual command.

## Regression Tests
- [x] Unit: `tests/app/unit/specs/main/elephantnote/tauriMarkTextSaveBridge.spec.js` — success and failure paths.
- [x] `pnpm test:desktop:acceptance` — real Tauri dev process, `file.save` writes and rereads the fixture note; 1009 logs.
- [x] `pnpm build:mac` — package rebuilt with the fix.
- [x] `pnpm test:desktop:acceptance:packaged` — real packaged Tauri binary writes and rereads the note; 1009 logs.

## Release Notes
- [x] CmdOrCtrl+S no longer claims success without a disk write on Tauri desktop.
- [x] The acceptance archive records `app:command:start/done`, `write:start/done`, `tauri_marktext_write_file`, and the disk reread.

## Risks
- [x] Save failures are now visible to the existing tab notification path instead of being hidden.
- [x] `response-file-save-as` and multi-tab save paths continue using the same writer.

## Rollout
- [x] Include the bridge fix and regression test in the desktop Tauri change set.
- [ ] Publish and run the cross-platform packaged workflow for Linux, Windows and macOS Intel.
