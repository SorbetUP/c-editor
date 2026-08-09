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
    name: 'paste a list inside a list item',
    initial: '- alpha',
    target: 'alpha',
    pasted: '- one\n- two',
    expected: '- alone\n- twopha\n'
  },
  {
    name: 'paste a code block inside a list item',
    initial: '- alpha',
    target: 'alpha',
    pasted: '```js\nconsole.log(1)\n```',
    expected: '- al\n  \n  ```js\n  console.log(1)pha\n  ```\n'
  },
  {
    name: 'paste a list inside a table cell',
    initial: '| A | B |\n| --- | --- |\n| alpha | beta |',
    target: 'alpha',
    pasted: '- one\n- two',
    expected: '| A                    | B    |\n| -------------------- | ---- |\n| al- one<br/>- twopha | beta |\n'
  },
  {
    name: 'paste a code block inside a table cell',
    initial: '| A | B |\n| --- | --- |\n| alpha | beta |',
    target: 'alpha',
    pasted: '```js\nconsole.log(1)\n```',
    expected: '| A                                     | B    |\n| ------------------------------------- | ---- |\n| al```js<br/>console.log(1)<br/>```pha | beta |\n'
  },
  {
    name: 'paste a list inside a blockquote',
    initial: '> alpha',
    target: 'alpha',
    pasted: '- one\n- two',
    expected: '> al\n> \n> - one\n> - twopha\n'
  },
  {
    name: 'paste a heading inside a blockquote',
    initial: '> alpha',
    target: 'alpha',
    pasted: '# Title',
    expected: '> al# Titlepha\n'
  }
]

describeBundled('Muya nested structured paste differential traces', () => {
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
          setJsSelectionByText(muya, testCase.target, 2)
          await muya.contentState.pasteHandler(
            clipboardEvent(testCase.pasted),
            'normal',
            testCase.pasted
          )
        },
        runRust: (rust) => {
          rust.setSelectionByText(testCase.target, 2)
          rust.request({ type: 'paste_markdown', markdown: testCase.pasted })
        }
      })
      jsEditor = result.jsEditor
      expect(result.jsMarkdown).toBe(result.rustMarkdown)
      expect(result.rustMarkdown).toBe(testCase.expected)
    })
  }
})
