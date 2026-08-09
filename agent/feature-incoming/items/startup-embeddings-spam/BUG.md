# Startup embeddings spam

- Request: Fix startup warnings from llama.cpp context initialization and prevent eager embedding/index building on app boot.
- Level: medium
- Date: 2026-06-19
- Slug: startup-embeddings-spam

## Summary
- [ ] Application startup emits repeated llama.cpp embedding warnings before the user interacts with search.

## Repro Steps
- [ ] Launch the app with `pnpm dev`.
- [ ] Open a workspace with notes.
- [ ] Observe startup logs and the search initialization path.

## Environment
- [ ] Desktop Electron app on the local workspace.
- [ ] Search uses the node-llama-cpp embedding runtime when available.

## Observed vs Expected
- Observed: startup logs show repeated `n_ctx_seq` and embedding override warnings, and search indexing starts before any explicit search action.
- Expected: startup should stay quiet, and embeddings/indexing should only begin when search is explicitly opened, queried, or rebuilt.

## Hypotheses
- [ ] The vault payload path triggers `search.initVault` during boot.
- [ ] `initVault` currently builds the semantic index eagerly.
- [ ] Semantic inspection may also rebuild the index implicitly.

## Investigation Plan
- [ ] Trace the boot path from vault loading to search initialization.
- [ ] Confirm whether `initVault` or `inspect` triggers embedding work.
- [ ] Add a regression test that asserts no embedding happens at `registerWindowVault` time.

## Fix Plan
- [ ] Make search initialization passive and move index creation to explicit rebuild or semantic query paths.
- [ ] Stop boot-time vault loading from calling search initialization eagerly.
- [ ] Keep deterministic fallback behavior for semantic search when no embedding model is configured.

## Regression Tests
- [ ] Unit test `registerWindowVault` does not call `embedText`.
- [ ] Unit test exact search works without building the semantic index.
- [ ] Unit test semantic search still builds on demand when the user asks for it.

## Release Notes
- [ ] Startup no longer runs semantic embedding work before the user asks for search.

## Risks
- [ ] First semantic query may still pay the indexing cost, but only when requested.
- [ ] Any UI that relied on eager initialization must tolerate a `not_initialized` status.

## Rollout
- [ ] Verify targeted search tests locally.
- [ ] Run the full unit suite after the change.
