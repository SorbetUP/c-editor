import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  fakeKeyEvent,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelection,
  setJsSelectionByText
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip
const traces = [
  {
    name: 'merge a paragraph into the previous paragraph with Backspace',
    initial: 'alpha\n\nbeta',
    expected: 'alphabeta\n',
    runJs: async (muya) => {
      setJsSelection(muya, 1, 0)
      muya.contentState.backspaceHandler(fakeKeyEvent())
    },
    runRust: (rust) => {
      rust.setSelection(1, 0)
      rust.request({ type: 'delete_backward' })
    }
  },
  {
    name: 'remove the first unordered list marker with Backspace',
    initial: '- alpha',
    expected: 'alpha\n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 0)
      muya.contentState.backspaceHandler(fakeKeyEvent())
    },
    runRust: (rust) => {
      rust.setSelection(0, 0)
      rust.request({ type: 'delete_backward' })
    }
  },
  {
    name: 'exit the final empty unordered list item with Enter',
    initial: '- alpha\n- ',
    expected: '- alpha\n\n\n',
    runJs: async (muya) => {
      setJsSelection(muya, 1, 0)
      muya.contentState.enterHandler(fakeKeyEvent())
    },
    runRust: (rust) => {
      rust.setSelection(1, 0)
      rust.request({ type: 'insert_paragraph' })
    }
  },
  {
    name: 'create an unchecked task when splitting at the end',
    initial: '- [x] alpha',
    expected: '- [x] alpha\n- [ ] \n',
    runJs: async (muya) => {
      setJsSelection(muya, 0, 5)
      muya.contentState.enterHandler(fakeKeyEvent())
    },
    runRust: (rust) => {
      rust.setSelection(0, 5)
      rust.request({ type: 'insert_paragraph' })
    }
  },
  {
    name: 'undo and redo one strong-format transaction',
    initial: 'alpha',
    expected: 'a**lph**a\n',
    checkpoints: ['a**lph**a\n', 'alpha\n', 'a**lph**a\n'],
    runJs: async (muya) => {
      setJsSelection(muya, 0, 1, 4)
      muya.format('strong')
      const formatted = muya.getMarkdown()
      muya.undo()
      const undone = muya.getMarkdown()
      muya.redo()
      return [formatted, undone, muya.getMarkdown()]
    },
    runRust: (rust) => {
      rust.setSelection(0, 1, 4)
      rust.request({ type: 'toggle_strong' })
      const formatted = rust.markdown()
      rust.request({ type: 'undo' })
      const undone = rust.markdown()
      rust.request({ type: 'redo' })
      return [formatted, undone, rust.markdown()]
    }
  },
  {
    name: 'undo and redo an inserted table row',
    initial: '| A | B |\n| :--- | ---: |\n| one | two |',
    expected: '| A   | B   |\n|:--- | ---:|\n| one | two |\n|     |     |\n',
    checkpoints: [
      '| A   | B   |\n|:--- | ---:|\n| one | two |\n|     |     |\n',
      '| A   | B   |\n|:--- | ---:|\n| one | two |\n',
      '| A   | B   |\n|:--- | ---:|\n| one | two |\n|     |     |\n'
    ],
    runJs: async (muya) => {
      setJsSelectionByText(muya, 'one', 0)
      muya.contentState.editTable({
        location: 'next',
        action: 'insert',
        target: 'row'
      })
      const edited = muya.getMarkdown()
      muya.undo()
      const undone = muya.getMarkdown()
      muya.redo()
      return [edited, undone, muya.getMarkdown()]
    },
    runRust: (rust) => {
      rust.setSelectionByText('one', 0)
      rust.request({ type: 'insert_table_row_after' })
      const edited = rust.markdown()
      rust.request({ type: 'undo' })
      const undone = rust.markdown()
      rust.request({ type: 'redo' })
      return [edited, undone, rust.markdown()]
    }
  },
  {
    name: 'undo and redo an inserted table column',
    initial: '| A | B |\n| :--- | ---: |\n| one | two |',
    expected: '| A   |     | B   |\n|:--- | --- | ---:|\n| one |     | two |\n',
    checkpoints: [
      '| A   |     | B   |\n|:--- | --- | ---:|\n| one |     | two |\n',
      '| A   | B   |\n|:--- | ---:|\n| one | two |\n',
      '| A   |     | B   |\n|:--- | --- | ---:|\n| one |     | two |\n'
    ],
    runJs: async (muya) => {
      setJsSelectionByText(muya, 'one', 0)
      muya.contentState.editTable({
        location: 'next',
        action: 'insert',
        target: 'column'
      })
      const edited = muya.getMarkdown()
      muya.undo()
      const undone = muya.getMarkdown()
      muya.redo()
      return [edited, undone, muya.getMarkdown()]
    },
    runRust: (rust) => {
      rust.setSelectionByText('one', 0)
      rust.request({ type: 'insert_table_column_after' })
      const edited = rust.markdown()
      rust.request({ type: 'undo' })
      const undone = rust.markdown()
      rust.request({ type: 'redo' })
      return [edited, undone, rust.markdown()]
    }
  }
]

describeBundled('Muya Backspace and history differential traces', () => {
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
      if (trace.checkpoints) {
        expect(result.jsResult).toEqual(result.rustResult)
        expect(result.rustResult).toEqual(trace.checkpoints)
      }
    })
  }
})
