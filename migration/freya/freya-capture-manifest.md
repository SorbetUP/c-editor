# Freya differential capture contract

`Elephant/freya/tests/differential_freya_capture.rs` is the real Freya side of
`migration/freya/differential-scenarios.json`. It is an adapter for the existing
application, not a second fixture or a self-comparison. The orchestrator owns
the fixture, run id, nonce, command digest, output directory and viewport
environment; the test must refuse to run without those values.

## Evidence rules

Every shared action is driven in order through `TestingRunner` pointer, click,
key, text and scroll APIs. A checkpoint is appended only after the action's
real postcondition has been checked. The manifest therefore cannot contain a
`passed` action for an action that only emitted frames. The adapter captures
the exact shared timestamps and action ids: `launch` has 3 frames and the
remaining 13 actions have 9 frames each, for 120 PNG frames total.

The state snapshot is evidence from the current Freya accessibility tree,
rendered geometry, rendered frame hashes and the fixture vault. It does not
use a journey bookkeeping object. In particular:

- `searchVisible`, menu visibility, visible entries, labels and geometry come
  from `TestingRunner.find_many` and `TestingNode.layout()`.
- scroll change is proved by distinct PNG hashes for the addressed `Editor
  scroll` target; the shared positive native wheel delta is translated to the
  negative Freya `TestingRunner` content delta because Freya's wheel API uses
  the opposite Y convention.
- rail order is read back from the real persisted
  `.elephantnote/workspace.json`; the drag also requires changed rendered
  frames and changed real rail geometry.
- hover actions require changed rendered frames. Hovered labels are not
  asserted from a Rust-side variable because the current accessibility tree
  does not expose hover state.
- the current Freya `Input` component does not expose its private value through
  the accessibility node. A non-empty search checkpoint therefore fails with
  a named `HARD_ISSUE` until the production element exposes that model value.

The adapter is split by responsibility under `Elephant/freya/tests/support/`:
`differential_scenario` owns contract parsing, environment binding and the
orchestrator fixture; `differential_frames` owns PNG timelines;
`differential_ui` owns accessibility/tree queries;
`differential_timelines` owns pointer, scroll and drag motion;
`differential_actions` owns the fourteen real postconditions; and
`differential_snapshot` plus `differential_manifest` own state/vault evidence
and final serialization. The test file is only the run orchestrator. None of
these modules stores a substitute journey state.

The search action is deliberately exactly the shared `write-text` action. The
adapter does not press Enter. The production Freya search currently submits
from `Input::on_submit` (keyboard Enter) rather than from the text change, so
the real run must fail with `HARD_ISSUE search-alpha` if the production
`on_input` path is still absent. Adding Enter would make the action sequence
different and would invalidate the differential evidence.

## Target provenance

The shared contract now records proven mappings for the formerly stale targets:

- `Editor scroll` is the real labeled scroll surface in
  `Elephant/freya/src/app/editor_view.rs#note_editor_host`, exercised by
  `Elephant/freya/tests/editor_lifecycle_freya_testing.rs#scroll_compaction_and_restart_state_boundary_is_explicit`.
- `Search` to `Hide sidebar` is the real rail drag path in
  `Elephant/freya/src/app/navigation.rs#rail_action`, exercised by
  `Elephant/freya/tests/shell_navigation_freya_testing.rs#rail_search_dragged_before_sidebar_toggle_persists_and_restores`.

The old `not-exposed` and hard-mismatch checkpoint declarations were removed
only after those source labels and real tests were inspected. The orchestrator
scenario validator rejects a required mapping that is still marked
`not-exposed` after it has provenance.

## Validation commands

Validate the scenario contract:

```sh
node --input-type=module -e 'import { loadScenario } from "./tools/freya-differential/lib/scenario.mjs"; await loadScenario("migration/freya/differential-scenarios.json"); console.log("scenario-valid")'
```

The historical partial adapter was intentionally retained as red evidence at
`/private/tmp/freya-red-before/manifest.json`. It is rejected by the
orchestrator manifest validator because it has no schema version, shared
scenario id, provenance, exact actions, checkpoints or PNG evidence. Validate
that failure without comparing it to itself:

```sh
node --input-type=module -e 'import { readFile } from "node:fs/promises"; import { loadScenario, materializeFixture } from "./tools/freya-differential/lib/scenario.mjs"; import { validateManifest } from "./tools/freya-differential/lib/evidence.mjs"; const scenario = await loadScenario("migration/freya/differential-scenarios.json"); const fixture = { id: scenario.fixture.id, files: [] }; const manifest = JSON.parse(await readFile("/private/tmp/freya-red-before/manifest.json")); const result = await validateManifest(manifest, { runtime: "freya", outputRoot: "/private/tmp/freya-red-before", runId: "red-before", commandSha256: "red-before", captureNonce: "red-before", scenario, fixture }); console.log(JSON.stringify({ issueCount: result.issues.length, issues: result.issues }, null, 2));'
```

Run the real adapter through the shared 14-action orchestrator (the output
directory must be empty):

```sh
node tools/freya-differential/orchestrate.mjs \
  --scenario migration/freya/differential-scenarios.json \
  --output /private/tmp/freya-differential-final \
  --tauri-command 'node tools/tauri-visual-capture/index.mjs' \
  --freya-command 'cargo test --manifest-path Elephant/freya/Cargo.toml --test differential_freya_capture -- --nocapture --test-threads=1'
```

The final runtime report must be read from
`/private/tmp/freya-differential-final/orchestration-report.json`, with the
per-runtime command logs beside it. A non-zero Freya command, a missing
manifest, a missing frame, a failed action or an unproven field is a blocker;
it is not converted into a passing manifest.
