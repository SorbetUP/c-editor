# Freya migration — functional contract from the Tauri reference

Status: **normative migration document, phase 1**

The goal of this document is not to describe an idealized Elephant. It records observable behaviour that is actually present in the Tauri/Vue application and turns that behaviour into explicit acceptance criteria for the Freya migration.

The Tauri implementation is the reference unless a later migration decision explicitly supersedes a behaviour.

## Evidence rules

A Freya feature is considered proven only when the relevant level below is satisfied.

1. **Visible** — the user can reach the control/state in a real Freya render.
2. **Interactive** — the real input path (mouse/keyboard/drag) changes the expected visible state.
3. **Effective** — the expected filesystem/runtime side effect occurs, not only a visual mock.
4. **Persistent** — when applicable, the effect survives close/reopen or re-reading the vault.
5. **Regression guarded** — an automated functional test executes the same user path.
6. **Differentially specified** — the expected result is traceable to the Tauri reference source.

Tests that only assert that an accessibility label exists are insufficient for a feature whose contract includes a filesystem or editor effect.

## Reference files inspected in phase 1

- `Elephant/frontend/src/renderer/src/components/sideBar/treeFile.vue`
- `Elephant/frontend/src/renderer/src/components/sideBar/treeFolder.vue`
- `Elephant/frontend/src/renderer/src/components/sideBar/index.vue` (component inventory)
- `Elephant/frontend/src/renderer/src/components/sideBar/search.vue` (component inventory)
- `Elephant/frontend/src/renderer/src/components/sideBar/searchResultItem.vue` (component inventory)
- `Elephant/frontend/src/renderer/src/components/search/index.vue`
- `Elephant/frontend/src/renderer/src/components/titleBar/index.vue`
- `Elephant/frontend/src/renderer/src/components/{about,commandPalette,editorWithTabs,exportSettings,import,loading,recent,rename,search,sideBar,titleBar,tweet}` (surface inventory)

The inventory is intentionally broader than the detailed contracts below. A feature is not marked specified merely because its component was discovered.

---

# 1. Vault tree / file navigation

Reference: `components/sideBar/treeFile.vue`.

## TFR-NAV-001 — Markdown file opens on one click

**Precondition**: a Markdown file is visible in the vault tree.

**Action**: one primary click on the file row.

**Expected**:

- the click is sufficient; no second click is required;
- if the file is not already in an editor tab, Tauri sends `mt::open-file` with the file pathname;
- if it is already open, the existing tab is selected instead of opening a duplicate;
- if the already-open file is already current, the action is a no-op.

**Migration invariant**: input de-duplication must be based on the logical target/action, never only on global pointer coordinates.

**Critical regression case**: a click on folder A followed immediately by a click on note B after a remount/reflow must activate B even if B appears under the same screen coordinate formerly occupied by A. Those are two logical activations, not a double-click on one target.

**Freya coverage**:

- `navigation_back_forward_freya_testing.rs` exercises a rapid `Projects -> Plan` sequence and currently acts as a regression gate for this invariant.
- `create_folder_note_freya_testing.rs` proves a created note can subsequently be opened into the real editor.

## TFR-NAV-002 — Non-Markdown tree file is not opened as a note

**Precondition**: a non-Markdown file is shown in the tree.

**Action**: primary click.

**Expected**: `handleFileClick` returns before the note-opening path. It must not be mounted as a Markdown editor document through this tree action.

**Freya test required**: explicit fixture containing a non-Markdown file and proof that the note editor is not mounted by the Markdown-open action.

## TFR-NAV-003 — Already-open note is reselected, not duplicated

**Precondition**: note N exists in the tab list and another note is current.

**Action**: primary click on N in the tree.

**Expected**: N becomes `currentFile`; a second tab for N is not created.

**Freya test required**: open A, open B, activate A again, assert a single logical A editor/tab and A content.

## TFR-NAV-004 — Clicking the current file is stable

**Precondition**: note N is already current.

**Action**: click N again.

**Expected**: no reopen/reload side effect is requested by the Tauri tree handler.

**Freya test required**: dirty editor state must not be destroyed or duplicated by reactivation.

---

# 2. Folder tree

Reference: `components/sideBar/treeFolder.vue`.

## TFR-FOLDER-001 — Folder click toggles expansion

**Action**: click the folder name row.

