<template>
  <div class="en-editor-layer">
    <section class="en-editor-panel" :style="editorLayoutStyle">
      <note-editor-top-bar
        :title="noteTitle"
        :note-date="noteDate"
        :tags="tags"
        :show-tag-hash="preferencesStore.showTagHashInEditor"
        :is-pinned="isPinned"
        :is-adding-tag="isAddingTag"
        :is-editing-tag="isEditingTag"
        :tag-draft="tagDraft"
        @update-title="updateTitle"
        @toggle-pin="togglePin"
        @close="closeOpenedNote"
        @start-tag-creation="startTagCreation"
        @edit-tag="beginEditTag"
        @delete-tag="deleteTag"
        @update-tag-draft="tagDraft = $event"
        @submit-tag="submitTag"
        @cancel-tag="cancelTag"
      />

      <div class="en-note-editor-shell">
        <div class="en-editor-host">
          <editor-with-tabs
            :markdown="markdown"
            :cursor="cursor"
            :muya-index-cursor="muyaIndexCursor"
            :source-code="sourceCode"
            :show-tab-bar="false"
            :text-direction="textDirection"
            :platform="platform"
            :to-editor-markdown="documentToEditorMarkdown"
            :from-editor-markdown="editorToDocumentMarkdown"
          />
        </div>
      </div>

      <note-editor-footer
        v-if="showEditorFooter"
        :word-count="wordCount"
        :character-count="characterCount"
        :is-typography-open="isTypographyOpen"
        :theme-icon="themeIcon"
        @toggle-typography="isTypographyOpen = !isTypographyOpen"
        @set-text-scale="setTextScale"
        @toggle-theme="toggleTheme"
      />
    </section>
  </div>
</template>

<script setup>
import { computed, inject, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Moon, SunMedium } from '@lucide/vue'
import { storeToRefs } from 'pinia'
import EditorWithTabs from '@/components/editorWithTabs'
import { useMainStore } from '@/store'
import { usePreferencesStore } from '@/store/preferences'
import { useEditorStore } from '@/store/editor'
import { useAddonsStore } from '@/store/addons'
import { useVaultStore } from '../../stores/vaultStore'
import NoteEditorFooter from './NoteEditorFooter.vue'
import NoteEditorTopBar from './NoteEditorTopBar.vue'
import { elephantnoteClient } from '../../services/elephantnoteClient'
import { formatShortDate } from '../../services/markdownMetaService'
import {
  getEditorMarkdownStats,
  getDocumentCreatedAt,
  getDocumentTitle,
  mergeEditorMarkdown,
  renameDocumentTitle,
  toEditorMarkdown
} from '../../utils/noteDocument'
import { parseMarkdownTags, updateMarkdownTags } from '../../utils/markdownTags'
import { getOppositeThemeVariant, getThemeMode } from 'common/elephantnote/appearance'
import { useSearchStore } from '../../stores/searchStore'
import { resolveLocalImageSource, toMarkdownImageSource } from 'elephant-shared/imageSource'
import {
  ELEPHANTNOTE_ASSETS_DIR,
  isHiddenAssetPath,
  sanitizeAssetName
} from 'elephant-shared/hiddenAssets'

const mainStore = useMainStore()
const editorStore = useEditorStore()
const preferencesStore = usePreferencesStore()
const addonsStore = useAddonsStore()
const store = useVaultStore()
const searchStore = useSearchStore()

const { platform } = storeToRefs(mainStore)
const { sourceCode, textDirection } = storeToRefs(preferencesStore)
const { currentFile } = storeToRefs(editorStore)

const AUTOSAVE_POLL_MS = 500
const AUTOSAVE_DELAY_MS = 160
const LARGE_EDIT_AUTOSAVE_DELAY_MS = 60
const HUGE_EDIT_AUTOSAVE_DELAY_MS = 20
const LARGE_EDIT_BYTES = 8 * 1024
const HUGE_EDIT_BYTES = 64 * 1024
const LOCAL_ASSET_EXTENSION_RE = /\.(?:png|jpe?g|gif|webp|svg|avif|bmp|ico)(?:[?#].*)?$/i
const MARKDOWN_IMAGE_RE = /(!\[[^\]]*\]\()([^)]*)(\))/g
const isAutosaveDebugEnabled = () =>
  window.localStorage.getItem('elephantnote:debugAutosave') === 'true'
