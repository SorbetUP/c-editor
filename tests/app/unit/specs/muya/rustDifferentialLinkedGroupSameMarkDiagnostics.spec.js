import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelection
} from './rustDifferentialHarness'

const fragmentEdges = (rust, mark) => rust
  .snapshot()
  .document.nodes
  .filter(
    (node) =>
      node.kind?.value?.type === 'mark_fragment' &&
      node.kind.value.mark === mark
  )
  .map((node) => node.kind.value.edge)

const describeBundled = bundled ? describe : describe.skip
const traces = [
  {
    name: 'toggle emphasis inside the start fragment of linked emphasis',
    initial: 'alpha **beta** gamma',
    expected: 'alpha **beta** gamma\n',
    edges: [],
    runJs: async (muya) => {
      setJsSelection(muya, 0, 2, 10)
      muya.format('em')
      setJsSelection(muya, 0, 4, 6)
      muya.format('em')
    },
    runRust: (rust) => {
      rust.setSelectionBetweenText('alpha ', 2, 'beta', 2)
      rust.request({ type: 'toggle_emphasis' })
      rust.setSelectionByText('pha ', 1, 3)
      rust.request({ type: 'toggle_emphasis' })
      return fragmentEdges(rust, 'emphasis')
    }
  },
  {
    name: 'toggle emphasis across part of linked emphasis',
    initial: 'alpha **beta** gamma',
    expected: 'alp*ha **bet*a** gamma\n',
    edges: ['start', 'middle', 'end'],
    runJs: async (muya) => {
      setJsSelection(muya, 0, 2, 10)
      muya.format('em')
      setJsSelection(muya, 0, 4, 13)
      muya.format('em')
    },
    runRust: (rust) => {
      rust.setSelectionBetweenText('alpha ', 2, 'beta', 2)
      rust.request({ type: 'toggle_emphasis' })
      rust.setSelectionBetweenText('pha ', 1, 'ta', 1)
      rust.request({ type: 'toggle_emphasis' })
      return fragmentEdges(rust, 'emphasis')
    }
  },
  {
    name: 'toggle strike inside the start fragment of linked strike',
    initial: '**alpha** beta *gamma*',
    expected: '**al~~pha** beta *gam~~ma*\n',
    edges: ['start', 'middle', 'end'],
    runJs: async (muya) => {
      setJsSelection(muya, 0, 4, 19)
      muya.format('del')
      setJsSelection(muya, 0, 7, 9)
      muya.format('del')
    },
    runRust: (rust) => {
      rust.setSelectionBetweenText('alpha', 2, 'gamma', 3)
      rust.request({ type: 'toggle_strike' })
      rust.setSelectionByText('pha', 1, 3)
      rust.request({ type: 'toggle_strike' })
      return fragmentEdges(rust, 'strike')
    }
  }
]

describeBundled('Muya same-mark linked-group differential traces', () => {
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