**Expected**: local `isCollapsed` is toggled. When expanded, child folders, creation input and files become visible; when collapsed, folder contents are hidden.

**Important distinction**: this sidebar-tree behaviour is not the same interaction as entering a directory in Freya's library page. Where Freya exposes both concepts, tests must distinguish **expand/collapse tree node** from **navigate library directory**.

## TFR-FOLDER-002 — Creating inside a folder expands it

When the `SIDEBAR::show-new-input` action targets the folder:

- the creation input receives focus;
- its value is reset to empty;
- the folder is forced expanded so the input is visible.

Pressing Enter calls `CREATE_FILE_DIRECTORY(createName)`.

**Freya test required**: create-in-folder must prove both the correct physical parent directory and visible placement after creation.

## TFR-FOLDER-003 — Folder rename input commits on Enter

When rename mode targets the folder:

- the rename input receives focus;
- it is initialized with the current folder name;
- Enter calls `RENAME_IN_SIDEBAR` only when the new name is non-empty.

**Freya test required**: rename folder, verify the old filesystem path is absent, new path exists, children remain present and navigation points to the new path.

## TFR-FOLDER-004 — Right click selects the logical target before context actions

The folder row intercepts `contextmenu`, prevents the browser default, calls `CHANGE_ACTIVE_ITEM(folder)`, then opens the sidebar context menu.

**Migration invariant**: Rename/Delete/Copy/etc. initiated from a context menu must affect the row that was right-clicked, not a stale previous selection.

---

# 3. File rename and context targeting

Reference: `components/sideBar/treeFile.vue`.

## TFR-RENAME-001 — Rename editor replaces the filename label

When `renameCache === file.pathname`, the visible label is replaced by a text input.

When the input is focused it is initialized to the existing filename.

Enter commits via `RENAME_IN_SIDEBAR(newName)` only when `newName` is non-empty.

Clicking inside the rename input stops the normal file click action.

**Migration invariant**: editing the name must never simultaneously open/reopen the note because the text field click bubbled to the card/tree activation handler.

## TFR-CONTEXT-001 — File context action targets clicked file

On right click Tauri:

1. prevents the default context menu;
2. changes `activeItem` to that file;
3. opens the application sidebar context menu with clipboard state.

**Freya test required**: right-click B while A was previously active, invoke a harmless B-specific action (or rename in fixture) and prove A is unchanged.

---

# 4. Editor find / replace

Reference: `components/search/index.vue`.

## TFR-SEARCH-001 — Search UI has explicit open state

The search bar is visible only while `showSearch` is true. Merely retaining an old query does not imply the panel is open.

Changing `searchValue` triggers the debounced search only if the search UI is currently open.

## TFR-SEARCH-002 — Search result position is observable

The UI displays `highlightIndex + 1 / highlightCount` and provides previous and next controls.

Acceptance needs to prove not just result count but that next/previous changes the active match in the editor.

## TFR-SEARCH-003 — Search modifiers are independent toggles

Tauri exposes three independent controls:

- case sensitive;
- whole word;
- regular expression.

Freya parity requires functional cases where each toggle changes the result set, plus combined modifiers.

## TFR-SEARCH-004 — Search and replace are distinct modes

The left control toggles search type. In replace mode a replacement input and two actions are visible:

- replace all;
- replace single/current.

A parity test must prove text mutation in the actual document model and saved file, not only changed UI text.

## TFR-SEARCH-005 — Invalid search input has a visible error path

The reference has `searchErrorMsg`, an error class on the input wrapper and a visible error message element. Invalid regex behaviour must therefore be non-crashing and observable.

---

# 5. Title bar and document status

Reference: `components/titleBar/index.vue`.

## TFR-TITLE-001 — Application/document title

- without a filename, the visible title is `Elephant`;
- with a pathname, up to the last three parent path segments are shown before the filename;
- the window/document title is derived from filename and project name.

## TFR-TITLE-002 — Dirty state is visible

The `save-dot` is shown when `isSaved` is false.

**Freya parity requirement**: a user edit must cause an observable dirty/saving state if Freya exposes this contract, and successful persistence must clear it. Functional tests must additionally read the physical Markdown file.

## TFR-TITLE-003 — Filename action triggers rename

The filename in the title bar has its own click action (`rename`). This interaction must not be confused with generic window dragging or card activation.

## TFR-TITLE-004 — Word counter cycles modes

Clicking the word-count control cycles, in order:

