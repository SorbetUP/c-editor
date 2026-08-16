# Freya migration — exact functional contracts from the Tauri reference

Status: **normative migration document — active inventory**

This document records behaviour that is actually implemented by Elephant's Tauri/Vue application and converts it into explicit acceptance contracts for the Freya migration. It is intentionally stricter than a feature checklist: discovery, implementation and proof are different states.

## Reference precedence

Elephant currently contains more than one frontend generation. For migration parity the precedence is:

1. **Current Tauri application:** `Elephant/frontend/app/**`. This is the primary reference whenever the feature exists there.
2. **Legacy renderer:** `Elephant/frontend/src/renderer/src/**`. Use it only for functionality that has no newer implementation or when tracing historical behaviour.
3. **Freya typed contracts:** `Elephant/freya/src/*_contract.rs`. These are useful provenance maps, but the Tauri source remains the behavioural evidence.

This precedence matters. For example, the current Library is implemented under `frontend/app/components/library/**`; the old sidebar tree is not a substitute for the current Library card contract.

## Proof vocabulary

A feature may be reported as:

- **DISCOVERED** — code/component located, behaviour not fully read;
- **SPECIFIED** — observable actions/results written below from source evidence;
- **IMPLEMENTED-UNPROVEN** — Freya has code but no executing functional proof;
- **PARTIALLY-PROVEN** — some normal/boundary paths execute;
- **PARITY-PROVEN** — source contract, functional side effects and CI all agree;
- **FAILING** — an executing regression demonstrates a mismatch.

A label-only smoke test cannot prove a filesystem/editor feature. Where the contract mutates the vault, the test must inspect the physical fixture.

---

# 1. Current Library — exact Tauri reference

Primary sources:

- `Elephant/frontend/app/components/library/NoteCard.vue`
- `Elephant/frontend/app/components/library/FolderCard.vue`
- `Elephant/frontend/app/components/library/LibraryGrid.vue`
- `Elephant/frontend/app/components/library/LibraryToolbar.vue`
- `Elephant/frontend/app/components/library/CreateEntryMenu.vue`
- `Elephant/frontend/app/stores/vaultStore.js`

The Freya `library_contract.rs` already records the same modern provenance for sort/view, paging and entry payload semantics.

## LIB-OPEN-001 — one click requests card activation

Current `NoteCard.vue` attaches `@click="handleCardClick"` to the entire card.

A single click is sufficient to request opening. Tauri deliberately waits **220 ms** (`CLICK_OPEN_DELAY_MS`) before emitting `open` so that a double-click on the title can be distinguished from a normal open.

This delay is target-local state (`openClickTimer` on that `NoteCard` instance), not a global pointer-coordinate double-click detector.

### Required invariant

Two clicks on **different logical cards** must remain different actions even if a remount/reflow places the second card at the same global pointer coordinates as the first.

A test may wait for the documented 220 ms single-click timer when validating exact timing. What is forbidden as “proof” is adding an arbitrary delay merely to escape a global-coordinate bug. The semantic assertion is target identity, not elapsed time.

## LIB-OPEN-002 — title double-click renames instead of opening

The title `<h3>` has `@dblclick.stop.prevent="beginRename"`.

`beginRename`:

1. clears a pending open timer;
2. closes the actions menu;
3. enters rename mode;
4. initializes the draft to the current title;
5. focuses and selects the rename input after the DOM update.

Therefore title double-click semantics are scoped to the title/card instance. They must never suppress activation of a different card mounted later at the same screen coordinate.

### Freya status

Freya now keeps `EventsCombos` on the title path but no longer filters **card-body activation** by global-coordinate double-click state. This is the intended fix for the folder-remount regression; CI remains the proof authority.

## LIB-RENAME-001 — rename commit/cancel

While renaming, Tauri displays a text input instead of the title.

- click inside the input stops card activation;
- Enter calls `commitRename`;
- Escape calls `cancelRename`;
- commit trims whitespace;
- empty title is ignored;
- unchanged title is ignored;
- a real change emits `rename` with `{ entry, title }`.

**Required functional proof:** filesystem path changes, old path disappears, content/children survive, unrelated siblings remain unchanged.

## LIB-RENAME-002 — clicking the card while renaming cancels rename

`handleCardClick` checks `isRenaming` first. If true it cancels rename and returns instead of opening the entry.

This must be tested independently from Enter/Escape.

## LIB-MENU-001 — actions menu is card-local

Each card has a More button labelled `Folder actions` or `Note actions`. It toggles the card's own popover and stops/prevents normal card activation.

The popover exposes:

- Rename;
- Delete;
- for folders only, Show in sidebar / Hide from sidebar.

Right-click (`contextmenu`) on the card also opens this card-local menu.

A context action must never operate on a stale previously active entry.

