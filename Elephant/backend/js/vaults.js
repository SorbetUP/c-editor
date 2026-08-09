import fs from 'fs-extra'
import path from 'path'
import { BrowserWindow, dialog, ipcMain } from 'electron'
import log from 'electron-log'
import {
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
  nextAvailableName,
  normalizeRelativePath,
  normalizeWorkspaceSidebar,
  parseMarkdownMeta,
  resolveInsideVault
} from './core'
import { importGoogleKeepExport } from './googleKeepImport'
import { mergeCalendarEvents, parseIcsCalendar } from 'common/elephantnote/calendar'
import {
  calendarEventToGoogleEvent,
  googleEventToCalendarEvent,
  normalizeGoogleCalendarConfig
} from 'common/elephantnote/googleCalendar'
import {
  createSourceId,
  extractHtmlTitle,
  htmlToReadableText,
  normalizeSourceRecord,
  normalizeSourceUrl,
  parseRssFeed
} from 'common/elephantnote/sources'
import { mergeWikiProposals, normalizeWikiRecord } from 'common/elephantnote/wiki'
import {
  buildWikiProposalsFromGraph,
  buildWikiChatContext,
  buildWikiSourceInsight,
  createGraphBackedWikiMarkdown
} from './wiki/wikiLibrary'
import { buildRagChatPrompt } from './chat/ragPrompt'
import { getSearchService, registerSearchIpc } from './search/searchIpc'
import { getSitePreviewService, registerSitePreviewIpc } from './sitePreview/sitePreviewIpc'
import {
  listAgents,
  registerAgent,
  registerElephantNoteAgentIpc,
  sendAgentMessage,
  unregisterAgent
} from './agents'
import { createElephantNoteApi, ELEPHANTNOTE_API_ACTIONS, registerElephantNoteApiIpc } from './api'
import { registerCompatibilityElephantNoteIpc } from './ipc/compatibilityElephantNoteIpc'
import { registerModelLibraryIpc } from './ipc/modelLibraryIpc'
import { updateMarkdownTitle } from './markdown'
import { migrateWorkspace } from './workspaceMigrations'
import { normalizeProgramEnvironments } from './programRuntime'
import { setFeatureFlag } from 'common/elephantnote/featureFlags'
import { normalizeVaultIcon } from 'common/elephantnote/appearance'
import {
  createAiRequestBody,
  extractAiResponseText,
  normalizeAiConfig,
  resolveAiEndpoint
} from 'common/elephantnote/aiProviders'
import {
  ATOMIC_AI_FEATURES,
  ATOMIC_MODEL_CATALOG,
  ATOMIC_PLUGIN_MANIFESTS,
  PROGRAMMATIC_TASK_TEMPLATES,
  EXTENSION_TASK_ACTIONS,
  EXTENSION_PLUGIN_RUNTIMES,
  createTaskRunResult,
  createTaskStepResult,
  createDefaultModelSelection,
  mergePluginState,
  mergeTaskState,
  resolvePluginRuntime,
  updatePluginState,
  updateTaskState
} from 'common/elephantnote/atomicWorkspace'
import { ensureAtomicSourcesSection, suggestAtomicTags } from 'common/elephantnote/atomicAiEngine'
import { getConfig, setConfig } from './config/elephantConfigStore'
import { modelRuntime, programRuntime, syncEngine } from './runtime/elephantRuntime'

export const initializeVault = async (vaultRoot, now = new Date()) => {
  log.info('[sync] initializeVault', { vaultRoot })
  const root = path.resolve(vaultRoot)
  const metaDir = path.join(root, WORKSPACE_DIR)
  const workspacePath = path.join(metaDir, WORKSPACE_FILE)
  const indexPath = path.join(metaDir, INDEX_FILE)
  const gettingStartedDir = path.join(root, 'Getting Started')
  const welcomePath = path.join(gettingStartedDir, 'Welcome.md')
  const compatibilityWelcomePath = path.join(gettingStartedDir, 'Welcome to ElephantNote.md')

  await fs.ensureDir(metaDir)
  await fs.ensureDir(gettingStartedDir)

  if (!(await fs.pathExists(workspacePath))) {
    await fs.writeJson(workspacePath, createWorkspace(root), { spaces: 2 })
  }
  if (!(await fs.pathExists(indexPath))) {
    await fs.writeJson(
      indexPath,
      { version: 1, updatedAt: now.toISOString(), entries: [] },
      { spaces: 2 }
    )
  }
  const calendarPath = path.join(metaDir, CALENDAR_FILE)
  if (!(await fs.pathExists(calendarPath))) {
    await fs.writeJson(
      calendarPath,
      { version: 1, updatedAt: now.toISOString(), events: [] },
      { spaces: 2 }
    )
  }
  const sourcesPath = path.join(metaDir, SOURCES_FILE)
  if (!(await fs.pathExists(sourcesPath))) {
    await fs.writeJson(
      sourcesPath,
      { version: 1, updatedAt: now.toISOString(), sources: [] },
      { spaces: 2 }
    )
  }
  const wikiPath = path.join(metaDir, WIKI_FILE)
  if (!(await fs.pathExists(wikiPath))) {
    await fs.writeJson(
      wikiPath,
      { version: 1, updatedAt: now.toISOString(), records: [] },
      { spaces: 2 }
    )
  }
  if (!(await fs.pathExists(welcomePath))) {
    if (await fs.pathExists(compatibilityWelcomePath)) {
      await fs.move(compatibilityWelcomePath, welcomePath)
    } else {
      await fs.writeFile(welcomePath, createWelcomeMarkdown(now), 'utf8')
    }
  }

  const workspace = await readWorkspace(root)
  const gettingStarted = workspace.sidebar?.find((entry) => entry.id === 'getting-started')
  if (gettingStarted) {
    gettingStarted.title = 'Getting started'
    gettingStarted.type = 'folder'
    gettingStarted.path = 'Getting Started'
    delete gettingStarted.items
    await writeWorkspace(root, workspace)
  }

  return workspace
}

const readWorkspace = async (vaultRoot) => {
  const workspacePath = path.join(vaultRoot, WORKSPACE_DIR, WORKSPACE_FILE)
  if (!(await fs.pathExists(workspacePath))) {
    return createWorkspace(vaultRoot)
  }
  const rawWorkspace = await fs.readJson(workspacePath)
  const migratedWorkspace = migrateWorkspace(rawWorkspace)
  if (JSON.stringify(migratedWorkspace) !== JSON.stringify(rawWorkspace)) {
    await writeWorkspace(vaultRoot, migratedWorkspace)
  }
  const workspace = migratedWorkspace
  const normalizedWorkspace = normalizeWorkspaceSidebar(workspace)
  if (
    JSON.stringify(normalizedWorkspace.sidebar || []) !== JSON.stringify(workspace.sidebar || [])
  ) {
    await writeWorkspace(vaultRoot, normalizedWorkspace)
  }
  return normalizedWorkspace
}

const writeWorkspace = async (vaultRoot, workspace) => {
  const workspacePath = path.join(vaultRoot, WORKSPACE_DIR, WORKSPACE_FILE)
  await fs.writeJson(workspacePath, workspace, { spaces: 2 })
}

const readCalendar = async (vaultRoot) => {
  const calendarPath = path.join(vaultRoot, WORKSPACE_DIR, CALENDAR_FILE)
  if (!(await fs.pathExists(calendarPath))) {
    return { version: 1, updatedAt: new Date().toISOString(), events: [] }
  }
  const calendar = await fs.readJson(calendarPath)
  return {
    version: 1,
    updatedAt: calendar.updatedAt || '',
    events: Array.isArray(calendar.events) ? calendar.events : []
  }
}

const writeCalendar = async (vaultRoot, calendar) => {
  const calendarPath = path.join(vaultRoot, WORKSPACE_DIR, CALENDAR_FILE)
  await fs.ensureDir(path.dirname(calendarPath))
  await fs.writeJson(
    calendarPath,
    {
      version: 1,
      updatedAt: new Date().toISOString(),
      events: calendar.events || []
    },
    { spaces: 2 }
  )
}

const readSources = async (vaultRoot) => {
  const sourcesPath = path.join(vaultRoot, WORKSPACE_DIR, SOURCES_FILE)
  if (!(await fs.pathExists(sourcesPath))) {
    return { version: 1, updatedAt: new Date().toISOString(), sources: [] }
  }
  const data = await fs.readJson(sourcesPath)
  return {
    version: 1,
    updatedAt: data.updatedAt || '',
    sources: Array.isArray(data.sources) ? data.sources.map(normalizeSourceRecord) : []
  }
}

const writeSources = async (vaultRoot, sources) => {
  const sourcesPath = path.join(vaultRoot, WORKSPACE_DIR, SOURCES_FILE)
  await fs.ensureDir(path.dirname(sourcesPath))
  await fs.writeJson(
    sourcesPath,
    {
      version: 1,
      updatedAt: new Date().toISOString(),
      sources: sources.map(normalizeSourceRecord)
    },
    { spaces: 2 }
  )
}