1. word;
2. paragraph;
3. character;
4. all / characters including space;
5. back to word.

The tooltip exposes word, character and paragraph totals.

## TFR-WINDOW-001 — Platform-specific title-bar behaviour

- double-clicking the title region toggles maximize only on macOS;
- with the custom title bar on non-macOS, explicit close/maximize-or-restore/minimize controls are rendered;
- maximize action exits fullscreen first, otherwise restores a maximized window, otherwise maximizes it.

These behaviours require platform/runtime tests and must not be marked proven by headless widget tests alone.

---

# 6. Functional surfaces discovered but not yet fully specified

The following reference areas exist and must be inspected before parity can be declared:

- About;
- Command palette;
- Editor with tabs;
- Export settings;
- Import;
- Loading/startup states;
- Recent notes;
- Rename dialog/workflow outside the sidebar inline path;
- Sidebar search and search result activation;
- Table of contents;
- Tweet/embedded content;
- Context menu command set;
- global commands/keyboard shortcuts;
- editor Rust integration;
- addons;
- settings/preferences;
- vault opening/switching;
- drag-and-drop;
- images/assets;
- drawings/Excalidraw;
- executable code blocks;
- Wiki/Graph/Knowledge/Open Models;
- synchronization and conflict handling.

Discovery is **not** proof. Each area must receive contracts like the sections above plus a test mapping.

---

# 7. Navigation/history acceptance matrix for Freya

These are app-level behavioural requirements, even where the historical Tauri UI expresses navigation differently.

| ID | Scenario | Required observable result |
|---|---|---|
| FH-001 | Root -> folder | Folder content replaces root library content; no editor is mounted. |
| FH-002 | Folder -> note immediately | Note opens on the first click, even if the pointer coordinates match the prior folder click. |
| FH-003 | Note -> Back | Exact parent library state is restored and editor disappears. |
| FH-004 | Back -> Forward | Exact note is reopened; forward becomes unavailable at history tip. |
| FH-005 | Back -> new navigation | Stale forward branch is discarded. |
| FH-006 | Multiple Back operations | States are restored in LIFO history order without duplicate phantom entries. |
| FH-007 | Close note | Closing does not delete or truncate the physical Markdown file. |
| FH-008 | Reopen note | Real persisted content is loaded, not a stale in-memory copy. |

No timing sleep is part of these contracts. A test that passes only after sleeping past a double-click threshold does not prove FH-002.

---

# 8. Required test design conventions

Every new Freya functional test should state which contract ID it proves.

For vault operations, fixtures should use a temporary physical directory and tests should assert filesystem effects directly.

For editor operations, prefer three-way proof where applicable:

1. visible editor state;
2. editor content/state after the user action;
3. physical file content after persistence.

For navigation, assert both the positive destination and the absence of the previous incompatible state. Example: after Back from an editor, assert the folder card exists **and** `NoteEditorHost` is absent.

For destructive operations, use isolated fixtures and prove unrelated sibling files remain byte-for-byte unchanged.

For race-prone interactions, do not introduce sleeps as the acceptance condition. Exercise the sequence at normal event-loop speed and synchronize only through Freya's normal test render/update primitive.

---

# 9. Current migration blocker captured by this specification

At the time this document was introduced, `LibraryCard` in Freya filters its body `on_mouse_up` with:

`EventsCombos::pressed(event.global_location).is_double()`

That means the semantic identity of the clicked entry is not part of double-click classification. After navigating into a folder, a newly mounted note can occupy the same coordinate, so the second logical click may be discarded.

This conflicts with TFR-NAV-001 and FH-002. The title-specific rename/double-click behaviour, if retained by Freya, must be scoped to the title target itself; it must not suppress activation of a different card.

The existing `navigation_back_forward_freya_testing.rs` intentionally performs the folder->note interaction without an artificial delay so this remains a real functional gate.

---

# 10. Definition of parity

A section is **PARITY-PROVEN** only when:

- its Tauri behaviour has been inspected and written as explicit contracts;
- Freya implements every non-superseded contract;
- automated functional tests cover normal, boundary and failure paths;
- filesystem/runtime side effects are verified where relevant;
- no test relies on artificial delays to evade interaction bugs;
- CI executes those tests and is green on the branch being evaluated.

Until all of those are true, the section should be reported as `specified`, `implemented-unproven`, `partially-proven`, or `failing`, never simply `done`.
