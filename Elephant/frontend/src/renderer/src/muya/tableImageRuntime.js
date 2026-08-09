import { markdownToJsonState, jsonStateToMarkdown } from './jsonStateRuntime.js'

export const tableCommand = (markdown, command, index = 0) => {
  const state = markdownToJsonState(markdown)
  const table = state.blocks.find((block) => block.type === 'table')
  if (!table) return markdown
  if (command === 'insert_row') table.rows.splice(index, 0, table.headers.map(() => ''))
  if (command === 'delete_row') table.rows.splice(index, 1)
  if (command === 'insert_column') {
    table.headers.splice(index, 0, '')
    table.alignments.splice(index, 0, 'default')
    table.rows.forEach((row) => row.splice(index, 0, ''))
  }
  if (command === 'delete_column') {
    table.headers.splice(index, 1)
    table.alignments.splice(index, 1)
    table.rows.forEach((row) => row.splice(index, 1))
  }
  if (command === 'align_left') table.alignments[index] = 'left'
  if (command === 'align_center') table.alignments[index] = 'center'
  if (command === 'align_right') table.alignments[index] = 'right'
  return jsonStateToMarkdown(state)
}

export const imageAtCursor = (markdown, cursor) => {
  const pattern = /!\[([^\]]*)\]\(([^)]+)\)(?:\{width=([^}]+)\})?/g
  for (const match of markdown.matchAll(pattern)) {
    const start = match.index
    const end = start + match[0].length
    if (cursor >= start && cursor <= end) return { start, end, alt: match[1], url: match[2], width: match[3] || null }
  }
  return null
}

export const imageToolbarState = (markdown, cursor) => {
  const image = imageAtCursor(markdown, cursor)
  if (!image) return { visible: false }
  return { visible: true, ...image, actions: ['resize', 'replace', 'caption', 'copy', 'delete'], sizes: ['25%', '50%', '75%', '100%'] }
}

export const resizeImageMarkdown = (markdown, cursor, width) => {
  const image = imageAtCursor(markdown, cursor)
  if (!image) return markdown
  const replacement = `![${image.alt}](${image.url}){width=${width}}`
  return markdown.slice(0, image.start) + replacement + markdown.slice(image.end)
}
