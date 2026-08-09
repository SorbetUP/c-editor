import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelectionByAnyText,
  setJsSelectionByText
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip

const selectText = (muya, target) => setJsSelectionByText(muya, target, 1)
const selectAnyText = (muya, target) => setJsSelectionByAnyText(muya, target, 1)

const traces = [
  {
    name: 'duplicate a paragraph block',
    initial: 'alpha\n\nbeta',
    target: 'alpha',
    expected: 'alpha\n\nalpha\n\nbeta\n',
    selectJs: selectText,
    runJs: (muya) => muya.duplicate(),
    command: { type: 'duplicate_block' }
  },
  {
    name: 'duplicate a heading block',
    initial: '# alpha\n\nbeta',
    target: 'alpha',
    expected: '# alpha\n\n# alpha\n\nbeta\n',
    selectJs: selectAnyText,
    runJs: (muya) => muya.duplicate(),
    command: { type: 'duplicate_block' }
  },
  {
    name: 'duplicate an entire nested list root',
    initial: '- parent\n  - child\n\nafter',
    target: 'child',
    expected: '- parent\n  - child\n\n- parent\n  - child\n\nafter\n',
    selectJs: selectText,
    runJs: (muya) => muya.duplicate(),
    command: { type: 'duplicate_block' }
  },
  {
    name: 'delete a middle paragraph and select the next block',
    initial: 'before\n\nalpha\n\nafter',
    target: 'alpha',
    expected: 'before\n\nafter\n',
    selectJs: selectText,
    runJs: (muya) => muya.deleteParagraph(),
    command: { type: 'delete_block' }
  },
  {
    name: 'delete the final paragraph and select the previous block',
    initial: 'before\n\nalpha',
    target: 'alpha',
    expected: 'before\n',
    selectJs: selectText,
    runJs: (muya) => muya.deleteParagraph(),
    command: { type: 'delete_block' }
  },
  {
    name: 'delete the only paragraph and create an empty replacement',
    initial: 'alpha',
    target: 'alpha',
    expected: '\n',
    selectJs: selectText,
    runJs: (muya) => muya.deleteParagraph(),
    command: { type: 'delete_block' }
  },
  {
    name: 'insert a new paragraph after the selected root block',
    initial: 'alpha\n\nbeta',
    target: 'alpha',
    expected: 'alpha\n\n\n\nbeta\n',
    selectJs: selectText,
    runJs: (muya) => muya.insertParagraph('after', '', true),
    command: { type: 'insert_paragraph_after_block' }
  },
  {
    name: 'insert a new root paragraph after a nested list',
    initial: '- parent\n  - child\n\nafter',
    target: 'child',
    expected: '- parent\n  - child\n\n\n\nafter\n',
    selectJs: selectText,
    runJs: (muya) => muya.insertParagraph('after', '', true),
    command: { type: 'insert_paragraph_after_block' }
  }
]

const historyTraces = [
  {
    name: 'undo and redo a duplicated root block',
    initial: 'alpha\n\nbeta',
    target: 'alpha',
    command: { type: 'duplicate_block' },
    applyJs: (muya) => muya.duplicate(),
    checkpoints: [
      'alpha\n\nalpha\n\nbeta\n',
      'alpha\n\nbeta\n',
      'alpha\n\nalpha\n\nbeta\n'
    ]
  },
  {
    name: 'undo and redo deletion of the only block',
    initial: 'alpha',
    target: 'alpha',
    command: { type: 'delete_block' },
    applyJs: (muya) => muya.deleteParagraph(),
    checkpoints: ['\n', 'alpha\n', '\n']
  },
  {
    name: 'undo and redo insertion of an empty root paragraph',
    initial: 'alpha\n\nbeta',
    target: 'alpha',
    command: { type: 'insert_paragraph_after_block' },
    applyJs: (muya) => muya.insertParagraph('after', '', true),
    checkpoints: [
      'alpha\n\n\n\nbeta\n',
      'alpha\n\nbeta\n',
      'alpha\n\n\n\nbeta\n'
    ]
  }
]

describeBundled('Muya paragraph menu differential traces', () => {
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
        initial: trace.initial,
        runJs: async (muya) => {
          trace.selectJs(muya, trace.target)
          await trace.runJs(muya)
        },
        runRust: (rust) => {
          rust.setSelectionByText(trace.target, 1)
          rust.request(trace.command)
        }
      })
      jsEditor = result.jsEditor
      expect(result.jsMarkdown).toBe(result.rustMarkdown)
      expect(result.rustMarkdown).toBe(trace.expected)
    })
  }

  for (const trace of historyTraces) {
    it(trace.name, async () => {
      const result = await runDifferentialTrace({
        initial: trace.initial,
        runJs: async (muya) => {
          setJsSelectionByText(muya, trace.target, 1)
          await trace.applyJs(muya)
          const changed = muya.getMarkdown()
          muya.undo()
          const undone = muya.getMarkdown()
          muya.redo()
          return [changed, undone, muya.getMarkdown()]
        },
        runRust: (rust) => {
          rust.setSelectionByText(trace.target, 1)
          rust.request(trace.command)
          const changed = rust.markdown()
          rust.request({ type: 'undo' })
          const undone = rust.markdown()
          rust.request({ type: 'redo' })
          return [changed, undone, rust.markdown()]
        }
      })
      jsEditor = result.jsEditor
      expect(result.jsResult).toEqual(result.rustResult)
      expect(result.rustResult).toEqual(trace.checkpoints)
    })
  }
})
