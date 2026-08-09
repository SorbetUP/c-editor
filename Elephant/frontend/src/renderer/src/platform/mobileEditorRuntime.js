import bus from '@/bus'

const RUNTIME_FLAG = '__ELEPHANTNOTE_MOBILE_EDITOR_RUNTIME__'
const CAMERA_CONSTRAINTS = {
  audio: false,
  video: {
    facingMode: { ideal: 'environment' },
    width: { ideal: 1920 },
    height: { ideal: 1080 }
  }
}

const isMobile = (target = globalThis) => Boolean(
  target.matchMedia?.('(max-width: 760px), (hover: none) and (pointer: coarse)').matches
)

const icon = (name) => {
  const icons = {
    plus: '<path d="M12 5v14M5 12h14"/>',
    image: '<rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="m21 15-5-5L5 21"/>',
    camera: '<path d="M14.5 4 16 7h3a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V9a2 2 0 0 1 2-2h3l1.5-3z"/><circle cx="12" cy="13" r="3"/>',
    switchCamera: '<path d="M7 7h8l-2.5-2.5M17 17H9l2.5 2.5"/><path d="M17 7a7 7 0 0 1 2 5M7 17a7 7 0 0 1-2-5"/>',
    draw: '<path d="m12 19 7-7 3 3-7 7-4 1zM18 13l-1.5-1.5"/><path d="M2 20c2-4 5-5 9-4"/>',
    format: '<path d="M4 20h16M7 16 12 4l5 12M9 12h6"/>',
    paragraph: '<path d="M13 4H9a4 4 0 0 0 0 8h4M13 4v16M17 4v16"/>',
    h1: '<path d="M4 5v14M12 5v14M4 12h8M17 10l2-2v11M17 19h4"/>',
    h2: '<path d="M4 5v14M12 5v14M4 12h8M16 10a3 3 0 1 1 5 2l-5 7h6"/>',
    bold: '<path d="M6 4h7a4 4 0 0 1 0 8H6zM6 12h8a4 4 0 0 1 0 8H6z"/>',
    italic: '<path d="M10 4h8M6 20h8M14 4 10 20"/>',
    strike: '<path d="M5 12h14M16 6.5A5 5 0 0 0 12 5c-3 0-5 1.5-5 3.5 0 1.8 1.4 2.7 5 3.5M8 17.5A6 6 0 0 0 12 19c3 0 5-1.4 5-3.5"/>',
    code: '<path d="m8 9-4 3 4 3M16 9l4 3-4 3M14 5l-4 14"/>',
    link: '<path d="M10 13a5 5 0 0 0 7.1.1l2-2a5 5 0 0 0-7.1-7.1l-1.1 1.1"/><path d="M14 11a5 5 0 0 0-7.1-.1l-2 2A5 5 0 0 0 12 20l1.1-1.1"/>',
    undo: '<path d="M9 7 4 12l5 5"/><path d="M4 12h9a7 7 0 0 1 7 7"/>',
    redo: '<path d="m15 7 5 5-5 5"/><path d="M20 12h-9a7 7 0 0 0-7 7"/>',
    more: '<circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/>',
    close: '<path d="m6 6 12 12M18 6 6 18"/>',
    check: '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="m7 12 3 3 7-7"/>',
    list: '<path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/>',
    numbered: '<path d="M10 6h11M10 12h11M10 18h11M4 5h1v3M4 11h2l-2 3h2M4 17h2l-2 2h2"/>',
    quote: '<path d="M3 21c3 0 7-1 7-8V5H4v8h4M14 21c3 0 7-1 7-8V5h-6v8h4"/>',
    table: '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M3 15h18M9 3v18M15 3v18"/>',
    divider: '<path d="M4 12h16"/>',
    share: '<circle cx="18" cy="5" r="3"/><circle cx="6" cy="12" r="3"/><circle cx="18" cy="19" r="3"/><path d="m8.6 10.5 6.8-4M8.6 13.5l6.8 4"/>',
    duplicate: '<rect x="8" y="8" width="12" height="12" rx="2"/><path d="M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"/>',
    tag: '<path d="M20 12 12 20 4 12V4h8z"/><circle cx="9" cy="9" r="1"/>'
  }
  return `<svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${icons[name] || icons.more}</svg>`
}

const button = ({ label, iconName, action, compact = false }) => {
  const element = document.createElement('button')
  element.type = 'button'
  element.className = compact ? 'en-mobile-editor-button compact' : 'en-mobile-sheet-action'
  element.dataset.action = action
  element.setAttribute('aria-label', label)
  element.innerHTML = `${icon(iconName)}${compact ? '' : `<span>${label}</span>`}`
  return element
}

