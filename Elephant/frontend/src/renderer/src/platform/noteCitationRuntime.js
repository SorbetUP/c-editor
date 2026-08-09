import { useVaultStore } from 'elephant-front/stores/vaultStore'
import { useEditorStore } from '@/store/editor'
import bus from '@/bus'

const EXTERNAL_PROTOCOL_RE = /^(?:https?:|mailto:|tel:|data:|javascript:|file:)/i
const MARKDOWN_PATH_RE = /\.md$/i
const MAX_QUOTE_ANCHOR_CHARS = 640
const RUNTIME_KEY = '__ELEPHANT_NOTE_CITATION_RUNTIME__'
const BUFFER_KEY = '__ELEPHANT_NOTE_CITATION_BUFFER__'

const decodeComponent = (value = '') => {
  try {
    return decodeURIComponent(value)
  } catch {
    return value
  }
}

export const normalizeVaultPath = (value = '') => {
  const parts = []
  for (const part of String(value || '').replace(/\\/g, '/').split('/')) {
    if (!part || part === '.') continue
    if (part === '..') {
      if (!parts.length) return ''
      parts.pop()
      continue
    }
    parts.push(part)
  }
  return parts.join('/')
}

const parentPath = (value = '') => {
  const parts = normalizeVaultPath(value).split('/').filter(Boolean)
  parts.pop()
  return parts.join('/')
}

export const normalizeCitationText = (value = '') => String(value || '')
  .replace(/\r\n?/g, '\n')
  .split('\n')
  .map((line) => line.replace(/[\t ]+$/g, ''))
  .join('\n')
  .trim()

const bytesToBase64 = (bytes) => {
  let binary = ''
  for (const byte of bytes) binary += String.fromCharCode(byte)
  return btoa(binary)
}

const base64ToBytes = (value) => {
  const binary = atob(value)
  return Uint8Array.from(binary, (character) => character.charCodeAt(0))
}

export const encodeQuoteAnchor = (value = '') => {
  const text = normalizeCitationText(value).slice(0, MAX_QUOTE_ANCHOR_CHARS)
  if (!text) return ''
  return bytesToBase64(new TextEncoder().encode(text))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/g, '')
}

export const decodeQuoteAnchor = (value = '') => {
  const encoded = String(value || '').replace(/-/g, '+').replace(/_/g, '/')
  if (!encoded) return ''
  try {
    const padded = encoded.padEnd(Math.ceil(encoded.length / 4) * 4, '=')
    return new TextDecoder().decode(base64ToBytes(padded))
  } catch {
    return ''
  }
}

const encodeVaultPath = (value = '') => normalizeVaultPath(value)
  .split('/')
  .filter(Boolean)
  .map((part) => encodeURIComponent(part))
  .join('/')

const escapeMarkdownLabel = (value = '') => String(value || '')
  .replace(/\\/g, '\\\\')
  .replace(/\]/g, '\\]')

export const buildNoteCitationMarkdown = ({
  text = '',
  notePath = '',
  noteTitle = ''
} = {}) => {
  const normalizedText = normalizeCitationText(text)
  const normalizedPath = normalizeVaultPath(notePath)
  if (!normalizedText || !normalizedPath || !MARKDOWN_PATH_RE.test(normalizedPath)) return ''

  const quotedText = normalizedText
    .split('\n')
    .map((line) => line ? `> ${line}` : '>')
    .join('\n')
  const title = escapeMarkdownLabel(
    String(noteTitle || '').trim() || normalizedPath.split('/').pop()?.replace(/\.md$/i, '') || 'Source'
  )
  const quoteAnchor = encodeQuoteAnchor(normalizedText)
  const destination = `</${encodeVaultPath(normalizedPath)}#quote=${quoteAnchor}>`
  return `${quotedText}\n>\n> — [${title}](${destination})`
}

