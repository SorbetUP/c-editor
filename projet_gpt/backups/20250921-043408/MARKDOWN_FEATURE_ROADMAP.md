# Markdown Engine Expansion Roadmap

## Objective
Add comprehensive Markdown feature support aligned with https://www.markdownguide.org/cheat-sheet/, extend repository metadata handling, and integrate UI affordances (settings-specific Markdown, cross-note links, restored note-compose icon).

## Workstreams

1. **Parser & AST Enhancements**
   - Inventory unsupported structures (tables variations, blockquotes, nested lists, task lists, footnotes, definition lists, fenced code, inline HTML scopes, link reference definitions).
   - Extend C parser (`engines/markdown/markdown.c/h`) with modular handlers per structure; ensure idempotent round-trip via `json.c`.
   - Preserve backwards compatibility by gating experimental parsing behind capability flags when necessary.

2. **Serialization & JSON Schema Updates**
   - Update `engines/markdown/json.c` to emit rich span metadata (bold, italic, underline, strikethrough, code, links, images, settings tokens).
   - Define JSON schema extension for settings blocks (`type: "settings"`) and cross-note links (`xref` property targeting note IDs).
   - Document schema deltas in `docs/markdown_schema.md`.

3. **Rendering & Editor Integration**
   - Ensure `engines/editor/editor.c` and render engine honor new element kinds (settings blocks, dividers, task list checkboxes, etc.).
   - Introduce settings-specific styling hook (e.g., tinted container) routed via render engine.
   - Implement link navigation hook: clicking an `xref` span triggers main controller note switch.

4. **Cross-Note Link Mechanism**
   - Decide link syntax (e.g., `[[note-id|Title]]` or `[Title](note://id)`), adjust parser + serializer.
   - Update `ENMainController` to resolve note identifiers, open target tab, handle missing targets gracefully.

5. **UI Updates**
   - Restore note-compose icon in sidebar (re-introduce Files/Editor icon as specified) with click handler wiring to open blank note creation flow.
   - Add settings Markdown preview within Settings tab to validate new formatting.

6. **Testing & Validation**
   - Expand unit tests in `engines/markdown/tests` to cover each new structure.
   - Build end-to-end sample documents under `examples/markdown/advanced.md` demonstrating new syntax.
   - Manual regression: run `versions/v4/build-v4.sh`, launch app, verify rendering, navigation, and new icon behavior.

## Deliverables
- Updated Markdown engine, JSON schema, and UI integration.
- Documentation snippet for contributors on new syntax (`AGENTS.md` addendum).
- Smoke-tested ElephantNotes V4 app with new icon and Markdown features.
