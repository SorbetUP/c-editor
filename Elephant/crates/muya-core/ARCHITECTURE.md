# Muya core architecture

`muya-core` is the DOM-independent source of truth for the future editor. It must not depend on Tauri, WebView APIs, Vue, Snabbdom, browser selections or filesystem services.

## Ownership layers

1. `model`: stable document nodes, IDs and UTF-16 observable source ranges.
2. `syntax`: ordered catalogue of Markdown block and inline constructions.
3. `parser`: Markdown to `Document`.
4. `serializer`: `Document` to Markdown, HTML or plain text.
5. `edit`: semantic commands, atomic operations and transactions.
6. `features`: isolated structural editing for complex constructs such as lists and tables.
7. `selection`: logical selection independent from DOM ranges.
8. `history`: invertible transactions, undo/redo and composition groups.
9. `view`: logical patches consumed by the browser adapter.
10. `session`: revisioned document, selection, history and command dispatch.
11. `protocol`: versioned serializable requests, responses, patches and errors.

## Current executable parser slice

The Rust parser and Markdown serializer currently execute these block constructions:

- ATX headings;
- Setext headings, canonicalized to ATX on serialization;
- paragraphs;
- thematic breaks;
- consecutive blockquote lines;
- fenced code blocks with language information;
- unordered lists;
- ordered lists with their starting number;
- task lists;
- structurally nested lists with mixed list kinds and preserved indentation;
- GFM-style tables with column alignment.

The inline tokenizer currently executes:

- escaped punctuation;
- code spans;
- images;
- inline links with optional titles;
- strong emphasis;
- emphasis;
- strikethrough;
- hard and soft line breaks;
- text fallback.

Inline parsing is recursive inside headings, paragraphs, blockquotes, list items, links and table cells. Fenced code content follows a separate literal path and is never tokenized as Markdown inline content. Empty editable containers receive an explicit `Text("")` node so selections always have an addressable target.

## Current executable editing slice

The model-level editing layer currently contains:

- DOM-independent logical selections using UTF-16 offsets;
- same-node forward and backward selection ordering;
- replacement and deletion across direct sibling text endpoints in one inline container;
- safe removal of fully covered inline subtrees between those endpoints;
- `InsertText`, `DeleteBackward` and `InsertParagraph` commands;
- `GraphemeCommand::DeleteBackward`, which removes one extended Unicode grapheme and delegates structural Backspace at offset zero;
- `MarkCommand::ToggleStrong`, `ToggleEmphasis` and `ToggleStrike`;
- complete and partial mark unwrapping for prefix, suffix and middle selections;
- mark application across fully selected top-level inline subtrees whose endpoints live in different nested wrappers;
- stable movement and exact undo restoration of nested marks, links and text nodes;
- plain and rich paragraph splitting;
- paragraph splitting from inside nested inline wrappers by cloning only the right-side wrapper chain;
- `ParagraphBoundaryCommand::InsertParagraph` at the start or end of a marked text node without creating empty wrappers;
- boundary-aware splitting for both document paragraphs and list items;
- movement of following inline subtrees at every nesting level without changing existing IDs;
- plain and rich paragraph joining;
- list-item splitting for unordered, ordered and task lists;
- new task items created unchecked;
- Enter on an empty item lifting its paragraph out of the list;
- middle empty items splitting one list into left and right lists;
- ordered right-side lists preserving the correct continuation number;
- Backspace-based merging with the previous list item;
- removal of the first list marker by lifting the same paragraph out of the list;
- automatic removal of a list container that becomes empty;
- structural nested-list parsing and serialization;
- `ListCommand::IndentItem` and `ListCommand::OutdentItem` with exact undo restoration;
- reuse of compatible existing nested lists and removal of nested containers that become empty;
- `TableCommand` operations for inserting and deleting body rows;
- `TableCommand` operations for inserting and deleting columns across all rows;
- table cell alignment and header flags preserved during structural edits;
- `TableNavigationCommand::NextCell` and `PreviousCell` for logical Tab navigation;
- automatic body-row insertion when navigating forward from the last cell;
- invertible subtree movement between containers;
- `SetParagraph` and `SetHeading(1..=6)` block transformations;
- UTF-16 boundary validation that rejects offsets inside surrogate pairs;
- invertible text replacement, leaf insertion and block-kind operations;
- detachable subtree snapshots with stable IDs and parent/child topology;
- invertible subtree removal, restoration and movement;
- transactions applied to a cloned document before commit, preventing partially applied failures;
- revision increments on successful document mutations;
- bounded transaction-based undo and redo history;
- IME/composition groups that commit as one undo entry or cancel atomically;
- ordered logical view patches, including subtree restoration and movement patches.

Structural undo and redo restore the same node IDs, topology, inline order, list nesting and table structure. Cross-node text replacement still requires direct sibling endpoint texts. Cross-wrapper mark application currently requires both boundary top-level subtrees to be fully selected; arbitrary partial boundary text across unrelated wrapper chains remains a later slice.

