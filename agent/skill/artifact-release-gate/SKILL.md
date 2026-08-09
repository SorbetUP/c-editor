---
name: artifact-release-gate
description: >
  Verify release and build artifacts: bundle existence, installability, size,
  diagnostics, checksums, provenance, and platform-specific packaging evidence.
argument-hint: '<artifact-or-release-task>'
---

# Artifact Release Gate

Use this skill when a task touches packaged builds, release workflows, upload-artifact steps, checksums, app bundles, installers, icons, or distribution evidence.

## Required evidence

- The expected artifact exists at the expected path.
- The artifact has a non-zero, reasonable size.
- The bundle contains required resources such as icons, config, and app metadata.
- Launch or inspect the app at the platform level when launch behavior is in scope.
- Diagnostics are uploaded only when useful and size-bounded.
- Release artifacts have checksums or provenance when the workflow publishes them.

## ElephantNote checks

- macOS Tauri `.app` bundle exists after the packaged build command.
- `pnpm tauri:mac:smoke` proves the packaged app opens a visible window when that is the target.
- Electron reference builds must stay separate from Tauri proof unless the task is Electron parity.
- Upload paths must avoid full vault data, local model caches, and dependency caches.
- Icon checks must inspect the actual resource used by the bundle, not only the source asset.

## Artifact policy

- PR artifacts: logs, generated docs, coverage, small diagnostics.
- Failure artifacts: bundle diagnostics only when they help debug the failure.
- Release artifacts: installers/bundles, checksums, provenance, and minimal metadata.

## APEX integration

- Analyze which artifact is supposed to prove the task.
- Plan the smallest inspection or launch check.
- Execute with explicit artifact names and paths.
- Validate by checking existence, size, and relevant content.
- eXamine for unsafe content and useless large uploads.
