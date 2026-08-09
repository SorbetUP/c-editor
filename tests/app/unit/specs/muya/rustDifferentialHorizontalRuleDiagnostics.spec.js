import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelection,
  setJsSelectionByText,
  settle
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip

const insertRuleInJs = async (muya) => {
  muya.updateParagraph('hr')
  await settle()
}

describeBundled('Muya horizontal rule differential traces', () => {
  let jsEditor = null

  beforeAll(initializeRustWasm)

  afterEach(() => {
    jsEditor?.destroy?.()
    jsEditor = null
    document.body.innerHTML = ''
  })

  it('keeps a non-empty paragraph unchanged', async () => {
    const result = await runDifferentialTrace({
      initial: 'alpha',
      runJs: async (muya) => {
        setJsSelectionByText(muya, 'alpha', 2)
        await insertRuleInJs(muya)
      },
      runRust: (rust) => {
        rust.setSelectionByText('alpha', 2)
        rust.request({ type: 'insert_horizontal_rule' })
      }
    })
    jsEditor = result.jsEditor
    expect(result.jsMarkdown).toBe(result.rustMarkdown)
    expect(result.rustMarkdown).toBe('alpha\n')
  })

  it('inserts a rule before an empty paragraph', async () => {
    const result = await runDifferentialTrace({
      initial: '',
      runJs: async (muya) => {
        setJsSelection(muya, 0, 0)
        await insertRuleInJs(muya)
      },
      runRust: (rust) => {
        rust.setSelection(0, 0)
        rust.request({ type: 'insert_horizontal_rule' })
      }
    })
    jsEditor = result.jsEditor
    expect(result.jsMarkdown).toBe(result.rustMarkdown)
    expect(result.rustMarkdown).toBe('---\n\n\n')
  })

  it('undoes and redoes one horizontal rule insertion', async () => {
    const checkpoints = ['---\n\n\n', '\n', '---\n\n\n']
    const result = await runDifferentialTrace({
      initial: '',
      runJs: async (muya) => {
        setJsSelection(muya, 0, 0)
        await insertRuleInJs(muya)
        const inserted = muya.getMarkdown()
        muya.undo()
        const undone = muya.getMarkdown()
        muya.redo()
        return [inserted, undone, muya.getMarkdown()]
      },
      runRust: (rust) => {
        rust.setSelection(0, 0)
        rust.request({ type: 'insert_horizontal_rule' })
        const inserted = rust.markdown()
        rust.request({ type: 'undo' })
        const undone = rust.markdown()
        rust.request({ type: 'redo' })
        return [inserted, undone, rust.markdown()]
      }
    })
    jsEditor = result.jsEditor
    expect(result.jsResult).toEqual(result.rustResult)
    expect(result.rustResult).toEqual(checkpoints)
  })
})
