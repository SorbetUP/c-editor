# ElephantNote Knowledge Core

`elephantnote-knowledge-core` is the native Rust knowledge engine for ElephantNote.

## Invariants

- User Markdown files are the canonical source of truth.
- All indexes are derived and rebuildable.
- Hidden folders are never indexed as user notes.
- Generated knowledge data lives under `.elephantnote/knowledge/`.
- No embedding model is required.
- Generated relations and citations must keep source evidence.
- Chat writes require explicit approval and an expected content hash.
- The core crate does not depend on Tauri, Vue, or the renderer.

## Current implementation

- structural Markdown analysis;
- stable document, section, and chunk identifiers;
- exact byte offsets for source navigation and guarded edits;
- block-aware chunking that preserves fenced code blocks;
- explicit wikilink extraction;
- SQLite storage with FTS5 chunk search;
- incremental vault rebuild based on BLAKE3 content hashes;
- stale-document pruning;
- validated chat action contracts.

## Planned modules

- canonical tag taxonomy and aliases;
- structured model-provider interface;
- title and tag suggestions with evidence;
- typed knowledge relations;
- graph projection for the existing graph UI;
- cited wiki proposals and incremental updates;
- approval and audit log for chat actions;
- migration away from the legacy embedding/search/wiki implementations.

## Chat action safety

Read-only search can execute directly. Creating a wiki or changing a note requires approval. Existing-note mutations must include the content hash observed by the model, preventing an older chat response from overwriting a newer user edit.
