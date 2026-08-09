import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelectionByText,
  settle
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip

const dispatchBrowserInput = async (target, inputType, data = null) => {
  if (!target) throw new Error('Muya JS input target is unavailable.')
  const event = new Event('input', { bubbles: true })
  Object.defineProperty(event, 'inputType', { configurable: true, value: inputType })
  Object.defineProperty(event, 'data', { configurable: true, value: data })
  target.dispatchEvent(event)
  await settle()
}

const insertThroughBrowserDom = async (text) => {
  const browserSelection = document.defaultView?.getSelection?.()
  if (!browserSelection || browserSelection.rangeCount === 0) {
    throw new Error('Muya JS browser selection is unavailable.')
  }
  const range = browserSelection.getRangeAt(0)
  range.deleteContents()
  const inserted = document.createTextNode(text)
  range.insertNode(inserted)
  range.setStartAfter(inserted)
  range.collapse(true)
  browserSelection.removeAllRanges()
  browserSelection.addRange(range)
  await dispatchBrowserInput(inserted.parentElement, 'insertText', text)
}

const deleteBackwardThroughBrowserDom = async () => {
  const browserSelection = document.defaultView?.getSelection?.()
  if (!browserSelection || browserSelection.rangeCount === 0) {
    throw new Error('Muya JS browser selection is unavailable.')
  }
  const range = browserSelection.getRangeAt(0)
  if (!range.collapsed || range.startContainer.nodeType !== 3) {
    throw new Error('Muya JS deletion requires a collapsed text selection.')
  }
  const text = range.startContainer
  const prefix = text.data.slice(0, range.startOffset)
  const segments = [
    ...new Intl.Segmenter(undefined, { granularity: 'grapheme' }).segment(prefix)
  ]
  const previous = segments.at(-1)
  if (!previous) throw new Error('Muya JS deletion has no preceding grapheme.')

  range.setStart(text, previous.index)
  range.deleteContents()
  range.collapse(true)
  browserSelection.removeAllRanges()
  browserSelection.addRange(range)
  await dispatchBrowserInput(text.parentElement, 'deleteContentBackward')
}

const traces = [
  {
    name: 'insert text inside inline code',
    initial: '`alpha`',
    expected: '`alXpha`\n',
    runJs: async (muya) => {
      setJsSelectionByText(muya, '`alpha`', 3)
      await insertThroughBrowserDom('X')
    },
    runRust: (rust) => {
      rust.setSelectionByCode('alpha', 2)
      rust.request({ type: 'insert_text', text: 'X' })
    }
  },
  {
    name: 'replace a selection inside inline code',
    initial: '`alpha`',
    expected: '`aXa`\n',
    runJs: async (muya) => {
      setJsSelectionByText(muya, '`alpha`', 2, 5)
      await insertThroughBrowserDom('X')
    },
    runRust: (rust) => {
      rust.setSelectionByCode('alpha', 1, 4)
      rust.request({ type: 'insert_text', text: 'X' })
    }
  },
  {
    name: 'delete one character inside inline code',
    initial: '`alpha`',
    expected: '`alha`\n',
    runJs: async (muya) => {
      setJsSelectionByText(muya, '`alpha`', 4)
      await deleteBackwardThroughBrowserDom()
    },
    runRust: (rust) => {
      rust.setSelectionByCode('alpha', 3)
      rust.request({ type: 'delete_backward' })
    }
  },
  {
    name: 'delete one emoji grapheme inside inline code',
    initial: '`A🇫🇷B`',
    expected: '`AB`\n',
    runJs: async (muya) => {
      setJsSelectionByText(muya, '`A🇫🇷B`', 6)
      await deleteBackwardThroughBrowserDom()
    },
    runRust: (rust) => {
      rust.setSelectionByCode('A🇫🇷B', 5)
      rust.request({ type: 'delete_backward' })
    }
  }
]

describeBundled('Muya inline-code editing differential traces', () => {
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

  it('undoes and redoes inline-code text insertion atomically', async () => {
    const changed = '`alXpha`\n'
    const result = await runDifferentialTrace({
      initial: '`alpha`',
      runJs: async (muya) => {
        setJsSelectionByText(muya, '`alpha`', 3)
        await insertThroughBrowserDom('X')
        const afterInsert = muya.getMarkdown()
        muya.undo()
        const afterUndo = muya.getMarkdown()
        muya.redo()
        return [afterInsert, afterUndo, muya.getMarkdown()]
      },
      runRust: (rust) => {
        rust.setSelectionByCode('alpha', 2)
        rust.request({ type: 'insert_text', text: 'X' })
        const afterInsert = rust.markdown()
        rust.request({ type: 'undo' })
        const afterUndo = rust.markdown()
        rust.request({ type: 'redo' })
        return [afterInsert, afterUndo, rust.markdown()]
      }
    })
    jsEditor = result.jsEditor
    expect(result.jsResult).toEqual(result.rustResult)
    expect(result.rustResult).toEqual([changed, '`alpha`\n', changed])
  })
})
