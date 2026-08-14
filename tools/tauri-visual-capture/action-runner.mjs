import { MissingPhysicalTargetError, executePhysicalAction } from './physical-actions.mjs'
import { readObservedState, waitForReadyObservation } from './observations.mjs'
import { snapshotFiles } from './fixture.mjs'

const addBlockedAction = (manifest, fixture, action, reason) => {
  manifest.actions.push({ index: action.index, id: action.id, logical: action.logical, status: 'blocked', hardFailure: reason, source: action.source || action.target || null })
  manifest.checkpoints.push({ id: action.checkpoint, afterAction: action.id, state: { blocked: true, reason }, vault: snapshotFiles(fixture.vaultRoot), frames: [] })
  manifest.hardFailures.push({ action: action.id, checkpoint: action.checkpoint, reason })
}

const captureTimeline = async (action, captureFrame, delay) => {
  const frames = []
  for (const [index, relativeMs] of action.frameTimes.entries()) {
    if (relativeMs) await delay(relativeMs - action.frameTimes[index - 1])
    frames.push(captureFrame(action, index, relativeMs, index === 0 ? 'before' : index === action.frameTimes.length - 1 ? 'after' : 'during'))
  }
  return frames
}

export const runSharedActions = async ({ shared, client, fixture, runtime, manifest, captureFrame, requestRoot, delay, log, proveProfile }) => {
  let launchReady = false
  let readyObserved = false
  let launchObservation = null
  let launchFailure = null
  try {
    const ready = await waitForReadyObservation(client, delay)
    readyObserved = true
    launchReady = ready.selector === '.en-library-grid'
    if (!launchReady) launchFailure = `shared fixture did not reach .en-library-grid; observed ${ready.selector}`
    launchObservation = await readObservedState(client, fixture)
    if (launchReady) manifest.profileIsolation = await proveProfile(client, fixture)
  } catch (error) {
    launchFailure = error.message
    if (manifest.profileIsolation.status === 'pending') manifest.profileIsolation = { ...manifest.profileIsolation, status: 'failed', reason: error.message }
    manifest.hardFailures.push({ action: 'launch', reason: error.message })
  }

  for (const action of shared.actions) {
    const startedAt = Date.now()
    log({ type: 'action:start', index: action.index, id: action.id, logical: action.logical })
    if (action.id === 'launch' && readyObserved) {
      const frames = await captureTimeline(action, captureFrame, delay)
      const observed = launchObservation || await readObservedState(client, fixture)
      const status = launchReady && manifest.profileIsolation.status === 'proven' ? 'passed' : 'blocked'
      if (status !== 'passed' && !manifest.hardFailures.some((failure) => failure.action === action.id)) manifest.hardFailures.push({ action: action.id, reason: launchFailure || 'launch/profile proof failed at runtime' })
      manifest.actions.push({ index: action.index, id: action.id, logical: action.logical, status, relativeStartMs: 0, relativeDoneMs: Date.now() - startedAt, source: 'shared-scenario' })
      manifest.checkpoints.push({ id: action.checkpoint, afterAction: action.id, state: observed.observed, vault: observed.fixtureVault, frames })
    } else if (action.id === 'launch') {
      addBlockedAction(manifest, fixture, action, launchFailure || 'launch did not reach the acceptance renderer')
    } else if (manifest.hardFailures.length > 0) {
      addBlockedAction(manifest, fixture, action, 'preceding shared action/profile prerequisite is blocked; no substitute action was dispatched')
    } else {
      const frames = []
      try {
        frames.push(captureFrame(action, 0, action.frameTimes[0] ?? 0, 'before'))
        const physical = await executePhysicalAction({
          action: shared.scenario.actions[action.index],
          pid: runtime.pid,
          requestDir: requestRoot,
          client,
          windowBounds: manifest.window?.bounds
        })
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
        addBlockedAction(manifest, fixture, action, reason)
      }
    }
    log({ type: 'action:done', index: action.index, id: action.id, status: manifest.actions.at(-1)?.status })
  }
}
