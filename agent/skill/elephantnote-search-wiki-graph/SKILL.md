---
name: elephantnote-search-wiki-graph
description: >
  Rules for exact search, semantic search, smart search, knowledge chunks,
  concept routing, graph views, clustering, and auto-wiki behavior.
argument-hint: '<search-wiki-graph-task>'
---

# ElephantNote Search / Wiki / Graph

Use this skill for exact search, semantic search, smart search, embeddings, knowledge chunks, concept routing, graph views, clustering, organization, and wiki generation.

## Invariants

- Exact search must scan real Markdown files and return real snippets.
- Semantic search must use a real index/provider or explicitly report why it falls back.
- Smart search must make fallback behavior visible, not pretend semantic search ran.
- Knowledge chunks need stable ids, source note metadata, and deterministic behavior.
- Wiki pages and indexes are internal app artifacts and should live in hidden wiki storage.
- Graph edges must come from real note links/tags/semantic evidence, not decorative random links.

## Read first

- `Elephant/backend/js/search/**` and `Elephant/backend/js/wiki/**`.
- `Elephant/shared/knowledge/**`.
- Graph/wiki/search frontend components and stores.
- Tests for `searchLibrary`, `ElephantSearchService`, graph helpers, wiki helpers, and concept routing.

## Verification

- Build a tiny vault fixture with known notes.
- Query exact terms and verify snippets/source paths.
- Query semantic/smart paths and verify index/fallback status.
- Verify graph nodes/edges match the fixture.
- Verify wiki artifacts are hidden from normal note browsing.

## Anti-slop

No deterministic fake embedding presented as production semantic search unless it is explicitly labeled and tested as fallback behavior.
