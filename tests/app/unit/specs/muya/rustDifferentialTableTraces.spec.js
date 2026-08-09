import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  fakeKeyEvent,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelectionByText
} from './rustDifferentialHarness'

const selectedRustText = (rust) => {
  const snapshot = rust.snapshot()
  const selected = snapshot.document.nodes.find(
    (node) => node.id === snapshot.selection.focus.node
  )
  return selected?.kind?.value?.value
}

const describeBundled = bundled ? describe : describe.skip
const traces = [
  {
    name: 'insert a body row after the current table row',
    initial: '| A | B |\n| :--- | ---: |\n| one | two |',
    expected: '| A   | B   |\n|:--- | ---:|\n| one | two |\n|     |     |\n',
    runJs: async (muya) => {
      setJsSelectionByText(muya, 'one', 0)
      muya.contentState.editTable({
        location: 'next',
        action: 'insert',
        target: 'row'
      })
    },
    runRust: (rust) => {
      rust.setSelectionByText('one', 0)
      rust.request({ type: 'insert_table_row_after' })
    }
  },
  {
    name: 'delete the current body row while preserving the next row',
    initial: '| A | B |\n| --- | --- |\n| one | two |\n| three | four |',
    expected: '| A     | B    |\n| ----- | ---- |\n| three | four |\n',
    runJs: async (muya) => {
      setJsSelectionByText(muya, 'one', 0)
      muya.contentState.editTable({
        location: 'current',
        action: 'delete',
        target: 'row'
      })
    },
    runRust: (rust) => {
      rust.setSelectionByText('one', 0)
      rust.request({ type: 'delete_table_row' })
    }
  },
  {
    name: 'insert a column after the current table column',
    initial: '| A | B |\n| :--- | ---: |\n| one | two |',
    expected: '| A   |     | B   |\n|:--- | --- | ---:|\n| one |     | two |\n',
    runJs: async (muya) => {
      setJsSelectionByText(muya, 'one', 0)
      muya.contentState.editTable({
        location: 'next',
        action: 'insert',
        target: 'column'
      })
    },
    runRust: (rust) => {
      rust.setSelectionByText('one', 0)
      rust.request({ type: 'insert_table_column_after' })
    }
  },
  {
    name: 'delete the current table column across every row',
    initial: '| A | B | C |\n| --- | :---: | ---: |\n| one | two | three |',
    expected: '| A   | C     |\n| --- | -----:|\n| one | three |\n',
    runJs: async (muya) => {
      setJsSelectionByText(muya, 'two', 0)
      muya.contentState.editTable({
        location: 'current',
        action: 'delete',
        target: 'column'
      })
    },
    runRust: (rust) => {
      rust.setSelectionByText('two', 0)
      rust.request({ type: 'delete_table_column' })
    }
  },
  {
    name: 'move to the next table cell with Tab',
    initial: '| A | B |\n| --- | --- |\n| one | two |',
    expected: '| A   | B   |\n| --- | --- |\n| one | two |\n',
    selectedText: 'two',
    runJs: async (muya) => {
      setJsSelectionByText(muya, 'one', 0)
      muya.contentState.tabHandler(fakeKeyEvent())
      return muya.contentState.getBlock(muya.contentState.cursor.start.key)?.text
    },
    runRust: (rust) => {
      rust.setSelectionByText('one', 0)
      rust.request({ type: 'next_table_cell' })
      return selectedRustText(rust)
    }
  },
  {
    name: 'wrap to the previous row with Shift-Tab',
    initial: '| A | B |\n| --- | --- |\n| one | two |\n| three | four |',
    expected: '| A     | B    |\n| ----- | ---- |\n| one   | two  |\n| three | four |\n',
    selectedText: 'two',
    runJs: async (muya) => {
      setJsSelectionByText(muya, 'three', 0)
      muya.contentState.tabHandler(fakeKeyEvent({ shiftKey: true }))
      return muya.contentState.getBlock(muya.contentState.cursor.start.key)?.text
    },
    runRust: (rust) => {
      rust.setSelectionByText('three', 0)
      rust.request({ type: 'previous_table_cell' })
      return selectedRustText(rust)
    }
  },
  {
    name: 'insert tab spaces in the final table cell',
    initial: '| A | B |\n| --- | --- |\n| one | two |',
    expected: '| A   | B   |\n| --- | --- |\n| one | two |\n',
    selectedText: 'two    ',
    runJs: async (muya) => {
      setJsSelectionByText(muya, 'two', 3)
      muya.contentState.tabHandler(fakeKeyEvent())
      return muya.contentState.getBlock(muya.contentState.cursor.start.key)?.text
    },
    runRust: (rust) => {
      rust.setSelectionByText('two', 3)
      rust.request({ type: 'next_table_cell' })
      return selectedRustText(rust)
    }
  }
]

describeBundled('Muya table differential traces', () => {
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
      if (Object.hasOwn(trace, 'selectedText')) {
        expect(result.jsResult).toBe(result.rustResult)
        expect(result.rustResult).toBe(trace.selectedText)
      }
    })
  }
})
