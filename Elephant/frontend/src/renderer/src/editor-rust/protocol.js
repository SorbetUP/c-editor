export const EDITOR_PROTOCOL_VERSION = 1

export const createEditorRequest = (expectedRevision, command) => ({
  protocol_version: EDITOR_PROTOCOL_VERSION,
  expected_revision: expectedRevision,
  command
})

export const parseEditorResponse = (rawResponse) => {
  const response = typeof rawResponse === 'string' ? JSON.parse(rawResponse) : rawResponse
  if (!response || typeof response !== 'object') {
    throw new TypeError('Elephant Rust returned an invalid response.')
  }
  if (!['snapshot', 'update', 'error'].includes(response.type)) {
    throw new TypeError(`Elephant Rust returned an unknown response type: ${String(response.type)}`)
  }
  if (!response.payload || typeof response.payload !== 'object') {
    throw new TypeError(`Elephant Rust response ${response.type} has no payload.`)
  }
  return response
}

export const editorCommands = Object.freeze({
  snapshot: () => ({ type: 'snapshot' }),
  setSelection: (selection) => ({ type: 'set_selection', selection }),
  insertText: (text) => ({ type: 'insert_text', text }),
  pasteMarkdown: (markdown) => ({ type: 'paste_markdown', markdown }),
  insertParagraph: () => ({ type: 'insert_paragraph' }),
  deleteBackward: () => ({ type: 'delete_backward' }),
  setParagraph: () => ({ type: 'set_paragraph' }),
  setHeading: (level) => ({ type: 'set_heading', level }),
  toggleStrong: () => ({ type: 'toggle_strong' }),
  toggleEmphasis: () => ({ type: 'toggle_emphasis' }),
  toggleStrike: () => ({ type: 'toggle_strike' }),
  duplicateBlock: () => ({ type: 'duplicate_block' }),
  deleteBlock: () => ({ type: 'delete_block' }),
  insertParagraphAfterBlock: () => ({ type: 'insert_paragraph_after_block' }),
  toggleBlockQuote: () => ({ type: 'toggle_block_quote' }),
  toggleCodeBlock: () => ({ type: 'toggle_code_block' }),
  setListKind: (kind) => ({ type: 'set_list_kind', kind }),
  insertHorizontalRule: () => ({ type: 'insert_horizontal_rule' }),
  createTable: (rows, columns) => ({ type: 'create_table', rows, columns }),
  insertImage: (source, alt = '', title = null) => ({
    type: 'insert_image',
    source,
    alt,
    title
  }),
  replaceImage: (image, source, alt = '', title = null) => ({
    type: 'replace_image',
    image,
    source,
    alt,
    title
  }),
  deleteImage: (image) => ({ type: 'delete_image', image }),
  indentListItem: () => ({ type: 'indent_list_item' }),
  outdentListItem: () => ({ type: 'outdent_list_item' }),
  setTaskChecked: (item, checked, autoCheck = false) => ({
    type: 'set_task_checked',
    item,
    checked,
    auto_check: autoCheck
  }),
  insertTableRowAfter: () => ({ type: 'insert_table_row_after' }),
  deleteTableRow: () => ({ type: 'delete_table_row' }),
  insertTableColumnAfter: () => ({ type: 'insert_table_column_after' }),
  deleteTableColumn: () => ({ type: 'delete_table_column' }),
  nextTableCell: () => ({ type: 'next_table_cell' }),
  previousTableCell: () => ({ type: 'previous_table_cell' }),
  beginComposition: () => ({ type: 'begin_composition' }),
  updateComposition: (text) => ({ type: 'update_composition', text }),
  commitComposition: () => ({ type: 'commit_composition' }),
  cancelComposition: () => ({ type: 'cancel_composition' }),
  undo: () => ({ type: 'undo' }),
  redo: () => ({ type: 'redo' })
})