const pushEditorLog = (level, message, details = {}) => {
  const entry = { at: new Date().toISOString(), level, message, details }
  window.__ELEPHANT_DEBUG_LOGS__ = Array.isArray(window.__ELEPHANT_DEBUG_LOGS__)
    ? window.__ELEPHANT_DEBUG_LOGS__
    : []
  window.__ELEPHANT_DEBUG_LOGS__.push(entry)
  if (window.__ELEPHANT_DEBUG_LOGS__.length > 1000) {
    window.__ELEPHANT_DEBUG_LOGS__.splice(0, window.__ELEPHANT_DEBUG_LOGS__.length - 1000)
  }
  const logger = console[level] || console.log
  logger.call(console, message, details)
}
const estimateEditDelta = (previousMarkdown = '', nextMarkdown = '') => {
  if (typeof nextMarkdown !== 'string') return 0
  if (typeof previousMarkdown !== 'string') return nextMarkdown.length
  return Math.abs(nextMarkdown.length - previousMarkdown.length)
}
const autosaveDelayFor = (_value, requestedDelay = AUTOSAVE_DELAY_MS, editDelta = 0) => {
  if (requestedDelay === 0) return 0
  const delta = Math.abs(Number(editDelta) || 0)
  if (delta >= HUGE_EDIT_BYTES) return Math.min(requestedDelay, HUGE_EDIT_AUTOSAVE_DELAY_MS)
  if (delta >= LARGE_EDIT_BYTES) return Math.min(requestedDelay, LARGE_EDIT_AUTOSAVE_DELAY_MS)
  return requestedDelay
}
const logAutosave = (level, message, details) => {
  pushEditorLog(level, message, details)
  if (!isAutosaveDebugEnabled()) return
  console[level]?.(message, details)
}

const isAddingTag = ref(false)
const isEditingTag = ref(false)
const editingTagIndex = ref(-1)
const tagDraft = ref('')
const isTypographyOpen = ref(false)
const textScale = ref(window.localStorage.getItem('elephantnote:editorTextScale') || 'normal')
const shellTheme = inject(
  'elephantnoteTheme',
  ref(window.localStorage.getItem('elephantnote:theme') || 'light')
)
const setShellTheme = inject('setElephantnoteTheme', (value) => {
  window.localStorage.setItem('elephantnote:theme', value)
})
let noteSaveTimer = null
let noteSaveInterval = null
let noteSaveInFlight = false
let pendingSaveAfterFlight = null
let lastSavedNotePath = ''
let lastSavedMarkdown = ''
let lastSeenNotePath = ''
let lastSeenMarkdown = ''

const editorExtensions = computed(() => addonsStore.getContributions('editor.extensions')
  .map((entry) => entry?.contribution)
  .filter(Boolean))
const openedNoteAbsolutePath = computed(() => {
  if (!store.activeVault?.path || !store.openedNotePath) return ''
  return window.path.join(store.activeVault.path, store.openedNotePath)
})
const findTabByPath = (pathname) =>
  editorStore.tabs.find(
    (tab) => tab?.pathname && window.fileUtils.isSamePathSync(tab.pathname, pathname)
  ) || null
const getActiveNoteFile = () => {
  const pathname = openedNoteAbsolutePath.value
  if (!pathname) return currentFile.value
  const tab = findTabByPath(pathname)
  if (tab) return tab
  if (
    currentFile.value?.pathname &&
    window.fileUtils.isSamePathSync(currentFile.value.pathname, pathname)
  ) {
    return currentFile.value
  }
  return null
}
const activeNoteFile = computed(
  () => getActiveNoteFile() || (openedNoteAbsolutePath.value ? null : currentFile.value)
)
const markdown = computed(() => {
  if (typeof activeNoteFile.value?.markdown === 'string') return activeNoteFile.value.markdown
  if (typeof currentFile.value?.markdown === 'string') {
    pushEditorLog('warn', '[elephantnote:editor] active-note-fallback-current-file', {
      openedNotePath: store.openedNotePath || null,
      currentFileId: currentFile.value?.id || null,
      currentFilePath: currentFile.value?.pathname || null,
      markdownLength: currentFile.value.markdown.length
    })
    return currentFile.value.markdown
  }
  return ''
})
const cursor = computed(() => activeNoteFile.value?.cursor || currentFile.value?.cursor || {})
const muyaIndexCursor = computed(() => activeNoteFile.value?.muyaIndexCursor || currentFile.value?.muyaIndexCursor || {})
const fallbackTitle = computed(
  () => activeNoteFile.value?.filename?.replace(/\.md$/i, '') || currentFile.value?.filename?.replace(/\.md$/i, '') || 'Untitled'
)
const documentToEditorMarkdown = (documentMarkdown) =>
  toEditorMarkdown(documentMarkdown, fallbackTitle.value)
