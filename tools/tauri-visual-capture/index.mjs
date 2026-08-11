#!/usr/bin/env node

import { spawn } from 'node:child_process'
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs'
import path from 'node:path'
import { tmpdir } from 'node:os'

import { createAcceptanceClient, stopProcessTree, waitForAcceptanceEndpoint } from './acceptance-client.mjs'
import { readPngDimensions, sha256File, validateSharedCaptureManifest } from './manifest.mjs'
import {
  captureNativeWindow,
  listNativeWindows,
  measureNativeWindowChrome,
  readProcessTable,
  selectNativeWindow,
  selectTauriChildProcess,
  setWindowGeometry
} from './native-window.mjs'
import { executePhysicalAction, MissingPhysicalTargetError } from './physical-actions.mjs'
import { probePlaywrightTauriAttach } from './playwright-probe.mjs'
import { DEFAULT_SHARED_SCENARIO, loadSharedScenario, materializeSharedFixture } from './scenario.mjs'

const root = path.resolve(import.meta.dirname, '../..')
const defaultOutput = path.join(root, 'test-results', 'tauri-visual-capture', new Date().toISOString().replaceAll(/[:.]/g, '-'))
const cliValue = (names) => {
  for (const name of names) {
    const index = process.argv.indexOf(name)
    if (index !== -1 && process.argv[index + 1]) return process.argv[index + 1]
  }
  return null
}
const outputRoot = path.resolve(process.env.DIFFERENTIAL_OUTPUT_DIR || process.env.ELEPHANT_TAURI_CAPTURE_OUTPUT || cliValue(['--output']) || defaultOutput)
const scenarioPath = path.resolve(process.env.DIFFERENTIAL_SCENARIO_PATH || process.env.ELEPHANT_TAURI_CAPTURE_SCENARIO || cliValue(['--shared-scenario', '--scenario']) || DEFAULT_SHARED_SCENARIO)
const frameRoot = path.join(outputRoot, 'frames')
const requestRoot = path.join(outputRoot, 'physical-requests')
const delay = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds))
const writeJson = (filename, value) => writeFileSync(filename, `${JSON.stringify(value, null, 2)}\n`, 'utf8')

mkdirSync(frameRoot, { recursive: true })
mkdirSync(requestRoot, { recursive: true })
const runLog = []
let processOutput = ''
const log = (entry) => runLog.push({ at: new Date().toISOString(), ...entry })
const writeLogs = () => {
  writeJson(path.join(outputRoot, 'run-log.json'), runLog)
  writeFileSync(path.join(outputRoot, 'tauri-process.log'), processOutput, 'utf8')
}

const snapshotFiles = (directory, relative = '') => {
  const files = []
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const entryRelative = path.posix.join(relative, entry.name)
    const absolute = path.join(directory, entry.name)
    if (entry.isDirectory()) files.push(...snapshotFiles(absolute, entryRelative))
    else if (entry.isFile()) files.push({ path: entryRelative, sha256: sha256File(absolute) })
  }
  return files.sort((left, right) => left.path.localeCompare(right.path))
}

const createFixture = async (scenario) => {
  const suppliedRoot = process.env.DIFFERENTIAL_FIXTURE_ROOT
  const fixtureRoot = suppliedRoot ? path.resolve(suppliedRoot) : mkdtempSync(path.join(tmpdir(), 'elephant-tauri-visual-capture-'))
  const roots = scenario.fixture.roots
  for (const relativeRoot of Object.values(roots)) mkdirSync(path.join(fixtureRoot, relativeRoot), { recursive: true })
  const fixture = await materializeSharedFixture(scenario, fixtureRoot)
  const configRoot = path.join(fixtureRoot, roots.config)
  const userDataRoot = path.join(fixtureRoot, roots.userData)
  const vaultRoot = path.join(fixtureRoot, roots.vault)
  // This is an explicit fixture for the app-specific profile contract. The
  // current product does not consume these variables; that is recorded as a
  // hard runtime limitation instead of changing HOME or claiming isolation.
  writeJson(path.join(configRoot, 'tauri-vaults.json'), {
    schemaVersion: 1,
    vaults: [{ id: 'e2e-vault', name: 'E2E Vault', path: vaultRoot, icon: 'vault', enabled: true }],
    activeVaultId: 'e2e-vault'
  })
  return {
    fixtureRoot,
    vaultRoot,
    configRoot,
    userDataRoot,
    fixture,
    supplied: Boolean(suppliedRoot),
    cleanup: !suppliedRoot
  }
}

