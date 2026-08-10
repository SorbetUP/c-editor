import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')

describe('library UI regression contracts', () => {
  it('renders separate DOM branches for grid and list layouts', () => {
    const grid = read('Elephant/frontend/app/components/library/LibraryGrid.vue')

    expect(grid).toContain('v-if="store.viewMode === \'grid\'"')
    expect(grid).toContain('en-library-grid-surface--grid')
    expect(grid).toContain('data-layout="grid"')
    expect(grid).toContain('v-else')
    expect(grid).toContain('en-library-grid-surface--list')
    expect(grid).toContain('data-layout="list"')
  })

  it('keeps sort and view controls as accessible single-button cycles', () => {
    const toolbar = read('Elephant/frontend/app/components/library/LibraryToolbar.vue')
    const grid = read('Elephant/frontend/app/components/library/LibraryGrid.vue')

    for (const sort of ['updated-newest', 'updated-oldest', 'title-az', 'title-za']) {
      expect(toolbar).toContain(`value: '${sort}'`)
    }
    expect(toolbar).toContain(':aria-label="`Sort: ${sortOption.label}`"')
    expect(toolbar).toContain(':aria-label="viewModeLabel"')
    expect(toolbar).toContain('@click="cycleSort"')
    expect(toolbar).toContain('@click="cycleView"')
    expect(toolbar).toContain(':mobile="true"')
    expect(grid).toContain("if (kind === 'drawing')")
    expect(grid).toContain("bus.emit('open-excalidraw-from-image', source)")
    expect(grid).toContain('const isExcalidrawPath')
    expect(grid).toContain('window.path.join(store.activeVault.path, entry.path)')
  })

  it('does not reintroduce card dates and isolates folder actions from opening', () => {
    const noteCard = read('Elephant/frontend/app/components/library/NoteCard.vue')
    const folderCard = read('Elephant/frontend/app/components/library/FolderCard.vue')

    for (const card of [noteCard, folderCard]) {
      expect(card).not.toContain('en-updated')
      expect(card).not.toContain('getNoteCardUpdatedLabel')
      expect(card).toContain('@click.stop.prevent="toggleSidebarVisibility"')
    }
    expect(noteCard).toContain('data-entry-action="sidebar"')
    expect(noteCard).toContain('data-entry-rename-input')
    expect(noteCard).toContain("const drawingPreviewSrc = ref('')")
    expect(noteCard).toContain('window.fileUtils?.readFile')
    expect(noteCard).toContain('URL.createObjectURL(blob)')
    expect(folderCard).toContain('data-entry-rename-input')
    expect(folderCard).toContain('aria-label="Delete folder"')
  })

  it('keeps compact card and floating-toolbar layout contracts', () => {
    const noteCard = read('Elephant/frontend/app/components/library/NoteCard.vue')
    const folderCard = read('Elephant/frontend/app/components/library/FolderCard.vue')
    const toolbar = read('Elephant/frontend/app/components/library/LibraryToolbar.vue')
    const runtimeFixes = read('Elephant/frontend/app/styles/runtime-layout-fixes.css')

    expect(noteCard).toContain('min-height: 176px;')
    expect(noteCard).toContain('min-height: 58px;')
    expect(noteCard).toContain('.en-library-grid.list .en-note-card')
    expect(folderCard).toContain('min-height: 176px;')
    expect(folderCard).toContain('childrenPreview')
    expect(folderCard).toContain('Folder contents preview')
    expect(toolbar).toContain('position: absolute;')
    expect(toolbar).toContain('box-shadow: 0 8px 22px')
    expect(toolbar).toContain('pointer-events: auto;')
    expect(runtimeFixes).toContain('.en-library-grid-surface--grid')
    expect(runtimeFixes).toContain('.en-library-grid-surface--list')
  })
})