const showError = (target, error, fallback = 'The action could not be completed.') => {
  const message = error?.message || String(error || fallback)
  console.error('[mobile-editor]', message, error)
  target.alert?.(message || fallback)
}

const getVaultRoot = async (target) => {
  const invoke = target.__TAURI__?.core?.invoke
  if (typeof invoke !== 'function') throw new Error('The Android vault backend is unavailable.')
  const payload = await invoke('tauri_vaults_get')
  const root = String(payload?.activeVault?.path || '')
  if (!root) throw new Error('Select a vault before adding media.')
  return root
}

const extensionForType = (type = '') => {
  if (/png/i.test(type)) return 'png'
  if (/webp/i.test(type)) return 'webp'
  if (/gif/i.test(type)) return 'gif'
  return 'jpg'
}

const saveAsset = async (target, blob, preferredName = '') => {
  const vaultRoot = await getVaultRoot(target)
  const extension = extensionForType(blob?.type)
  const rawName = preferredName || `mobile-${Date.now()}.${extension}`
  const safeName = String(rawName).replace(/[^a-zA-Z0-9._-]+/g, '-').replace(/^-+|-+$/g, '') || `mobile-${Date.now()}.${extension}`
  const assetsDir = target.path.join(vaultRoot, '.assets')
  const destination = target.path.join(assetsDir, safeName)
  await target.fileUtils.ensureDir(assetsDir)
  await target.fileUtils.writeFile(destination, blob)
  bus.emit('insert-image', destination)
  target.dispatchEvent(new CustomEvent('elephantnote:vault-files-changed'))
  return destination
}

const createSheet = (title, actions) => {
  const backdrop = document.createElement('div')
  backdrop.className = 'en-mobile-editor-sheet-backdrop'
  backdrop.innerHTML = `
    <section class="en-mobile-editor-sheet" role="dialog" aria-modal="true" aria-label="${title}">
      <header><strong>${title}</strong><button type="button" aria-label="Close">${icon('close')}</button></header>
      <div class="en-mobile-editor-sheet-actions"></div>
    </section>
  `
  const actionHost = backdrop.querySelector('.en-mobile-editor-sheet-actions')
  for (const action of actions) actionHost.appendChild(button(action))
  const close = () => backdrop.remove()
  backdrop.addEventListener('click', (event) => {
    if (event.target === backdrop || event.target.closest('header button')) close()
  })
  return { backdrop, close }
}

const requestNativeCameraPermission = async (target) => {
  const scanner = target.__TAURI__?.barcodeScanner
  if (!scanner?.checkPermissions || !scanner?.requestPermissions) return
  const raw = await scanner.checkPermissions()
  const state = typeof raw === 'string' ? raw : (raw?.camera || raw?.permission || raw?.state)
  if (state === 'granted') return
  const requested = await scanner.requestPermissions()
  const next = typeof requested === 'string' ? requested : (requested?.camera || requested?.permission || requested?.state)
  if (next !== 'granted') throw new Error('Camera permission was refused.')
}

const requestCameraStream = async (target, facingMode) => {
  if (!target.navigator.mediaDevices?.getUserMedia) {
    throw new Error('Camera capture is not available on this Android WebView.')
  }
  return target.navigator.mediaDevices.getUserMedia({
    ...CAMERA_CONSTRAINTS,
    video: { ...CAMERA_CONSTRAINTS.video, facingMode: { ideal: facingMode } }
  })
}

