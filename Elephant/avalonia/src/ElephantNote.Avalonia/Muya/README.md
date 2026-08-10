# Avalonia Muya boundary

`IMuyaHost` is the public Muya contract. `NativeWebViewMuyaHost` is its
NativeWebView-backed adapter. `AvaloniaMuyaWebView` adapts the Avalonia control
to the shell-neutral `IMuyaWebViewAdapter`; it does not own a document.

`MuyaEditorHost` remains only as a compatibility façade for the first Avalonia
scaffold. New shell code should depend on `IMuyaHost`, not that concrete name.

## State ownership

There is one authority for each kind of state:

| State | Authority | Boundary behavior |
| --- | --- | --- |
| DOM, rendered blocks, caret, selection, keyboard and IME composition | Muya JS | Muya emits Markdown snapshots; the host never reconstructs the DOM. |
| Markdown parsing, structural transformation, serialization and validation | Rust/WASM | Rust returns validation metadata (`ValidationEngine` and `ValidationRevision`). It is not a second editor or persistence state. |
| Vault files, filesystem access, lifecycle and persistence | Avalonia | The shell consumes `ContentChanged`/`SaveRequested` and writes the file. |
| Last Markdown snapshot known to the shell | `IMuyaHost.Content` | A transport/persistence snapshot only; it is not a competing live editor model. |

The browser wire field `rustRevision` is retained for compatibility with the
current Muya bundle. In C# it is exposed semantically as
`ValidationRevision`; the legacy `RustRevision` property is only a source-
compatibility alias. No Rust snapshot is stored or reconciled by the host.

## Public contract

`IMuyaHost` exposes:

- `OpenDocumentAsync(MuyaOpenDocumentRequest, CancellationToken)`;
- `SetMarkdownAsync(string, CancellationToken)`;
- `Content`, `Document`, and lifecycle status properties;
- `ContentChanged`, `SaveRequested`, and `Error`;
- `FocusAsync(CancellationToken)`, which returns `false` when no ready view is
  attached.

Undo/redo are deliberately not in the contract yet. The current native page
does not expose explicit undo/redo commands through its host bridge, so adding
those methods would claim behavior that this boundary cannot prove.

`SetContentAsync` remains on the compatibility façade and delegates to
`SetMarkdownAsync`.

## Message protocol

The page sends JSON through `invokeCSharpAction`:

```js
invokeCSharpAction(JSON.stringify({ type: "ready" }))
invokeCSharpAction(JSON.stringify({
  type: "content-changed", documentId, content,
  engine: "muya-js+muya-rust", rustRevision
}))
invokeCSharpAction(JSON.stringify({
  type: "save-request", requestId, documentId, content,
  engine: "muya-js+muya-rust", rustRevision
}))
invokeCSharpAction(JSON.stringify({
  type: "error", code, message, documentId, requestId
}))
```

After navigation, the host sends `open-document` or `set-content` by invoking:

```js
window.__ELEPHANT_MUYA_HOST__?.receive(message)
```

The standalone source page lives under `Elephant/frontend/src/muya/native`.
`pnpm muya:avalonia:build` bundles that page and stages it under the
application's ignored `Assets/Muya/` output directory. This adapter remains
valid without the generated page; the shell can select its explicit fallback.
