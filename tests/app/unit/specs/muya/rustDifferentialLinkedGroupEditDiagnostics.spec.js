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
    name: 'apply strike across part of an existing linked emphasis group',
    initial: 'alpha **beta** gamma',
    expected: 'al*p~~ha **be*t~~a** gamma\n',
    strikeEdges: ['start', 'middle', 'end'],
    runJs: async (muya) => {
      setJsSelection(muya, 0, 2, 10)
      muya.format('em')
      setJsSelection(muya, 0, 4, 13)
      muya.format('del')
    },
    runRust: (rust) => {
      rust.setSelectionBetweenText('alpha ', 2, 'beta', 2)
      rust.request({ type: 'toggle_emphasis' })
      rust.setSelectionBetweenText('pha ', 1, 'ta', 1)
      rust.request({ type: 'toggle_strike' })
      return fragmentEdges(rust, 'strike')
    }
  },
  {
    name: 'apply strong inside the start fragment of linked emphasis',
    initial: 'alpha **beta** gamma',
    expected: 'al***pha** **be*ta** gamma\n',
    emphasisEdges: ['start', 'end'],
    runJs: async (muya) => {
      setJsSelection(muya, 0, 2, 10)
      muya.format('em')
      setJsSelection(muya, 0, 3, 6)
      muya.format('strong')
    },
    runRust: (rust) => {
      rust.setSelectionBetweenText('alpha ', 2, 'beta', 2)
      rust.request({ type: 'toggle_emphasis' })
      rust.setSelectionByText('pha ', 0, 3)
      rust.request({ type: 'toggle_strong' })
      return fragmentEdges(rust, 'emphasis')
    }
  },
  {
    name: 'reopen overlapping linked emphasis and strike groups',
    initial: 'al*p~~ha **be*t~~a** gamma',
    expected: 'al*p~~ha **be*t~~a** gamma\n',
    groups: {
      emphasis: ['start', 'end'],
      strike: ['start', 'middle', 'end']
    },
    runJs: async () => {},
    runRust: (rust) => ({
      emphasis: fragmentEdges(rust, 'emphasis'),
      strike: fragmentEdges(rust, 'strike')
    })
  }
]

describeBundled('Muya linked-group edit differential traces', () => {
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
      if (trace.strikeEdges) expect(result.rustResult).toEqual(trace.strikeEdges)
      if (trace.emphasisEdges) expect(result.rustResult).toEqual(trace.emphasisEdges)
      if (trace.groups) expect(result.rustResult).toEqual(trace.groups)
    })
  }
})
