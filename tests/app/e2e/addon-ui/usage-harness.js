const fs = require('node:fs')
const path = require('node:path')
const { createSeededVaultFixture, launchElectron } = require('../helpers')

const root = process.cwd()
const catalog = JSON.parse(fs.readFileSync(path.join(root, 'addons', 'catalog.json'), 'utf8'))

const escapeRegExp = (value) => String(value).replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

const waitForAddon = async (page, addonId) => {
  await page.waitForFunction((id) => {
    const snapshot = window.__ELEPHANT_ADDONS__?.get?.(id)
    return snapshot?.enabled === true && snapshot?.status === 'enabled'
  }, addonId)
}

const enableWithDependencies = (page, addonId) => page.evaluate(async (id) => {
  const manager = window.__ELEPHANT_ADDONS__
  const visited = new Set()
  const enable = async (currentId) => {
    if (visited.has(currentId)) return
    visited.add(currentId)
    const snapshot = manager?.get?.(currentId)
    if (!snapshot) throw new Error(`Addon ${currentId} is not installed in the physical catalogue`)
    const dependencies = new Set(Object.keys(snapshot.manifest?.requires || {}))
    if (snapshot.manifest?.parentAddonId) dependencies.add(snapshot.manifest.parentAddonId)
    for (const dependency of dependencies) await enable(dependency)
    const latest = manager.get(currentId)
    if (!latest.enabled) await manager.enable(currentId)
  }
  await enable(id)
}, addonId)

const ensureAddon = async (page, addonId) => {
  const state = await page.evaluate((id) => {
    const snapshot = window.__ELEPHANT_ADDONS__?.get?.(id)
    return snapshot ? { enabled: snapshot.enabled === true, status: snapshot.status } : null
  }, addonId)
  if (!state) throw new Error(`${addonId} is absent from the physical catalogue`)
  if (!state.enabled) await enableWithDependencies(page, addonId)
  await waitForAddon(page, addonId)
}

const openSettings = async (page, section = '') => {
  const dialog = page.getByRole('dialog', { name: 'ElephantNote settings' })
  if (await dialog.count() === 0 || !(await dialog.isVisible().catch(() => false))) {
    await page.getByRole('button', { name: 'Settings', exact: true }).last().click()
  }
  await dialog.waitFor({ state: 'visible' })
  if (!section) return dialog
  const navigationButton = dialog.locator('.en-settings-nav button').filter({
    hasText: new RegExp(`^\\s*${escapeRegExp(section)}\\s*$`)
  }).first()
  await navigationButton.click()
  return dialog
}

const closeSettings = async (page) => {
  const close = page.getByRole('button', { name: 'Close settings', exact: true })
  if (await close.count() && await close.isVisible().catch(() => false)) await close.click()
}

const openSettingsSection = async (page, label) => {
  const dialog = await openSettings(page, label)
  await page.locator('.en-settings-content').waitFor({ state: 'visible' })
  return dialog
}

const openAddonView = async (page, title, selector) => {
  const railButton = page.locator('.en-rail button[aria-label]').filter({
    hasText: new RegExp(`^\\s*${escapeRegExp(title)}\\s*$`)
  }).first()
  if (await railButton.count() === 0) {
    await page.locator(`.en-rail button[aria-label="${title}"]`).first().click()
  } else {
    await railButton.click()
  }
  const view = page.locator(selector)
  await view.waitFor({ state: 'visible' })
  return view
}

const readVaultFile = (fixture, relativePath) => {
  const fullPath = path.join(fixture.vaultRoot, ...String(relativePath).split('/'))
  return fs.readFileSync(fullPath, 'utf8')
}

const readAddonHttpTrace = (fixture) => {
  const filename = path.join(fixture.configRoot, 'official-addons-http-trace.json')
  if (!fs.existsSync(filename)) return []
  return JSON.parse(fs.readFileSync(filename, 'utf8'))
}

const launchUiApp = async (testInfo, prepareFixture) => {
  const fixture = await createSeededVaultFixture()
  if (prepareFixture) await prepareFixture(fixture)
  const launch = await launchElectron([], {
    userDataPath: fixture.userDataPath,
    env: {
      ELEPHANTNOTE_CONFIG_DIR: fixture.configRoot,
      ELEPHANT_E2E_VAULT_ROOT: fixture.vaultRoot,
      ELEPHANTNOTE_MUYA_RUNTIME: 'rust',
      ELEPHANT_E2E_OFFICIAL_ADDONS: 'all',
      ELEPHANT_E2E_INTERCEPT_EXTERNAL_AI: '1'
    }
  })
  const errors = []
  const recordError = (prefix, value) => errors.push(`${prefix}: ${value?.stack || value?.message || String(value)}`)
  launch.page.on('pageerror', (error) => recordError('pageerror', error))
  launch.page.on('console', (message) => {
    if (message.type() === 'error') recordError('console.error', message.text())
  })
  await launch.page.setViewportSize({ width: 1366, height: 900 })
  await launch.page.waitForSelector('.en-library-grid', { state: 'visible', timeout: 30000 })
  await launch.page.waitForFunction(() => Boolean(window.__ELEPHANT_ADDONS__?.external))
  await launch.page.waitForFunction((count) => (
    (window.__ELEPHANT_ADDONS__?.external?.records?.size || 0) >= count
  ), catalog.addons.length)
  return {
    ...launch,
    fixture,
    errors,
    testInfo,
    async checkpoint(name, value) {
      const resolved = typeof value === 'function' ? await value() : value
      const filename = testInfo.outputPath(`${name}.json`)
      fs.writeFileSync(filename, `${JSON.stringify(resolved, null, 2)}\n`)
      await testInfo.attach(name, { path: filename, contentType: 'application/json' })
      return resolved
    },
    async close() {
      await launch.app.close().catch(() => {})
      fs.rmSync(fixture.root, { recursive: true, force: true })
    }
  }
}

module.exports = {
  catalog,
  closeSettings,
  enableWithDependencies,
  ensureAddon,
  launchUiApp,
  openAddonView,
  openSettings,
  openSettingsSection,
  readAddonHttpTrace,
  readVaultFile,
  waitForAddon
}
