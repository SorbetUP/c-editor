# Muya JS live rendering loses content during input

- Request: Tester une note réelle par commandes applicatives et corriger le rendu Markdown live façon MarkText sans Playwright.
- Level: high
- Date: 2026-07-19
- Slug: muya-js-live-rendering-loses-content-during-input

## Summary
- [x] Production editor now mounts Muya JS; Rust and CodeMirror are absent from the live surface.
- [x] Real application-command probing reproduced the DOM/state synchronization gap during contenteditable input.

## Repro Steps
- [x] Launch the Tauri app with `ELEPHANT_ACCEPTANCE_TAURI_PORT=0`.
- [x] Open `Getting Started/New Folder/Untitled.md` through `openNote`.
- [x] Inspect `[data-testid="muya-runtime-editor"]` with `readDisplayed`.
- [x] Drive Markdown through the programmable bridge and inspect `readDisplayed`, `readState`, and `readNote`.

## Environment
- [x] macOS, Tauri runtime, Muya JS runtime, Node 22, app version 0.18.9.
- [x] Renderer surface: `contenteditable`, `data-testid="muya-runtime-editor"`.

## Observed vs Expected
- Observed: browser/contenteditable input could change the DOM while the Muya JSON state and saved Markdown remained stale; a zero-delay render could also move the caret between characters.
- Expected: every real input is synchronized from the DOM, Markdown syntax is rendered live, caret position remains stable, and save/read round-trips the exact Markdown.

## Hypotheses
- [x] The old Rust-only acceptance `insertText` path dispatched `beforeinput` but did not mutate a Muya JS contenteditable surface.
- [x] The JS runtime returned its previous JSON state after block-local rendering instead of treating the edited DOM as authoritative.
- [x] Immediate rerendering during rapid typing caused caret loss during syntax promotion.

## Investigation Plan
- [x] Compare DOM HTML, displayed text, runtime state, persisted note, and acceptance logs after each command.
- [x] Verify absence of Rust and CodeMirror selectors in the production surface.

## Fix Plan
- [x] Keep Muya JS as the production editor and preserve the addon runtime resource compatibility layer.
- [x] Read the edited DOM back into Muya state after input synchronization.
- [x] Coalesce live input rendering over one animation-sized 16 ms window.
- [x] Avoid replacing an already-rendered block on every character and preserve the caret on syntax promotion.
- [x] Make the programmable `insertText` command perform a real contenteditable insertion and emit a document-window input event.

## Regression Tests
- [x] Unit suite: 3210 passed, 171 skipped.
- [x] Renderer build completed successfully.
- [x] Command probe: `setMarkdown` rendered `## Live heading` and the body; `save` persisted; `readNote` returned the exact Markdown.
- [x] Added a dedicated acceptance-bridge unit test proving `insertText` mutates the Muya JS contenteditable surface and emits `input`.

## Release Notes
- [x] Muya JS is the active production editor; Rust remains available only for explicit legacy tooling/tests.

## Risks
- [ ] Native browser Enter/selection behavior still needs a dedicated command-level regression test for every block type.

## Rollout
- [x] Verify with the application command bridge before accepting a manual desktop test.