const launchTauri = (fixture, appPath) => {
  const environment = {
    ...process.env,
    // HOME is intentionally inherited unchanged. Only app-specific variables
    // are supplied for the explicit fixture/profile contract.
    ELEPHANTNOTE_CONFIG_DIR: fixture.configRoot,
    ELEPHANTNOTE_USER_DATA_DIR: fixture.userDataRoot,
    ELEPHANTNOTE_PROFILE_DIR: fixture.fixtureRoot,
    ELEPHANTNOTE_FIXTURE_ROOT: fixture.fixtureRoot,
    ELEPHANT_ACCEPTANCE_TAURI_PORT: '0',
    ELEPHANT_ACCEPTANCE_HIDE_WINDOW: '0',
    TAURI_FRONTEND_PATH: root
  }
  if (environment.HOME !== process.env.HOME) throw new Error('launch attempted to repurpose HOME')
  log({ type: 'launch:start', appPath, launcherControl: 'real-tauri-child', homeChanged: false, appSpecificEnv: ['ELEPHANTNOTE_CONFIG_DIR', 'ELEPHANTNOTE_USER_DATA_DIR', 'ELEPHANTNOTE_PROFILE_DIR', 'ELEPHANTNOTE_FIXTURE_ROOT'] })
  return spawn(appPath, [], { cwd: root, env: environment, detached: true, stdio: ['ignore', 'pipe', 'pipe'] })
}

const waitForTauriWindow = async (child, appPath) => {
  const deadline = Date.now() + 120000
  let last = 'no actual Tauri child/window candidate'
  while (Date.now() < deadline) {
    try {
      const table = readProcessTable()
      const process = selectTauriChildProcess({ launcherPid: child.pid, appPath, table })
      const windows = listNativeWindows([process.pid])
      const window = selectNativeWindow({ windows, pid: process.pid, names: ['Elephant'] })
      log({ type: 'window:selected', launcherPid: child.pid, tauriPid: process.pid, executable: process.comm, args: process.args, windowId: window.windowId, ownerPid: window.ownerPid, ownerName: window.ownerName, title: window.title, bounds: window.bounds })
      return { process, window }
    } catch (error) {
      last = error.message
    }
    await delay(500)
  }
  throw new Error(`timed out waiting for a visible native Tauri child window: ${last}`)
}

const refreshWindow = (runtime) => {
  const table = readProcessTable()
  const process = selectTauriChildProcess({ launcherPid: runtime.launcherPid, appPath: runtime.appPath, table })
  if (process.pid !== runtime.pid) throw new Error(`Tauri child changed from ${runtime.pid} to ${process.pid}`)
  const windows = listNativeWindows([runtime.pid])
  runtime.window = selectNativeWindow({ windows, pid: runtime.pid, names: ['Elephant'] })
  runtime.process = process
  return runtime.window
}

