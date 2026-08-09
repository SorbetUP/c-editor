---
name: apex
description: >
  Structured implementation using the APEX method (Analyze → Plan → Execute →
  eXamine). Use when implementing a feature or fixing a bug that benefits from
  a clear, deliberate workflow instead of jumping straight to code. For clean
  code, maintainability, over-engineering, boilerplate, and unnecessary
  abstraction checks, apply the local Ponytail skill rules.
argument-hint: '[-a] [-x] [-t] [-v] [-s] [-e] [-k] [-m] [-pr] <task-description>'
---

# APEX

Implement `$ARGUMENTS` with a deliberate workflow. Think before each phase and load only the step needed next.

## Step files

- Init: `steps/step-00-init.md`
- Analyze: `steps/step-01-analyze.md`
- Plan: `steps/step-02-plan.md`
- Tasks: `steps/step-03-tasks.md`
- Execute: `steps/step-04-execute.md`
- Validate: `steps/step-05-validate.md`
- eXamine: `steps/step-06-examine.md`
- Resolve: `steps/step-07-resolve.md`
- Tests: `steps/step-08-tests.md`
- Verify: `steps/step-09-verify.md`
- Finish: `steps/step-10-finish.md`

Use `templates/run-report.md` when `-s` or a long run needs persistent notes.

## Flags

Lowercase turns a flag on; uppercase in a user request turns it off.

| Flag  | Meaning                                                                                            |
| ----- | -------------------------------------------------------------------------------------------------- |
| `-a`  | Auto-approve low-risk choices; still stop for data loss, security, or unclear acceptance criteria. |
| `-x`  | Run adversarial examine after validation.                                                          |
| `-t`  | Add or update tests for the behavior.                                                              |
| `-v`  | Verify the user-visible/runtime behavior, not just unit tests.                                     |
| `-s`  | Save/report step outputs using the run-report template.                                            |
| `-e`  | Economy mode: no parallel/subagent expansion; keep context narrow.                                 |
| `-k`  | Break larger work into dependency tasks.                                                           |
| `-m`  | Use task decomposition for independent work streams.                                               |
| `-pr` | Prepare PR-style final summary after validation.                                                   |

## Clean code rule

When this workflow asks for clean code, maintainability, or over-engineering review, use `../ponytail/SKILL.md` as the standard: smallest working diff, deletion over addition, no unrequested abstractions, stdlib/native platform first, and no fake or speculative scaffolding.

## Repo and language routing

At Init and Analyze, load `../repo-skill-router/SKILL.md` for non-trivial repo work. Detect the stack from manifests, configs, folders, workflows, and existing skills. If the current task depends on a major present language/framework/runtime and no focused skill exists, add or propose a small local skill for that surface before broad implementation.

For ElephantNote, route JavaScript/Vue/Vite/Vitest/Playwright and renderer bridge work to `../javascript-vue-runtime/SKILL.md`; route Rust/Cargo/Tauri command and `Elephant/backend/tauri` work to `../rust-tauri-runtime/SKILL.md`; route Tauri packaging/window confidence to `../tauri-ci-verifier/SKILL.md`.

## CI and verification routing

When the task touches CI, tests, Tauri, filesystem behavior, packaging, or release confidence, load the narrow skill that matches the APEX phase:

| APEX phase | Load these skills when relevant                                                                                                      |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| Init       | `../repo-skill-router/SKILL.md`                                                                                                      |
| Analyze    | `../repo-skill-router/SKILL.md`, `../ci-architect/SKILL.md`, `../github-actions-linter/SKILL.md`, `../cross-platform-paths/SKILL.md` |
| Plan       | `../ci-architect/SKILL.md`, `../anti-fake-tests/SKILL.md`, `../tauri-ci-verifier/SKILL.md`                                           |
| Execute    | `../github-actions-linter/SKILL.md`, `../runtime-ci-hardening/SKILL.md`, `../supply-chain-verifier/SKILL.md`                         |
| Validate   | `../anti-fake-tests/SKILL.md`, `../tauri-ci-verifier/SKILL.md`, `../ci-stability/SKILL.md`                                           |
| eXamine    | `../github-actions-security/SKILL.md`, `../artifact-release-gate/SKILL.md`, `../real-verification/SKILL.md`                          |

For ElephantNote, a green CI is not enough by itself: the selected gate must prove the user-visible or runtime contract touched by the change.

## A — Analyze

Gather just enough context to act with confidence:

- Use `Glob`/`Grep` to find the files and patterns you'll touch.
- Read the closest existing example and follow its conventions.
- Detect the repo language/runtime stack and load matching skills.
- Restate the task as 2-4 concrete acceptance criteria.
- Note open questions; ask only if a wrong assumption would be costly.

## P — Plan

Write a short, file-by-file plan before editing:

- List each file to create or change and what changes in it.
- Pick the simplest approach that satisfies the criteria.
- Apply Ponytail before choosing custom code: reuse existing code, stdlib, native platform features, or installed dependencies before writing new abstractions.
- Add/propose missing language/runtime skills only when the current repo stack and task require them.
- Order the steps so the code stays runnable along the way.

## E — Execute

Implement the plan:

- Match the surrounding code's naming, structure, and idioms.
- Stay strictly in scope — no "while I'm here" refactors.
- Comments only where intent is genuinely non-obvious.
- Run the formatter if the project has one.

## X — eXamine

Validate, then optionally review:

1. **Validate**: run `lint`, `typecheck`, and relevant tests. Fix only what you broke; re-run until clean.
2. **Review** (only if `-x`): re-read the diff as a skeptic and apply Ponytail, active runtime skills, and verification skills.

## Output

```md
## APEX complete

**Task:** {what was implemented}
**Criteria:** {✓ per acceptance criterion}
**Files changed:** {list}
**Validation:** ✓ lint ✓ typecheck ✓ tests
**Verification:** {runtime/workflow evidence or explicit reason skipped}
**Skills:** {loaded/added skills and why}
```

## Rules

- One step at a time — finish each phase before the next.
- Stay in scope; ship the smallest change that meets the criteria.
- Clean code review = Ponytail review, not subjective style polish.
- If blocked after 2 attempts, report the blocker and stop.
- Never claim full success without the relevant validation or verification evidence.
