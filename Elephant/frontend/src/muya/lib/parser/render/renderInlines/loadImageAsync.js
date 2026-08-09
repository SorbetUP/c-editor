import { getUniqueId } from '../../../utils'
import { insertAfter, operateClassName } from '../../../utils/domManipulate'
import { CLASS_OR_ID } from '../../../config'
import { isStandardMarkdownImagePath } from 'common/elephantnote/imageAssetContract'

const isFileUrl = (value = '') => /^file:\/\//i.test(String(value || '').trim())
const isAbsoluteLocalPath = (value = '') => /^\//.test(value) || /^[a-zA-Z]:[\\/]/.test(value)
const MAX_DIAGNOSTIC_LOGS = 1000

const removeQueryAndHash = (value = '') => String(value || '').split(/[?#]/)[0]

const safeDecodeUri = (value = '') => {
  try {
    return decodeURI(value)
  } catch {
    return value
  }
}

const resolveLocalFilePath = (value = '') => {
  const raw = safeDecodeUri(removeQueryAndHash(String(value || '').trim()))
  if (!raw) return ''
  if (isFileUrl(raw)) {
    const withoutProtocol = raw.replace(/^file:\/\/\/?/i, '/')
    return withoutProtocol.replace(/^\/([a-zA-Z]:\/)/, '$1')
  }
  return isAbsoluteLocalPath(raw) ? raw : ''
}

const normalizeSlashes = (value = '') => String(value || '').replace(/\\/g, '/')
const isManagedImagePath = (value = '') => isStandardMarkdownImagePath(normalizeSlashes(value))
const errorDetails = (error) => ({
  name: error?.name || 'Error',
  message: error?.message || String(error || ''),
  stack: error?.stack || ''
})

const pushImageLog = (level, message, details = {}, force = false) => {
  const pathname = details?.localPath || details?.pathname || details?.src || ''
  if (!force && !isManagedImagePath(pathname)) return null
  const target = globalThis.window || globalThis
  const entry = {
    time: new Date().toISOString(),
    level,
    message: `[image-loader] ${message}`,
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

const appendCacheBuster = (value = '', dispMsec) => {
  const separator = value.includes('?') ? '&' : '?'
  return `${value}${separator}msec=${dispMsec}`
}

const createDomImageSrc = (src = '', dispMsec = Date.now()) => {
  const normalized = String(src || '').trim()
  if (!normalized) return ''
  if (isFileUrl(normalized)) return appendCacheBuster(normalized, dispMsec)
  if (isAbsoluteLocalPath(safeDecodeUri(normalized))) return appendCacheBuster(`file://${normalized.replace(/ /g, '%20')}`, dispMsec)
  return normalized.replace(/ /g, '%20')
}

const setImageFailureInfo = (imageText, { src = '', domsrc = '', reason = 'load-error' } = {}) => {
  if (!imageText) return
  imageText.dataset.imageSrc = src
  imageText.dataset.imageDomsrc = domsrc
  imageText.dataset.imageError = reason
  imageText.setAttribute('title', `Failed to load image\nSource: ${src || 'empty'}\nResolved: ${domsrc || 'empty'}\nReason: ${reason}`)
  const imageContainer = imageText.querySelector('.ag-image-container')
  if (imageContainer) {
    imageContainer.dataset.imageSrc = src
    imageContainer.dataset.imageDomsrc = domsrc
    imageContainer.dataset.imageError = reason
  }
}

const addImageToContainer = (imageText, img, className) => {
  if (imageText.classList.contains('ag-inline-image')) {
    const imageContainer = imageText.querySelector('.ag-image-container')
    const oldImage = imageContainer.querySelector('img')
    if (oldImage) oldImage.remove()
    imageContainer.appendChild(img)
    imageText.classList.remove('ag-image-loading')
    imageText.classList.remove('ag-image-fail')
    imageText.classList.add('ag-image-success')
    imageText.removeAttribute('title')
    delete imageText.dataset.imageError
  } else {
    insertAfter(img, imageText)
    operateClassName(imageText, 'add', className)
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

const cachedExists = (pathname) => {
  try {
    return typeof window.fileUtils?.pathExistsSync === 'function'
      ? Boolean(window.fileUtils.pathExistsSync(pathname))
      : null
  } catch (error) {
    pushImageLog('warn', 'pathExistsSync cache probe failed', {
      localPath: pathname,
      error: errorDetails(error)
    })
    return null
  }
}

const readLocalImageDataUrl = async (pathname = '') => {
  if (!pathname) throw new Error('empty-local-image-path')
  const readFile = window.fileUtils?.readFile
  if (typeof readFile !== 'function') throw new Error('fileUtils.readFile-unavailable')

  const cacheState = cachedExists(pathname)
  pushImageLog('info', 'local read:start', {
    localPath: pathname,
    cachedExists: cacheState,
    cacheBypassed: cacheState === false
  })

  try {
    // pathExistsSync is only a metadata-cache lookup in the Tauri bridge. A false
    // value must never prevent a real read of a file that was just written.
    const data = await readFile.call(window.fileUtils, pathname)
    const byteLength = dataByteLength(data)
    let dataUrl = ''
    if (data instanceof Blob) {
      const buffer = new Uint8Array(await data.arrayBuffer())
      dataUrl = `data:${mimeFromPath(pathname)};base64,${bytesToBase64(buffer)}`
    } else if (data instanceof ArrayBuffer) {
      dataUrl = `data:${mimeFromPath(pathname)};base64,${bytesToBase64(new Uint8Array(data))}`
    } else if (ArrayBuffer.isView(data)) {
      dataUrl = `data:${mimeFromPath(pathname)};base64,${bytesToBase64(new Uint8Array(data.buffer, data.byteOffset, data.byteLength))}`
    } else if (typeof data === 'string') {
      const bytes = new TextEncoder().encode(data)
      dataUrl = `data:${mimeFromPath(pathname)};base64,${bytesToBase64(bytes)}`
    }
    if (!dataUrl || byteLength <= 0) throw new Error('local-image-read-returned-empty-data')
    pushImageLog('info', 'local read:success', {
      localPath: pathname,
      cachedExists: cacheState,
      byteLength,
      mime: mimeFromPath(pathname)
    })
    return dataUrl
  } catch (error) {
    pushImageLog('error', 'local read:failed', {
      localPath: pathname,
      cachedExists: cacheState,
      error: errorDetails(error)
    })
    throw error
  }
}

export default function loadImageAsync(imageInfo, attrs, className, imageClass) {
  let { src } = imageInfo
  let id
  let isSuccess
  let w
  let h
  let domsrc

  src = String(src || '').replace(/ /g, '%20')
  const localPath = resolveLocalFilePath(src)

  let reload = false
  if (this.loadImageMap.has(src)) {
    const cachedImageInfo = this.loadImageMap.get(src)
    if (cachedImageInfo.dispMsec !== cachedImageInfo.touchMsec) reload = true
  } else {
    reload = true
  }
  if (reload) {
    let addedToImageContainer = false
    id = getUniqueId()

    const img = document.createElement('img')
    const dispMsec = Date.now()
    const touchMsec = dispMsec
    domsrc = createDomImageSrc(src, dispMsec)
    img.dataset.originalSrc = src
    img.dataset.localPath = localPath
    img.dataset.localResolvedPath = localPath
    img.dataset.resolvedSrc = domsrc
    if (attrs.alt) img.alt = attrs.alt.replace(/[`*{}[\]()#+\-.!_>~:|<>$]/g, '')
    if (attrs.title) img.setAttribute('title', attrs.title)
    if (attrs.width && typeof attrs.width === 'number') img.setAttribute('width', attrs.width)
    if (attrs.height && typeof attrs.height === 'number') img.setAttribute('height', attrs.height)
    if (imageClass) img.classList.add(imageClass)

    if (this.urlMap.has(src)) this.urlMap.delete(src)

    const imageText = document.querySelector(`#${id}`)
    if (imageText) {
      addImageToContainer(imageText, img, className)
      addedToImageContainer = true
    }

    const fail = (reason = 'image-load-error', error = null) => {
      const imageText = document.querySelector(`#${id}`)
      if (imageText) {
        operateClassName(imageText, 'remove', CLASS_OR_ID.AG_IMAGE_LOADING)
        operateClassName(imageText, 'add', CLASS_OR_ID.AG_IMAGE_FAIL)
        setImageFailureInfo(imageText, { src, domsrc, reason })
        const image = imageText.querySelector('img')
        if (image) image.remove()
      }
      pushImageLog('error', 'image load:failed', {
        src,
        domsrc,
        localPath,
        reason,
        error: error ? errorDetails(error) : null
      }, !localPath)
      if (this.urlMap.has(src)) this.urlMap.delete(src)
      this.loadImageMap.set(src, { id, isSuccess: false, domsrc, localPath, error: reason, addedToImageContainer: false })
    }

    img.onload = () => {
      const imageText = document.querySelector(`#${id}`)
      if (imageText && !addedToImageContainer) {
        addImageToContainer(imageText, img, className)
        addedToImageContainer = true
      }
      this.loadImageMap.set(src, {
        id,
        isSuccess: true,
        img,
        width: img.naturalWidth,
        height: img.naturalHeight,
        dispMsec,
        touchMsec,
        domsrc,
        localPath,
        addedToImageContainer
      })
      pushImageLog('info', 'image element load:success', {
        src,
        localPath,
        width: img.naturalWidth,
        height: img.naturalHeight,
        addedToImageContainer
      })
    }
    img.onerror = (event) => fail(
      localPath ? 'local-data-url-load-error' : 'image-load-error',
      event?.error || null
    )

    if (localPath) {
      readLocalImageDataUrl(localPath)
        .then((dataUrl) => {
          domsrc = dataUrl
          img.dataset.localImageLoaded = 'true'
          img.dataset.resolvedSrc = dataUrl
          img.src = dataUrl
        })
        .catch((error) => fail(error?.message || 'local-file-read-error', error))
      img.dataset.resolvedSrc = localPath
    } else {
      img.src = domsrc
    }
  } else {
    const cachedImageInfo = this.loadImageMap.get(src)
    id = cachedImageInfo.id
    isSuccess = cachedImageInfo.isSuccess
    w = cachedImageInfo.width
    h = cachedImageInfo.height
    domsrc = cachedImageInfo.domsrc
    if (!cachedImageInfo.addedToImageContainer && cachedImageInfo.img) {
      const imageText = document.querySelector(`#${id}`)
      if (imageText) {
        addImageToContainer(imageText, cachedImageInfo.img, className)
        this.loadImageMap.set(src, { ...cachedImageInfo, addedToImageContainer: true })
      }
    }
  }

  return { id, isSuccess, domsrc, width: w, height: h }
}