The parser, serializer and editing slices have Rust unit tests. Full original-Muya-vs-Rust differential characterization is still required before any capability can replace its JavaScript oracle.

## Revisioned editor session

`EditorSession` owns one `Document`, logical `Selection` and `History` instance. Its public dispatcher:

- rejects commands whose `expected_revision` does not match the document revision;
- validates browser-provided UTF-16 selection boundaries;
- routes core, mark, grapheme, list, table and paragraph-boundary commands;
- keeps selection-only navigation out of document history and does not increment revision for it;
- returns forward patches for edits and inverse patches for undo, redo and cancelled composition;
- exposes explicit snapshots for initialization and desynchronization recovery;
- creates an addressable empty paragraph when imported Markdown has no editable text node.

The session is the runtime boundary used by `muya-wasm`. Per-keystroke Tauri IPC remains forbidden: commands execute synchronously in-process with the WebView.

## Versioned adapter protocol

Protocol version `1` provides serializable:

- `EditorRequest` with protocol version, expected revision and typed command;
- `EditorResponse` variants for snapshot, update and error;
- snapshots containing Markdown plus a preorder logical document tree with stable node IDs;
- stable command tags such as `insert_text`, `delete_backward`, `toggle_strong`, `next_table_cell` and composition lifecycle events;
- stable `ViewPatch` tags such as `replace_text`, `insert_node`, `move_node` and `remove_node`;
- structured error codes including `revision_mismatch` and `invalid_utf16_boundary`.

The protocol routes high-level behavior to the modern engines: Backspace uses grapheme deletion, formatting uses `MarkCommand`, and Enter falls back to `ParagraphBoundaryCommand` when a normal split reaches a mark boundary.

## Experimental WASM and browser adapter

`Elephant/crates/muya-wasm` exposes `MuyaEditor`, `handle_json`, `snapshot_json` and the current revision through `wasm-bindgen`. CI verifies host tests, formatting and compilation for `wasm32-unknown-unknown`.

The opt-in browser layer under `frontend/src/muya/lib/rust` currently provides:

- a serialized command queue with revision tracking and structured protocol errors;
- full-snapshot recovery after patch desynchronization;
- a transactional logical JavaScript mirror used to validate every patch batch;
- an incremental DOM renderer with stable `data-muya-rust-id` elements;
- text, node, subtree, movement, removal and block-kind patch application;
- DOM-to-logical and logical-to-DOM UTF-16 selection conversion;
- `beforeinput`, paragraph, Backspace, formatting, undo/redo and table-Tab command mapping;
- IME composition replacement using a tracked logical range rather than repeated blind insertion;
- an isolated runtime that can render into an explicitly supplied container and detach all input listeners on destruction.

This runtime is disabled by default through `experimentalRustEditor: null`. It never claims the existing Muya DOM implicitly, and JavaScript Muya remains the visible production editor. The injected factory and explicit `domContainer` are currently required for opt-in execution.

## Markdown construction catalogue

Block constructions are registered in precedence order:

- frontmatter;
- fenced and indented code;
- ATX and Setext headings;
- thematic break;
- blockquote;
- task, ordered and unordered lists;
- table;
- HTML block;
- math block;
- diagram;
- footnote definition;
- reference definition;
- paragraph fallback.

Inline constructions are also registered in precedence order:

- escape;
- code span;
- image and link forms;
- autolink;
- strong, emphasis and strikethrough;
- inline math and HTML;
- emoji;
- superscript and subscript;
- footnote reference;
- hard and soft line breaks;
- text fallback.

A feature's syntax recognizer must not own editing, rendering or DOM behavior. Complex features use isolated modules such as `features/list`, `features/table` and `features/table_navigation` for semantic commands.

## Adapter boundary

The browser adapter owns:

- keyboard, `beforeinput` and composition events;
- mapping physical Tab/Shift+Tab and Backspace events to protocol commands;
- DOM creation and ordered logical patch application;
- browser `Selection` and `Range` conversion;
- geometry, scrolling and floating widgets;
- asynchronous image, KaTeX and Mermaid integration.

The first four responsibilities now have an experimental implementation. Geometry, scrolling, widgets and asynchronous rich renderers still remain on the JavaScript side. Tauri remains responsible for native files, operating-system integration and services. Neither adapter may become the canonical document model.

## Migration invariant

A JavaScript capability is removed only when its complete Rust slice is active:

1. syntax recognition;
2. model representation;
3. serialization;
4. semantic editing commands;
5. selection transformation;
6. undo/redo behavior;
7. view patches;
8. differential characterization against original Muya;
9. activation through the browser adapter.

File-by-file translations remain useful for inventory and parity, but runtime cutover is performed capability by capability.
