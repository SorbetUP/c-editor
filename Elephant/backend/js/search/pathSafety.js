import path from 'path'

export const IGNORED_PATH_SEGMENTS = Object.freeze([
  '.elephantnote',
  '.git',
  'node_modules',
  'dist',
  'build',
  '.cache'
])

export const assertPathInsideVault = (vaultRoot, targetPath) => {
  const root = path.resolve(vaultRoot || '')
  const target = path.resolve(targetPath || '')
  const relative = path.relative(root, target)

  if (!root || !target || relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error('Path must stay inside the active vault.')
  }
}

export const isIgnoredPath = (relativePath) => {
  if (!relativePath || typeof relativePath !== 'string') return false

  const normalized = relativePath.replace(/\\/g, '/')
  const segments = normalized.split('/').filter(Boolean)

  return segments.some((segment) => IGNORED_PATH_SEGMENTS.includes(segment))
}

export const isMarkdownFile = (filePath) => {
  if (!filePath || typeof filePath !== 'string') return false
  const ext = path.extname(filePath).toLowerCase()
  return ext === '.md' || ext === '.markdown'
}
