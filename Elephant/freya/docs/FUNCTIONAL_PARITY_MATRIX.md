# Freya migration — functional parity matrix

Status: **living acceptance matrix**. This file complements `TAURI_FUNCTIONAL_REFERENCE.md` and is intentionally stricter than a feature checklist.

## Reference hierarchy

1. `Elephant/frontend/app/**` is the current Tauri/Vue behaviour reference.
2. `Elephant/backend/tauri/**` is the current native/runtime/storage reference.
3. `Elephant/frontend/src/renderer/src/**` is consulted only for behaviour that is still intentionally carried by the current app through legacy editor modules or has no current `frontend/app` replacement.
4. `Elephant/freya/**` is the migration implementation and cannot be used as the sole evidence for defining parity.

## Status vocabulary

- `DISCOVERED`: reference surface/file found, behaviour not yet fully specified.
- `SPECIFIED`: observable contract written, implementation may still diverge.
- `IMPLEMENTED-UNPROVEN`: Freya implementation exists but current branch has no complete passing functional proof yet.
- `PARTIALLY-PROVEN`: functional tests prove meaningful paths but important boundaries remain.
- `PARITY-PROVEN`: current Tauri behaviour has explicit contracts, real effects are exercised, failure/boundary paths are covered, and the current CI branch is green.
- `FAILING`: a current functional/differential proof demonstrates a divergence.

A source file or a passing unit test alone never upgrades a surface to `PARITY-PROVEN`.

---

## 1. Vault opening, registry and switching

**Current Tauri/runtime reference**

- `frontend/app/stores/vaultStore.js`
- `backend/tauri/src/vault/**`
- `backend/tauri/src/vault_layout.rs`

**Freya implementation/proofs already present**

- `vault_adapter_reproducibility.rs`
- `vault_registry_freya_testing.rs`
- `vault_inaccessible_freya_testing.rs`
- `shell_navigation_freya_testing.rs`
- `freya_shell_acceptance.rs`

**Status:** `PARTIALLY-PROVEN` pending a green current-head CI run.

### VAULT-001 — First launch with no usable vault

The application must expose an actual vault chooser/recovery state and must not fabricate a vault. A cancelled picker leaves the application in a recoverable no-vault state.

### VAULT-002 — Opening a vault

Opening an existing directory must load its real root entries, register/activate it in the canonical vault registry, and remember it for later launches. A failed registration or persistence operation must be observable.

### VAULT-003 — Switching vaults

Switching vaults must replace every vault-scoped state boundary: current directory, opened note/drawing, Library data, workspace metadata and addon/runtime context. Data from vault A must never remain actionable after vault B becomes active.

### VAULT-004 — Missing registered vault

A registry entry whose directory disappeared must fail visibly while preserving other registered vaults as recovery actions.

### VAULT-005 — Restart persistence

After a successful switch, a new process must restore the selected vault from persisted state rather than an in-memory singleton.

**Still required:** one full UI functional test that creates two physical temporary vaults, switches A→B→A, restarts, and proves file isolation at every step.

---

## 2. Library, cards, pagination and navigation

**Current Tauri reference**

- `frontend/app/components/library/LibraryGrid.vue`
- `frontend/app/components/library/LibraryToolbar.vue`
- `frontend/app/components/library/NoteCard.vue`
- `frontend/app/components/library/FolderCard.vue`
- `frontend/app/stores/vaultStore.js`

**Important exact current contract:** `NoteCard.vue` uses `CLICK_OPEN_DELAY_MS = 220`. Card activation is deferred so a title double-click can become Rename without first opening the entry.

**Freya proofs**

- `library_basic_controls_freya_testing.rs`
- `library_create_folder_freya_testing.rs`
- `library_double_click_freya_testing.rs`
- `library_note_actions_freya_testing.rs`
- `library_pagination_parity_freya_testing.rs`
- `library_pin_action_freya_testing.rs`
- `library_settings_freya_testing.rs`
- `library_toolbar_reference_freya_testing.rs`
- `navigation_back_forward_freya_testing.rs`
- `navigation_history_extended_freya_testing.rs`

