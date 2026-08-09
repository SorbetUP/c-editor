import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  fakeKeyEvent,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelection
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip
const traces = [
  {
    name: 'toggle strong inside one paragraph',
    initial: 'alpha',
    expected: 'a**lph**a\n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 1, 4)
      muya.format('strong')
    },
    runRust: (rust) => {
      rust.setSelection(0, 1, 4)
      rust.request({ type: 'toggle_strong' })
    }
  },
  {
    name: 'convert a paragraph to heading level two',
    initial: 'alpha',
    expected: '## alpha\n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 2)
      muya.updateParagraph('heading 2')
    },
    runRust: (rust) => {
      rust.setSelection(0, 2)
      rust.request({ type: 'set_heading', level: 2 })
    }
  },
  {
    name: 'split a plain paragraph at the caret',
    initial: 'alpha',
    expected: 'al\n\npha\n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 2)
      muya.contentState.enterHandler(fakeKeyEvent())
    },
    runRust: (rust) => {
      rust.setSelection(0, 2)
      rust.request({ type: 'insert_paragraph' })
    }
  },
  {
    name: 'split an unordered list item at the caret',
    initial: '- alpha',
    expected: '- al\n- pha\n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 2)
      muya.contentState.enterHandler(fakeKeyEvent())
    },
    runRust: (rust) => {
      rust.setSelection(0, 2)
      rust.request({ type: 'insert_paragraph' })
    }
  }
]

describeBundled('Muya JavaScript and Rust differential traces', () => {
  let jsEditor = null

  beforeAll(initializeRustWasm)

  afterEach(() => {
    jsEditor?.destroy?.()
    jsEditor = null
    document.body.innerHTML = ''
  })

  for (const trace of traces) {
    it(trace.name, async () => {
      const result = await runDifferentialTrace(trace)
      jsEditor = result.jsEditor
      expect(result.jsMarkdown).toBe(result.rustMarkdown)
      expect(result.rustMarkdown).toBe(trace.expected)
    })
  }
})
