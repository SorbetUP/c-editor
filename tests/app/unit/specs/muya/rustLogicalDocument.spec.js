import { describe, expect, it } from 'vitest'

import {
  MuyaRustLogicalDocument,
  createLogicalPatchAdapter
} from '../../../../../Elephant/frontend/src/muya/lib/rust/logicalDocument'

const documentNode = (id, children = []) => ({
  id,
  parent: null,
  children,
  kind: { layer: 'document' },
  source: null
})

const paragraphNode = (id, parent, children = []) => ({
  id,
  parent,
  children,
  kind: { layer: 'block', value: { type: 'paragraph' } },
  source: null
})

const textNode = (id, parent, value) => ({
  id,
  parent,
  children: [],
  kind: { layer: 'inline', value: { type: 'text', value } },
  source: null
})

const snapshot = () => ({
  markdown: 'A😀B\n\ntail',
  document: {
    root: 1,
    nodes: [
      documentNode(1, [2, 4]),
      paragraphNode(2, 1, [3]),
      textNode(3, 2, 'A😀B'),
      paragraphNode(4, 1, [5]),
      textNode(5, 4, 'tail')
    ]
  },
  revision: 0,
  selection: {
    anchor: { node: 3, offset_utf16: 0 },
    focus: { node: 3, offset_utf16: 0 }
  },
  can_undo: false,
  can_redo: false,
  composition_active: false
})

const update = (revision) => ({ revision })

const text = (document, id) => document.node(id).kind.value.value

describe('MuyaRustLogicalDocument', () => {
  it('applies every logical patch family while preserving a valid tree', () => {
    const document = new MuyaRustLogicalDocument(snapshot())

    document.applyPatches(
      [
        {
          type: 'replace_text',
          node: 3,
          range: { start: 1, end: 3 },
          inserted: 'X'
        }
      ],
      update(1)
    )
    expect(text(document, 3)).toBe('AXB')

    document.applyPatches(
      [
        {
          type: 'insert_node',
          parent: 2,
          index: 1,
          node: textNode(6, null, '!')
        }
      ],
      update(2)
    )
    expect(document.node(2).children).toEqual([3, 6])

    document.applyPatches(
      [{ type: 'move_node', node: 6, new_parent: 4, new_index: 0 }],
      update(3)
    )
    expect(document.node(2).children).toEqual([3])
    expect(document.node(4).children).toEqual([6, 5])

    document.applyPatches(
      [{ type: 'set_block_kind', node: 4, kind: { type: 'heading', level: 2 } }],
      update(4)
    )
    expect(document.node(4).kind.value).toEqual({ type: 'heading', level: 2 })

    document.applyPatches([{ type: 'remove_node', node: 5 }], update(5))
    expect(document.node(5)).toBeNull()

    document.applyPatches(
      [
        {
          type: 'insert_subtree',
          parent: 4,
          index: 1,
          subtree: {
            root: 7,
            nodes: [
              {
                id: 7,
                parent: null,
                children: [8],
                kind: { layer: 'inline', value: { type: 'strong' } },
                source: null
              },
              textNode(8, 7, 'bold')
            ]
          }
        }
      ],
      update(6)
    )

    expect(document.node(4).children).toEqual([6, 7])
    expect(document.node(8).parent).toBe(7)
    expect(document.validate()).toBe(true)
    expect(document.toProtocolDocument().nodes.map((node) => node.id)).toEqual([1, 2, 3, 4, 6, 7, 8])
  })

  it('rolls back the complete patch batch when a later patch is invalid', () => {
    const document = new MuyaRustLogicalDocument(snapshot())
    const before = document.toProtocolDocument()

    expect(() =>
      document.applyPatches(
        [
          {
            type: 'replace_text',
            node: 3,
            range: { start: 0, end: 1 },
            inserted: 'Z'
          },
          { type: 'move_node', node: 99, new_parent: 4, new_index: 0 }
        ],
        update(1)
      )
    ).toThrow('node 99 was not found')

    expect(document.revision).toBe(0)
    expect(document.toProtocolDocument()).toEqual(before)
    expect(text(document, 3)).toBe('A😀B')
  })

  it('rejects UTF-16 ranges that split a surrogate pair', () => {
    const document = new MuyaRustLogicalDocument(snapshot())
    expect(() =>
      document.applyPatches(
        [
          {
            type: 'replace_text',
            node: 3,
            range: { start: 2, end: 3 },
            inserted: ''
          }
        ],
        update(1)
      )
    ).toThrow('Invalid UTF-16 range')
    expect(text(document, 3)).toBe('A😀B')
  })

  it('provides bridge callbacks backed by one persistent logical document', async () => {
    const commits = []
    const adapter = createLogicalPatchAdapter({
      onCommit: (document) => commits.push({ revision: document.revision, value: text(document, 3) })
    })

    await adapter.applySnapshot(snapshot())
    await adapter.applyPatches(
      [
        {
          type: 'replace_text',
          node: 3,
          range: { start: 0, end: 1 },
          inserted: 'Z'
        }
      ],
      update(1)
    )

    expect(adapter.document.revision).toBe(1)
    expect(commits).toEqual([
      { revision: 0, value: 'A😀B' },
      { revision: 1, value: 'Z😀B' }
    ])
  })
})
