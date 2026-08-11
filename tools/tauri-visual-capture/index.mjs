#!/usr/bin/env node

import { spawn } from 'node:child_process'
import { mkdirSync, rmSync, writeFileSync } from 'node:fs'
import path from 'node:path'

import { createAcceptanceClient, stopProcessTree, waitForAcceptanceEndpoint } from './acceptance-client.mjs'
import { runSharedActions } from './action-runner.mjs'
import { createFixture, proveProfileIsolation } from './fixture.mjs'
import { delay, createRunLogger, outputRoot, prepareOutput, requestRoot, reserveLocalPort, root, scenarioPath, writeJson } from './capture-config.mjs'
import { validateSharedCaptureManifest } from './manifest.mjs'
import { createFrameCapture, measureContent, refreshWindow, waitForTauriWindow } from './native-capture.mjs'
import { probePlaywrightTauriAttach } from './playwright-probe.mjs'
import { loadSharedScenario } from './scenario.mjs'
import { listNativeWindows, setWindowGeometry } from './native-window.mjs'

const { log, appendProcessOutput, write: writeLogs } = createRunLogger()

const launchTauri = (fixture, appPath, acceptancePort) => {
  const environment = {
    ...process.env,
    // HOME is intentionally inherited unchanged. The product hook is enabled
    // only by the existing acceptance server contract plus this profile root.
    ELEPHANT_ACCEPTANCE_PROFILE_DIR: fixture.fixtureRoot,
    ELEPHANT_ACCEPTANCE_TAURI_PORT: String(acceptancePort),
    ELEPHANT_ACCEPTANCE_HIDE_WINDOW: '0',
    TAURI_FRONTEND_PATH: root
  }
  if (environment.HOME !== process.env.HOME) throw new Error('launch attempted to repurpose HOME')
  log({ type: 'launch:start', appPath, launcherControl: 'real-tauri-child', homeChanged: false, acceptancePort, appSpecificEnv: ['ELEPHANT_ACCEPTANCE_PROFILE_DIR', 'ELEPHANT_ACCEPTANCE_TAURI_PORT'], acceptanceContract: 'ELEPHANT_ACCEPTANCE_TAURI_PORT + ELEPHANT_ACCEPTANCE_PROFILE_DIR' })
  return spawn(appPath, [], { cwd: root, env: environment, detached: true, stdio: ['ignore', 'pipe', 'pipe'] })
}

const createManifest = (shared, fixture, probe) => ({
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
  physicalInput: { status: 'not-attempted', proofStatus: 'NOT PROVEN', controlPlane: 'native-cg-event', bridgeFallback: false, events: [], targetResolution: 'macOS Accessibility AXUIElement' },
  comparison: { ready: false, reason: 'Strict pixel comparison requires a real Tauri WebDriver/Playwright page and normalized content pixels; current run is native-window evidence only.' },
  profileIsolation: { status: 'pending', homeChanged: false, requestedEnv: ['ELEPHANT_ACCEPTANCE_TAURI_PORT', 'ELEPHANT_ACCEPTANCE_PROFILE_DIR'], acceptanceContract: 'existing Tauri acceptance server plus explicit profile root', observed: null },
  playwright: probe,
  window: null,
  actions: [],
  checkpoints: [],
  frames: [],
  hardFailures: [],
  logs: { run: 'run-log.json', process: 'tauri-process.log', physicalRequests: 'physical-requests/' }
})

const main = async () => {
  if (process.platform !== 'darwin') throw new Error(`Task E requires macOS native capture, got ${process.platform}`)
  prepareOutput()
  const shared = await loadSharedScenario(scenarioPath)
  if (shared.actions.length !== 14) throw new Error(`shared scenario changed unexpectedly: expected 14 actions, got ${shared.actions.length}`)
  const probe = await probePlaywrightTauriAttach()
  writeJson(path.join(outputRoot, 'playwright-probe.json'), probe)
  log({ type: 'playwright:probe', status: probe.status, detail: probe.detail, version: probe.playwrightVersion })
  if (probe.status !== 'unsupported') throw new Error(`Playwright attach probe did not prove the expected limitation: ${probe.status}`)

  const fixture = await createFixture(shared.scenario, { writeJson })
  const appPath = path.resolve(process.env.ELEPHANT_TAURI_APP_PATH || path.join(root, 'build', 'scripts', 'build_dev.sh'))
  const manifest = createManifest(shared, fixture, probe)
  let child
  let runtime
  try {
    const acceptancePort = await reserveLocalPort()
    child = launchTauri(fixture, appPath, acceptancePort)
    const acceptance = await waitForAcceptanceEndpoint(child, { expectedPort: acceptancePort, onOutput: (stream, text) => { appendProcessOutput(stream, text); log({ type: 'process:output', stream, length: text.length }) } })
    const client = createAcceptanceClient(acceptance.endpoint, { log })
    const health = await client.health()
    if (health.transport !== 'tauri') throw new Error(`acceptance transport was not Tauri: ${JSON.stringify(health)}`)
    const native = await waitForTauriWindow(child, appPath, delay, log)
    runtime = { launcherPid: child.pid, pid: native.process.pid, process: native.process, window: native.window, appPath }
    const geometry = shared.scenario.viewport
    const geometryResult = setWindowGeometry({ processName: runtime.window.ownerName, x: 80, y: 60, width: geometry.width, height: geometry.height })
    if (!geometryResult.ok) throw new Error(`cannot set fixed native window geometry: ${geometryResult.stderr || geometryResult.stdout}`)
    await delay(250)
    refreshWindow(runtime)
    if (runtime.window.bounds.width !== geometry.width || runtime.window.bounds.height !== geometry.height) throw new Error(`fixed native geometry was not observed: ${JSON.stringify(runtime.window.bounds)}`)
    const accessibility = await import('./native-window.mjs').then(({ inspectAccessibility }) => inspectAccessibility({ pid: runtime.pid }))
    writeJson(path.join(outputRoot, 'accessibility-tree.json'), accessibility)
    manifest.window = { id: runtime.window.windowId, ownerPid: runtime.pid, launcherPid: runtime.launcherPid, ownerName: runtime.window.ownerName, title: runtime.window.title, bounds: runtime.window.bounds, contentBounds: null, fixedGeometryVerified: true, processProof: { launcherPid: runtime.launcherPid, tauriPid: runtime.pid, executable: runtime.process.comm, args: runtime.process.args, descendant: runtime.process.descendant, selection: runtime.process.selection } }
    const content = measureContent(runtime, accessibility)
    manifest.window.contentBounds = content
    const captureFrame = createFrameCapture({ runtime, content, outputRoot, manifest, log })
    await runSharedActions({ shared, client, fixture, runtime, manifest, captureFrame, requestRoot, delay, log, proveProfile: proveProfileIsolation })
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
