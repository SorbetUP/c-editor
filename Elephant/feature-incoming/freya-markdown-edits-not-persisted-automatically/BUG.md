# Freya Markdown edits not persisted automatically

- Request: L'éditeur de note accepte visuellement la saisie mais les changements ne sont pas écrits automatiquement sur disque lorsque la préférence autoSave est absente.
- Level: high
- Date: 2026-08-19
- Slug: freya-markdown-edits-not-persisted-automatically

## Summary
- [x] Reproduced in the packaged Freya test app: text appeared in the editor but the file did not change while the profile had no `autoSave` key.
- [x] Root cause isolated: Freya's missing-preference default was `auto_save=false`, while Tauri enables autosave when the preference is absent.

## Repro Steps
- [x] Launch `.cache/FreyaTest.app` with `/tmp/elephant-freya-ui/profile/preferences.json` containing no `autoSave` key.
- [x] Open `Alpha.md`, focus the paragraph, and type text.
- [x] Wait longer than the autosave interval and inspect `/tmp/elephant-freya-ui/vault/Alpha.md`.
- [x] Before the fix, the in-memory view changed but the file remained unchanged; manual Save wrote it.

## Environment
- [x] macOS desktop, Freya packaged test app, Rust/Freya 0.4.1.
- [x] Runtime log: `/tmp/elephant-freya-ui/profile/runtime.log`.

## Observed vs Expected
- Observed: typing produced `[freya][editor] action:complete action=mutation ... dirty=true`, but no autosave occurred with the missing preference default.
- Expected: match Tauri startup policy: missing `autoSave` enables autosave, and the edited Markdown is written automatically.

## Hypotheses
- [x] Primary cause confirmed: the Freya lifecycle default diverged from `AppShell.vue`, which sets `autoSave` to true when absent.
- [ ] Secondary editor issues may remain in focus/selection parity and must be tested separately after persistence is fixed.

## Investigation Plan
- [x] Compare Tauri `AppShell.vue` startup policy with Freya `EditorPreferences`.
- [x] Reproduce with Computer Use and inspect the real Markdown file and runtime log.
- [x] Verify manual Save as a control path.

## Fix Plan
- [x] Change Freya's missing-preference default to `auto_save=true`.
- [x] Use a 1000 ms default effective delay, matching Tauri's capped autosave scheduling behavior.
- [x] Preserve explicit `autoSave=false` profile settings.

## Regression Tests
- [x] Add `missing_preferences_match_tauri_autosave_startup_policy`.
- [x] Existing explicit-policy autosave test passes with `autoSave=true`.
- [x] Computer Use runtime proof: edit, wait, then inspect the real file; the edited content was written without pressing Save.
- [ ] Add a packaged acceptance test that starts from a profile with no `autoSave` key and asserts the same path automatically.

## Release Notes
- [x] Freya now defaults to automatic Markdown persistence when the preference has never been set, matching Tauri.

## Risks
- [x] Users who explicitly set `autoSave=false` remain opted out.
- [ ] The existing Freya lifecycle suite has an unrelated hook-order panic in its failure-path test; it needs separate stabilization before being used as full-suite proof.

## Rollout
- [x] Rebuild and sign `.cache/FreyaTest.app` after the change.
- [ ] Run the packaged clean-profile acceptance suite before release.

## Follow-up: caret clicks and Tauri editor chrome

- Request: the pointer must place the caret on the clicked line, the note must accept typing and keyboard movement, and Freya must not show the extra bottom editor surface.
- Reproduced before the follow-up fix: the packaged app accepted a pointer click and text insertion, but its stale build still displayed the Freya-only footer controls at the bottom of the viewport.
- Fix: route paragraph pointer-down and global pointer-move through Freya's native `PointerEventData` path, and remove the Freya-only footer, word-count/status controls, text-scale buttons, and editor-theme shortcut. The editor now receives the shell palette from the same source as the library instead of maintaining a private theme toggle.
- Regression proof: `editor_keyboard_freya_testing` passes 8/8, including `pointer_click_places_the_caret_at_the_clicked_line_position` and assertions that the footer/extra controls are absent.
- Packaged Computer Use proof on macOS after rebuild/signing: the editor opened without the bottom surface; clicking inside the body and typing inserted `CLICK` at the clicked text position. The resulting synthetic profile file was written by autosave.
- Remaining limitation: the broader lifecycle failure-path test still hits the pre-existing Freya hook-order panic; it is not used as proof for this caret fix.