const openCamera = async (target, onCaptured) => {
  let facingMode = 'environment'
  let stream

  // Request Android permission before mounting the camera UI. The permission
  // sheet now appears over the note instead of over a half-initialized camera.
  try {
    await requestNativeCameraPermission(target)
    stream = await requestCameraStream(target, facingMode)
  } catch (error) {
    if (error?.name === 'NotAllowedError') {
      throw new Error('Camera permission was refused. Allow Camera for Elephant in Android settings, then try again.')
    }
    throw error
  }

  const backdrop = document.createElement('div')
  backdrop.className = 'en-mobile-camera-backdrop'
  backdrop.innerHTML = `
    <section class="en-mobile-camera" role="dialog" aria-modal="true" aria-label="Camera">
      <video autoplay playsinline muted></video>
      <div class="en-mobile-camera-actions">
        <button type="button" class="icon" data-camera="cancel" aria-label="Close camera">${icon('close')}</button>
        <button type="button" class="capture" data-camera="capture" aria-label="Take photo"></button>
        <button type="button" class="icon" data-camera="switch" aria-label="Switch camera">${icon('switchCamera')}</button>
      </div>
      <p class="en-mobile-camera-error" aria-live="polite"></p>
    </section>
  `
  document.body.appendChild(backdrop)
  const video = backdrop.querySelector('video')
  const errorHost = backdrop.querySelector('.en-mobile-camera-error')

  const attachStream = async () => {
    video.srcObject = stream
    await video.play()
    errorHost.textContent = ''
  }

  let stopped = false
  const onAndroidBack = (event) => {
    event.preventDefault()
    stop()
  }
  const stop = () => {
    if (stopped) return
    stopped = true
    stream?.getTracks?.().forEach((track) => track.stop())
    if (video) video.srcObject = null
    target.removeEventListener('elephantnote:android-back', onAndroidBack)
    backdrop.remove()
  }
  target.addEventListener('elephantnote:android-back', onAndroidBack)

  const switchCamera = async () => {
    stream?.getTracks?.().forEach((track) => track.stop())
    facingMode = facingMode === 'environment' ? 'user' : 'environment'
    try {
      stream = await requestCameraStream(target, facingMode)
      await attachStream()
    } catch (error) {
      errorHost.textContent = `Unable to switch camera: ${error?.message || error}`
    }
  }

  backdrop.addEventListener('click', async (event) => {
    const action = event.target.closest('[data-camera]')?.dataset.camera
    if (action === 'cancel') stop()
    if (action === 'switch') await switchCamera()
    if (action === 'capture' && video.videoWidth > 0) {
      const canvas = document.createElement('canvas')
      canvas.width = video.videoWidth
      canvas.height = video.videoHeight
      canvas.getContext('2d').drawImage(video, 0, 0)
      const blob = await new Promise((resolve) => canvas.toBlob(resolve, 'image/jpeg', 0.9))
      if (blob) await onCaptured(blob)
      stop()
    }
  })

  await attachStream()
}

const chooseGalleryImage = (target, onPicked) => {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = 'image/*'
  input.hidden = true
  input.addEventListener('change', async () => {
    const file = input.files?.[0]
    try {
      if (file) await onPicked(file)
    } catch (error) {
      showError(target, error, 'The image could not be imported.')
    } finally {
      input.remove()
    }
  }, { once: true })
  document.body.appendChild(input)
  input.click()
}

const runCommand = (command) => {
  bus.emit('elephantnote-writing-command', command)
}

const insertActions = () => [
  { label: 'Take a photo', iconName: 'camera', action: 'camera' },
  { label: 'Add an image', iconName: 'image', action: 'gallery' },
  { label: 'Drawing', iconName: 'draw', action: 'excalidraw' },
  { label: 'Checklist', iconName: 'check', action: 'tasks' },
  { label: 'Bullet list', iconName: 'list', action: 'bullets' },
  { label: 'Numbered list', iconName: 'numbered', action: 'numbers' },
  { label: 'Quote', iconName: 'quote', action: 'quote' },
  { label: 'Table', iconName: 'table', action: 'table' },
  { label: 'Horizontal rule', iconName: 'divider', action: 'horizontal-rule' }
]

const formattingActions = () => [
  { label: 'Body text', iconName: 'paragraph', action: 'paragraph' },
  { label: 'Heading 1', iconName: 'h1', action: 'heading-1' },
  { label: 'Heading 2', iconName: 'h2', action: 'heading-2' },
  { label: 'Bold', iconName: 'bold', action: 'bold' },
  { label: 'Italic', iconName: 'italic', action: 'italic' },
  { label: 'Strikethrough', iconName: 'strike', action: 'strike' },
  { label: 'Inline code', iconName: 'code', action: 'code' },
  { label: 'Link', iconName: 'link', action: 'link' }
]

const moreActions = () => [
  { label: 'Share note', iconName: 'share', action: 'share-note' },
  { label: 'Duplicate note', iconName: 'duplicate', action: 'duplicate-note' },
  { label: 'Add or remove tags', iconName: 'tag', action: 'manage-tags' }
]

const shareCurrentNote = async (target) => {
  const title = target.document.querySelector('.en-note-title-input')?.value?.trim() || 'Note'
  const runtime = target.__ELEPHANT_ADDON_HOST__?.get?.('editor.runtime')
  const body = String(runtime?.getMarkdown?.() || '').trim()
  const text = body || title
  const invoke = target.__TAURI__?.core?.invoke
  if (typeof invoke === 'function') {
    await invoke('tauri_android_share_text', { title, text })
    return
  }
  if (typeof target.navigator.share === 'function') {
    await target.navigator.share({ title, text })
    return
  }
  throw new Error('System sharing is unavailable on this platform.')
}

