import { createHash } from 'node:crypto'
import { mkdir, writeFile } from 'node:fs/promises'
import path from 'node:path'

export function buildManifest (run, actions, checkpoints, status = 'passed', failure = null, supplemental = null) {
  const evidenceId = `source-playwright:${run.runId}:${process.pid}`
  return {
    schemaVersion: 1,
    scenarioId: run.scenario.id,
    runtime: 'source-playwright',
    status,
    viewport: run.viewport,
    scaleFactor: run.viewport.scaleFactor,
    deviceScaleFactor: run.viewport.deviceScaleFactor,
    visualSurface: 'application-content-only',
    provenance: {
      runtime: 'source-playwright',
      driver: 'playwright-electron',
      controlPlane: 'playwright-electron',
      orchestratorRuntime: run.orchestratorRuntime,
      runId: run.runId,
      captureId: `source-playwright:${run.runId}`,
      commandSha256: run.commandSha256,
      captureNonce: run.captureNonce,
      real: true,
      renderer: true,
      tauri: false,
      synthetic: false,
      captureMode: 'real-electron-renderer',
      artifactRoot: path.resolve(run.outputRoot),
      sourceEvidenceId: evidenceId
    },
    fixture: run.fixture,
    actions,
    checkpoints,
    frames: checkpoints.flatMap((checkpoint) => checkpoint.frames.map((frame) => ({ actionId: checkpoint.afterAction, checkpoint: checkpoint.id, frame }))),
    supplemental,
    logs: { run: 'run-log.json', renderer: 'renderer.log', errors: 'errors.log' },
    capture: { render: 'Playwright page.screenshot', pointerInput: 'Playwright mouse/keyboard', dragDrop: 'Playwright page.mouse path + locator.drop(DataTransfer)', intervalMs: 50, settleAfterActionMs: 250 },
    validation: { allActionsExecuted: status === 'passed', allTargetsPlaywrightResolved: status === 'passed', postconditionsPassed: status === 'passed', stateAndVaultHashesCaptured: checkpoints.length > 0 },
    failure
  }
}

export async function writeManifest (run, actions, checkpoints, status = 'passed', failure = null, supplemental = null) {
  await mkdir(run.outputRoot, { recursive: true })
  const manifest = buildManifest(run, actions, checkpoints, status, failure, supplemental)
  await writeFile(path.join(run.outputRoot, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n')
  return manifest
}

export async function writeLogs (run, events, rendererLines, errors) {
  await mkdir(run.outputRoot, { recursive: true })
  await writeFile(path.join(run.outputRoot, 'run-log.json'), JSON.stringify(events, null, 2) + '\n')
  await writeFile(path.join(run.outputRoot, 'renderer.log'), rendererLines.join('\n') + (rendererLines.length ? '\n' : ''))
  await writeFile(path.join(run.outputRoot, 'errors.log'), JSON.stringify(errors, null, 2) + '\n')
}

export function hashJson (value) {
  return createHash('sha256').update(JSON.stringify(value)).digest('hex')
}