## Follow-up: invisible caret, accidental selection, and crash evidence

- Reproduced in the rebuilt packaged app: the caret was not visible because the paragraph used Freya's default black cursor on the dark Tauri-compatible palette.
- Reproduced the strange selection: global pointer movement was forwarded to `freya-edit` even without an active drag, so moving the mouse after a click could extend the selection. Local mouse-up did not always reset the editor's click state.
- Fix: pointer selection now updates Muya focus immediately, paragraph cursor/highlight colors are explicitly derived from the active palette, pointer movement is gated by an explicit drag state, and local mouse-up always sends `EditableEvent::Release`.
- Regression proof: `cargo test --lib` passes 167/167; `editor_keyboard_freya_testing` passes 8/8, including a click followed by a non-drag mouse move before typing.
- Packaged Computer Use proof: the caret is visibly rendered at line end, a later click clears the prior range and places one caret, and repeated pointer-selection actions produce no new fatal log after the final launch.
- The historical fatal log in `/tmp/elephant-freya-ui/profile/runtime.log` was also inspected. Its older crash was a Freya State borrow in the prior drawing implementation (`drawing.rs:715`) and a separate old ScrollView hook panic, not the current editor caret path; the final packaged binary was rebuilt after the editor fix.

## Follow-up: Enter and live Muya paragraph focus

- Reproduced in the packaged app: `Return` created a new Markdown paragraph, but the following text was still routed to the previous paragraph (`NodeId(6)`), so the visible result was broken and the new line could render as an empty list-like block.
- Tauri reference inspected: `NoteEditorHost.vue` mounts the real `EditorWithTabs`/Muya runtime, and Muya's `insertParagraph('after', '', true)` changes the active cursor before the next `change` event is persisted. Freya must preserve that same focus hand-off rather than behave as a single display buffer.
- Fix: editable blocks are keyed by their Muya `NodeId`; after Enter, the new block remains a distinct Freya component and the focus target points at its text node. The next key event is now delivered to the new block.
- Regression proof: `enter_then_type_targets_the_new_muya_paragraph` passes; the packaged Computer Use scenario visibly produces a separate `NEWLINE` paragraph and the saved Markdown contains the same separate line. Runtime logs show Enter followed by typing on the new node (`NodeId(17)`), not the old node (`NodeId(6)`).

## Follow-up: continuous writing and save-watch feedback

- Reproduced in a real packaged journey: type on the first paragraph, press Return, immediately type on the second paragraph, delete and reinsert a character, then wait for autosave. Before the fix the watcher treated its own earlier save as an external conflict while the second paragraph was dirty; this produced a visible rollback/error and could be followed by a Freya hook panic.
- Tauri reference: Muya stays mounted and emits `change`; its own persistence does not replace the live editor instance. Freya now ignores watcher contents equal to either the live serialized document or the last known saved Markdown, preserving the active session while new edits are pending.
- Tauri's `AUTOSAVE_DELAY_MS` is 160 ms; Freya's missing-preference default now uses the same delay.
- Final packaged proof after the fix: first line `... one`, second line `two`, Backspace removed the final character and a further Backspace merged the empty paragraph back into the first line. The file was persisted, the caret remained visible, and no new fatal event occurred after the final launch.
- Strict pixel-perfect Muya/Tauri parity remains `PARTIALLY PROVEN`: the native Freya renderer uses the real Muya model but is not yet the exact DOM/CSS Muya runtime surface used by Tauri.
