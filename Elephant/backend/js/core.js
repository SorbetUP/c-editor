import path from 'path'
export {
  WORKSPACE_DIR,
  WORKSPACE_FILE,
  INDEX_FILE,
  CALENDAR_FILE,
  SOURCES_FILE,
  WIKI_FILE,
  createId,
  createWorkspace,
  createWelcomeMarkdown,
  isIgnoredVaultEntry,
  isPathInsideRelativePath,
  nextAvailableName,
  normalizeRelativePath,
  normalizeWorkspaceSidebar,
  parseMarkdownMeta
} from 'common/elephantnote/workspace'

import { normalizeRelativePath } from 'common/elephantnote/workspace'

export const resolveInsideVault = (vaultRoot, relativePath = '') => {
  const root = path.resolve(vaultRoot)
  const target = path.resolve(root, normalizeRelativePath(relativePath))
  const relative = path.relative(root, target)
  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error('Path must stay inside the active vault.')
  }
  return target
}
