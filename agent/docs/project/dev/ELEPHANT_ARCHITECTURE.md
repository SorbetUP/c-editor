# Elephant Architecture

Elephant is separated from the legacy MarkText scaffold by stable aliases and
portable contracts. The current Electron app remains the first platform target,
but feature code should be placed so it can later be hosted by Android, Web, or
desktop shells without rewriting domain behavior.

## Layers

- `Elephant/shared`: pure domain contracts and helpers. This layer must not use
  Electron, DOM, Node-only IO, or Vue APIs. It is the portability boundary for
  API actions, feature flags, providers, calendars, wiki data, sources, and
  atomic workspace definitions.
- `Elephant/back/app`: Electron main-process adapters. This layer owns local
  filesystem access, vault persistence, native dialogs, process execution,
  search indexing, sync, importers, site preview servers, local model runtimes,
  and IPC registration.
- `Elephant/front/app`: Vue renderer adapters. This layer owns UI state,
  components, keyboard/drag interactions, graph rendering, editor hosting,
  settings panels, and calls into the portable API client.
- `src/preload`: the platform bridge. It exposes legacy IPC and the versioned
  `elephantnote.api` bridge to the renderer.
- `src/muya`: the Markdown editing engine inherited from MarkText. Treat it as
  an editor engine dependency until a dedicated Markdown service boundary fully
  wraps document parsing, transforms, and rendering.

## Portable API Contract

The versioned contract lives in `Elephant/shared/apiContracts.js`.

- `ELEPHANTNOTE_API_DOMAINS` groups actions by architectural domain.
- `ELEPHANTNOTE_API_ACTIONS` is generated from those domains for compatibility
  with existing code.
- `API_PAYLOAD_SCHEMAS` is generated from the same source, so a new action has
  one authoritative action name and one validator.
- `Elephant/shared/apiActions.js` and `Elephant/back/app/apiSchemas.js` remain
  compatibility facades for existing imports.

New cross-process features should start by adding or extending a domain contract
in `apiContracts.js`, then wiring the Electron handler in `Elephant/back/app`
and the renderer client method in
`Elephant/front/app/services/elephantnoteClient/domainClients.js`.

## Workspace Contract

The portable workspace and vault rules live in `Elephant/shared/workspace.js`.

- Workspace metadata file names, default workspace shape, welcome markdown,
  relative path normalization, ignored vault entries, Markdown frontmatter
  extraction, available-name generation, and sidebar normalization are pure
  shared rules.
- `Elephant/back/app/core.js` is the Electron/Node adapter for these rules. It
  re-exports the portable contract and adds filesystem path resolution for the
  active vault.
- `Elephant/back/app/vaults.js` owns persistence and native IO, but it should
  call the shared workspace contract for domain shape decisions.

Android/Web/Desktop ports should reuse `Elephant/shared/workspace.js` directly
and provide their own storage adapter instead of depending on Electron IPC or
Node filesystem APIs.

## Workspace Insights Contract

Derived knowledge views live in `Elephant/shared/workspaceInsights.js`.

- Workspace statistics, recent-note ordering, tag topics, calendar buckets, and
  the local note graph are pure functions over portable entry records.
- Vue stores may cache or expose those derived models, but they should not own
  graph semantics.
- Graph renderers such as `GraphView`, `LocalGraphView`, `SigmaCanvas`, and
  future Web/Desktop/Android canvases should consume this model instead of
  rebuilding note/folder/tag relationships independently.

The current local graph model intentionally stays small: folder edges represent
filesystem containment and tag edges connect notes that share a topic. Future
backlink, Markdown AST, Excalidraw, semantic similarity, and plugin-provided
edges should extend the shared model before adding renderer-specific visuals.

## Markdown Document Contract

Persisted note Markdown rules live in `Elephant/shared/markdownDocument.js`.

- Frontmatter parsing, document title resolution, editor-body extraction,
  note-document rehydration, visible markdown statistics, title renaming, and
  frontmatter tag edits are pure shared functions.
- `Elephant/front/app/utils/noteDocument.js` and
  `Elephant/front/app/utils/markdownTags.js` are compatibility facades for
  existing renderer imports.
- `src/muya` and `EditorWithTabs` remain the current editing engine and UI host.
  They should receive editor-visible Markdown and return editor-visible
  Markdown; persisted document shape belongs to the shared contract.

Advanced Markdown features such as backlinks, callouts, custom blocks,
frontmatter schemas, embedded Excalidraw references, Markdown AST transforms,
and plugin-provided syntax should first extend the shared document contract or a
new shared Markdown AST adapter before adding renderer-specific controls.

## Excalidraw Asset Contract

