# Differential run report — Freya / Tauri

Date: 2026-08-11
Branch: `nsb/freya-native-migration`
Revision under test: `84b3784272c083b72316f5591c524f032f8ff0cf` plus the uncommitted differential harness files listed below
Platform: macOS, arm64

## Status

`NOT PROVEN`

Freya produced a real temporal evidence sequence. The real Tauri process also
started and reached `renderer:ready`, but the Tauri acceptance journey failed
before a complete scenario and the repository has no Playwright adapter for the
macOS Tauri WKWebView. Therefore no Tauri/Freya visual equality claim is made.

## Freya evidence

Command:

```text
ELEPHANT_FREYA_EVIDENCE_DIR=/private/tmp/freya-evidence-final \
cargo test --manifest-path Elephant/freya/Cargo.toml \
  --test differential_freya_capture -- --nocapture
```

Result: `1 passed` in 5.58s.

The test used `TestingRunner`, `app_with_vault`, real filesystem writes,
`render_to_file`, accessibility/layout snapshots, pointer movement, click,
scroll, `animation_clock` and 50 ms `poll_n` steps. The retained run contains
64 PNGs: 1 static startup image and 63 temporal frames, plus JSON snapshots.

The same evidence directory compared against itself with the native PNG
comparator:

```text
node tools/freya-differential/compare.mjs \
  /private/tmp/freya-evidence-final /private/tmp/freya-evidence-final \
  --report test-results/freya-differential/latest/freya-self-report.json
```

Result: `ok`; 64/64 images compared, 0 missing, 0 extra, 0 changed pixels.
This proves the capture and comparator, not cross-runtime parity.

The retained Freya artifacts are under
`test-results/freya-differential/latest/freya/`.

## Tauri evidence

Command:

```text
ELEPHANT_ACCEPTANCE_UI_ONLY=1 \
ELEPHANT_ACCEPTANCE_SHOW_WINDOW=0 \
ELEPHANT_ACCEPTANCE_SKIP_BUILD=1 \
pnpm test:desktop:acceptance
```

The real Tauri binary started, printed
`ELEPHANT_ACCEPTANCE_TAURI_PORT=52135`, installed the acceptance bridge and
reached `[acceptance-tauri] renderer:ready`. It then exited with code 1 at the
existing Create-menu acceptance assertion:

```text
Error: Create menu is incomplete
excalidrawLogo.exists: true
excalidrawLogo.visible: false
```

Artifacts:

- `test-results/acceptance/latest-tauri.log`
- `test-results/acceptance/latest.json`

No Tauri screenshot or temporal frame was produced by this runner.

## Playwright evidence and control-plane limit

The repository's Playwright tests use `_electron.launch` and an Electron
preload shim. A focused run of the existing parity contract ended with 6/6
tests failing because Electron aborted with `SIGABRT` before a page was
created. That is a separate Electron failure, not Tauri proof.

The Tauri acceptance runner uses `fetch` against a local HTTP bridge and
synthetic DOM commands. It is not a Playwright `Page`, and it exposes no
`page.screenshot`, trace, video, pointer timeline or native window frame
capture. The local audit is in
`migration/freya/tauri-playwright-evidence.md`.

Tauri's current testing documentation recommends WebDriver/WebdriverIO for
real-app automation and specifically distinguishes the renderer-only browser
mode from the native app path:
[Tauri WebDriver testing](https://v2.tauri.app/develop/tests/webdriver/).
Adding that control plane is a separate implementation decision; it was not
silently substituted for the requested Playwright path.

## Required parity still unproven

The shared scenario records these as hard gaps in the current Freya shell:

- editable Muya target and real text input/save path;
- addressable scroll state and scroll-frame comparison;
- close-note control;
- drag-start/drag-over/drop behavior;
- complete Tauri visual capture and the same action timeline;
- restart comparison and persisted state equivalence across both runtimes.

The Freya manifest records these gaps in
`test-results/freya-differential/latest/freya/manifest.json`. They are not
replaced by direct state mutation or a placeholder control.

## Files added for this evidence tranche

- `migration/freya/differential-protocol.md`
- `migration/freya/differential-scenarios.json`
- `migration/freya/tauri-playwright-evidence.md`
- `Elephant/freya/tests/differential_freya_capture.rs`
- `tools/freya-differential/compare.mjs`
- `tools/freya-differential/README.md`

No existing Vue/Tauri implementation was removed or bypassed.
