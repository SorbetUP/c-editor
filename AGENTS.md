# Elephant repository instructions — mandatory

These instructions apply to the entire repository. A more local `AGENTS.md` may add stricter rules, but it may not weaken or bypass this file.

## Repository truth

- `develop` is the current product baseline. Preserve its working editor, vault, navigation, settings, rendering, persistence, desktop and mobile behavior unless the user explicitly requests a change.
- The official add-on migration is not presumed correct merely because packages build or unit tests pass. Treat every migrated add-on as unverified until its real user path is proven.
- Historical working branches, commits and pull requests are source material. When a working implementation exists, integrate that implementation. Do not recreate an approximation from memory.
- The application, not the test suite, is the final source of truth.

## Prime directive

Preserve known-working behavior and make the smallest correct change. It is better to stop with a precise `BLOCKED` or `NOT PROVEN` report than to invent an implementation, weaken a test, hide an error, fake a runtime, or claim success without evidence.

## Mandatory workflow before editing

For every non-trivial change:

1. Inspect the current branch, working tree and relevant history.
2. Identify the exact working baseline and every source branch, commit or PR that already contains relevant behavior.
3. Trace the complete path from user action to UI, state, IPC/API, backend or add-on runtime, persistence and error handling.
4. Reproduce the reported failure before changing code.
5. Record the failing command, runtime log, screenshot, DOM state, persisted state or artifact.
6. Define what evidence will prove the fix, including which real application scenario must run.
7. Only then modify code.

If the failure cannot be reproduced, report that fact. Do not silently replace the task with a guessed implementation.

## Existing implementation and branch provenance

### Never rewrite a working feature by hand

When an earlier branch, commit or PR contains a working implementation:

- inspect its full diff and dependencies;
- merge, cherry-pick or transplant the actual implementation with traceable provenance;
- preserve its UI, state model, API contracts, tests and runtime wiring;
- resolve conflicts deliberately, file by file;
- document the source branch and commit SHA in the delivery report.

Forbidden:

- recreating the feature from screenshots or a summary while ignoring available source code;
- replacing a complete implementation with a smaller placeholder;
- copying only the visible UI while omitting backend, persistence or errors;
- deleting the original path before parity is demonstrated;
- calling an approximate reimplementation a merge.

If a clean integration is not possible, stop and explain the concrete conflict. Do not fabricate a substitute.

## UI preservation rule

Moving a core feature into an add-on is an extraction, not a redesign.

Unless the user explicitly requests a redesign, preserve:

- the existing components and layout;
- routes, tabs, panels and settings placement;
- class names, stable selectors and `data-testid` contracts;
- translations and labels;
- keyboard, mouse and mobile interactions;
- persisted settings and migration behavior;
- loading, empty, success and error states;
- accessibility behavior.

Before an extraction, capture the baseline with screenshots, DOM/selector checks and persisted-state fixtures. After the extraction, compare against that baseline.

Do not introduce replacement interfaces, generic settings cards, duplicate pages, extra wrappers, temporary panels or new navigation merely because they are easier to implement.

Forbidden UI integration patterns unless the task explicitly requires them and they are justified:

- `MutationObserver`-based feature injection;
- arbitrary timeouts used to wait for UI ownership;
- querying the first matching generic settings group and inserting content there;
- monkey-patching unrelated components;
- duplicating an existing component inside an add-on;
- hiding a regression with CSS.

## Add-on extraction and parity contract

A core-to-add-on migration is complete only when all of the following are proven.

### Ownership

- The implementation is physically absent from the core bundle when the add-on is not installed.
- The add-on owns its UI, commands, resources, services and persisted configuration through explicit host APIs.
- There is no hidden fallback to the old core implementation.
- Disabling or uninstalling the add-on leaves no active route, panel, command, toolbar item, background process, listener or service.

### Lifecycle

Test the complete lifecycle from a clean profile and clean vault:

1. app starts without the add-on;
2. add-on installs from its real package;
3. add-on enables;
4. its real UI opens;
5. its primary action produces the expected external or persisted effect;
6. its error path is visible and logged;
7. the app restarts and the state remains correct;
8. add-on disables and all owned behavior disappears;
9. add-on uninstalls and leaves no runtime residue;
10. add-on reinstalls and works again.