## LIB-PIN-001 — pin/unpin is independent from opening

The pin button is visible while hovering or when already pinned. It is labelled `Pin entry` / `Unpin entry`, stops normal opening, calls `store.togglePinnedEntry(path)`, then closes the menu state.

Required test: pin B while A is open, prove B changes pin state and A/editor state is unaffected; then verify ordering/persistence according to `vaultStore`.

## LIB-SIDEBAR-001 — folder sidebar visibility

Folders have an action that toggles `store.toggleEntrySidebarVisibility(entry)`. Its label reflects current state (`Show in sidebar` / `Hide from sidebar`). The action is unavailable for non-folders.

## LIB-DELETE-001 — deletion is explicit and card-local

Delete closes the card menu and emits `delete(entry)`.

Required destructive proof must use an isolated vault fixture and assert:

- target removed;
- sibling files byte-identical;
- UI no longer exposes target;
- current navigation/editor state remains coherent.

## LIB-DND-001 — only folders accept library drops

Card drag starts by serializing the entry with its effective kind/title/preview. Drop handling only runs for folder targets.

On drag-over Tauri computes `canDropEntryOnDirectory(draggedEntry, targetPath)` and sets `dropEffect` to `move` only when allowed. An accepted drop calls `store.moveEntry(draggedEntry, targetPath)`.

Required tests must cover:

- note -> folder;
- folder -> folder;
- self drop rejection;
- descendant/cycle rejection;
- invalid target rejection;
- filesystem move plus visible refresh;
- no data loss after move.

## LIB-PREVIEW-001 — folder preview is bounded

A folder card shows at most the first **3** `childrenPreview` items. Empty folders display `No items yet` / `Empty folder` semantics.

Preview titles remove `.md` and `.excalidraw` display suffixes; icons distinguish folder, drawing and normal file/note.

## LIB-PREVIEW-002 — drawing preview lifecycle

For drawings, Tauri resolves the preview against the active vault, loads bytes when `fileUtils.readFile` is available, creates/revokes object URLs, guards stale async loads by `drawingPreviewLoadId`, and falls back to the source path on load failure.

Required tests: valid preview, missing preview, stale-load replacement and cleanup/no crash.

## LIB-CARD-001 — note metadata

Normal note cards expose the computed excerpt and all tags as `#tag`. Drawing and folder cards use their specialized bodies instead.

---

# 2. Library toolbar

Reference: `frontend/app/components/library/LibraryToolbar.vue`.

## LIB-TOOLBAR-001 — sort cycle order is exact

The cycle is:

1. `updated-newest` — label `Updated newest`;
2. `updated-oldest` — label `Updated oldest`;
3. `title-az` — label `Title A-Z`;
4. `title-za` — label `Title Z-A`;
5. back to `updated-newest`.

Legacy store value `title` is normalized to `title-az` for the toolbar.

**Freya functional coverage:** `library_toolbar_reference_freya_testing.rs` clicks the rendered control through the full cycle.

## LIB-TOOLBAR-002 — grid/list cycle is bidirectional

- when current view is grid, action label is `Show notes as list`;
- when current view is list, action label is `Show notes as grid`;
- each click toggles between the two states.

**Freya functional coverage:** `library_toolbar_reference_freya_testing.rs` executes both transitions.

## LIB-CREATE-001 — creation is disabled when unavailable

Create is disabled when the toolbar is busy or there is no active vault. It advertises `aria-busy` while a create action is running.

## LIB-CREATE-002 — three create actions

The current toolbar dispatches three creation keys:

- note -> `store.createNote()`;
- folder -> `store.createFolder()`;
- drawing -> `openNewDrawing()`.

The busy guard prevents concurrent create actions. Failure becomes a visible action error and is logged; busy state is cleared in `finally`.

**Freya existing coverage:** `create_folder_note_freya_testing.rs` proves physical folder creation and physical note creation/opening. Drawing creation requires its own end-to-end proof.

---

# 3. Library paging / ordering contracts

Reference: `LibraryGrid.vue`, `vaultStore.js`, and the provenance recorded in `freya/src/library_contract.rs`.

Current typed migration constants are:

- directory page size: **120**;
- render chunk size: **72**;
- scroll prefetch threshold: **720 px**.

Required proof includes datasets above every boundary: 71/72/73 rendered items and 119/120/121 directory items, followed by repeated paging to prove there is no silent fixed cap.

Sorting must be validated on actual rendered order, not only the enum/control label. Pinned-first ordering also needs independent functional proof.

---

# 4. Freya navigation/history contracts

These contracts describe the observable workspace created by the Library migration.

