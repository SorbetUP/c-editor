import { describe, expect, it } from 'vitest'

import {
  rustBusCommand,
  rustParagraphCommand
} from '../../../../../Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditorCommands'

describe('Elephant Rust editor command routing', () => {
  it.each([
    ['blockquote', { type: 'toggle_block_quote' }],
    ['pre', { type: 'toggle_code_block' }],
    ['ul-bullet', { type: 'set_list_kind', kind: 'unordered' }],
    ['ul-task', { type: 'set_list_kind', kind: 'task' }],
    ['ol-order', { type: 'set_list_kind', kind: 'ordered' }]
  ])('routes paragraph menu %s to Rust', (type, command) => {
    expect(rustParagraphCommand(type)).toEqual(command)
    expect(rustBusCommand('paragraph', type)).toEqual(command)
  })

  it('keeps paragraph, heading and horizontal rule commands on Rust', () => {
    expect(rustParagraphCommand('paragraph')).toEqual({ type: 'set_paragraph' })
    expect(rustParagraphCommand('heading 4')).toEqual({
      type: 'set_heading',
      level: 4
    })
    expect(rustBusCommand('insert-horizontal-rule')).toEqual({
      type: 'insert_horizontal_rule'
    })
  })

  it('preserves the selected table dimensions', () => {
    expect(rustBusCommand('createTable', { rows: 4, columns: 3 })).toEqual({
      type: 'create_table',
      rows: 4,
      columns: 3
    })
  })

  it('preserves image insertion, replacement and deletion payloads', () => {
    expect(
      rustBusCommand('insert-image', {
        source: 'assets/image.png',
        alt: 'diagram',
        title: 'Architecture'
      })
    ).toEqual({
      type: 'insert_image',
      source: 'assets/image.png',
      alt: 'diagram',
      title: 'Architecture'
    })
    expect(rustBusCommand('insert-image', 'image.png')).toEqual({
      type: 'insert_image',
      source: 'image.png',
      alt: '',
      title: null
    })
    expect(rustBusCommand('replace-image', {
      image: 42,
      source: 'new.png',
      alt: 'new',
      title: 'Replacement'
    })).toEqual({
      type: 'replace_image',
      image: 42,
      source: 'new.png',
      alt: 'new',
      title: 'Replacement'
    })
    expect(rustBusCommand('delete-image', { image: 42 })).toEqual({
      type: 'delete_image',
      image: 42
    })
  })

  it('does not invent a fallback for unsupported menu entries', () => {
    expect(rustParagraphCommand('table')).toBeNull()
    expect(rustBusCommand('unknown')).toBeNull()
  })
})