const measureContent = (runtime, accessibility) => {
  const windowBounds = runtime.window.bounds
  const webAreas = accessibility.elements.filter((element) => element.role === 'AXWebArea' && element.rect && element.rect.width > 100 && element.rect.height > 100)
  const nativeControls = accessibility.elements.filter((element) => ['AXCloseButton', 'AXMinimizeButton', 'AXFullScreenButton', 'AXToolbarButton'].includes(element.subrole) && element.rect && element.rect.width > 0 && element.rect.height > 0)
  if (webAreas.length === 1) {
    const contentBounds = webAreas[0].rect
    const chrome = accessibility.elements.filter((element) => ['AXTitleBar', 'AXToolbar'].includes(element.role) && element.rect && element.rect.width > 0 && element.rect.height > 0)
    const overlapsChrome = chrome.some((element) => {
      const bottom = element.rect.y + element.rect.height
      return element.rect.y <= windowBounds.y + 2 && bottom > contentBounds.y + 1
    })
    if (overlapsChrome) throw new Error(`native chrome overlaps the measured AXWebArea; content crop cannot be proven: ${JSON.stringify(chrome)}`)
    return { contentBounds, chrome, nativeWindowDecorated: chrome.length > 0, nativeChromeExcluded: true, measured: true, measuredBy: 'macOS Accessibility AXWebArea' }
  }
  if (nativeControls.length > 0) {
    const chromeBottom = Math.max(...nativeControls.map((element) => element.rect.y + element.rect.height))
    const contentBounds = { x: windowBounds.x, y: chromeBottom, width: windowBounds.width, height: Math.max(0, windowBounds.height - (chromeBottom - windowBounds.y)) }
    if (contentBounds.height < 1) throw new Error(`measured native controls consume the whole native window: ${JSON.stringify(nativeControls)}`)
    return { contentBounds, chrome: nativeControls.map((element) => ({ subrole: element.subrole, rect: element.rect })), nativeWindowDecorated: true, nativeChromeExcluded: true, measured: true, measuredBy: 'macOS Accessibility native window controls' }
  }
  const hierarchy = measureNativeWindowChrome({ processName: runtime.window.ownerName })
  if (hierarchy.windowBounds.width !== windowBounds.width || hierarchy.windowBounds.height !== windowBounds.height) throw new Error(`System Events and CGWindow bounds disagree: ${JSON.stringify({ cgWindow: windowBounds, systemEvents: hierarchy.windowBounds })}`)
  if (!hierarchy.titleBar) {
    return { contentBounds: windowBounds, chrome: [], nativeWindowDecorated: false, nativeChromeExcluded: true, measured: true, measuredBy: hierarchy.measuredBy }
  }
  const titleBarBottom = hierarchy.titleBar.y + hierarchy.titleBar.height
  const contentBounds = { x: windowBounds.x, y: titleBarBottom, width: windowBounds.width, height: Math.max(0, windowBounds.height - (titleBarBottom - windowBounds.y)) }
  if (contentBounds.height < 1) throw new Error(`measured title bar consumes the whole native window: ${JSON.stringify(hierarchy)}`)
  return { contentBounds, chrome: [hierarchy.titleBar], nativeWindowDecorated: true, nativeChromeExcluded: true, measured: true, measuredBy: hierarchy.measuredBy }
}

const waitForReadyObservation = async (client) => {
  const selectors = ['.en-library-grid', '.en-empty-card', '.en-no-vault', '.en-shell']
  const deadline = Date.now() + 20000
  let last = null
  while (Date.now() < deadline) {
    for (const selector of selectors) {
      last = await client.command('readDom', selector)
      if (last.exists && last.visible) return { selector, observation: last }
    }
    await delay(100)
  }
  throw new Error(`Tauri ready observation timed out: ${JSON.stringify(last)}`)
}

const readObservedState = async (client, fixture) => {
  const state = await client.command('readState')
  const selectors = ['.en-shell', '.en-library-grid', '.en-search-overlay', '.en-note-editor-shell', '.en-create-menu-popover']
  const dom = {}
  for (const selector of selectors) dom[selector] = await client.command('readDom', selector)
  return {
    observed: { state, dom },
    fixtureVault: snapshotFiles(fixture.vaultRoot)
  }
}

const frameFile = (action, index, relativeMs) => path.join(action.checkpoint, 'frames', `${String(action.index).padStart(2, '0')}-${action.id}-${String(index).padStart(2, '0')}-${relativeMs}ms.png`)

