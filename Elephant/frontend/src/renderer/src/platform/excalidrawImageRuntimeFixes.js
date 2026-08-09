import bus from '@/bus'
import { convertFileSrc } from '@tauri-apps/api/core'

const LOCAL_FILE_PREFIX = 'local-file://'
const EXCALIDRAW_ASSET_RE = /(?:^|\/)\.assets\/excalidraw-[^/?#]+\.png(?:[?#].*)?$/i
const INSTALLED_ATTR = 'data-elephant-excalidraw-edit-installed'
const CACHE_BUST_ATTR = 'data-elephant-excalidraw-cache-bust'
const MAX_DIAGNOSTIC_LOGS = 1000
const MAX_RESOLUTION_LOG_KEYS = 200

let cacheBustSerial = Date.now()
const loggedPathResolutions = new Set()

const normalizeSlashes = (value = '') => String(value || '').replace(/\\/g, '/')
const stripQueryAndHash = (value = '') => String(value || '').split(/[?#]/)[0]
const decodeSafe = (value = '') => {
  try {
    return decodeURIComponent(String(value || ''))
  } catch {
    return String(value || '')
  }
}

const errorDetails = (error) => ({
  name: error?.name || 'Error',
  message: error?.message || String(error || ''),
  stack: error?.stack || ''
})

const pushExcalidrawImageLog = (level, message, details = {}) => {
  const target = globalThis.window || globalThis
  const entry = {
    time: new Date().toISOString(),
    level,
    message: `[excalidraw-image] ${message}`,
    details
  }
  target.__ELEPHANT_DEBUG_LOGS__ = Array.isArray(target.__ELEPHANT_DEBUG_LOGS__)
    ? target.__ELEPHANT_DEBUG_LOGS__
    : []
  target.__ELEPHANT_DEBUG_LOGS__.push(entry)
  if (target.__ELEPHANT_DEBUG_LOGS__.length > MAX_DIAGNOSTIC_LOGS) {
    target.__ELEPHANT_DEBUG_LOGS__.splice(0, target.__ELEPHANT_DEBUG_LOGS__.length - MAX_DIAGNOSTIC_LOGS)
  }
  const logger = console[level] || console.log
  logger.call(console, entry.message, details)
  return entry
}

const localSourceToPath = (value = '') => {
  const text = stripQueryAndHash(value)
  if (text.startsWith(LOCAL_FILE_PREFIX)) return decodeSafe(text.slice(LOCAL_FILE_PREFIX.length))
  if (/^file:/i.test(text)) {
    try {
      return decodeSafe(new URL(text).pathname)
    } catch {
      return decodeSafe(text.replace(/^file:\/\//i, ''))
    }
  }
  return decodeSafe(text)
}

const isAbsoluteLocalPath = (pathname = '') => {
  const value = normalizeSlashes(pathname)
  return value.startsWith('/') || /^[a-zA-Z]:\//.test(value) || value.startsWith('//')
}

const activeVaultRoot = () => {
  try {
    return normalizeSlashes(globalThis.__ELEPHANT_GET_ACTIVE_VAULT_PATH__?.() || '')
  } catch (error) {
    pushExcalidrawImageLog('warn', 'active vault resolver failed', { error: errorDetails(error) })
    return ''
  }
}

const logResolvedPathOnce = (source, vaultRoot, vaultRelativePath, resolved) => {
  const key = `${source}\n${resolved}`
  if (loggedPathResolutions.has(key)) return
  if (loggedPathResolutions.size >= MAX_RESOLUTION_LOG_KEYS) loggedPathResolutions.clear()
  loggedPathResolutions.add(key)
  pushExcalidrawImageLog('info', 'resolved vault-relative drawing path', {
    source,
    vaultRoot,
    vaultRelativePath,
    resolved
  })
}

const resolveExcalidrawAssetPath = (source = '') => {
  const decoded = normalizeSlashes(localSourceToPath(source))
  if (!decoded || isAbsoluteLocalPath(decoded)) return decoded
  const marker = '.assets/'
  const markerIndex = decoded.indexOf(marker)
  const vaultRoot = activeVaultRoot()
  if (!vaultRoot || markerIndex < 0) return decoded
  const vaultRelativePath = decoded.slice(markerIndex)
  const resolved = globalThis.window?.path?.join
    ? globalThis.window.path.join(vaultRoot, vaultRelativePath)
    : `${vaultRoot.replace(/\/+$/, '')}/${vaultRelativePath}`
  logResolvedPathOnce(source, vaultRoot, vaultRelativePath, resolved)
  return normalizeSlashes(resolved)
}

const pathToDisplayUrl = (pathname) => {
  try {
    return convertFileSrc(pathname)
  } catch (error) {
    pushExcalidrawImageLog('warn', 'convertFileSrc failed; using local-file fallback', {
      pathname,
      error: errorDetails(error)
    })
    return `${LOCAL_FILE_PREFIX}${pathname}`
  }
}

const mimeFromPath = (pathname = '') => {
  const ext = String(pathname || '').split('.').pop()?.toLowerCase()
  switch (ext) {
    case 'jpg':
    case 'jpeg':
      return 'image/jpeg'
    case 'gif':
      return 'image/gif'
    case 'webp':
      return 'image/webp'
    case 'svg':
      return 'image/svg+xml'
    case 'avif':
      return 'image/avif'
    default:
      return 'image/png'
  }
}

const bytesToBase64 = (bytes) => {
  let binary = ''
  const chunkSize = 0x8000
  for (let index = 0; index < bytes.length; index += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(index, index + chunkSize))
  }
  return btoa(binary)
}

const dataByteLength = (data) => {
  if (data instanceof Blob) return data.size
  if (data instanceof ArrayBuffer) return data.byteLength
  if (ArrayBuffer.isView(data)) return data.byteLength
  if (typeof data === 'string') return new TextEncoder().encode(data).byteLength
  return 0
}

const fileExistsCacheState = (pathname) => {
  try {
    return typeof window.fileUtils?.pathExistsSync === 'function'
      ? Boolean(window.fileUtils.pathExistsSync(pathname))
      : null
  } catch (error) {
    pushExcalidrawImageLog('warn', 'pathExistsSync cache probe failed', {
      pathname,
      error: errorDetails(error)
    })
    return null
  }
}

const readLocalImageDataUrl = async (pathname = '') => {
  if (!pathname) throw new Error('empty-local-image-path')
  const readFile = window.fileUtils?.readFile
  if (typeof readFile !== 'function') throw new Error('fileUtils.readFile-unavailable')

  const cachedExists = fileExistsCacheState(pathname)
  pushExcalidrawImageLog('info', 'local image read:start', {
    pathname,
    cachedExists,
    cacheBypassed: cachedExists === false
  })

  try {
    // The Tauri bridge exposes pathExistsSync as a metadata-cache lookup. A cold
    // cache is not proof that a freshly-written preview is missing, so readFile
    // remains the authoritative operation.
    const data = await readFile.call(window.fileUtils, pathname)
    const byteLength = dataByteLength(data)
    let dataUrl = ''
    if (data instanceof Blob) {
      const bytes = new Uint8Array(await data.arrayBuffer())
      dataUrl = `data:${mimeFromPath(pathname)};base64,${bytesToBase64(bytes)}`
    } else if (data instanceof ArrayBuffer) {
      dataUrl = `data:${mimeFromPath(pathname)};base64,${bytesToBase64(new Uint8Array(data))}`
    } else if (ArrayBuffer.isView(data)) {
      const bytes = new Uint8Array(data.buffer, data.byteOffset, data.byteLength)
      dataUrl = `data:${mimeFromPath(pathname)};base64,${bytesToBase64(bytes)}`
    } else if (typeof data === 'string') {
      const bytes = new TextEncoder().encode(data)
      dataUrl = `data:${mimeFromPath(pathname)};base64,${bytesToBase64(bytes)}`
    }
    if (!dataUrl || byteLength <= 0) throw new Error('local-image-read-returned-empty-data')
    pushExcalidrawImageLog('info', 'local image read:success', {
      pathname,
      cachedExists,
      byteLength,
      mime: mimeFromPath(pathname)
    })
    return dataUrl
  } catch (error) {
    pushExcalidrawImageLog('error', 'local image read:failed', {
      pathname,
      cachedExists,
      error: errorDetails(error)
    })
    throw error
  }
}

const isTransientLocalImageSource = (img) => {
  const source = String(img?.getAttribute('src') || img?.currentSrc || '').trim()
  return !img?.complete && /^(?:blob:|data:)/i.test(source)
}

const imageSource = (img) => {
  const candidates = [
    img.getAttribute('data-src'),
    img.getAttribute('data-origin-src'),
    img.getAttribute('data-original-src'),
    img.getAttribute('src'),
    img.currentSrc
  ].filter(Boolean)
  return candidates.find((candidate) => EXCALIDRAW_ASSET_RE.test(normalizeSlashes(localSourceToPath(candidate)))) || ''
}

const removeLocalImageLoaders = (img) => {
  const root = img.closest('.ag-image, .ag-image-container, .image-wrapper, p, div') || img.parentElement
  if (!root) return
  const loaders = root.querySelectorAll('[class*="loading"], [class*="spinner"], .ag-image-loading, .image-loading')
  for (const loader of loaders) {
    if (loader === img || loader.contains(img)) continue
    loader.style.display = 'none'
    loader.style.opacity = '0'
    loader.style.pointerEvents = 'none'
    loader.setAttribute('aria-hidden', 'true')
  }
}

const refreshImageSource = (img, source) => {
  if (img?.dataset?.localImageLoaded === 'true') return
  const pathname = resolveExcalidrawAssetPath(source)
  if (!pathname || !EXCALIDRAW_ASSET_RE.test(normalizeSlashes(pathname))) return
  if (!isAbsoluteLocalPath(pathname)) {
    pushExcalidrawImageLog('warn', 'drawing source remains relative after resolution', {
      source,
      pathname,
      vaultRoot: activeVaultRoot()
    })
    return
  }
  if (isTransientLocalImageSource(img)) return
  const token = `${pathname}:${cacheBustSerial}`
  if (img.getAttribute(CACHE_BUST_ATTR) === token) return
  img.setAttribute(CACHE_BUST_ATTR, token)
  img.dataset.elephantExcalidrawPath = pathname
  const nextSrc = `${pathToDisplayUrl(pathname)}?v=${cacheBustSerial}`
  if (img.getAttribute('src') !== nextSrc) img.setAttribute('src', nextSrc)
}

const ensureEditButton = (img, source) => {
  const container = img.closest('.ag-image, .ag-image-container, .image-wrapper') || img.parentElement
  if (!container || container.getAttribute(INSTALLED_ATTR) === 'true') return
  container.setAttribute(INSTALLED_ATTR, 'true')
  container.classList.add('en-excalidraw-image-container')
  if (getComputedStyle(container).position === 'static') container.style.position = 'relative'
  img.classList.add('en-excalidraw-image')
  img.setAttribute('title', 'Edit Excalidraw drawing')

  const button = document.createElement('button')
  button.type = 'button'
  button.className = 'en-excalidraw-edit-button'
  button.textContent = '✎ Excalidraw'
  button.title = 'Edit Excalidraw drawing'
  button.addEventListener('pointerdown', (event) => {
    event.preventDefault()
    event.stopPropagation()
  }, true)
  button.addEventListener('click', (event) => {
    event.preventDefault()
    event.stopPropagation()
    const pathname = img.dataset.elephantExcalidrawPath || resolveExcalidrawAssetPath(source)
    pushExcalidrawImageLog('info', 'edit requested', { source, pathname })
    bus.emit('open-excalidraw-from-image', pathname)
  }, true)
  container.appendChild(button)
}

const repairFailedImageContainer = async (container) => {
  if (!container?.classList?.contains('ag-image-fail')) return false
  const source = container.dataset.imageSrc || container.dataset.imageDomsrc || ''
  const pathname = resolveExcalidrawAssetPath(source)
  if (!pathname || !EXCALIDRAW_ASSET_RE.test(normalizeSlashes(pathname))) return false
  if (!isAbsoluteLocalPath(pathname)) {
    pushExcalidrawImageLog('error', 'failed image repair cannot resolve an absolute path', {
      source,
      pathname,
      vaultRoot: activeVaultRoot()
    })
    return false
  }

  pushExcalidrawImageLog('info', 'failed image repair:start', {
    pathname,
    source,
    previousError: container.dataset.imageError || ''
  })

  try {
    const dataUrl = await readLocalImageDataUrl(pathname)
    const imageContainer = container.querySelector('.ag-image-container') || container
    const existingImage = imageContainer.querySelector?.('img')
    if (existingImage) existingImage.remove()
    const img = document.createElement('img')
    img.dataset.originalSrc = source
    img.dataset.localPath = pathname
    img.dataset.elephantExcalidrawPath = pathname
    img.dataset.localImageLoaded = 'true'
    img.dataset.resolvedSrc = dataUrl
    img.src = dataUrl
    imageContainer.appendChild(img)
    container.classList.remove('ag-image-fail', 'ag-image-loading')
    container.classList.add('ag-image-success')
    container.removeAttribute('title')
    delete container.dataset.imageError
    ensureEditButton(img, pathname)
    pushExcalidrawImageLog('info', 'failed image repair:success', { source, pathname })
    return true
  } catch (error) {
    container.dataset.imageError = error?.message || 'local-file-read-error'
    pushExcalidrawImageLog('error', 'failed image repair:failed', {
      pathname,
      source,
      error: errorDetails(error)
    })
    return false
  }
}

const repairImage = (img) => {
  const source = imageSource(img) || img.dataset.elephantExcalidrawPath || ''
  if (!source) return false
  const resolvedPath = resolveExcalidrawAssetPath(source)
  if (resolvedPath) img.dataset.elephantExcalidrawPath = resolvedPath
  removeLocalImageLoaders(img)
  refreshImageSource(img, resolvedPath || source)
  ensureEditButton(img, resolvedPath || source)
  return true
}

const reloadImageFromDisk = async (img, source, reason = 'cache-invalidation') => {
  const pathname = resolveExcalidrawAssetPath(source)
  if (!pathname || !isAbsoluteLocalPath(pathname) || !EXCALIDRAW_ASSET_RE.test(normalizeSlashes(pathname))) return false
  pushExcalidrawImageLog('info', 'cache refresh:start', { pathname, reason })
  try {
    const dataUrl = await readLocalImageDataUrl(pathname)
    img.dataset.originalSrc = img.dataset.originalSrc || source
    img.dataset.localPath = pathname
    img.dataset.elephantExcalidrawPath = pathname
    img.dataset.localImageLoaded = 'true'
    img.dataset.resolvedSrc = dataUrl
    img.setAttribute(CACHE_BUST_ATTR, `${pathname}:${cacheBustSerial}`)
    img.src = dataUrl
    pushExcalidrawImageLog('info', 'cache refresh:success', {
      pathname,
      reason,
      dataUrlLength: dataUrl.length
    })
    return true
  } catch (error) {
    pushExcalidrawImageLog('error', 'cache refresh:failed', {
      pathname,
      reason,
      error: errorDetails(error)
    })
    return false
  }
}

const repairAllImages = () => {
  const images = document.querySelectorAll('img')
  let repaired = 0
  for (const img of images) {
    if (repairImage(img)) repaired += 1
  }
  const failedContainers = document.querySelectorAll('.ag-image-fail[data-image-src], .ag-image-fail[data-image-domsrc]')
  for (const container of failedContainers) void repairFailedImageContainer(container)
  return repaired
}

const refreshAllDrawings = () => {
  cacheBustSerial = Date.now()
  const refreshTargets = []
  for (const img of document.querySelectorAll('img')) {
    const source = imageSource(img) || img.dataset.elephantExcalidrawPath || ''
    if (!source) continue
    repairImage(img)
    refreshTargets.push({ img, source })
  }
  const failedContainers = document.querySelectorAll('.ag-image-fail[data-image-src], .ag-image-fail[data-image-domsrc]')
  for (const container of failedContainers) void repairFailedImageContainer(container)
  for (const { img, source } of refreshTargets) void reloadImageFromDisk(img, source)
  pushExcalidrawImageLog('info', 'image cache invalidated', {
    cacheBustSerial,
    vaultRoot: activeVaultRoot(),
    refreshTargets: refreshTargets.length,
    failedContainers: failedContainers.length
  })
}

const removeInstalledUi = () => {
  for (const button of document.querySelectorAll('.en-excalidraw-edit-button')) button.remove()
  for (const container of document.querySelectorAll(`[${INSTALLED_ATTR}]`)) {
    container.removeAttribute(INSTALLED_ATTR)
    container.classList.remove('en-excalidraw-image-container')
  }
  for (const img of document.querySelectorAll('.en-excalidraw-image')) {
    img.classList.remove('en-excalidraw-image')
    img.removeAttribute('title')
    img.removeAttribute(CACHE_BUST_ATTR)
    delete img.dataset.elephantExcalidrawPath
  }
}

export const installExcalidrawImageRuntimeFixes = (target = globalThis) => {
  if (!target?.document) return { dispose() {} }
  const existing = target.__ELEPHANT_EXCALIDRAW_IMAGE_RUNTIME_FIXES__
  if (existing?.dispose) return existing

  let disposed = false
  const repairSoon = () => {
    if (disposed) return
    const schedule = typeof target.requestAnimationFrame === 'function'
      ? target.requestAnimationFrame.bind(target)
      : (typeof target.setTimeout === 'function' ? target.setTimeout.bind(target) : globalThis.setTimeout.bind(globalThis))
    schedule(() => {
      if (!disposed) repairAllImages()
    })
  }
  const observer = new MutationObserver((records) => {
    if (disposed) return
    let closedDialog = false
    for (const record of records) {
      for (const node of record.removedNodes || []) {
        if (node?.nodeType === 1 && (node.matches?.('.en-excalidraw-overlay') || node.querySelector?.('.en-excalidraw-overlay'))) closedDialog = true
      }
    }
    if (closedDialog) {
      const delay = typeof target.setTimeout === 'function' ? target.setTimeout.bind(target) : globalThis.setTimeout.bind(globalThis)
      delay(() => {
        if (!disposed) refreshAllDrawings()
      }, 80)
      return
    }
    repairSoon()
  })
  const handleLoad = (event) => {
    if (!disposed && event.target?.tagName === 'IMG') repairImage(event.target)
  }
  const handleError = (event) => {
    if (disposed || event.target?.tagName !== 'IMG') return
    const img = event.target
    const source = imageSource(img) || img.dataset.elephantExcalidrawPath || ''
    if (source) {
      pushExcalidrawImageLog('warn', 'browser image element emitted an error', {
        source,
        resolvedPath: resolveExcalidrawAssetPath(source),
        src: img.getAttribute('src') || '',
        dataSource: img.getAttribute('data-src') || ''
      })
      // A Tauri webview can reject an asset:// URL even when the file exists
      // and the Rust file bridge can read it. Re-applying the same URL only
      // loops the browser error; read the bytes and replace the source with a
      // data URL instead.
      void reloadImageFromDisk(img, source, 'browser-error')
    }
    repairImage(img)
  }

  observer.observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: ['src', 'data-src', 'class'] })
  document.addEventListener('load', handleLoad, true)
  document.addEventListener('error', handleError, true)
  bus.on('invalidate-image-cache', refreshAllDrawings)

  const runtime = {
    dispose() {
      disposed = true
      observer.disconnect()
      document.removeEventListener('load', handleLoad, true)
      document.removeEventListener('error', handleError, true)
      bus.off?.('invalidate-image-cache', refreshAllDrawings)
      removeInstalledUi()
      loggedPathResolutions.clear()
      if (target.__ELEPHANT_EXCALIDRAW_IMAGE_RUNTIME_FIXES__ === runtime) {
        delete target.__ELEPHANT_EXCALIDRAW_IMAGE_RUNTIME_FIXES__
      }
      pushExcalidrawImageLog('info', 'runtime fixes disposed')
    }
  }
  target.__ELEPHANT_EXCALIDRAW_IMAGE_RUNTIME_FIXES__ = runtime
  repairSoon()
  pushExcalidrawImageLog('info', 'runtime fixes installed', {
    hasReadFile: typeof target.fileUtils?.readFile === 'function',
    hasStat: typeof target.fileUtils?.stat === 'function',
    hasPathExistsSync: typeof target.fileUtils?.pathExistsSync === 'function',
    vaultRoot: activeVaultRoot()
  })
  return runtime
}
