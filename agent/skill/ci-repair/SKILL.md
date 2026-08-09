---
name: ci-repair
description: >
  Fix failing CI by reading the real remote CI state, repairing the root cause,
  and keeping tests meaningful.
argument-hint: '<ci-failure>'
---

# CI Repair

Use this skill for failing CI, local test failures, typecheck/lint failures, and build regressions.

## Remote CI rule

When repository CI is available, inspect the workflow runs for the head commit. A local command list is not enough for the final report.

## Procedure

1. Record the head commit SHA.
2. Inspect workflow runs for that commit.
3. Inspect failed jobs and the failing step.
4. Locate whether the failure is caused by code, test expectation, environment, or workflow config.
5. Apply Ponytail before coding: smallest correct diff, existing helpers first, no speculative abstractions.
6. Fix the smallest real cause.
7. Push the fix and inspect the new commit CI state.
8. Report success, failure, pending, running, or blocked honestly.

## Integrity rules

- Tests must prove observable behavior.
- Keep assertions meaningful.
- Keep useful coverage.
- Keep code easy to test with explicit inputs, outputs, errors, and deterministic fixtures.
- Keep code readable, efficient, and focused on the requested change.

## Output

```md
CI repair:

- head commit:
- workflows checked:
- failing step:
- root cause:
- fix:
- CI result:
- still pending or not proven:
```