const main = async () => {
  if (process.platform !== 'darwin') throw new Error(`Task E requires macOS native capture, got ${process.platform}`)
  const shared = await loadSharedScenario(scenarioPath)
  if (shared.actions.length !== 14) throw new Error(`shared scenario changed unexpectedly: expected 14 actions, got ${shared.actions.length}`)
  const probe = await probePlaywrightTauriAttach()
  writeJson(path.join(outputRoot, 'playwright-probe.json'), probe)
  log({ type: 'playwright:probe', status: probe.status, detail: probe.detail, version: probe.playwrightVersion })
  if (probe.status !== 'unsupported') throw new Error(`Playwright attach probe did not prove the expected limitation: ${probe.status}`)

  const fixture = await createFixture(shared.scenario)
  const appPath = path.resolve(process.env.ELEPHANT_TAURI_APP_PATH || path.join(root, 'build', 'scripts', 'build_dev.sh'))
  const manifest = {
    schemaVersion: 1,
    scenarioId: shared.scenario.id,
    runtime: 'tauri',
    status: 'running',
    viewport: shared.scenario.viewport,
    scaleFactor: shared.scenario.viewport.scaleFactor,
    deviceScaleFactor: shared.scenario.viewport.deviceScaleFactor,
    visualSurface: 'application-content-only',
    sourceBaseline: '2c27f01ba659cf846a980d7cc758252450ea6849',
    fixture: fixture.fixture,
    provenance: {
      driver: 'native-cg-event',
      controlPlane: 'native-cg-event',
      runId: process.env.DIFFERENTIAL_RUN_ID || `standalone-${process.pid}`,
      captureId: `tauri:${process.env.DIFFERENTIAL_RUN_ID || `standalone-${process.pid}`}`,
      commandSha256: process.env.DIFFERENTIAL_COMMAND_SHA256 || null,
      captureNonce: process.env.DIFFERENTIAL_CAPTURE_NONCE || null,
      real: true,
      synthetic: false,
      captureMode: 'real-tauri-native-window',
      artifactRoot: outputRoot,
      sourceEvidenceId: `tauri:${process.pid}:${Date.now()}`
    },
    physicalInput: { status: 'not-attempted-profile-blocker', proofStatus: 'NOT PROVEN', controlPlane: 'native-cg-event', bridgeFallback: false, events: [], targetResolution: 'macOS Accessibility AXUIElement' },
    comparison: { ready: false, reason: 'Strict pixel comparison requires a real Tauri WebDriver/Playwright page and normalized content pixels; current run is native-window evidence only.' },
    profileIsolation: { status: 'not-proven', homeChanged: false, requestedEnv: ['ELEPHANTNOTE_CONFIG_DIR', 'ELEPHANTNOTE_USER_DATA_DIR', 'ELEPHANTNOTE_PROFILE_DIR', 'ELEPHANTNOTE_FIXTURE_ROOT'], reason: 'Current Tauri product source resolves app_config_dir/app_data_dir without consuming an app-specific override.' },
    playwright: probe,
    window: null,
    actions: [],
    checkpoints: [],
    frames: [],
    hardFailures: [],
    logs: { run: 'run-log.json', process: 'tauri-process.log', physicalRequests: 'physical-requests/' }
  }
  let child
  let runtime
  try {
    child = launchTauri(fixture, appPath)
    let endpoint = null
    const acceptance = await waitForAcceptanceEndpoint(child, { onOutput: (stream, text) => { processOutput += `[${stream}] ${text}`; log({ type: 'process:output', stream, length: text.length }) } })
    endpoint = acceptance.endpoint
    const client = createAcceptanceClient(endpoint, { log })
    const health = await client.health()
    if (health.transport !== 'tauri') throw new Error(`acceptance transport was not Tauri: ${JSON.stringify(health)}`)
    const native = await waitForTauriWindow(child, appPath)
    runtime = { launcherPid: child.pid, pid: native.process.pid, process: native.process, window: native.window, appPath }
    const geometry = shared.scenario.viewport
    const geometryResult = setWindowGeometry({ processName: runtime.window.ownerName, x: 80, y: 60, width: geometry.width, height: geometry.height })
    if (!geometryResult.ok) throw new Error(`cannot set fixed native window geometry: ${geometryResult.stderr || geometryResult.stdout}`)
    await delay(250)
    refreshWindow(runtime)
    if (runtime.window.bounds.width !== geometry.width || runtime.window.bounds.height !== geometry.height) throw new Error(`fixed native geometry was not observed: ${JSON.stringify(runtime.window.bounds)}`)

    const accessibility = await import('./native-window.mjs').then(({ inspectAccessibility }) => inspectAccessibility({ pid: runtime.pid }))
    writeJson(path.join(outputRoot, 'accessibility-tree.json'), accessibility)
    manifest.window = { id: runtime.window.windowId, ownerPid: runtime.pid, launcherPid: runtime.launcherPid, ownerName: runtime.window.ownerName, title: runtime.window.title, bounds: runtime.window.bounds, contentBounds: null, fixedGeometryVerified: true, processProof: { launcherPid: runtime.launcherPid, tauriPid: runtime.pid, executable: runtime.process.comm, args: runtime.process.args, descendant: true, selection: runtime.process.selection } }
    const content = measureContent(runtime, accessibility)
    manifest.window.contentBounds = content

    const captureFrame = (action, index, relativeMs, kind) => {
      refreshWindow(runtime)
      const relativePath = frameFile(action, index, relativeMs)
      const filename = path.join(outputRoot, relativePath)
      mkdirSync(path.dirname(filename), { recursive: true })
      const captured = captureNativeWindow({ windowId: runtime.window.windowId, filename, outputWidth: runtime.window.bounds.width, outputHeight: runtime.window.bounds.height, contentBounds: content.contentBounds, windowBounds: runtime.window.bounds })
      const dimensions = readPngDimensions(filename)
      const frame = { index, relativeMs, kind, path: relativePath, sourcePath: filename, bytes: statSync(filename).size, width: dimensions.width, height: dimensions.height, sha256: sha256File(filename), contentOnly: true, physicalCapture: true, captureMethod: captured.method, contentCrop: captured.content?.crop || null, captureBounds: captured.content?.captureBounds || null, captureFallbackError: captured.fallbackError || null, capturedAtMs: Date.now() }
      manifest.frames.push({ ...frame, actionId: action.id, checkpoint: action.checkpoint })
      log({ type: 'frame:capture', action: action.id, index, relativeMs, kind, width: frame.width, height: frame.height, method: frame.captureMethod, fallbackError: frame.captureFallbackError })
      return frame
    }

    const addBlockedAction = (action, reason) => {
      manifest.actions.push({ index: action.index, id: action.id, logical: action.logical, status: 'blocked', hardFailure: reason, source: action.source || action.target || null })
      manifest.checkpoints.push({ id: action.checkpoint, afterAction: action.id, state: { blocked: true, reason }, vault: snapshotFiles(fixture.vaultRoot), frames: [] })
      manifest.hardFailures.push({ action: action.id, checkpoint: action.checkpoint, reason })
    }

    let launchReady = false
    let readyObserved = false
    try {
      const ready = await waitForReadyObservation(client)
      readyObserved = true
      launchReady = ready.selector === '.en-library-grid'
      if (!launchReady) manifest.hardFailures.push({ action: 'launch', reason: `shared fixture did not reach .en-library-grid; observed ${ready.selector}` })
    } catch (error) {
      manifest.hardFailures.push({ action: 'launch', reason: error.message })
    }

    for (const action of shared.actions) {
      const startedAt = Date.now()
      log({ type: 'action:start', index: action.index, id: action.id, logical: action.logical })
      if (action.id === 'launch' && readyObserved) {
        const frames = []
        for (const [index, relativeMs] of action.frameTimes.entries()) { if (relativeMs) await delay(relativeMs - action.frameTimes[index - 1]); frames.push(captureFrame(action, index, relativeMs, index === 0 ? 'before' : index === action.frameTimes.length - 1 ? 'after' : 'during')) }
        const observed = await readObservedState(client, fixture)
        const status = launchReady && manifest.profileIsolation.status === 'proven' ? 'passed' : 'blocked'
        if (status !== 'passed') manifest.hardFailures.push({ action: action.id, reason: 'shared fixture/profile isolation is not proven by the production Tauri path' })
        manifest.actions.push({ index: action.index, id: action.id, logical: action.logical, status, relativeStartMs: 0, relativeDoneMs: Date.now() - startedAt, source: 'shared-scenario' })
        manifest.checkpoints.push({ id: action.checkpoint, afterAction: action.id, state: observed.observed, vault: observed.fixtureVault, frames })
      } else if (action.id === 'launch') {
        addBlockedAction(action, 'deterministic shared fixture is not active through a production-supported app-specific profile override')
      } else if (manifest.hardFailures.length > 0) {
        addBlockedAction(action, 'preceding shared action/profile prerequisite is blocked; no substitute action was dispatched')
      } else {
        const frames = []
        try {
          frames.push(captureFrame(action, 0, action.frameTimes[0] ?? 0, 'before'))
          const physical = executePhysicalAction({ action: shared.scenario.actions[action.index], pid: runtime.pid, requestDir: requestRoot })
          manifest.physicalInput.status = 'dispatched'
          manifest.physicalInput.events.push(...physical.events)
          for (let index = 1; index < action.frameTimes.length; index += 1) {
            const relativeMs = action.frameTimes[index]
            await delay(Math.max(0, relativeMs - (action.frameTimes[index - 1] || 0)))
            frames.push(captureFrame(action, index, relativeMs, index === action.frameTimes.length - 1 ? 'after' : 'during'))
          }
          await delay(shared.scenario.capture.frameTimeline.settleAfterActionMs)
          const observed = await readObservedState(client, fixture)
          manifest.actions.push({ index: action.index, id: action.id, logical: action.logical, status: 'passed', relativeStartMs: 0, relativeDoneMs: Date.now() - startedAt, source: 'shared-scenario', physical })
          manifest.checkpoints.push({ id: action.checkpoint, afterAction: action.id, state: observed.observed, vault: observed.fixtureVault, frames })
        } catch (error) {
          const reason = error instanceof MissingPhysicalTargetError ? error.message : error.stack || String(error)
          manifest.physicalInput.status = 'NOT PROVEN'
          manifest.physicalInput.failure = reason
          manifest.hardFailures.push({ action: action.id, reason })
          addBlockedAction(action, reason)
        }
      }
      log({ type: 'action:done', index: action.index, id: action.id, status: manifest.actions.at(-1)?.status })
    }
    manifest.status = manifest.hardFailures.length === 0 ? 'passed' : 'failed'
    manifest.validation = validateSharedCaptureManifest(manifest, { outputRoot, scenario: shared.scenario, fixture: fixture.fixture })
    writeJson(path.join(outputRoot, 'manifest.json'), manifest)
    if (!manifest.validation.ok) throw new Error(`strict shared capture validation failed: ${manifest.validation.errors.join('; ')}`)
    return manifest
  } catch (error) {
    manifest.status = 'failed'
    manifest.error = error.stack || String(error)
    manifest.validation = validateSharedCaptureManifest(manifest, { outputRoot, scenario: shared.scenario, fixture: fixture.fixture })
    writeJson(path.join(outputRoot, 'manifest.json'), manifest)
    log({ type: 'run:failed', error: manifest.error })
    throw error
  } finally {
    await stopProcessTree(child)
    writeLogs()
    if (fixture?.cleanup) rmSync(fixture.fixtureRoot, { recursive: true, force: true })
  }
}

try {
  const result = await main()
  process.stdout.write(`${JSON.stringify({ status: result.status, outputRoot, actions: result.actions.length, frames: result.frames.length }, null, 2)}\n`)
} catch (error) {
  process.stderr.write(`[tauri-visual-capture] ${error.stack || error}\n`)
  process.exitCode = 1
}
