# ElephantNote Avalonia migration

This directory is the native Avalonia client branch for ElephantNote. It is a separate .NET 8 desktop executable, not a replacement of the existing Tauri renderer. The only web surface is an isolated local Muya bundle hosted by Avalonia's `NativeWebView`; the shell, persistence and navigation remain native C# and XAML. The future Web/WASM architecture is described in [docs/architecture.md](docs/architecture.md), but no Avalonia Browser client is delivered here.

## Current native slice

The first slice keeps the user-visible shape of the web shell while connecting real local behavior:

- vertical navigation rail and vault sidebar;
- local vault discovery using the existing `tauri-vaults.json` contract when it is available;
- Markdown folders and notes, including safe root containment checks;
- note creation, folder creation, open, edit and save;
- text search over Markdown files;
- persisted light/dark and autosave preferences in the existing `preferences.json` contract;
- visible errors and correlated start/done/error entries in `avalonia.log`;
- Muya Markdown editing through an explicit C# protocol boundary; the bundled
  page keeps Muya as the visual oracle and validates each document mutation
  with the real Rust/WASM `muya-core` mirror, with a native TextBox fallback
  when the bundle or platform WebView engine is unavailable.

The existing Vue/Tauri application remains the product baseline. This native slice does not silently pretend to support features whose production adapters are not ported yet. Addons, Excalidraw, graph, calendar, chat, AI runtimes, sync, mobile behavior, Muya plugin parity, keyboard parity and the complete settings surface are explicitly pending native adapters.

## Build and run

Requires the .NET 8 SDK or newer. Avalonia packages are pinned to 12.1.1.

```bash
pnpm muya:avalonia:build
dotnet restore Elephant/avalonia/ElephantNote.Avalonia.sln
pnpm avalonia:build
dotnet test Elephant/avalonia/ElephantNote.Avalonia.sln -c Release --no-restore
pnpm avalonia:run
```

The config root can be overridden for deterministic tests and acceptance runs with `ELEPHANTNOTE_CONFIG_DIR`. The app preserves unknown keys while updating `preferences.json` and uses the existing vault registry filename `tauri-vaults.json` in that root.

On macOS, use the detached launcher when the app must not take over the working terminal or appear on screen. It wraps the self-contained Native AOT output in a real `.app`, starts it with LaunchServices using `open -g -n`, and writes runtime evidence to `avalonia.log`:

```bash
dotnet publish Elephant/avalonia/src/ElephantNote.Avalonia/ElephantNote.Avalonia.csproj \
  -c Release -r osx-arm64 --self-contained true \
  -p:PublishAot=true -p:PublishSingleFile=true \
  -p:PublishDir=Elephant/avalonia/build/artifacts/publish/macos-arm64-aot/
./Elephant/avalonia/build/run-background-macos.sh
```

The launcher returns immediately; the detached process has no terminal TTY and the normal background mode uses a transparent, non-activated window. Muya still initializes through the native WebView boundary, while the rest of the shell remains Avalonia XAML.

## Migration contract

| Web/Tauri surface | Avalonia status |
| --- | --- |
| App shell, rail, sidebar and library | Native first slice |
| Vault and Markdown persistence | Native first slice |
| Search | Native first slice |
| Settings and theme | Native first slice, reduced scope |
| Muya Markdown editor core | Isolated local bundle through `NativeWebView` plus Rust/WASM logical mirror; plugin/UI parity pending |
| Excalidraw and assets | Pending native drawing adapter |
| Addon lifecycle and UI contributions | Pending explicit host API adapter |
| Graph, wiki, calendar, chat and AI | Pending domain/runtime adapters |
| Android/iOS and packaged desktop proof | Not proven on this machine |

The native Muya usage proof can run independently of the .NET SDK:

```bash
pnpm test:avalonia:muya:e2e
```

It opens the generated page, edits a real contenteditable surface, checks a
Rust snapshot, exercises the save shortcut protocol and retains a screenshot.
The complete Avalonia-window proof still requires the .NET 8 SDK and the
platform WebView runtime.

The browser test is evidence for the Muya bundle and its message contract only;
it is not evidence that Avalonia `NativeWebView` or a future Avalonia Browser
target works. See [docs/architecture.md](docs/architecture.md) for the
explicit proof boundary.
