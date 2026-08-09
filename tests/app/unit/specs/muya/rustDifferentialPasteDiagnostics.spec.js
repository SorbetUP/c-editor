import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelection
} from './rustDifferentialHarness'

const clipboardEvent = (text, html = '') => ({
  preventDefault: () => {},
  stopPropagation: () => {},
  clipboardData: {
    getData: (type) => {
      if (type === 'text/plain') return text
      if (type === 'text/html') return html
      return ''
    }
  }
})

const describeBundled = bundled ? describe : describe.skip
const traces = [
  {
    name: 'paste plain text at a collapsed caret',
    initial: 'alpha',
    expected: 'alXYZpha\n',
    selection: [2, 2],
    text: 'XYZ',
    html: '',
    markdown: 'XYZ'
  },
  {
    name: 'replace a selected range with plain text',
    initial: 'alpha',
    expected: 'aXa\n',
    selection: [1, 4],
    text: 'X',
    html: '',
    markdown: 'X'
  },
  {
    name: 'paste multiline Markdown between existing text',
    initial: 'alpha',
    expected: 'alone\n\ntwopha\n',
    selection: [2, 2],
    text: 'one\n\ntwo',
    html: '',
    markdown: 'one\n\ntwo'
  },
  {
    name: 'paste semantic HTML into a paragraph',
    initial: 'alpha',
    expected: 'al**bold** and *soft*pha\n',
    selection: [2, 2],
    text: 'bold and soft',
    html: '<p><strong>bold</strong> and <em>soft</em></p>',
    markdown: '**bold** and *soft*'
  },
  {
    name: 'undo and redo multiline Markdown paste',
    initial: 'alpha',
    expected: 'alone\n\ntwopha\n',
    selection: [2, 2],
    text: 'one\n\ntwo',
    html: '',
    markdown: 'one\n\ntwo',
    checkpoints: ['alone\n\ntwopha\n', 'alpha\n', 'alone\n\ntwopha\n']
  },
  {
    name: 'undo and redo semantic HTML paste',
    initial: 'alpha',
    expected: 'al**bold** and *soft*pha\n',
    selection: [2, 2],
    text: 'bold and soft',
    html: '<p><strong>bold</strong> and <em>soft</em></p>',
    markdown: '**bold** and *soft*',
    checkpoints: [
      'al**bold** and *soft*pha\n',
      'alpha\n',
      'al**bold** and *soft*pha\n'
    ]
  }
]

describeBundled('Muya paste differential traces', () => {
  let jsEditor = null

  beforeAll(initializeRustWasm)

  afterEach(() => {
    jsEditor?.destroy?.()
    jsEditor = null
    document.body.innerHTML = ''
  })

  for (const trace of traces) {
    it(trace.name, async () => {
      const result = await runDifferentialTrace({
        initial: trace.initial,
        runJs: async (muya) => {
          setJsSelection(muya, 0, trace.selection[0], trace.selection[1])
          await muya.contentState.pasteHandler(
            clipboardEvent(trace.text, trace.html),
            'normal',
            trace.text,
            trace.html || undefined
          )
          if (!trace.checkpoints) return undefined
          const pasted = muya.getMarkdown()
          muya.undo()
          const undone = muya.getMarkdown()
          muya.redo()
          return [pasted, undone, muya.getMarkdown()]
        },
        runRust: (rust) => {
          rust.setSelection(0, trace.selection[0], trace.selection[1])
          rust.request({ type: 'paste_markdown', markdown: trace.markdown })
          if (!trace.checkpoints) return undefined
          const pasted = rust.markdown()
          rust.request({ type: 'undo' })
          const undone = rust.markdown()
          rust.request({ type: 'redo' })
          return [pasted, undone, rust.markdown()]
        }
      })
      jsEditor = result.jsEditor
      expect(result.jsMarkdown).toBe(result.rustMarkdown)
      expect(result.rustMarkdown).toBe(trace.expected)
      if (trace.checkpoints) {
        expect(result.jsResult).toEqual(result.rustResult)
        expect(result.rustResult).toEqual(trace.checkpoints)
      }
    })
  }
})
