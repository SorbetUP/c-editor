import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')

describe('mobile integration repair', () => {
  it('loads every recovered mobile layer from the production renderer', () => {
    const main = read('Elephant/frontend/src/renderer/src/main.js')
    expect(main).toContain("import './mobile-shell-layout.css'")
    expect(main).toContain("import './mobile-android.css'")
    expect(main).toContain("import './mobile-native-ux.css'")
    expect(main).toContain("import './mobile-editor-round2.css'")
    expect(main).toContain("import './mobile-library-chrome.css'")
    expect(main).toContain("import './platform/mobileLibraryChromeRuntime'")
  })

  it('keeps the drawer as an overlay instead of shrinking the note grid', () => {
    const shell = read('Elephant/frontend/src/renderer/src/mobile-shell-layout.css')
    expect(shell).toContain('.en-body.en-sidebar-hidden')
    expect(shell).toContain('display: block !important')
    expect(shell).toContain('.en-mobile-shell .en-sidebar')
    expect(shell).toContain('position: fixed')
    expect(shell).toContain('width: min(82vw, 340px)')
    expect(shell).toContain('grid-column: auto !important')
  })

  it('keeps mobile cards and settings controls readable', () => {
    const mobile = read('Elephant/frontend/src/renderer/src/mobile-android.css')
    expect(mobile).toContain('grid-template-columns: repeat(2, minmax(0, 1fr))')
    expect(mobile).toContain('.en-library-grid.list .en-note-card')
    expect(mobile).toContain('width: 48px !important')
    expect(mobile).toContain('height: 28px !important')
    expect(mobile).toContain('.en-addons-tabs')
  })

  it('restores mobile sort and view controls without changing the library store contract', () => {
    const runtime = read('Elephant/frontend/src/renderer/src/platform/mobileLibraryChromeRuntime.js')
    expect(runtime).toContain('.en-view-toggle button[title="List"]')
    expect(runtime).toContain('.en-library-actions .en-select')
    expect(runtime).toContain('en-mobile-library-controls')
    expect(runtime).toContain('en-mobile-sort-sheet-backdrop')
  })
})
