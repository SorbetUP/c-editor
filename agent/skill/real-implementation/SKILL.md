---
name: real-implementation
description: >
  Prevent placeholder or surface-only features. A feature is real only when the
  user-visible path is connected to real state, persistence, runtime behavior,
  errors, and tests.
argument-hint: '<feature-or-fix>'
---

# Real Implementation

Use this skill before claiming that a feature, fix, bridge, sync path, AI path, editor path, or CI gate is implemented.

## Real feature contract

A feature is implemented only when:

- the UI action or API entrypoint calls the intended real path;
- the backend or runtime path exists;
- state changes are persisted when persistence is part of the contract;
- errors are surfaced instead of hidden;
- tests or CI cover the meaningful behavior;
- incomplete surfaces are explicitly marked as not implemented.

## Not enough

- Rendering the UI only.
- Returning a hardcoded success value.
- Adding a button without connecting the action.
- Adding a mock that replaces the behavior being claimed.
- Adding a test that cannot fail on a broken implementation.
- Updating docs without code when code was requested.

## Implementation checklist

1. Trace caller to backend/runtime.
2. Identify durable state or external effect.
3. Add or update a test that fails if the real effect is missing.
4. Verify the error path.
5. Remove or label incomplete surfaces honestly.
6. Report exactly what remains unfinished.

## APEX integration

- Analyze: trace the full path.
- Plan: choose the smallest real implementation path.
- Execute: connect the actual behavior, not only the visible surface.
- Tests: prove the meaningful effect.
- Finish: separate implemented, not implemented, and not proven.
