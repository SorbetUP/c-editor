import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelectionByText
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip

const clipboardEvent = (text) => ({
  preventDefault: () => {},
  stopPropagation: () => {},
  clipboardData: {
    getData: (type) => (type === 'text/plain' ? text : '')
  }
})

const cases = [
  {
    name: 'paste plain text inside a list item',
    initial: '- alpha',
    target: 'alpha',
    offset: 2,
    pasted: 'XYZ',
    expected: '- alXYZpha\n'
  },
  {
    name: 'paste multiline Markdown inside a list item',
    initial: '- alpha',
    target: 'alpha',
    offset: 2,
    pasted: 'one\n\ntwo',
    expected: '- alone\n  \n  twopha\n'
  },
  {
    name: 'paste plain text inside a table cell',
    initial: '| A | B |\n| --- | --- |\n| alpha | beta |',
    target: 'alpha',
    offset: 2,
    pasted: 'XYZ',
    expected: '| A        | B    |\n| -------- | ---- |\n| alXYZpha | beta |\n'
  },
  {
    name: 'paste multiline Markdown inside a table cell',
    initial: '| A | B |\n| --- | --- |\n| alpha | beta |',
    target: 'alpha',
    offset: 2,
    pasted: 'one\n\ntwo',
    expected: '| A                     | B    |\n| --------------------- | ---- |\n| alone<br/><br/>twopha | beta |\n'
  },
  {
    name: 'paste plain text inside a blockquote',
    initial: '> alpha',
    target: 'alpha',
    offset: 2,
    pasted: 'XYZ',
    expected: '> alXYZpha\n'
  },
  {
    name: 'paste multiline Markdown inside a blockquote',
    initial: '> alpha',
    target: 'alpha',
    offset: 2,
    pasted: 'one\n\ntwo',
    expected: '> alone\n> \n> twopha\n'
  }
]

describeBundled('Muya nested paste differential traces', () => {
  let jsEditor = null

  beforeAll(initializeRustWasm)

  afterEach(() => {
    jsEditor?.destroy?.()
    jsEditor = null
    document.body.innerHTML = ''
  })

  for (const testCase of cases) {
    it(testCase.name, async () => {
      const result = await runDifferentialTrace({
        initial: testCase.initial,
        runJs: async (muya) => {
          setJsSelectionByText(muya, testCase.target, testCase.offset)
          await muya.contentState.pasteHandler(
            clipboardEvent(testCase.pasted),
            'normal',
            testCase.pasted
          )
        },
        runRust: (rust) => {
          rust.setSelectionByText(testCase.target, testCase.offset)
          rust.request({ type: 'paste_markdown', markdown: testCase.pasted })
        }
      })
      jsEditor = result.jsEditor
      expect(result.jsMarkdown).toBe(result.rustMarkdown)
      expect(result.rustMarkdown).toBe(testCase.expected)
    })
  }
})
