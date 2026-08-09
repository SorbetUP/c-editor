---
name: ci-stability
description: >
  Stabilize intermittent CI checks by replacing timing guesses with observable
  readiness checks, isolated test data, and clear cleanup.
argument-hint: '<unstable-check>'
---

# CI Stability

Use this skill when a check is inconsistent across local runs or GitHub runners.

## Procedure

1. Record the exact command, runner OS, and shortest useful log snippet.
2. Find the shared state: temp root, cache, file watcher, timer, process, or generated artifact.
3. Prefer readiness conditions over fixed delays.
4. Use a unique temp vault or workspace per test.
5. Clean up in a way that is safe to run twice.
6. Repeat the narrow command when practical.

## ElephantNote cases

- Tauri app launch checks.
- Vault tree updates after create, rename, move, or delete.
- Sync startup and queue completion.
- Search index rebuild completion.
- Image and Excalidraw save completion.

## APEX integration

- Analyze the missing readiness signal.
- Plan a deterministic wait or state assertion.
- Execute without weakening assertions.
- Validate by repeating the narrow command.
- eXamine whether a real regression would still fail.
