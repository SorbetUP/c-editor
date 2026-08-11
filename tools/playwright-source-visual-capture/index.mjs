import path from 'node:path'
import { appendFile } from 'node:fs/promises'
import { bindRun } from './scenario.mjs'
import { launchSourceRenderer } from './renderer.mjs'
import { dispatchAction } from './actions.mjs'
import { snapshotState, vaultSnapshot } from './observations.mjs'
import { SourceCapture } from './timeline.mjs'
import { hashJson, writeLogs, writeManifest } from './manifest.mjs'
import { assertActionPostcondition } from './postconditions.mjs'

async function main () {
  const run = await bindRun()
  const mark = (event, details = {}) => appendFile(path.join(run.outputRoot, 'progress.log'), JSON.stringify({ at: new Date().toISOString(), event, ...details }) + '\n')
  await mark('run:bound')
  const capture = new SourceCapture(run)
  const events = [{ event: 'run:start', runtime: 'source-playwright', renderer: true, tauri: false, runId: run.runId }]
  const rendererLines = []
  const errors = []
  let launched = null
  const actions = []
  const checkpoints = []
  try {
    await mark('renderer:start')
    launched = await launchSourceRenderer(run)
    await mark('renderer:ready')
    const { app, page } = launched
    page.on('console', (message) => rendererLines.push(`[${message.type()}] ${message.text()}`))
    page.on('pageerror', (error) => errors.push({ type: 'pageerror', message: error.stack || error.message }))
    page.on('requestfailed', (request) => errors.push({ type: 'requestfailed', method: request.method(), url: request.url(), error: request.failure()?.errorText || '' }))
    for (const [index, action] of run.scenario.actions.entries()) {
      const started = Date.now()
      const before = action.id === 'launch' ? null : await snapshotState(page, run, { id: `before-${action.id}` }, [])
      events.push({ event: 'action:start', index, id: action.id, logical: action.logical })
      await mark('action:start', { index, id: action.id })
      const frames = await capture.runAction(page, action, () => dispatchAction(page, action))
      await mark('action:frames-done', { index, id: action.id, frameCount: frames.length })
      const state = await snapshotState(page, run, action, frames)
      await mark('action:state-done', { index, id: action.id })
      const effect = assertActionPostcondition(action, {
        before,
        after: state,
        frames,
        dropObserved: rendererLines.some((line) => line.includes('[icon-rail] drag:drop'))
      })
      state.actionEffect = effect
      const vault = await vaultSnapshot(run.vaultRoot)
      const checkpoint = { id: action.checkpoint, afterAction: action.id, state, stateHash: hashJson(state), vault, vaultHash: hashJson(vault), frames }
      actions.push({ index, id: action.id, logical: action.logical, status: 'passed', relativeStartMs: 0, relativeDoneMs: Date.now() - started, source: 'shared-scenario', controlPlane: 'playwright-electron' })
      checkpoints.push(checkpoint)
      events.push({ event: 'action:done', index, id: action.id, status: 'passed', durationMs: Date.now() - started, frameCount: frames.length })
    }
    events.push({ event: 'run:done', status: 'passed', actions: actions.length, frames: checkpoints.reduce((count, checkpoint) => count + checkpoint.frames.length, 0) })
    await writeManifest(run, actions, checkpoints)
    await writeLogs(run, events, rendererLines, errors)
    await app.close()
    process.stdout.write(JSON.stringify({ status: 'passed', runtime: 'source-playwright', actions: actions.length, frames: checkpoints.reduce((count, checkpoint) => count + checkpoint.frames.length, 0), manifest: path.join(run.outputRoot, 'manifest.json') }) + '\n')
  } catch (error) {
    const failedAction = run.scenario.actions[actions.length]
    const failure = { message: error.stack || error.message, action: failedAction?.id || null }
    events.push({ event: 'run:error', ...failure })
    await writeManifest(run, actions, checkpoints, 'failed', failure)
    await writeLogs(run, events, rendererLines, [...errors, { type: 'capture', ...failure }])
    await launched?.app?.close().catch(() => {})
    process.stderr.write(`[source-playwright] ${failure.message}\n`)
    process.exitCode = 1
  }
}

if (import.meta.url === `file://${process.argv[1]}`) await main()
