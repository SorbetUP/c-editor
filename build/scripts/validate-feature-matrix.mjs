import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import vm from 'node:vm'
import { spawnSync } from 'node:child_process'
import { createRequire } from 'node:module'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const require = createRequire(import.meta.url)
const readText = (relativePath) => fs.readFileSync(path.join(root, relativePath), 'utf8')
const readJson = (relativePath) => JSON.parse(readText(relativePath))
const fail = (message) => {
  console.error(`[feature-matrix] FAIL ${message}`)
  process.exitCode = 1
}
const pass = (message, detail = undefined) => {
  console.log(`[feature-matrix] PASS ${message}${detail === undefined ? '' : ` ${JSON.stringify(detail)}`}`)
}

const matrix = readJson('tests/app/usage/feature-matrix.json')
const linuxCatalog = readJson('tests/app/usage/linux/scenarios.json')
const addonCatalog = readJson('addons/catalog.json')
const linuxSuite = readText('tests/app/e2e/linux-usage-regressions.spec.js')
const officialAddonSuite = readText('tests/app/e2e/official-addons-regressions.spec.js')
const packageJson = readJson('package.json')
const workflow = readText('.github/workflows/e2e.yml')
const observableRunner = readText('build/scripts/run-observable.mjs')

for (const relativePath of [
  'build/scripts/run-observable.mjs',
  'build/scripts/validate-feature-matrix.mjs',
  'tests/app/e2e/observable-preload-patch.js',
  'tests/app/e2e/official-addon-preload-patch.js',
  'tests/app/e2e/official-addons-regressions.spec.js',
  'tests/app/e2e/tauri-preload-entry.js',
  'tests/app/e2e/playwright.config.js'
]) {
  const result = spawnSync(process.execPath, ['--check', path.join(root, relativePath)], {
    cwd: root,
    encoding: 'utf8'
  })
  if (result.status !== 0) {
    fail(`${relativePath} does not parse: ${String(result.stderr || result.stdout || '').trim()}`)
  } else {
    pass('JavaScript syntax', relativePath)
  }
}

try {
  const applyObservablePatch = require(path.join(root, 'tests/app/e2e/observable-preload-patch.js'))
  const applyOfficialAddonPatch = require(path.join(root, 'tests/app/e2e/official-addon-preload-patch.js'))
  const originalPreload = readText('tests/app/e2e/tauri-preload.js')
  const observablePreload = applyObservablePatch(originalPreload)
  const generatedPreload = applyOfficialAddonPatch(observablePreload)
  new vm.Script(generatedPreload, { filename: 'generated-observable-official-addon-tauri-preload.js' })
  const helperBoundary = generatedPreload.indexOf('const primitivePreference =')
  const fixtureBoundary = generatedPreload.indexOf('const officialAddonFixture =')
  const implementationBoundary = generatedPreload.indexOf('const invokeImplementation = async')
  const observableBoundary = generatedPreload.indexOf('const invoke = async')
  if (!(helperBoundary >= 0 && fixtureBoundary > helperBoundary && implementationBoundary > fixtureBoundary && observableBoundary > implementationBoundary)) {
    fail('preload patches are not composed after initialized helpers and before the observable invoke wrapper')
  } else {
    pass('generated observable official addon preload parses in runtime dependency order', {
      bytes: Buffer.byteLength(generatedPreload),
      helperBoundary,
      fixtureBoundary,
      implementationBoundary,
      observableBoundary
    })
  }
  for (const marker of [
    '[e2e-tauri:invoke:start]',
    '[e2e-tauri:invoke:done]',
    '[e2e-tauri:invoke:error]',
    'Unhandled E2E Tauri invoke',
    'tauri_addons_read_module',
    'tauri_addons_install'
  ]) {
    if (!generatedPreload.includes(marker)) fail(`generated preload omits ${marker}`)
  }
} catch (error) {
  fail(`generated observable official addon preload is invalid: ${error?.stack || error?.message || String(error)}`)
}

if (matrix.schemaVersion !== 1) fail(`unsupported schemaVersion ${matrix.schemaVersion}`)
else pass('feature matrix schema version', matrix.schemaVersion)

const requiredKinds = new Set(matrix.policy?.requiredScenarioKinds || [])
const linuxScenarioIds = new Set((linuxCatalog.scenarios || []).map((scenario) => scenario.id))
const officialScenarioIds = new Set((matrix.officialAddonScenarios || []).map((scenario) => scenario.id))
const knownScenarioIds = new Set([...linuxScenarioIds, ...officialScenarioIds])
const minimum = Number(matrix.policy?.minimumScenariosPerFeature || 0)

