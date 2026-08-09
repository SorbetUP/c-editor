---
name: anti-fake-tests
description: >
  Design and review tests so they prove real behavior instead of surface-only,
  placeholder, skipped, or misleading success. Use for every ElephantNote feature
  gate and APEX Tests/Validate phases.
argument-hint: '<feature-or-test-change>'
---

# Anti-Fake Tests

Use this skill whenever tests are added, changed, skipped, mocked, or used as proof that a feature works. Also load `../test-integrity/SKILL.md` for any non-trivial test change.

## Test validity rule

A test is valid only if it checks an observable contract that would fail if the implementation were broken.

Good evidence includes:

- exact file created, modified, moved, hidden, or deleted;
- exact serialized payload crossing Electron/Tauri/API boundaries;
- exact user-visible state after an action;
- exact error surfaced for invalid input;
- exact sync/search/index result from a real path;
- exact artifact or bundle produced by the build command.

## Forbidden patterns

- Mocking the whole implementation under test.
- Rendering a component and asserting only that it exists.
- Snapshotting empty or irrelevant output.
- Using `skip`, `only`, `todo`, `pass`, or broad catch blocks to hide failures.
- Replacing a real failing e2e/integration test with a weaker smoke check.
- Claiming filesystem, sync, AI, or Tauri behavior from pure static tests alone.

## ElephantNote examples

- Asset handling: assert the file is written under `.assets` and the Markdown link resolves.
- Dashboard: assert dashboard notes live under `.dashboard`, not vault root.
- Vault tree: assert create, rename, move, and delete change the real tree model and disk path.
- Tauri bridge: assert the correct command name and payload cross the bridge.
- Sync: assert the second vault receives the real file and conflict behavior is explicit.
- Search: assert the indexed note is returned by the intended exact or semantic path.

## Required review questions

- What exact product behavior is protected?
- Would the test fail if the real implementation was disconnected?
- Which mocks are present and why are they not replacing the behavior under test?
- What assertion proves the observable contract?
- Which CI job or local command runs the test?

## APEX integration

- Analyze: identify the real observable contract.
- Plan: write the failing test before or beside the implementation.
- Execute: do not loosen assertions to match broken behavior.
- Validate: run the narrow test and then the relevant suite.
- eXamine: try to make the test pass with a disconnected implementation; if it still passes, rewrite it.
