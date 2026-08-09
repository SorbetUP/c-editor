# Muya JavaScript to Rust migration

The migration is intentionally stacked on `refactor/muya-js-modular-parity`.

## Invariant

For every JavaScript capability migrated to Rust:

1. the decomposed JavaScript module remains the behavioral oracle;
2. the Rust module mirrors the same responsibility and observable semantics;
3. JavaScript UTF-16 offsets are preserved explicitly where applicable;
4. parity tests must pass before any runtime cutover;
5. runtime cutover happens at a stable editor boundary, never by adding one Tauri IPC round trip per keystroke or helper call;
6. the JavaScript implementation is removed only after the Rust path is active and the original-vs-candidate characterization suite remains green.

## Current stable Rust boundary

`Elephant/crates/muya-core` owns a revisioned `EditorSession` containing:

- the canonical `Document`;
- logical UTF-16 `Selection`;
- transaction-based `History` with grouped IME composition;
- semantic command dispatch;
- ordered logical `ViewPatch` output;
- explicit snapshots for initialization and recovery.

Editor protocol version `1` serializes typed requests and snapshot/update/error responses. Requests carry an expected document revision so stale or reordered browser commands are rejected before mutation. Snapshots include Markdown plus a preorder logical document tree with stable node IDs.

The protocol routes user intent to semantic engines:

- Backspace to grapheme-aware deletion;
- formatting to partial and cross-wrapper mark transforms;
- Enter to ordinary paragraph splitting with boundary-aware fallback;
- Tab navigation to logical table navigation;
- composition events to grouped history.

## Experimental WASM/WebView path

`Elephant/crates/muya-wasm` exposes the session through `wasm-bindgen`. The browser adapter under `Elephant/frontend/src/muya/lib/rust` provides:

- serialized revisioned requests;
- snapshot recovery;
- transactional logical patch validation;
- incremental DOM patching with stable node IDs;
- browser/logical selection conversion;
- `beforeinput`, table navigation and IME routing;
- lifecycle-safe isolated runtime setup and teardown.

The normal build resolves a disabled WASM stub. The experimental bundle is generated and built with:

```bash
cargo install wasm-bindgen-cli --version <resolved-version> --locked
pnpm tauri:web:build:rust-editor
```

`pnpm muya:wasm:build` resolves the exact `wasm-bindgen` dependency version through Cargo metadata and refuses a mismatched CLI. Generated JavaScript/WASM artifacts are ignored by Git and consumed through the Vite alias `muya-rust-wasm-bundle`.

At runtime, the feature remains opt-in:

```js
experimentalRustEditor: {
  useBundledWasm: true,
  domContainer: isolatedElement,
  captureInput: true
}
```

An injected `factory(markdown)` may still be used by tests or specialized hosts. No configuration implicitly takes ownership of the existing Muya DOM.

The visible production editor still runs Muya JavaScript by default. This is an experimental execution path, not a production cutover.

## First migrated foundation

| JavaScript oracle | Rust module | State |
| --- | --- | --- |
| `Elephant/frontend/src/muya/lib/utils/primitives.js` | `Elephant/backend/tauri/src/muya_engine/utils/primitives.rs` | Ported and unit-tested; awaiting capability cutover |
| `Elephant/frontend/src/muya/lib/parser/marked/stringTools.js` | `Elephant/backend/tauri/src/muya_engine/parser/marked/string_tools.rs` | Ported with UTF-16 offset parity tests; awaiting capability cutover |
| `Elephant/frontend/src/muya/lib/parser/marked/tableTools.js` | `Elephant/backend/tauri/src/muya_engine/parser/marked/table_tools.rs` | Ported and unit-tested; awaiting capability cutover |
| `Elephant/frontend/src/muya/lib/parser/marked/uniqueId.js` | `Elephant/backend/tauri/src/muya_engine/parser/marked/unique_id.rs` | Ported with thread-safe monotonic IDs; awaiting capability cutover |

## Cutover order

1. Pure marked/parser helpers.
2. Block and inline tokenization.
3. Markdown import/export and block tree.
4. Stateless edit transformations.
5. Cursor, selection, history and composition state machine.
6. Revisioned editor session and serializable protocol.
7. WASM/WebView adapter and ordered DOM patch application.
8. Packaged experimental WASM build and isolated runtime validation.
9. Differential characterization against original Muya on real editing traces.
10. Capability-by-capability frontend activation.
11. JavaScript deletion only after the activated Rust path remains green.

The old JavaScript file is not deleted merely because a Rust translation exists. Deletion is the final step of each coherent slice, after differential validation and runtime activation.
