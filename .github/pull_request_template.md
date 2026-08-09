## Change scope

- Requested behavior:
- Explicitly out of scope:
- Affected platforms:
- Affected add-ons/runtimes:

## Source provenance

- Known-good branch/commit/PR inspected:
- Code integrated from:
- Why this was integrated rather than recreated:

## UI and behavior preservation

- [ ] No existing UI was redesigned unless explicitly requested.
- [ ] Existing components, routes, selectors, translations and settings were preserved.
- [ ] Baseline screenshots/DOM/state were captured when UI ownership moved.
- [ ] No `MutationObserver`, timeout injection, generic settings insertion or duplicate UI was introduced.

## Add-on lifecycle, when applicable

- [ ] Absent before installation.
- [ ] Installed from the real package.
- [ ] Enabled and opened its real UI.
- [ ] Primary action produced the expected persisted/external effect.
- [ ] Error path was visible and logged.
- [ ] Restart preserved correct state.
- [ ] Disable removed all owned UI/actions/services.
- [ ] Uninstall removed all owned UI/actions/services/processes.
- [ ] Reinstall succeeded.
- [ ] Physical implementation is absent from the core bundle.

## AI/runtime proof, when applicable

- [ ] Real executable/service started.
- [ ] Real handshake succeeded.
- [ ] Provider/model discovery came from the runtime.
- [ ] Real production request returned/streamed through the UI.
- [ ] Cancellation was exercised.
- [ ] Persistence/restart was exercised.
- [ ] Timeout, stderr/non-zero exit or malformed response was surfaced and logged.
- [ ] Shutdown cleanup was exercised.

## Regression evidence

- Broken revision/commit:
- Exact pre-fix command/scenario:
- Pre-fix failure artifact:
- Regression test:
- Why this test fails for the real bug:
- Red-before-fix output:
- Green-after-fix output:
- Sabotage/mutation check performed:

## Real application evidence

- [ ] Exact final commit tested from a clean workspace.
- [ ] Real Tauri app executed.
- [ ] Packaged Tauri app executed for distributable/user-visible desktop changes.
- [ ] Android APK installed and exercised for Android changes.
- [ ] Required platform-specific build executed.

Commands actually executed:

```text

```

Logs and artifacts:

```text

```

## Test integrity

- [ ] No assertion was weakened to make the change pass.
- [ ] No snapshot was accepted without inspection.
- [ ] No mock replaces the behavior being claimed.
- [ ] No error is caught and ignored.
- [ ] Changed tests preserve the previous contract unless the user explicitly changed it.

Tests changed and justification:

```text

```

## Delivery status

- Status: `PROVEN` / `PARTIALLY PROVEN` / `NOT PROVEN` / `BLOCKED`
- Remaining limitations or missing proof:
