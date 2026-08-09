# ElephantNote knowledge organization roadmap

This roadmap defines the target architecture and implementation plan for zero-manual-organization knowledge retrieval in ElephantNote.

The goal is not to add a better search bar only. The goal is to build a personal knowledge system where notes, documents, sources, images, OCR text, generated wikis, and LLM answers are connected through a verifiable concept graph.

## 1. Product objective

ElephantNote should allow a user to drop notes and documents without organizing them manually, while the application automatically builds:

- reliable search;
- semantic clustering;
- concept wikis;
- wiki-to-wiki links;
- source-backed summaries;
- chunk-level retrieval;
- LLM context routing;
- maintenance suggestions such as split, merge, rename, stale wiki, and missing citation warnings.

The expected behavior for ambiguous queries is not to guess one precise answer. For example, a query such as `apple` should surface candidate concepts:

- `Apple Inc.`;
- `Apple fruit`;
- `Apple typography`;
- ambiguous or mixed results.

The user can then select a concept, or the LLM can keep several candidate concepts through beam search and rerank them using evidence.

## 2. Core design principle

A wiki is not a folder and not only an index.

A wiki is the readable center of a semantic region. It should be a real note written in a wiki style, with:

- a stable concept title;
- a concise definition;
- a longer explanation;
- cited source chunks;
- child wikis;
- parent wikis;
- neighbor wikis;
- aliases;
- discriminative terms;
- positive examples;
- negative examples;
- ambiguity notes;
- freshness and confidence metadata.

The source of truth remains the original notes and chunks. Wiki pages are synthesis and routing layers. Every important wiki claim must be traceable to source chunks.

## 3. Existing implementation state

The current code already exposes important building blocks:

- `searchStore` supports `exact`, `semantic`, and `smart` modes.
- `searchLibrary` indexes Markdown files and supports exact and semantic search.
- `atomicAiEngine` contains deterministic embedding fallback, chunking, source extraction, tag suggestions, semantic links, and cited RAG helpers.
- `graphLibrary` builds note, folder, tag, and semantic graph edges.
- `wikiLibrary` builds wiki proposals from graph clusters.
- `domainClients` already exposes `wiki`, `search`, `rag`, `ocr`, `models`, and `atomicFeatures` domains.

Main limitation: the current graph clusters are too close to folder clusters. The target system needs concept clusters based on chunk-level semantic communities, not directory structure.

## 4. Target architecture

```text
Raw vault files
  -> document reader
  -> markdown/text/OCR extraction
  -> chunker
  -> lexical index
  -> vector index
  -> entity and alias index
  -> chunk similarity graph
  -> concept clustering
  -> concept wiki generation
  -> wiki graph
  -> search router
  -> RAG context builder
  -> cited LLM answer
```

The architecture must stay local-first and should degrade gracefully when no LLM or embedding model is available.

## 5. Scientific and technical anchors

Use the following research directions as design anchors:

- RAPTOR: recursive clustering and summarization for hierarchical retrieval.
- GraphRAG: graph/community summaries and global question answering.
- LightRAG: low-level and high-level retrieval over graph plus vectors.
- HippoRAG: graph-based long-term memory and Personalized PageRank retrieval.
- BERTopic: embedding-based topic modeling with c-TF-IDF topic representation.
- BEIR: robust retrieval evaluation and hybrid retrieval baselines.
- MTEB: embedding evaluation across clustering, retrieval, STS, classification, and reranking.
- RAGAS / ARES: RAG evaluation with context precision, context recall, faithfulness, and answer relevance.

Do not copy any paper directly. Use them as architectural references for ElephantNote's own constraints: personal notes, local-first execution, incremental updates, and interactive UX.

## 6. Data model additions

### 6.1 Chunk record

Add a durable chunk-level index.

```ts
interface KnowledgeChunk {
  id: string
  vaultId: string
  documentPath: string
  chunkIndex: number
  headingPath: string[]
  text: string
  textHash: string
  tokenCount: number
  language: string
  createdAt: string
  updatedAt: string
  embedding?: number[]
  embeddingModel?: string
  lexicalTerms: string[]
  entities: KnowledgeEntityRef[]
  citations: SourceCitation[]
}
```

### 6.2 Concept wiki record

```ts
interface ConceptWiki {
  id: string
  vaultId: string
  title: string
  slug: string
  aliases: string[]
  summary: string
  definition: string
  bodyPath: string
  status: 'proposed' | 'accepted' | 'stale' | 'needs-review' | 'archived'
  parentWikiIds: string[]
  childWikiIds: string[]
  neighborWikiIds: string[]
  positivePrototypeChunkIds: string[]
  negativePrototypeChunkIds: string[]
  memberChunkIds: string[]
  memberDocumentPaths: string[]
  discriminativeTerms: string[]
  excludedTerms: string[]
  confidence: number
  cohesion: number
  separation: number
  createdAt: string
  updatedAt: string
  lastRegeneratedAt: string
}
```

### 6.3 Chunk-to-wiki membership

```ts
interface ChunkWikiMembership {
  chunkId: string
  wikiId: string
  score: number
  confidence: number
  signals: {
    embedding: number
    lexical: number
    entity: number
    graph: number
    citation: number
    recency: number
    negativeEvidence: number
  }
  explanation: string
  createdAt: string
  updatedAt: string
}
```

### 6.4 Wiki graph edge

```ts
interface WikiGraphEdge {
  id: string
  sourceWikiId: string
  targetWikiId: string
  type: 'parent' | 'child' | 'neighbor' | 'contrast' | 'alias-risk' | 'source-overlap' | 'semantic-overlap'
  weight: number
  evidenceChunkIds: string[]
  explanation: string
}
```

## 7. Storage plan

Use hidden metadata files under the vault workspace directory.

Suggested files:

```text
.elephantnote/
  knowledge-index.json
  chunk-index.json
  vector-index.json or vector-index.sqlite
  concept-wikis.json
  wiki-memberships.json
  wiki-graph.json
  retrieval-eval.json
```

Initial implementation may use JSON for simplicity. Medium-term implementation should move large indexes to SQLite or a compact local index format.

## 8. Retrieval pipeline

### 8.1 Query normalization

Todo:

- [ ] Normalize accents and casing.
- [ ] Detect language.
- [ ] Extract quoted exact terms.
- [ ] Extract entities.
- [ ] Detect if query is broad, narrow, navigational, or question-like.
- [ ] Detect ambiguous short query such as `apple`, `kernel`, `phase`, `rust`, `python`.

### 8.2 Candidate generation

Todo:

- [ ] Add real lexical retrieval beyond `includes()`.
- [ ] Use ripgrep for fast exact file search.
- [ ] Add BM25 or BM25-like scoring for chunks.
- [ ] Keep current exact mode as fallback.
- [ ] Add fuzzy matching for typos and orthographic variants.
- [ ] Add vector retrieval over chunks.
- [ ] Add vector retrieval over wiki summaries.
- [ ] Add vector retrieval over wiki prototypes.
- [ ] Add graph expansion from initially matched chunks.
- [ ] Merge candidates with source-specific scores.

### 8.3 Concept routing

Todo:

- [ ] Retrieve top candidate wikis for the query.
- [ ] Keep top-k concepts rather than selecting only one.
- [ ] Implement semantic beam search over wiki hierarchy.
- [ ] Use query-to-wiki, query-to-prototype, and query-to-member-chunk similarities.
- [ ] Penalize wikis with strong negative evidence.
- [ ] Return ambiguous concept candidates when confidence is low.

Expected behavior:

```text
Query: apple
Result:
  1. Apple Inc.
  2. Apple fruit
  3. Apple typography
  4. Ambiguous chunks
```

### 8.4 Local retrieval inside selected concepts

Todo:

- [ ] Search inside member chunks of selected wikis.
- [ ] Expand to child wikis when query is broad.
- [ ] Expand to neighbor wikis when query contains ambiguity.
- [ ] Expand to parent wikis when too few chunks are found.
- [ ] Rerank chunks after graph expansion.

### 8.5 Final reranking

Todo:

- [ ] Build a lightweight reranker with deterministic features first.
- [ ] Add optional local cross-encoder or LLM reranking later.
- [ ] Include lexical score, vector score, concept-membership score, recency, source quality, graph centrality, and user interaction score.
- [ ] Keep explainable scoring for debugging.

## 9. Concept clustering pipeline

### 9.1 Chunk-first indexing

Todo:

- [ ] Replace document-level clustering with chunk-level clustering.
- [ ] Keep document-level metadata for UI and opening notes.
- [ ] Allow one note to belong to several wikis through different chunks.
- [ ] Store chunk hashes to avoid recomputing unchanged chunks.
- [ ] Preserve heading context for each chunk.

### 9.2 Similarity graph

Todo:

- [ ] Build a k-nearest-neighbor graph between chunks.
- [ ] Add edges for shared entities.
- [ ] Add edges for shared citations.
- [ ] Add edges for markdown links and backlinks.
- [ ] Add edges for folder proximity, but with lower weight.
- [ ] Add edges for tag proximity, but never let tags dominate.

### 9.3 Community detection

Todo:

- [ ] Implement a first simple connected-component/community algorithm.
- [ ] Then add Leiden or Louvain-style modularity clustering if dependency cost is acceptable.
- [ ] Compute cluster cohesion.
- [ ] Compute cluster separation from nearest other clusters.
- [ ] Flag mixed clusters for split.
- [ ] Flag tiny near-duplicate clusters for merge.

### 9.4 Topic representation

Todo:

- [ ] Compute c-TF-IDF-like discriminative terms per cluster.
- [ ] Extract representative chunks.
- [ ] Extract representative titles.
- [ ] Extract frequent entities.
- [ ] Extract contrastive terms against neighbor clusters.
- [ ] Generate candidate title and aliases.

## 10. Wiki generation pipeline

### 10.1 Draft generation

Todo:

- [ ] Generate a proposed wiki for each stable cluster.
- [ ] Use only cited chunks as evidence.
- [ ] Separate `definition`, `summary`, `details`, `sources`, `related wikis`, and `ambiguities`.
- [ ] Include a section: `What belongs here`.
- [ ] Include a section: `What does not belong here`.
- [ ] Include a section: `Ambiguous with`.
- [ ] Include source citations for every important claim.

### 10.2 Wiki body template

Target generated wiki format:

```markdown
# Apple typography

## Definition
Short definition of the concept.

## Summary
Readable synthesis with internal citations.

## What belongs here
- SF Pro, San Francisco font, typeface notes, Apple HIG typography.

## What does not belong here
- Apple Inc. business notes.
- Apple fruit or food notes.

## Key source notes
- [[path/to/note.md]] — relevant excerpt.

## Source chunks
- chunk id / heading / excerpt.

## Child wikis
- [[SF Pro]]
- [[Apple Human Interface Guidelines]]

## Neighbor wikis
- [[Apple Inc.]]
- [[Design systems]]

## Maintenance
- cohesion score
- separation score
- stale status
```

### 10.3 Verification

Todo:

- [ ] Add citation coverage check.
- [ ] Add unsupported-claim detector using deterministic heuristics first.
- [ ] Add optional LLM judge later.
- [ ] Mark wiki as `needs-review` when citation coverage is weak.
- [ ] Mark wiki as `stale` when many member chunks changed after generation.

### 10.4 Acceptance flow

Todo:

- [ ] Keep generated wikis proposed by default.
- [ ] Let user accept, dismiss, regenerate, split, or merge.
- [ ] Do not require user action for search to work.
- [ ] Use accepted wikis as stronger prototypes.
- [ ] Use dismissed wikis as negative feedback.

## 11. Wiki hierarchy and graph

Todo:

- [ ] Create parent wikis when too many sibling wikis exist.
- [ ] Create child wikis when a wiki is too large or internally mixed.
- [ ] Link neighbor wikis when they overlap but should not merge.
- [ ] Link contrast wikis when terms are ambiguous.
- [ ] Support multiple parents when concepts are not tree-shaped.
- [ ] Avoid strict binary-tree routing.
- [ ] Implement beam search over the wiki graph.
- [ ] Keep graph edges explainable with evidence chunks.

Important rule:

