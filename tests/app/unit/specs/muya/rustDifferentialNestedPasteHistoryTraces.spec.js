import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelectionByText
} from './rustDifferentialHarness'

const clipboardEvent = (text) => ({
  preventDefault: () => {},
  stopPropagation: () => {},
  clipboardData: {
    getData: (type) => (type === 'text/plain' ? text : '')
  }
})

const describeBundled = bundled ? describe : describe.skip
const traces = [
  {
    name: 'undo and redo multiline paste inside a list item',
    initial: '- alpha',
    target: 'alpha',
    pasted: 'one\n\ntwo',
    expected: '- alone\n  \n  twopha\n',
    checkpoints: [
      '- alone\n  \n  twopha\n',
      '- alpha\n',
      '- alone\n  \n  twopha\n'
    ]
  },
  {
    name: 'undo and redo multiline paste inside a table cell',
    initial: '| A | B |\n| --- | --- |\n| alpha | beta |',
    target: 'alpha',
    pasted: 'one\n\ntwo',
    expected: '| A                     | B    |\n| --------------------- | ---- |\n| alone<br/><br/>twopha | beta |\n',
    checkpoints: [
      '| A                     | B    |\n| --------------------- | ---- |\n| alone<br/><br/>twopha | beta |\n',
      '| A     | B    |\n| ----- | ---- |\n| alpha | beta |\n',
      '| A                     | B    |\n| --------------------- | ---- |\n| alone<br/><br/>twopha | beta |\n'
    ]
  },
  {
    name: 'undo and redo multiline paste inside a blockquote',
    initial: '> alpha',
    target: 'alpha',
    pasted: 'one\n\ntwo',
    expected: '> alone\n> \n> twopha\n',
    checkpoints: [
      '> alone\n> \n> twopha\n',
      '> alpha\n',
      '> alone\n> \n> twopha\n'
    ]
  }
]

describeBundled('Muya nested paste history differential traces', () => {
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
          setJsSelectionByText(muya, trace.target, 2)
          await muya.contentState.pasteHandler(
            clipboardEvent(trace.pasted),
            'normal',
            trace.pasted
          )
          const pasted = muya.getMarkdown()
          muya.undo()
          const undone = muya.getMarkdown()
          muya.redo()
          return [pasted, undone, muya.getMarkdown()]
        },
        runRust: (rust) => {
          rust.setSelectionByText(trace.target, 2)
          rust.request({ type: 'paste_markdown', markdown: trace.pasted })
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
      expect(result.jsResult).toEqual(result.rustResult)
      expect(result.rustResult).toEqual(trace.checkpoints)
    })
  }
})