**Status:** `IMPLEMENTED-UNPROVEN` on the latest branch until all updated independent suites pass together.

### LIB-OPEN-001

A card single-click schedules the logical entry activation after the current 220 ms Tauri card delay.

### LIB-OPEN-002

A title double-click enters inline Rename and cancels the pending open. The second click must never land on a newly mounted child because the parent was opened too early.

### LIB-PAGE-001

Backend directory pages are 120 entries; the render window is bounded separately. Reaching the bottom must be able to request successive pages until the actual final page, not only page 2.

### LIB-DND-001

Internal card drag/drop moves the real filesystem entry only onto an allowed directory and refreshes the Library. Invalid self/descendant/parent targets must not mutate disk.

### LIB-SIDEBAR-001

Folder “Show/Hide from sidebar” mutates the canonical `.elephantnote/config/workspace.json` sidebar array and the live sidebar consistently.

---

## 3. Autosave, close/save and external filesystem changes

**Current Tauri reference**

- `frontend/app/components/editor/NoteEditorHost.vue`
- `frontend/app/stores/vaultStore.js`

Observed current Tauri constants in `NoteEditorHost.vue`:

- autosave polling: 500 ms;
- default save delay: 160 ms;
- large edit delay: at most 60 ms for deltas ≥ 8 KiB;
- huge edit delay: at most 20 ms for deltas ≥ 64 KiB.

**Existing Freya proofs**

- `editor_lifecycle_freya_testing.rs` covers canonical autosave policy, immediate close flush, save-shortcut persistence and save failure visibility.
- `vault_watch_freya_testing.rs` covers physical directory fingerprint refresh.
- `editor_external_change_freya_testing.rs` now exercises the real watcher with an open note.

**Status:** `IMPLEMENTED-UNPROVEN` pending latest CI.

### SAVE-001 — Real-time persistence

A user edit must change the actual Markdown file according to the active autosave policy. “Saved” visual state without a successful file write is failure.

### SAVE-002 — Close flush

Closing a dirty note must flush the current Markdown before unmounting the editor. If the write fails, the editor remains open and dirty with a visible error.

### EXT-001 — Clean external modification

If the physical note changes while the editor is clean, the watcher reloads the external revision. The next local edit must be based on that new revision, never on stale pre-change Markdown.

### EXT-002 — Dirty external modification

If disk changes while the editor has unsaved local edits, automatic reload must refuse to silently merge/overwrite. The conflict is visible, the dirty editor remains mounted, and external bytes are not overwritten by the watcher.

### EXT-003 — External create/delete/rename

Creating/deleting/renaming entries through the OS file explorer must be reflected in the current directory without restarting the app. Open-note deletion/rename requires a defined recovery contract and a dedicated test.

---

## 4. Tabs and opened-note identity

**Current Tauri evidence**

`frontend/app/components/editor/NoteEditorHost.vue` still uses the legacy `EditorWithTabs`/`useEditorStore`, resolves an opened physical note against `editorStore.tabs`, and deduplicates by normalized pathname. However the current host passes `show-tab-bar="false"`.

**Status:** `DISCOVERED` / product behaviour needs exact specification before claiming visible tab parity.

### TAB-001 — Identity

The same physical pathname may have only one logical editor tab/document identity. Re-selecting it must not duplicate mutable editor state.

### TAB-002 — Current modern UI

Because the current `NoteEditorHost` explicitly hides the tab bar, Freya must not invent a visible multi-tab bar as “parity” unless a later product decision intentionally supersedes the current Tauri UI.

### TAB-003 — Legacy compatibility boundary

Underlying editor APIs that still expect `editorStore.tabs` must continue to receive the correct active file even when the tab bar itself is hidden.

**Still required:** characterize open A → open B → reopen A, dirty-state retention, and close behaviour against the current Tauri host rather than the older standalone tab UI.

