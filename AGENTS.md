# Repository Guidelines

## Project Structure & Module Organization
- `engines/` hosts the C engines (cursor, editor, markdown, render, search) with module-specific `Makefile`s and resulting `lib*.a` outputs.
- `MarkdownEditorApp/` provides the macOS shell built by the root `Makefile`; UI helpers live in top-level `*.m`/`*.h` files.
- `tools/` contains developer tooling (`tui/`, `debug/`, `build/`) that link against the engines for interactive and scripted trials.
- `web/site/` and `web/wasm/` serve WebAssembly bindings and HTML demos, while `flutter/` covers the Dart front end; keep automation in `scripts/` and archived binaries in `versions/` or `old_builds/`.

## Build, Test, and Development Commands
- Run `make`, `make run`, and `make clean` from the repo root to build, launch, or reset the macOS bundle.
- `cd engines/editor && make static|wasm|test` compiles the core library, emits WASM bindings, or produces `./editor_test`.
- `cd tools/tui && make run|test|demo` drives the cursor engine interactively or through scripted assertions.
- Serve the web demo via `cd web/site && python3 -m http.server 8001`; execute `flutter run` inside `flutter/` for mobile/desktop.
- `scripts/validate-release.sh` and `scripts/run-full-validation.sh` orchestrate end-to-end checks, including fuzzing and property suites.

## Coding Style & Naming Conventions
- Use 4-space indentation and explicit braces in C/Objective-C, mirroring `engines/cursor/cursor_manager.c`.
- Prefer `snake_case` for C functions and globals, `ALL_CAPS` for macros, and `CamelCase` classes/selectors for Objective-C components such as `VaultManagerController`.
- Keep headers self-contained with include guards and favor `static` helpers for internal utilities; never commit generated `*.o`, `.wasm`, or bundle artifacts.

## Testing Guidelines
- Co-locate engine tests beside their modules (`engines/*/test_*.c`) and surface them through each module's `make test` target.
- Expand cursor scenarios in `tools/tui/` via `scriptable_tui` or `cursor_demo`; populate `tests/fixtures/` when new fixtures are required by validation scripts.
- Run `scripts/validate-release.sh` (or the fuller `run-full-validation.sh`) before PRs that touch parsing, rendering, or ABI surfaces, and capture sanitizer or fuzz logs for reviewers.

## Commit & Pull Request Guidelines
- Follow the imperative tone used in history (`git log -5 --oneline`): start with a capitalized verb, note the feature area, and skip terminal punctuation.
- Keep commits focused, document the build/test commands you exercised in the PR description, and link related issues or tickets.
- For UI-facing updates (web, Flutter, macOS), attach before/after screenshots or short recordings and flag any new assets or scripts reviewers must fetch.