```text
The system is not a hard dichotomy tree.
It is a graph with hierarchical shortcuts.
```

## 12. LLM integration

### 12.1 Concept-aware RAG

Todo:

- [ ] Replace naive top-k chunks with concept-aware retrieval.
- [ ] Retrieve candidate wikis first.
- [ ] Retrieve chunks inside and around those wikis.
- [ ] Include wiki summaries as routing context.
- [ ] Include source chunks as factual context.
- [ ] Include neighbor/contrast wikis for ambiguous questions.
- [ ] Force the LLM to cite source chunks, not only wiki summaries.

### 12.2 Prompt structure

Target context:

```text
User query
Candidate concepts
Selected wiki summaries
Neighbor wiki warnings
Source chunks
Citation IDs
Required answer style
```

### 12.3 Answer behavior

Todo:

- [ ] If ambiguous, ask or show concept candidates.
- [ ] If enough evidence exists, answer directly with citations.
- [ ] If wiki summary conflicts with source chunks, prefer source chunks.
- [ ] If confidence is low, say so clearly.

## 13. UI and UX plan

### 13.1 Search modal

Todo:

- [ ] Split results into `Wikis`, `Notes`, `Passages`, `Sources`, and `Actions`.
- [ ] For broad ambiguous queries, show wikis first.
- [ ] For precise queries, show exact passages first.
- [ ] Add concept confidence badges.
- [ ] Add `Why this result?` debug panel.
- [ ] Add quick action: `Search inside this wiki`.
- [ ] Add quick action: `Ask using this wiki`.
- [ ] Add quick action: `Open related graph`.

### 13.2 Wiki page UI

Todo:

- [ ] Render generated wiki as a normal editable note.
- [ ] Show generated metadata in a collapsible panel.
- [ ] Show cited chunks.
- [ ] Show child and neighbor wikis.
- [ ] Show maintenance warnings.
- [ ] Allow regenerate/split/merge actions.

### 13.3 Graph UI

Todo:

- [ ] Add a concept graph mode separate from note graph mode.
- [ ] Let user click a wiki node to see source chunks.
- [ ] Dim unrelated nodes on hover.
- [ ] Highlight direct children, parents, neighbors, and contrast nodes.
- [ ] Show preview card with wiki summary and top cited notes.

## 14. Maintenance automation

Todo:

- [ ] Detect stale wiki when member chunks changed.
- [ ] Detect orphan chunks without wiki membership.
- [ ] Detect duplicate wikis.
- [ ] Detect overlarge wikis.
- [ ] Detect mixed wikis using low cohesion / high entropy.
- [ ] Detect weak wikis with too few citations.
- [ ] Detect ambiguous title collisions.
- [ ] Generate maintenance tasks but do not block normal usage.

## 15. Testing plan

### 15.1 Unit tests

Todo:

- [ ] `chunkAtomicMarkdown` preserves heading context.
- [ ] Chunk hashing is stable.
- [ ] Lexical index ranks exact title matches higher than body-only matches.
- [ ] Semantic index returns chunk-level matches.
- [ ] Concept membership uses positive and negative evidence.
- [ ] Wiki generation always includes sources.
- [ ] Wiki graph edges are explainable.
- [ ] Ambiguous query keeps multiple candidate wikis.

### 15.2 Fixture datasets

Create small deterministic fixtures:

```text
fixtures/knowledge/apple/
  apple-inc.md
  apple-fruit.md
  apple-typography.md
  mixed-apple-design.md
  ambiguous-apple.md
```

Expected behavior:

- [ ] Query `apple` returns three wikis.
- [ ] Query `SF Pro` ranks `Apple typography` first.
- [ ] Query `iPhone revenue` ranks `Apple Inc.` first.
- [ ] Query `apple pie` ranks `Apple fruit` first.
- [ ] Mixed note contributes chunks to several wikis.

### 15.3 Integration tests

Todo:

- [ ] Build index on a fresh vault.
- [ ] Rebuild index after editing a note.
- [ ] Move note and preserve memberships.
- [ ] Rename note and preserve citations.
- [ ] Delete note and remove stale memberships.
- [ ] Accept wiki and use it in later search.
- [ ] Dismiss wiki and ensure it does not reappear unchanged immediately.

