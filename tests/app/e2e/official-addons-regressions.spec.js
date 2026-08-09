const fs = require('node:fs')
const path = require('node:path')
const { test, expect } = require('playwright/test')
const { createSeededVaultFixture, launchElectron } = require('./helpers')

const root = process.cwd()
const catalogPath = path.join(root, 'addons', 'catalog.json')
if (!fs.existsSync(catalogPath)) {
  throw new Error('Official addon E2E requires pnpm addons:sync before Playwright starts')
}
const catalog = JSON.parse(fs.readFileSync(catalogPath, 'utf8'))
const featureMatrix = JSON.parse(fs.readFileSync(path.join(root, 'tests/app/usage/feature-matrix.json'), 'utf8'))
const serviceAddonIds = new Set(featureMatrix.officialAddons
  .filter((addon) => addon.runtime === 'service')
  .map((addon) => addon.id))

const scenarioMarkers = Object.freeze({
  install: 'official-addon-install-enable-contribute',
  reload: 'official-addon-reload-restores-state',
  visibleFailure: 'official-addon-failure-is-visible',
  cleanup: 'official-addon-disable-uninstall-cleans-up'
})

const recursiveFiles = (directory) => {
  if (!fs.existsSync(directory)) return []
  const results = []
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const filename = path.join(directory, entry.name)
    if (entry.isDirectory()) results.push(...recursiveFiles(filename))
    else results.push(filename)
  }
  return results
}