const readWiki = async (vaultRoot) => {
  const wikiPath = path.join(vaultRoot, WORKSPACE_DIR, WIKI_FILE)
  if (!(await fs.pathExists(wikiPath))) {
    return { version: 1, updatedAt: new Date().toISOString(), records: [] }
  }
  const data = await fs.readJson(wikiPath)
  return {
    version: 1,
    updatedAt: data.updatedAt || '',
    records: Array.isArray(data.records) ? data.records.map(normalizeWikiRecord) : []
  }
}

const writeWiki = async (vaultRoot, records) => {
  const wikiPath = path.join(vaultRoot, WORKSPACE_DIR, WIKI_FILE)
  await fs.ensureDir(path.dirname(wikiPath))
  await fs.writeJson(
    wikiPath,
    {
      version: 1,
      updatedAt: new Date().toISOString(),
      records: records.map(normalizeWikiRecord)
    },
    { spaces: 2 }
  )
}

const getActiveVault = () => {
  const config = getConfig()
  const vault = config.vaults.find((vault) => vault.id === config.activeVaultId) || null
  syncEngine.setCwd(vault?.path || '')
  return vault
}

const loadVaultPayload = async (vault) => {
  log.info('[sync] loadVaultPayload', { vaultPath: vault?.path || '' })
  if (!vault) {
    return { ...getConfig(), activeVault: null, workspace: null, entries: [] }
  }
  await initializeVault(vault.path)
  const workspace = await readWorkspace(vault.path)
  const entries = await listDirectoryForVault(vault, '')
  return { ...getConfig(), activeVault: vault, workspace, entries }
}

const upsertVault = (vaultPath) => {
  log.info('[sync] upsertVault', { vaultPath })
  const absolutePath = path.resolve(vaultPath)
  const config = getConfig()
  const vaultName = path.basename(absolutePath) || 'Personal'
  const existing = config.vaults.find((vault) => vault.path === absolutePath)
  const vault = existing || {
    id: createId(vaultName),
    name: vaultName,
    path: absolutePath,
    icon: '',
    lastOpenedAt: new Date().toISOString()
  }

  vault.lastOpenedAt = new Date().toISOString()
  if (!existing) {
    let nextId = vault.id
    let suffix = 2
    while (config.vaults.some((item) => item.id === nextId)) {
      nextId = `${vault.id}-${suffix}`
      suffix += 1
    }
    vault.id = nextId
    config.vaults.push(vault)
  }
  config.activeVaultId = vault.id
  setConfig(config)
  return vault
}

const setVaultIcon = (vaultId, icon = '') => {
  const config = getConfig()
  const vault = config.vaults.find((item) => item.id === vaultId)
  if (!vault) throw new Error('Unknown ElephantNote vault.')
  vault.icon = normalizeVaultIcon(icon)
  setConfig(config)
  return loadVaultPayload(vault)
}

const setVaultName = (vaultId, name = '') => {
  const nextName = String(name || '').trim()
  if (!nextName) throw new Error('Vault name cannot be empty.')
  const config = getConfig()
  const vault = config.vaults.find((item) => item.id === vaultId)
  if (!vault) throw new Error('Unknown ElephantNote vault.')
  vault.name = nextName
  setConfig(config)
  return loadVaultPayload(vault)
}

const removeVault = (vaultId) => {
  const config = getConfig()
  const vault = config.vaults.find((item) => item.id === vaultId)
  if (!vault) throw new Error('Unknown ElephantNote vault.')
  config.vaults = config.vaults.filter((item) => item.id !== vaultId)
  if (config.activeVaultId === vaultId) {
    config.activeVaultId = config.vaults[0]?.id || null
  }
  setConfig(config)
  return loadVaultPayload(getActiveVault())
}

const listDirectoryForVault = async (vault, relativePath = '') => {
  const directory = resolveInsideVault(vault.path, relativePath)
  const dirents = await fs.readdir(directory, { withFileTypes: true })
  const entries = []

  for (const dirent of dirents) {
    if (isIgnoredVaultEntry(dirent.name)) continue
    const fullPath = path.join(directory, dirent.name)
    const relative = path.relative(vault.path, fullPath)
    const stats = await fs.stat(fullPath)

    if (dirent.isDirectory()) {
      const childNames = await fs.readdir(fullPath)
      entries.push({
        kind: 'folder',
        title: dirent.name,
        path: relative,
        noteCount: childNames.filter((name) => name.toLowerCase().endsWith('.md')).length,
        updatedAt: stats.mtime.toISOString()
      })
    } else if (dirent.isFile() && dirent.name.toLowerCase().endsWith('.md')) {
      const markdown = await fs.readFile(fullPath, 'utf8')
      entries.push({
        kind: 'note',
        path: relative,
        filename: dirent.name,
        updatedAt: stats.mtime.toISOString(),
        ...parseMarkdownMeta(markdown, dirent.name)
      })
    }
  }

  return entries.sort((a, b) => new Date(b.updatedAt) - new Date(a.updatedAt))
}

const listMarkdownNotesRecursive = async (vault, relativePath = '') => {
  const directory = resolveInsideVault(vault.path, relativePath)
  const dirents = await fs.readdir(directory, { withFileTypes: true })
  const notes = []

  for (const dirent of dirents) {
    if (isIgnoredVaultEntry(dirent.name)) continue
    const fullPath = path.join(directory, dirent.name)
    const relative = path.relative(vault.path, fullPath)
    if (dirent.isDirectory()) {
      notes.push(...(await listMarkdownNotesRecursive(vault, relative)))
    } else if (dirent.isFile() && dirent.name.toLowerCase().endsWith('.md')) {
      const stats = await fs.stat(fullPath)
      const markdown = await fs.readFile(fullPath, 'utf8')
      notes.push({
        kind: 'note',
        path: relative,
        filename: dirent.name,
        updatedAt: stats.mtime.toISOString(),
        ...parseMarkdownMeta(markdown, dirent.name)
      })
    }
  }

  return notes.sort((a, b) => new Date(b.updatedAt) - new Date(a.updatedAt))
}

const createNote = async ({ relativePath = '', filename = '', title = '' } = {}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const directory = resolveInsideVault(vault.path, relativePath)
  await fs.ensureDir(directory)
  const desiredFilename = String(filename || '').trim()
  const nextFilename = desiredFilename
    ? desiredFilename
    : nextAvailableName('Untitled.md', (name) => fs.existsSync(path.join(directory, name)))
  const fullPath = path.join(directory, nextFilename)
  const now = new Date().toISOString()
  const exists = await fs.pathExists(fullPath)
  if (exists) {
    const markdown = await fs.readFile(fullPath, 'utf8').catch(() => '')
    const meta = parseMarkdownMeta(markdown, nextFilename)
    return {
      note: {
        path: path.relative(vault.path, fullPath),
        fullPath,
        title: meta.title || title || path.basename(nextFilename, path.extname(nextFilename))
      },
      entries: await listDirectoryForVault(vault, relativePath)
    }
  }
  await fs.writeFile(
    fullPath,
    `---\ntitle: "${String(title || 'Untitled').replace(/"/g, '\\"')}"\ntype: "note"\ntags: []\ncreatedAt: "${now}"\nupdatedAt: "${now}"\n---\n\n# ${String(title || 'Untitled')}\n`,
    'utf8'
  )
  getSearchService()
    .indexFile(fullPath)
    .catch(() => {})
  return {
    note: { path: path.relative(vault.path, fullPath), fullPath },
    entries: await listDirectoryForVault(vault, relativePath)
  }
}

const createFolder = async ({ relativePath = '' } = {}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const directory = resolveInsideVault(vault.path, relativePath)
  await fs.ensureDir(directory)
  const folderName = nextAvailableName('New Folder', (name) =>
    fs.existsSync(path.join(directory, name))
  )
  const fullPath = path.join(directory, folderName)
  await fs.ensureDir(fullPath)
  return {
    folder: { path: path.relative(vault.path, fullPath), fullPath },
    entries: await listDirectoryForVault(vault, relativePath)
  }
}

const getSidebarEntryType = async (vault, relativePath, requestedType) => {
  if (requestedType === 'note' || requestedType === 'folder') return requestedType
  const stats = await fs.stat(resolveInsideVault(vault.path, relativePath))
  return stats.isDirectory() ? 'folder' : 'note'
}

