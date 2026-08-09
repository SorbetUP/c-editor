---
name: runtime-ci-hardening
description: >
  Inspect what CI jobs actually do at runtime: process behavior, network use,
  generated files, caches, and logs. Use for release, packaging, and external
  download workflows.
argument-hint: '<runtime-ci-task>'
---

# Runtime CI Hardening

Use this skill when static workflow review is not enough: packaging, release, model/runtime downloads, generated binaries, or jobs that call external tooling.

## Runtime review

- Identify commands that download, generate, or execute binaries.
- Confirm logs show expected commands and no unrelated noisy side effects.
- Keep network access and external calls limited to the job that needs them.
- Keep generated files inside expected project paths.
- Keep caches narrow and tied to lockfiles.
- Inspect failure artifacts before assuming the build problem is solved.

## ElephantNote focus areas

- Tauri helper binary setup.
- Tauri packaged builds.
- Electron reference bundle generation.
- Local model runtime setup.
- Sync Docker workflows.
- License and dependency generation scripts.

## APEX integration

- Analyze which runtime side effects are expected.
- Plan the smallest observation: logs, generated file list, artifact path, or command output.
- Execute without broadening permissions.
- Validate that the expected side effect occurred.
- eXamine for unexpected generated files, oversized artifacts, and unrelated external calls.
