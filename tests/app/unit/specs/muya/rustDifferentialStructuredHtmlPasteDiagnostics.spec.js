import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelection
} from './rustDifferentialHarness'

const clipboardEvent = (text, html) => ({
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
    name: 'paste an unordered HTML list',
    text: 'one\ntwo',
    html: '<ul><li>one</li><li>two</li></ul>',
    markdown: '- one\n- two',
    expected: 'al\n\n- one\n- twopha\n'
  },
  {
    name: 'paste an ordered HTML list',
    text: 'one\ntwo',
    html: '<ol><li>one</li><li>two</li></ol>',
    markdown: '1. one\n2. two',
    expected: 'al\n\n1. one\n2. twopha\n'
  },
  {
    name: 'paste an HTML blockquote',
    text: 'quote',
    html: '<blockquote><p>quote</p></blockquote>',
    markdown: '> quote',
    expected: 'al\n\n> quotepha\n'
  },
  {
    name: 'paste an HTML preformatted code block',
    text: 'console.log(1)',
    html: '<pre><code class="language-js">console.log(1)</code></pre>',
    markdown: '```js\nconsole.log(1)\n```',
    expected: 'al\n\n```js\nconsole.log(1)pha\n```\n'
  },
  {
    name: 'paste an HTML table',
    text: 'A\tB\none\ttwo',
    html: '<table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>one</td><td>two</td></tr></tbody></table>',
    markdown: '| A | B |\n| --- | --- |\n| one | two |',
    expected: 'al\n\n| A   | B      |\n| --- | ------ |\n| one | twopha |\n'
  },
  {
    name: 'paste an HTML image',
    text: 'alt',
    html: '<p><img src="image.png" alt="alt"></p>',
    markdown: '![alt](image.png)',
    expected: 'al![alt](image.png)pha\n'
  }
]

describeBundled('Muya structured HTML paste differential traces', () => {
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
            clipboardEvent(trace.text, trace.html),
            'normal',
            trace.text,
            trace.html
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
