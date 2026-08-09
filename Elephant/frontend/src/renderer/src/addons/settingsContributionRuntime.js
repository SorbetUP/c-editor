import { ADDON_EXTENSION_POINTS } from './extensionPoints'

const SETTINGS_CONTENT = '.en-settings-content'
const SETTINGS_HOST_ATTR = 'data-elephant-addon-settings-host'
const SETTINGS_SLOT_ATTR = 'data-elephant-addon-settings-slot'

const normalizeText = (value, fallback = '') => {
  if (typeof value !== 'string') return fallback
  return value.trim() || fallback
}

const activeSettingsSection = (content) => {
  const explicit = normalizeText(content?.dataset?.activeSection).toLowerCase()
  if (explicit) return explicit
  const title = normalizeText(content?.querySelector?.('.en-settings-page-title h1')?.textContent).toLowerCase()
  const labels = {
    appearance: 'appearance',
    editor: 'editor',
    vaults: 'vaults',
    addons: 'addons',
    sync: 'sync',
    ai: 'ai'
  }
  return labels[title] || ''
}

const targetSection = (contribution = {}) => {
  return normalizeText(
    contribution.section || contribution.targetSection || contribution.parentSection,
    'addons'
  ).toLowerCase()
}

const targetSlot = (contribution = {}) => normalizeText(
  contribution.slot || contribution.mountSlot || contribution.settingsSlot
)

const hasRenderableSettings = (contribution = {}) => {
  return typeof contribution.render === 'function' || (Array.isArray(contribution.fields) && contribution.fields.length > 0)
}

const removeNode = (node) => node?.remove?.()

const collectScopeAttributes = (content) => {
  const names = new Set()
  let current = content
  while (current) {
    for (const attribute of current.attributes || []) {
      if (attribute.name.startsWith('data-v-')) names.add(attribute.name)
    }
    if (current.classList?.contains('en-settings-panel')) break
    current = current.parentElement
  }
  return [...names]
}

const applyScopeAttributes = (root, attributeNames) => {
  if (!root || !attributeNames.length) return
  const elements = [root, ...root.querySelectorAll?.('*') || []]
  for (const element of elements) {
    if (!element?.setAttribute) continue
    for (const name of attributeNames) element.setAttribute(name, '')
  }
}

const observeScopedContent = (root, attributeNames) => {
  applyScopeAttributes(root, attributeNames)
  if (!attributeNames.length || typeof MutationObserver !== 'function') return () => {}
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      for (const node of mutation.addedNodes || []) {
        if (node?.nodeType === 1) applyScopeAttributes(node, attributeNames)
      }
    }
  })
  observer.observe(root, { childList: true, subtree: true })
  return () => observer.disconnect()
}

const findSlot = (content, slotName) => {
  if (!slotName) return content
  return [...content.querySelectorAll(`[${SETTINGS_SLOT_ATTR}]`)]
    .find((node) => normalizeText(node.getAttribute(SETTINGS_SLOT_ATTR)) === slotName) || null
}

const renderField = (documentRef, field, contribution) => {
  const row = documentRef.createElement('div')
  row.className = 'en-settings-row'
  const copy = documentRef.createElement('div')
  copy.className = 'en-settings-row-copy'
  const title = documentRef.createElement('strong')
  title.textContent = normalizeText(field?.label, normalizeText(field?.id, 'Setting'))
  const description = documentRef.createElement('span')
  description.textContent = normalizeText(field?.description)
  copy.append(title, description)
  row.append(copy)

  const read = () => typeof field?.get === 'function' ? field.get() : field?.value
  const write = async (value) => {
    if (typeof field?.set === 'function') await field.set(value)
    else field.value = value
    if (typeof contribution?.onChange === 'function') await contribution.onChange(field.id, value)
  }

  if (field?.type === 'boolean') {
    const button = documentRef.createElement('button')
    button.className = `en-switch${read() === true ? ' active' : ''}`
    button.type = 'button'
    button.setAttribute('role', 'switch')
    button.setAttribute('aria-checked', read() === true ? 'true' : 'false')
    button.addEventListener('click', async () => {
      const next = !(read() === true)
      await write(next)
      button.classList.toggle('active', next)
      button.setAttribute('aria-checked', next ? 'true' : 'false')
    })
    button.append(documentRef.createElement('span'))
    row.append(button)
  } else if (field?.type === 'select') {
    const select = documentRef.createElement('select')
    select.className = 'en-compact-select'
    for (const optionDefinition of field.options || []) {
      const option = documentRef.createElement('option')
      const value = typeof optionDefinition === 'object' ? optionDefinition.value : optionDefinition
      option.value = String(value ?? '')
      option.textContent = typeof optionDefinition === 'object'
        ? normalizeText(optionDefinition.label, String(value ?? ''))
        : String(value ?? '')
      option.selected = option.value === String(read() ?? '')
      select.append(option)
    }
    select.addEventListener('change', () => write(select.value))
    row.append(select)
  } else if (field?.type === 'button') {
    const button = documentRef.createElement('button')
    button.className = field.danger ? 'en-danger-button' : 'en-secondary-button'
    button.type = 'button'
    button.textContent = normalizeText(field.buttonLabel, normalizeText(field.label, 'Run'))
    button.addEventListener('click', () => field.run?.())
    row.append(button)
  } else {
    const input = documentRef.createElement('input')
    input.className = 'en-compact-input'
    input.type = field?.type === 'number' ? 'number' : 'text'
    input.value = String(read() ?? '')
    input.placeholder = normalizeText(field?.placeholder)
    input.addEventListener('change', () => write(field?.type === 'number' ? Number(input.value) : input.value))
    row.append(input)
  }

  return row
}

