const FALLBACK_CLASS = 'en-addon-content-disabled'
const BADGE_CLASS = 'en-addon-content-fallback-badge'
const STYLE_ID = 'elephant-addon-content-fallback-style'
const ORIGINAL_TITLE_ATTR = 'data-addon-content-original-title'

const normalizeSlashes = (value = '') => String(value || '').replaceAll('\\', '/')

const sourceValue = (img) => [
  img?.getAttribute?.('data-src'),
  img?.getAttribute?.('data-origin-src'),
  img?.getAttribute?.('data-original-src'),
  img?.getAttribute?.('src'),
  img?.currentSrc
].find(Boolean) || ''

const sourcePath = (value = '') => {
  const text = String(value || '').split(/[?#]/)[0]
  if (/^file:/i.test(text)) {
    try {
      return normalizeSlashes(decodeURIComponent(new URL(text).pathname))
    } catch {
      return normalizeSlashes(text.replace(/^file:\/\//i, ''))
    }
  }
  return normalizeSlashes(text)
}

const globToRegExp = (pattern = '') => {
  const escaped = normalizeSlashes(pattern).replace(/[.+?^${}()|[\]\\]/g, '\\$&')
  const expression = escaped
    .replaceAll('**', '::DOUBLE_STAR::')
    .replaceAll('*', '[^/]*')
    .replaceAll('::DOUBLE_STAR::', '.*')
  return new RegExp(`(?:^|/)${expression.replace(/^\.\*\//, '')}$`, 'i')
}

const collectDescriptors = (manager) => {
  const manifests = new Map()
  for (const entry of manager.listBuiltinCatalog?.() || []) {
    if (entry?.manifest?.id) manifests.set(entry.manifest.id, entry.manifest)
  }
  for (const addon of manager.list?.() || []) {
    if (addon?.manifest?.id) manifests.set(addon.manifest.id, addon.manifest)
  }

  const descriptors = []
  for (const manifest of manifests.values()) {
    for (const contentType of manifest.contentTypes || []) {
      try {
        descriptors.push({
          addonId: manifest.id,
          addonName: manifest.name,
          ...contentType,
          matcher: globToRegExp(contentType.sourcePattern)
        })
      } catch (error) {
        console.warn('[addons] ignored invalid content fallback pattern', {
          addonId: manifest.id,
          pattern: contentType.sourcePattern,
          error
        })
      }
    }
  }
  return descriptors
}

const findDescriptor = (img, descriptors) => {
  const pathname = sourcePath(sourceValue(img))
  if (!pathname) return null
  return descriptors.find((descriptor) => descriptor.kind === 'image' && descriptor.matcher.test(pathname)) || null
}

const contentContainer = (img) => img.closest('.ag-image, .ag-image-container, .image-wrapper, figure') || img.parentElement

const restoreImage = (img) => {
  if (!img?.hasAttribute?.(ORIGINAL_TITLE_ATTR)) return
  const originalTitle = img.getAttribute(ORIGINAL_TITLE_ATTR) || ''
  if (originalTitle) img.setAttribute('title', originalTitle)
  else img.removeAttribute('title')
  img.removeAttribute(ORIGINAL_TITLE_ATTR)
  img.removeAttribute('aria-hidden')
}

const clearFallback = (container) => {
  if (!container) return
  container.classList.remove(FALLBACK_CLASS)
  container.removeAttribute('data-addon-content-owner')
  container.removeAttribute('data-addon-content-type')
  container.removeAttribute('data-addon-content-presentation')
  container.removeAttribute('data-addon-content-installed')
  container.querySelector?.(`:scope > .${BADGE_CLASS}`)?.remove()
  for (const img of container.querySelectorAll?.('img') || []) restoreImage(img)
}

const fallbackPresentation = (descriptor, installed) => installed
  ? descriptor.disabledPresentation
  : 'placeholder'

const isCurrentFallback = (container, descriptor, installed) => (
  container?.classList?.contains(FALLBACK_CLASS) &&
  container.dataset.addonContentOwner === descriptor.addonId &&
  container.dataset.addonContentType === descriptor.id &&
  container.dataset.addonContentPresentation === fallbackPresentation(descriptor, installed) &&
  container.dataset.addonContentInstalled === String(installed)
)

const applyFallback = (img, descriptor, installed) => {
  const container = contentContainer(img)
  if (!container) return
  if (isCurrentFallback(container, descriptor, installed)) {
    for (const control of container.querySelectorAll('.en-excalidraw-edit-button')) control.remove()
    return
  }

  clearFallback(container)
  const presentation = fallbackPresentation(descriptor, installed)
  container.classList.add(FALLBACK_CLASS)
  container.dataset.addonContentOwner = descriptor.addonId
  container.dataset.addonContentType = descriptor.id
  container.dataset.addonContentPresentation = presentation
  container.dataset.addonContentInstalled = String(installed)
  if (!img.hasAttribute(ORIGINAL_TITLE_ATTR)) img.setAttribute(ORIGINAL_TITLE_ATTR, img.getAttribute('title') || '')

  const title = installed
    ? `${descriptor.disabledLabel} — enable ${descriptor.addonName} to edit`
    : 'Optional content unavailable — install the matching addon to view or edit it'
  img.setAttribute('title', title)

  if (presentation === 'hidden') img.setAttribute('aria-hidden', 'true')
  else img.removeAttribute('aria-hidden')

  const badge = document.createElement('span')
  badge.className = BADGE_CLASS
  badge.setAttribute('contenteditable', 'false')
  badge.textContent = installed
    ? `${descriptor.disabledLabel} · ${descriptor.addonName} disabled`
    : 'Optional content unavailable'
  container.appendChild(badge)

  for (const control of container.querySelectorAll('.en-excalidraw-edit-button')) control.remove()
}

const ensureStyles = () => {
  if (document.getElementById(STYLE_ID)) return
  const style = document.createElement('style')
  style.id = STYLE_ID
  style.textContent = `
.${FALLBACK_CLASS} { position: relative !important; }
.${FALLBACK_CLASS}[data-addon-content-presentation="static-preview"] img { opacity: .88; filter: saturate(.82); }
.${FALLBACK_CLASS}[data-addon-content-presentation="placeholder"] img,
.${FALLBACK_CLASS}[data-addon-content-presentation="hidden"] img { visibility: hidden !important; min-height: 96px; }
.${FALLBACK_CLASS} .${BADGE_CLASS} { position: absolute; inset: auto 10px 10px 10px; z-index: 4; display: inline-flex; width: max-content; max-width: calc(100% - 20px); padding: 5px 8px; border: 1px solid var(--en-border, rgba(128,128,128,.4)); border-radius: 7px; background: color-mix(in srgb, var(--en-surface, #fff) 92%, transparent); color: var(--en-muted, #667085); font: 600 11px/1.2 system-ui, sans-serif; pointer-events: none; }
`
  document.head.appendChild(style)
}

export const installAddonContentFallbackRuntime = (manager, target = globalThis) => {
  if (!target?.document || !manager) return { dispose() {} }
  const existing = target.__ELEPHANT_ADDON_CONTENT_FALLBACKS__
  if (existing?.dispose) return existing

  ensureStyles()
  let disposed = false
  let descriptors = collectDescriptors(manager)
  let scheduled = false

  const refresh = () => {
    if (disposed) return
    descriptors = collectDescriptors(manager)
    for (const img of document.querySelectorAll('img')) {
      const descriptor = findDescriptor(img, descriptors)
      const container = contentContainer(img)
      if (!descriptor) {
        if (container?.classList?.contains(FALLBACK_CLASS)) clearFallback(container)
        continue
      }
      const addon = manager.get?.(descriptor.addonId)
      if (addon?.enabled === true) clearFallback(container)
      else applyFallback(img, descriptor, Boolean(addon))
    }
  }

  const scheduleRefresh = () => {
    if (disposed || scheduled) return
    scheduled = true
    const schedule = target.requestAnimationFrame?.bind(target) || target.setTimeout?.bind(target) || setTimeout
    schedule(() => {
      scheduled = false
      refresh()
    })
  }

  const observer = new MutationObserver(scheduleRefresh)
  observer.observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: ['src', 'data-src', 'data-origin-src', 'data-original-src'] })
  const offChanged = manager.on?.('changed', scheduleRefresh) || (() => {})
  const offRegistered = manager.on?.('registered', scheduleRefresh) || (() => {})
  const offUnregistered = manager.on?.('unregistered', scheduleRefresh) || (() => {})
  const blockDisabledInteraction = (event) => {
    if (!event.target?.closest?.(`.${FALLBACK_CLASS}`)) return
    event.preventDefault()
    event.stopPropagation()
  }
  document.addEventListener('pointerdown', blockDisabledInteraction, true)

  const runtime = {
    refresh,
    dispose() {
      disposed = true
      observer.disconnect()
      offChanged()
      offRegistered()
      offUnregistered()
      document.removeEventListener('pointerdown', blockDisabledInteraction, true)
      for (const container of document.querySelectorAll(`.${FALLBACK_CLASS}`)) clearFallback(container)
      document.getElementById(STYLE_ID)?.remove()
      if (target.__ELEPHANT_ADDON_CONTENT_FALLBACKS__ === runtime) delete target.__ELEPHANT_ADDON_CONTENT_FALLBACKS__
    }
  }

  target.__ELEPHANT_ADDON_CONTENT_FALLBACKS__ = runtime
  scheduleRefresh()
  return runtime
}