const editorToDocumentMarkdown = (editorMarkdown) =>
  mergeEditorMarkdown(markdown.value, editorMarkdown, fallbackTitle.value)
const visibleMarkdown = computed(() => documentToEditorMarkdown(markdown.value))
const documentMeta = computed(() => {
  const content = markdown.value || ''
  const createdAt = getDocumentCreatedAt(content)
  return {
    title: getDocumentTitle(content, fallbackTitle.value),
    tags: parseMarkdownTags(content),
    date: createdAt ? formatShortDate(createdAt) : formatShortDate(new Date())
  }
})
const noteTitle = computed(() => documentMeta.value.title)
const tags = computed(() => documentMeta.value.tags)
const noteDate = computed(() => documentMeta.value.date)
const editorMarkdownStats = computed(() => getEditorMarkdownStats(visibleMarkdown.value))
const wordCount = computed(() => editorMarkdownStats.value.word)
const characterCount = computed(() => editorMarkdownStats.value.character)
const showEditorFooter = computed(() => preferencesStore.showEditorFooter === true)
const editorMarginPx = computed(() => {
  const value = Number(preferencesStore.noteEditorMargin)
  if (!Number.isFinite(value)) return 24
  return Math.max(8, Math.min(160, Math.round(value)))
})
const editorLayoutStyle = computed(() => ({
  '--en-note-editor-gutter': `${editorMarginPx.value}px`,
  '--en-note-editor-gutter-left': `${Math.min(168, editorMarginPx.value + 8)}px`,
  '--en-note-editor-gutter-right': `${editorMarginPx.value}px`
}))
const themeIcon = computed(() => (getThemeMode(shellTheme.value) === 'dark' ? SunMedium : Moon))
const currentNoteRelativePath = computed(() => {
  if (store.openedNotePath) return store.openedNotePath
  const pathname = currentFile.value?.pathname
  const vaultPath = store.activeVault?.path
  if (!pathname || !vaultPath) return ''
  const relativePath = window.path.relative(vaultPath, pathname)
  if (!relativePath || relativePath.startsWith('..') || window.path.isAbsolute(relativePath))
    return ''
  return relativePath
})
const relativePathForFile = (file) => {
  const pathname = file?.pathname
  const vaultPath = store.activeVault?.path
  if (!pathname || !vaultPath) return ''
  const normalizePrivatePath = (value) => String(value || '').replace(/^\/private\//, '/')
  const relativePath = window.path.relative(
    normalizePrivatePath(vaultPath),
    normalizePrivatePath(pathname)
  )
  if (!relativePath || relativePath.startsWith('..') || window.path.isAbsolute(relativePath)) return ''
  return relativePath
}
const isPinned = computed(() => {
  const pathname = currentNoteRelativePath.value
  return !!pathname && store.pinnedNotePaths.includes(pathname)
})
const currentNoteDirectory = computed(() => {
  const pathname =
    activeNoteFile.value?.pathname || currentFile.value?.pathname || openedNoteAbsolutePath.value
  if (pathname) return window.path.dirname(pathname)
  if (store.activeVault?.path) {
    return window.path.join(store.activeVault.path, store.currentPath || '')
  }
  return ''
})
const vaultAssetsDirectory = computed(() =>
  store.activeVault?.path ? window.path.join(store.activeVault.path, ELEPHANTNOTE_ASSETS_DIR) : ''
)

const entryArray = (value) => Array.isArray(value) ? value : []

const applyNoteMetadata = (entry, pathname, metadata = {}) => {
  if (!entry || entry.path !== pathname) return entry
  return {
    ...entry,
    title: metadata.title || entry.title,
    tags: Array.isArray(metadata.tags) ? metadata.tags : entry.tags,
    updatedAt: metadata.updatedAt || entry.updatedAt
  }
}

const syncVisibleNoteMetadata = (pathname, metadata = {}) => {
  if (!pathname || !Object.keys(metadata).length) return
  if (typeof store.updateNoteMetadata === 'function') {
    store.updateNoteMetadata(pathname, metadata)
  } else {
    store.entries = entryArray(store.entries).map((entry) => applyNoteMetadata(entry, pathname, metadata))
    store.openedNotes = entryArray(store.openedNotes).map((entry) =>
      applyNoteMetadata(entry, pathname, metadata)
    )
  }
  store.rootEntries = entryArray(store.rootEntries).map((entry) => applyNoteMetadata(entry, pathname, metadata))
}

const getNoteParentPath = (relativePath = '') => {
  const parentPath = window.path.dirname(relativePath)
  return parentPath === '.' ? '' : parentPath
}

const markFileSavedIfCurrent = (file, notePath, savedMarkdown) => {
  if (!file || file.markdown !== savedMarkdown) return
  file.isSaved = true
  if (currentFile.value?.id === file.id && currentFile.value.markdown === savedMarkdown) {
    currentFile.value.isSaved = true
  }
  if (file.id) {
    window.tauri?.ipcRenderer?.send?.('mt::tab-saved', file.id)
  }
  lastSavedNotePath = notePath
  lastSavedMarkdown = savedMarkdown
}

const refreshSavedEntries = async (notePath, result) => {
  const parentPath = getNoteParentPath(notePath)
  if (Array.isArray(result?.entries)) {
    if (parentPath === store.currentPath) {
      store.entries = result.entries
    }
    if (!parentPath) {
      store.rootEntries = result.entries
      return
    }
  }
  try {
    store.rootEntries = entryArray(await elephantnoteClient.directory.list({
      relativePath: '',
      limit: 121,
      includePreview: true
    }))
    if (parentPath === store.currentPath) {
      store.entries = entryArray(await elephantnoteClient.directory.list({
        relativePath: store.currentPath,
        limit: 121,
        includePreview: true
      }))
    }
  } catch (error) {
    pushEditorLog('warn', '[elephantnote:save] unable to refresh entries after save', {
      error: error?.message || String(error)
    })
  }
}

const pathExists = (pathname) => !!pathname && !!window.fileUtils?.pathExistsSync?.(pathname)
const isUntitledPlaceholder = (notePath = '', markdown = '') => {
  const title = String(notePath).split('/').pop()?.replace(/\.md$/i, '') || ''
  return /^Untitled(?: \d+)?$/i.test(title) && String(markdown).trim() === `# ${title}`
}
const normalizeSlashPath = (pathname = '') => String(pathname || '').replace(/\\/g, '/')
const isExternalAssetReference = (value = '') =>
  /^(?:https?:|data:|blob:|#)/i.test(String(value || '').trim())
const isVaultAssetAbsolutePath = (pathname = '') => {
  const root = store.activeVault?.path
  if (!root || !pathname) return false
  const relative = normalizeSlashPath(window.path.relative(root, pathname))
  return isHiddenAssetPath(relative) && relative.split('/')[0] === ELEPHANTNOTE_ASSETS_DIR
}
const ensureVaultAssetsDirectory = async () => {
  if (!vaultAssetsDirectory.value)
    throw new Error('Cannot resolve vault .assets directory without an active vault.')
  await window.fileUtils.ensureDir(vaultAssetsDirectory.value)
  return vaultAssetsDirectory.value
}
const uniqueVaultAssetPath = async (preferredName = 'asset') => {
  const directory = await ensureVaultAssetsDirectory()
  const safeName = sanitizeAssetName(preferredName, 'asset')
  const extension = window.path.extname(safeName)
  const baseName = extension ? safeName.slice(0, -extension.length) : safeName
  let index = 0
  let candidate = window.path.join(directory, safeName)
  while (pathExists(candidate)) {
    index += 1
    candidate = window.path.join(directory, `${baseName}-${index}${extension}`)
  }
  return candidate
}
const assetMarkdownSource = (targetPath) =>
  toMarkdownImageSource(targetPath, store.activeVault?.path || currentNoteDirectory.value)
const readLocalBlob = async (pathname, type = '') => {
  const content = await window.fileUtils.readFile(pathname)
  if (content instanceof Blob) return content
  return new Blob([content], type ? { type } : undefined)
}
const copyLocalFile = async (sourcePath, targetPath, type = '') => {
  await window.fileUtils.ensureDir(window.path.dirname(targetPath))
  if (normalizeSlashPath(sourcePath) === normalizeSlashPath(targetPath)) return targetPath
  if (typeof window.fileUtils?.copyFile === 'function') {
    await window.fileUtils.copyFile(sourcePath, targetPath)
    return targetPath
  }
  const blob = await readLocalBlob(sourcePath, type)
  await window.fileUtils.writeFile(targetPath, blob)
  return targetPath
}
const copyAddonAssetCompanions = async (sourcePath, targetPath) => {
  for (const extension of editorExtensions.value) {
    if (typeof extension.copyAssetCompanions !== 'function') continue
    try {
      await extension.copyAssetCompanions({
        sourcePath,
        targetPath,
        pathExists,
        copyFile: copyLocalFile,
        activeVaultPath: store.activeVault?.path || '',
        noteDirectory: currentNoteDirectory.value
      })
    } catch (error) {
      pushEditorLog('warn', '[elephantnote:assets] addon companion copy failed', {
        extensionId: extension.id || '',
        sourcePath,
        targetPath,
        error: error?.message || String(error)
      })
    }
  }
}
const copyLocalAssetIntoVault = async (sourcePath, preferredName = '') => {
  if (!sourcePath || isExternalAssetReference(sourcePath)) return ''
  if (isVaultAssetAbsolutePath(sourcePath)) return sourcePath
  if (!pathExists(sourcePath)) {
    pushEditorLog('warn', '[elephantnote:assets] source missing, cannot copy to .assets', {
      sourcePath
    })
    return ''
  }
  const targetPath = await uniqueVaultAssetPath(preferredName || window.path.basename(sourcePath))
  await copyLocalFile(sourcePath, targetPath)
  await copyAddonAssetCompanions(sourcePath, targetPath)
  pushEditorLog('info', '[elephantnote:assets] copied asset into hidden vault .assets', {
    sourcePath,
    targetPath
  })
  return targetPath
}
const parseMarkdownDestination = (raw = '') => {
  const value = String(raw || '').trim()
  if (!value) return { source: '', suffix: '' }
  if (value.startsWith('<')) {
    const end = value.indexOf('>')
    if (end > 0) return { source: value.slice(1, end), suffix: value.slice(end + 1) }
  }
  const match = value.match(/^(\S+)(.*)$/)
  return { source: match?.[1] || value, suffix: match?.[2] || '' }
}
const rewriteMarkdownAssetReferences = async (markdownText, notePath) => {
  if (!store.activeVault?.path || typeof markdownText !== 'string' || !markdownText.includes(']('))
    return markdownText
  const replacements = []
  for (const match of markdownText.matchAll(MARKDOWN_IMAGE_RE)) {
    const rawDestination = match[2]
    const { source, suffix } = parseMarkdownDestination(rawDestination)
    if (
      !source ||
      isExternalAssetReference(source) ||
      isHiddenAssetPath(source) ||
      !LOCAL_ASSET_EXTENSION_RE.test(source)
    )
      continue
    const sourcePath = resolveLocalImageSource(source, currentNoteDirectory.value)
    if (!sourcePath || isVaultAssetAbsolutePath(sourcePath)) continue
    const targetPath = await copyLocalAssetIntoVault(sourcePath, window.path.basename(sourcePath))
    if (!targetPath) continue
    replacements.push({
      start: match.index + match[1].length,
      end: match.index + match[1].length + rawDestination.length,
      replacement: `${assetMarkdownSource(targetPath)}${suffix}`,
      source,
      targetPath
    })
  }
  if (!replacements.length) return markdownText
  let next = markdownText
  for (const replacement of replacements.reverse()) {
    next = `${next.slice(0, replacement.start)}${replacement.replacement}${next.slice(replacement.end)}`
  }
  pushEditorLog('info', '[elephantnote:assets] rewrote markdown asset references to .assets', {
    notePath,
    count: replacements.length
  })
  return next
}
const syncAssetRewrittenMarkdown = (file, rewrittenMarkdown) => {
  if (!file || typeof rewrittenMarkdown !== 'string') return
  if (file.markdown !== rewrittenMarkdown) {
    file.markdown = rewrittenMarkdown
    file.isSaved = false
  }
  if (currentFile.value?.id === file.id && currentFile.value.markdown !== rewrittenMarkdown) {
    currentFile.value.markdown = rewrittenMarkdown
    currentFile.value.isSaved = false
  }
  lastSeenMarkdown = rewrittenMarkdown
}

const persistNoteMarkdown = async (
  notePath,
  nextMarkdown,
  file = activeNoteFile.value || currentFile.value,
  reason = 'unknown'
) => {
  if (!store.activeVault?.path || !notePath || typeof nextMarkdown !== 'string') return false
  if (isUntitledPlaceholder(notePath, nextMarkdown)) {
    pushEditorLog('info', '[elephantnote:save] skipped empty untitled placeholder', {
      notePath,
      reason
    })
    markFileSavedIfCurrent(file, notePath, nextMarkdown)
    return true
  }
  if (noteSaveInFlight) {
    pendingSaveAfterFlight = { notePath, nextMarkdown, file, reason }
    pushEditorLog('info', '[elephantnote:save] write queued while another save is in flight', {
      notePath,
      reason
    })
    return false
  }
  noteSaveInFlight = true
  logAutosave('info', '[elephantnote:save] write:start', {
    notePath,
    length: nextMarkdown.length,
    reason
  })
  let markdownToWrite = nextMarkdown
  try {
    markdownToWrite = await rewriteMarkdownAssetReferences(nextMarkdown, notePath)
    if (markdownToWrite !== nextMarkdown) syncAssetRewrittenMarkdown(file, markdownToWrite)
  } catch (assetError) {
    pushEditorLog('error', '[elephantnote:assets] markdown asset relocation failed before save', {
      notePath,
      error: assetError?.message || String(assetError)
    })
  }
  try {
    const result = await elephantnoteClient.notes.write({
      relativePath: notePath,
      markdown: markdownToWrite
    })
    await refreshSavedEntries(notePath, result)
    markFileSavedIfCurrent(file, notePath, markdownToWrite)
    logAutosave('info', '[elephantnote:save] write:done', {
      notePath,
      length: markdownToWrite.length,
      via: 'notes.write'
    })
    return true
  } catch (apiError) {
    pushEditorLog('warn', '[elephantnote:save] notes.write failed; trying direct file write', {
      notePath,
      error: apiError?.message || String(apiError)
    })
    try {
      await window.fileUtils.writeFile(
        window.path.join(store.activeVault.path, notePath),
        markdownToWrite
      )
      await refreshSavedEntries(notePath, null)
      markFileSavedIfCurrent(file, notePath, markdownToWrite)
      logAutosave('info', '[elephantnote:save] write:done', {
        notePath,
        length: markdownToWrite.length,
        via: 'fileUtils.writeFile'
      })
      return true
    } catch (fileError) {
      pushEditorLog('error', '[elephantnote:save] write:failed', {
        notePath,
        apiError: apiError?.message || String(apiError),
        fileError: fileError?.message || String(fileError)
      })
      if (file?.id) {
        window.tauri?.ipcRenderer?.send?.(
          'mt::tab-save-failure',
          file.id,
          fileError?.message || apiError?.message || 'Unable to save note.'
        )
      }
      return false
    }
  } finally {
    noteSaveInFlight = false
    const pending = pendingSaveAfterFlight
    pendingSaveAfterFlight = null
    if (
      pending &&
      (pending.notePath !== lastSavedNotePath || pending.nextMarkdown !== lastSavedMarkdown)
    ) {
      scheduleNoteSave(
        pending.notePath,
        pending.nextMarkdown,
        pending.file,
        0,
        `${pending.reason}:after-flight`
      )
    }
  }
}

const scheduleNoteSave = (
  notePath,
  nextMarkdown,
  file = activeNoteFile.value || currentFile.value,
  delay = AUTOSAVE_DELAY_MS,
  reason = 'unknown',
  editDelta = 0
) => {
  if (!notePath || typeof nextMarkdown !== 'string') return
  if (lastSavedNotePath === notePath && lastSavedMarkdown === nextMarkdown) return
  if (noteSaveTimer) window.clearTimeout(noteSaveTimer)
  const effectiveDelay = autosaveDelayFor(nextMarkdown, delay, editDelta)
  logAutosave('info', '[elephantnote:save] schedule', {
    notePath,
    length: nextMarkdown.length,
    editDelta,
    delay: effectiveDelay,
    reason
  })
  noteSaveTimer = window.setTimeout(() => {
    noteSaveTimer = null
    void persistNoteMarkdown(notePath, nextMarkdown, file, reason)
  }, effectiveDelay)
}

const rememberObservedMarkdown = (notePath, nextMarkdown, file, reason = 'observe') => {
  lastSeenNotePath = notePath
  lastSeenMarkdown = nextMarkdown
  pushEditorLog('info', '[elephantnote:save] observed active markdown', {
    notePath,
    length: nextMarkdown.length,
    reason,
    isSaved: file?.isSaved
  })
  if (file?.isSaved === false) {
    scheduleNoteSave(notePath, nextMarkdown, file, 0, `${reason}:first-unsaved`)
    return
  }
  lastSavedNotePath = notePath
  lastSavedMarkdown = nextMarkdown
}

const pollActiveMarkdownSave = (reason = 'poll') => {
  const file = getActiveNoteFile() || currentFile.value
  const notePath = relativePathForFile(file) || currentNoteRelativePath.value || store.openedNotePath
  const nextMarkdown = file?.markdown
  if (!notePath || !file?.id || typeof nextMarkdown !== 'string') return
  if (lastSeenNotePath !== notePath) {
    rememberObservedMarkdown(notePath, nextMarkdown, file, reason)
    return
  }
  if (lastSeenMarkdown === nextMarkdown) return
  const editDelta = estimateEditDelta(lastSeenMarkdown, nextMarkdown)
  lastSeenMarkdown = nextMarkdown
  scheduleNoteSave(notePath, nextMarkdown, file, AUTOSAVE_DELAY_MS, reason, editDelta)
}

const flushActiveNoteSave = async (reason = 'flush') => {
  if (noteSaveTimer) {
    window.clearTimeout(noteSaveTimer)
    noteSaveTimer = null
  }
  const file = getActiveNoteFile() || currentFile.value
  const notePath = relativePathForFile(file) || lastSeenNotePath || lastSavedNotePath || currentNoteRelativePath.value || store.openedNotePath
  const nextMarkdown = file?.markdown
  if (!notePath || !file?.id || typeof nextMarkdown !== 'string') return false
  if (lastSavedNotePath === notePath && lastSavedMarkdown === nextMarkdown) return true
  pushEditorLog('info', '[elephantnote:save] flush active note', {
    notePath,
    filePath: file?.pathname || null,
    openedNotePath: store.openedNotePath || null,
    reason,
    length: nextMarkdown.length
  })
  return persistNoteMarkdown(notePath, nextMarkdown, file, reason)
}

const closeOpenedNote = async () => {
  await flushActiveNoteSave('close-note')
  store.closeNote()
}

const selectOpenedNoteTab = () => {
  const pathname = openedNoteAbsolutePath.value
  if (!pathname || !editorStore.tabs?.length) return
  if (
    currentFile.value?.pathname &&
    window.fileUtils.isSamePathSync(currentFile.value.pathname, pathname)
  )
    return
  const hasTab = editorStore.tabs.some(
    (tab) => tab.pathname && window.fileUtils.isSamePathSync(tab.pathname, pathname)
  )
  if (hasTab) editorStore.SWITCH_TAB_BY_FILEPATH(pathname)
}

watch(openedNoteAbsolutePath, selectOpenedNoteTab, { immediate: true })
watch(() => editorStore.tabs.length, selectOpenedNoteTab)
watch(
  () => ({
    notePath: currentNoteRelativePath.value || store.openedNotePath,
    markdown: markdown.value,
    file: activeNoteFile.value || currentFile.value
  }),
  ({ notePath, markdown: nextMarkdown, file }, previous) => {
    if (!notePath || !file?.id || typeof nextMarkdown !== 'string') return
    if (previous?.notePath !== notePath) {
      rememberObservedMarkdown(notePath, nextMarkdown, file, 'vue-watch')
      return
    }
    if (previous?.markdown === nextMarkdown) return
    const editDelta = estimateEditDelta(previous?.markdown, nextMarkdown)
    lastSeenNotePath = notePath
    lastSeenMarkdown = nextMarkdown
    scheduleNoteSave(notePath, nextMarkdown, file, AUTOSAVE_DELAY_MS, 'vue-watch', editDelta)
  }
)

const updateCurrentFileMarkdown = (nextMarkdown, metadata = {}) => {
  const file = activeNoteFile.value || currentFile.value
  if (!file) return
  const previousMarkdown = file.markdown || markdown.value || ''
  file.markdown = nextMarkdown
  file.isSaved = false
  if (currentFile.value && file.id === currentFile.value.id) {
    currentFile.value.markdown = nextMarkdown
    currentFile.value.isSaved = false
  }
  const notePath = currentNoteRelativePath.value || store.openedNotePath
  if (notePath) {
    syncVisibleNoteMetadata(notePath, metadata)
    if (typeof searchStore.updateNoteIndex === 'function') {
      searchStore.updateNoteIndex(notePath, nextMarkdown, metadata)
    }
    lastSeenNotePath = notePath
    lastSeenMarkdown = nextMarkdown
    pushEditorLog('info', '[elephantnote:editor] markdown updated from UI action', {
      notePath,
      length: nextMarkdown.length,
      metadata
    })
    scheduleNoteSave(
      notePath,
      nextMarkdown,
      file,
      0,
      'toolbar-edit',
      estimateEditDelta(previousMarkdown, nextMarkdown)
    )
  }
}

const updateTitle = (nextTitle) => {
  const title = String(nextTitle || '').trim() || fallbackTitle.value
  updateCurrentFileMarkdown(renameDocumentTitle(markdown.value, title, fallbackTitle.value), {
    title
  })
}
const togglePin = () => {
  const pathname = currentNoteRelativePath.value
  if (!pathname) return
  const toggle = store.togglePin || store.togglePinnedNote || store.togglePinnedEntry
  if (typeof toggle === 'function') toggle.call(store, pathname)
}

const startTagCreation = () => {
  isAddingTag.value = true
  isEditingTag.value = false
  editingTagIndex.value = -1
  tagDraft.value = ''
}
const beginEditTag = (index) => {
  isEditingTag.value = true
  isAddingTag.value = false
  editingTagIndex.value = index
  tagDraft.value = tags.value[index] || ''
}
const cancelTag = () => {
  isAddingTag.value = false
  isEditingTag.value = false
  editingTagIndex.value = -1
  tagDraft.value = ''
}
const submitTag = () => {
  const tag = tagDraft.value.trim()
  if (!tag) {
    cancelTag()
    return
  }
  const nextTags = [...tags.value]
  if (isEditingTag.value && editingTagIndex.value >= 0) {
    nextTags[editingTagIndex.value] = tag
  } else if (!nextTags.includes(tag)) {
    nextTags.push(tag)
  }
  updateCurrentFileMarkdown(updateMarkdownTags(markdown.value, nextTags, noteTitle.value), {
    tags: nextTags
  })
  cancelTag()
}
const deleteTag = (index) => {
  const nextTags = tags.value.filter((_tag, currentIndex) => currentIndex !== index)
  updateCurrentFileMarkdown(updateMarkdownTags(markdown.value, nextTags, noteTitle.value), {
    tags: nextTags
  })
}
const setTextScale = (value) => {
  textScale.value = value
  window.localStorage.setItem('elephantnote:editorTextScale', value)
}
const toggleTheme = () => {
  const next = getOppositeThemeVariant(shellTheme.value)
  shellTheme.value = next
  setShellTheme(next)
}

watch(
  [currentNoteRelativePath, () => activeNoteFile.value?.id, () => currentFile.value?.id, () => store.openedNotePath],
  ([notePath, activeFileId, currentFileId, openedPath], previous) => {
    pushEditorLog('info', '[elephantnote:editor] identity-change', {
      notePath: notePath || null,
      activeFileId: activeFileId || null,
      currentFileId: currentFileId || null,
      openedPath: openedPath || null,
      previous: previous ? {
        notePath: previous[0] || null,
        activeFileId: previous[1] || null,
        currentFileId: previous[2] || null,
        openedPath: previous[3] || null
      } : null,
      tabCount: editorStore.tabs.length,
      activeVaultId: store.activeVaultId || null
    })
    pushEditorLog('info', '[elephantnote:editor] input-state', {
      notePath: notePath || null,
      activeFileId: activeFileId || null,
      markdownLength: typeof activeNoteFile.value?.markdown === 'string'
        ? activeNoteFile.value.markdown.length
        : null,
      visibleMarkdownLength: visibleMarkdown.value.length,
      hasEditorDocument: Boolean(activeFileId && typeof activeNoteFile.value?.markdown === 'string')
    })
  },
  { immediate: true }
)

onMounted(() => {
  pushEditorLog('info', '[elephantnote:editor] mounted', {
    notePath: currentNoteRelativePath.value,
    vault: store.activeVault?.path
  })
  pollActiveMarkdownSave('mount')
  noteSaveInterval = window.setInterval(() => pollActiveMarkdownSave('interval'), AUTOSAVE_POLL_MS)
})
onBeforeUnmount(() => {
  pushEditorLog('info', '[elephantnote:editor] before unmount', {
    notePath: currentNoteRelativePath.value
  })
  if (noteSaveInterval) {
    window.clearInterval(noteSaveInterval)
    noteSaveInterval = null
  }
  void flushActiveNoteSave('unmount')
})
</script>
