# Muya live rendering test

Open the real app route:

```text
/#/muya-runtime-test
```

Set the mode to:

```text
active
```

Manual check:

1. Click inside the center editor panel.
2. Edit a paragraph so its content becomes `# Live title`.
3. The center editor should rerender that block as a heading.
4. The source Markdown panel should update to `# Live title`.
5. The live HTML preview panel should update at the same time.
6. The runtime state panel should show a `heading` block.

Automated live test:

```bash
pnpm test:unit -- test/unit/muyaLiveRenderingRuntime.spec.js
```

Full runtime test set:

```bash
pnpm test:unit -- test/unit/muyaFullEditorRuntime.spec.js test/unit/muyaRuntimeVueIntegration.spec.js test/unit/muyaGlobalRuntimeBridge.spec.js test/unit/muyaRuntimeRoute.spec.js test/unit/muyaLiveRenderingRuntime.spec.js
```
