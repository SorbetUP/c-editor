import fs from 'fs-extra'
import path from 'path'

export const IGNORED_SITE_SEGMENTS = Object.freeze([
  '.elephantnote',
  '.git',
  'node_modules',
  'dist',
  'build',
  '.cache'
])

export const ALLOWED_SITE_EXTENSIONS = Object.freeze([
  '.md',
  '.markdown',
  '.png',
  '.jpg',
  '.jpeg',
  '.webp',
  '.gif',
  '.svg',
  '.css',
  '.js',
  '.json'
])

const hasPathValue = (value) => typeof value === 'string' && value.trim().length > 0

export const assertPathInsideVault = (vaultRoot, targetPath) => {
  if (!hasPathValue(vaultRoot) || !hasPathValue(targetPath)) {
    throw new Error('The selected folder is outside the active vault.')
  }

  const root = path.resolve(vaultRoot)
  const target = path.resolve(targetPath)
  const relative = path.relative(root, target)

  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error('The selected folder is outside the active vault.')
  }
}

export const isIgnoredForSite = (relativePath = '') => {
  if (typeof relativePath !== 'string') return false
  const segments = relativePath.replace(/\\/g, '/').split('/').filter(Boolean)
  return segments.some((segment) => IGNORED_SITE_SEGMENTS.includes(segment))
}

export const isAllowedSiteFile = (filePath = '') => {
  return ALLOWED_SITE_EXTENSIONS.includes(path.extname(filePath).toLowerCase())
}

const hasMarkdownNotes = async(sourceFolder, vaultRoot, currentDir = sourceFolder) => {
  const dirents = await fs.readdir(currentDir, { withFileTypes: true })

  for (const dirent of dirents) {
    const fullPath = path.join(currentDir, dirent.name)
    const relativePath = path.relative(vaultRoot, fullPath)
    if (isIgnoredForSite(relativePath)) continue

    if (dirent.isDirectory()) {
      if (await hasMarkdownNotes(sourceFolder, vaultRoot, fullPath)) return true
    } else if (dirent.isFile()) {
      const ext = path.extname(dirent.name).toLowerCase()
      if (ext === '.md' || ext === '.markdown') return true
    }
  }

  return false
}

export const isValidSiteSourceFolder = async(vaultRoot, folderPath) => {
  assertPathInsideVault(vaultRoot, folderPath)
  const relativePath = path.relative(path.resolve(vaultRoot), path.resolve(folderPath))
  if (isIgnoredForSite(relativePath)) return false
  const stats = await fs.stat(folderPath).catch(() => null)
  if (!stats?.isDirectory()) return false
  return hasMarkdownNotes(folderPath, vaultRoot)
}