const attachSidebarEntry = async ({ relativePath, title, type } = {}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const normalizedPath = normalizeRelativePath(relativePath)
  if (!normalizedPath) throw new Error('A path is required.')
  const fullPath = resolveInsideVault(vault.path, normalizedPath)
  if (!(await fs.pathExists(fullPath))) throw new Error('Entry not found.')
  const entryType = await getSidebarEntryType(vault, normalizedPath, type)
  const workspace = await readWorkspace(vault.path)
  workspace.sidebar = (workspace.sidebar || []).filter((item) => item.path !== normalizedPath)
  const fallbackTitle = path.basename(normalizedPath).replace(/\.md$/i, '')
  workspace.sidebar.push({
    id: createId(`${entryType}-${normalizedPath}`),
    title: String(title || fallbackTitle),
    type: entryType,
    path: normalizedPath,
    collapsed: false
  })
  await writeWorkspace(vault.path, workspace)
  return {
    workspace,
    entries: await listDirectoryForVault(
      vault,
      path.dirname(normalizedPath) === '.' ? '' : path.dirname(normalizedPath)
    )
  }
}

const detachSidebarEntry = async ({ relativePath } = {}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const normalizedPath = normalizeRelativePath(relativePath)
  if (!normalizedPath) throw new Error('A path is required.')
  const workspace = await readWorkspace(vault.path)
  workspace.sidebar = (workspace.sidebar || []).filter((item) => item.path !== normalizedPath)
  await writeWorkspace(vault.path, workspace)
  return { workspace, entries: await listDirectoryForVault(vault, '') }
}

const replaceWorkspacePathPrefix = (workspace, oldRelativePath, newRelativePath) => {
  const oldPath = normalizeRelativePath(oldRelativePath)
  const newPath = normalizeRelativePath(newRelativePath)
  if (!oldPath || !newPath) return workspace

  for (const category of workspace.sidebar || []) {
    if (category.path === oldPath) {
      category.path = newPath
      category.title = path.basename(newPath).replace(/\.md$/i, '')
    } else if (
      category.path?.startsWith(`${oldPath}${path.sep}`) ||
      category.path?.startsWith(`${oldPath}/`)
    ) {
      category.path = `${newPath}${category.path.slice(oldPath.length)}`
    }
    for (const item of category.items || []) {
      const itemPath = normalizeRelativePath(item.path)
      if (itemPath === oldPath) {
        item.path = newPath
        item.title = path.basename(newPath).replace(/\.md$/i, '')
      } else if (
        itemPath.startsWith(`${oldPath}${path.sep}`) ||
        itemPath.startsWith(`${oldPath}/`)
      ) {
        item.path = `${newPath}${itemPath.slice(oldPath.length)}`
      }
    }
  }
  return workspace
}

const removeWorkspacePathPrefix = (workspace, relativePath) => {
  const targetPath = normalizeRelativePath(relativePath)
  if (!targetPath) return workspace
  workspace.sidebar = (workspace.sidebar || [])
    .map((category) => ({
      ...category,
      items: (category.items || []).filter((item) => {
        const itemPath = normalizeRelativePath(item.path)
        return (
          itemPath !== targetPath &&
          !itemPath.startsWith(`${targetPath}${path.sep}`) &&
          !itemPath.startsWith(`${targetPath}/`)
        )
      })
    }))
    .filter((category) => {
      const categoryPath = normalizeRelativePath(category.path)
      if (
        categoryPath === targetPath ||
        categoryPath.startsWith(`${targetPath}${path.sep}`) ||
        categoryPath.startsWith(`${targetPath}/`)
      ) {
        return false
      }
      return category.items?.length || category.type || category.id === 'getting-started'
    })
  return workspace
}

const getRelativeParentPath = (relativePath) => {
  const parent = path.dirname(normalizeRelativePath(relativePath))
  return parent === '.' ? '' : normalizeRelativePath(parent)
}

const isTargetInsideSource = (sourcePath, targetDirectoryPath) => {
  const source = normalizeRelativePath(sourcePath)
  const target = normalizeRelativePath(targetDirectoryPath)
  if (!source || !target) return false
  return target === source || target.startsWith(`${source}/`)
}

const renameEntry = async ({ relativePath, title } = {}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const normalizedPath = normalizeRelativePath(relativePath)
  const normalizedTitle = String(title || '')
    .trim()
    .replace(/[\\/]/g, '-')
  if (!normalizedPath || !normalizedTitle) throw new Error('A path and a new name are required.')

  const source = resolveInsideVault(vault.path, normalizedPath)
  if (!(await fs.pathExists(source))) throw new Error('Entry not found.')

  const stats = await fs.stat(source)
  const extension = stats.isFile() ? path.extname(source) : ''
  const nextName =
    stats.isFile() && !path.extname(normalizedTitle)
      ? `${normalizedTitle}${extension}`
      : normalizedTitle
  const target = resolveInsideVault(vault.path, path.join(path.dirname(normalizedPath), nextName))
  if (source === target) {
    return {
      workspace: await readWorkspace(vault.path),
      entries: await listDirectoryForVault(
        vault,
        path.dirname(normalizedPath) === '.' ? '' : path.dirname(normalizedPath)
      )
    }
  }
  if (await fs.pathExists(target)) throw new Error('An item with this name already exists.')

  await fs.move(source, target)
  if (stats.isFile() && path.extname(source).toLowerCase() === '.md') {
    const currentMarkdown = await fs.readFile(target, 'utf8')
    const nextMarkdown = updateMarkdownTitle(currentMarkdown, normalizedTitle)
    if (nextMarkdown !== currentMarkdown) {
      await fs.writeFile(target, nextMarkdown, 'utf8')
    }
  }
  getSearchService()
    .deleteFile(source)
    .catch(() => {})
  getSearchService()
    .indexFile(target)
    .catch(() => {})
  const workspace = replaceWorkspacePathPrefix(
    await readWorkspace(vault.path),
    normalizedPath,
    path.relative(vault.path, target)
  )
  await writeWorkspace(vault.path, workspace)

  const parentPath = path.dirname(normalizedPath) === '.' ? '' : path.dirname(normalizedPath)
  return {
    workspace,
    entries: await listDirectoryForVault(vault, parentPath)
  }
}

const moveEntry = async ({ relativePath, targetDirectoryPath = '' } = {}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')

  const normalizedPath = normalizeRelativePath(relativePath)
  const normalizedTargetDirectory = normalizeRelativePath(targetDirectoryPath || '')
  if (!normalizedPath) throw new Error('A source path is required.')
  if (isTargetInsideSource(normalizedPath, normalizedTargetDirectory)) {
    throw new Error('Cannot move a folder into itself or one of its subfolders.')
  }

  const source = resolveInsideVault(vault.path, normalizedPath)
  if (!(await fs.pathExists(source))) throw new Error('Entry not found.')

  const sourceStats = await fs.stat(source)
  const targetDirectory = resolveInsideVault(vault.path, normalizedTargetDirectory)
  if (!(await fs.pathExists(targetDirectory))) throw new Error('Destination folder not found.')
  const targetStats = await fs.stat(targetDirectory)
  if (!targetStats.isDirectory()) throw new Error('Destination is not a folder.')

  const oldParentPath = getRelativeParentPath(normalizedPath)
  if (oldParentPath === normalizedTargetDirectory) {
    return {
      workspace: await readWorkspace(vault.path),
      entries: await listDirectoryForVault(vault, oldParentPath)
    }
  }

  const nextRelativePath = normalizeRelativePath(
    path.join(normalizedTargetDirectory, path.basename(normalizedPath))
  )
  const target = resolveInsideVault(vault.path, nextRelativePath)
  if (await fs.pathExists(target))
    throw new Error('An item with this name already exists in the destination folder.')

  await fs.move(source, target)

  if (sourceStats.isDirectory()) {
    const walkAndRefresh = async (currentSourcePath, currentTargetPath) => {
      const entries = await fs.readdir(currentTargetPath, { withFileTypes: true })
      for (const entry of entries) {
        const nextTargetPath = path.join(currentTargetPath, entry.name)
        const nextSourcePath = path.join(currentSourcePath, entry.name)
        if (entry.isDirectory()) {
          await walkAndRefresh(nextSourcePath, nextTargetPath)
        } else if (entry.isFile() && nextTargetPath.toLowerCase().endsWith('.md')) {
          getSearchService()
            .deleteFile(nextSourcePath)
            .catch(() => {})
          getSearchService()
            .indexFile(nextTargetPath)
            .catch(() => {})
        }
      }
    }

    await walkAndRefresh(source, target)
  } else {
    getSearchService()
      .deleteFile(source)
      .catch(() => {})
    if (path.extname(target).toLowerCase() === '.md') {
      getSearchService()
        .indexFile(target)
        .catch(() => {})
    }
  }

  const workspace = replaceWorkspacePathPrefix(
    await readWorkspace(vault.path),
    normalizedPath,
    nextRelativePath
  )
  await writeWorkspace(vault.path, workspace)

  return {
    workspace,
    entries: await listDirectoryForVault(vault, normalizedTargetDirectory)
  }
}

