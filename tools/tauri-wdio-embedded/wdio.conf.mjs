import path from 'node:path'
import os from 'node:os'
import { mkdir, mkdtemp, stat, writeFile } from 'node:fs/promises'
import { browser } from '@wdio/globals'

import { loadScenario, materializeFixture } from '../../tools/freya-differential/lib/scenario.mjs'

const projectRoot = path.resolve(import.meta.dirname, '../..')
const scenarioPath = path.resolve(
  process.env.DIFFERENTIAL_SCENARIO_PATH ??
    path.join(projectRoot, 'migration/freya/differential-scenarios.json')
)
const scenario = await loadScenario(scenarioPath)
const outputRoot = path.resolve(
  process.env.DIFFERENTIAL_OUTPUT_DIR ??
    path.join(projectRoot, 'test-results/tauri-wdio-embedded', `run-${Date.now()}`)
)
const fixtureRoot = path.resolve(
  process.env.DIFFERENTIAL_FIXTURE_ROOT ??
    await mkdtemp(path.join(os.tmpdir(), 'elephant-tauri-wdio-'))
)
await mkdir(outputRoot, { recursive: true })
await mkdir(fixtureRoot, { recursive: true })
if (!process.env.DIFFERENTIAL_FIXTURE_ROOT) await materializeFixture(scenario, fixtureRoot)
const fixtureRoots = scenario.fixture?.roots ?? { vault: 'vault', config: 'config', userData: 'user-data' }
const vaultRoot = path.join(fixtureRoot, fixtureRoots.vault)
const configRoot = path.join(fixtureRoot, fixtureRoots.config)
await writeFile(
  path.join(configRoot, 'tauri-vaults.json'),
  `${JSON.stringify({
    schemaVersion: 1,
    vaults: [{
      id: 'e2e-vault',
      name: 'E2E Vault',
      path: vaultRoot,
      icon: 'vault',
      lastOpenedAt: '2026-06-22T10:00:00.000Z',
      enabled: true
    }],
    activeVaultId: 'e2e-vault'
  }, null, 2)}\n`,
  'utf8'
)
await writeFile(
  path.join(configRoot, 'preferences.json'),
  `${JSON.stringify({
    iconRailOrder: ['vault', 'sidebar-toggle', 'search']
  }, null, 2)}\n`,
  'utf8'
)
const appBinaryPath = path.resolve(
  process.env.ELEPHANT_TAURI_ACCEPTANCE_BINARY ??
    path.join(projectRoot, 'Elephant/backend/tauri/target/debug/Elephant')
)

export const config = {
  runner: 'local',
  specs: ['./tests/differential.e2e.mjs'],
  maxInstances: 1,
  logLevel: process.env.WDIO_LOG_LEVEL ?? 'info',
  services: [[
    '@wdio/tauri-service',
    {
      appBinaryPath,
      driverProvider: 'embedded',
      embeddedPort: Number(process.env.TAURI_WEBDRIVER_PORT ?? 4445),
      outputDir: outputRoot,
      captureBackendLogs: true,
      captureFrontendLogs: true,
      env: {
        ELEPHANT_ACCEPTANCE_TAURI_PORT: process.env.ELEPHANT_ACCEPTANCE_TAURI_PORT ?? '0',
        ELEPHANT_ACCEPTANCE_PROFILE_DIR: fixtureRoot,
        ELEPHANT_ACCEPTANCE_HIDE_WINDOW: '0',
        TAURI_FRONTEND_PATH: projectRoot
      }
    }
  ]],
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    timeout: 300000
  },
  before: async function () {
    // The embedded service's automatic focus recovery relies on the optional
    // Tauri core.invoke bridge. Selecting the current WebDriver window once
    // marks it as explicit and keeps the service from probing that bridge on
    // every DOM command. The app remains the real Tauri window/session.
    await browser.switchToWindow(await browser.getWindowHandle())
    const devicePixelRatio = await browser.execute(() => window.devicePixelRatio || 1)
    // macOS runners expose only 684 points in the visible work area. Keep the
    // real 840-point application viewport by allowing the native window to
    // extend below that work area; WebDriver still captures the full webview.
    await browser.setWindowPosition(0, -25)
    await browser.setWindowSize(
      Math.round(scenario.viewport.width * devicePixelRatio),
      Math.round(scenario.viewport.height * devicePixelRatio)
    )
  },
  capabilities: [{
    browserName: 'tauri',
    'tauri:options': { application: appBinaryPath }
  }]
}

export const differentialContext = { projectRoot, scenarioPath, scenario, outputRoot, fixtureRoot, appBinaryPath }

export default config