### 15.4 RAG evaluation tests

Todo:

- [ ] Build a fixed question set.
- [ ] Compare chunks-only RAG vs wiki+chunks RAG.
- [ ] Measure context precision.
- [ ] Measure context recall.
- [ ] Measure answer faithfulness.
- [ ] Measure citation coverage.
- [ ] Add regression tests for hallucinated wiki claims.

## 16. Performance plan

Todo:

- [ ] Avoid rebuilding the full index on every file change.
- [ ] Use content hashes for incremental updates.
- [ ] Batch embeddings.
- [ ] Cache query embeddings.
- [ ] Keep exact search fast even before semantic index is ready.
- [ ] Move O(n²) similarity to approximate kNN or candidate pruning.
- [ ] Put expensive wiki generation behind background jobs or explicit rebuild.
- [ ] Add index-size and memory telemetry.

## 17. Security and privacy plan

Todo:

- [ ] Keep local-first indexing by default.
- [ ] Never send notes to remote providers unless user enabled that provider.
- [ ] Show which provider generated a wiki.
- [ ] Store model and provider provenance in wiki metadata.
- [ ] Redact API keys from logs.
- [ ] Keep source citation paths inside the vault only.
- [ ] Validate all file paths with existing path-safety helpers.

## 18. Implementation phases

### Phase 1: Foundation

- [ ] Add durable chunk index.
- [ ] Add chunk hash and incremental update support.
- [ ] Add BM25-like lexical scoring.
- [ ] Add chunk-level semantic search.
- [ ] Add tests for ambiguous Apple fixture.

### Phase 2: Concept index

- [ ] Add `ConceptWiki` and `ChunkWikiMembership` model.
- [ ] Build chunk similarity graph.
- [ ] Build first semantic communities.
- [ ] Compute discriminative terms.
- [ ] Generate concept candidates.
- [ ] Expose concept inspection API.

### Phase 3: Wiki generation

- [ ] Generate real wiki pages from concept candidates.
- [ ] Add citations and source chunk sections.
- [ ] Add parent, child, neighbor, and contrast links.
- [ ] Add accept/dismiss/regenerate flow.
- [ ] Add stale/wiki maintenance state.

### Phase 4: Concept-aware search

- [ ] Route queries through candidate wikis.
- [ ] Add beam search over wiki graph.
- [ ] Add search-inside-wiki.
- [ ] Add concept result sections to search modal.
- [ ] Add `Why this result?` explanations.

### Phase 5: Concept-aware RAG

- [ ] Use selected wikis as LLM routing context.
- [ ] Use source chunks as factual context.
- [ ] Include neighbor and contrast wikis for ambiguity.
- [ ] Add citation enforcement.
- [ ] Add RAG regression evaluation.

### Phase 6: Optimization and polish

- [ ] Replace JSON indexes with SQLite or compact local index if needed.
- [ ] Add approximate kNN for large vaults.
- [ ] Add maintenance dashboard.
- [ ] Add graph visualization for wiki concepts.
- [ ] Add telemetry-free local quality metrics.

## 19. Acceptance criteria

The feature is acceptable when:

- a user can dump hundreds or thousands of notes without organizing them manually;
- broad ambiguous queries show concept wikis instead of pretending there is one exact answer;
- one note can contribute different chunks to different wikis;
- generated wikis are readable and useful as real notes;
- every important wiki claim is backed by internal source citations;
- the LLM answers better with wiki+chunks than with chunks alone;
- stale or weak wikis are detected automatically;
- exact search remains fast and reliable;
- semantic search degrades gracefully without a local embedding model.

## 20. First concrete PR after this roadmap

Recommended first implementation PR:

```text
feat(search): add chunk-level knowledge index and ambiguous concept fixtures
```

Scope:

- create chunk index types;
- add stable chunk hashing;
- add Apple ambiguity fixture;
- add chunk-level lexical and semantic tests;
- expose chunk inspection in search inspect output;
- do not generate final wikis yet.

This keeps the first PR small enough while preparing the real concept wiki system.