---

## 5. Markdown editor behaviour

**Current Tauri reference**

- `frontend/app/components/editor/NoteEditorHost.vue`
- `NoteEditorTopBar.vue`, `NoteEditorToolbar.vue`, `NoteEditorFooter.vue`
- legacy `EditorWithTabs`/Muya modules imported by the current host
- current document/meta helpers under `frontend/app/utils/**`

**Freya proofs already present**

- `editor_keyboard_freya_testing.rs`
- `editor_rich_interactions_freya_testing.rs`
- `editor_toolbar_parity.rs`
- `editor_undo_redo_freya_testing.rs`
- `editor_link_freya_testing.rs`
- `editor_tags_freya_testing.rs`
- `markdown_tags_freya_testing.rs`
- `muya_note_lifecycle_freya_testing.rs`
- `note_title_edit_freya_testing.rs`

**Status:** `PARTIALLY-PROVEN` pending current CI and broader syntax characterization.

Required characterisation set must include at minimum:

- Enter/new paragraph and empty lines;
- headings H1–H6;
- emphasis/strong/strike;
- inline code and fenced code;
- ordered/unordered/task lists including indentation/outdent;
- blockquote;
- links, wikilinks and fragments;
- tables;
- horizontal rules;
- undo/redo across structural transformations;
- IME/non-ASCII input;
- clipboard paste of plain text and Markdown;
- selection replacement and multiline deletion;
- very large document edits;
- cursor survival across save/re-render.

Every mutation test that is expected to persist must additionally inspect the physical Markdown file.

---

## 6. Global Search overlay and editor find/replace

### Global Search

**Current Tauri reference**

- `frontend/app/components/shell/AppShell.vue`
- `frontend/app/search/SearchModal.vue`
- `SearchResultItem.vue`, `SearchSettingsPanel.vue`, `SearchStatusBadge.vue`

`SearchModal` is mounted as a sibling of `MainContent`. Search therefore overlays, rather than replaces, the active Library/editor/workspace.

**Freya proofs**

- `search_mode_freya_testing.rs`
- `search_runtime_freya_testing.rs`
- `explorer_search_graph_freya_testing.rs`
- `search_overlay_workspace_freya_testing.rs`
- differential Tauri/Freya journey

**Status:** `IMPLEMENTED-UNPROVEN` after fixing the previously observed `search-open` differential frame.

### SEARCH-OVERLAY-001

Opening Search preserves the underlying workspace and its loaded data while mounting the Search input/modal above it.

### SEARCH-OVERLAY-002

Closing Search restores interaction with the exact same underlying workspace state; it does not reload/reset the Library solely because Search closed.

### Editor find/replace

The older renderer `frontend/src/renderer/src/components/search/index.vue` defines document-level Find/Replace with case-sensitive, whole-word and regex toggles plus replace-one/replace-all. No current modern replacement was established during this inspection.

**Status:** `SPECIFIED FROM LEGACY REFERENCE`, not parity-proven.

Before migration parity can be claimed, determine whether this behaviour is still reachable through the current editor command surface; if yes, add real-document tests and physical-file checks for replace operations.

---

## 7. Context menus and card actions

**Current Tauri reference:** `frontend/app/components/library/NoteCard.vue`.

Confirmed current card actions include pin/unpin, action popover, Rename, Delete, and for folders Show/Hide from sidebar. Right-click opens the card action menu.

**Freya status:** `PARTIALLY-PROVEN` through Library action tests.

### CONTEXT-001

Right-click/action invocation must target the card under the pointer, never stale previously selected state.

### CONTEXT-002

Rename uses an inline prefilled input, Enter commits, Escape cancels, and clicks inside the editor must not also open the card.

### CONTEXT-003

Delete must mutate the real vault/trash contract and failure must remain visible.

**Still required:** enumerate and characterize non-Library context menus in current Tauri editor/sidebar surfaces.

