---
name: elephantnote-sync
description: >
  Rules for real local-first sync across desktop/mobile, rclone, LAN pairing,
  conflict behavior, and two-vault verification.
argument-hint: '<sync-task>'
---

# ElephantNote Sync

Use this skill for sync settings, device pairing, rclone, LAN peer protocol, mobile/desktop sync, conflict handling, and sync UI.

## Invariants

- Sync must move real files, not only update UI status.
- Local-first vault data remains usable offline.
- Pairing must be understandable: device list, pair action, selected vaults, status, error recovery.
- A sync run must have observable logs and a clear result.
- Conflict behavior must be explicit and tested.
- Desktop, mobile, macOS, and Docker/dev flows must not depend on an external service for the core local/LAN path.

## Read first

- `Elephant/backend/js/sync/**`.
- `Elephant/shared/sync.js`.
- `Elephant/frontend/app/components/settings/SyncSettingsPanel.vue`.
- Unit tests for rclone, LAN protocol, sync contract, and production flow.
- `.github/workflows/sync-docker.yml` when CI is involved.

## Verification

- Two vaults or two test instances.
- Create, modify, delete, and conflict cases.
- Confirm files really exist on both sides after sync.
- Confirm UI status follows the real sync result.
- Confirm logs explain failures.

## Anti-slop

No fake paired device, fake progress bar, fake successful sync, or hidden network dependency.
