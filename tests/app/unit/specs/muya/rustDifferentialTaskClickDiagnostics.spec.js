import { afterEach, beforeAll, describe, expect, it } from 'vitest'

import {
  bundled,
  initializeRustWasm,
  runDifferentialTrace,
  settle
} from './rustDifferentialHarness'

const describeBundled = bundled ? describe : describe.skip

const taskBoxes = (muya) =>
  Array.from(muya.container.querySelectorAll('input.ag-task-list-item-checkbox'))

const clone = (value) => JSON.parse(JSON.stringify(value))

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

const clickTask = async (muya, index, checked, autoCheck = false) => {
  muya.options.autoCheck = autoCheck
  const checkbox = taskBoxes(muya)[index]
  if (!checkbox) throw new Error(`Task checkbox ${index} is unavailable.`)
  if (checkbox.checked !== checked) checkbox.click()
  await settle()
  muya._markdownBlockCache?.clear()
}

const rustTaskItems = (rust) =>
  rust.snapshot().document.nodes.filter(
    (node) =>
      node.kind?.layer === 'block' &&
      node.kind?.value?.type === 'list_item' &&
      typeof node.kind?.value?.checked === 'boolean'
  )

const setRustTask = (rust, index, checked, autoCheck = false) => {
  const item = rustTaskItems(rust)[index]
  if (!item) throw new Error(`Rust task item ${index} is unavailable.`)
  rust.request({
    type: 'set_task_checked',
    item: item.id,
    checked,
    auto_check: autoCheck
  })
}

const traces = [
  {
    name: 'check an unchecked task',
    initial: '- [ ] alpha',
    expected: '- [x] alpha\n',
    runJs: (muya) => clickTask(muya, 0, true),
    runRust: (rust) => setRustTask(rust, 0, true)
  },
  {
    name: 'uncheck a checked task',
    initial: '- [x] alpha',
    expected: '- [ ] alpha\n',
    runJs: (muya) => clickTask(muya, 0, false),
    runRust: (rust) => setRustTask(rust, 0, false)
  },
  {
    name: 'keep nested tasks unchanged when autoCheck is disabled',
    initial: '- [ ] parent\n  - [ ] child',
    expected: '- [x] parent\n  - [ ] child\n',
    runJs: (muya) => clickTask(muya, 0, true),
    runRust: (rust) => setRustTask(rust, 0, true)
  },
  {
    name: 'cascade task state to descendants when autoCheck is enabled',
    initial: '- [ ] parent\n  - [ ] child',
    expected: '- [x] parent\n  - [x] child\n',
    runJs: (muya) => clickTask(muya, 0, true, true),
    runRust: (rust) => setRustTask(rust, 0, true, true)
  },
  {
    name: 'undo and redo one task checkbox mutation',
    initial: '- [ ] alpha',
    expected: '- [x] alpha\n',
    checkpoints: ['- [x] alpha\n', '- [ ] alpha\n', '- [x] alpha\n'],
    runJs: async (muya) => {
      const initialState = captureJsHistoryState(muya, false)
      await clickTask(muya, 0, true)
      const changedState = captureJsHistoryState(muya, true)
      seedJsHistory(muya, initialState, changedState)
      const checked = readJsMarkdown(muya)
      muya.undo()
      await settle()
      const undone = readJsMarkdown(muya)
      muya.redo()
      await settle()
      return [checked, undone, readJsMarkdown(muya)]
    },
    runRust: (rust) => {
      setRustTask(rust, 0, true)
      const checked = rust.markdown()
      rust.request({ type: 'undo' })
      const undone = rust.markdown()
      rust.request({ type: 'redo' })
      return [checked, undone, rust.markdown()]
    }
  },
  {
    name: 'undo and redo an autoCheck task cascade atomically',
    initial: '- [ ] parent\n  - [ ] child',
    expected: '- [x] parent\n  - [x] child\n',
    checkpoints: [
      '- [x] parent\n  - [x] child\n',
      '- [ ] parent\n  - [ ] child\n',
      '- [x] parent\n  - [x] child\n'
    ],
    runJs: async (muya) => {
      const initialState = captureJsHistoryState(muya, false)
      await clickTask(muya, 0, true, true)
      const changedState = captureJsHistoryState(muya, true)
      seedJsHistory(muya, initialState, changedState)
      const checked = readJsMarkdown(muya)
      muya.undo()
      await settle()
      const undone = readJsMarkdown(muya)
      muya.redo()
      await settle()
      return [checked, undone, readJsMarkdown(muya)]
    },
    runRust: (rust) => {
      setRustTask(rust, 0, true, true)
      const checked = rust.markdown()
      rust.request({ type: 'undo' })
      const undone = rust.markdown()
      rust.request({ type: 'redo' })
      return [checked, undone, rust.markdown()]
    }
  }
]

describeBundled('Muya task checkbox differential traces', () => {
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
