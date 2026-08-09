# Muya runtime architecture

This document defines the target architecture for replacing Muya behavior without rewriting the ElephantNote Vue UI.

## Goal

The goal is not to create a second editor UI. The goal is to provide a Muya-compatible runtime that the existing Vue editor can call progressively.

The runtime is split into deterministic, testable layers:

1. Markdown deterministic engine in Rust.
2. Browser/editor runtime in JavaScript.
3. Source snapshot comparison against real Muya.
4. Vue integration layer.

## Layers

### 1. Rust deterministic Markdown engine

Location:

```text
src-tauri/src/markdown/
```

Responsibilities:

- parse Markdown structure;
- render deterministic HTML contracts;
- expose Muya-like tokens/extras;
- parse frontmatter;
- detect math, diagrams, references, footnotes, tables and edge cases;
- validate against source snapshots.

This layer must stay independent from browser DOM.

### 2. JavaScript Muya runtime

Location:

```text
src/renderer/src/muya/
```

Public entrypoint:

```text
src/renderer/src/muya/index.js
```

Internal modules:

```text
selectionRuntime.js       DOM editor and browser selection snapshots
jsonStateRuntime.js       Markdown <-> JSONState <-> HTML
operationsRuntime.js      OT operations and grouped history
clipboardRuntime.js       copy/paste and rich HTML normalization
tableImageRuntime.js      table commands and image toolbar state
menusPreviewRuntime.js    footnotes, slash menu, floating toolbar, previews
fullEditorRuntime.js      runtime composition facade
runtimeFlags.js           disabled/shadow/active rollout modes
useMuyaRuntimeEditor.js   Vue composable
MuyaRuntimeEditor.vue     Vue wrapper component
```

### 3. Source snapshot adapter

Location:

```text
scripts/adapters/real-muya-renderer.mjs
scripts/generate-muya-source-snapshots.mjs
```

Responsibilities:

- use real Muya when available;
- generate strict source snapshots;
- refuse 100% claims when fallback snapshots are used.

### 4. Vue integration layer

Available integration API:

```text
src/renderer/src/muya/useMuyaRuntimeEditor.js
src/renderer/src/muya/MuyaRuntimeEditor.vue
```

The Vue UI should import only:

```js
import { createMuyaFullEditorRuntime, MuyaRuntimeEditor, useMuyaRuntimeEditor } from '@/muya'
```

It must not import internal modules directly unless a component owns that exact concern.

Rollout modes:

```text
disabled  do not mount the replacement runtime
shadow    mount for comparison/telemetry without replacing production behavior
active    use the replacement runtime as the editor runtime
```

## Runtime facade contract

`createMuyaFullEditorRuntime(root, markdown, options)` returns:

```text
state
markdown
html
setMarkdown()
snapshotSelection()
restoreSelection()
applyOperation()
undo()
redo()
pasteClipboard()
copy()
table()
imageToolbar()
resizeImage()
footnotePopup()
upsertFootnote()
slashCommands()
floatingToolbar()
previewBlock()
```

## Testing strategy

### JS unit/runtime tests

Locations:

```text
test/unit/muyaFullEditorRuntime.spec.js
test/unit/muyaRuntimeVueIntegration.spec.js
```

These tests validate the runtime facade and the Vue wrapper in one pass.

### Rust tests

Location:

```text
src-tauri/src/markdown/*_tests.rs
```

These tests validate deterministic Markdown contracts and source snapshots.

### Snapshot tests

Location:

```text
elephant_tauri/parity/
```

These snapshots are the bridge between the existing Electron/Muya behavior and the Rust/JS replacement.

## 100% rule

The project may not claim 100% Muya parity until all of the following are true:

1. `scripts/adapters/real-muya-renderer.mjs` generates snapshots from real Muya.
2. `muya_source_snapshots_meta.json` reports `real-electron-muya-renderer`.
3. Rust deterministic tests pass.
4. JS runtime tests pass.
5. The runtime is wired into the real Vue editor.
6. Pixel/behavior tests confirm that the existing Vue UI did not regress.

## Current status

The architecture exists, the runtime facade is testable, and the Vue composable/wrapper integration exists. The next step is to wire `MuyaRuntimeEditor` or `useMuyaRuntimeEditor` into the actual production editor component behind the `shadow` flag, then compare behavior against the current Electron/Muya editor.