Packaging or registration alone is not functional validation.

### API discipline

- Add only the smallest reusable host API needed by the real feature.
- Keep APIs explicit, versioned where appropriate, validated and permission-scoped.
- Do not expose unrestricted internals merely to make one add-on easy to port.
- Do not preserve a hidden core dependency behind an add-on-shaped wrapper.
- Do not deprecate existing APIs during a migration unless the user explicitly requested it and compatibility is proven.

### Official add-on proof

For official add-on changes, run the existing focused tests and the real Tauri acceptance scenario. At minimum, use the relevant commands among:

```bash
node build/scripts/verify-agent-governance.mjs
pnpm test:unit
pnpm test:official-addons:e2e
pnpm test:desktop:acceptance
pnpm test:desktop:acceptance:packaged
pnpm prod:check
```

The exact command set depends on the changed surface, but unit-only proof is never enough for a user-visible add-on claim.

## AI and runtime engines, including Marvin

An AI provider, Marvin engine, Codex connection, open-model runtime, local model service or sidecar is considered available only after its real protocol and process are exercised.

A valid runtime proof must cover the applicable items:

- the real executable or service starts;
- executable path, version and launch arguments are logged without secrets;
- protocol handshake succeeds;
- provider and model discovery come from the runtime, not a hardcoded list;
- a real request is sent through the production path;
- streaming or incremental output reaches the actual UI when supported;
- cancellation reaches the runtime;
- conversation/configuration persistence works after restart;
- runtime stderr, non-zero exits, malformed replies and timeouts surface as visible errors;
- shutdown cleans up the process;
- logs let a developer identify the exact failing layer.

Forbidden:

- hardcoded `connected`, `available`, model lists or `{ ok: true }` responses;
- treating process spawn as proof that inference works;
- treating a status endpoint as proof that chat works;
- replacing the real runtime with a mock and then claiming the engine works;
- silently routing Marvin or another provider to a different engine;
- swallowing provider errors and returning an empty response;
- showing an enabled UI when the backend path is absent.

A fake server or subprocess may validate protocol parsing deterministically, but it proves only the protocol adapter. It does not prove the real Marvin, Codex, llama, OpenCode or provider runtime. Report those proof levels separately.

## Ponytail-style engineering constraints

Use a modular, explicit architecture instead of accumulating patches.

- Prefer small, focused modules with one responsibility. As a guideline, handwritten files should remain near or below 200 lines; justify exceptions.
- Separate UI, domain state, host API, platform/runtime adapter and persistence.
- Reuse existing components and services instead of copying them.
- Keep dependencies directional and explicit.
- Validate data at boundaries.
- Make ownership and lifecycle cleanup obvious.
- Use typed or schema-checked contracts where the codebase supports them.
- Keep functions small enough to understand and test.
- Remove dead compatibility code only after parity is proven.

Forbidden without explicit justification:

- giant components or services mixing unrelated concerns;
- duplicate implementations of the same feature;
- global mutable registries with hidden side effects;
- broad catch blocks that suppress errors;
- boolean success returns that discard diagnostic information;
- copy-pasted provider or add-on logic;
- unrelated refactors in a bug-fix PR;
- generated-looking boilerplate that is not wired into production.

If the requested change cannot be implemented cleanly within the available context, report the limitation instead of adding architectural debt.

## Logging and observability contract

Logs are part of the feature. Adding one generic line such as `started` or `failed` is not sufficient.

For every affected user action, capture the applicable events across renderer, host/Tauri and add-on runtime:

- action start, completion and failure;
- correlation/request ID;
- add-on ID, command/resource/service name;
- sanitized paths and selected runtime executable;
- relevant payload shape, never credentials or private note content unless the test fixture is synthetic;
- state transition;
- duration;
- exit code, signal and stderr summary for subprocesses;
- full error chain or stack at the layer where it is handled;
- cleanup and shutdown result.

Errors must be both visible to the user and present in retrievable logs. Do not catch and ignore errors. Do not print secrets, tokens, authorization headers or raw credentials.

When a task asks for logs, prove the log entries by running the failing or successful scenario and include the resulting log path or captured artifact. Merely adding logging statements is not runtime proof.

## Test integrity rules

Read and obey:

- `agent/skill/test-integrity/SKILL.md`
- `agent/skill/truthful-delivery/SKILL.md`
- `agent/skill/real-implementation/SKILL.md`
- `agent/skill/completion-audit/SKILL.md`
- `agent/skill/elephant-change-safety/SKILL.md`

### Red-before-green requirement

For every bug fix or regression:

1. reproduce the broken behavior on the pre-fix revision;
2. add or identify a test that fails for that exact behavior;
3. retain the failure output;
4. apply the smallest fix;
5. show the same test passing afterward;
6. run the broader relevant suites;
7. run the real application scenario.

When practical, temporarily break the implementation after writing a new test to demonstrate that the test turns red, then restore the implementation. A test that remains green when the behavior is removed is not evidence.

### Tests may not redefine success

Do not:

- weaken or delete an assertion to accommodate a regression;
- update snapshots without manually inspecting the behavioral change;
- change expected UI or API output unless the task explicitly changes the contract;
- mock the implementation being claimed;
- test a reduced helper and claim the complete product path works;
- accept any truthy value when a precise effect can be asserted;
- catch the tested error and ignore it;
- skip a failing test without an explicit user-approved reason;
- mark flaky or platform-dependent behavior as passing without evidence.

Every changed test must state which product behavior it protects and which concrete defect makes it fail.

## Real application validation

Compilation, linting, unit tests, package creation and mocked browser tests are separate evidence classes. None alone proves that Elephant works.

For desktop behavior, use the real Tauri application and, when the claim concerns distributable behavior, the packaged application:

```bash
pnpm test:desktop:acceptance
pnpm test:desktop:acceptance:packaged
```

The acceptance run must use a clean temporary profile/vault and retain renderer, Tauri and add-on logs plus generated artifacts.

For Android behavior, build and install the new APK on an emulator or physical device, clear prior app state when appropriate, exercise the real permission and document-provider flow, kill and relaunch the process, and retain `adb logcat` plus screenshots/video. Desktop simulation does not prove Android behavior.

For macOS-specific behavior, run the real macOS build. For Linux- or Windows-specific behavior, run that platform. Do not claim cross-platform validation from one OS.

## Clean-state and regression requirements

Before final delivery:

- validate from the exact final commit;
- use a clean checkout or equivalent clean CI workspace;
- remove stale build outputs when they could affect the result;
- use a fresh test vault/profile for lifecycle tests;
- verify that no unrelated feature regressed;
- inspect the final diff for accidental generated files, duplicate code, test weakening and missing cleanup.

## Scope control

- One bug or bounded migration at a time.
- No opportunistic redesign.
- No mass refactor bundled with a fix.
- No new framework, abstraction or custom language unless the existing acceptance harness cannot express the required scenario and the limitation is demonstrated.
- Extend the existing observable Tauri acceptance runner before inventing a second automation system.
- Keep commits reviewable and attributable.

## Delivery and claim policy

Every final claim must use one of these states:

- `PROVEN`: exact runtime/test evidence from the final commit exists.
- `PARTIALLY PROVEN`: some evidence exists, but a required platform/runtime check is missing.
- `NOT PROVEN`: implementation or static inspection exists without sufficient execution evidence.
- `BLOCKED`: a named blocker prevents proof or completion.
- `OUT OF SCOPE`: deliberately excluded.

A delivery report must include:

- final commit SHA and branch;
- source branch/commit provenance when code was integrated;
- files and behavior changed;
- exact commands executed;
- pre-fix failure evidence;
- post-fix evidence;
- runtime environment and platform;
- log and artifact paths;
- tests changed and why;
- what remains unproven or broken.

Forbidden unsupported phrases include:

- `everything works`;
- `all tests pass` without exact command and final result;
- `fully validated` without real runtime proof;
- `the engine works` after only a status probe;
- `logs were added` without an executed log artifact;
- `merged the branch` when the feature was manually recreated;
- `production-ready` without packaged and platform-appropriate validation.

## Definition of done

A task is done only when the requested product behavior is implemented through the real production path, the original regression is proven, meaningful tests fail before and pass after, the real application scenario succeeds on the required platform, logs and artifacts are retained, the final diff is clean, and every remaining limitation is reported honestly.

When any required item is missing, the task is not done. State the missing proof instead of pretending otherwise.
