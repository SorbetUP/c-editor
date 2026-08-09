---
name: root-cause-debugging
description: >
  Diagnose bugs from the real call path and fix the shared root cause instead
  of patching one visible symptom.
argument-hint: '<bug-report>'
---

# Root Cause Debugging

Use this skill for bug reports, crashes, broken UI actions, data not saving, loading loops, and failed commands.

## Procedure

1. Restate the symptom.
2. Find the first boundary where expected state diverges from actual state.
3. Trace caller → shared helper → bridge/service → runtime/filesystem → UI refresh.
4. Grep sibling callers before editing a shared function.
5. Fix the earliest shared cause that covers all affected paths.
6. Add one regression check that would fail on the old bug.

## Evidence to collect

- Error message or log line.
- File/function where the bad state starts.
- All sibling callers affected.
- Test or command proving the fix.

## Rules

- Do not add guards in every caller when the shared helper can reject/normalize correctly once.
- Do not suppress errors that the user needs to see.
- Do not replace a real failure with an empty result.
