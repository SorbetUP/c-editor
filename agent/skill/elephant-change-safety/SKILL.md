---
name: elephant-change-safety
description: >
  Preserve Elephant's known-working behavior while changing core, add-ons,
  AI runtimes, UI, persistence or platform code. Requires source provenance,
  parity-first extraction, red-before-green tests, real Tauri execution,
  observable logs and truthful delivery states.
argument-hint: '<feature-fix-or-migration>'
---

# Elephant Change Safety

Use this skill for every non-trivial Elephant change, especially:

- moving a core feature into an add-on;
- restoring an implementation from another branch or PR;
- modifying official add-ons;
- changing AI, Marvin, Codex, OpenCode, llama or model-provider runtimes;
- changing editor, vault, filesystem, persistence or mobile behavior;
- changing test infrastructure, CI or logging;
- claiming that a regression is fixed.

This skill operationalizes the repository-wide requirements in `/AGENTS.md`.

## Phase 0 — establish truth

Record:

```text
Target branch:
Target commit:
Reported behavior:
Known-good branch/commit/PR:
Affected platforms:
Affected add-ons/runtimes:
Required real-app scenario:
```

Inspect history before implementation. If a known-good branch exists, its code is the primary source. A textual description or screenshot is secondary evidence.

## Phase 1 — inventory the complete path

Map the production path:

```text
user action
  -> visible component
  -> store/domain state
  -> host API / IPC / add-on resource
  -> Tauri/backend/service/sidecar
  -> persisted or external effect
  -> returned state
  -> visible success/error
  -> logs and cleanup
```

List every file participating in the path. Note existing tests, selectors, settings keys, persisted files, commands, resource names, permissions and process lifecycles.

Do not start by creating a new interface or abstraction. First understand and preserve the existing one.

## Phase 2 — reproduce and freeze the baseline

For regressions:

- run the current broken behavior;
- save exact logs and screenshots;
- identify the first incorrect layer;
- add a regression test that fails for the user-visible defect.

For core-to-add-on extraction:

- capture baseline screenshots and stable DOM selectors;
- capture settings and persisted-state fixtures;
- run the existing core workflow and save its observable outputs;
- define a parity matrix before removing core ownership.

Example parity matrix:

| Surface | Before extraction | With add-on enabled | With add-on disabled/uninstalled |
| --- | --- | --- | --- |
| navigation | existing item | identical item and behavior | absent |
| settings | existing controls | identical controls and values | absent or intentional host shell only |
| command | real effect | same real effect | unavailable with explicit error |
| persistence | survives restart | survives restart | no active add-on state |
| errors | visible and logged | same or better | no orphan handler |

## Phase 3 — integrate, do not imitate

When code exists on another branch:

1. inspect the branch diff and dependencies;
2. identify exact source commits;
3. merge/cherry-pick where possible;
4. otherwise transplant complete modules with provenance;
5. preserve UI and behavior before adapting host APIs;
6. adapt the existing code to explicit add-on APIs;
7. never reconstruct it from memory.

A conflict is not permission to replace the implementation with a placeholder. Resolve the conflict or report `BLOCKED`.

## Phase 4 — implement the smallest clean path

Use Ponytail-style decomposition:

- focused modules;
- explicit UI/domain/runtime/persistence boundaries;
- no duplicate feature implementation;
- no hidden core fallback;
- no arbitrary UI injection;
- no swallowed errors;
- lifecycle cleanup colocated with lifecycle setup;
- validated host API boundaries;
- no unrelated redesign or refactor.

When a new host API is necessary, demonstrate:

- why existing APIs are insufficient;
- the minimal contract;
- permission and path-safety behavior;
- error shape;
- a real caller;
- a meaningful test;
- compatibility with existing add-ons.

## Phase 5 — prove add-on lifecycle

For each affected official add-on, prove as applicable:

```text
[ ] absent before installation
[ ] real package installs
[ ] enable succeeds
[ ] UI surface opens
[ ] primary user action succeeds through production path
[ ] persisted/external effect is correct
[ ] error path is visible and logged
[ ] restart preserves correct state
[ ] disable removes owned behavior
[ ] uninstall removes owned behavior and runtime
[ ] reinstall succeeds
[ ] no implementation leaked into core bundle
```

A registry entry, package archive, valid manifest, running service or status object is not enough.

## Phase 6 — prove AI/runtime behavior

For Marvin or any AI runtime, separate proof levels:

### Adapter proof

A deterministic fake process/server may prove framing, parsing, timeout and error handling.

### Real-runtime proof

The actual runtime must prove startup, handshake, model discovery, request, response/streaming, cancellation, persistence, errors and shutdown.

### UI proof

The actual Elephant UI must prove provider selection, model selection, send, streamed/rendered response, cancellation, reload/persistence and actionable error display.

Never collapse these proof levels into a single `works` claim.

## Phase 7 — logging proof

For every affected command, retain observable evidence with correlation IDs across layers.

Expected event pattern:

```text
renderer action:start
host command:start
addon/service request:start
addon/service request:done|error
host command:done|error
renderer action:done|error
cleanup:done|error
```

Include duration, sanitized target/path, add-on/runtime identity and error chain. Verify the lines by running the scenario. Static logging code is `NOT PROVEN` until executed.

## Phase 8 — red/green and sabotage check

For each new or changed regression test:

1. show it failing on the broken revision;
2. show it passing on the fixed revision;
3. temporarily remove or break the essential implementation when practical;
4. confirm the test fails;
5. restore the implementation;
6. confirm it passes again.

Reject a test that passes with a hardcoded result, empty implementation, full mock or missing external effect.

## Phase 9 — real application validation

Select commands based on impact:

```bash
node build/scripts/verify-agent-governance.mjs
pnpm test:unit
pnpm test:official-addons:e2e
pnpm tauri:check
pnpm tauri:web:build
pnpm test:desktop:acceptance
pnpm test:desktop:acceptance:packaged
pnpm prod:check
```

For Android-impacting work, also build/install the APK and run the real device/emulator flow with `adb logcat`. For platform-specific work, validate that platform.

Run from the exact final commit in a clean workspace. Keep the command output and `test-results` artifacts.

## Phase 10 — final audit

Inspect the final diff and answer:

```text
Did any existing UI change without explicit request?
Did any working branch implementation get replaced rather than integrated?
Did any test expectation get weakened?
Does any mock replace the claimed behavior?
Is any success value hardcoded?
Is any core fallback hiding an add-on failure?
Are errors swallowed?
Are lifecycle listeners/processes cleaned up?
Were logs actually observed?
Was the packaged app actually run?
What remains unproven?
```

Any unresolved `yes` or missing proof prevents a `PROVEN` completion claim.

## Required delivery record

```text
Status: PROVEN | PARTIALLY PROVEN | NOT PROVEN | BLOCKED
Branch:
Final commit:
Integrated source branch/commit/PR:
Changed behavior:
Pre-fix reproduction:
Regression test red evidence:
Regression test green evidence:
Real application scenario:
Platforms actually executed:
Commands:
Logs/artifacts:
Tests modified and justification:
Remaining limitations:
```

Do not merge or claim completion while required evidence is missing.
