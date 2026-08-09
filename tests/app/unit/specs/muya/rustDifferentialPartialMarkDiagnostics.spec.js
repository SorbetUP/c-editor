import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelection
} from './rustDifferentialHarness'

const fragmentEdges = (rust) => rust
  .snapshot()
  .document.nodes
  .filter((node) => node.kind?.value?.type === 'mark_fragment')
  .map((node) => node.kind.value.edge)

const describeBundled = bundled ? describe : describe.skip
const traces = [
  {
    name: 'strike from inside strong through inside emphasis',
    initial: '**alpha** beta *gamma*',
    expected: '**al~~pha** beta *gam~~ma*\n',
    edges: ['start', 'middle', 'end'],
    runJs: async (muya) => {
      setJsSelection(muya, 0, 4, 19)
      muya.format('del')
    },
    runRust: (rust) => {
      rust.setSelectionBetweenText('alpha', 2, 'gamma', 3)
      rust.request({ type: 'toggle_strike' })
      return fragmentEdges(rust)
    }
  },
  {
    name: 'emphasis from plain text through inside strong',
    initial: 'alpha **beta** gamma',
    expected: 'al*pha **be*ta** gamma\n',
    edges: ['start', 'end'],
    runJs: async (muya) => {
      setJsSelection(muya, 0, 2, 10)
      muya.format('em')
    },
    runRust: (rust) => {
      rust.setSelectionBetweenText('alpha ', 2, 'beta', 2)
      rust.request({ type: 'toggle_emphasis' })
      return fragmentEdges(rust)
    }
  },
  {
    name: 'reopen a strike crossing strong and emphasis wrappers',
    initial: '**al~~pha** beta *gam~~ma*',
    expected: '**al~~pha** beta *gam~~ma*\n',
    edges: ['start', 'middle', 'end'],
    runJs: async () => {},
    runRust: (rust) => fragmentEdges(rust)
  },
  {
    name: 'reopen emphasis crossing plain text and strong',
    initial: 'al*pha **be*ta** gamma',
    expected: 'al*pha **be*ta** gamma\n',
    edges: ['start', 'end'],
    runJs: async () => {},
    runRust: (rust) => fragmentEdges(rust)
  },
  {
    name: 'keep a linked strike group on repeated toggle',
    initial: '**alpha** beta *gamma*',
    expected: '**al~~pha** beta *gam~~ma*\n',
    edges: ['start', 'middle', 'end'],
    runJs: async (muya) => {
      setJsSelection(muya, 0, 4, 19)
      muya.format('del')
      muya.format('del')
    },
    runRust: (rust) => {
      rust.setSelectionBetweenText('alpha', 2, 'gamma', 3)
      rust.request({ type: 'toggle_strike' })
      rust.request({ type: 'toggle_strike' })
      return fragmentEdges(rust)
    }
  },
  {
    name: 'toggle a linked emphasis group off',
    initial: 'alpha **beta** gamma',
    expected: 'alpha **beta** gamma\n',
    edges: [],
    runJs: async (muya) => {
      setJsSelection(muya, 0, 2, 10)
      muya.format('em')
      muya.format('em')
    },
    runRust: (rust) => {
      rust.setSelectionBetweenText('alpha ', 2, 'beta', 2)
      rust.request({ type: 'toggle_emphasis' })
      rust.request({ type: 'toggle_emphasis' })
      return fragmentEdges(rust)
    }
  },
  {
    name: 'strike across text nodes inside one strong subtree',
    initial: '**alpha *beta* gamma**',
    expected: '**al~~pha *beta* gam~~ma**\n',
    edges: ['start', 'middle', 'end'],
    runJs: async (muya) => {
      setJsSelection(muya, 0, 4, 18)
      muya.format('del')
    },
    runRust: (rust) => {
      rust.setSelectionBetweenText('alpha ', 2, ' gamma', 4)
      rust.request({ type: 'toggle_strike' })
      return fragmentEdges(rust)
    }
  }
]

describeBundled('Muya partial cross-wrapper mark differential traces', () => {
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
      expect(result.rustResult).toEqual(trace.edges)
    })
  }
})