const renderContribution = (documentRef, content, target, entry, manager, addonHost) => {
  const contribution = entry.contribution || {}
  const bare = contribution.chrome === false || contribution.renderMode === 'bare'
  const section = documentRef.createElement(bare ? 'div' : 'section')
  section.className = bare
    ? 'en-addon-settings-bare-host'
    : 'en-settings-group en-addon-settings-extension'
  section.setAttribute(SETTINGS_HOST_ATTR, 'true')
  section.dataset.addonId = entry.addonId
  section.dataset.contributionId = normalizeText(contribution.id)

  let body = section
  if (!bare) {
    const heading = documentRef.createElement('div')
    heading.className = 'en-addon-settings-extension-heading'
    const title = documentRef.createElement('strong')
    title.textContent = normalizeText(contribution.title, contribution.id || entry.addonId)
    const description = documentRef.createElement('span')
    description.textContent = normalizeText(contribution.description)
    heading.append(title, description)
    section.append(heading)

    body = documentRef.createElement('div')
    body.className = 'en-addon-settings-extension-body'
    section.append(body)
  }

  const cleanup = []
  for (const field of contribution.fields || []) body.append(renderField(documentRef, field, contribution))

  if (typeof contribution.render === 'function') {
    const result = contribution.render(body, {
      addonId: entry.addonId,
      contribution,
      manager,
      host: addonHost
    })
    if (typeof result === 'function') cleanup.push(result)
    else if (typeof result?.dispose === 'function') cleanup.push(() => result.dispose())
    else if (result?.nodeType) {
      body.append(result)
      cleanup.push(() => removeNode(result))
    }
  }

  target.append(section)
  const stopScopeObserver = observeScopedContent(section, collectScopeAttributes(content))

  return {
    section,
    target,
    dispose() {
      stopScopeObserver()
      for (const dispose of cleanup.reverse()) {
        try { dispose() } catch {}
      }
      removeNode(section)
    }
  }
}

const contributionSignature = (entry, mountTarget) => [
  entry.addonId,
  entry.contribution?.id || '',
  targetSlot(entry.contribution),
  mountTarget?.dataset?.activeAddonSlotKey || ''
].join(':')

export const installSettingsContributionRuntime = (manager, options = {}) => {
  const target = options.target || globalThis
  const documentRef = target?.document
  if (!documentRef || !manager) return { dispose() {} }

  let disposed = false
  let scheduled = false
  let mounted = []

  const clear = () => {
    for (const entry of mounted.splice(0).reverse()) entry.dispose()
  }

  const sync = () => {
    scheduled = false
    if (disposed) return
    const content = documentRef.querySelector(SETTINGS_CONTENT)
    if (!content) {
      clear()
      return
    }

    const activeSection = activeSettingsSection(content)
    const candidates = [
      ...manager.getContributions(ADDON_EXTENSION_POINTS.settingsSections),
      ...manager.getContributions(ADDON_EXTENSION_POINTS.settingsPages)
    ].filter((entry) => targetSection(entry.contribution) === activeSection && hasRenderableSettings(entry.contribution))

    const descriptors = candidates
      .map((entry) => ({
        entry,
        slotName: targetSlot(entry.contribution),
        target: findSlot(content, targetSlot(entry.contribution))
      }))
      .filter(({ target: mountTarget }) => Boolean(mountTarget))
      .map((descriptor) => ({
        ...descriptor,
        signature: contributionSignature(descriptor.entry, descriptor.target)
      }))

    const desiredSignatures = new Set(descriptors.map((descriptor) => descriptor.signature))
    const kept = []
    for (const current of mounted) {
      const keep = desiredSignatures.has(current.signature) && current.section.isConnected && current.target.isConnected
      if (keep) kept.push(current)
      else current.dispose()
    }
    mounted = kept

    if (!activeSection || !descriptors.length) return

    const mountedSignatures = new Set(mounted.map((entry) => entry.signature))
    let mountedRootContribution = false
    for (const descriptor of descriptors) {
      if (mountedSignatures.has(descriptor.signature) || !descriptor.target.isConnected) continue
      const rendered = renderContribution(documentRef, content, descriptor.target, descriptor.entry, manager, manager.host)
      rendered.signature = descriptor.signature
      mounted.push(rendered)
      mountedSignatures.add(descriptor.signature)
      if (!descriptor.slotName) mountedRootContribution = true
    }

    // A root settings page can create named slots used by another addon (for
    // example AI creates the Codex provider slot). Resolve those slots on the
    // next pass instead of clearing and remounting the root page in a loop.
    if (mountedRootContribution) schedule()
  }

  const schedule = () => {
    if (scheduled || disposed) return
    scheduled = true
    queueMicrotask(sync)
  }

  const observer = new MutationObserver(schedule)
  observer.observe(documentRef.body, { childList: true, subtree: true })
  const offContribution = manager.on('contribution:changed', schedule)
  const offEnabled = manager.on('enabled', schedule)
  const offDisabled = manager.on('disabled', schedule)
  schedule()

  return {
    sync: schedule,
    dispose() {
      disposed = true
      observer.disconnect()
      offContribution()
      offEnabled()
      offDisabled()
      clear()
    }
  }
}