export const resolveInternalNoteLink = ({
  href = '',
  currentNotePath = '',
  appOrigin = globalThis.location?.origin || ''
} = {}) => {
  const raw = String(href || '').trim()
  const current = normalizeVaultPath(currentNotePath)
  if (!raw || !current) return null

  if (raw.startsWith('#')) {
    const anchor = decodeComponent(raw.slice(1))
    return {
      path: current,
      anchor,
      quote: anchor.startsWith('quote=') ? decodeQuoteAnchor(anchor.slice(6)) : ''
    }
  }

  let pathPart = raw
  let anchor = ''
  let vaultRooted = false

  if (/^[a-z][a-z0-9+.-]*:/i.test(raw)) {
    if (EXTERNAL_PROTOCOL_RE.test(raw)) {
      let parsed
      try {
        parsed = new URL(raw)
      } catch {
        return null
      }
      if (!appOrigin || parsed.origin !== appOrigin) return null
      pathPart = parsed.pathname.replace(/^\/+/, '')
      anchor = decodeComponent(parsed.hash.replace(/^#/, ''))
      vaultRooted = true
    } else {
      return null
    }
  } else {
    const hashIndex = raw.indexOf('#')
    if (hashIndex >= 0) {
      anchor = decodeComponent(raw.slice(hashIndex + 1))
      pathPart = raw.slice(0, hashIndex)
    }
    pathPart = pathPart.split('?')[0]
    vaultRooted = pathPart.startsWith('/')
  }

  pathPart = decodeComponent(pathPart).replace(/\\/g, '/')
  if (!MARKDOWN_PATH_RE.test(pathPart)) return null

  const path = vaultRooted
    ? normalizeVaultPath(pathPart.replace(/^\/+/, ''))
    : normalizeVaultPath(`${parentPath(current)}/${pathPart}`)
  if (!path || !MARKDOWN_PATH_RE.test(path)) return null

  return {
    path,
    anchor,
    quote: anchor.startsWith('quote=') ? decodeQuoteAnchor(anchor.slice(6)) : ''
  }
}

const comparableText = (value = '') => String(value || '')
  .normalize('NFKC')
  .replace(/\s+/g, ' ')
  .trim()
  .toLocaleLowerCase()

const markdownAnchorSlug = (value = '') => String(value || '')
  .trim()
  .toLowerCase()
  .normalize('NFKD')
  .replace(/[\u0300-\u036f]/g, '')
  .replace(/[^\p{L}\p{N}]+/gu, '-')
  .replace(/^-+|-+$/g, '')

const flashTarget = (target, windowObject) => {
  if (!target) return
  const previousOutline = target.style.outline
  const previousOutlineOffset = target.style.outlineOffset
  const previousBackground = target.style.background
  target.style.outline = '2px solid var(--en-accent, #7c83ff)'
  target.style.outlineOffset = '4px'
  target.style.background = 'color-mix(in srgb, var(--en-accent, #7c83ff) 12%, transparent)'
  windowObject.setTimeout(() => {
    target.style.outline = previousOutline
    target.style.outlineOffset = previousOutlineOffset
    target.style.background = previousBackground
  }, 2200)
}

const scrollToReference = ({ quote = '', anchor = '' }, windowObject, attempt = 0) => {
  if (attempt > 30) return false
  const editorHost = windowObject.document.querySelector('.en-editor-host')
  if (!editorHost) {
    windowObject.setTimeout(() => scrollToReference({ quote, anchor }, windowObject, attempt + 1), 70)
    return false
  }

  let target = null
  const normalizedQuote = comparableText(quote)
  if (normalizedQuote) {
    const candidates = [
      ...editorHost.querySelectorAll(
        '[data-block-id], [data-block-type], p, blockquote, li, h1, h2, h3, h4, h5, h6, pre'
      )
    ]
      .filter((element) => comparableText(element.textContent).includes(normalizedQuote))
      .sort((left, right) => left.textContent.length - right.textContent.length)
    target = candidates[0] || null
  }

  if (!target && anchor && !anchor.startsWith('quote=')) {
    const decoded = decodeComponent(anchor)
    const escaped = globalThis.CSS?.escape
      ? globalThis.CSS.escape(decoded)
      : decoded.replace(/["\\]/g, '\\$&')
    target = editorHost.querySelector(`#${escaped}, [name="${escaped}"], [data-id="${escaped}"]`)
    if (!target) {
      const slug = markdownAnchorSlug(decoded)
      target = [...editorHost.querySelectorAll('h1, h2, h3, h4, h5, h6, [data-block-type="heading"]')]
        .find((element) => markdownAnchorSlug(element.textContent) === slug) || null
    }
  }

  if (!target) {
    windowObject.setTimeout(() => scrollToReference({ quote, anchor }, windowObject, attempt + 1), 70)
    return false
  }

  target.scrollIntoView?.({ behavior: 'smooth', block: 'center' })
  flashTarget(target, windowObject)
  return true
}

const appendDebugLog = (windowObject, level, message, details = {}) => {
  const entry = { at: new Date().toISOString(), level, message, details }
  windowObject.__ELEPHANT_DEBUG_LOGS__ = Array.isArray(windowObject.__ELEPHANT_DEBUG_LOGS__)
    ? windowObject.__ELEPHANT_DEBUG_LOGS__
    : []
  windowObject.__ELEPHANT_DEBUG_LOGS__.push(entry)
  if (Array.isArray(windowObject.__ELEPHANT_ACCEPTANCE_LOGS__)) windowObject.__ELEPHANT_ACCEPTANCE_LOGS__.push(entry)
  if (windowObject.__ELEPHANT_DEBUG_LOGS__.length > 1000) {
    windowObject.__ELEPHANT_DEBUG_LOGS__.splice(0, windowObject.__ELEPHANT_DEBUG_LOGS__.length - 1000)
  }
  const logger = windowObject.console?.[level] || windowObject.console?.log
  logger?.call(windowObject.console, message, details)
}

const writeClipboardText = async (text, windowObject) => {
  if (windowObject.navigator?.clipboard?.writeText) {
    await windowObject.navigator.clipboard.writeText(text)
    return
  }
  const textarea = windowObject.document.createElement('textarea')
  textarea.value = text
  textarea.setAttribute('readonly', '')
  textarea.style.position = 'fixed'
  textarea.style.opacity = '0'
  windowObject.document.body.appendChild(textarea)
  textarea.select()
  const copied = windowObject.document.execCommand?.('copy')
  textarea.remove()
  if (!copied) throw new Error('Clipboard API is unavailable.')
}

const createFeedback = (message, windowObject, isError = false) => {
  windowObject.document.querySelector('[data-elephant-citation-feedback]')?.remove()
  const feedback = windowObject.document.createElement('div')
  feedback.dataset.elephantCitationFeedback = 'true'
  feedback.setAttribute('role', isError ? 'alert' : 'status')
  feedback.textContent = message
  feedback.style.cssText = [
    'position:fixed',
    'right:20px',
    'bottom:20px',
    'z-index:10000',
    'max-width:min(420px,calc(100vw - 40px))',
    'padding:10px 14px',
    'border:1px solid var(--en-border,#3b4352)',
    'border-radius:10px',
    'background:var(--en-surface,#171d27)',
    'color:var(--en-text,#eef3fb)',
    'box-shadow:0 18px 44px rgba(0,0,0,.28)',
    'font:500 13px/1.4 system-ui,-apple-system,sans-serif'
  ].join(';')
  if (isError) feedback.style.borderColor = '#ef4444'
  windowObject.document.body.appendChild(feedback)
  windowObject.setTimeout(() => feedback.remove(), 2200)
}

const appendCitationToCurrentNote = (editorStore, markdown) => {
  if (!editorStore?.currentFile?.id) throw new Error('Ouvrez une note avant de coller une citation.')
  const current = String(editorStore.currentFile.markdown || '')
  const separator = current && !current.endsWith('\n') ? '\n\n' : ''
  const next = `${current}${separator}${markdown}\n`
  editorStore.currentFile.markdown = next
  editorStore.currentFile.isSaved = false
  bus.emit('file-changed', {
    id: editorStore.currentFile.id,
    markdown: next,
    cursor: editorStore.currentFile.cursor,
    renderCursor: true,
    history: editorStore.currentFile.history,
    scrollTop: editorStore.currentFile.scrollTop
  })
  return next
}

const selectionBelongsToEditor = (selection, editorHost) => {
  if (!selection || selection.rangeCount === 0 || selection.isCollapsed || !editorHost) return false
  const anchorElement = selection.anchorNode?.nodeType === 1
    ? selection.anchorNode
    : selection.anchorNode?.parentElement
  const focusElement = selection.focusNode?.nodeType === 1
    ? selection.focusNode
    : selection.focusNode?.parentElement
  return !!anchorElement && !!focusElement && editorHost.contains(anchorElement) && editorHost.contains(focusElement)
}

const cloneSelectionRange = (selection) => {
  if (!selection?.rangeCount) return null
  return selection.getRangeAt(0).cloneRange()
}

const restoreSelectionRange = (target, range) => {
  if (!range) return false
  const selection = target.getSelection?.()
  if (!selection) return false
  selection.removeAllRanges()
  selection.addRange(range.cloneRange())
  return true
}

const getSelectionText = (selection) => {
  if (!selection) return ''
  const text = selection.toString?.() || ''
  if (text) return text
  return selection.rangeCount ? selection.getRangeAt(0).toString() : ''
}

const createCitationButton = (copyCitation, windowObject) => {
  const button = windowObject.document.createElement('button')
  button.type = 'button'
  button.dataset.elephantNoteCitation = 'true'
  button.className = 'en-note-action-button en-note-citation-button'
  button.title = 'Citer le texte sélectionné'
  button.setAttribute('aria-label', 'Citer le texte sélectionné')
  button.style.cssText = [
    'width:30px',
    'height:30px',
    'display:inline-flex',
    'align-items:center',
    'justify-content:center',
    'padding:0',
    'border:1px solid var(--en-border)',
    'border-radius:8px',
    'background:transparent',
    'color:var(--en-text)',
    'cursor:pointer',
    '-webkit-app-region:no-drag'
  ].join(';')
  button.innerHTML = '<svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 21c3 0 7-1 7-8V5c0-1.1-.9-2-2-2H5c-1.1 0-2 .9-2 2v6c0 1.1.9 2 2 2h4"/><path d="M14 21c3 0 7-1 7-8V5c0-1.1-.9-2-2-2h-3c-1.1 0-2 .9-2 2v6c0 1.1.9 2 2 2h4"/></svg>'
  button.addEventListener('mouseenter', () => {
    button.style.background = 'var(--en-soft)'
  })
  button.addEventListener('mouseleave', () => {
    button.style.background = 'transparent'
  })
  button.addEventListener('click', copyCitation)
  return button
}

export const installNoteCitationRuntime = ({
  pinia,
  vaultStore = null,
  editorStore: providedEditorStore = null,
  target = globalThis.window
} = {}) => {
  if (!target?.document) return { dispose() {} }
  target[RUNTIME_KEY]?.dispose?.()

  const store = vaultStore || useVaultStore(pinia)
  const editorStore = providedEditorStore || (pinia ? useEditorStore(pinia) : null)
  let citationButton = null
  let selectionButton = null
  let lastEditorSelectionRange = null
  let palette = null
  let contextMenu = null
  const buffer = Array.isArray(target[BUFFER_KEY]) ? target[BUFFER_KEY] : []
  target[BUFFER_KEY] = buffer

  const removeContextMenu = () => {
    contextMenu?.remove()
    contextMenu = null
  }

  const deleteCitation = (item) => {
    const index = buffer.findIndex((entry) => entry.id === item.id)
    if (index >= 0) buffer.splice(index, 1)
    removeContextMenu()
    renderPalette()
    appendDebugLog(target, 'info', '[elephantnote:citation] buffer item deleted', { id: item.id })
  }

  const pasteCitation = (item) => {
    try {
      appendCitationToCurrentNote(editorStore, item.markdown)
      createFeedback('Citation collée dans la note.', target)
      appendDebugLog(target, 'info', '[elephantnote:citation] buffer item pasted', {
        id: item.id,
        targetPath: store.openedNotePath || ''
      })
      return true
    } catch (error) {
      createFeedback(error?.message || String(error), target, true)
      appendDebugLog(target, 'error', '[elephantnote:citation] paste failed', {
        id: item.id,
        error: error?.message || String(error)
      })
      return false
    }
  }

  const showCitationInfo = (item, x, y) => {
    removeContextMenu()
    contextMenu = target.document.createElement('div')
    contextMenu.dataset.elephantCitationContext = 'true'
    contextMenu.setAttribute('role', 'menu')
    contextMenu.style.cssText = 'position:fixed;z-index:10001;display:grid;gap:6px;min-width:240px;padding:10px;border:1px solid var(--en-border);border-radius:10px;background:var(--en-surface);color:var(--en-text);box-shadow:0 18px 44px rgba(0,0,0,.28);'
    const info = target.document.createElement('div')
    info.textContent = `${item.title} · ${item.path}`
    info.style.cssText = 'padding:4px 6px;font-size:12px;white-space:normal;'
    const remove = target.document.createElement('button')
    remove.type = 'button'
    remove.textContent = 'Supprimer la citation'
    remove.setAttribute('aria-label', `Supprimer la citation ${item.title}`)
    remove.onclick = () => deleteCitation(item)
    contextMenu.append(info, remove)
    target.document.body.appendChild(contextMenu)
    contextMenu.style.left = `${Math.max(8, Math.min(Number(x) || 20, target.innerWidth - 270))}px`
    contextMenu.style.top = `${Math.max(8, Math.min(Number(y) || 20, target.innerHeight - 130))}px`
  }

  const renderPalette = () => {
    if (!target.document.body) return
    palette?.remove()
    palette = target.document.createElement('div')
    palette.dataset.elephantCitationPalette = 'true'
    palette.setAttribute('aria-label', 'Citations en attente')
    palette.style.cssText = 'position:fixed;right:20px;bottom:20px;z-index:10000;display:grid;gap:6px;max-width:min(360px,calc(100vw - 40px));'
    for (const item of buffer.slice().reverse()) {
      const button = target.document.createElement('button')
      button.type = 'button'
      button.dataset.elephantCitationBufferItem = item.id
      button.className = 'en-note-citation-buffer-item'
      button.title = 'Cliquer pour coller · clic droit pour les informations'
      button.setAttribute('aria-label', `Coller la citation ${item.title}`)
      button.textContent = `↳ ${item.title}: ${item.text.slice(0, 72)}${item.text.length > 72 ? '…' : ''}`
      button.style.cssText = 'padding:9px 12px;border:1px solid var(--en-border);border-radius:10px;background:var(--en-surface);color:var(--en-text);box-shadow:0 12px 30px rgba(0,0,0,.22);text-align:left;cursor:pointer;'
      button.onclick = () => pasteCitation(item)
      button.oncontextmenu = (event) => {
        event.preventDefault()
        showCitationInfo(item, event.clientX, event.clientY)
      }
      palette.appendChild(button)
    }
    if (buffer.length) target.document.body.appendChild(palette)
  }

  const copyCitation = async () => {
    const editorHost = target.document.querySelector('.en-editor-host')
    let selection = target.getSelection?.()
    if (!selectionBelongsToEditor(selection, editorHost) || !getSelectionText(selection)) {
      restoreSelectionRange(target, lastEditorSelectionRange)
      selection = target.getSelection?.()
    }
    if (!selectionBelongsToEditor(selection, editorHost)) {
      createFeedback('Sélectionnez d’abord le texte à citer dans la note.', target, true)
      appendDebugLog(target, 'warn', '[elephantnote:citation] no editor selection to cite', {
        notePath: store.openedNotePath || ''
      })
      return
    }

    const notePath = normalizeVaultPath(store.openedNotePath || '')
    const noteTitle = target.document.querySelector('.en-note-title-input')?.value ||
      notePath.split('/').pop()?.replace(/\.md$/i, '') ||
      'Source'
    const citation = buildNoteCitationMarkdown({
      text: getSelectionText(selection),
      notePath,
      noteTitle
    })
    if (!citation) {
      createFeedback('Impossible de créer la citation pour cette note.', target, true)
      return
    }

    try {
      const item = {
        id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        path: notePath,
        title: noteTitle,
        text: normalizeCitationText(getSelectionText(selection)),
        markdown: citation,
        createdAt: new Date().toISOString()
      }
      buffer.push(item)
      while (buffer.length > 8) buffer.shift()
      await writeClipboardText(citation, target)
      selectionButton?.remove()
      selectionButton = null
      renderPalette()
      createFeedback('Citation copiée et ajoutée au tampon. Cliquez-la dans une autre note pour la coller.', target)
      appendDebugLog(target, 'info', '[elephantnote:citation] copied selected note text', {
        notePath,
        selectedLength: normalizeCitationText(getSelectionText(selection)).length,
        citationLength: citation.length,
        bufferSize: buffer.length
      })
    } catch (error) {
      createFeedback('La citation n’a pas pu être copiée.', target, true)
      appendDebugLog(target, 'error', '[elephantnote:citation] clipboard write failed', {
        notePath,
        error: error?.message || String(error)
      })
    }
  }

  const updateSelectionButton = () => {
    const editorHost = target.document.querySelector('.en-editor-host')
    const selection = target.getSelection?.()
    if (!selectionBelongsToEditor(selection, editorHost)) {
      selectionButton?.remove()
      selectionButton = null
      return
    }
    lastEditorSelectionRange = cloneSelectionRange(selection)
    if (selectionButton) return
    selectionButton = target.document.createElement('button')
    selectionButton.type = 'button'
    selectionButton.dataset.elephantCitationSelectionAction = 'true'
    selectionButton.className = 'en-note-citation-selection-action'
    selectionButton.setAttribute('aria-label', 'Ajouter la sélection aux citations')
    selectionButton.textContent = 'Citer'
    selectionButton.style.cssText = 'position:fixed;right:20px;bottom:20px;z-index:10000;padding:9px 13px;border:1px solid var(--en-primary);border-radius:10px;background:var(--en-primary);color:#fff;cursor:pointer;box-shadow:0 12px 30px rgba(0,0,0,.25);'
    selectionButton.onclick = () => void copyCitation()
    target.document.body.appendChild(selectionButton)
  }

  const ensureCitationButton = () => {
    const actionRail = target.document.querySelector('.en-note-topbar-actions')
    if (!actionRail) return
    const existing = actionRail.querySelector('[data-elephant-note-citation]')
    if (existing) {
      citationButton = existing
      return
    }
    citationButton = createCitationButton(copyCitation, target)
    actionRail.prepend(citationButton)
  }

  const handleInternalLinkClick = (event) => {
    if (
      event.defaultPrevented ||
      event.button > 0 ||
      event.metaKey ||
      event.ctrlKey ||
      event.altKey ||
      event.shiftKey
    ) return
    const anchorElement = event.target?.closest?.('.en-editor-host a[href]')
    if (!anchorElement) return
    const resolved = resolveInternalNoteLink({
      href: anchorElement.getAttribute('href') || anchorElement.href,
      currentNotePath: store.openedNotePath || '',
      appOrigin: target.location?.origin || ''
    })
    if (!resolved) return

    event.preventDefault()
    event.stopPropagation()
    event.stopImmediatePropagation?.()
    appendDebugLog(target, 'info', '[elephantnote:citation] internal note link activated', {
      from: store.openedNotePath || '',
      to: resolved.path,
      hasQuote: !!resolved.quote,
      anchor: resolved.anchor || ''
    })

    if (resolved.path !== store.openedNotePath) {
      store.openNote({
        path: resolved.path,
        title: resolved.path.split('/').pop()?.replace(/\.md$/i, '') || 'Untitled',
        kind: 'note',
        type: 'note'
      })
    }
    if (resolved.quote || resolved.anchor) {
      target.setTimeout(() => scrollToReference(resolved, target), 0)
    }
  }

  const observer = new target.MutationObserver(ensureCitationButton)
  observer.observe(target.document.documentElement, { childList: true, subtree: true })
  target.document.addEventListener('click', handleInternalLinkClick, true)
  target.document.addEventListener('selectionchange', updateSelectionButton)
  target.document.addEventListener('mouseup', updateSelectionButton)
  ensureCitationButton()
  renderPalette()

  const runtime = {
    copyCitation,
    pasteCitation,
    buffer,
    dispose() {
      observer.disconnect()
      target.document.removeEventListener('click', handleInternalLinkClick, true)
      target.document.removeEventListener('selectionchange', updateSelectionButton)
      target.document.removeEventListener('mouseup', updateSelectionButton)
      citationButton?.remove()
      selectionButton?.remove()
      lastEditorSelectionRange = null
      palette?.remove()
      removeContextMenu()
      target.document.querySelector('[data-elephant-citation-feedback]')?.remove()
      if (target[RUNTIME_KEY] === runtime) delete target[RUNTIME_KEY]
    }
  }
  target[RUNTIME_KEY] = runtime
  appendDebugLog(target, 'info', '[elephantnote:citation] runtime installed')
  return runtime
}
