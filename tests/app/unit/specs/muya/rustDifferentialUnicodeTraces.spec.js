import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import selection from '../../../../../Elephant/frontend/src/muya/lib/selection'
import {
  bundled,
  fakeKeyEvent,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelection
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip

const editableBlocks = (muya) => {
  const output = []
  const visit = (block) => {
    if (block?.functionType === 'paragraphContent') output.push(block)
    for (const child of block?.children || []) visit(child)
  }
  for (const block of muya.contentState.getBlocks()) visit(block)
  return output
}

const applyBrowserNativeBackspace = (muya, grapheme) => {
  const block = editableBlocks(muya)[0]
  if (!block) throw new Error('Muya JavaScript text block was not found.')

  const caret = 1 + grapheme.length
  setJsSelection(muya, 0, caret)
  muya.contentState.backspaceHandler(fakeKeyEvent())

  const paragraph = document.querySelector(`#${block.key}`)
  if (!paragraph) throw new Error(`Muya JavaScript paragraph ${block.key} was not found.`)
  paragraph.textContent = 'AB'
  selection.setCursorRange({
    anchor: { key: block.key, offset: 1 },
    focus: { key: block.key, offset: 1 }
  })
  muya.contentState.inputHandler(
    fakeKeyEvent({ type: 'input', inputType: 'deleteContentBackward', data: null })
  )
}

const backspaceTrace = (name, grapheme) => ({
  name,
  initial: `A${grapheme}B`,
  expected: 'AB\n',
  runJs: async (muya) => applyBrowserNativeBackspace(muya, grapheme),
  runRust: (rust) => {
    rust.setSelection(0, 1 + grapheme.length)
    rust.request({ type: 'delete_backward' })
  }
})

const traces = [
  backspaceTrace('sync browser deletion of one astral emoji', '😀'),
  backspaceTrace('sync browser deletion of one regional-indicator flag', '🇫🇷'),
  backspaceTrace('sync browser deletion of one ZWJ family', '👨‍👩‍👧‍👦'),
  backspaceTrace('sync browser deletion of one combining-accent grapheme', 'e\u0301')
]

describeBundled('Muya UTF-16 differential traces', () => {
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
