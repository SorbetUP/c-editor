import path from 'node:path'
import os from 'node:os'
import { mkdir, mkdtemp, stat } from 'node:fs/promises'

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
  capabilities: [{
    browserName: 'tauri',
    'tauri:options': { application: appBinaryPath }
  }]
}

export const differentialContext = { projectRoot, scenarioPath, scenario, outputRoot, fixtureRoot, appBinaryPath }

export default config
