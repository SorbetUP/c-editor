---
name: elephantnote-debugging
description: >
  Rules for debugging loading loops, async races, bridge failures, missing logs,
  stale editor state, and hard-to-reproduce runtime bugs.
argument-hint: '<debug-task>'
---

# ElephantNote Debugging

Use this skill for loading loops, missing UI updates, silent command failures, stale editor state, async races, and bugs where screenshots alone are not enough.

## Invariants

- Add actionable logs near the real boundary: UI action, store mutation, bridge call, backend command, disk write, runtime event.
- Logs must not expose secrets or huge payloads.
- Use structured tags where possible so `window.__ELEPHANT_DEBUG_LOGS__` and backend logs can be searched.
- A loading state must have success, error, and timeout/abort behavior where appropriate.
- Fix races at the source of truth, not by delaying UI with arbitrary timers.

## Read first

- The UI component, store, service, and backend command in the failing path.
- Existing debug/logging helpers.
- Tests covering async state or runtime bridge behavior.

## Verification

- Reproduce or simulate the stuck state.
- Confirm logs show each transition.
- Confirm success updates UI and disk/runtime state.
- Confirm failure surfaces an error instead of spinning forever.

## Anti-slop

No silent catch. No infinite spinner. No `setTimeout` masking a stale state overwrite unless the test proves it is the correct synchronization primitive.
