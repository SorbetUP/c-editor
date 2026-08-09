import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelection,
  setJsSelectionByText
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip
const emptyTable = '|     |     |\n| --- | --- |\n|     |     |\n'

const cases = [
  {
    name: 'create a 2x2 table in an empty document',
    initial: '',
    expected: emptyTable,
    selectJs: (muya) => setJsSelection(muya, 0, 0),
    selectRust: (rust) => rust.setSelection(0, 0)
  },
  {
    name: 'create a 2x2 table after a non-empty paragraph',
    initial: 'alpha',
    expected: `alpha\n\n${emptyTable}`,
    selectJs: (muya) => setJsSelectionByText(muya, 'alpha', 2),
    selectRust: (rust) => rust.setSelectionByText('alpha', 2)
  }
]

describeBundled('Muya table creation differential traces', () => {
  let jsEditor = null

  beforeAll(initializeRustWasm)

  afterEach(() => {
    jsEditor?.destroy?.()
    jsEditor = null
    document.body.innerHTML = ''
  })

  for (const testCase of cases) {
    it(testCase.name, async () => {
      const result = await runDifferentialTrace({
        initial: testCase.initial,
        runJs: (muya) => {
          testCase.selectJs(muya)
          muya.createTable({ rows: 2, columns: 2 })
        },
        runRust: (rust) => {
          testCase.selectRust(rust)
          rust.request({ type: 'create_table', rows: 2, columns: 2 })
        }
      })
      jsEditor = result.jsEditor
      expect(result.jsMarkdown).toBe(result.rustMarkdown)
      expect(result.rustMarkdown).toBe(testCase.expected)
    })
  }

  it('undoes and redoes table creation atomically', async () => {
    const changed = `alpha\n\n${emptyTable}`
    const result = await runDifferentialTrace({
      initial: 'alpha',
      runJs: (muya) => {
        setJsSelectionByText(muya, 'alpha', 2)
        muya.createTable({ rows: 2, columns: 2 })
        const afterCreate = muya.getMarkdown()
        muya.undo()
        const afterUndo = muya.getMarkdown()
        muya.redo()
        return [afterCreate, afterUndo, muya.getMarkdown()]
      },
      runRust: (rust) => {
        rust.setSelectionByText('alpha', 2)
        rust.request({ type: 'create_table', rows: 2, columns: 2 })
        const afterCreate = rust.markdown()
        rust.request({ type: 'undo' })
        const afterUndo = rust.markdown()
        rust.request({ type: 'redo' })
        return [afterCreate, afterUndo, rust.markdown()]
      }
    })
    jsEditor = result.jsEditor
    expect(result.jsResult).toEqual(result.rustResult)
    expect(result.rustResult).toEqual([changed, 'alpha\n', changed])
  })
})
