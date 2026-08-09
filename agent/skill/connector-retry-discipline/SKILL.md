---
name: connector-retry-discipline
description: >
  Handle failed connector writes or actions by diagnosing the likely cause,
  changing strategy, and retrying with a safer smaller operation.
argument-hint: '<failed-connector-action>'
---

# Connector Retry Discipline

Use this skill when a connector action fails or is blocked.

## Rule

A failed connector call is not a reason to stop after one attempt. Diagnose, reduce risk, and retry differently.

## Procedure

1. Record the action, target path, and high-level failure type.
2. Do not repeat the exact same payload unless the failure was transient.
3. Reduce the operation size or split the patch.
4. Remove ambiguous wording or complex formatting if the connector appears to reject content.
5. Re-fetch the latest file SHA before retrying an update.
6. Try a narrower file or smaller incremental update.
7. If multiple strategies fail, report what was attempted and keep the partial state honest.

## Safer retry patterns

- Create a smaller skill file first, then expand later.
- Update one section instead of replacing a whole long file.
- Avoid large tables when a short list works.
- Avoid broad rewrites of files that changed recently.
- Use existing files that are already routed if creating a new file is blocked.

## Integrity rule

Never claim the failed write succeeded. If a retry succeeds, cite or inspect the resulting file before reporting success.

## APEX integration

- Execute: retry connector failures with a changed strategy.
- Resolve: record failed attempt, changed strategy, and final result.
- Finish: mention any connector-limited partial work honestly.
