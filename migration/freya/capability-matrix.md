# Freya capability matrix

This matrix separates upstream capability from capability proven in Elephant. Upstream references: [Freya repository](https://github.com/marc2332/freya), [Freya platform documentation](https://docs.freyaui.dev/freya/_docs/platforms/index.html), [Freya components](https://docs.rs/freya/latest/freya/_docs/ui_and_components/index.html), and [freya-testing documentation](https://docs.freyaui.dev/freya_testing/).

| Capability | Upstream indication | Elephant proof | Status |
|---|---|---|---|
| Native desktop window | Freya desktop runtime | crate compiles; headless shell tests pass | PARTIALLY PROVEN |
| Layout, text and controls | Freya components/layout primitives | shell renders and navigation state changes under `freya-testing` | PARTIALLY PROVEN |
| Headless render artifact | `freya-testing` runner/render APIs | PNG artifact is created by native shell test | PROVEN_FOR_SHELL |
| Rich text/editor | Freya ecosystem has editor-related components/examples | Elephant Muya editor is not wired | NOT PROVEN |
| Markdown viewer | upstream examples/docs | no Elephant route or persistence path | NOT PROVEN |
| Canvas/graphics | Freya/Skia APIs | Excalidraw is still React/WebView; no native drawing port | NOT PROVEN |
| Router/state | Freya APIs/examples | local navigation state only; no persisted route contract | PARTIALLY PROVEN |
| Android | documented as experimental | no device build or IME test | NOT PROVEN |
| Web islands | webview/embedded capabilities exist upstream | no isolated Excalidraw host contract yet | NOT PROVEN |
| Packaging | framework support is not product acceptance | no Freya distributable acceptance run | NOT PROVEN |

The complete platform status is in [`freya-platform-matrix.json`](freya-platform-matrix.json). A green upstream row never substitutes for a real Elephant runtime scenario.
