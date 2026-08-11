# Strict Tauri/Freya differential runner

`tools/freya-differential/orchestrate.mjs` is the execution gate for the
scenario in `migration/freya/differential-scenarios.json`. It does not launch a
replacement UI and it does not create screenshots. It prepares an independent
fixture for each runtime, invokes the two commands supplied by the caller,
validates their evidence manifests, aligns metadata, and invokes the existing
`compare.mjs` PNG comparator.

## Invocation

```bash
node tools/freya-differential/orchestrate.mjs \
  --scenario migration/freya/differential-scenarios.json \
  --tauri-command '<real Tauri WebDriver capture command>' \
  --freya-command '<real freya-testing capture command>' \
  --output test-results/freya-differential/orchestrated
```

The output directory must be empty. Pixel tolerances are zero unless explicitly
provided with `--threshold` and `--max-different-ratio`. A non-zero exit is
returned for command failures, missing or extra actions/checkpoints/frames,
viewport or scale mismatches, state/vault mismatches, invalid provenance, and
PNG comparison failures.

## Capture command contract

Each command receives these environment variables:

- `DIFFERENTIAL_RUNTIME`: `tauri` or `freya`;
- `DIFFERENTIAL_SCENARIO_PATH`: absolute scenario path;
- `DIFFERENTIAL_OUTPUT_DIR`: the runtime artifact root;
- `DIFFERENTIAL_FIXTURE_ROOT`: the independently seeded fixture root;
- `DIFFERENTIAL_RUN_ID`: runtime-specific run id;
- `DIFFERENTIAL_COMMAND_SHA256`: digest of the configured command;
- `DIFFERENTIAL_CAPTURE_NONCE`: per-invocation nonce;
- `DIFFERENTIAL_EXPECTED_VIEWPORT_JSON`: scenario viewport JSON.

The command must echo `DIFFERENTIAL_CAPTURE_NONCE` as
`provenance.captureNonce` and write `<DIFFERENTIAL_OUTPUT_DIR>/manifest.json`.
The manifest
must contain `schemaVersion: 1`, the matching `scenarioId`, the runtime,
provenance, viewport and scale factors, the fixture file/hash list, every
scenario action in order with `status: "passed"`, every checkpoint with a
normalized state and vault snapshot, and every expected frame with:
`index`, action-relative `relativeMs`, relative PNG `path`, absolute
`sourcePath`, and the SHA-256 of the PNG bytes.

Tauri evidence must declare `driver: "webdriver"` and
`controlPlane: "webdriver"` for this scenario. Freya evidence must declare
`driver: "freya-testing"` and `controlPlane: "testing-runner"`. The runner
binds `runId`, `captureId`, command digest, and artifact root to the invocation;
it rejects synthetic evidence, shared source ids/roots, reused source files,
and identical declared evidence hashes. Equal PNG hashes alone are not rejected
because exact visual equality is the intended success condition.

Frame staging uses hard links when possible and copies only as a filesystem
fallback; the PNG bytes are hash-checked before staging and are never rewritten.
The comparison directory therefore contains the same pixels under aligned
checkpoint/frame names, while provenance checks still point to the original
runtime source files.

## Evidence output

The runner retains:

- `<runtime>/manifest.json` and `<runtime>.command.log`;
- `comparison-report.json` from `compare.mjs`;
- `comparison/<runtime>/...` aligned PNG paths;
- `diffs/` when the comparator finds changed pixels;
- `orchestration-report.json` with normalized metadata, all issues, command
  results, and the comparator report.

`status: "ok"` means the supplied commands satisfied this strict evidence
contract and the comparator returned success. It is not a product-parity claim
when a command is a fixture, a non-real runtime, or an unverified control plane;
the command provenance and retained logs remain part of the delivery evidence.
