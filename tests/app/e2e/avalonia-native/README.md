# Avalonia native acceptance

This harness is the native acceptance boundary for the first Avalonia
vertical. It is intentionally separate from
`tests/app/e2e/avalonia-muya-usage.spec.js`, which tests the Muya bundle in a
browser and is not an Avalonia end-to-end test.

The harness uses macOS Accessibility/System Events to drive the real Avalonia
window and `screencapture` to retain screenshots of that native window. It
does not use Playwright, Chromium, JavaScript injection, `innerHTML`, or DOM
mutation.

## Scenario

1. Create a temporary vault and `tauri-vaults.json` fixture.
2. Launch the published Native AOT executable when available, otherwise run
   the already-built Avalonia project with `dotnet --no-build`.
3. Find the real `Elephant` window through macOS Accessibility.
4. Capture the initial native window screenshot and accessibility snapshot.
5. Activate the Markdown note through its native accessibility element.
6. Verify that the `Muya Markdown editor` NativeWebView is visible.
7. Focus that native control and send a real keyboard edit, then `Meta+S`.
8. Verify the Markdown file on disk.
9. Close the note with `Back to notes`, reopen it, and verify the file again.
10. Capture the reopened native window screenshot.

## Run

Build the application first, then run:

```bash
node tests/app/e2e/avalonia-native/acceptance.mjs --keep-artifacts
```

To force a specific executable:

```bash
AVALONIA_ACCEPTANCE_EXECUTABLE=/absolute/path/ElephantNote.Avalonia \
  node tests/app/e2e/avalonia-native/acceptance.mjs --keep-artifacts
```

The default result is written under a temporary directory and includes:

- `result.json` with status, events, paths and evidence;
- `avalonia.stdout.log` and `avalonia.stderr.log`;
- `native-window-initial.png`, `native-window-editor.png` and
  `native-window-reopened.png` when screenshots are permitted;
- accessibility snapshots for the initial, editor and reopened states;
- the temporary vault and the persisted Markdown note.

Exit codes are deliberately explicit:

- `0`: the complete native scenario passed;
- `1`: Avalonia launched but a native user-path assertion failed;
- `77`: the scenario was not run because the required GUI, Accessibility
  permission, Avalonia runtime, display/render timer or platform was
  unavailable.

The `77` result is not a pass. In particular, a successful standalone Muya
browser test cannot change a `77` result into proof of Avalonia
`NativeWebView` integration.

## Limitations

The current driver is macOS-specific because it relies on System Events and
`screencapture`. It requires Accessibility permission for the terminal or
Node process and Screen Recording permission for screenshots. If Avalonia
fails before creating a window (for example, the native render timer is not
available in a headless session), the harness reports `GUI_UNAVAILABLE` and
retains the process logs instead of falling back to Chromium.

The content assertion after reopening is intentionally made against the real
Markdown file. The harness does not inspect the WebView DOM. The accessibility
snapshot and screenshot prove that the native editor surface is present, but
platform WebView text exposure can vary independently of filesystem
persistence.
