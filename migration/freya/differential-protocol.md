# Freya / Tauri differential protocol

This is the contract for proving that the native Freya migration produces the
same user-visible result as the working Tauri application. A successful build
or an accessibility-only test is not a parity result.

## One scenario, two adapters

The scenario is data, not code. Both adapters consume the same fixture, viewport,
scale factor, logical actions, checkpoints and timeline. An adapter may use a
different locator mechanism, but it may not skip an action because its runtime
has no convenient equivalent.

Every action records:

- `action.start` and `action.done`/`action.error` with a correlation id;
- the logical action and its concrete target;
- the normalized visible state;
- the persisted vault state after the action;
- a screenshot before, during and after actions that can move or animate.

The fixture must be copied into separate temporary roots before each runtime
starts. It must contain the same bytes and the same metadata timestamps. A
runtime is not allowed to repair or seed the other runtime's output.

## Evidence classes

The report keeps these assertions separate:

1. **Semantic:** visible labels/roles, enabled state, selected route, editor
   content and error surface.
2. **Geometry:** viewport dimensions and the bounding boxes of stable targets.
3. **Raster:** PNG dimensions and pixel/perceptual difference at each
   checkpoint.
4. **Temporal:** a frame sequence around pointer movement, press/release,
   scroll, open/close and text input. A final screenshot cannot discharge this
   check.
5. **Persistence:** sorted vault file paths, bytes and relevant metadata after
   each mutating action and after a restart.

The comparator must fail on missing frames, missing states, different image
dimensions or a sequence-length mismatch. Thresholds are explicit in the
report; a threshold is not permission to hide a missing control or a changed
layout.

## Required timeline

The initial migration tranche uses this minimum timeline, in this order:

1. launch the clean fixture and capture the idle shell;
2. move the pointer across the primary navigation and capture hover frames;
3. open and close the create menu, capturing the press, open and close states;
4. create a note and verify both the visible card and the file bytes;
5. enter the `Projects` directory and open `Plan`;
6. type a deterministic suffix, undo it, type it again and save;
7. scroll the editor/list and capture the scroll frames;
8. open search, enter a query, clear it and return to notes;
9. open settings, switch sections, then close settings;
10. restart both runtimes and compare the restored note and layout.

If a runtime cannot execute an action, the run is `NOT PROVEN`; the adapter
must not replace it with a direct state mutation.

## Freya adapter

`freya-testing` is the native test surface. It must use `TestingRunner` with
`app_with_vault`, `click_cursor`/`press_cursor`/`release_cursor`,
`move_cursor`, `scroll`, `write_text`, `press_key` and `poll_n`. The renderer
output comes from `TestingRunner::render_to_file`; frame advancement comes from
the runner ticker/animation clock, not from a fake timer in the test.

The semantic snapshot consists of every visible accessibility label and its
layout rectangle, plus the current fixture bytes. It is recorded for every
checkpoint and every temporal frame.

## Tauri adapter

The repository's existing Playwright tests launch Electron and inject a Tauri
preload; that is useful for renderer regressions but is not a Tauri-binary
proof. The existing desktop acceptance runner launches the real Tauri binary
and drives its production event bridge, but it is an HTTP acceptance channel,
not Playwright browser control.

On macOS, the Tauri WebView is WKWebView. Playwright's `connectOverCDP` is a
Chromium protocol and cannot be used to relabel this WebView. Tauri's supported
real-app automation path is its WebDriver integration (WebdriverIO's embedded
service on macOS). Until that service is present, the real Tauri run must be
reported as a separate acceptance evidence class and any screenshot capture
must state exactly how the native window was captured.

The differential report therefore has a mandatory `tauri.controlPlane` field
with one of `webdriver`, `acceptance-http`, or `renderer-electron`. Only
`webdriver` is eligible for a claim that Playwright-style UI actions were
executed against the real Tauri WebView. `acceptance-http` proves the real
Tauri production path but not browser-level pointer automation; it cannot be
silently upgraded to `webdriver`.

## Completion rule

Parity is `PROVEN` only when semantic, geometry, raster, temporal and
persistence comparisons pass for the complete timeline on the final revision,
with real Tauri runtime logs and retained artifacts. Any missing native
control, unimplemented action, skipped frame, failed restart, or unverified
control plane leaves the result `NOT PROVEN` or `BLOCKED` with the exact
artifact path and command.
