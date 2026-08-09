---
name: repo-skill-router
description: >
  Detect the repository languages, frameworks, runtimes, package managers,
  test tools, CI surfaces, and missing local skills before an APEX run. Use this
  in APEX Init and Analyze so the model loads or proposes the right skills for
  the actual repo instead of working generically.
argument-hint: '<repo-task>'
---

# Repository Skill Router

Use this skill at the beginning of non-trivial repo work.

## Purpose

Before changing code, identify the repository stack and load the matching skills. If a stack surface is important but no skill exists, create or propose a small local skill for it before implementation.

## Stack detection checklist

Inspect lightweight evidence first:

- manifests: `package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`, `pom.xml`, `build.gradle`, `composer.json`, `Gemfile`, `deno.json`, `bun.lockb`, `pnpm-lock.yaml`;
- CI files: `.github/workflows/**`, `.github/actions/**`;
- runtime folders: `Elephant/backend/tauri`, `android`, `ios`, `electron`, `web`, `server`, `tests`, `test`, `e2e`;
- config files: `vite.config.*`, `vitest.config.*`, `playwright.config.*`, `tsconfig*.json`, `eslint.config.*`, `rustfmt.toml`, `clippy.toml`;
- docs and agent files: `README.md`, `agent/AGENTS.md`, `agent/skill/**`.

## Skill routing rules

- JavaScript or TypeScript: load a JS/TS runtime, package-manager, test, lint, and bundler skill when present.
- Vue or frontend UI: load UI, accessibility, component-test, and visual/runtime verification skills when present.
- Rust: load Rust, cargo, error-handling, path-safety, and FFI/bridge skills when present.
- Tauri: load `../tauri-ci-verifier/SKILL.md` and the project Tauri skill.
- Electron: load Electron/preload/security bridge skills when present.
- GitHub Actions: load `../ci-architect/SKILL.md`, `../github-actions-linter/SKILL.md`, and `../github-actions-security/SKILL.md`.
- Filesystem/vault/sync: load `../cross-platform-paths/SKILL.md`, `../anti-fake-tests/SKILL.md`, and the relevant ElephantNote project skill.
- Dependencies or release: load `../supply-chain-verifier/SKILL.md` and `../artifact-release-gate/SKILL.md`.

## Missing-skill rule

When the repo has a major language/framework/runtime and no local skill covers it:

1. Name the missing skill using `agent/skill/<domain>/SKILL.md`.
2. Keep it small and concrete: invariants, files to read, commands, tests, and anti-patterns.
3. Add it to `agent/skill/README.md`.
4. Add a guard or test when the skill is important to future maintenance.
5. Do not create speculative skills for technologies not present in the repo.

## ElephantNote detected baseline

ElephantNote currently includes JavaScript/Vue/Vite/Vitest/Playwright, Rust/Cargo/Tauri, Electron preload/runtime bridges, GitHub Actions, Docker sync smoke flows, Markdown/vault filesystem behavior, Android/Tauri mobile scripts, and local AI/model runtime code. Load project skills narrowly for the files touched.

## APEX integration

- Init: detect stack and select initial skills.
- Analyze: refine routing after files are found.
- Plan: add missing skill only if it would change implementation quality now.
- Validate: verify that newly added skills are listed and guarded.
