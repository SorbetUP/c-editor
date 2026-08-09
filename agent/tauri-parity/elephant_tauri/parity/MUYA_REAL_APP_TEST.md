# Real app Muya runtime test

This page validates the replacement Muya runtime inside the real renderer application, without replacing the production editor yet.

## Route

Open:

```text
/#/muya-runtime-test
```

The route is registered in:

```text
src/renderer/src/router/index.js
```

The page component is:

```text
src/renderer/src/pages/muya-runtime-test.vue
```

## What it tests

The page mounts:

```text
MuyaRuntimeEditor
```

and shows three panels:

1. source Markdown textarea;
2. real runtime editor;
3. runtime state preview.

It tests in real renderer conditions:

- global bootstrap bridge;
- Vue wrapper;
- Markdown sync;
- paste HTML/text;
- table parsing;
- task list parsing;
- footnotes;
- image toolbar source data;
- math block source data;
- Mermaid block source data;
- JSONState block list.

## Manual test flow

1. Start the app.
2. Open `/#/muya-runtime-test`.
3. Set mode to `active`.
4. Edit the source Markdown panel.
5. Paste rich HTML into the runtime editor panel.
6. Verify the Markdown source updates.
7. Verify the runtime state panel lists expected block types.
8. Use browser devtools to run:

```js
window.__ELEPHANT_MUYA_RUNTIME__.mode()
window.__ELEPHANT_MUYA_RUNTIME__.setMode('shadow')
window.__ELEPHANT_MUYA_RUNTIME__.setMode('active')
```

## Automated tests

Run:

```bash
pnpm test:unit -- test/unit/muyaFullEditorRuntime.spec.js test/unit/muyaRuntimeVueIntegration.spec.js test/unit/muyaGlobalRuntimeBridge.spec.js test/unit/muyaRuntimeRoute.spec.js
```

Then run:

```bash
bash scripts/tauri-rust-check.sh
```

## Production editor replacement rule

Do not replace the production editor directly from this test page. First use this page to stabilize the runtime, then wire `MuyaRuntimeEditor` into the actual editor component behind `shadow`, then switch to `active` after parity tests pass.
