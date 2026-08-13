'use strict'

const fs = require('node:fs')
const { test, expect } = require('playwright/test')
const harness = require('./example-addon-ui/tauri-harness')
const finance = require('./example-addon-ui/finance-notes')
const platformProof = require('./example-addon-ui/platform-proof')
const trustedLab = require('./example-addon-ui/trusted-workspace-lab')

const scenarios = [finance, platformProof, trustedLab]
test.setTimeout(300000)
test.describe.configure({ mode: 'serial' })

for (const scenario of scenarios) {
  test(`[example-addon-ui:${scenario.addonId}] install, activate, use, reload and clean up`, async (fixtures, testInfo) => {
  void fixtures
    testInfo.setTimeout(300000)
    const fixture = harness.createFixture()
    let phase = 'package'
    const packagePath = harness.packageAddon(scenario.addonId.split('.').at(-1), fixture.fixtureRoot)
    let app
    try {
      phase = 'start-tauri'
      app = await harness.startTauri(fixture)
      phase = 'enable-community'
      await harness.setCommunityEnabled(app, true)
      phase = 'install-package'
      const installed = await harness.installPackage(app, packagePath, scenario.addonId)
      phase = 'restart-after-install'
      await app.restart()
      phase = 'wait-disabled'
      await harness.waitForAddon(app, scenario.addonId, (addon) => addon?.enabled === false)
      if (!scenario.hidden) {
        phase = 'enable-addon'
        await app.command('enableAddon', scenario.addonId)
        await harness.waitForAddon(app, scenario.addonId, (addon) => addon?.enabled === true && addon.status === 'enabled')
      }
      phase = 'run-visible-scenario'
      const result = await scenario.run({ app, fixture, harness, expect, installed })
      phase = 'cleanup'
      await harness.cleanup(app, scenario.addonId, scenario.hidden)
      harness.writeEvidence(testInfo, `${scenario.addonId}-ui-usage`, {
        addonId: scenario.addonId,
        packagePath,
        vaultRoot: fixture.vaultRoot,
        result,
        tauriLog: app.output().slice(-20000)
      })
    } catch (error) {
      harness.writeEvidence(testInfo, `${scenario.addonId}-ui-failure`, {
        addonId: scenario.addonId,
        phase,
        packagePath,
        vaultRoot: fixture.vaultRoot,
        error: error?.stack || String(error),
        tauriLog: app?.output?.().slice(-20000) || ''
      })
      throw new Error(`${error?.message || String(error)}; phase=${phase}`)
    } finally {
      if (app) await app.close().catch(() => {})
      fs.rmSync(fixture.fixtureRoot, { recursive: true, force: true })
    }
  })
}
