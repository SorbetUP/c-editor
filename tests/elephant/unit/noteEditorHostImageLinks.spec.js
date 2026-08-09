import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

const read = (path) => readFileSync(path, 'utf8')
const editor = read('Elephant/frontend/app/components/editor/NoteEditorHost.vue')
const shell = read('Elephant/frontend/app/components/shell/AppShell.vue')
const addon = read('Elephant/frontend/src/renderer/src/addons/builtin/excalidraw.js')
const overlay = read('Elephant/frontend/src/renderer/src/addons/builtin/ui/ExcalidrawEditorOverlay.vue')
const dialog = read('Elephant/frontend/app/components/editor/ExcalidrawDialog.vue')
const writingBridge = read('Elephant/frontend/src/renderer/src/platform/writingCommandBridge.js')

describe('addon-owned Excalidraw note integration', () => {
  it('does not persist the editor-only Untitled placeholder as note content', () => {
    expect(editor).toContain('isUntitledPlaceholder')
    expect(editor).toContain('[elephantnote:save] skipped empty untitled placeholder')
  })

  it('keeps the base note editor free of Excalidraw UI and commands', () => {
    expect(editor).not.toContain('ExcalidrawDialog')
    expect(editor).not.toContain('isExcalidrawOpen')
    expect(editor).not.toContain('openExcalidrawFromImage')
    expect(editor).not.toContain('getExcalidrawScenePath')
    expect(editor).not.toContain("bus.on('ELEPHANT::open-excalidraw'")
    expect(editor).not.toContain("bus.on('open-excalidraw-from-image'")
  })

  it('keeps ordinary image relocation generic and delegates companion files to addons', () => {
    expect(editor).toContain("from 'elephant-shared/hiddenAssets'")
    expect(editor).not.toContain("from 'elephant-shared/excalidrawAssets'")
    expect(editor).toContain('copyAddonAssetCompanions')
    expect(editor).toContain('extension.copyAssetCompanions')
    expect(editor).not.toContain('copied excalidraw sidecar')
    expect(editor).not.toContain('|excalidraw)')
  })

  it('lets the Excalidraw addon own its overlay, writing command and image action', () => {
    expect(addon).toContain('component: ExcalidrawEditorOverlay')
    expect(addon).toContain("zone: 'shell.right'")
    expect(addon).toContain('writingCommands: [{')
    expect(addon).toContain("id: 'excalidraw'")
    expect(addon).toContain('imageToolbarItems: [{')
    expect(addon).toContain('copyAssetCompanions: copyDrawingSidecar')
    expect(overlay).toContain('<ExcalidrawDialog')
    expect(overlay).toContain('const openFromImage = async')
    expect(overlay).toContain('const save = async')
    expect(overlay).toContain("bus.on('ELEPHANT::open-excalidraw', open)")
    expect(overlay).toContain("bus.on('open-excalidraw-from-image', openFromImage)")
    expect(shell).toContain("entry?.contribution?.zone === 'shell.right'")
    expect(dialog).toContain('data-testid="excalidraw-dialog"')
    expect(dialog).toContain('data-testid="excalidraw-close"')
    expect(dialog).toContain("message: `[excalidraw-dialog] ${event}`")
  })

  it('removes the hardcoded Excalidraw case from the core writing bridge', () => {
    expect(writingBridge).toContain('addonWritingCommands')
    expect(writingBridge).toContain("manager.getContributions('editor.extensions')")
    expect(writingBridge).not.toContain("case 'excalidraw':")
    expect(writingBridge).not.toContain('openExcalidraw')
  })
})
