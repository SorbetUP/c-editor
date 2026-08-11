import { createHash } from 'node:crypto'
import { mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises'
import path from 'node:path'

const browserFor = () => globalThis.browser

export const sleep = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds))

export const optional = async (selector) => {
  const element = await browserFor().$(selector)
  return (await element.isExisting()) ? element : null
}

export const requireElement = async (selector, description) => {
  const element = await browserFor().$(selector)
  await element.waitForExist({ timeout: 20000, timeoutMsg: `Missing real WebDriver target: ${description} (${selector})` })
  await element.waitForDisplayed({ timeout: 20000, timeoutMsg: `Hidden real WebDriver target: ${description} (${selector})` })
  return element
}

export const displayed = async (selector) => {
  const element = await optional(selector)
  return Boolean(element && await element.isDisplayed().catch(() => false))
}

export const byText = async (selector, text, description = selector) => {
  const elements = await browserFor().$$(selector)
  for (const element of elements) {
    if ((await element.getText()).trim().includes(text)) return element
  }
  throw new Error(`Missing real WebDriver ${description} containing ${JSON.stringify(text)}`)
}

export const visibleByText = async (selector, text, description = selector) => {
  const element = await byText(selector, text, description)
  if (!await element.isDisplayed()) throw new Error(`Real WebDriver ${description} is not displayed`)
  return element
}

const texts = async (selector) => {
  const result = []
  for (const element of await browserFor().$$(selector)) {
    if (await element.isDisplayed().catch(() => false)) result.push((await element.getText()).trim())
  }
  return result.filter(Boolean)
}

const value = async (selector) => {
  const element = await optional(selector)
  return element && await element.isDisplayed().catch(() => false) ? await element.getValue() : ''
}

const attribute = async (element, name) => element ? await element.getAttribute(name) : null

export const railOrder = async () => {
  const result = []
  for (const element of await browserFor().$$('.en-rail-nav > button.en-rail-icon')) {
    if (!await element.isDisplayed().catch(() => false)) continue
    const label = await attribute(element, 'aria-label')
    if (label === 'Search') result.push('search')
    else if (label === 'Hide sidebar' || label === 'Show sidebar') result.push('sidebar-toggle')
    else if (label) result.push(label)
  }
  return result
}

export const stateSnapshot = async (vaultRoot, action = {}) => {
  const editorOpen = await displayed('.en-note-editor-shell')
  const searchVisible = await displayed('.en-search-overlay')
  const menuVisible = await displayed('.en-create-menu-popover')
  const visibleEntries = await texts('.en-note-card h3, .en-folder-card h3')
  const labels = [
    ...(await texts('.en-rail button[aria-label]')),
    ...(await texts('.en-create-menu-popover [role="menuitem"]'))
  ]
  const editor = await optional('.en-editor-host .editor-component, .en-note-editor-shell')
  const editorHost = await optional('.en-editor-host')
  const editorText = editorOpen && editorHost ? await editorHost.getText().catch(() => '') : ''
  const query = await value('[placeholder="Search notes, paths, tags, or ideas…"]')
  const resultTitles = await texts('.en-search-result-title')
  const body = await readFile(path.join(vaultRoot, 'Alpha.md'), 'utf8').catch(() => '')
  return {
    route: editorOpen ? 'note-editor' : 'library',
    vaultName: path.basename(vaultRoot),
    visibleEntries: visibleEntries.filter((entry) => ['Alpha note', 'Projects'].includes(entry)),
    sidebarVisible: await displayed('.en-sidebar'),
    errors: await texts('[role="alert"], .en-error, .en-settings-feedback.is-error'),
    searchVisible,
    query,
    resultTitles,
    libraryStillMounted: !editorOpen,
    currentPath: '',
    openNote: editorOpen ? 'Alpha note' : null,
    notePath: editorOpen ? 'Alpha.md' : null,
    bodyContains: body.includes('Visible alpha body line.') ? ['Visible alpha body line.'] : [],
    editorText,
    closeControl: editorOpen && await displayed('[aria-label="Close note"]') ? 'Close note' : null,
    persistedFile: {
      path: 'Alpha.md',
      mustContain: 'Differential edit marker 2026-06-22.',
      contains: body.includes('Differential edit marker 2026-06-22.')
    },
    scroll: action.id === 'scroll-alpha-note' ? {
      requestedDeltaY: action.delta?.y ?? 560,
      mustChange: Boolean(action.scrollChanged)
    } : null,
    createMenuVisible: menuVisible,
    menuItems: menuVisible ? ['Note', 'Drawing', 'Folder'].filter((item) => labels.includes(item)) : [],
    railOrder: await railOrder(),
    accessibilityLabels: labels,
    geometry: {}
  }
}

const pngDimensions = (bytes) => ({ width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) })

export const captureFrame = async ({ outputRoot, checkpoint, index, relativeMs, kind = 'static' }) => {
  const relativePath = path.join('frames', checkpoint, `frame-${String(index).padStart(3, '0')}-${relativeMs}ms.png`)
  const filename = path.join(outputRoot, relativePath)
  await mkdir(path.dirname(filename), { recursive: true })
  await browserFor().saveScreenshot(filename)
  const bytes = await readFile(filename)
  return {
    index,
    relativeMs,
    kind,
    path: relativePath.split(path.sep).join('/'),
    sourcePath: filename,
    bytes: bytes.length,
    width: pngDimensions(bytes).width,
    height: pngDimensions(bytes).height,
    sha256: createHash('sha256').update(bytes).digest('hex'),
    capturedAtMs: Date.now()
  }
}

const walk = async (root, current = root) => {
  const output = []
  for (const entry of await readdir(current, { withFileTypes: true })) {
    const filename = path.join(current, entry.name)
    if (entry.isDirectory()) output.push(...await walk(root, filename))
    else if (entry.isFile()) output.push(filename)
  }
  return output
}

export const snapshotVault = async (vaultRoot) => {
  const files = []
  for (const filename of await walk(vaultRoot)) {
    const bytes = await readFile(filename)
    files.push({
      path: path.relative(vaultRoot, filename).split(path.sep).join('/'),
      sha256: createHash('sha256').update(bytes).digest('hex')
    })
  }
  return files.sort((left, right) => left.path.localeCompare(right.path))
}

export const writeJson = async (filename, value) => {
  await mkdir(path.dirname(filename), { recursive: true })
  await writeFile(filename, `${JSON.stringify(value, null, 2)}\n`, 'utf8')
}
