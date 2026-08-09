---
name: elephantnote-ci
description: >
  Rules for CI repair, workflow changes, build/test gates, Tauri CI, Docker sync
  tests, and honest green status.
argument-hint: '<ci-task>'
---

# ElephantNote CI

Use this skill for GitHub Actions, failing tests, build scripts, lint/typecheck, Tauri CI, Docker sync tests, release checks, and quality gates.

## Invariants

- CI passing must mean the relevant behavior is actually checked.
- Tests must keep meaningful assertions and useful coverage.
- Fix the implementation before changing a test expectation.
- Keep CI commands close to local developer commands.
- Mac/Tauri window/build confidence requires a real build/bootstrap check, not just JS unit tests.
- When GitHub Actions is available, final validation must include the workflow state for the head commit.

## GitHub CI loop

For repository changes, use GitHub Actions as the default validation source:

1. Record the head commit SHA.
2. Inspect workflow runs for that commit.
3. Inspect failed jobs and the failing step.
4. Repair the real cause with the smallest clean change.
5. Let the next commit trigger CI and inspect it again.
6. Report green, failed, pending, running, or blocked honestly.

Do not hand the user a local command checklist as the only validation when GitHub CI can be inspected.

## CI skill stack

Load these narrower skills when the task matches their surface:

- `../ci-architect/SKILL.md` for job topology, runner strategy, caching, and artifact policy.
- `../github-actions-linter/SKILL.md` for workflow YAML, contexts, triggers, and shell snippets.
- `../github-actions-security/SKILL.md` for workflow permissions and review of sensitive CI paths.
- `../anti-fake-tests/SKILL.md` for tests that must prove observable behavior.
- `../tauri-ci-verifier/SKILL.md` for Rust/Tauri bridge, packaged app, and window confidence.
- `../cross-platform-paths/SKILL.md` for filesystem behavior across OS and runtimes.
- `../ci-stability/SKILL.md` for intermittent checks and readiness conditions.
- `../runtime-ci-hardening/SKILL.md` for generated files, logs, cache scope, and runtime behavior.
- `../supply-chain-verifier/SKILL.md` for dependencies, lockfiles, license gates, and scanners.
- `../artifact-release-gate/SKILL.md` for bundle, checksum, diagnostics, and release evidence.
- `../ci-repair/SKILL.md` for the GitHub CI repair loop.

## Read first

- Failing workflow log and exact command.
- Workflow file and local package scripts.
- Tests that failed.
- Implementation touched by the failure.
- The APEX phase currently being executed.

## Procedure

1. Reproduce the failing command locally when useful.
2. Identify whether the failure is test bug, implementation bug, environment bug, or workflow bug.
3. Load the narrow CI skill that covers the failing surface.
4. Apply Ponytail before coding: smallest correct diff, reuse existing helpers, no speculative abstractions.
5. Fix the smallest real cause.
6. Inspect GitHub Actions for the pushed commit.
7. If CI must wait, report the run state honestly.

## Anti-slop

No broad ignore, no deleting coverage, no replacing a real e2e check with a weaker check.
