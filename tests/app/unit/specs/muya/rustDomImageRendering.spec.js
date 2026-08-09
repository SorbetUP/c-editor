import { beforeEach, describe, expect, it } from 'vitest'

import { MuyaRustDomRenderer } from '../../../../../Elephant/frontend/src/muya/lib/rust/domRenderer'

const image = (id, source) => ({
  id,
  parent: 2,
  children: [],
  kind: {
    layer: 'inline',
    value: { type: 'image', source, alt: `image-${id}`, title: `title-${id}` }
  },
  source: null
})

const snapshot = () => ({
  markdown: '',
  document: {
    root: 1,
    nodes: [
      { id: 1, parent: null, children: [2], kind: { layer: 'document' }, source: null },
      {
        id: 2,
        parent: 1,
        children: [3, 4, 5],
        kind: { layer: 'block', value: { type: 'paragraph' } },
        source: null
      },
      image(3, 'assets/diagram.png'),
      image(4, 'data:image/png;base64,AAAA'),
      image(5, 'javascript:alert(1)')
    ]
  },
  revision: 0,
  selection: null,
  can_undo: false,
  can_redo: false,
  composition_active: false
})

describe('Muya Rust DOM image rendering', () => {
  let container
  let renderer

  beforeEach(() => {
    container = document.createElement('section')
    document.body.replaceChildren(container)
    renderer = new MuyaRustDomRenderer(container)
    renderer.applySnapshot(snapshot())
  })

  it('renders relative and embedded image sources with semantic attributes', () => {
    expect(renderer.element(3).getAttribute('src')).toBe('assets/diagram.png')
    expect(renderer.element(4).getAttribute('src')).toBe('data:image/png;base64,AAAA')
    expect(renderer.element(3).getAttribute('alt')).toBe('image-3')
    expect(renderer.element(3).getAttribute('title')).toBe('title-3')
  })

  it('keeps dangerous schemes out of the live src attribute', () => {
    expect(renderer.element(5).hasAttribute('src')).toBe(false)
    expect(renderer.element(5).getAttribute('data-source')).toBe('javascript:alert(1)')
  })
})
