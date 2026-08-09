import { formatShortDate } from '../services/markdownMetaService'

const cardTitleFromName = (entry) => String(entry?.name || entry?.filename || '').replace(/\.md$/i, '')

const FRONTMATTER_KEYS = new Set([
  'title',
  'type',
  'tags',
  'createdAt',
  'updatedAt',
  'created',
  'updated',
  'id'
])

const FRONTMATTER_BLOCK_PATTERN = /^---[ \t]*\r?\n[\s\S]*?\r?\n[ \t]*---[ \t]*(?:\r?\n|$)/
const INLINE_FRONTMATTER_PAIR_PATTERN = new RegExp(
  `(?:^|\\s)(?:${Array.from(FRONTMATTER_KEYS).join('|')}):\\s*(?:"[^"]*"|'[^']*'|\\[[^\\]]*\\]|[^\\s]+)`,
  'gi'
)

const looksLikeInlineFrontmatter = (line = '') => {
  const value = String(line || '').trim()
  if (!value.startsWith('---')) return false
  const metadata = value.replace(/^---\s*/, '').trim()
  if (!metadata) return true
  const keyMatch = metadata.match(/^([A-Za-z][\w-]*):\s*/)
  return !!keyMatch && FRONTMATTER_KEYS.has(keyMatch[1])
}

const stripInlineFrontmatterPrefix = (value = '') => {
  const raw = String(value || '').trim()
  if (!looksLikeInlineFrontmatter(raw)) return raw

  const closedInline = raw.match(/^---\s+.*?\s+---\s*(.*)$/)
  if (closedInline) return closedInline[1].trim()

  INLINE_FRONTMATTER_PAIR_PATTERN.lastIndex = 0
  let lastMatch = null
  let match = INLINE_FRONTMATTER_PAIR_PATTERN.exec(raw)
  while (match) {
    lastMatch = match
    match = INLINE_FRONTMATTER_PAIR_PATTERN.exec(raw)
  }
  if (!lastMatch) return raw.replace(/^---\s*/, '').trim()

  const end = Number(lastMatch.index || 0) + lastMatch[0].length
  return raw.slice(end).replace(/^\s*---\s*/, '').trim()
}

const stripFrontmatter = (value = '') => {
  const raw = String(value || '').trim()
  if (!raw) return ''
  const block = raw.match(FRONTMATTER_BLOCK_PATTERN)
  if (block) return raw.slice(block[0].length).trim()
  return stripInlineFrontmatterPrefix(raw)
}

const stripLeadingDocumentTitle = (value = '') => {
  const raw = String(value || '').trim()
  if (!raw) return ''

  const firstLineEnd = raw.indexOf('\n')
  const firstLine = (firstLineEnd < 0 ? raw : raw.slice(0, firstLineEnd)).replace(/\r$/, '')
  if (/^#\s+\S/.test(firstLine)) return firstLineEnd < 0 ? '' : raw.slice(firstLineEnd + 1).trim()

  return raw.replace(/^#{1,6}\s+/, '').trim()
}

const cleanPreview = (value) => stripLeadingDocumentTitle(stripFrontmatter(value))

export const getNoteCardTitle = (entry) => entry?.title?.trim() || cardTitleFromName(entry) || 'Untitled'

export const getNoteCardTypeLabel = (entry) => entry?.type?.trim() || 'Note'

export const getNoteCardUpdatedLabel = (entry) => formatShortDate(entry?.updatedAt)

export const getNoteCardExcerpt = (entry) => cleanPreview(entry?.excerpt || entry?.markdown || entry?.content) || 'No preview yet.'
