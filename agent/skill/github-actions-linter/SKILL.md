---
name: github-actions-linter
description: >
  Verify GitHub Actions workflow syntax, contexts, shell snippets, triggers,
  permissions, and reusable action usage before CI YAML changes are trusted.
  Use with actionlint/yamllint-style checks and APEX validation.
argument-hint: '<workflow-change>'
---

# GitHub Actions Linter

Use this skill whenever `.github/workflows/**` or `.github/actions/**` changes.

## Required checks

Run or add the closest available equivalent of:

```bash
actionlint .github/workflows/*.yml
```

When `actionlint` is not installed, perform the same review manually and report that the static tool was not run.

## What to inspect

- Invalid contexts such as using `secrets` where they are unavailable.
- Invalid `needs`, job IDs, outputs, and matrix references.
- Shell snippets missing `set -euo pipefail` or explicit failure handling.
- `pull_request_target` usage and any step that checks out untrusted code.
- Triggers that accidentally skip `assistant/**` or PR updates.
- Missing or too-broad permissions.
- Commands that rely on tools not installed in the runner.

## ElephantNote workflow rules

- The main CI must run `scripts/verify-critical-flows.mjs` before general tests.
- Tauri workflows must keep a blocking cargo check for `Elephant/backend/tauri`.
- macOS packaged-window checks must not be replaced by a JS-only smoke test.
- Sync Docker smoke checks must remain explicit when sync behavior changes.

## APEX integration

- **Analyze**: read the changed workflow and the local script it invokes.
- **Plan**: choose the smallest YAML change and keep local commands reproducible.
- **Validate**: run lint/static checks for the workflow.
- **eXamine**: re-read the diff for hidden always-pass gates.

## Never accept

- `|| true` around a required command.
- Blanket `continue-on-error` for correctness, security, build, or package checks.
- Deleting a failing step instead of fixing the cause.
- A workflow change that cannot be reproduced locally or explained from logs.
