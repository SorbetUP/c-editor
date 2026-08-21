# Elephant engineering rules — mandatory

These instructions apply to the whole repository. The purpose is simple: deliver the requested Elephant behavior reliably, with the least unnecessary work.

## 1. Delivery first

The user's requested behavior is the task. Architecture, refactors, abstractions, CI, documentation and cleanup exist only to support that behavior.

- Work on one complete user-visible vertical slice at a time.
- Do not replace a requested feature with an architecture project.
- Do not add unrelated frameworks, crates, services, migrations, ADRs or generic infrastructure because they may be useful later.
- Prefer the smallest compatible change to a broad redesign.
- Source volume is not progress. A feature working through the real product path is progress.
- If a feature cannot yet be executed/tested, stop adding dependent feature code and unblock execution first. Do not accumulate days of `implemented but not proven` work.
- Report a blocker when it is discovered, not at the end after continuing to build on top of it.

## 2. Reuse before rewriting

Before implementing anything non-trivial, search the current repo, relevant branches/commits/PRs, Elephant-Addons and the upstream/reference project.

If the requested behavior already exists:

- transplant, cherry-pick, merge or port the real implementation;
- preserve its dependencies, UI, state, persistence and error behavior;
- keep provenance of the source branch/commit/project;
- adapt only what is necessary for the current runtime.

Do not recreate an approximation from screenshots, summaries or memory when working source exists. For Excalidraw-, Muya- or other parity work, the reference codebase is the implementation specification: port behavior component-by-component rather than redesigning it.

Reinventing an existing solution requires a concrete demonstrated incompatibility, not a preference for a cleaner architecture.

## 3. Short implementation loop

For every slice, use this order:

1. Identify the exact user action and expected result.
2. Find/reuse the closest working implementation.
3. Reproduce the current failure or missing behavior when applicable.
4. Implement the smallest production-path change.
5. Compile/build immediately.
6. Execute the real path immediately.
7. Assert the visible result, persisted state and relevant error path.
8. Only after that passes, move to the next slice.

Never stack several unexecuted slices and hope CI will validate them later.

## 4. The user's manual test must become an automated test

The user must not be the first person to discover that Elephant does not start, a button does nothing, a request returns `fetch failed`, a file cannot be saved, or a migrated editor is incomplete.

For user-visible behavior, CI/acceptance should reproduce what the user would do:

- start the real application/runtime;
- use a clean temporary profile/vault when possible;
- perform the actual interaction;
- verify UI/state/persistence/side effect;
- restart when persistence/lifecycle matters;
- capture logs/artifacts on failure.

Unit tests are useful but do not replace the real path. A fake proves only the boundary it replaces.

If CI cannot execute the real scenario because CI itself is broken, fixing or replacing the executable verification path becomes the immediate task. Do not continue feature development underneath a broken verifier.

## 5. Preserve working Elephant behavior

`develop` and known-working historical branches are behavioral baselines unless the user explicitly requests a change.

A migration/extraction is not a redesign. Preserve as applicable:

- components, layout and navigation;
- labels/translations;
- keyboard, mouse, touch and mobile interactions;
- persisted settings/data migrations;
- loading, empty, success and error states;
- desktop/mobile behavior;
- logs and error visibility.

Do not delete the old working path before parity is demonstrated.

## 6. Add-ons and modularity

For add-on work:

- the add-on owns its own behavior;
- disabling/uninstalling it removes that behavior;
- no hidden fallback remains in core;
- expose only the smallest host API required by the real feature;
- do not invent broad generic APIs to make one port aesthetically clean;
- lifecycle and cleanup must be real, not registration-only.

Test the meaningful lifecycle: install/enable/use/restart/disable/uninstall/reinstall when the feature requires those states.

## 7. UI and editor parity

For Note/Markdown, Draw/Excalidraw and similar ports:

- prioritize behavioral parity over internal elegance;
- compare directly against the reference implementation;
- port existing algorithms/components/contracts instead of creating simplified substitutes;
- make basic editing interactions work before adding architecture around them;
- do not claim parity from screenshots or rendering alone;
- test input, selection, editing, undo/redo, persistence, shortcuts and the feature-specific interactions the user actually cares about.

## 8. Runtime/provider behavior

A runtime/provider is working only when the real executable/service and protocol path are exercised. Spawn/status alone is insufficient.

Verify as applicable: startup, handshake/discovery, real request, output/streaming, cancellation, persistence, error propagation and shutdown. Do not hardcode fake availability or silently route to another engine.

## 9. Logging

Logs exist to diagnose the real failing layer, not to satisfy a checklist.

For affected user actions, retain enough information to identify action start/end/failure, request correlation, relevant component/add-on, sanitized path/runtime identity, state transition and full error chain. Never log secrets.

Do not spend time building a logging framework when existing logging can expose the failure sufficiently.

## 10. Scope and architecture

Use modular code, but modularity is a means, not the deliverable.

- No opportunistic redesign during a bug/feature task.
- No mass refactor unless the requested feature is concretely blocked by the current structure.
- No arbitrary file-size refactor while the feature is unfinished.
- No new abstraction without at least one current production caller.
- No duplicate implementation of a working feature.
- Prefer existing components/services/test harnesses.

When a minimal patch and a broad architecture both solve the current requirement correctly, choose the minimal patch unless the user explicitly asks for architectural work.

## 11. Truthful but useful status

Never say `everything works`, `fully validated`, `production-ready` or equivalent without actual execution evidence.

But do not use `NOT PROVEN` as a substitute for delivery either. If required execution evidence is missing, that is an immediate blocker to resolve before continuing dependent work.

A useful handoff is concise:

- what user-visible behavior now works;
- what exact test/interaction actually ran;
- what still fails, if anything;
- the concrete blocker/fix, if blocked.

Do not bury the result under a long architecture report unless the user asked for one.

## 12. Definition of done

A task is done when the requested behavior works through the real production path on the relevant target, its important regression/error path is exercised, and the user should be able to perform the same interaction without discovering a fundamental failure first.

If that condition is not met, keep working on the blocker rather than expanding the scope.
