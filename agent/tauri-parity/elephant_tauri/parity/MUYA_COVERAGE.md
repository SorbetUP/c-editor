# Muya parity coverage

This file is the migration contract for replacing Muya behavior with Rust/Tauri behavior without changing the Vue UI.

Reference source: the Electron/MarkText Muya implementation is the source of truth. The Rust engine/runtime is not considered equivalent until every observable item below has fixtures and tests.

## Status scale

- 0%: not started.
- 25%: basic shape exists, but incomplete.
- 50%: common cases pass unit fixtures.
- 75%: edge cases and UI contract mostly covered.
- 100%: covered by parity fixtures and verified against the Electron/Muya behavior.

## Source comparison layer

The deterministic engine now has five fixture/snapshot layers:

- `muya_deterministic_cases.json`: Markdown inputs and expected deterministic fields.
- `muya_edge_cases.json`: CommonMark/Muya edge-case inputs for autolinks, escapes, heading attrs and fence variants.
- `muya_source_snapshots.json`: source-of-truth snapshots consumed by Rust snapshot tests.
- `muya_source_snapshots_meta.json`: generation metadata, including whether the snapshots came from the real renderer or the fallback contract adapter.
- Rust snapshot tests: compare the Rust deterministic contracts against these snapshots.

`pnpm muya:snapshots` regenerates contract snapshots for development.

The strict 100% path is:

```bash
pnpm add -D @muyajs/core
node scripts/generate-muya-source-snapshots.mjs --strict-real --adapter=scripts/adapters/real-muya-renderer.mjs
bash scripts/tauri-rust-check.sh
```

`--strict-real` must fail if the adapter or `@muyajs/core` is unavailable. The engine is only allowed to claim 100% when `muya_source_snapshots_meta.json` reports `mode: real-electron-muya-renderer` and all Rust tests pass.

## Markdown engine parity

| Area | Current estimate | Notes |
|---|---:|---|
| CommonMark block parsing | 78% | Headings, paragraphs, blockquotes, lists, HR, code blocks and fence variants exist. Nested list metadata, deterministic fixtures, edge fixtures and source-snapshot comparisons exist. |
| GFM tables | 78% | Render/token support exists, row/column contracts exist, deterministic alignment/export fixtures exist, and source-snapshot comparison exists. |
| Task lists | 78% | HTML classes are normalized to `task-list` and `task-list-item`; tokens expose `task_marker`; nested checked item metadata exists. |
| Inline marks | 80% | Strong/emphasis/strike/code/link tokens exist, nested inline mark deterministic fixtures exist, and source-snapshot comparison exists. |
| Links and images | 78% | Direct links, reference-style links/images and autolinks are covered by extras/edge fixtures. Advanced title/escaping cases still need expansion. |
| Footnotes | 78% | Definitions, references, footnote HTML contract, fixtures and source-snapshot comparison now exist. |
| HTML blocks/inline HTML | 70% | Token coverage and sanitization contract exist. Dangerous payload tests are kept in Rust tests, not JSON fixtures. |
| Math blocks/inline math | 75% | Inline and block math emit extras plus KaTeX-like HTML contract. A strict real-Muya adapter scaffold now exists but must be run with `@muyajs/core`. |
| Diagrams | 75% | Mermaid/flowchart/sequence/vega/plantuml fences emit extras plus diagram HTML contract. A strict real-Muya adapter scaffold now exists but real preview parity still needs generated snapshots. |
| Frontmatter | 78% | YAML-like scalars, booleans, numbers, null, inline arrays, block arrays, simple objects and source-snapshot comparison are covered. Full YAML spec is not implemented. |
| Escapes and attributes | 75% | Escaped Markdown punctuation, autolink normalization, heading attributes and fence metadata now have edge contracts and tests. |
| Export HTML contract | 75% | Rendering exists; task classes, math/diagram placeholders, reference links, footnotes, sanitized HTML, table alignment, edge cases, source snapshots and a real adapter scaffold are covered. Exact MarkText export normalization requires strict generated snapshots. |

## Editor interaction parity

| Area | Current estimate | Notes |
|---|---:|---|
| DOM editor real | 45% | A contenteditable DOM editor runtime exists in `selectionRuntime.js`, with selection snapshot/restore. It is not yet wired as the production editor. |
| Browser selection | 45% | Node-path based selection serialization/restore is implemented and unit-tested. Complex browser selection edge cases still need real browser tests. |
| JSONState compatible Muya | 45% | JSONState-like parse/render/HTML runtime exists for headings, paragraphs, blockquotes, lists, tasks, code, math and tables. It is not full Muya JSONState yet. |
| OT operations | 40% | Insert/delete/replace, transform and transaction composition exist in `operationsRuntime.js`. Collaborative OT parity is not complete. |
| Grouped history | 45% | Grouped undo/redo exists for runtime transactions. Full Muya grouped history semantics still need source comparison. |
| Clipboard HTML complex | 45% | Clipboard runtime sanitizes and converts pasted HTML from rich sources into Markdown. More Word/Notion/web cases need fixtures. |
| Paste from Word/Notion/web | 45% | Word/Notion style attributes and rich HTML are normalized in `clipboardRuntime.js`; not exhaustive yet. |
| Table editing UI complete | 45% | Row/column insert/delete and alignment commands exist in `tableImageRuntime.js`; UI integration and all table gestures are not complete. |
| Image resize/edit toolbar | 45% | Image-under-cursor, toolbar state and Markdown width update exist. Real resize handles and asset pipeline integration are not complete. |
| Footnote popup tool | 45% | Footnote popup state and upsert behavior exist. Real floating popup UI is not wired yet. |
| Slash command menu | 45% | Slash command filtering and snippets exist. Real menu UI/event integration is not wired yet. |
| Floating toolbars | 45% | Floating toolbar state/actions exist. DOM placement and collision behavior need browser tests. |
| Preview blocks | 45% | KaTeX-like and diagram preview descriptors exist. Real KaTeX/Mermaid/Vega/PlantUML renderers are not wired in runtime yet. |
| Round-trip Markdown HTML exact | 35% | JSONState Markdown/HTML round-trip primitives exist; exact Muya round-trip is not complete. |
| CommonMark/GFM edge cases | 75% | Rust deterministic edge contracts exist; renderer runtime still needs more JS fixtures. |

## App migration parity

| Area | Current estimate | Notes |
|---|---:|---|
| Vue UI reuse | 80% | Tauri uses the same AppShell path again. Pixel diff tests are still missing. |
| Runtime bridge | 45% | Muya editor contract commands and renderer runtime contracts are now exposed/available; many Electron APIs are still stubs. |
| Notes read/write | 40% | Basic Rust commands exist. Full tab/editor integration still incomplete. |
| Vault layout | 65% | Hidden/visible layout exists. More migration tests needed. |
| Search | 15% | Basic query/rebuild exists. Not equivalent to Electron yet. |
| Attachments/images | 10% | Basic hidden assets storage exists. UI parity incomplete. |
| Drawings/Excalidraw | 5% | Placeholder scene commands exist. |
| Sync | 5% | Status/plan placeholders only. |

## Honest global estimate

- Muya deterministic Markdown engine: about 78-85%.
- Muya full editor behavior: about 40-50%.
- Full Tauri replacement of Electron app: about 25-32%.

The engine/runtime is not allowed to claim 100% until strict real snapshots are generated through `scripts/adapters/real-muya-renderer.mjs`, `muya_source_snapshots_meta.json` reports `real-electron-muya-renderer`, the renderer runtime is wired into the production editor, and every Rust + JS parity test stays green.
