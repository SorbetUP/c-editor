import {
  ensureNoteDocument,
  getDocumentCreatedAt,
  getDocumentTitle,
  getEditorMarkdownStats,
  mergeEditorMarkdown as mergeBaseEditorMarkdown,
  parseFrontmatter,
  renameDocumentTitle,
  serializeFrontmatterValue,
  stripDisplayedTitle,
  toEditorMarkdown as toBaseEditorMarkdown
} from 'common/elephantnote/markdownDocument'

const EMPTY_TRAILING_TASK_RE = /(^|\n)([ \t]*[-+*][ \t]+\[[ xX]\])[ \t]*$/

const preserveEmptyTrailingTask = (markdown = '') => {
  const value = String(markdown || '')
  if (!EMPTY_TRAILING_TASK_RE.test(value)) return value
  return `${value.replace(/[ \t]+$/, '')} `
}

const toEditorMarkdown = (markdown = '', fallbackTitle = 'Untitled') =>
  toBaseEditorMarkdown(preserveEmptyTrailingTask(markdown), fallbackTitle)

const mergeEditorMarkdown = (currentDocument = '', editorMarkdown = '', fallbackTitle = 'Untitled') => {
  const preserveTrailingSpace = EMPTY_TRAILING_TASK_RE.test(String(editorMarkdown || ''))
  const merged = mergeBaseEditorMarkdown(currentDocument, editorMarkdown, fallbackTitle)
  return preserveTrailingSpace ? preserveEmptyTrailingTask(merged) : merged
}

export {
  ensureNoteDocument,
  getDocumentCreatedAt,
  getDocumentTitle,
  getEditorMarkdownStats,
  mergeEditorMarkdown,
  parseFrontmatter,
  renameDocumentTitle,
  serializeFrontmatterValue,
  stripDisplayedTitle,
  toEditorMarkdown
}
