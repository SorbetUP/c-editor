---
name: real-verification
description: >
  Prove that a change works with concrete commands, runtime checks, logs,
  disk state, screenshots, or user-visible workflows. Use whenever asked to
  verify, test, prove, or confirm that something works.
argument-hint: '<thing-to-verify>'
---

# Real Verification

Use this skill when the user asks whether something actually works, or when a task changes user-visible behavior.

## Evidence ladder

Use the highest relevant evidence level available:

1. Exact command output.
2. Targeted unit/integration test.
3. Runtime log proving the real path executed.
4. Disk/database state after the user action.
5. App/browser/simulator workflow and screenshot when UI matters.
6. CI run result when the task is about repository health.

## Required report

```md
Verification:

- command/workflow:
- result:
- evidence:
- not verified:
```

## Rules

- Do not claim runtime verification from static reading.
- Do not claim UI verification from a unit test alone.
- Do not claim sync/search/AI works unless a real sync/search/provider/index path ran.
- If blocked, state exactly what blocked verification.
