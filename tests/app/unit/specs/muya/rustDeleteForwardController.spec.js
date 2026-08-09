import { describe, expect, it } from 'vitest'

import { MuyaRustLogicalDocument } from '../../../../../Elephant/frontend/src/muya/lib/rust/logicalDocument'
import { planDeleteForward } from '../../../../../Elephant/frontend/src/muya/lib/rust/inputController/deleteForward'

const textNode = (id, parent, value) => ({
  id,
  parent,
  children: [],
  kind: { layer: 'inline', value: { type: 'text', value } },
  source: null
})

const paragraph = (id, parent, child) => ({
  id,
  parent,
  children: [child],
  kind: { layer: 'block', value: { type: 'paragraph' } },
  source: null
})

const rendererFor = (...values) => {
  const children = []
  const nodes = [{ id: 1, parent: null, children, kind: { layer: 'document' }, source: null }]
  values.forEach((value, index) => {
    const block = 2 + index * 2
    const text = block + 1
    children.push(block)
    nodes.push(paragraph(block, 1, text), textNode(text, block, value))
  })
  return {
    logical: new MuyaRustLogicalDocument({
      document: { root: 1, nodes },
      revision: 0
    })
  }
}

const caret = (node, offset) => ({
  anchor: { node, offset_utf16: offset },
  focus: { node, offset_utf16: offset }
})

describe('Rust delete-forward planning', () => {
  it('advances over one ASCII grapheme before invoking Rust backspace', () => {
    expect(planDeleteForward(rendererFor('alpha'), caret(3, 2))).toEqual({
      selection: caret(3, 3),
      command: { type: 'delete_backward' }
    })
  })

  it('advances over an entire UTF-16 emoji grapheme', () => {
    expect(planDeleteForward(rendererFor('A😀B'), caret(3, 1))).toEqual({
      selection: caret(3, 3),
      command: { type: 'delete_backward' }
    })
  })

  it('preserves a selected range for Rust to delete atomically', () => {
    const selection = {
      anchor: { node: 3, offset_utf16: 1 },
      focus: { node: 3, offset_utf16: 3 }
    }
    expect(planDeleteForward(rendererFor('alpha'), selection)).toEqual({
      selection,
      command: { type: 'delete_backward' }
    })
  })

  it('joins the next paragraph by backspacing at its first text node', () => {
    expect(planDeleteForward(rendererFor('left', 'right'), caret(3, 4))).toEqual({
      selection: caret(5, 0),
      command: { type: 'delete_backward' }
    })
  })

  it('prevents browser mutation without issuing a destructive command at document end', () => {
    expect(planDeleteForward(rendererFor('alpha'), caret(3, 5))).toBeNull()
  })
})
