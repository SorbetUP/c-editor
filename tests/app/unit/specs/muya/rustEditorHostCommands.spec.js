import { describe, expect, it } from 'vitest'

import {
  rustBusCommand,
  rustFormatCommand,
  rustParagraphCommand
} from '../../../../../Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditorCommands'

describe('Rust editor host commands', () => {
  it.each([
    ['strong', { type: 'toggle_strong' }],
    ['bold', { type: 'toggle_strong' }],
    ['emphasis', { type: 'toggle_emphasis' }],
    ['italic', { type: 'toggle_emphasis' }],
    ['strike', { type: 'toggle_strike' }],
    ['strikethrough', { type: 'toggle_strike' }]
  ])('maps format %s to the Rust protocol', (format, expected) => {
    expect(rustFormatCommand(format)).toEqual(expected)
  })

  it.each([
    ['paragraph', { type: 'set_paragraph' }],
    ['p', { type: 'set_paragraph' }],
    ['heading 1', { type: 'set_heading', level: 1 }],
    ['heading-4', { type: 'set_heading', level: 4 }],
    ['heading_6', { type: 'set_heading', level: 6 }]
  ])('maps paragraph style %s to the Rust protocol', (paragraph, expected) => {
    expect(rustParagraphCommand(paragraph)).toEqual(expected)
  })

  it('maps global history and paragraph menu commands', () => {
    expect(rustBusCommand('undo')).toEqual({ type: 'undo' })
    expect(rustBusCommand('redo')).toEqual({ type: 'redo' })
    expect(rustBusCommand('insertParagraph')).toEqual({ type: 'insert_paragraph' })
    expect(rustBusCommand('createParagraph')).toEqual({
      type: 'insert_paragraph_after_block'
    })
    expect(rustBusCommand('duplicate')).toEqual({ type: 'duplicate_block' })
    expect(rustBusCommand('deleteParagraph')).toEqual({ type: 'delete_block' })
  })

  it('does not invent Rust commands for unsupported menu actions', () => {
    expect(rustFormatCommand('code')).toBeNull()
    expect(rustParagraphCommand('table')).toBeNull()
    expect(rustBusCommand('insertImage')).toBeNull()
  })
})
