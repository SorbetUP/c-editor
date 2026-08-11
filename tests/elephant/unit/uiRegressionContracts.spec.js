import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')

describe('ElephantNote UI regression contracts', () => {
  it('keeps the library create action compact and floating', () => {
    const toolbar = read('Elephant/frontend/app/components/library/LibraryToolbar.vue')
    const menu = read('Elephant/frontend/app/components/library/CreateEntryMenu.vue')

    expect(toolbar).toContain(':mobile="true"')
    expect(toolbar).toContain('en-sort-cycle')
    expect(toolbar).toContain('en-view-cycle')
    expect(toolbar).toContain('title-za')
    expect(menu).toContain('data-testid="excalidraw-logo"')
    expect(menu).toContain('data-excalidraw-asset="shared-muya-icon"')
  })

  it('renames entries inline and exposes icon-only folder actions', () => {
    const card = read('Elephant/frontend/app/components/library/NoteCard.vue')
    const grid = read('Elephant/frontend/app/components/library/LibraryGrid.vue')

    expect(card).toContain('data-entry-rename-input')
    expect(card).toContain('@keydown.enter.stop.prevent="commitRename"')
    expect(card).toContain('@keydown.esc.stop.prevent="cancelRename"')
    expect(card).toContain('data-entry-action="delete"')
    expect(card).toContain('data-entry-action="rename"')
    expect(card).toContain('data-entry-action="sidebar"')
    expect(grid).not.toContain('en-library-rename-form')
  })

  it('keeps list layout rules on cards and removes the editor exit border', () => {
    const card = read('Elephant/frontend/app/components/library/NoteCard.vue')
    const shellStyles = read('Elephant/frontend/app/styles/app-shell.css')

    expect(card).toContain('.en-library-grid.list .en-note-card')
    expect(card).not.toContain(':global(.en-library-grid.list)')
    expect(shellStyles).toContain('border-top: 0;')
  })

  it('uses narrow editor gutters and does not center pointer selections', () => {
    const host = read('Elephant/frontend/app/components/editor/NoteEditorHost.vue')
    const runtime = read(
      'Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditor.vue'
    )

    expect(runtime).toContain('data-testid="muya-runtime-editor"')
    expect(host).toContain('Math.min(48, Math.round(value))')
    expect(host).toContain("'--en-note-editor-gutter-left': `${editorMarginPx.value}px`")
    expect(runtime).toContain('pointerSelection')
    expect(runtime).toContain('selectionChange')
  })

  it('styles Rust task checkboxes and keeps Muya image resizing persistent', () => {
    const elements = read('Elephant/frontend/src/renderer/src/editor-rust/domRenderer/elements.js')
    const stylesheet = read('Elephant/frontend/app/styles/app-shell.css')
    const runtime = read(
      'Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditor.vue'
    )
    const transformer = read('Elephant/frontend/src/muya/lib/ui/transformer/index.js')

    expect(elements).toContain('data-muya-rust-task-checkbox')
    expect(stylesheet).toContain('[data-muya-rust-task-checkbox]')
    expect(stylesheet).toContain('input.ag-task-list-item-checkbox')
    expect(stylesheet).toContain('appearance: none')
    expect(stylesheet).toContain('input.ag-task-list-item-checkbox::before')
    expect(stylesheet).toContain('[data-muya-rust-task-checkbox]::before')
    expect(stylesheet).toContain('content: none !important')
    expect(stylesheet).toContain('max-width: 100%')
    expect(transformer).toContain("updateImage(this.imageInfo, 'width', this.width)")
    expect(runtime).toContain('input.ag-task-list-item-checkbox')
    expect(runtime).toContain('dispatchChange()')
  })

  it('guards empty editor padding and installs selection listeners once', () => {
    const runtime = read(
      'Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditor.vue'
    )
    const selectionRuntime = read('Elephant/frontend/src/renderer/src/muya/selectionRuntime.js')

    expect(runtime).toContain('clearPaddingOnlySelection(window.getSelection?.(), container)')
    expect(runtime).toContain("document.addEventListener('selectionchange', clearPaddingSelection)")
    expect(runtime).toContain("document.removeEventListener('selectionchange', clearPaddingSelection)")
    const scrollToCordsBody = runtime.slice(runtime.indexOf('const scrollToCords ='), runtime.indexOf('const scrollToHighlight ='))
    expect(scrollToCordsBody).not.toContain('addEventListener')
    expect(selectionRuntime).toContain('range.commonAncestorContainer === editorRoot')
  })

  it('does not emit requests for fonts that are absent from the repository', () => {
    const theme = read('Elephant/frontend/src/muya/themes/default.css')
    expect(theme).toContain("local('Open Sans Regular')")
    expect(theme).toContain("local('DejaVu Sans Mono')")
    expect(theme).not.toContain("url('./fonts/open-sans")
    expect(theme).not.toContain("url('./fonts/DejaVuSans")
  })

  it('keeps citation controls icon-only and retains a pasteable buffer', () => {
    const runtime = read('Elephant/frontend/src/renderer/src/platform/noteCitationRuntime.js')
    expect(runtime).toContain('data-elephant-citation-selection-action')
    expect(runtime).not.toContain('data-elephant-note-citation')
    expect(runtime).toContain('data-lucide="quote"')
    expect(runtime).toContain('data-elephant-citation-buffer-item')
    expect(runtime).toContain('appendCitationToCurrentNote')
    expect(runtime).not.toContain("selectionButton.textContent = 'Citer'")
  })

  it('supports entry drops into notes and app-owned folder links', () => {
    const host = read('Elephant/frontend/app/components/editor/NoteEditorHost.vue')
    const runtime = read('Elephant/frontend/src/renderer/src/platform/noteCitationRuntime.js')

    expect(host).toContain('data-entry-drop-target="note-editor"')
    expect(host).toContain('elephant://entry/')
    expect(host).toContain('droppedKind')
    expect(runtime).toContain('resolveInternalEntryLink')
    expect(runtime).toContain('store.openDirectory(entryLink.path)')
  })

  it('labels the sidebar section as notes', () => {
    const sidebar = read('Elephant/frontend/app/components/navigation/SidebarNav.vue')
    expect(sidebar).toContain('<span class="en-tags-label">Notes</span>')
    expect(sidebar).toContain('aria-label="Search notes"')
  })

  it('routes Cmd/Ctrl+F to the active editor or global search', () => {
    const shell = read('Elephant/frontend/app/components/shell/AppShell.vue')
    const bridge = read('Elephant/frontend/src/renderer/src/platform/runtimeBridge.js')
    const acceptanceBridge = read(
      'Elephant/frontend/src/renderer/src/platform/acceptanceTestBridge.js'
    )
    const acceptance = read('build/scripts/run-desktop-acceptance.mjs')

    expect(shell).toContain("key.toLowerCase() === 'f'")
    expect(shell).toContain("bus.emit('find', 'find')")
    expect(shell).toContain('openSearch()')
    expect(bridge).toContain("'edit.find': 'CmdOrCtrl+F'")
    expect(acceptanceBridge).toContain('metaKey: !!modifiers.metaKey')
    expect(acceptance).toContain("waitForVisibleDom('.search-bar'")
  })

  it('keeps the vault tooltip safe for the Tauri acceptance transport', () => {
    const rail = read('Elephant/frontend/app/components/navigation/IconRail.vue')
    expect(rail).toContain('open vault switcher')
    expect(rail).not.toContain(' — open vault switcher')
  })

  it('uses the non-deprecated Element Plus dialog header slot', () => {
    const editor = read(
      'Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditor.vue'
    )
    expect(editor).toContain('<template #header>')
    expect(editor).not.toContain('<template #title>')
  })

  it('keeps structural workspace surfaces flat and leaves elevation to local controls', () => {
    const shell = read('Elephant/frontend/app/components/shell/AppShell.vue')
    const settings = read('Elephant/frontend/app/components/settings/SettingsPanel.vue')
    const stylesheet = read('Elephant/frontend/app/styles/app-shell.css')
    const runtimeStyles = read('Elephant/frontend/app/styles/app-shell-runtime-fixes.css')

    expect(shell).not.toContain('en-floating-surfaces')
    expect(settings).not.toContain('Floating surfaces')
    expect(stylesheet).not.toContain('en-floating-surfaces')
    expect(runtimeStyles).not.toContain('en-floating-surfaces')
    expect(stylesheet).toContain('.en-note-editor-shell')
    expect(stylesheet).toContain('background: var(--en-bg);')
  })

  it('persists appearance preferences through the Tauri preference API', () => {
    const preferences = read('Elephant/frontend/src/renderer/src/store/preferences.js')

    expect(preferences).toContain("invoke('tauri_prefs_all')")
    expect(preferences).toContain("invoke('tauri_prefs_set', { key: type, value })")
    expect(preferences).toContain('persistent load failed')
    expect(preferences).toContain('persistent write failed')
  })

  it('keeps the editor writing surface full width with a small gutter fallback', () => {
    const theme = read('Elephant/frontend/src/muya/themes/default.css')
    const topbar = read('Elephant/frontend/app/components/editor/NoteEditorTopBar.vue')

    expect(theme).toContain('width: 100%;')
    expect(theme).toContain('max-width: none;')
    expect(theme).toContain('var(--en-note-editor-gutter, 12px)')
    expect(topbar).toContain('min-height: 52px')
    expect(topbar).toContain('min-height: 36px')
    expect(topbar).toContain('var(--en-note-editor-gutter, 12px)')
  })

  it('opens the addon catalogue without a title checkbox gate', () => {
    const addons = read('Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue')
    const composable = read('Elephant/frontend/app/components/settings/useAddonsSettings.js')

    expect(addons).not.toContain('en-community-title-check')
    expect(addons).not.toContain('role="checkbox"')
    expect(addons).toContain('Install addon from file')
    expect(composable).toContain('CATALOG_TIMEOUT_MS')
    expect(composable).toContain('await nextTick()')
    expect(composable).toContain('void refreshCatalog()')
  })

  it('keeps Electron E2E runs hidden unless visual debugging is explicit', () => {
    const electronMain = read('tests/app/e2e/electron-main.js')
    expect(electronMain).toContain("process.env.ELEPHANT_E2E_SHOW_WINDOW === '1'")
    expect(electronMain).toContain("process.env.ELEPHANT_E2E_HIDE_WINDOW !== '1'")
    expect(electronMain).toContain('show: showTestWindow')
  })

  it('keeps the real Tauri acceptance window hidden unless visual debugging is explicit', () => {
    const tauriEntry = read('Elephant/backend/tauri/src/lib_min.rs')
    const acceptance = read('build/scripts/run-desktop-acceptance.mjs')
    expect(tauriEntry).toContain('ELEPHANT_ACCEPTANCE_HIDE_WINDOW')
    expect(acceptance).toContain("process.env.ELEPHANT_ACCEPTANCE_SHOW_WINDOW === '1'")
    expect(acceptance).toContain(
      "ELEPHANT_ACCEPTANCE_HIDE_WINDOW: showAcceptanceWindow ? '0' : '1'"
    )
  })

  it('keeps the mobile drawer toggle reversible and the sidebar toggle icon neutral until hover', () => {
    const shell = read('Elephant/frontend/app/components/shell/AppShell.vue')
    const rail = read('Elephant/frontend/app/components/navigation/IconRail.vue')

    expect(shell).toContain('toggleMobileSidebar')
    expect(shell).toContain("drawerProgress > 0 ? 'Close navigation' : 'Open navigation'")
    expect(rail).toContain('en-rail-sidebar-neutral-icon')
    expect(rail).toContain('en-rail-sidebar-direction-icon')
  })

  it('keeps the desktop chrome flat and avoids a duplicate editor border', () => {
    const topbar = read('Elephant/frontend/app/components/shell/TopVaultBar.vue')
    const shellStyles = read('Elephant/frontend/app/styles/app-shell.css')
    const runtimeStyles = read('Elephant/frontend/app/styles/app-shell-runtime-fixes.css')

    expect(topbar).toContain('display: none;')
    expect(shellStyles).toContain('border-top: 0;')
    expect(runtimeStyles).toContain('.en-topstrip')
    expect(runtimeStyles).toContain('box-shadow: none !important;')
  })
})
