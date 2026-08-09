# External Addon Platform Proof

`examples/addons/platform-proof` is the reproducible acceptance test for the public addon API.

It verifies the following real path:

```text
.enaddon archive
→ Rust package validation and extraction
→ per-vault registry
→ Community Addons consent
→ isolated JavaScript Worker
→ command registration
→ Rust capability broker
→ private storage
→ scoped note write
→ scoped note read-back
→ result returned to the UI
→ vault refresh
→ generated note opened in the editor
```

## Build the installable package

```bash
node build/scripts/package-addon.mjs \
  examples/addons/platform-proof \
  build/addons/platform-proof.enaddon
```

The packager is dependency-free and produces a standard ZIP-based `.enaddon` archive.

## Manual acceptance test

1. Start ElephantNote with an active vault.
2. Open **Settings → Addons**.
3. Turn on Community Addons.
4. Select **Install** and choose `build/addons/platform-proof.enaddon`.
5. Enable **Addon Platform Proof**.
6. Expand the addon and run **Run addon platform proof**.
7. Confirm that `Addon Proof/External Addon Platform Proof.md` opens automatically.
8. Run the command again and confirm that the stored proof counter increments from 1 to 2.

A successful note contains a marker similar to:

```text
ELEPHANT_ADDON_PROOF:2:2026-07-09T23:00:00.000Z
```

The incrementing counter is stored under:

```text
.elephantnote/addons/data/com.elephantnote.examples.platform-proof/storage.json
```

This makes the proof distinguishable from a static demo or an in-memory mock.

## Automated validation

The `Addon Platform Validation` workflow:

- executes the platform-proof addon source against the public API contract;
- verifies two runs and persistent storage semantics;
- packages the installable `.enaddon` file;
- validates the ZIP archive;
- builds the Tauri web bundle;
- uploads the installable proof package with the validation logs.
