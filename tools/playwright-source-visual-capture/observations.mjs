import { readFile, readdir } from 'node:fs/promises'
import path from 'node:path'

const visible = async (locator) => locator.count().then(async (count) => {
  for (let index = 0; index < count; index += 1) if (await locator.nth(index).isVisible().catch(() => false)) return true
  return false
})

const textList = async (locator) => (await locator.allTextContents()).map((value) => value.trim()).filter(Boolean)

export async function optionalText (locator, method = 'textContent') {
  if (await locator.count() === 0) return ''
  return String(await locator.first()[method]({ timeout: 0 }).catch(() => '') || '').trim()
}

export async function optionalInputValue (locator) {
  if (await locator.count() === 0) return null
  return await locator.first().inputValue({ timeout: 0 }).catch(() => null)
}

async function domLabels (page) {
  return page.locator('[aria-label],button,[role="menuitem"],[data-testid]').evaluateAll((elements) => {
    const labels = elements.map((element) => String(element.getAttribute('aria-label') || element.textContent || element.getAttribute('data-testid') || '').trim()).filter(Boolean)
    return [...new Set(labels)].sort()
  })
}

async function domGeometry (page) {
  return page.locator('[aria-label],button,[role="menuitem"],[data-testid]').evaluateAll((elements) => {
    const map = {}
    for (const element of elements) {
      const label = String(element.getAttribute('aria-label') || element.textContent || element.getAttribute('data-testid') || '').trim()
      if (!label) continue
      const rect = element.getBoundingClientRect()
      const entry = { x: Number(rect.x.toFixed(3)), y: Number(rect.y.toFixed(3)), width: Number(rect.width.toFixed(3)), height: Number(rect.height.toFixed(3)) }
      ;(map[label] ||= []).push(entry)
    }
    return map
  })
}

export async function railState (page, configRoot) {
  const labels = await page.locator('.en-rail-nav .en-rail-icon').evaluateAll((elements) => elements.map((element) => element.getAttribute('aria-label')).filter(Boolean))
  const order = labels.map((label) => label === 'Search' ? 'search' : label === 'Hide sidebar' || label === 'Show sidebar' ? 'sidebar-toggle' : label.includes('open vault') ? 'vault' : label)
  const preferencesPath = path.join(configRoot, 'preferences.json')
  let persistedOrder = null
  try {
    const preferences = JSON.parse(await readFile(preferencesPath, 'utf8'))
    if (Array.isArray(preferences?.iconRailOrder)) persistedOrder = preferences.iconRailOrder
  } catch {}
  return { order, persistedOrder, persistedPath: 'preferences.json' }
}

export async function editorScrollState (page) {
  const target = page.locator('.en-editor-host .editor-component').first()
  if (await target.count() === 0) return null
  return target.evaluate((element) => ({
    target: '.en-editor-host .editor-component',
    scrollTop: Number(element.scrollTop.toFixed(3)),
    scrollHeight: element.scrollHeight,
    clientHeight: element.clientHeight,
    scrollable: element.scrollHeight > element.clientHeight
  }))
}

async function vaultSnapshot (root) {
  const result = []
  async function walk (current, relative = '') {
    for (const entry of (await readdir(current, { withFileTypes: true })).sort((left, right) => left.name.localeCompare(right.name))) {
      const absolute = path.join(current, entry.name)
      const next = path.posix.join(relative, entry.name)
      if (entry.isDirectory()) await walk(absolute, next)
      else if (entry.isFile()) {
        const bytes = await readFile(absolute)
        const { createHash } = await import('node:crypto')
        result.push({ path: next, sha256: createHash('sha256').update(bytes).digest('hex') })
      }
    }
  }
  await walk(root)
  return result
}

export async function snapshotState (page, run, action, frames) {
  const editorVisible = await visible(page.locator('.en-note-editor-shell'))
  const searchInput = page.getByPlaceholder('Search notes, paths, tags, or ideas…')
  const searchVisible = await visible(searchInput)
  const query = searchVisible ? await searchInput.inputValue() : null
  const editor = page.getByTestId('muya-runtime-editor')
  const editorText = editorVisible ? await optionalText(editor, 'innerText') : ''
  const cards = await textList(page.locator('.en-note-card h3'))
  const resultTitles = await textList(page.locator('.en-search-result-title'))
  const menuItems = []
  for (const label of ['Note', 'Drawing', 'Folder']) if (await visible(page.getByRole('menuitem', { name: new RegExp(`^${label}\\b`) }))) menuItems.push(label)
  const vaultName = await optionalText(page.locator('.en-top-vault-name'))
  const errors = await textList(page.locator('.en-addons-feedback.error'))
  const noteTitle = editorVisible ? await optionalInputValue(page.getByRole('textbox', { name: 'Note title' })) : null
  const alpha = await readFile(path.join(run.vaultRoot, 'Alpha.md'), 'utf8').catch(() => '')
  const scroll = await editorScrollState(page)
  const rail = await railState(page, run.configRoot)
  const labels = await domLabels(page)
  return {
    route: editorVisible ? 'note-editor' : 'library',
    vaultName: String(vaultName || '').trim(),
    visibleEntries: cards,
    sidebarVisible: await visible(page.locator('.en-sidebar')),
    errors,
    searchVisible,
    query,
    resultTitles,
    libraryStillMounted: await visible(page.locator('.en-library-grid')),
    currentPath: '',
    openNote: noteTitle,
    notePath: editorVisible ? 'Alpha.md' : null,
    bodyContains: ['Visible alpha body line.', 'Differential edit marker 2026-06-22.'].filter((value) => editorText.includes(value)),
    editorText,
    closeControl: editorVisible ? 'Close note' : null,
    persistedFile: { path: 'Alpha.md', mustContain: 'Differential edit marker 2026-06-22.', contains: alpha.includes('Differential edit marker 2026-06-22.') },
    scroll: scroll ? { ...scroll, ...(action.id === 'scroll-alpha-note' ? { requestedDeltaY: action.delta.y } : {}) } : null,
    createMenuVisible: menuItems.length > 0,
    menuItems,
    railOrder: rail.order,
    railPersistence: rail,
    accessibilityLabels: labels,
    geometry: await domGeometry(page)
  }
}

export { vaultSnapshot }
