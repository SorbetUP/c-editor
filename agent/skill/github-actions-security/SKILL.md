---
name: github-actions-security
description: >
  Harden GitHub Actions workflows against secret leaks, privilege escalation,
  unsafe PR execution, and supply-chain attacks. Use for CI security review and
  APEX eXamine when workflow files change.
argument-hint: '<workflow-security-task>'
---

# GitHub Actions Security

Use this skill when a task touches workflow permissions, secrets, third-party actions, PR triggers, release publishing, artifacts, or CI hardening.

## Required security posture

- Default to `permissions: contents: read`.
- Grant write permissions only in the job that needs them and only for the needed scope.
- Avoid `pull_request_target` unless the workflow never executes untrusted code.
- Pin high-risk third-party actions or justify why version tags are acceptable.
- Never expose secrets to fork PR code, build logs, generated files, artifacts, or caches.
- Treat downloaded binaries and `curl | bash` patterns as hostile until verified.

## Review checklist

1. Identify every secret or token the workflow can access.
2. Identify every step that runs code from the PR branch.
3. Confirm the checkout ref is safe for the event type.
4. Confirm artifacts cannot smuggle secrets or private config.
5. Confirm caches do not include tokens, credentials, or local vault data.
6. Confirm release jobs are isolated from PR jobs.
7. Confirm security scanning remains blocking where it protects users.

## ElephantNote-specific checks

- Tauri packaging must not upload private signing material.
- Sync tests must not persist real user vault data in artifacts.
- Local AI/model download checks must not trust unverified remote scripts.
- Electron preload and Tauri bridge security regressions must stay covered by the critical-flow guard.

## Tools to use when available

- `zizmor` or equivalent workflow security scanner.
- CodeQL for code scanning.
- Dependabot for dependency vulnerability PRs.
- Runtime hardening or network egress review for release jobs.

## Red flags

- `secrets: inherit` in broad reusable workflows.
- `GITHUB_TOKEN` with write scopes in a job that builds PR code.
- Debug commands printing environment variables.
- Uploading entire home, workspace, target, or cache directories as artifacts.
