import bus from '@/bus'
import ExcalidrawEditorOverlay from './ui/ExcalidrawEditorOverlay.vue'
import { getExcalidrawScenePath } from 'elephant-front/services/excalidraw'
import { useVaultStore } from 'elephant-front/stores/vaultStore'
import { installExcalidrawMarkdownCleanup } from '../../platform/excalidrawMarkdownCleanup'
import { installExcalidrawImageRuntimeFixes } from '../../platform/excalidrawImageRuntimeFixes'

const CORE_FEATURE_ID = 'core.excalidraw'
const EXCALIDRAW_ASSET_RE = /(?:^|\/)\.assets\/excalidraw-[^/?#]+\.png(?:[?#].*)?$/i
const EXCALIDRAW_ICON = 'data:image/svg+xml;base64,PHN2ZyB2aWV3Qm94PSIwIDAgNjQgNjQiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZyI+PHJlY3QgeD0iMyIgeT0iMyIgd2lkdGg9IjU4IiBoZWlnaHQ9IjU4IiByeD0iMTciIGZpbGw9IiM2OTY1REIiLz48cGF0aCBkPSJNMTguNSAzOS41IDM0LjggMTguOGMxLjctMi4xIDQuOC0yLjQgNi45LS43bDQuMiAzLjRjMi4xIDEuNyAyLjQgNC44LjcgNi45TDMwLjMgNDkuMWwtMTEuOCAyLjd2LTEyLjNaIiBzdHJva2U9IndoaXRlIiBzdHJva2Utd2lkdGg9IjQuNCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIvPjxwYXRoIGQ9Im0yMSAzOSA5LjYgNy41TTM1IDE5LjRsMTEgOC43IiBzdHJva2U9IndoaXRlIiBzdHJva2Utd2lkdGg9IjQuNCIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIi8+PC9zdmc+'

const normalizeSource = (value = '') => String(value || '')
  .replaceAll('\\', '/')
  .split(/[?#]/)[0]

const imageSource = (imageInfo = {}) => imageInfo.token?.attrs?.src || imageInfo.token?.src || ''
const isExcalidrawImage = (imageInfo) => EXCALIDRAW_ASSET_RE.test(normalizeSource(imageSource(imageInfo)))

const editDrawing = ({ imageInfo }) => {
  const src = imageSource(imageInfo)
  if (src) bus.emit('open-excalidraw-from-image', src)
}

const createDrawing = () => {
  bus.emit('ELEPHANT::open-excalidraw', {
    fileName: `excalidraw-${Date.now()}.png`,
    title: 'Excalidraw',
    saveMode: 'png',
    insertOnSave: true
  })
}

const copyDrawingSidecar = async ({ sourcePath, targetPath, pathExists, copyFile }) => {
  if (!EXCALIDRAW_ASSET_RE.test(normalizeSource(sourcePath))) return
  const sourceScenePath = getExcalidrawScenePath(sourcePath)
  if (!sourceScenePath || sourceScenePath === sourcePath || !pathExists(sourceScenePath)) return
  await copyFile(sourceScenePath, getExcalidrawScenePath(targetPath), 'application/vnd.excalidraw+json')
}

const dispatchLifecycle = (eventName) => {
  if (typeof globalThis.dispatchEvent !== 'function' || typeof CustomEvent !== 'function') return
  globalThis.dispatchEvent(new CustomEvent(eventName, { detail: { coreFeatureId: CORE_FEATURE_ID } }))
}

export const excalidrawCoreFeature = Object.freeze({
  id: CORE_FEATURE_ID,
  activate(ctx) {
    const vaultStore = useVaultStore()
    const getActiveVaultPath = () => vaultStore.activeVault?.path || ''
    globalThis.__ELEPHANT_GET_ACTIVE_VAULT_PATH__ = getActiveVaultPath
    globalThis.__ELEPHANT_EXCALIDRAW_ENABLED__ = true
    document.documentElement.dataset.elephantExcalidrawEnabled = 'true'
    const cleanupRuntime = installExcalidrawMarkdownCleanup()
    const imageRuntime = installExcalidrawImageRuntimeFixes(globalThis)

    ctx.registerContribution('layout.zones', {
      id: `${CORE_FEATURE_ID}.editor-overlay`,
      // Keep the editor dialog mounted at shell scope. NoteEditorHost is
      // intentionally replaced during navigation, and mounting the dialog
      // there made Escape/Close race with the host unmount and left users in
      // a full-screen Excalidraw surface.
      zone: 'shell.right',
      order: 40,
      component: ExcalidrawEditorOverlay
    })

    ctx.addEditorExtension({
      id: `${CORE_FEATURE_ID}.editor-integration`,
      imageToolbarItems: [{
        id: 'edit-drawing',
        tooltip: 'Edit Excalidraw drawing',
        icon: EXCALIDRAW_ICON,
        localOnly: true,
        when: ({ imageInfo }) => isExcalidrawImage(imageInfo),
        run: editDrawing
      }],
      quickInsertItems: [{
        group: 'Writing tools',
        title: 'Excalidraw',
        subTitle: 'Insert drawing',
        commandId: 'excalidraw',
        icon: EXCALIDRAW_ICON
      }],
      writingCommands: [{ id: 'excalidraw', run: createDrawing }],
      copyAssetCompanions: copyDrawingSidecar
    })

    globalThis.__ELEPHANT_ADDON_CONTENT_FALLBACKS__?.refresh?.()
    dispatchLifecycle('elephantnote:excalidraw-core-enabled')

    return () => {
      cleanupRuntime?.dispose?.()
      imageRuntime?.dispose?.()
      if (globalThis.__ELEPHANT_GET_ACTIVE_VAULT_PATH__ === getActiveVaultPath) {
        delete globalThis.__ELEPHANT_GET_ACTIVE_VAULT_PATH__
      }
      delete globalThis.__ELEPHANT_EXCALIDRAW_ENABLED__
      delete document.documentElement.dataset.elephantExcalidrawEnabled
      queueMicrotask(() => globalThis.__ELEPHANT_ADDON_CONTENT_FALLBACKS__?.refresh?.())
      dispatchLifecycle('elephantnote:excalidraw-core-disabled')
    }
  }
})
