import { describe, expect, it } from 'vitest'
import { JSDOM } from 'jsdom'

import installBootstrapGlobals from '../../../Elephant/frontend/src/renderer/src/platform/bootstrapGlobals.js'

describe('global Rust editor compatibility bridge', () => {
  it('installs the Rust runtime bridge during bootstrap globals', () => {
    const dom = new JSDOM('<div id="editor"></div>')
    const target = {
      document: dom.window.document,
      getSelection: dom.window.getSelection.bind(dom.window)
    }
    installBootstrapGlobals(target)
    expect(target.__ELEPHANT_MUYA_RUNTIME__).toBeTruthy()
    expect(target.__ELEPHANT_MUYA_RUNTIME__.mode()).toBe('rust')
    expect(target.__ELEPHANT_MUYA_RUNTIME__.setMode('shadow')).toBe('rust')
    expect(target.__ELEPHANT_MUYA_RUNTIME__.enabled()).toBe(true)
    expect(target.__ELEPHANT_MUYA_RUNTIME__.active()).toBe(true)
  })

  it('can create a runtime editor from the global bridge', () => {
    const dom = new JSDOM('<div id="editor"></div>')
    const target = {
      document: dom.window.document,
      getSelection: dom.window.getSelection.bind(dom.window)
    }
    installBootstrapGlobals(target)
    const root = dom.window.document.getElementById('editor')
    const runtime = target.__ELEPHANT_MUYA_RUNTIME__.createEditor(root, '# Global')
    expect(root.getAttribute('data-muya-editor')).toBe('true')
    expect(runtime.markdown).toContain('# Global')
  })
})