const deleteEntry = async ({ relativePath } = {}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const normalizedPath = normalizeRelativePath(relativePath)
  if (!normalizedPath) throw new Error('A path is required.')

  const target = resolveInsideVault(vault.path, normalizedPath)
  if (!(await fs.pathExists(target))) throw new Error('Entry not found.')
  await fs.remove(target)
  getSearchService()
    .deleteFile(target)
    .catch(() => {})

  const workspace = removeWorkspacePathPrefix(await readWorkspace(vault.path), normalizedPath)
  await writeWorkspace(vault.path, workspace)

  const parentPath = path.dirname(normalizedPath) === '.' ? '' : path.dirname(normalizedPath)
  return {
    workspace,
    entries: await listDirectoryForVault(vault, parentPath)
  }
}

const importGoogleKeep = async (event) => {
  const win = BrowserWindow.fromWebContents(event.sender)
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')

  const exportSelection = await dialog.showOpenDialog(win, {
    title: 'Select Google Keep export',
    properties: ['openFile'],
    filters: [{ name: 'Google Keep export', extensions: ['zip', 'json'] }]
  })

  if (exportSelection.canceled || !exportSelection.filePaths?.[0]) {
    return { canceled: true }
  }

  const destinationSelection = await dialog.showOpenDialog(win, {
    title: 'Choose destination folder inside the vault',
    defaultPath: vault.path,
    properties: ['openDirectory', 'createDirectory']
  })

  if (destinationSelection.canceled || !destinationSelection.filePaths?.[0]) {
    return { canceled: true }
  }

  const selectedDestination = destinationSelection.filePaths[0]
  const destinationPath = resolveInsideVault(
    vault.path,
    path.relative(vault.path, selectedDestination)
  )
  const result = await importGoogleKeepExport({
    sourcePath: exportSelection.filePaths[0],
    destinationPath
  })
  await Promise.all(
    (result.files || []).map((filePath) =>
      getSearchService()
        .indexFile(filePath)
        .catch(() => {})
    )
  )

  win.webContents.send('mt::show-notification', {
    title: 'Google Keep import complete',
    message: `Imported ${result.imported} note${result.imported === 1 ? '' : 's'} into ${path.relative(vault.path, destinationPath) || 'the vault root'}.`,
    type: 'info',
    time: 8000
  })

  return {
    canceled: false,
    entries: await listDirectoryForVault(vault, path.relative(vault.path, destinationPath)),
    ...result
  }
}

const importGoogleKeepFromPaths = async ({ sourcePath, destinationRelativePath = '' } = {}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const destinationPath = resolveInsideVault(vault.path, destinationRelativePath)
  const result = await importGoogleKeepExport({ sourcePath, destinationPath })
  await Promise.all(
    (result.files || []).map((filePath) =>
      getSearchService()
        .indexFile(filePath)
        .catch(() => {})
    )
  )
  return {
    canceled: false,
    entries: await listDirectoryForVault(vault, path.relative(vault.path, destinationPath)),
    ...result
  }
}

const listCalendarEvents = async () => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  await initializeVault(vault.path)
  return readCalendar(vault.path)
}

const importGoogleCalendarFromPath = async ({ sourcePath } = {}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const ics = await fs.readFile(sourcePath, 'utf8')
  const importedEvents = parseIcsCalendar(ics, { source: 'google-calendar' })
  const calendar = await readCalendar(vault.path)
  const events = mergeCalendarEvents(calendar.events, importedEvents)
  await writeCalendar(vault.path, { events })
  return {
    imported: importedEvents.length,
    calendar: await readCalendar(vault.path)
  }
}

const importGoogleCalendar = async (event) => {
  const win = BrowserWindow.fromWebContents(event.sender)
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')

  const selection = await dialog.showOpenDialog(win, {
    title: 'Select Google Calendar ICS export',
    properties: ['openFile'],
    filters: [{ name: 'Calendar export', extensions: ['ics'] }]
  })

  if (selection.canceled || !selection.filePaths?.[0]) {
    return { canceled: true }
  }

  const result = await importGoogleCalendarFromPath({ sourcePath: selection.filePaths[0] })
  win.webContents.send('mt::show-notification', {
    title: 'Google Calendar import complete',
    message: `Imported ${result.imported} event${result.imported === 1 ? '' : 's'}.`,
    type: 'info',
    time: 8000
  })
  return {
    canceled: false,
    ...result
  }
}

const refreshGoogleAccessToken = async (config) => {
  if (config.accessToken) return config.accessToken
  if (!config.clientId || !config.clientSecret || !config.refreshToken) {
    throw new Error(
      'Google Calendar OAuth config requires client id, client secret, and refresh token.'
    )
  }
  const response = await fetch('https://oauth2.googleapis.com/token', {
    method: 'POST',
    headers: { 'content-type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({
      client_id: config.clientId,
      client_secret: config.clientSecret,
      refresh_token: config.refreshToken,
      grant_type: 'refresh_token'
    }).toString()
  })
  const data = await response.json()
  if (!response.ok)
    throw new Error(data?.error_description || data?.error || 'Google OAuth token refresh failed.')
  return data.access_token
}

const syncGoogleCalendar = async () => {
  log.info('[sync] syncGoogleCalendar:start')
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const config = normalizeGoogleCalendarConfig(getConfig().googleCalendarConfig)
  if (!config.enabled) throw new Error('Google Calendar sync is not enabled.')
  const accessToken = await refreshGoogleAccessToken(config)
  const calendarId = encodeURIComponent(config.calendarId)
  const headers = { authorization: `Bearer ${accessToken}`, 'content-type': 'application/json' }
  const response = await fetch(
    `https://www.googleapis.com/calendar/v3/calendars/${calendarId}/events?singleEvents=true&orderBy=startTime`,
    {
      headers
    }
  )
  const data = await response.json()
  if (!response.ok) throw new Error(data?.error?.message || 'Google Calendar pull failed.')

  const calendar = await readCalendar(vault.path)
  const incoming = (data.items || []).map((event) =>
    googleEventToCalendarEvent(event, config.calendarId)
  )
  const merged = mergeCalendarEvents(calendar.events, incoming)
  const localOnly = merged.filter((event) => event.source !== 'google-calendar')
  let pushed = 0
  for (const event of localOnly) {
    const pushResponse = await fetch(
      `https://www.googleapis.com/calendar/v3/calendars/${calendarId}/events`,
      {
        method: 'POST',
        headers,
        body: JSON.stringify(calendarEventToGoogleEvent(event))
      }
    )
    if (pushResponse.ok) pushed += 1
  }
  await writeCalendar(vault.path, { events: merged })
  return {
    pulled: incoming.length,
    pushed,
    calendar: await readCalendar(vault.path)
  }
}

const createMarkdownFromSource = ({
  title,
  url,
  body,
  importedAt = new Date().toISOString()
}) => `---
title: "${String(title || 'Imported source').replace(/"/g, '\\"')}"
type: "source"
tags: ["source"]
sourceUrl: "${url}"
createdAt: "${importedAt}"
updatedAt: "${importedAt}"
---

# ${title || 'Imported source'}

Source: ${url}

${body || ''}
`

const safeSourceFilename = (title, url) => {
  const stem =
    String(title || createSourceId(url) || 'Imported source')
      .replace(/[\\/]/g, '-')
      .replace(/[<>:"|?*]/g, '')
      .trim()
      .slice(0, 80) || 'Imported source'
  return `${stem}.md`
}

const listSources = async () => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  await initializeVault(vault.path)
  return readSources(vault.path)
}

const writeSourceNote = async ({
  url,
  title,
  body,
  destinationRelativePath = 'Sources',
  type = 'url',
  metadata = {}
}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const destination = resolveInsideVault(vault.path, destinationRelativePath)
  await fs.ensureDir(destination)
  const filename = nextAvailableName(safeSourceFilename(title, url), (name) =>
    fs.existsSync(path.join(destination, name))
  )
  const fullPath = path.join(destination, filename)
  await fs.writeFile(fullPath, createMarkdownFromSource({ title, url, body }), 'utf8')
  getSearchService()
    .indexFile(fullPath)
    .catch(() => {})

  const sourceRecord = normalizeSourceRecord({
    url,
    title,
    type,
    notePath: path.relative(vault.path, fullPath),
    metadata
  })
  const sourceData = await readSources(vault.path)
  await writeSources(vault.path, [
    ...sourceData.sources.filter((source) => source.id !== sourceRecord.id),
    sourceRecord
  ])
  return sourceRecord
}

const ingestSourceUrl = async ({ url, destinationRelativePath = 'Sources' } = {}) => {
  const normalizedUrl = normalizeSourceUrl(url)
  const response = await fetch(normalizedUrl)
  if (!response.ok) throw new Error(`Source returned HTTP ${response.status}.`)
  const html = await response.text()
  const title = extractHtmlTitle(html, normalizedUrl)
  const body = htmlToReadableText(html)
  const source = await writeSourceNote({
    url: normalizedUrl,
    title,
    body,
    destinationRelativePath,
    type: 'url'
  })
  return {
    source,
    sources: await listSources()
  }
}

const importRssSource = async ({ url, destinationRelativePath = 'Sources', limit = 20 } = {}) => {
  const normalizedUrl = normalizeSourceUrl(url)
  const response = await fetch(normalizedUrl)
  if (!response.ok) throw new Error(`RSS source returned HTTP ${response.status}.`)
  const xml = await response.text()
  const items = parseRssFeed(xml).slice(0, Number(limit) || 20)
  const sources = []
  for (const item of items) {
    sources.push(
      await writeSourceNote({
        url: item.url,
        title: item.title,
        body: item.description,
        destinationRelativePath,
        type: 'rss',
        metadata: {
          feedUrl: normalizedUrl,
          publishedAt: item.publishedAt
        }
      })
    )
  }
  return {
    imported: sources.length,
    sources: await listSources()
  }
}

const proposeWikiRecords = async (windowId = null) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  await initializeVault(vault.path)
  const wiki = await readWiki(vault.path)
  const searchService = getSearchService()
  let inspection = await searchService.inspectIndex(windowId)
  if (!inspection?.graph?.nodes?.length) {
    await searchService.rebuildIndex(windowId)
    inspection = await searchService.inspectIndex(windowId)
  }
  const records = mergeWikiProposals(
    wiki.records,
    buildWikiProposalsFromGraph({
      graph: inspection?.graph,
      now: new Date()
    })
  )
  await writeWiki(vault.path, records)
  return readWiki(vault.path)
}

