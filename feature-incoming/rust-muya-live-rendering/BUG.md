# Rust Muya live rendering

- Request: Le rendu Markdown Rust nest pas mis a jour en temps reel comme Muya pendant la saisie.
- Level: high
- Date: 2026-07-19
- Slug: rust-muya-live-rendering

## Summary
- [x] The Rust editor received an empty model because `runtimeEditor.vue` converted an already converted editor document a second time.
- [ ] Confirm live visual updates for headings, emphasis and lists after the correction in a manual Tauri session.

## Repro Steps
- [x] Start `pnpm tauri:dev`.
- [x] Open a note containing a title and body text.
- [x] Type Markdown continuously, including `- dddd` and `# ddd`.
- [x] Inspect the renderer logs and the visible editor surface.

## Environment
- [x] macOS arm64, Tauri dev runtime, Rust Muya enabled, CodeMirror disabled.
- [x] Observed log: `currentMarkdownLength: 122`, `visibleMarkdownLength: 26`, then Rust `markdownLength: 0`.

## Observed vs Expected
- Observed: Rust mounted with an empty document; `beforeinput` events had `selection: null`, preventing patches and live rendering.
- Expected: Rust mounts with the visible editor Markdown, applies view patches immediately, and updates formatted DOM while typing.

## Hypotheses
- [x] Double application of `toEditorMarkdown` stripped the document before Rust initialization.
- [ ] The Rust core or DOM adapter still has a separate latency/parity issue for a specific Markdown feature.

## Investigation Plan
- [x] Trace document Markdown -> visible Markdown -> runtime model value.
- [x] Run unit tests for Rust Vue runtime and DOM patching.
- [ ] Run a manual Tauri session after the correction and inspect each keystroke.

## Fix Plan
- [x] Pass the already normalized `markdown` prop directly to `RustMuyaRuntimeEditor`.
- [ ] Add an end-to-end assertion that the DOM changes after each live input batch.

## Regression Tests
- [x] `tests/app/unit/specs/muya/rustVueRuntime.spec.js`.
- [x] `tests/app/unit/acceptanceTestBridge.spec.js`.
- [x] Tauri acceptance keyboard probe reaches `beforeinput -> patches:received -> markdown-change -> disk`.
- [ ] Add formatted live-render assertions for heading/list/strong transitions.

## Release Notes
- [x] Rust Muya no longer receives a double-converted empty editor value on note mount.

## Risks
- [ ] Formatting-specific live rendering may still differ from legacy Muya and requires a dedicated visual/DOM parity test.

## Rollout
- [x] Correction is isolated to the Rust editor prop binding; no CodeMirror fallback is reintroduced.
