import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelection
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip
const traces = [
  {
    name: 'apply emphasis inside one paragraph',
    initial: 'alpha',
    expected: 'a*lph*a\n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 1, 4)
      muya.format('em')
    },
    runRust: (rust) => {
      rust.setSelection(0, 1, 4)
      rust.request({ type: 'toggle_emphasis' })
    }
  },
  {
    name: 'apply strikethrough inside one paragraph',
    initial: 'alpha',
    expected: 'a~~lph~~a\n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 1, 4)
      muya.format('del')
    },
    runRust: (rust) => {
      rust.setSelection(0, 1, 4)
      rust.request({ type: 'toggle_strike' })
    }
  },
  {
    name: 'nest emphasis inside a strong selection',
    initial: '**alpha**',
    expected: '**a*lph*a**\n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 3, 6)
      muya.format('em')
    },
    runRust: (rust) => {
      rust.setSelection(0, 1, 4)
      rust.request({ type: 'toggle_emphasis' })
    }
  },
  {
    name: 'keep fully selected emphasis nested in strong as a no-op',
    initial: '***alpha***',
    expected: '***alpha***\n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 3, 8)
      muya.format('em')
    },
    runRust: (rust) => {
      rust.setSelectionByTextInMark('alpha', 'emphasis', 0, 5)
      rust.request({ type: 'toggle_emphasis' })
    }
  },
  {
    name: 'remove the whole strong mark from a partial inner selection',
    initial: '**alpha**',
    expected: 'alpha\n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 3, 6)
      muya.format('strong')
    },
    runRust: (rust) => {
      rust.setSelection(0, 1, 4)
      rust.request({ type: 'toggle_strong' })
    }
  }
]

describeBundled('Muya mark differential traces', () => {
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