---

## 8. Drag/drop into the editor, files and images

**Current Tauri reference:** `frontend/app/components/editor/NoteEditorHost.vue`.

The current editor shell is an explicit `data-entry-drop-target="note-editor"` with `dragover` and `drop` handlers. It imports the shared dragged-entry parser, local image resolver, Markdown image-source conversion and hidden-assets helpers.

The current host resolves the vault hidden asset directory through `ELEPHANTNOTE_ASSETS_DIR`, creates it when needed, sanitizes asset names, and allocates collision-free destination names.

**Status:** `DISCOVERED` / needs complete functional proof.

### DROP-001 — Internal note/folder/file drop

Dropping a vault entry into an open note inserts the appropriate Markdown representation at the editor insertion point and must not move/delete the source entry unless the contract explicitly says move.

### DROP-002 — External image drop

An external local image must be copied into the canonical hidden asset directory, receive a safe collision-free name, and insert a Markdown image reference that resolves after restart.

### DROP-003 — External non-image file drop

A supported external file must produce a clickable Markdown link/reference. The file-opening behaviour must resolve through the installed addon/runtime when one owns the type, otherwise through the platform default application according to the current product contract.

### DROP-004 — Asset safety

No dropped path may escape the vault through `..`, symlink/path confusion, or absolute-path injection. Existing assets must not be silently overwritten on name collision.

### DROP-005 — Position

Insertion must occur at the user's intended editor position/selection, not always at document end.

**Required next tests:** PNG/JPEG with duplicate names, non-image PDF/text file, internal note link drop, traversal-like filename, restart and link resolution.

---

## 9. Excalidraw / drawings

**Current Tauri reference**

- `frontend/app/components/editor/ExcalidrawDialog.vue`
- drawing paths/previews in Library components
- settings `ExcalidrawMark.vue`

**Freya proofs**

- `drawing_freya_testing.rs`
- `drawing_canvas_freya_testing.rs`
- `canvas_freya_testing.rs`

**Status:** `PARTIALLY-PROVEN` pending current CI and full round-trip parity.

Required end-to-end contract:

1. Create Drawing from the real Create menu.
2. Persist drawing data under the canonical vault asset/drawing path.
3. Show a Library preview.
4. Reopen the same drawing for editing.
5. Save changes and reopen after process restart.
6. Preserve normal note/vault data when the drawing runtime fails.

No placeholder/native fake canvas counts as Excalidraw parity if the Tauri path uses the actual Excalidraw surface.

---

## 10. Settings and preferences

**Current Tauri reference**

- `frontend/app/components/settings/SettingsPanel.vue`
- `IconRailLayoutSettings.vue`
- `AddonsSettingsPanel.vue`, `AddonSettingsRow.vue`
- `AiProviderSettingsPanel.vue`, `ChatgptSubscriptionCard.vue`
- `SyncSettingsPanel.vue`

**Freya proofs**

- `settings_effects_freya_testing.rs`
- `settings_modal_lifecycle_freya_testing.rs`
- `settings_parity_freya_testing.rs`
- settings portion of `library_settings_freya_testing.rs`

**Status:** `IMPLEMENTED-UNPROVEN` pending updated suites.

### SETTINGS-001

Changing a preference must update the live behaviour and canonical persisted preference value without dropping unknown/future keys.

### SETTINGS-002

Malformed persisted preferences are visible as an error and are not silently replaced with defaults on disk.

### SETTINGS-003

Search within Settings is live; opening a result selects the exact owning section/control and exits search-result mode.

### SETTINGS-004

Controls with bounds/disabled states must enforce those semantics both visually and in persistence.

### SETTINGS-005

Addon settings must exercise an actual runtime package/worker, not a registry-only fake.

---

## 11. Import / export

The older renderer contains explicit Import and Export Settings surfaces. A complete modern `frontend/app` equivalent has not yet been established in this inspection.

**Status:** `DISCOVERED`, not yet specified against the modern Tauri surface.