const listWikiRecords = async (windowId = null) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  await initializeVault(vault.path)
  const wiki = await readWiki(vault.path)
  if (wiki.records.some((record) => record.status === 'proposed' || record.status === 'accepted')) {
    return wiki
  }
  return proposeWikiRecords(windowId)
}

const updateWikiRecordStatus = async (id, status, windowId = null) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  await initializeVault(vault.path)
  const wiki = await readWiki(vault.path)
  const target = wiki.records.find((record) => record.id === id)
  if (!target) throw new Error('Wiki proposal not found.')
  const now = new Date()
  const records = wiki.records.map((record) =>
    record.id === id
      ? normalizeWikiRecord({ ...record, status, updatedAt: now.toISOString() })
      : record
  )
  await writeWiki(vault.path, records)
  return readWiki(vault.path)
}

const acceptWikiRecord = async ({ id } = {}, windowId = null) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  await initializeVault(vault.path)
  const wiki = await readWiki(vault.path)
  const target = wiki.records.find((record) => record.id === id)
  if (!target) throw new Error('Wiki proposal not found.')

  const destination = resolveInsideVault(vault.path, 'Wiki')
  await fs.ensureDir(destination)
  const filename = nextAvailableName(
    `${target.title.replace(/[\\/]/g, '-').slice(0, 80) || 'Topic'}.md`,
    (name) => fs.existsSync(path.join(destination, name))
  )
  const fullPath = path.join(destination, filename)
  await fs.writeFile(fullPath, createGraphBackedWikiMarkdown(target), 'utf8')
  getSearchService()
    .indexFile(fullPath, windowId)
    .catch(() => {})

  const notePath = path.relative(vault.path, fullPath)
  const records = wiki.records.map((record) =>
    record.id === id
      ? normalizeWikiRecord({
          ...record,
          status: 'accepted',
          notePath,
          updatedAt: new Date().toISOString()
        })
      : record
  )
  await writeWiki(vault.path, records)
  return {
    wiki: await readWiki(vault.path),
    note: {
      path: notePath,
      fullPath
    },
    entries: await listDirectoryForVault(vault, 'Wiki')
  }
}

const createWikiPageFromRecord = async ({ record = {}, windowId = null } = {}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  await initializeVault(vault.path)
  const destination = resolveInsideVault(vault.path, 'Wiki')
  await fs.ensureDir(destination)
  const title = String(record.title || record.topic || 'Wiki Topic').trim() || 'Wiki Topic'
  const filename = nextAvailableName(
    `${title.replace(/[\\/]/g, '-').slice(0, 80) || 'Topic'}.md`,
    (name) => fs.existsSync(path.join(destination, name))
  )
  const fullPath = path.join(destination, filename)
  await fs.writeFile(fullPath, createGraphBackedWikiMarkdown(record), 'utf8')
  getSearchService()
    .indexFile(fullPath, windowId)
    .catch(() => {})

  return {
    note: {
      path: path.relative(vault.path, fullPath),
      fullPath
    },
    entries: await listDirectoryForVault(vault, 'Wiki')
  }
}

const inspectWikiSource = async (
  { path: sourcePath = '', record = null } = {},
  windowId = null
) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  await initializeVault(vault.path)
  const inspection = await getSearchService().inspectIndex(windowId)
  return buildWikiSourceInsight({
    graph: inspection?.graph || {},
    path: sourcePath,
    record
  })
}

const inspectWikiContext = async (
  { path: sourcePath = '', record = null, limit = 12 } = {},
  windowId = null
) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  await initializeVault(vault.path)
  const inspection = await getSearchService().inspectIndex(windowId)
  return buildWikiChatContext({
    graph: inspection?.graph || {},
    path: sourcePath,
    record,
    limit
  })
}

const dismissWikiRecord = async ({ id } = {}, windowId = null) =>
  updateWikiRecordStatus(id, 'dismissed', windowId)

const MODEL_PURPOSE_TO_SELECTION_SLOT = Object.freeze({
  ocr: 'ocr',
  chat: 'chat',
  rag: 'chat',
  tagging: 'tagging',
  naming: 'naming',
  wiki: 'wiki',
  summary: 'summary',
  agent: 'agent'
})

const getSelectedModelForPurpose = (purpose = 'chat') => {
  const selection = {
    ...createDefaultModelSelection(),
    ...(getConfig().atomicModelSelection || {})
  }
  const slot = MODEL_PURPOSE_TO_SELECTION_SLOT[purpose] || purpose
  return String(selection[slot] || '').trim()
}

const createConfiguredAiRuntime = ({ purpose = 'chat', override = {} } = {}) => {
  const baseConfig = normalizeAiConfig({
    ...getConfig().aiConfig,
    ...override
  })
  const selectedModel = getSelectedModelForPurpose(purpose)
  const model = String(override.model || selectedModel || baseConfig.model || '').trim()
  const endpoint = resolveAiEndpoint({
    endpoint: override.endpoint || baseConfig.endpoint,
    transport: baseConfig.transport
  })
  return {
    ...baseConfig,
    endpoint,
    model,
    purpose
  }
}

const resolveConfiguredLocalModel = (config = {}) => {
  return (
    ATOMIC_MODEL_CATALOG.find(
      (item) =>
        item.id === config.model ||
        item.model === config.model ||
        item.uri === config.model ||
        item.pull === config.model
    ) || {
      id: config.model,
      name: config.model,
      provider: 'node-llama-cpp',
      task: config.purpose === 'embedding' ? 'embedding' : 'chat-completion',
      model: config.model,
      uri: config.model,
      pull: config.model
    }
  )
}

export const createNodeLlamaCppEmbeddingProvider = (deps = {}) =>
  createNodeLlamaCppEmbeddingProviderImpl({
    ...deps,
    getSelectedModel: deps.getSelectedModel || getSelectedModelForPurpose,
    resolveLocalModel:
      deps.resolveLocalModel || ((config) => modelRuntime.modelLibrary.resolveLocalModel(config)),
    resolveConfiguredLocalModel: deps.resolveConfiguredLocalModel || resolveConfiguredLocalModel,
    runtime: deps.runtime || modelRuntime,
    logger: deps.logger || log
  })

const callConfiguredAi = async (messages = [], options = {}) => {
  const config = createConfiguredAiRuntime(options)
  if (config.transport === 'node-llama-cpp') {
    return modelRuntime.nodeLlamaCppRuntime.generateChat({
      model: resolveConfiguredLocalModel(config),
      messages,
      maxTokens: 180,
      temperature: 0.2
    })
  }
  if (config.transport === 'browser') {
    throw new Error(
      'Browser model runtime is no longer the local AI path. Use node-llama-cpp for local chat and embeddings.'
    )
  }
  if (!config.endpoint || !config.model) {
    return ''
  }
  const response = await fetch(config.endpoint, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...(config.apiKey ? { authorization: `Bearer ${config.apiKey}` } : {})
    },
    body: JSON.stringify(
      createAiRequestBody({
        transport: config.transport,
        model: config.model,
        messages
      })
    )
  })
  const text = await response.text()
  const data = text ? JSON.parse(text) : {}
  if (!response.ok) {
    throw new Error(
      data?.error?.message || data?.message || `AI endpoint returned HTTP ${response.status}.`
    )
  }
  return extractAiResponseText(data)
}

