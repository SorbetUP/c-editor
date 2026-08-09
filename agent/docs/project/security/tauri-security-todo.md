# Tauri security hardening todo

This document tracks the security work required before the Tauri build can be considered safe for public distribution.

Status legend:

- `[ ]` Not started
- `[~]` Partially implemented or guarded, but not release-safe
- `[x]` Done

## Release gate

The Tauri build must not be considered release-ready until all `Critical` and `High` tasks below are closed or explicitly re-reviewed with a narrow, documented residual risk.

## Critical

### [ ] Narrow Tauri filesystem capabilities

Current risk:

- `src-tauri/capabilities/default.json` still grants broad filesystem access to the main window.
- The current capability includes `fs:default`, direct read/write/mkdir/remove/rename/copy/stat permissions, and broad scopes such as `$HOME/**/*`, `$DOCUMENT/**/*`, `$DOWNLOAD/**/*`, and `$DESKTOP/**/*`.
- This weakens the Rust-side vault guards because any renderer compromise can call the Tauri FS plugin directly over broad user directories.

Required work:

- Remove `fs:default` from the main capability.
- Remove direct write-like filesystem permissions from the renderer where possible: `fs:allow-write-file`, `fs:allow-remove`, `fs:allow-rename`, `fs:allow-copy-file`, `fs:allow-mkdir`.
- Route note, attachment, drawing, model, and config writes through Rust commands that canonicalize paths and enforce vault/model-dir boundaries.
- Replace broad scopes with app-data-only scopes plus explicit user-selected vault grants when Tauri supports the final permission model needed by the app.
- Remove the accepted baseline entries for broad filesystem scopes once fixed.

Acceptance tests:

- A frontend call to the FS plugin cannot read `$HOME/.ssh/*`.
- A frontend call to the FS plugin cannot write/delete files outside the active vault.
- `pnpm security:guard` fails if `fs:default` or broad `$HOME/**/*`-style scopes are reintroduced.

### [ ] Harden production CSP and disable global Tauri API exposure

Current risk:

- `src-tauri/tauri.conf.json` has `withGlobalTauri: true`.
- The CSP currently allows `unsafe-inline`, `unsafe-eval`, `wasm-unsafe-eval`, `blob:`, broad localhost connections, and multiple remote hosts.
- A renderer XSS would therefore have a powerful bridge to native APIs.

Required work:

- Set `withGlobalTauri` to `false` for production.
- Replace global Tauri usage with explicit imports from `@tauri-apps/api`.
- Remove `unsafe-inline` and `unsafe-eval` from production CSP.
- Keep `wasm-unsafe-eval` only if a documented dependency requires it.
- Replace `http://localhost:*` and `http://127.0.0.1:*` with exact ports used by the app.
- Split dev CSP and release CSP if development still needs relaxed rules.

Acceptance tests:

- Production build has no `window.__TAURI__` global.
- Production CSP does not contain `unsafe-inline` or `unsafe-eval`.
- App still renders editor, graph, model library, and chat under the stricter CSP.

### [ ] Disable arbitrary external llama-server execution in release

Current risk:

- The local runtime can use a configured `llamaServerPath` / `serverPath` / `llamaBinary`.
- The current basename validation only ensures the executable is named like `llama-server`; it does not prove the binary is trusted.
- A malicious or compromised renderer could point to a fake local executable named `llama-server`.

Required work:

- In release builds, only allow app-managed llama-server binaries from `resource_dir` or `app_local_data_dir`.
- Validate the selected binary path by canonicalizing it and checking it is inside an app-managed runtime directory.
- Add an optional SHA-256 allowlist for downloaded or bundled runtime binaries.
- Gate arbitrary path mode behind debug builds or an explicit unsafe developer setting that is unavailable in distributed builds.
- Log a clear warning when path mode is rejected.

Acceptance tests:

- Release-mode validation rejects `/tmp/llama-server` even if it exists.
- Release-mode validation accepts only the bundled/app-managed runtime path.
- Debug-mode path override remains testable if intentionally kept.

## High

### [ ] Restrict model download sources and size

Current risk:

- `tauri_models_download` accepts `hf:` and arbitrary `http(s)` URLs.
- Downloads are streamed to disk without a clearly enforced maximum size, host allowlist, or localhost/private-network rejection.
- This allows SSRF-like requests, network probing, untrusted model sources, and disk exhaustion.

Required work:

- Prefer `hf:owner/repo/file.gguf` as the only default model download format.
- If generic HTTPS URLs remain supported, require an explicit allowlist.
- Reject `http://` in release.
- Reject localhost, loopback, link-local, and private-network targets.
- Require `.gguf` file names for model downloads.
- Enforce a maximum model size before and during streaming.
- Write a manifest with source URL, expected size, hash if available, and install timestamp.
- Remove temporary files reliably on failure and cancellation.

Acceptance tests:

- Download from `http://127.0.0.1:...` is rejected.
- Download from `http://localhost:...` is rejected.
- Download from `http://10.x.x.x`, `172.16.x.x`, and `192.168.x.x` is rejected.
- Download exceeding the configured size limit is interrupted and temp files are removed.
- A valid Hugging Face `.gguf` URL still works.

### [ ] Reuse model safety checks for activation and runtime resolution

Current risk:

- `model_safety::tauri_models_delete` has strong model-dir and `.gguf` checks.
- Some activation/runtime paths can still resolve a direct absolute path if the file exists.

Required work:

- Create one shared `resolve_safe_model_path` function for delete, activate, deactivate, and chat runtime resolution.
- Require canonical model paths to be inside the ElephantNote model directory unless the user explicitly selected a safe imported model flow.
- Require `.gguf` extension before passing a file to llama.cpp.
- Add tests for absolute paths outside the model directory.

Acceptance tests:

- Activating `/tmp/fake.gguf` fails unless it was imported into the managed model directory.
- Chat runtime cannot start a model outside the managed model directory.
- Deleting a model outside the managed model directory remains rejected.

### [ ] Replace generic `shell_exec` with strict export commands

Current risk:

- `shell_exec` currently exposes `pandoc` execution with frontend-supplied args.
- The command is narrower than an unrestricted shell but still exposes a native process surface.

Required work:

- Remove generic `shell_exec(command, args, cwd, env)` from the release handler.
- Replace it with command-specific APIs, for example `tauri_export_note_to_pdf(relative_path, options)`.
- Canonicalize input and output paths inside vault/export directories.
- Allowlist supported Pandoc options.
- Reject filters, Lua scripts, custom readers/writers, arbitrary output paths, and network-related options unless explicitly reviewed.

Acceptance tests:

- Renderer cannot invoke arbitrary `pandoc` args.
- Export cannot write outside the active vault/export directory.
- Export succeeds for a normal Markdown note.

### [ ] Protect AI provider secrets

Current risk:

- `tauri_ai_config_set` writes the received config JSON directly.
- `tauri_ai_config_test` echoes the config back to the renderer.
- If provider API keys are added to this config, they may be stored in cleartext and reflected in UI/logs.

Required work:

- Define a strict AI config schema.
- Store secrets in platform secure storage instead of the JSON config file.
- Redact fields named `apiKey`, `token`, `secret`, `authorization`, and similar before returning configs or logging errors.
- Ensure exported diagnostic logs never include full secrets.

Acceptance tests:

- Saving an OpenAI/OpenRouter key does not write the raw key into `tauri-ai-config.json`.
- `tauri_ai_config_get` returns only redacted key metadata.
- `tauri_ai_config_test` never echoes raw secrets.

## Medium

### [ ] Strengthen guardrail script and remove accepted debt progressively

Current state:

- `pnpm security:guard` exists and detects broad Tauri scopes, dangerous CI permissions, committed `.env` files, private keys, and common API tokens.
- Existing broad Tauri FS findings are accepted in `security/guardrails-baseline.json`.

Required work:

- Add checks for `withGlobalTauri: true` in production config.
- Add checks for CSP `unsafe-inline` and `unsafe-eval` in production config.
- Add checks for broad localhost CSP entries in release config.
- Add checks for `Command::new` call sites that execute configurable paths.
- Remove baseline entries as each issue is fixed.

Acceptance tests:

- CI fails if broad FS scopes are reintroduced after removal.
- CI fails if production CSP regresses to `unsafe-eval`.
- CI fails if a new generic command execution path is added without a reviewed allowlist.

### [ ] Add renderer XSS regression tests for Markdown/HTML rendering

Current risk:

- Notes are untrusted user content.
- Any XSS in Markdown preview, graph preview, wiki preview, or attachment preview becomes more serious because this is a desktop app with native APIs.

Required work:

- Identify every HTML insertion path in the renderer.
- Enforce DOMPurify or equivalent sanitization for rendered untrusted HTML.
- Add malicious Markdown fixtures containing scripts, event handlers, SVG payloads, iframes, and malicious links.
- Confirm the stricter CSP blocks inline script execution.

Acceptance tests:

- Markdown containing `<script>` does not execute.
- Markdown containing `<img onerror=...>` does not execute.
- SVG payloads are sanitized or blocked.
- External links open only through the reviewed opener flow.

### [ ] Reduce sensitive path exposure in UI payloads

Current risk:

- Several APIs return `fullPath` values to the renderer.
- This is useful for debugging but increases privacy exposure and makes logs more sensitive.

Required work:

- Return relative paths by default.
- Return `fullPath` only in explicit debug mode or where strictly required.
- Scrub full paths from user-facing error messages where possible.

Acceptance tests:

- Normal note list/read/search payloads do not expose absolute paths unless debug mode is enabled.
- Debug logs redact home directory prefixes where possible.

## Low / cleanup

### [ ] Update package metadata inherited from MarkText

Current issue:

- Some metadata still points to MarkText authors/homepage/repository.
- This is not a direct vulnerability, but it is confusing for security reporting and provenance.

Required work:

- Update `package.json` and `src-tauri/Cargo.toml` metadata to ElephantNote/SorbetUP.
- Add a clear security contact or SECURITY.md before public release.

Acceptance tests:

- Package metadata points to the ElephantNote repository.
- SECURITY.md explains how to report vulnerabilities.

## Suggested implementation order

1. Land guardrail checks for CSP/global Tauri/regressions.
2. Remove direct renderer FS permissions and migrate remaining frontend FS calls to Rust commands.
3. Harden CSP and remove `withGlobalTauri`.
4. Lock down llama-server execution.
5. Lock down model downloads and model path resolution.
6. Replace `shell_exec` with explicit export commands.
7. Add renderer XSS tests and secret-storage tests.

## Definition of done for release

- No broad filesystem scope remains in production capabilities.
- No generic native command execution is exposed to the renderer.
- Model downloads are source-restricted, size-limited, and private-network-blocked.
- Local runtime execution uses only trusted app-managed binaries in release.
- AI secrets are never stored or echoed in plaintext.
- Production CSP is restrictive and does not contain `unsafe-inline` or `unsafe-eval`.
- Security guardrails fail on any regression in the above areas.
