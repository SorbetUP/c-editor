import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelectionByAnyText,
  setJsSelectionByText,
  settle
} from './rustDifferentialHarness'
import {
  blockTypeCases,
  blockTypeHistoryCases
} from './rustDifferentialBlockTypeCases'

const describeBundled = bundled ? describe : describe.skip
const clone = (value) => JSON.parse(JSON.stringify(value))

const selectJs = (muya, target, anyText = false) => {
  const select = anyText ? setJsSelectionByAnyText : setJsSelectionByText
  select(muya, target, 1)
}

const captureJsHistoryState = (muya, isEdit) => {
  const { blocks, renderRange, cursor } = muya.contentState
  return {
    blocks: clone(blocks),
    renderRange: clone(renderRange),
    cursor: {
      start: { key: cursor.start.key, offset: cursor.start.offset },
      end: { key: cursor.end.key, offset: cursor.end.offset },
      isEdit
    }
  }
}

const seedJsHistory = (muya, initialState, changedState) => {
  const { history } = muya.contentState
  history.clearHistory()
  history.push(initialState)
  history.push(changedState)
}

const readJsMarkdown = (muya) => {
  muya._markdownBlockCache?.clear()
  return muya.getMarkdown()
}

const updateJsParagraph = async (muya, paragraphType) => {
  await muya.updateParagraph(paragraphType)
  await settle()
}

describeBundled('Muya block type differential traces', () => {
  let jsEditor = null

  beforeAll(initializeRustWasm)

  afterEach(() => {
    jsEditor?.destroy?.()
    jsEditor = null
    document.body.innerHTML = ''
  })

  for (const [name, initial, target, paragraphType, expected, command, anyText] of blockTypeCases) {
    it(name, async () => {
      const result = await runDifferentialTrace({
        initial,
        runJs: async (muya) => {
          selectJs(muya, target, anyText)
          await updateJsParagraph(muya, paragraphType)
        },
        runRust: (rust) => {
          rust.setSelectionByText(target, 1)
          rust.request(command)
        }
      })
      jsEditor = result.jsEditor
      expect(result.jsMarkdown).toBe(result.rustMarkdown)
      expect(result.rustMarkdown).toBe(expected)
    })
  }

  for (const trace of blockTypeHistoryCases) {
    it(trace.name, async () => {
      const result = await runDifferentialTrace({
        initial: trace.initial,
        runJs: async (muya) => {
          selectJs(muya, trace.target)
          const initialState = captureJsHistoryState(muya, false)
          await updateJsParagraph(muya, trace.paragraphType)
          const changedState = captureJsHistoryState(muya, true)
          seedJsHistory(muya, initialState, changedState)
          const changed = readJsMarkdown(muya)
          muya.undo()
          await settle()
          const undone = readJsMarkdown(muya)
          muya.redo()
          await settle()
          return [changed, undone, readJsMarkdown(muya)]
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
