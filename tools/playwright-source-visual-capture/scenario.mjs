import { mkdir, readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { loadScenario, materializeFixture } from '../freya-differential/lib/scenario.mjs'

const required = (name) => {
  const value = process.env[name]
  if (!value) throw new Error(`source-playwright capture requires ${name}`)
  return path.resolve(value)
}

const parseViewport = (value, scenario) => {
  const parsed = JSON.parse(value)
  if (JSON.stringify(parsed) !== JSON.stringify(scenario.viewport)) {
    throw new Error('DIFFERENTIAL_EXPECTED_VIEWPORT_JSON differs from the shared scenario')
  }
  return parsed
}

export async function bindRun () {
  const scenarioPath = required('DIFFERENTIAL_SCENARIO_PATH')
  const outputRoot = required('DIFFERENTIAL_OUTPUT_DIR')
  const fixtureRoot = required('DIFFERENTIAL_FIXTURE_ROOT')
  await ensureOutputRoot(outputRoot)
  const scenario = await loadScenario(scenarioPath)
  const viewport = parseViewport(process.env.DIFFERENTIAL_EXPECTED_VIEWPORT_JSON, scenario)
  const fixture = await materializeFixture(scenario, fixtureRoot)
  const roots = scenario.fixture.roots
  const vaultRoot = path.join(fixtureRoot, roots.vault)
  const configRoot = path.join(fixtureRoot, roots.config)
  const userDataRoot = path.join(fixtureRoot, roots.userData)
  await bindElectronConfig(configRoot, vaultRoot)
  return {
    scenario,
    scenarioPath,
    outputRoot,
    fixtureRoot,
    vaultRoot,
    configRoot,
    userDataRoot,
    fixture,
    viewport,
    runId: process.env.DIFFERENTIAL_RUN_ID,
    commandSha256: process.env.DIFFERENTIAL_COMMAND_SHA256,
    captureNonce: process.env.DIFFERENTIAL_CAPTURE_NONCE,
    orchestratorRuntime: process.env.DIFFERENTIAL_RUNTIME || null
  }
}

export async function ensureOutputRoot (outputRoot) {
  await mkdir(outputRoot, { recursive: true })
}

async function bindElectronConfig (configRoot, vaultRoot) {
  const configPath = path.join(configRoot, 'elephantnote.json')
  let current = {}
  try {
    current = JSON.parse(await readFile(configPath, 'utf8'))
  } catch (error) {
    if (error.code !== 'ENOENT') throw new Error(`read source fixture config: ${error.message}`)
  }
  const vaults = Array.isArray(current.vaults) ? current.vaults : []
  const configured = vaults.find((vault) => vault?.id === 'e2e-vault') || {
    id: 'e2e-vault', name: 'E2E Vault', icon: 'vault'
  }
  await writeFile(configPath, JSON.stringify({
    ...current,
    vaults: [{ ...configured, name: 'E2E Vault', path: vaultRoot }],
    activeVaultId: 'e2e-vault'
  }, null, 2) + '\n')
}
