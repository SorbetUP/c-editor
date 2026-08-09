<template>
  <ExcalidrawDialog
    v-if="isOpen"
    :title="title"
    :file-name="fileName"
    :theme="shellTheme"
    :initial-blob="initialBlob"
    :save-mode="saveMode"
    :insert-on-save="insertOnSave"
    @close="close"
    @save="save"
  />
</template>

<script setup>
import { computed, inject, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import bus from '@/bus'
import { useEditorStore } from '@/store/editor'
import ExcalidrawDialog from 'elephant-front/components/editor/ExcalidrawDialog.vue'
import { useVaultStore } from 'elephant-front/stores/vaultStore'
import { getExcalidrawPreviewPath, getExcalidrawScenePath } from 'elephant-front/services/excalidraw'
import { resolveLocalImageSource, toMarkdownImageSource } from 'elephant-shared/imageSource'
import {
  ELEPHANTNOTE_ASSETS_DIR,
  getVaultAssetRelativePath,
  isHiddenAssetPath,
  sanitizeAssetName
} from 'elephant-shared/excalidrawAssets'

const editorStore = useEditorStore()
const vaultStore = useVaultStore()
const { currentFile } = storeToRefs(editorStore)
const shellTheme = inject('elephantnoteTheme', ref(window.localStorage.getItem('elephantnote:theme') || 'light'))
const MAX_DIAGNOSTIC_LOGS = 1000

const isOpen = ref(false)
const initialBlob = ref(null)
const targetPath = ref('')
const scenePath = ref('')
const insertOnSave = ref(false)
const fileName = ref('excalidraw.png')
const title = ref('Excalidraw')
const saveMode = ref('png')

const activeFile = computed(() => {
  const openedPath = vaultStore.openedNotePath
  if (openedPath && vaultStore.activeVault?.path) {
    const absolutePath = window.path.join(vaultStore.activeVault.path, openedPath)
    const tab = editorStore.tabs.find((entry) => entry?.pathname && window.fileUtils.isSamePathSync(entry.pathname, absolutePath))
    if (tab) return tab
  }
  return currentFile.value || null
})

const currentNoteDirectory = computed(() => {
  const pathname = activeFile.value?.pathname
  if (pathname) return window.path.dirname(pathname)
  if (vaultStore.activeVault?.path) return window.path.join(vaultStore.activeVault.path, vaultStore.currentPath || '')
  return ''
})

const errorDetails = (error) => ({
  name: error?.name || 'Error',
  message: error?.message || String(error || ''),
  stack: error?.stack || ''
})

const log = (level, message, details = {}) => {
  const entry = {
    time: new Date().toISOString(),
    level,
    message: `[excalidraw-addon] ${message}`,
    details
  }
  window.__ELEPHANT_DEBUG_LOGS__ = Array.isArray(window.__ELEPHANT_DEBUG_LOGS__)
    ? window.__ELEPHANT_DEBUG_LOGS__
    : []
  window.__ELEPHANT_DEBUG_LOGS__.push(entry)
  if (window.__ELEPHANT_DEBUG_LOGS__.length > MAX_DIAGNOSTIC_LOGS) {
    window.__ELEPHANT_DEBUG_LOGS__.splice(0, window.__ELEPHANT_DEBUG_LOGS__.length - MAX_DIAGNOSTIC_LOGS)
  }
  const logger = console[level] || console.log
  logger.call(console, entry.message, details)
  return entry
}

const normalizeSlashPath = (pathname = '') => String(pathname || '').replace(/\\/g, '/')
const stripQueryAndHash = (value = '') => String(value || '').split(/[?#]/)[0]
const decodeSource = (value = '') => {
  try {
    return decodeURI(stripQueryAndHash(value))
  } catch {
    return stripQueryAndHash(value)
  }
}

const resolveDrawingPath = (source = '') => {
  const decoded = normalizeSlashPath(decodeSource(source))
  const root = vaultStore.activeVault?.path || ''
  const marker = `${ELEPHANTNOTE_ASSETS_DIR}/`
  const markerIndex = decoded.indexOf(marker)
  if (root && markerIndex >= 0) {
    return window.path.join(root, decoded.slice(markerIndex))
  }
  return resolveLocalImageSource(decoded, currentNoteDirectory.value)
}

const pathExists = async (pathname) => {
  if (!pathname) return false
  let cached = null
  try {
    cached = typeof window.fileUtils?.pathExistsSync === 'function'
      ? Boolean(window.fileUtils.pathExistsSync(pathname))
      : null
  } catch (error) {
    log('warn', 'cached path probe failed', { pathname, error: errorDetails(error) })
  }

  if (typeof window.fileUtils?.stat === 'function') {
    try {
      const info = await window.fileUtils.stat(pathname)
      const exists = Boolean(
        info?.isFile === true ||
        info?.isDirectory === true ||
        Number.isFinite(info?.size)
      )
      log('info', 'filesystem path probe', {
        pathname,
        cached,
        exists,
        isFile: info?.isFile ?? null,
        isDirectory: info?.isDirectory ?? null,
        size: Number.isFinite(info?.size) ? info.size : null
      })
      return exists
    } catch (error) {
      log('warn', 'filesystem path probe failed', {
        pathname,
        cached,
        error: errorDetails(error)
      })
      return false
    }
  }

  log('warn', 'filesystem stat unavailable; using metadata cache only', { pathname, cached })
  return cached === true
}

const isVaultAssetAbsolutePath = (pathname = '') => {
  const root = vaultStore.activeVault?.path
  if (!root || !pathname) return false
  const relative = normalizeSlashPath(window.path.relative(root, pathname))
  return isHiddenAssetPath(relative) && relative.split('/')[0] === ELEPHANTNOTE_ASSETS_DIR
}

const ensureAssetsDirectory = async () => {
  const root = vaultStore.activeVault?.path
  if (!root) throw new Error('Cannot use Excalidraw without an active vault.')
  const directory = window.path.join(root, ELEPHANTNOTE_ASSETS_DIR)
  log('info', 'ensuring assets directory', { directory })
  await window.fileUtils.ensureDir(directory)
  return directory
}

const assetName = (value = '') => sanitizeAssetName(
  value || `excalidraw-${Date.now()}.png`,
  `excalidraw-${Date.now()}.png`
)

const uniqueAssetPath = async (preferredName = '') => {
  const directory = await ensureAssetsDirectory()
  const safeName = assetName(preferredName)
  const extension = window.path.extname(safeName)
  const baseName = extension ? safeName.slice(0, -extension.length) : safeName
  let index = 0
  let candidate = window.path.join(directory, safeName)
  while (await pathExists(candidate)) {
    index += 1
    candidate = window.path.join(directory, `${baseName}-${index}${extension}`)
  }
  log('info', 'selected unique asset path', { preferredName, candidate, collisionCount: index })
  return candidate
}

const readBlob = async (pathname, type = '') => {
  log('info', 'file read:start', { pathname, type })
  try {
    const content = await window.fileUtils.readFile(pathname)
    const blob = content instanceof Blob ? content : new Blob([content], type ? { type } : undefined)
    if (!blob.size) throw new Error('Excalidraw file read returned an empty payload.')
    log('info', 'file read:success', { pathname, type: blob.type || type, size: blob.size })
    return blob
  } catch (error) {
    log('error', 'file read:failed', { pathname, type, error: errorDetails(error) })
    throw error
  }
}

const copyFile = async (source, target, type = '') => {
  log('info', 'file copy:start', { source, target, type })
  await window.fileUtils.ensureDir(window.path.dirname(target))
  if (normalizeSlashPath(source) === normalizeSlashPath(target)) {
    log('info', 'file copy skipped because source equals target', { source, target })
    return target
  }
  try {
    if (typeof window.fileUtils?.copyFile === 'function') {
      await window.fileUtils.copyFile(source, target)
    } else {
      await window.fileUtils.writeFile(target, await readBlob(source, type))
    }
    log('info', 'file copy:success', { source, target, type })
    return target
  } catch (error) {
    log('error', 'file copy:failed', { source, target, type, error: errorDetails(error) })
    throw error
  }
}

const copyDrawingIntoVault = async (sourcePreviewPath) => {
  if (!sourcePreviewPath) return ''
  if (!(await pathExists(sourcePreviewPath))) {
    log('error', 'drawing preview does not exist', { sourcePreviewPath })
    return ''
  }
  if (isVaultAssetAbsolutePath(sourcePreviewPath)) {
    log('info', 'drawing preview already belongs to active vault', { sourcePreviewPath })
    return sourcePreviewPath
  }
  const destination = await uniqueAssetPath(window.path.basename(sourcePreviewPath))
  await copyFile(sourcePreviewPath, destination)
  const sourceScene = getExcalidrawScenePath(sourcePreviewPath)
  if (sourceScene && sourceScene !== sourcePreviewPath && await pathExists(sourceScene)) {
    await copyFile(sourceScene, getExcalidrawScenePath(destination), 'application/vnd.excalidraw+json')
  }
  log('info', 'drawing copied into active vault', { sourcePreviewPath, destination })
  return destination
}

const targetAssetPath = async (preferredName = '') => {
  const root = vaultStore.activeVault?.path
  if (!root) throw new Error('Cannot use Excalidraw without an active vault.')
  await ensureAssetsDirectory()
  const destination = window.path.join(root, getVaultAssetRelativePath(assetName(preferredName)))
  log('info', 'resolved target asset path', { preferredName, destination })
  return destination
}

const open = async ({ markdown, fileName: requestedName, title: requestedTitle, saveMode: requestedMode, insertOnSave: shouldInsert } = {}) => {
  try {
    const nextName = assetName(requestedName || `excalidraw-${Date.now()}.png`)
    const nextTarget = await targetAssetPath(nextName)
    targetPath.value = nextTarget
    scenePath.value = getExcalidrawScenePath(nextTarget)
    title.value = requestedTitle || 'Excalidraw'
    fileName.value = nextName
    saveMode.value = requestedMode || 'png'
    insertOnSave.value = Boolean(shouldInsert)
    initialBlob.value = markdown instanceof Blob ? markdown : null
    isOpen.value = true
    log('info', 'editor opened', {
      targetPath: nextTarget,
      scenePath: scenePath.value,
      insertOnSave: insertOnSave.value,
      initialBlobSize: initialBlob.value?.size || 0
    })
  } catch (error) {
    log('error', 'editor open failed', { error: errorDetails(error) })
    throw error
  }
}

const openFromImage = async (src) => {
  log('info', 'open existing drawing:start', {
    source: src,
    noteDirectory: currentNoteDirectory.value,
    vaultRoot: vaultStore.activeVault?.path || ''
  })
  try {
    const imagePath = resolveDrawingPath(src)
    if (!imagePath) throw new Error('Unable to resolve Excalidraw preview path.')
    const rawPreviewPath = window.path.extname(imagePath).toLowerCase() === '.excalidraw'
      ? getExcalidrawPreviewPath(imagePath)
      : imagePath
    const previewPath = await copyDrawingIntoVault(rawPreviewPath)
    if (!previewPath) throw new Error(`Excalidraw preview is unavailable: ${rawPreviewPath}`)
    const nextScenePath = getExcalidrawScenePath(previewPath)
    const sceneExists = await pathExists(nextScenePath)
    const previewExists = sceneExists ? true : await pathExists(previewPath)
    const blob = sceneExists
      ? await readBlob(nextScenePath, 'application/vnd.excalidraw+json')
      : previewExists ? await readBlob(previewPath) : null

    if (!blob) throw new Error('Neither the Excalidraw scene nor its PNG preview could be read.')

    await open({
      markdown: blob,
      fileName: window.path.basename(previewPath),
      title: 'Excalidraw',
      saveMode: 'png',
      insertOnSave: false
    })
    targetPath.value = previewPath
    scenePath.value = nextScenePath
    log('info', 'open existing drawing:success', {
      source: src,
      imagePath,
      previewPath,
      scenePath: nextScenePath,
      sceneExists,
      previewExists,
      payloadSize: blob.size
    })
  } catch (error) {
    log('error', 'open existing drawing:failed', {
      source: src,
      error: errorDetails(error)
    })
  }
}

const close = () => {
  log('info', 'editor closed', { targetPath: targetPath.value, scenePath: scenePath.value, reason: 'user-or-navigation' })
  isOpen.value = false
  initialBlob.value = null
}

watch(() => vaultStore.openedNotePath, (nextPath, previousPath) => {
  if (!isOpen.value || nextPath === previousPath) return
  close()
})

const appendPreviewToNote = (previewPath, resolvedName) => {
  const file = activeFile.value
  if (!file) {
    log('error', 'cannot insert preview because no active editor file exists', { previewPath, resolvedName })
    return false
  }
  const source = toMarkdownImageSource(previewPath, vaultStore.activeVault?.path || currentNoteDirectory.value)
  const imageMarkdown = `![${resolvedName}](${source})`
  const nextMarkdown = [String(file.markdown || '').trimEnd(), imageMarkdown].filter(Boolean).join('\n\n')
  file.markdown = nextMarkdown
  file.isSaved = false
  if (currentFile.value?.id === file.id) {
    currentFile.value.markdown = nextMarkdown
    currentFile.value.isSaved = false
  }
  log('info', 'preview inserted into active note', {
    previewPath,
    markdownSource: source,
    notePath: file.pathname || null,
    markdownLength: nextMarkdown.length
  })
  return true
}

const save = async ({ imageBlob, blob, sceneBlob, fileName: requestedName } = {}) => {
  const writableImage = imageBlob || blob
  if (!writableImage) throw new Error('Excalidraw did not provide an image payload.')
  const resolvedName = assetName(requestedName || fileName.value)
  const destination = targetPath.value && window.path.basename(targetPath.value) === resolvedName
    ? targetPath.value
    : await targetAssetPath(resolvedName)
  const destinationScene = getExcalidrawScenePath(destination)

  log('info', 'save:start', {
    destination,
    destinationScene,
    imageSize: writableImage?.size || 0,
    sceneSize: sceneBlob?.size || 0,
    insertOnSave: insertOnSave.value
  })

  try {
    await window.fileUtils.ensureDir(window.path.dirname(destination))
    await window.fileUtils.writeFile(destination, writableImage)
    if (sceneBlob) await window.fileUtils.writeFile(destinationScene, sceneBlob)
    targetPath.value = destination
    scenePath.value = destinationScene
    fileName.value = resolvedName
    const inserted = insertOnSave.value ? appendPreviewToNote(destination, resolvedName) : false
    log('info', 'save:success', {
      destination,
      destinationScene,
      inserted,
      imageSize: writableImage?.size || 0,
      sceneSize: sceneBlob?.size || 0
    })
    bus.emit('invalidate-image-cache')
    log('info', 'image cache invalidation emitted', { destination })
    close()
  } catch (error) {
    log('error', 'save:failed', {
      destination,
      destinationScene,
      error: errorDetails(error)
    })
    throw error
  }
}

onMounted(() => {
  bus.on('ELEPHANT::open-excalidraw', open)
  bus.on('open-excalidraw-from-image', openFromImage)
  log('info', 'overlay mounted', {
    vaultRoot: vaultStore.activeVault?.path || '',
    noteDirectory: currentNoteDirectory.value
  })
})

onBeforeUnmount(() => {
  bus.off('ELEPHANT::open-excalidraw', open)
  bus.off('open-excalidraw-from-image', openFromImage)
  close()
  log('info', 'overlay unmounted')
})
</script>