const attachSheetActions = (target, sheet) => {
  sheet.backdrop.addEventListener('click', async (event) => {
    const action = event.target.closest('[data-action]')?.dataset.action
    if (!action) return
    sheet.close()
    try {
      if (action === 'camera') {
        await openCamera(target, (blob) => saveAsset(target, blob, `photo-${Date.now()}.jpg`))
        return
      }
      if (action === 'gallery') {
        chooseGalleryImage(target, (file) => saveAsset(target, file, file.name))
        return
      }
      if (action === 'share-note') {
        await shareCurrentNote(target)
        return
      }
      if (action === 'duplicate-note') {
        bus.emit('elephantnote:duplicate-note')
        return
      }
      if (action === 'manage-tags') {
        bus.emit('elephantnote:open-tags')
        return
      }
      runCommand(action)
    } catch (error) {
      showError(target, error)
    }
  })
}

const createToolbar = (target) => {
  const toolbar = document.createElement('nav')
  toolbar.className = 'en-mobile-editor-toolbar'
  toolbar.setAttribute('aria-label', 'Note editing tools')
  toolbar.append(
    button({ label: 'Insert', iconName: 'plus', action: 'insert', compact: true }),
    button({ label: 'Formatting', iconName: 'format', action: 'format', compact: true }),
    button({ label: 'Undo', iconName: 'undo', action: 'undo', compact: true }),
    button({ label: 'Redo', iconName: 'redo', action: 'redo', compact: true }),
    button({ label: 'More', iconName: 'more', action: 'more', compact: true })
  )

  toolbar.addEventListener('pointerdown', (event) => event.stopPropagation())
  toolbar.addEventListener('click', (event) => {
    const action = event.target.closest('[data-action]')?.dataset.action
    if (!action) return
    if (action === 'undo' || action === 'redo') {
      bus.emit(action)
      return
    }
    const actions = action === 'format'
      ? formattingActions()
      : action === 'more'
        ? moreActions()
        : insertActions()
    const title = action === 'format' ? 'Formatting' : action === 'more' ? 'Note actions' : 'Insert'
    const sheet = createSheet(title, actions)
    attachSheetActions(target, sheet)
    target.document.body.appendChild(sheet.backdrop)
  })
  return toolbar
}

const updateKeyboardOffset = (target) => {
  const viewport = target.visualViewport
  const offset = viewport
    ? Math.max(0, Math.round(target.innerHeight - viewport.height - viewport.offsetTop))
    : 0
  target.document.documentElement.style.setProperty('--en-mobile-keyboard-offset', `${offset}px`)
  target.document.documentElement.classList.toggle('en-mobile-keyboard-open', offset > 80)
}

const syncToolbar = (target) => {
  const noteOpen = isMobile(target) && !!target.document.querySelector('.en-main.has-editor-open .en-editor-layer')
  const existing = target.document.querySelector('.en-mobile-editor-toolbar')
  if (!noteOpen) {
    existing?.remove()
    return
  }
  if (!existing) target.document.querySelector('.en-editor-layer')?.appendChild(createToolbar(target))
  updateKeyboardOffset(target)
}

export const installMobileEditorRuntime = (target = globalThis) => {
  if (!target?.document || target[RUNTIME_FLAG]) return false
  target[RUNTIME_FLAG] = true
  let scheduled = false
  const schedule = () => {
    if (scheduled) return
    scheduled = true
    target.requestAnimationFrame(() => {
      scheduled = false
      syncToolbar(target)
    })
  }
  const observer = new target.MutationObserver(schedule)
  observer.observe(target.document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: ['class'] })
  const viewport = target.visualViewport
  target.addEventListener('resize', schedule)
  viewport?.addEventListener('resize', schedule)
  viewport?.addEventListener('scroll', schedule)
  schedule()
  target.__ELEPHANTNOTE_MOBILE_EDITOR_DISPOSE__ = () => {
    observer.disconnect()
    target.removeEventListener('resize', schedule)
    viewport?.removeEventListener('resize', schedule)
    viewport?.removeEventListener('scroll', schedule)
    target.document.querySelector('.en-mobile-editor-toolbar')?.remove()
    target.document.documentElement.style.removeProperty('--en-mobile-keyboard-offset')
    target.document.documentElement.classList.remove('en-mobile-keyboard-open')
    target[RUNTIME_FLAG] = false
  }
  return true
}

installMobileEditorRuntime()