Before Freya parity can be claimed, determine current reachable commands and define:

- supported import source formats;
- conflict/duplicate naming rules;
- folder hierarchy preservation;
- asset copying/relinking;
- export target formats;
- export of images/assets/drawings;
- cancellation and partial-failure atomicity;
- no mutation of source files.

Every importer must be tested with a physical fixture tree and byte-level source preservation.

---

## 12. Wiki

**Current Tauri evidence:** `frontend/app/components/views/wikiViewHelpers.js` and addon/workspace routing; Freya has `wiki_view_freya_testing.rs`.

**Status:** `PARTIALLY-PROVEN` only for the currently implemented native view paths.

Required product-level contract still includes:

- proposal visibility;
- accept;
- reject;
- direct creation;
- update existing wiki note;
- source citations/links;
- search notes as source material;
- strict write confinement under `Wiki/**` when the operation is a Wiki creation/update;
- no silent writes elsewhere on failed/ambiguous proposal.

A complete proof must inspect the physical `Wiki/` tree after every accept/reject/create path.

---

## 13. Graph, including >200 notes

**Current Tauri reference:** `frontend/app/components/views/AtomicGraphView.vue` plus `frontend/app/graph/**`.

**Freya proofs currently discovered:**

- `explorer_search_graph_freya_testing.rs` proves two real Markdown notes, a real `[[Beta]]` edge, filtering, selection, recenter and refresh failure.
- `graph_canvas_freya_testing.rs` covers graph-canvas interaction.

**Status:** `PARTIALLY-PROVEN`.

### GRAPH-001

Nodes are derived from real notes and edges from real wikilinks/backlinks according to the current graph adapter.

### GRAPH-002

Tags and filters affect visibility without deleting graph data.

### GRAPH-003 — hard regression: no silent 200-node limit

A physical vault containing at least **205 Markdown notes** must expose all expected nodes after rebuild. The test must specifically assert nodes 000, 199, 200 and 204 plus total count. A result of exactly 200 is failure.

### GRAPH-004

SVG/native rendered graph surface must remain usable with >200 nodes: no panic, empty viewport, stale previous graph or silently truncated interaction list.

This 205-note functional test is still required.

---

## 14. Knowledge

A dedicated complete current Tauri source inspection has not yet been completed in this matrix pass.

**Status:** `DISCOVERED/UNPROVEN`.

Required contract before parity declaration:

- sidecar/service starts from the real application path;
- index rebuild operates on the physical current vault;
- search returns results from the rebuilt index;
- inspect/details are consistent with indexed files;
- knowledge graph is generated from the same indexed corpus;
- stopping/restarting does not corrupt index state;
- switching vaults cannot leak indexed results from the previous vault;
- missing/corrupt sidecar/index produces visible recoverable failure.

No mocked process or prewritten result JSON counts as sidecar proof.

---

## 15. Open Models / local AI

**Current Tauri evidence:** `frontend/app/components/views/modelsViewHelpers.js`, AI provider settings, and existing Freya `models_activation_freya_testing.rs`.

**Status:** `PARTIALLY-PROVEN` at the view/activation boundary; sidecar/resource lifecycle still needs stronger proof.

Required contract:

- start real local model sidecar/server;
- stop it cleanly;
- expose model resources/state;
- activate/deactivate local AI provider;
- failed start is visible and does not leave an orphan process;
- application exit cleans owned process;
- restart restores only persisted configuration, never a fake running state.

---

## 16. Addons, addon API and permissions

**Current Tauri reference**

- `frontend/app/components/settings/AddonsSettingsPanel.vue`
- `AddonSettingsRow.vue`
- `frontend/app/components/views/AddonWorkspaceHost.vue`
- `AddonWorkspaceRouter.vue`
- addon runtime/backend modules

**Freya proofs**

- `addon_lifecycle_freya_testing.rs`
- `addon_runtime_freya_testing.rs`
- `addon_worker_freya_testing.rs`
- real-worker path in `settings_parity_freya_testing.rs`
- `native_workspace_navigation_freya_testing.rs`

