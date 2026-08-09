import { describe, expect, it } from 'vitest'

import { MuyaRustLogicalDocument } from '../../../../../Elephant/frontend/src/muya/lib/rust/logicalDocument'
import {
  DELETE_UNIT_INPUTS,
  planDeleteUnit
} from '../../../../../Elephant/frontend/src/muya/lib/rust/inputController/deleteUnits'

const rendererFor = (value) => ({
  logical: new MuyaRustLogicalDocument({
    revision: 0,
    document: {
      root: 1,
      nodes: [
        { id: 1, parent: null, children: [2], kind: { layer: 'document' }, source: null },
        {
          id: 2,
          parent: 1,
          children: [3],
          kind: { layer: 'block', value: { type: 'paragraph' } },
          source: null
        },
        {
          id: 3,
          parent: 2,
          children: [],
          kind: { layer: 'inline', value: { type: 'text', value } },
          source: null
        }
      ]
    }
  })
})

const caret = (offset) => ({
  anchor: { node: 3, offset_utf16: offset },
  focus: { node: 3, offset_utf16: offset }
})

const expected = (start, end) => ({
  selection: {
    anchor: { node: 3, offset_utf16: start },
    focus: { node: 3, offset_utf16: end }
  },
  command: { type: 'delete_backward' }
})

describe('Rust word and line deletion planning', () => {
  it('declares every captured browser deletion unit', () => {
    expect([...DELETE_UNIT_INPUTS]).toEqual([
      'deleteWordBackward',
      'deleteWordForward',
      'deleteSoftLineBackward',
      'deleteSoftLineForward',
      'deleteHardLineBackward',
      'deleteHardLineForward',
      'deleteEntireSoftLine'
    ])
  })

  it('deletes the previous and next word as Rust ranges', () => {
    const renderer = rendererFor('one two three')
    expect(planDeleteUnit(renderer, caret(7), 'deleteWordBackward')).toEqual(expected(4, 7))
    expect(planDeleteUnit(renderer, caret(4), 'deleteWordForward')).toEqual(expected(4, 7))
  })

  it('deletes backward, forward, or the whole current line', () => {
    const renderer = rendererFor('one\ntwo\nthree')
    expect(planDeleteUnit(renderer, caret(6), 'deleteSoftLineBackward')).toEqual(expected(4, 6))
    expect(planDeleteUnit(renderer, caret(5), 'deleteHardLineForward')).toEqual(expected(5, 7))
    expect(planDeleteUnit(renderer, caret(5), 'deleteEntireSoftLine')).toEqual(expected(4, 7))
  })

  it('preserves an explicit browser selection for one Rust deletion', () => {
    const selection = {
      anchor: { node: 3, offset_utf16: 1 },
      focus: { node: 3, offset_utf16: 8 }
    }
    expect(planDeleteUnit(rendererFor('one two three'), selection, 'deleteWordForward')).toEqual({
      selection,
      command: { type: 'delete_backward' }
    })
  })
})
