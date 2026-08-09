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
    name: 'unwrap a fully selected strong mark',
    initial: '**alpha**',
    expected: 'alpha\n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 2, 7)
      muya.format('strong')
    },
    runRust: (rust) => {
      rust.setSelection(0, 0, 5)
      rust.request({ type: 'toggle_strong' })
    }
  },
  {
    name: 'split at the start of a strong mark without an empty wrapper',
    initial: 'before**bold**after',
    expected: 'before\n\n**bold**after\n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 6)
      muya.contentState.enterHandler(fakeKeyEvent())
    },
    runRust: (rust) => {
      rust.setSelection(1, 0)
      rust.request({ type: 'insert_paragraph' })
    }
  },
  {
    name: 'continue ordered numbering when splitting an item',
    initial: '3. alpha',
    expected: '3. al\n4. pha\n',
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
    name: 'preserve a checked task state when splitting in the middle',
    initial: '- [x] alpha',
    expected: '- [x] al\n- [x] pha\n',
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

describeBundled('Muya structural differential traces', () => {
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
