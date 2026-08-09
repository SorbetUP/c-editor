import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  fakeKeyEvent,
  initializeRustWasm,
  runDifferentialTrace,
  setJsSelection
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip

const nextGraphemeEnd = (value, offset) => {
  if (typeof Intl?.Segmenter === 'function') {
    const segments = new Intl.Segmenter(undefined, { granularity: 'grapheme' }).segment(value)
    for (const segment of segments) {
      const end = segment.index + segment.segment.length
      if (segment.index <= offset && offset < end) return end
    }
  }
  const next = Array.from(value.slice(offset))[0]
  return next ? offset + next.length : offset
}

const simulateNativeDeleteForward = (muya) => {
  const domSelection = window.getSelection()
  if (!domSelection?.rangeCount) return
  const range = domSelection.getRangeAt(0)
  const target = range.startContainer.nodeType === Node.TEXT_NODE
    ? range.startContainer.parentElement
    : range.startContainer

  if (range.collapsed && range.startContainer.nodeType === Node.TEXT_NODE) {
    const value = range.startContainer.nodeValue || ''
    const end = nextGraphemeEnd(value, range.startOffset)
    range.setEnd(range.startContainer, end)
  }
  range.deleteContents()
  range.collapse(true)
  domSelection.removeAllRanges()
  domSelection.addRange(range)

  ;(target || muya.container).dispatchEvent(
    new InputEvent('input', {
      inputType: 'deleteContentForward',
      bubbles: true,
      cancelable: false
    })
  )
}

const deleteForward = async (muya) => {
  const event = fakeKeyEvent({ key: 'Delete', code: 'Delete', keyCode: 46, which: 46 })
  await muya.contentState.deleteHandler(event)
  if (!event.preventDefault.mock.calls.length) simulateNativeDeleteForward(muya)
}

const cases = [
  {
    name: 'delete the next ASCII grapheme',
    initial: 'alpha',
    textIndex: 0,
    start: 2,
    rustSelection: [0, 3],
    expected: 'alha\n'
  },
  {
    name: 'delete the next UTF-16 emoji grapheme',
    initial: 'A😀B',
    textIndex: 0,
    start: 1,
    rustSelection: [0, 3],
    expected: 'AB\n'
  },
  {
    name: 'delete a selected range forwards',
    initial: 'alpha',
    textIndex: 0,
    start: 1,
    end: 3,
    rustSelection: [0, 1, 3],
    expected: 'aha\n'
  },
  {
    name: 'join the next paragraph at a forward boundary',
    initial: 'left\n\nright',
    textIndex: 0,
    start: 4,
    rustSelection: [1, 0],
    expected: 'leftright\n'
  },
  {
    name: 'keep the final paragraph end unchanged',
    initial: 'alpha',
    textIndex: 0,
    start: 5,
    rustSelection: null,
    expected: 'alpha\n'
  }
]

describeBundled('Muya delete-forward differential traces', () => {
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
        runJs: async (muya) => {
          setJsSelection(
            muya,
            testCase.textIndex,
            testCase.start,
            testCase.end ?? testCase.start
          )
          await deleteForward(muya)
        },
        runRust: (rust) => {
          if (!testCase.rustSelection) return
          rust.setSelection(...testCase.rustSelection)
          rust.request({ type: 'delete_backward' })
        }
      })
      jsEditor = result.jsEditor
      expect(result.jsMarkdown).toBe(result.rustMarkdown)
      expect(result.rustMarkdown).toBe(testCase.expected)
    })
  }
})
