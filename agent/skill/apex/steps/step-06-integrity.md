# APEX Integrity Note

Use during review and final response for non-trivial repo work.

## Related skills

- `../../truthful-delivery/SKILL.md`
- `../../real-implementation/SKILL.md`
- `../../test-integrity/SKILL.md`
- `../../completion-audit/SKILL.md`
- `../../connector-retry-discipline/SKILL.md`
- `../../elephant-change-safety/SKILL.md`

## Checks

- Each important statement has a proof state: `PROVEN`, `PARTIALLY PROVEN`, `NOT PROVEN`, `BLOCKED`, or `OUT OF SCOPE`.
- Known-good branch or commit provenance was inspected before recreating or replacing code.
- Existing UI and product contracts were preserved unless the user explicitly requested a change.
- Features are connected to real code paths, durable state, errors and cleanup.
- Add-on migrations prove install, enable, real use, restart, disable, uninstall and physical core absence when applicable.
- AI/Marvin/runtime claims distinguish adapter mocks from real runtime and UI proof.
- Tests check the behavior they name, fail when that behavior is broken, and do not replace the implementation with mocks.
- User-visible claims include real Tauri or platform runtime evidence, not only compilation or unit tests.
- Logging claims include executed log artifacts with enough detail to locate the failing layer.
- Connector failures are retried with a changed strategy.
- Final response names pending, unproven or blocked work explicitly.
