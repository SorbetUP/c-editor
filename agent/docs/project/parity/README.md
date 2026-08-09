# Parity backlog generation

This folder contains the generated, per-subfeature Electron/Tauri parity backlog.

The source of truth is:

```text
scripts/generate-parity-docs.mjs
```

Generate the expanded backlog with:

```bash
pnpm docs:parity
```

This produces:

```text
docs/parity/index.md
docs/parity/generated/*.md
```

The generator currently emits:

```text
25 subfeature files
410 task lines per subfeature
10,250 generated task lines minimum
```

Use this generated backlog as the detailed execution plan. The shorter document `docs/TAURI_ELECTRON_PARITY_TODO.md` remains the human summary and priority checklist.

Rules:

- Do not close a task by modifying tests only.
- Every closed task must be linked to a production-code fix, a real production import/mount test, or a runtime/visual parity result.
- Prefer tests that import real stores/components over artificial harness tests.
- Visual parity must be measured with real screenshots after functional parity is stable.
- Keep Electron and Tauri parity deltas explicit.
