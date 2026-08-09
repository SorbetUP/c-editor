import { beforeEach, describe, expect, it } from 'vitest'

import {
  MuyaRustDomRenderer,
  NODE_ATTRIBUTE,
  createDomPatchAdapter
} from '../../../../../Elephant/frontend/src/muya/lib/rust/domRenderer'

const documentNode = (children) => ({
  id: 1,
  parent: null,
  children,
  kind: { layer: 'document' },
  source: null
})

const paragraphNode = (id, children) => ({
  id,
  parent: 1,
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

const markFragmentNode = (id, parent, children, mark, group, edge) => ({
  id,
  parent,
  children,
  kind: {
    layer: 'inline',
    value: { type: 'mark_fragment', mark, group, edge }
  },
  source: null
})

const snapshot = () => ({
  markdown: 'alpha\n\nomega',
  document: {
    root: 1,
    nodes: [
      documentNode([2, 4]),
      paragraphNode(2, [3]),
      textNode(3, 2, 'alpha'),
      paragraphNode(4, [5]),
      textNode(5, 4, 'omega')
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

describe('MuyaRustDomRenderer', () => {
  let container

  beforeEach(() => {
    container = document.createElement('section')
    document.body.replaceChildren(container)
  })

  it('renders a snapshot and applies patches incrementally with stable elements', () => {
    const renderer = new MuyaRustDomRenderer(container)
    renderer.applySnapshot(snapshot())

    const firstParagraph = renderer.element(2)
    const firstText = renderer.element(3)
    expect(firstParagraph.tagName).toBe('P')
    expect(firstText.textContent).toBe('alpha')
    expect(container.querySelectorAll(`[${NODE_ATTRIBUTE}]`)).toHaveLength(4)

    renderer.applyPatches(
      [
        {
          type: 'replace_text',
          node: 3,
          range: { start: 0, end: 5 },
          inserted: 'ALPHA'
        }
      ],
      update(1)
    )
    expect(renderer.element(3)).toBe(firstText)
    expect(firstText.textContent).toBe('ALPHA')

    renderer.applyPatches(
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
    const inserted = renderer.element(6)
    expect(firstParagraph.children[1]).toBe(inserted)

    renderer.applyPatches(
      [{ type: 'move_node', node: 6, new_parent: 4, new_index: 0 }],
      update(3)
    )
    expect(renderer.element(6)).toBe(inserted)
    expect(renderer.element(4).children[0]).toBe(inserted)

    renderer.applyPatches(
      [{ type: 'set_block_kind', node: 4, kind: { type: 'heading', level: 2 } }],
      update(4)
    )
    expect(renderer.element(4).tagName).toBe('H2')
    expect(renderer.element(6)).toBe(inserted)

    renderer.applyPatches(
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
        },
        { type: 'remove_node', node: 5 }
      ],
      update(5)
    )

    expect(renderer.element(7).tagName).toBe('STRONG')
    expect(renderer.element(8).textContent).toBe('bold')
    expect(renderer.element(5)).toBeNull()
    expect(renderer.validateDom()).toBe(true)
  })

  it('renders linked mark fragments as semantic elements', () => {
    const renderer = new MuyaRustDomRenderer(container)
    renderer.applySnapshot({
      ...snapshot(),
      markdown: '**al~~pha** beta *gam~~ma*',
      document: {
        root: 1,
        nodes: [
          documentNode([2]),
          paragraphNode(2, [3, 5, 7]),
          markFragmentNode(3, 2, [4], 'strike', 42, 'start'),
          textNode(4, 3, 'pha'),
          markFragmentNode(5, 2, [6], 'strike', 42, 'middle'),
          textNode(6, 5, ' beta '),
          markFragmentNode(7, 2, [8], 'strike', 42, 'end'),
          textNode(8, 7, 'gam')
        ]
      },
      selection: {
        anchor: { node: 4, offset_utf16: 0 },
        focus: { node: 8, offset_utf16: 3 }
      }
    })

    const fragments = [renderer.element(3), renderer.element(5), renderer.element(7)]
    expect(fragments.map((element) => element.tagName)).toEqual(['DEL', 'DEL', 'DEL'])
    expect(fragments.map((element) => element.dataset.muyaRustMarkGroup)).toEqual([
      '42',
      '42',
      '42'
    ])
    expect(fragments.map((element) => element.dataset.muyaRustMarkEdge)).toEqual([
      'start',
      'middle',
      'end'
    ])
    expect(fragments.map((element) => element.textContent)).toEqual(['pha', ' beta ', 'gam'])
    expect(renderer.validateDom()).toBe(true)
  })

  it('restores a UTF-16 caret inside the generated text node', () => {
    const renderer = new MuyaRustDomRenderer(container)
    renderer.applySnapshot(snapshot())
    renderer.restoreSelection({
      anchor: { node: 3, offset_utf16: 2 },
      focus: { node: 3, offset_utf16: 2 }
    })

    const selection = document.defaultView.getSelection()
    expect(selection.anchorNode.data).toBe('alpha')
    expect(selection.anchorOffset).toBe(2)
    expect(selection.isCollapsed).toBe(true)
  })

  it('exposes bridge callbacks through createDomPatchAdapter', async () => {
    const adapter = createDomPatchAdapter(container)
    await adapter.applySnapshot(snapshot())
    await adapter.applyPatches(
      [
        {
          type: 'replace_text',
          node: 3,
          range: { start: 0, end: 1 },
          inserted: 'A'
        }
      ],
      update(1)
    )
    await adapter.onSelection({
      anchor: { node: 3, offset_utf16: 1 },
      focus: { node: 3, offset_utf16: 1 }
    })

    expect(adapter.renderer.element(3).textContent).toBe('Alpha')
    expect(document.defaultView.getSelection().anchorOffset).toBe(1)
  })
})