Portable Excalidraw artifact rules live in `Elephant/shared/excalidrawAssets.js`.

- MIME type, output file names, `.excalidraw` sidecar paths, PNG preview paths,
  save base-name normalization, background colors, and empty scene defaults are
  shared pure data/functions.
- `Elephant/front/app/services/excalidraw.js` owns the renderer-specific
  Excalidraw package loading, blob conversion, image dimension probing, and
  export calls.
- `Elephant/front/app/components/editor/ExcalidrawDialog.vue` owns the React
  mount inside Vue and should not encode persistence naming rules.

Future Android/Web/Desktop ports should reuse the shared artifact contract and
provide their own file/blob storage adapter plus platform-specific Excalidraw
host.

## Appearance Contract

Portable theme and vault icon definitions live in
`Elephant/shared/appearance.js`.

- Theme ids, local-storage key names, CSS token maps, and theme normalization
  are pure shared data/functions. `AppShell.vue` applies those tokens to the
  DOM, but it should not own token semantics.
- Vault icon options are stored as portable ids plus display labels and generic
  Lucide component names. Renderer code maps those ids to concrete Vue icon
  components; main-process persistence stores only normalized ids.
- Legacy icon aliases such as `book` are normalized by the shared contract so
  existing vault metadata remains compatible while future platforms can render
  their own icon sets.

Future theme packs, custom icon sets, plugin-provided icons, or per-vault
appearance settings should extend this shared appearance contract before adding
platform-specific controls.

## Sync Contract

Portable sync operation shapes live in `Elephant/shared/sync.js`.

- Allowed operations, their default order, queue item shape, status snapshots,
  history records, and sync error codes are pure shared data/functions.
- `Elephant/back/app/sync/GitSyncEngine.js` is the current Electron/Node
  adapter that executes those portable operations through Git.
- Renderer code should enqueue operations from the shared plan instead of
  hard-coding operation strings.

Future cloud, LAN, mobile, or vault-provider sync adapters should keep the same
shared operation/status contract and replace only the platform execution layer.

## Extensions Contract

Portable plugin and automation definitions live in
`Elephant/shared/extensions.js`.

- Built-in plugin manifests, plugin runtime route names, task templates, task
  action ids, merged plugin/task state, and task result shapes are pure shared
  data/functions.
- `Elephant/shared/atomicWorkspace.js` re-exports these definitions for
  existing Atomic settings/catalog imports.
- `Elephant/back/app/vaults.js` remains the current Electron adapter that maps
  plugin runtime routes and task action ids to local functions such as calendar
  sync, source ingestion, MCP tool calls, wiki proposals, and vault scans.

Future plugin hosts should add manifest/runtime contracts in the shared
extension layer first, then provide platform-specific adapters for execution,
permissions, sandboxing, and UI surfaces.

## Domain Ownership

- Core workspace and vaults: vault selection, directory listing, notes, folders,
  sidebars, entry moves, and metadata belong to the `vaults` and `documents`
  API domains, with portable data rules in `Elephant/shared/workspace.js`.
- Persistence and sync: local filesystem persistence stays behind
  `Elephant/back/app/vaults.js`; background sync operation shapes live in
  `Elephant/shared/sync.js`, while Git execution belongs to
  `Elephant/back/app/sync`.
- Markdown and editor hosting: persisted document rules live in
  `Elephant/shared/markdownDocument.js`; renderer editor components live under
  `Elephant/front/app/components/editor`; engine-specific Muya calls should be
  wrapped before adding advanced Markdown behavior.
- Knowledge graph, wiki, RAG, and search: query/index behavior belongs to
  `Elephant/back/app/search` and the `knowledge` API domain; derived local graph
  semantics live in `Elephant/shared/workspaceInsights.js`; graph visualization
  stays in renderer graph/view components.
- Local AI and models: provider configuration, model selection, model download,
  and Atomic catalog actions belong to the `aiRuntime` API domain. Runtime
  processes stay in `Elephant/back/app/modelRuntime.js`,
  `Elephant/back/app/atomic`, and `Elephant/back/app/runtime`.
- Themes and icons: shared definitions should remain pure data in
  `Elephant/shared/appearance.js`; rendering and asset choices belong to
  `Elephant/front/app`.
- Plugins, MCP, tasks, and programs: manifests, runtime route ids, task actions,
  and result shapes live in `Elephant/shared/extensions.js`; extension
  execution belongs to the `plugins` and `automation` API domains.
  Shell/process execution stays in the main process.