const assertNativePackageEvidence = async (addon, testInfo) => {
  const addonRoot = path.join(root, 'addons', 'official', addon.slug)
  const manifestPath = path.join(root, 'addons', addon.manifestPath)
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'))
  const evidence = {
    addonId: addon.id,
    sourceRoot: addonRoot,
    manifestPath,
    nativeRequested: manifest.permissions?.native === true,
    sidecars: manifest.native?.sidecars || {},
    buildDescriptor: path.join(addonRoot, 'addon.build.json'),
    nativeManifest: path.join(addonRoot, 'native', 'Cargo.toml'),
    releaseRoot: path.join(root, 'build', 'out', 'addons', 'releases', addon.id),
    releaseFiles: []
  }

  expect(evidence.nativeRequested, `${addon.id} must declare native permission`).toBe(true)
  expect(Object.keys(evidence.sidecars).length, `${addon.id} must declare sidecars`).toBeGreaterThan(0)
  expect(fs.existsSync(evidence.buildDescriptor), `${addon.id} is missing addon.build.json`).toBe(true)
  expect(fs.existsSync(evidence.nativeManifest), `${addon.id} is missing native/Cargo.toml`).toBe(true)

  evidence.releaseFiles = recursiveFiles(evidence.releaseRoot)
    .map((filename) => path.relative(root, filename).replaceAll('\\', '/'))
  if (process.env.ELEPHANT_E2E_REQUIRE_NATIVE_PACKAGES === '1') {
    expect(evidence.releaseFiles.some((filename) => filename.endsWith('.enaddon')),
      `${addon.id} has no built .enaddon package under ${evidence.releaseRoot}`).toBe(true)
  }

  const evidencePath = testInfo.outputPath(`${addon.id}-native-package-evidence.json`)
  fs.writeFileSync(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`)
  await testInfo.attach(`${addon.id}-native-package-evidence`, {
    path: evidencePath,
    contentType: 'application/json'
  })
  console.log(`[official-addon-evidence:${addon.id}] ${JSON.stringify(evidence)}`)
  return evidence
}

const launchOfficialAddonApp = async (testInfo) => {
  const fixture = await createSeededVaultFixture()
  const launch = await launchElectron([], {
    userDataPath: fixture.userDataPath,
    env: {
      ELEPHANTNOTE_CONFIG_DIR: fixture.configRoot,
      ELEPHANT_E2E_VAULT_ROOT: fixture.vaultRoot,
      ELEPHANTNOTE_MUYA_RUNTIME: 'rust',
      ELEPHANT_E2E_OFFICIAL_ADDONS: 'all'
    }
  })
  const { app, page } = launch
  const errors = []
  page.on('pageerror', (error) => errors.push(`pageerror: ${error?.stack || error?.message || String(error)}`))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(`console.error: ${message.text()}`)
  })
  await page.setViewportSize({ width: 1366, height: 900 })
  await page.waitForSelector('.en-library-grid', { state: 'visible', timeout: 30000 })
  await expect.poll(() => page.evaluate(() => Boolean(window.__ELEPHANT_ADDONS__?.external)), {
    timeout: 30000
  }).toBe(true)
  await expect.poll(() => page.evaluate(() => window.__ELEPHANT_ADDONS__?.external?.records?.size || 0), {
    timeout: 30000
  }).toBe(catalog.addons.length)

  return {
    app,
    page,
    fixture,
    errors,
    async checkpointState(name, value) {
      const resolved = typeof value === 'function' ? await value() : value
      const filename = testInfo.outputPath(`${name}.json`)
      fs.writeFileSync(filename, `${JSON.stringify(resolved, null, 2)}\n`)
      await testInfo.attach(name, { path: filename, contentType: 'application/json' })
      console.log(`[official-addon-checkpoint:${name}] ${JSON.stringify(resolved)}`)
    },
    async close() {
      await app.close().catch(() => {})
      fs.rmSync(fixture.root, { recursive: true, force: true })
    }
  }
}

const stateFor = (page, addonId) => page.evaluate(async (id) => {
  const manager = window.__ELEPHANT_ADDONS__
  const snapshot = manager?.get(id) || null
  const contributions = []
  const contributionMap = manager?.getContributionMap?.() || {}
  for (const [area, entries] of Object.entries(contributionMap)) {
    for (const entry of entries || []) {
      if (entry?.addonId !== id) continue
      contributions.push({
        area,
        id: entry?.contribution?.id || '',
        title: entry?.contribution?.title || entry?.contribution?.label || ''
      })
    }
  }
  const record = manager?.external?.getRecord?.(id) || null
  const trust = record
    ? await manager.external.getTrustState(id).catch((error) => ({ error: error?.message || String(error) }))
    : null
  const service = snapshot?.manifest?.permissions?.native
    ? await window.__TAURI__.core.invoke('tauri_addons_service_status', { addonId: id })
      .catch((error) => ({ error: error?.message || String(error) }))
    : null
  return {
    snapshot,
    contributions,
    official: manager?.external?.isOfficial?.(id) || false,
    trusted: manager?.external?.isTrusted?.(id) || false,
    trust,
    service,
    resources: manager?.host?.list?.() || []
  }
}, addonId)

const enableWithDependencies = (page, addonId) => page.evaluate(async (id) => {
  const manager = window.__ELEPHANT_ADDONS__
  const visited = new Set()
  const order = []
  const enable = async (currentId) => {
    if (visited.has(currentId)) return
    visited.add(currentId)
    const current = manager.get(currentId)
    if (!current) throw new Error(`Missing installed dependency ${currentId} for ${id}`)
    const dependencies = new Set(Object.keys(current.manifest?.requires || {}))
    if (current.manifest?.parentAddonId) dependencies.add(current.manifest.parentAddonId)
    for (const dependencyId of dependencies) await enable(dependencyId)
    const latest = manager.get(currentId)
    if (!latest.enabled) await manager.enable(currentId)
    order.push(currentId)
  }
  await enable(id)
  return order
}, addonId)

const uninstallWithDependents = (page, addonId) => page.evaluate(async (id) => {
  const manager = window.__ELEPHANT_ADDONS__
  const removed = []
  const visited = new Set()
  const uninstall = async (currentId) => {
    if (visited.has(currentId) || !manager.get(currentId)) return
    visited.add(currentId)
    for (const dependentId of manager.getDependents(currentId)) await uninstall(dependentId)
    const current = manager.get(currentId)
    if (current?.enabled || current?.status === 'error') await manager.disable(currentId).catch(() => {})
    await manager.external.uninstall(currentId)
    removed.push(currentId)
  }
  await uninstall(id)
  return removed
}, addonId)

const installOfficialAddon = (page, addonId) => page.evaluate(async (id) => {
  const manager = window.__ELEPHANT_ADDONS__
  const record = await manager.external.installFromPath(`official:${id}`)
  return { record, snapshot: manager.get(id) }
}, addonId)

const probeAddonFunctionality = (page, addonId, resourcesBefore = []) => page.evaluate(async ({ id, resourcesBefore }) => {
  const manager = window.__ELEPHANT_ADDONS__
  const contributionMap = manager.getContributionMap?.() || {}
  const probes = []
  for (const [area, entries] of Object.entries(contributionMap)) {
    for (const entry of entries || []) {
      if (entry?.addonId !== id) continue
      const contribution = entry.contribution || {}
      const probe = {
        area,
        id: contribution.id || '',
        title: contribution.title || contribution.label || '',
        executable: typeof contribution.run === 'function',
        stateful: typeof contribution.getState === 'function'
      }
      if (probe.stateful) {
        try {
          const value = await contribution.getState({ e2e: true, probe: true })
          probe.state = value === undefined ? null : value
        } catch (error) {
          probe.stateError = error?.message || String(error)
        }
      }
      probes.push(probe)
    }
  }
  const resourcesAfter = manager.host?.list?.() || []
  const newResources = resourcesAfter.filter((name) => !resourcesBefore.includes(name))
  return { probes, resourcesBefore, resourcesAfter, newResources }
}, { id: addonId, resourcesBefore })

for (const addon of catalog.addons) {
  test(`[official-addon:${addon.id}] install, enable, probe, reload and clean up`, async ({ page: _page }, testInfo) => {
    void _page
    if (serviceAddonIds.has(addon.id)) await assertNativePackageEvidence(addon, testInfo)
    const context = await launchOfficialAddonApp(testInfo)
    try {
      const preinstalled = await stateFor(context.page, addon.id)
      expect(preinstalled.snapshot, `${addon.id} was not materialized from the physical catalogue`).not.toBeNull()
      const removedForInstall = await uninstallWithDependents(context.page, addon.id)
      expect((await stateFor(context.page, addon.id)).snapshot).toBeNull()

      const installation = await installOfficialAddon(context.page, addon.id)
      expect(installation.record?.manifest?.id, `${scenarioMarkers.install}: wrong installed package`).toBe(addon.id)
      expect(installation.snapshot?.enabled).toBe(false)
      const installed = await stateFor(context.page, addon.id)
      expect(installed.official, `${addon.id} lost its official trust marker`).toBe(true)
      expect(installed.trust?.approved, `${addon.id} incorrectly requires community full-access approval`).toBe(true)

      const enableOrder = await enableWithDependencies(context.page, addon.id)
      const enabled = await stateFor(context.page, addon.id)
      expect(enabled.snapshot?.enabled, `${scenarioMarkers.install}: ${addon.id} did not enable`).toBe(true)
      expect(enabled.snapshot?.status).toBe('enabled')
      expect(enabled.snapshot?.error).toBeNull()
      if (enabled.service) {
        expect(enabled.service.error, `${addon.id} service status failed: ${enabled.service.error || ''}`).toBeUndefined()
      }

      const functionalProbe = await probeAddonFunctionality(context.page, addon.id, installed.resources)
      expect(functionalProbe.probes.length + functionalProbe.newResources.length,
        `${addon.id} enabled without any observable contribution or resource`).toBeGreaterThan(0)
      expect(functionalProbe.probes.filter((probe) => probe.stateError),
        `${addon.id} view/state probes failed`).toEqual([])
      await context.checkpointState(`${addon.id}-installed-enabled-probed`, {
        preinstalled,
        removedForInstall,
        installation,
        installed,
        enableOrder,
        enabled,
        functionalProbe
      })

      await context.page.reload()
      await context.page.waitForSelector('.en-library-grid', { state: 'visible', timeout: 30000 })
      await expect.poll(async () => (await stateFor(context.page, addon.id)).snapshot?.status, {
        timeout: 30000
      }).toBe('enabled')
      const reloaded = await stateFor(context.page, addon.id)
      expect(reloaded.snapshot?.enabled, `${scenarioMarkers.reload}: ${addon.id} was not restored`).toBe(true)
      expect(reloaded.snapshot?.error).toBeNull()
      await context.checkpointState(`${addon.id}-reloaded`, reloaded)

      const removed = await uninstallWithDependents(context.page, addon.id)
      const afterCleanup = await stateFor(context.page, addon.id)
      expect(afterCleanup.snapshot, `${scenarioMarkers.cleanup}: ${addon.id} remained registered`).toBeNull()
      await context.checkpointState(`${addon.id}-cleanup`, { removed, afterCleanup })

      expect(context.errors, context.errors.join('\n')).toEqual([])
      expect(await context.page.locator('.en-addons-feedback.error:visible').count()).toBe(0)
    } finally {
      await context.close()
    }
  })
}

test(`[official-addon-platform] ${scenarioMarkers.visibleFailure}`, async ({ page: _page }, testInfo) => {
  void _page
  const context = await launchOfficialAddonApp(testInfo)
  try {
    const result = await context.page.evaluate(async () => {
      const manager = window.__ELEPHANT_ADDONS__
      const id = 'elephant.e2e-visible-activation-failure'
      manager.register({
        manifest: {
          id,
          name: 'Visible activation failure',
          version: '1.0.0',
          description: 'Deliberate E2E failure used to verify error observability.',
          author: 'Elephant CI',
          source: 'builtin',
          defaultEnabled: false,
          removable: true
        },
        async activate() {
          throw new Error('E2E deliberate addon activation failure 7719')
        }
      })
      let thrown = ''
      try { await manager.enable(id) } catch (error) { thrown = error?.message || String(error) }
      return { thrown, snapshot: manager.get(id) }
    })
    expect(result.thrown).toContain('E2E deliberate addon activation failure 7719')
    expect(result.snapshot?.status).toBe('error')
    expect(result.snapshot?.error?.message).toContain('E2E deliberate addon activation failure 7719')
    await expect.poll(() => context.errors.some((entry) => entry.includes('addon activation failed'))).toBe(true)
    await context.checkpointState('official-addon-visible-failure', { result, errors: context.errors })
  } finally {
    await context.close()
  }
})
