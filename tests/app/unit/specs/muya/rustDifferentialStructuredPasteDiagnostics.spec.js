import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelection
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
    name: 'paste a heading into paragraph text',
    markdown: '# Title',
    expected: 'al# Titlepha\n'
  },
  {
    name: 'paste a blockquote into paragraph text',
    markdown: '> quote',
    expected: 'al\n\n> quotepha\n'
  },
  {
    name: 'paste a list into paragraph text',
    markdown: '- one\n- two',
    expected: 'al\n\n- one\n- twopha\n'
  },
  {
    name: 'paste a fenced code block into paragraph text',
    markdown: '```js\nconsole.log(1)\n```',
    expected: 'al\n\n```js\nconsole.log(1)pha\n```\n'
  },
  {
    name: 'paste a table into paragraph text',
    markdown: '| A | B |\n| --- | --- |\n| one | two |',
    expected: 'al\n\n| A   | B      |\n| --- | ------ |\n| one | twopha |\n'
  },
  {
    name: 'paste a Markdown image into paragraph text',
    markdown: '![alt](image.png)',
    expected: 'al![alt](image.png)pha\n'
  }
]

describeBundled('Muya structured paste differential traces', () => {
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
        initial: 'alpha',
        runJs: async (muya) => {
          setJsSelection(muya, 0, 2, 2)
          await muya.contentState.pasteHandler(
            clipboardEvent(trace.markdown),
            'normal',
            trace.markdown
          )
        },
        runRust: (rust) => {
          rust.setSelection(0, 2, 2)
          rust.request({ type: 'paste_markdown', markdown: trace.markdown })
        }
      })
      jsEditor = result.jsEditor
      expect(result.jsMarkdown).toBe(result.rustMarkdown)
      expect(result.rustMarkdown).toBe(trace.expected)
    })
  }
})