| ID | Scenario | Required result |
|---|---|---|
| FH-001 | Root -> folder | Folder contents replace root contents; editor absent. |
| FH-002 | Folder -> note | First logical activation of the newly mounted note succeeds; global pointer reuse cannot swallow it. |
| FH-003 | Note -> Back | Parent library state restored; editor absent. |
| FH-004 | Back -> Forward | Exact previous destination restored; Forward becomes unavailable at history tip. |
| FH-005 | Back -> new navigation | Stale Forward branch is invalidated. |
| FH-006 | Multiple Back | History restores logical states in order without phantom duplicates. |
| FH-007 | Close note | Closing never deletes/truncates physical Markdown bytes. |
| FH-008 | Reopen note | Persisted content is loaded again; no stale replacement/truncation. |

Functional coverage:

- `navigation_back_forward_freya_testing.rs` — base folder/note/back/forward journey;
- `navigation_history_extended_freya_testing.rs` — folder Back/Forward, Forward invalidation, close byte preservation, close/reopen.

The original failing implementation classified body clicks with `EventsCombos::pressed(global_location)`, so a newly mounted card at the same coordinates could be discarded. The migrated body path is now target activation without that global-coordinate filter; title double-click remains separate.

---

# 5. Legacy renderer contracts still relevant outside the modern Library

The following behaviours were inspected under `frontend/src/renderer/src/**`. They remain specifications only where no newer `frontend/app` implementation supersedes them.

## LEGACY-TREE-001 — Markdown tree file click

`components/sideBar/treeFile.vue` opens a Markdown file on one click. If already open it selects the existing tab; if already current it does nothing; non-Markdown entries return before the note-open path.

## LEGACY-TREE-002 — tree folder expansion

`components/sideBar/treeFolder.vue` toggles local collapse state on folder-name click. Create-in-folder forces expansion and focuses an empty input. Enter dispatches create. Inline rename initializes the existing name and commits non-empty values on Enter.

This tree expand/collapse contract is distinct from navigating into a modern Library folder card.

## LEGACY-SEARCH-001 — editor find/replace surface

`components/search/index.vue` exposes:

- explicit open/closed search state;
- active match index and match count;
- previous/next;
- case-sensitive toggle;
- whole-word toggle;
- regexp toggle;
- search vs replace mode;
- replace current and replace all;
- visible error state for invalid search/regexp input.

Parity tests must mutate/read the actual editor and persisted file for replace operations.

## LEGACY-TITLE-001 — title/document status

`components/titleBar/index.vue` exposes:

- `Elephant` when no filename is active;
- path context plus filename;
- unsaved `save-dot` when `isSaved` is false;
- filename rename action;
- word counter cycling word -> paragraph -> character -> all -> word;
- macOS title double-click maximize;
- custom non-macOS close/maximize/minimize controls.

Window behaviours require real platform/runtime tests; headless rendering alone is insufficient.

---

# 6. Surfaces to specify next

The following are discovered but are **not** considered specified merely by being listed:

- vault chooser/open/switch/reopen;
- startup/loading/error recovery;
- Recent notes;
- sidebar search/results;
- command palette and every command/shortcut;
- editor tabs and tab restoration;
- editor keyboard editing/Markdown transformations;
- autosave and external filesystem synchronization;
- context menus and every destructive action;
- external file drag into vault;
- image/file drop into note;
- executable code blocks;
- Excalidraw/drawing editing and persistence;
- settings and every preference;
- import/export, especially Google Keep;
- Wiki;
- Graph including >200 nodes;
- Knowledge sidecar/index/search/graph;
- Open Models sidecar/model resource lifecycle;
- addons API/runtime/permissions/inter-addon interactions;
- synchronization, deletes and conflict handling;
- mobile-specific interaction contracts;
- platform packaging/runtime on Bazzite, macOS and Windows.

Each section must be expanded into action/result/failure contracts and mapped to executable tests.

---

# 7. Test design rules

For every new functional test:

1. name the contract IDs it proves;
2. use the real user input path where practical;
3. assert the positive destination and absence of incompatible previous state;
4. verify filesystem/runtime side effects directly;
5. isolate destructive fixtures and prove siblings remain unchanged;
6. cover normal, boundary, failure and race/reflow paths;
7. distinguish a **documented product timer** (for example Tauri's 220 ms card-open delay) from an arbitrary sleep used to hide a bug;
8. never mark a feature proven until the CI workflow executing that test is green on the evaluated commit.

# 8. Definition of PARITY-PROVEN

A feature is PARITY-PROVEN only when:

- the highest-precedence Tauri implementation has been inspected;
- every non-superseded observable behaviour is written as a contract;
- Freya implements the contract;
- tests exercise normal + boundary + error/race paths;
- physical side effects are checked where applicable;
- CI actually executes the relevant tests and succeeds on the branch head.

Anything less must remain `SPECIFIED`, `IMPLEMENTED-UNPROVEN`, `PARTIALLY-PROVEN` or `FAILING`.