for (const feature of matrix.coreFeatures || []) {
  const scenarios = Array.isArray(feature.scenarios) ? feature.scenarios : []
  if (scenarios.length < minimum) {
    fail(`${feature.id} has ${scenarios.length} scenarios; minimum is ${minimum}`)
    continue
  }
  const kinds = new Set(scenarios.map((scenario) => scenario.kind))
  for (const kind of requiredKinds) {
    if (!kinds.has(kind)) fail(`${feature.id} does not cover scenario kind ${kind}`)
  }
  for (const scenario of scenarios) {
    if (!knownScenarioIds.has(scenario.id)) fail(`${feature.id} references unknown scenario ${scenario.id}`)
  }
  pass('core feature coverage', {
    id: feature.id,
    scenarios: scenarios.map((scenario) => scenario.id),
    kinds: [...kinds]
  })
}

for (const id of linuxScenarioIds) {
  if (!linuxSuite.includes(`defineUsageTest('${id}'`)) fail(`Linux scenario ${id} is not connected to defineUsageTest`)
}
pass('Linux usage catalog is connected to executable tests', { count: linuxScenarioIds.size })

const matrixAddonIds = (matrix.officialAddons || []).map((addon) => addon.id).sort()
const catalogAddonIds = (addonCatalog.addons || []).map((addon) => addon.id).sort()
if (JSON.stringify(matrixAddonIds) !== JSON.stringify(catalogAddonIds)) {
  fail(`official addon matrix mismatch: matrix=${JSON.stringify(matrixAddonIds)} catalog=${JSON.stringify(catalogAddonIds)}`)
} else {
  pass('all official catalogue addons are represented', { count: catalogAddonIds.length })
}

for (const scenario of matrix.officialAddonScenarios || []) {
  if (!officialAddonSuite.includes(scenario.id)) fail(`official addon scenario ${scenario.id} has no executable marker`)
}
for (const addonId of matrixAddonIds) {
  if (!officialAddonSuite.includes('for (const addon of catalog.addons)')) {
    fail('official addon suite is not data-driven from catalog.addons')
    break
  }
  if (!addonId.startsWith('elephant.')) fail(`invalid official addon id ${addonId}`)
}
for (const requiredOperation of [
  'installFromPath(`official:${id}`)',
  'enableWithDependencies',
  'probeAddonFunctionality',
  'page.reload()',
  'uninstallWithDependents'
]) {
  if (!officialAddonSuite.includes(requiredOperation)) fail(`official addon suite omits ${requiredOperation}`)
}
pass('official addon lifecycle suite is catalogue-driven', {
  addons: matrixAddonIds.length,
  scenariosPerAddon: (matrix.officialAddonScenarios || []).length
})

const serviceAddons = new Set((matrix.officialAddons || [])
  .filter((addon) => addon.runtime === 'service')
  .map((addon) => addon.id))
for (const addonId of serviceAddons) {
  if (!officialAddonSuite.includes('assertNativePackageEvidence')) {
    fail(`service addon ${addonId} has no native package evidence assertion`)
    break
  }
}
pass('service-backed addons require native package evidence', { count: serviceAddons.size })

const observableScripts = ['tauri:dev', 'test', 'test:unit', 'test:e2e', 'test:official-addons:e2e']
for (const scriptName of observableScripts) {
  const script = packageJson.scripts?.[scriptName] || ''
  if (!script.includes('run-observable.mjs')) fail(`${scriptName} bypasses the observable command runner`)
  else pass('observable script', { scriptName, command: script })
}

for (const marker of ['run-start', 'child-spawned', 'output', 'run-finish', 'signal-received']) {
  if (!observableRunner.includes(`'${marker}'`)) fail(`observable runner does not emit ${marker}`)
}
if (!observableRunner.includes('[REDACTED_SECRET_VALUE]')) fail('observable runner does not redact credential values')
pass('observable runner emits structured lifecycle and output events')

for (const workflowMarker of [
  'pnpm test:feature-matrix',
  'pnpm test:e2e',
  'test-results/observability/**',
  'official-addon-evidence/**',
  'build/out/addons/releases/**'
]) {
  if (!workflow.includes(workflowMarker)) fail(`E2E workflow does not retain ${workflowMarker}`)
}
pass('E2E workflow keeps feature, addon and observability evidence')

if (process.exitCode) {
  console.error('[feature-matrix] Validation failed')
  process.exit(process.exitCode)
}

console.log(`[feature-matrix] COMPLETE core=${matrix.coreFeatures.length} addons=${matrixAddonIds.length} linuxScenarios=${linuxScenarioIds.size}`)