**Status:** `PARTIALLY-PROVEN` pending latest CI and capability matrix coverage.

Required capability matrix for each API family:

- read vault file;
- write/create within permitted vault paths;
- delete/trash;
- search;
- register view/command/settings contribution;
- open supported resource;
- sidecar/process capability where permitted;
- network capability where permitted;
- denial when permission is absent;
- denial of path traversal/out-of-vault access;
- enable/disable lifecycle and disposer execution;
- uninstall removes package state and contributions without deleting user notes.

Tests must distinguish full-rights and sandbox/permission-denied behaviour when both modes are supported.

---

## 17. Sync and conflicts

**Current Tauri reference:** `frontend/app/components/settings/SyncSettingsPanel.vue` plus backend sync modules.

**Status:** `DISCOVERED/UNPROVEN` in this matrix pass; existing Freya sync view code is not by itself a full P2P proof.

Required two-peer functional matrix:

- pair two clean vault peers;
- upload A→B;
- download B→A;
- create directory/note;
- update same note;
- delete propagation;
- offline edit then reconnect;
- simultaneous conflicting edits produce the defined conflict artifact rather than silent last-writer loss;
- excluded configuration paths are not synchronized;
- conflict cleanup retention policy is respected;
- process restart/resume;
- corrupted/unknown peer data cannot escape the vault root.

A single-process state-machine unit test cannot mark P2P sync `PARITY-PROVEN`.

---

## 18. Bazzite / Linux packaging

**Current Tauri packaging reference:** `backend/tauri/tauri.linux.conf.json` defines Linux bundle targets `deb` and `appimage`, with a frameless resizable 1280×840 main window.

**Freya status:** `UNPROVEN FOR PRODUCTION BAZZITE`.

Required Bazzite proof is not “cargo check on Linux”. It must use an installed/bundled artifact in a real graphical Linux/Wayland session and cover:

- cold launch;
- native vault picker;
- create/open/save/restart note;
- external filesystem watcher;
- drag/drop from file manager;
- image/file asset handling;
- clipboard;
- Excalidraw/web island if used;
- addon worker/runtime and sidecars;
- local AI process if shipped;
- file links opening through addon/default application;
- window resize/maximize/minimize/fullscreen;
- font/icon/resource bundling;
- no dependency on repository working directory;
- no writable-resource assumption inside immutable application bundle;
- clean logs/no panic during normal journey.

The production gate should exercise both the portable AppImage-like path chosen for Freya and, if retained, the native package/Flatpak route. The current Tauri `deb`/`appimage` configuration is reference evidence only; it does not prove the Freya executable is packaged correctly.

---

## 19. Priority order for additional functional tests

The next tests should be added in this order because failures can cause data loss or make other tests misleading:

1. external dirty-file conflict and clean reload — now added, CI pending;
2. two-vault switch/restart isolation;
3. editor external image + external file drop with physical `.assets` verification;
4. 205-note Graph no-truncation regression;
5. Wiki accept/reject/create physical `Wiki/**` confinement;
6. addon capability allow/deny matrix;
7. real Knowledge sidecar start/rebuild/search/inspect/graph;
8. real Open Models start/stop/resources/activation;
9. two-peer sync and conflict artifact;
10. packaged Bazzite graphical acceptance journey.

## 20. Global definition of done

For every row above, `PARITY-PROVEN` requires all of the following simultaneously:

- current Tauri source inspected;
- exact user action and observable effect specified;
- Freya path exercises the production implementation;
- physical filesystem/process/network side effect asserted when relevant;
- normal, boundary and failure case present;
- no fake sidecar, fake file effect or label-only proof;
- current branch CI runs the test independently;
- current branch CI is green;
- differential journey is within its configured visual/state thresholds where that surface participates.

Until then the matrix must retain the narrower status rather than using “done”.