const testConfiguredAi = async (payload = {}) => {
  const config = createConfiguredAiRuntime({ purpose: 'chat', override: payload })
  log.info('[ai] testConfiguredAi:start', {
    transport: config.transport,
    endpoint: config.endpoint,
    model: config.model
  })
  if (config.transport === 'node-llama-cpp') {
    const result = await modelRuntime.testNodeLlamaCppModel(resolveConfiguredLocalModel(config), {
      includeEmbeddings: config.purpose === 'embedding'
    })
    log.info('[ai] testConfiguredAi:node-llama-cpp:done', {
      latencyMs: result.latencyMs,
      model: config.model
    })
    return {
      ...result,
      transport: config.transport,
      endpoint: config.endpoint,
      model: config.model
    }
  }
  if (config.transport === 'browser') {
    throw new Error(
      'Browser model runtime is no longer the local AI path. Use node-llama-cpp for local chat and embeddings.'
    )
  }
  if (!config.endpoint) throw new Error('AI endpoint is missing.')
  if (!config.model) throw new Error('AI model is missing.')
  const startedAt = Date.now()
  const response = await callConfiguredAi(
    [
      { role: 'system', content: 'You are a health-check endpoint. Reply with OK only.' },
      { role: 'user', content: 'Return OK.' }
    ],
    { purpose: 'chat', override: config }
  )
  log.info('[ai] testConfiguredAi:remote:done', {
    latencyMs: Date.now() - startedAt,
    transport: config.transport,
    endpoint: config.endpoint,
    model: config.model
  })
  return {
    ok: true,
    provider: config.preset,
    transport: config.transport,
    endpoint: config.endpoint,
    model: config.model,
    latencyMs: Date.now() - startedAt,
    response: response || ''
  }
}

const readCitedSearchResults = async ({ query, limit, context }) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const results = await getSearchService().search(
    {
      query,
      mode: 'smart',
      limit
    },
    getApiWindowId(context)
  )
  const citations = []
  for (const result of results) {
    const relativePath = normalizeRelativePath(result.relativePath)
    if (!relativePath) continue
    const fullPath = resolveInsideVault(vault.path, relativePath)
    const markdown = await fs.readFile(fullPath, 'utf8').catch(() => '')
    citations.push({
      title: result.title || path.basename(relativePath, path.extname(relativePath)),
      path: relativePath,
      score: result.score || 0,
      snippet: result.snippets?.[0]?.text || markdown.slice(0, 280).replace(/\s+/g, ' ').trim()
    })
  }
  return citations
}

const chatWithRag = async ({ message, limit = 6, messages = [] } = {}, context = {}) => {
  const citations = await readCitedSearchResults({ query: message, limit, context })
  const wikiContext = citations[0]?.path
    ? await inspectWikiContext(
        {
          path: citations[0].path,
          limit: Math.max(3, Number(limit) || 6)
        },
        getApiWindowId(context)
      ).catch(() => null)
    : null
  const { systemMessage, prompt } = buildRagChatPrompt({
    message,
    messages,
    citations,
    wikiContext
  })
  let answer = ''
  try {
    answer = await callConfiguredAi(
      [
        {
          role: 'system',
          content: systemMessage
        },
        {
          role: 'user',
          content: prompt
        }
      ],
      { purpose: 'chat' }
    )
  } catch (error) {
    answer = ''
  }
  if (!answer) {
    answer = citations.length
      ? `I found ${citations.length} relevant local note${citations.length === 1 ? '' : 's'}: ${citations.map((citation, index) => `[${index + 1}] ${citation.title}`).join(', ')}.`
      : 'I did not find matching local notes for this question.'
  }
  return {
    answer,
    citations,
    wikiContext
  }
}

const extractTagsFromText = (text = '') => {
  return suggestAtomicTags(text, 8)
}

const replaceMarkdownTags = (markdown, tags) => {
  const serialized = `tags: [${tags.map((tag) => `"${tag.replace(/"/g, '\\"')}"`).join(', ')}]`
  if (/^---\r?\n[\s\S]*?\r?\n---/.test(markdown)) {
    if (/^---\r?\n[\s\S]*?^\s*tags:\s*(?:\[.*?\]|$)/m.test(markdown)) {
      return markdown.replace(/^\s*tags:\s*(?:\[.*?\]|$)/m, serialized)
    }
    return markdown.replace(/^---\r?\n/, `---\n${serialized}\n`)
  }
  return `---\n${serialized}\n---\n\n${markdown}`
}