- Excalidraw and canvas: artifact naming and sidecar rules live in
  `Elephant/shared/excalidrawAssets.js`; renderer integration lives in
  `Elephant/front/app/services/excalidraw.js` and canvas/editor components;
  persisted canvas artifacts should go through the documents/vault boundary.
- Publishing/site preview: local servers and builds belong to
  `Elephant/back/app/sitePreview`; renderer controls stay in
  `Elephant/front/app/sitePreview`.

## Portability Rules

- Shared modules are data and pure functions only.
- Renderer code calls `elephantnoteClient` instead of raw IPC for new behavior.
- Main-process modules depend on shared contracts, not renderer components.
- Preload exposes transport, not business logic.
- New feature flags, provider presets, plugin manifests, model catalogs, and
  sync operation shapes should be represented as shared contracts before adding
  UI around them.

## Final Extension Review

This review maps each heavy future feature named in the refactor goal to its
current portable boundary and platform adapter.

| Future area | Portable contract | Current adapters | Verification |
| --- | --- | --- | --- |
| Local AI and models | `Elephant/shared/aiProviders.js`, `Elephant/shared/atomicWorkspace.js`, `aiRuntime` API domain in `Elephant/shared/apiContracts.js` | Main runtime in `Elephant/back/app/modelRuntime.js` and `Elephant/back/app/runtime`; browser runtime in `Elephant/front/app/services/browserModelRuntime.js`; settings/chat UI consume the shared model catalog | `Elephant/tests/unit/aiProviders.spec.js`, `Elephant/tests/unit/atomicWorkspace.spec.js`, `Elephant/tests/unit/modelRuntime.spec.js`, `Elephant/tests/unit/api.spec.js` |
| Advanced Markdown | `Elephant/shared/markdownDocument.js` | Renderer compatibility facades in `Elephant/front/app/utils/noteDocument.js` and `Elephant/front/app/utils/markdownTags.js`; Muya remains an engine dependency behind editor components | `Elephant/tests/unit/markdownDocument.spec.js`, `Elephant/tests/unit/noteDocument.spec.js`, `Elephant/tests/unit/markdownTags.spec.js` |
| Note graph | `Elephant/shared/workspaceInsights.js` | Pinia vault store exposes shared derived models; graph views render those models or Atomic graph API results | `Elephant/tests/unit/workspaceInsights.spec.js`, `Elephant/tests/unit/vaultStore.spec.js` |
| Themes and icons | `Elephant/shared/appearance.js` | `AppShell.vue` applies shared CSS tokens; `IconRail.vue` maps portable vault icon ids to Lucide Vue components; `vaults.js` persists normalized icon ids | `Elephant/tests/unit/appearance.spec.js`, `Elephant/tests/unit/vaultStore.spec.js` |
| Sync | `Elephant/shared/sync.js`, `sync` API domain in `Elephant/shared/apiContracts.js` | `Elephant/back/app/sync/GitSyncEngine.js` executes the Git adapter; `navigationStore.js` enqueues the shared default operation plan | `Elephant/tests/unit/syncContract.spec.js`, `Elephant/tests/unit/sync/GitSyncEngine.spec.js`, `Elephant/tests/unit/api.spec.js` |
| Vaults and persistence | `Elephant/shared/workspace.js`, `vaults` and `documents` API domains in `Elephant/shared/apiContracts.js` | `Elephant/back/app/core.js` adapts shared workspace rules to filesystem paths; `Elephant/back/app/vaults.js` owns Electron dialogs and local IO | `Elephant/tests/unit/core.spec.js`, `Elephant/tests/unit/vaultStore.spec.js`, `Elephant/tests/unit/api.spec.js` |
| Plugins and automation | `Elephant/shared/extensions.js`, re-exported through `Elephant/shared/atomicWorkspace.js`; `plugins` and `automation` API domains | `Elephant/back/app/vaults.js` maps runtime route ids and task action ids to current Electron/local functions; settings UI consumes the shared manifests and tasks | `Elephant/tests/unit/extensions.spec.js`, `Elephant/tests/unit/atomicWorkspace.spec.js`, `Elephant/tests/unit/api.spec.js` |
| Excalidraw | `Elephant/shared/excalidrawAssets.js` | `Elephant/front/app/services/excalidraw.js` loads the Excalidraw package, probes blobs and exports scenes; `ExcalidrawDialog.vue` hosts the React canvas | `Elephant/tests/unit/excalidrawAssets.spec.js`, `Elephant/tests/unit/excalidraw.spec.js` |

The remaining legacy MarkText/Muya code is intentionally treated as an editor
engine dependency. New platform work should extend the shared contracts above
first, then add Android/Web/Desktop adapters without moving domain behavior
into renderer components, preload bridges, or Electron-only modules.
