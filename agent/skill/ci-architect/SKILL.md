---
name: ci-architect
description: >
  Design GitHub Actions CI topology for ElephantNote: job boundaries, matrix shape,
  dependency caching, runtime gates, artifact policy, and PR/release separation.
  Use during the APEX Analyze and Plan phases before editing workflow YAML.
argument-hint: '<ci-design-task>'
---

# CI Architect

Use this skill when a task changes CI topology, workflow jobs, runner selection, cache strategy, or release gates.

## Goal

A green CI must mean the project passed the smallest set of checks that gives real confidence for the touched surface. Do not make one monolithic job that hides which layer failed.

## Required job layers

For ElephantNote, prefer these layers unless the change is intentionally narrower:

1. **Workflow hygiene**: workflow syntax/lint/static security checks.
2. **Quality gate**: critical-flow guard, security guardrails, lint, unit/contract tests.
3. **Tauri Rust**: `cargo check` and Rust unit tests for `Elephant/backend/tauri`.
4. **Tauri web build**: `pnpm tauri:web:build`.
5. **Packaged runtime**: macOS packaged app/window smoke when the task concerns Tauri launch confidence.
6. **Sync/runtime smoke**: Docker pair or local runtime checks when the task touches sync, filesystem, or IPC.
7. **Release gate**: artifacts, checksums, provenance, installability, and bundle diagnostics.

## Design rules

- Keep fast PR checks separate from expensive packaging checks.
- Use `concurrency` to cancel obsolete runs on the same PR/ref.
- Use `permissions: contents: read` by default; add write permissions only to the exact job that needs them.
- Cache package managers and cargo outputs with keys tied to lockfiles.
- Upload artifacts only for diagnostics or release evidence; avoid massive always-on bundle uploads.
- Prefer local developer commands in CI so failures are reproducible.
- Treat macOS runner minutes as scarce: only package there when the evidence requires it.

## APEX integration

- **Analyze**: identify which project surface changed and which existing job proves it.
- **Plan**: add or modify only the necessary jobs and scripts.
- **Execute**: keep YAML minimal and explicit.
- **Validate/eXamine**: prove the new gate can fail for the right reason and does not silently pass.

## Anti-patterns

- One giant `ci` job with every command and no diagnostic split.
- `continue-on-error` on a blocking correctness check.
- Matrix explosion without a reason.
- Uploading full app bundles on every PR when only logs are needed.
- Green workflows that skip the exact path changed by the PR.
