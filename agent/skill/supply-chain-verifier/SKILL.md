---
name: supply-chain-verifier
description: >
  Review dependency, lockfile, license, audit, CodeQL, Dependabot, binary,
  and generated-artifact changes for ElephantNote.
argument-hint: '<dependency-or-release-task>'
---

# Supply Chain Verifier

Use this skill when a task touches dependencies, lockfiles, release workflows, bundled binaries, generated artifacts, audit output, or license validation.

## Required checks

- Keep lockfiles updated with the dependency manifest.
- Run the repository license validation when dependencies change.
- Keep Dependabot coverage for npm/pnpm, cargo, and GitHub Actions where present.
- Keep CodeQL or equivalent code scanning enabled for supported languages.
- Verify generated files are intentional and not stale build output.
- Keep binary downloads explicit, versioned, and reproducible where possible.

## ElephantNote-specific concerns

- Tauri helper binaries must not be silently replaced by untracked downloads.
- Local AI runtime dependencies must be optional when the feature is optional.
- Release artifacts should have checksums or provenance when used outside CI.
- CI artifacts must not include private vaults, secrets, local model caches, or full dependency caches.
- License gates must remain blocking for release/build flows.

## Review checklist

1. What dependency or generated artifact changed?
2. Which lockfile or manifest proves the version?
3. Which CI job validates license/security/build impact?
4. Is the artifact reproducible from source or intentionally checked in?
5. Could the artifact contain user data, tokens, or local machine paths?

## APEX integration

- Analyze dependency/artifact scope.
- Plan the smallest validation gate.
- Execute without broad updates unrelated to the task.
- Validate with license, audit, build, or scanner evidence.
- eXamine for accidental user data or stale generated files.