const autotagNote = async ({ relativePath } = {}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const normalizedPath = normalizeRelativePath(relativePath)
  const fullPath = resolveInsideVault(vault.path, normalizedPath)
  const markdown = await fs.readFile(fullPath, 'utf8')
  let tags = []
  try {
    const response = await callConfiguredAi(
      [
        {
          role: 'system',
          content: 'Return only a JSON array of 3 to 8 lowercase tags for this markdown note.'
        },
        {
          role: 'user',
          content: markdown.slice(0, 12000)
        }
      ],
      { purpose: 'tagging' }
    )
    tags = JSON.parse(response)
      .map((tag) => String(tag).trim().replace(/^#/, ''))
      .filter(Boolean)
  } catch {
    tags = extractTagsFromText(markdown)
  }
  tags = [...new Set(tags)].slice(0, 8)
  const nextMarkdown = ensureAtomicSourcesSection(replaceMarkdownTags(markdown, tags))
  await fs.writeFile(fullPath, nextMarkdown, 'utf8')
  getSearchService()
    .indexFile(fullPath)
    .catch(() => {})
  return {
    relativePath: normalizedPath,
    tags
  }
}

const listMcpTools = () => [
  { name: 'search.query', description: 'Search local vault notes by keyword or meaning.' },
  { name: 'notes.create', description: 'Create a note in the active vault.' },
  { name: 'notes.autotag', description: 'Generate tags for a markdown note.' },
  { name: 'sources.ingestUrl', description: 'Import a URL as a local source note.' },
  { name: 'wiki.propose', description: 'Generate cited wiki proposals.' },
  { name: 'wiki.createPage', description: 'Create a wiki page from a cited proposal.' },
  { name: 'wiki.sourceInfo', description: 'Inspect a source note inside the semantic graph.' },
  {
    name: 'wiki.context',
    description: 'Inspect a source note with compact graph context for chat.'
  },
  { name: 'rag.chat', description: 'Ask a cited RAG question over local notes.' }
]

const callMcpTool = async ({ name, arguments: args = {} } = {}, context = {}) => {
  if (name === 'search.query') {
    return getSearchService().search(
      {
        query: args.query,
        mode: args.mode || 'smart',
        limit: args.limit || 10
      },
      getApiWindowId(context)
    )
  }
  if (name === 'notes.create') return createNote(args)
  if (name === 'notes.autotag') return autotagNote(args)
  if (name === 'sources.ingestUrl') return ingestSourceUrl(args)
  if (name === 'wiki.propose') return proposeWikiRecords()
  if (name === 'wiki.createPage') return createWikiPageFromRecord(args)
  if (name === 'wiki.sourceInfo') return inspectWikiSource(args, context.windowId)
  if (name === 'wiki.context') return inspectWikiContext(args, context.windowId)
  if (name === 'rag.chat') return chatWithRag(args, context)
  throw new Error(`Unknown MCP tool: ${name}.`)
}

const runPlugin = async ({ id, input = {} } = {}, context = {}) => {
  const plugin = mergePluginState(ATOMIC_PLUGIN_MANIFESTS, getConfig().atomicPluginState).find(
    (item) => item.id === id
  )
  if (!plugin) throw new Error('Unknown plugin.')
  if (!plugin.enabled) throw new Error('Plugin is disabled.')
  const runtime = resolvePluginRuntime(plugin)
  if (runtime === EXTENSION_PLUGIN_RUNTIMES.GOOGLE_CALENDAR_SYNC) return syncGoogleCalendar()
  if (runtime === EXTENSION_PLUGIN_RUNTIMES.SOURCE_INGEST_URL) return ingestSourceUrl(input)
  if (runtime === EXTENSION_PLUGIN_RUNTIMES.MCP_TOOL_CALL) return callMcpTool(input, context)
  throw new Error(`Plugin has no runtime: ${id}.`)
}

const runProgram = async ({ id, command, cwd = '' } = {}) => {
  const environments = normalizeProgramEnvironments(getConfig().programEnvironments)
  const environment = environments[id]
  if (!environment) throw new Error('Unknown program environment.')
  const vault = getActiveVault()
  return programRuntime.run({
    environment,
    command,
    cwd: cwd || vault?.path || process.cwd()
  })
}

const runProgrammaticTask = async ({ id } = {}) => {
  const vault = getActiveVault()
  if (!vault) throw new Error('No active ElephantNote vault.')
  const task = mergeTaskState(PROGRAMMATIC_TASK_TEMPLATES, getConfig().atomicTaskState).find(
    (item) => item.id === id
  )
  if (!task) throw new Error('Unknown task.')

  const steps = []
  for (const action of task.actions) {
    if (
      action === EXTENSION_TASK_ACTIONS.WIKI_PROPOSE ||
      action === EXTENSION_TASK_ACTIONS.WIKI_PROPOSAL
    ) {
      const wiki = await proposeWikiRecords()
      steps.push(
        createTaskStepResult({
          action,
          ok: true,
          summary: `${wiki.records.length} wiki record${wiki.records.length === 1 ? '' : 's'}`
        })
      )
    } else if (action === EXTENSION_TASK_ACTIONS.CALENDAR_SUMMARY) {
      const calendar = await readCalendar(vault.path)
      steps.push(
        createTaskStepResult({
          action,
          ok: true,
          summary: `${calendar.events.length} calendar event${calendar.events.length === 1 ? '' : 's'}`
        })
      )
    } else if (action === EXTENSION_TASK_ACTIONS.SEARCH_RECENT) {
      const notes = await listMarkdownNotesRecursive(vault)
      steps.push(
        createTaskStepResult({
          action,
          ok: true,
          summary: `${notes.slice(0, 8).length} recent note${notes.length === 1 ? '' : 's'}`
        })
      )
    } else {
      steps.push(
        createTaskStepResult({
          action,
          ok: false,
          summary: 'Action is defined but not executable locally yet.'
        })
      )
    }
  }

  const result = createTaskRunResult(steps)
  const config = getConfig()
  config.atomicTaskState = updateTaskState(PROGRAMMATIC_TASK_TEMPLATES, config.atomicTaskState, {
    id,
    enabled: task.enabled,
    lastRunAt: new Date().toISOString(),
    lastResult: result
  })
  setConfig(config)
  return mergeTaskState(PROGRAMMATIC_TASK_TEMPLATES, getConfig().atomicTaskState)
}

const getApiWindowId = (context = {}) => {
  if (context.windowId !== undefined && context.windowId !== null) return context.windowId
  const webContents = context.event?.sender
  if (!webContents) return null
  return BrowserWindow.fromWebContents(webContents)?.id ?? null
}

const createElephantNoteMainApi = () => {
  const searchService = getSearchService()
  log.info('[api] createElephantNoteMainApi')
  const sitePreviewService = getSitePreviewService()
  return createElephantNoteApi({
    handlers: {
      [ELEPHANTNOTE_API_ACTIONS.VAULTS_GET]: async () => loadVaultPayload(getActiveVault()),
      [ELEPHANTNOTE_API_ACTIONS.VAULTS_SELECT]: async (_payload, { event }) => {
        const win = BrowserWindow.fromWebContents(event.sender)
        const result = await dialog.showOpenDialog(win, {
          properties: ['openDirectory', 'createDirectory']
        })
        if (result.canceled || !result.filePaths?.[0]) {
          return { canceled: true }
        }
        await initializeVault(result.filePaths[0])
        const vault = upsertVault(result.filePaths[0])
        return { canceled: false, ...(await loadVaultPayload(vault)) }
      },
      [ELEPHANTNOTE_API_ACTIONS.VAULTS_SET_ACTIVE]: async ({ vaultId }) => {
        const config = getConfig()
        const vault = config.vaults.find((item) => item.id === vaultId)
        if (!vault) throw new Error('Unknown ElephantNote vault.')
        config.activeVaultId = vault.id
        setConfig(config)
        return loadVaultPayload(vault)
      },
      [ELEPHANTNOTE_API_ACTIONS.VAULTS_SET_ICON]: async ({ vaultId, icon }) =>
        setVaultIcon(vaultId, icon),
      [ELEPHANTNOTE_API_ACTIONS.VAULTS_SET_NAME]: async ({ vaultId, name }) =>
        setVaultName(vaultId, name),
      [ELEPHANTNOTE_API_ACTIONS.VAULTS_REMOVE]: async ({ vaultId }) => removeVault(vaultId),
      [ELEPHANTNOTE_API_ACTIONS.DIRECTORY_LIST]: async ({ relativePath = '' } = {}) => {
        const vault = getActiveVault()
        if (!vault) throw new Error('No active ElephantNote vault.')
        return listDirectoryForVault(vault, normalizeRelativePath(relativePath))
      },
      [ELEPHANTNOTE_API_ACTIONS.NOTES_CREATE]: async (payload) => createNote(payload),
      [ELEPHANTNOTE_API_ACTIONS.FOLDERS_CREATE]: async (payload) => createFolder(payload),
      [ELEPHANTNOTE_API_ACTIONS.SIDEBAR_ATTACH]: async (payload) => attachSidebarEntry(payload),
      [ELEPHANTNOTE_API_ACTIONS.SIDEBAR_DETACH]: async (payload) => detachSidebarEntry(payload),
      [ELEPHANTNOTE_API_ACTIONS.ENTRIES_RENAME]: async (payload) => renameEntry(payload),
      [ELEPHANTNOTE_API_ACTIONS.ENTRIES_MOVE]: async (payload) => moveEntry(payload),
      [ELEPHANTNOTE_API_ACTIONS.ENTRIES_DELETE]: async (payload) => deleteEntry(payload),
      [ELEPHANTNOTE_API_ACTIONS.IMPORT_GOOGLE_KEEP]: async (_payload, { event }) =>
        importGoogleKeep(event),
      [ELEPHANTNOTE_API_ACTIONS.IMPORT_GOOGLE_KEEP_FROM_PATHS]: async (payload) =>
        importGoogleKeepFromPaths(payload),
      [ELEPHANTNOTE_API_ACTIONS.CALENDAR_LIST]: async () => listCalendarEvents(),
      [ELEPHANTNOTE_API_ACTIONS.CALENDAR_IMPORT_GOOGLE]: async (_payload, { event }) =>
        importGoogleCalendar(event),
      [ELEPHANTNOTE_API_ACTIONS.CALENDAR_IMPORT_GOOGLE_FROM_PATH]: async (payload) =>
        importGoogleCalendarFromPath(payload),
      [ELEPHANTNOTE_API_ACTIONS.CALENDAR_GOOGLE_CONFIG_GET]: async () =>
        getConfig().googleCalendarConfig,
      [ELEPHANTNOTE_API_ACTIONS.CALENDAR_GOOGLE_CONFIG_SET]: async (payload) => {
        const config = getConfig()
        config.googleCalendarConfig = normalizeGoogleCalendarConfig({
          ...config.googleCalendarConfig,
          ...payload
        })
        setConfig(config)
        return getConfig().googleCalendarConfig
      },
      [ELEPHANTNOTE_API_ACTIONS.CALENDAR_GOOGLE_SYNC]: async () => syncGoogleCalendar(),
      [ELEPHANTNOTE_API_ACTIONS.SOURCES_LIST]: async () => listSources(),
      [ELEPHANTNOTE_API_ACTIONS.SOURCES_INGEST_URL]: async (payload) => ingestSourceUrl(payload),
      [ELEPHANTNOTE_API_ACTIONS.SOURCES_IMPORT_RSS]: async (payload) => importRssSource(payload),
      [ELEPHANTNOTE_API_ACTIONS.WIKI_LIST]: async (_payload, context) =>
        listWikiRecords(getApiWindowId(context)),
      [ELEPHANTNOTE_API_ACTIONS.WIKI_PROPOSE]: async (_payload, context) =>
        proposeWikiRecords(getApiWindowId(context)),
      [ELEPHANTNOTE_API_ACTIONS.WIKI_ACCEPT]: async (payload, context) =>
        acceptWikiRecord(payload, getApiWindowId(context)),
      [ELEPHANTNOTE_API_ACTIONS.WIKI_DISMISS]: async (payload, context) =>
        dismissWikiRecord(payload, getApiWindowId(context)),
      [ELEPHANTNOTE_API_ACTIONS.WIKI_SOURCE_INFO]: async (payload, context) =>
        inspectWikiSource(payload, getApiWindowId(context)),
      [ELEPHANTNOTE_API_ACTIONS.WIKI_CONTEXT]: async (payload, context) =>
        inspectWikiContext(payload, getApiWindowId(context)),
      [ELEPHANTNOTE_API_ACTIONS.SEARCH_INIT_VAULT]: async ({ vaultPath }, context) => {
        const windowId = getApiWindowId(context)
        log.info('[search] api:init-vault', { windowId, vaultPath })
        return searchService.registerWindowVault(windowId, vaultPath)
      },
      [ELEPHANTNOTE_API_ACTIONS.SEARCH_QUERY]: async (payload, context) => {
        const windowId = getApiWindowId(context)
        log.info('[search] api:query', {
          windowId,
          mode: payload?.mode || 'smart',
          limit: payload?.limit || 20
        })
        return searchService.search(payload, windowId)
      },
      [ELEPHANTNOTE_API_ACTIONS.SEARCH_STATUS]: async (_payload, context) => {
        const windowId = getApiWindowId(context)
        log.info('[search] api:status', { windowId })
        return searchService.getStatus(windowId)
      },
      [ELEPHANTNOTE_API_ACTIONS.SEARCH_INSPECT]: async (_payload, context) => {
        const windowId = getApiWindowId(context)
        log.info('[search] api:inspect', { windowId })
        return searchService.inspectIndex(windowId)
      },
      [ELEPHANTNOTE_API_ACTIONS.SEARCH_REBUILD]: async (_payload, context) => {
        const windowId = getApiWindowId(context)
        log.info('[search] api:rebuild', { windowId })
        return searchService.rebuildIndex(windowId)
      },
      [ELEPHANTNOTE_API_ACTIONS.SEARCH_CLEAR]: async (_payload, context) => {
        const windowId = getApiWindowId(context)
        log.info('[search] api:clear', { windowId })
        return searchService.clearIndex(windowId)
      },
      [ELEPHANTNOTE_API_ACTIONS.SEARCH_DISABLE]: async () => {
        log.warn('[search] api:disable')
        return searchService.disable()
      },
      [ELEPHANTNOTE_API_ACTIONS.SEARCH_ENABLE]: async () => {
        log.info('[search] api:enable')
        return searchService.enable()
      },
      [ELEPHANTNOTE_API_ACTIONS.SITES_PREVIEW_FOLDER]: async (payload) =>
        sitePreviewService.previewFolder(payload),
      [ELEPHANTNOTE_API_ACTIONS.SITES_BUILD_FOLDER]: async (payload) =>
        sitePreviewService.buildFolder(payload),
      [ELEPHANTNOTE_API_ACTIONS.SITES_STOP]: async ({ siteId }) =>
        sitePreviewService.stopPreview(siteId),
      [ELEPHANTNOTE_API_ACTIONS.SITES_STATUS]: async ({ siteId }) =>
        sitePreviewService.getStatus(siteId),
      [ELEPHANTNOTE_API_ACTIONS.SITES_OPEN_EXTERNAL]: async ({ url }) =>
        sitePreviewService.openExternal(url),
      [ELEPHANTNOTE_API_ACTIONS.AGENTS_LIST]: async () => listAgents(),
      [ELEPHANTNOTE_API_ACTIONS.AGENTS_REGISTER]: async (payload) => registerAgent(payload),
      [ELEPHANTNOTE_API_ACTIONS.AGENTS_UNREGISTER]: async ({ id }) => unregisterAgent(id),
      [ELEPHANTNOTE_API_ACTIONS.AGENTS_SEND]: async (payload) => sendAgentMessage(payload),
      [ELEPHANTNOTE_API_ACTIONS.RAG_CHAT]: async (payload, context) =>
        chatWithRag(payload, context),
      [ELEPHANTNOTE_API_ACTIONS.NOTES_AUTOTAG]: async (payload) => autotagNote(payload),
      [ELEPHANTNOTE_API_ACTIONS.MCP_TOOLS_LIST]: async () => listMcpTools(),
      [ELEPHANTNOTE_API_ACTIONS.MCP_TOOLS_CALL]: async (payload, context) =>
        callMcpTool(payload, context),
      [ELEPHANTNOTE_API_ACTIONS.AI_CONFIG_GET]: async () => getConfig().aiConfig,
      [ELEPHANTNOTE_API_ACTIONS.AI_CONFIG_TEST]: async (payload) => testConfiguredAi(payload),
      [ELEPHANTNOTE_API_ACTIONS.AI_CONFIG_SET]: async (payload) => {
        const config = getConfig()
        config.aiConfig = normalizeAiConfig(payload)
        setConfig(config)
        if (config.aiConfig.codexLinkEnabled && config.aiConfig.preset === 'codex') {
          registerAgent({
            id: 'codex',
            name: 'Codex',
            transport: config.aiConfig.transport,
            endpoint: config.aiConfig.endpoint,
            model: config.aiConfig.model,
            apiKey: config.aiConfig.apiKey,
            capabilities: ['chat', 'code']
          })
        }
        return config.aiConfig
      },
      [ELEPHANTNOTE_API_ACTIONS.FEATURES_GET]: async () => getConfig().featureFlags,
      [ELEPHANTNOTE_API_ACTIONS.FEATURES_SET]: async ({ key, enabled }) => {
        const config = getConfig()
        config.featureFlags = setFeatureFlag(config.featureFlags, key, enabled)
        setConfig(config)
        return config.featureFlags
      },
      [ELEPHANTNOTE_API_ACTIONS.ATOMIC_CATALOG_GET]: async () => ({
        models: ATOMIC_MODEL_CATALOG,
        features: ATOMIC_AI_FEATURES,
        plugins: ATOMIC_PLUGIN_MANIFESTS,
        tasks: PROGRAMMATIC_TASK_TEMPLATES
      }),
      [ELEPHANTNOTE_API_ACTIONS.MODEL_SELECTION_GET]: async () => getConfig().atomicModelSelection,
      [ELEPHANTNOTE_API_ACTIONS.MODEL_SELECTION_SET]: async (payload) => {
        const config = getConfig()
        config.atomicModelSelection = {
          ...createDefaultModelSelection(),
          ...payload
        }
        setConfig(config)
        return getConfig().atomicModelSelection
      },
      [ELEPHANTNOTE_API_ACTIONS.MODELS_LOCAL_LIST]: async () => {
        log.info('[model] api:list-local')
        return modelRuntime.listLocalModels()
      },
      [ELEPHANTNOTE_API_ACTIONS.MODELS_DOWNLOAD]: async ({ id }) => {
        const model = ATOMIC_MODEL_CATALOG.find((item) => item.id === id)
        if (!model) throw new Error('Unknown model.')
        log.info('[model] api:download', { id })
        return modelRuntime.downloadModel(model)
      },
      [ELEPHANTNOTE_API_ACTIONS.OCR_EXTRACT]: async (payload) =>
        modelRuntime.extractImageText(payload),
      [ELEPHANTNOTE_API_ACTIONS.PLUGINS_LIST]: async () =>
        mergePluginState(ATOMIC_PLUGIN_MANIFESTS, getConfig().atomicPluginState),
      [ELEPHANTNOTE_API_ACTIONS.PLUGINS_SET]: async (payload) => {
        const config = getConfig()
        config.atomicPluginState = updatePluginState(
          ATOMIC_PLUGIN_MANIFESTS,
          config.atomicPluginState,
          payload
        )
        setConfig(config)
        return mergePluginState(ATOMIC_PLUGIN_MANIFESTS, getConfig().atomicPluginState)
      },
      [ELEPHANTNOTE_API_ACTIONS.PLUGINS_RUN]: async (payload, context) =>
        runPlugin(payload, context),
      [ELEPHANTNOTE_API_ACTIONS.TASKS_LIST]: async () =>
        mergeTaskState(PROGRAMMATIC_TASK_TEMPLATES, getConfig().atomicTaskState),
      [ELEPHANTNOTE_API_ACTIONS.TASKS_SET]: async (payload) => {
        const config = getConfig()
        config.atomicTaskState = updateTaskState(
          PROGRAMMATIC_TASK_TEMPLATES,
          config.atomicTaskState,
          payload
        )
        setConfig(config)
        return mergeTaskState(PROGRAMMATIC_TASK_TEMPLATES, getConfig().atomicTaskState)
      },
      [ELEPHANTNOTE_API_ACTIONS.TASKS_RUN]: async (payload) => runProgrammaticTask(payload),
      [ELEPHANTNOTE_API_ACTIONS.PROGRAMS_LIST]: async () => getConfig().programEnvironments,
      [ELEPHANTNOTE_API_ACTIONS.PROGRAMS_SET]: async ({ environments = {} }) => {
        const config = getConfig()
        config.programEnvironments = normalizeProgramEnvironments(environments)
        setConfig(config)
        return getConfig().programEnvironments
      },
      [ELEPHANTNOTE_API_ACTIONS.PROGRAMS_RUN]: async (payload) => runProgram(payload),
      [ELEPHANTNOTE_API_ACTIONS.SYNC_STATUS]: async () => {
        log.info('[sync] api:status')
        return syncEngine.status()
      },
      [ELEPHANTNOTE_API_ACTIONS.SYNC_ENQUEUE]: async ({ operation, payload = {} }) => {
        log.info('[sync] api:enqueue', { operation })
        return syncEngine.enqueue({ operation, payload })
      },
      [ELEPHANTNOTE_API_ACTIONS.SYNC_RUN]: async (payload = {}) => {
        log.info('[sync] api:run', {
          hasInit: Boolean(payload.init),
          hasSnapshot: Boolean(payload.snapshot)
        })
        syncEngine.enqueuePlan(payload)
        return syncEngine.run()
      }
    }
  })
}

export const registerElephantNoteIpc = () => {
  const api = createElephantNoteMainApi()
  registerElephantNoteApiIpc({ ipcMain, api })
  registerSearchIpc()
  registerSitePreviewIpc()
  registerElephantNoteAgentIpc()
  registerModelLibraryIpc({ ipcMain })

  registerCompatibilityElephantNoteIpc({ ipcMain, api })
}
