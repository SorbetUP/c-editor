import { readFile, realpath, stat } from 'node:fs/promises'
import path from 'node:path'
import {
  hashFile,
  normalizeValue,
  pathWithin,
  pngDimensions,
  safeRelativePath,
  stableJson
} from './common.mjs'
import { expectedActions, expectedCheckpoints, fixtureDigest } from './scenario.mjs'

function issue (type, message, details = {}) {
  return { type, message, ...details }
}

function exact (left, right) {
  return stableJson(normalizeValue(left)) === stableJson(normalizeValue(right))
}

function expectedViewport (scenario) {
  return normalizeValue(scenario.viewport)
}

// These fields describe the rendering/control implementation rather than a
// user-visible contract. They remain in each runtime's raw evidence, but the
// cross-runtime state comparison only compares observable application state.
function parityState (state) {
  if (!state || typeof state !== 'object') return state
  const { accessibilityLabels, geometry, ...observable } = state
  return observable
}

function parityVault (vault) {
  if (!Array.isArray(vault)) return vault
  return vault.filter((entry) => {
    const relativePath = entry?.path ?? entry?.relativePath ?? ''
    return !relativePath.startsWith('.elephantnote/index/') &&
      !relativePath.startsWith('.elephantnote/config/')
  })
}

export async function validateManifest (manifest, context) {
  const { runtime, outputRoot, runId, commandSha256, captureNonce, scenario, fixture } = context
  const issues = []
  const expected = expectedActions(scenario)
  const expectedCheckpointList = expectedCheckpoints(scenario)
  const normalized = { runtime, viewport: null, actions: [], checkpoints: [] }
  const frames = []
  const retainedArtifactRoot = await realpath(outputRoot)

  if (!manifest || typeof manifest !== 'object') {
    return { issues: [issue('manifest', `${runtime} capture did not produce a JSON object`)], normalized, frames }
  }
  if (manifest.schemaVersion !== 1) issues.push(issue('manifest', `${runtime} manifest schemaVersion must be 1`))
  if (manifest.scenarioId !== scenario.id) issues.push(issue('scenario-mismatch', `${runtime} manifest scenario id differs`, { expected: scenario.id, actual: manifest.scenarioId }))
  const acceptedManifestRuntimes = runtime === 'tauri' ? ['tauri', 'tauri-wdio-embedded'] : [runtime]
  if (!acceptedManifestRuntimes.includes(manifest.runtime)) issues.push(issue('runtime-mismatch', `${runtime} manifest identifies another runtime`, { actual: manifest.runtime }))

  const provenance = manifest.provenance
  if (!provenance || typeof provenance !== 'object') {
    issues.push(issue('provenance', `${runtime} manifest is missing provenance`))
  } else {
    if (provenance.runId !== runId) issues.push(issue('provenance', `${runtime} runId does not match the orchestrator invocation`, { expected: runId, actual: provenance.runId }))
    if (provenance.commandSha256 !== commandSha256) issues.push(issue('provenance', `${runtime} command digest does not match the configured capture command`))
    if (provenance.captureNonce !== captureNonce) issues.push(issue('provenance', `${runtime} capture nonce does not match the orchestrator invocation`))
    const acceptedCaptureIds = runtime === 'tauri'
      ? [`tauri:${runId}`, `tauri-wdio-embedded:${runId}`]
      : [`${runtime}:${runId}`]
    if (!acceptedCaptureIds.includes(provenance.captureId)) issues.push(issue('provenance', `${runtime} captureId is not bound to its runtime and run`))
    const expectedDriver = runtime === 'tauri' ? 'webdriver' : 'freya-testing'
    if (provenance.driver !== expectedDriver) issues.push(issue('control-plane', `${runtime} capture driver must be ${expectedDriver}`, { actual: provenance.driver }))
    if (runtime === 'tauri' && !['webdriver', 'acceptance-http'].includes(provenance.controlPlane)) {
      issues.push(issue('control-plane', `Tauri evidence must identify webdriver or acceptance-http`, { actual: provenance.controlPlane }))
    }
    if (runtime === 'tauri' && scenario.adapters?.tauri?.driver === 'playwright' && provenance.controlPlane !== 'webdriver') {
      issues.push(issue('control-plane', 'This scenario requires a real Tauri WebDriver control plane for its Playwright claim', { actual: provenance.controlPlane }))
    }
    if (runtime === 'freya' && provenance.controlPlane !== 'testing-runner') {
      issues.push(issue('control-plane', `Freya evidence must identify testing-runner`, { actual: provenance.controlPlane }))
    }
    if (provenance.real !== true || provenance.synthetic === true || ['mock', 'fixture', 'synthetic'].includes(provenance.captureMode)) {
      issues.push(issue('synthetic-evidence', `${runtime} evidence is marked synthetic or non-runtime`))
    }
    if (typeof provenance.artifactRoot !== 'string' || path.resolve(provenance.artifactRoot) !== path.resolve(outputRoot)) {
      issues.push(issue('provenance', `${runtime} artifactRoot is not the output directory supplied to its command`))
    }
  }

  const viewport = manifest.viewport
  normalized.viewport = normalizeValue(viewport)
  if (!exact(viewport, scenario.viewport)) issues.push(issue('viewport-mismatch', `${runtime} viewport differs from the scenario`, { expected: expectedViewport(scenario), actual: normalized.viewport }))
  if (manifest.scaleFactor !== scenario.viewport.scaleFactor || manifest.deviceScaleFactor !== scenario.viewport.deviceScaleFactor) {
    issues.push(issue('scale-mismatch', `${runtime} scale factors differ from the scenario`))
  }

  if (!manifest.fixture || manifest.fixture.id !== fixture.id || !exact(manifest.fixture.files, fixture.files)) {
    issues.push(issue('fixture-mismatch', `${runtime} fixture bytes do not match the orchestrator-seeded fixture`, {
      expectedDigest: fixtureDigest(fixture),
      actual: manifest.fixture
    }))
  }

  const actualActions = Array.isArray(manifest.actions) ? manifest.actions : []
  const actualActionIds = actualActions.map((action) => action?.id)
  for (const [index, expectedAction] of expected.entries()) {
    const action = actualActions[index]
    if (!action) {
      issues.push(issue('action-missing', `${runtime} capture is missing action ${expectedAction.id}`, { action: expectedAction.id }))
      continue
    }
    if (action.id !== expectedAction.id) issues.push(issue('action-order', `${runtime} action index ${index} is misaligned`, { expected: expectedAction.id, actual: action.id }))
    if (action.logical !== expectedAction.logical) issues.push(issue('action-mismatch', `${runtime} action ${expectedAction.id} has a different logical description`))
    if (action.index !== index) issues.push(issue('action-order', `${runtime} action ${expectedAction.id} reports a different index`))
    if (action.status !== 'passed') issues.push(issue('action-failed', `${runtime} action ${expectedAction.id} did not pass`, { status: action.status }))
    normalized.actions.push({ index, id: action.id, logical: action.logical, status: action.status })
  }
  for (const id of actualActionIds.slice(expected.length)) issues.push(issue('action-extra', `${runtime} capture contains an unexpected action`, { action: id }))

  const expectedByCheckpoint = new Map(expected.map((action) => [action.checkpoint, action]))
  const actualCheckpoints = Array.isArray(manifest.checkpoints) ? manifest.checkpoints : []
  const actualCheckpointIds = new Set(actualCheckpoints.map((checkpoint) => checkpoint?.id))
  for (const checkpoint of expectedCheckpointList) {
    const actual = actualCheckpoints.find((candidate) => candidate?.id === checkpoint.id)
    const expectedAction = expectedByCheckpoint.get(checkpoint.id)
    if (!actual) {
      issues.push(issue('checkpoint-missing', `${runtime} capture is missing checkpoint ${checkpoint.id}`, { checkpoint: checkpoint.id }))
      continue
    }
    if (actual.afterAction !== expectedAction?.id) issues.push(issue('checkpoint-mismatch', `${runtime} checkpoint ${checkpoint.id} is attached to the wrong action`))
    if (!Object.prototype.hasOwnProperty.call(actual, 'state')) issues.push(issue('state-missing', `${runtime} checkpoint ${checkpoint.id} has no state snapshot`))
    if (!Array.isArray(actual.vault)) issues.push(issue('vault-missing', `${runtime} checkpoint ${checkpoint.id} has no vault snapshot`))
    const normalizedCheckpoint = {
      id: checkpoint.id,
      afterAction: actual.afterAction,
      state: normalizeValue(actual.state),
      vault: normalizeValue(actual.vault),
      frames: []
    }
    const expectedFrames = expectedAction?.frameTimes ?? [0]
    const actualFrames = Array.isArray(actual.frames) ? actual.frames : []
    if (actualFrames.length !== expectedFrames.length) {
      issues.push(issue('frame-sequence', `${runtime} checkpoint ${checkpoint.id} has the wrong frame count`, { expected: expectedFrames.length, actual: actualFrames.length }))
    }
    const seenIndexes = new Set()
    let previousTimestamp = -Infinity
    for (let index = 0; index < actualFrames.length; index += 1) {
      const frame = actualFrames[index]
      const expectedIndex = index
      if (!frame || frame.index !== expectedIndex || seenIndexes.has(frame.index)) {
        issues.push(issue('frame-sequence', `${runtime} checkpoint ${checkpoint.id} has a missing or misaligned frame index`, { expectedIndex, actualIndex: frame?.index }))
      }
      seenIndexes.add(frame?.index)
      if (!Number.isFinite(frame?.relativeMs) || frame.relativeMs < previousTimestamp) {
        issues.push(issue('frame-sequence', `${runtime} checkpoint ${checkpoint.id} has non-monotonic temporal timestamps`))
      }
      previousTimestamp = frame?.relativeMs ?? previousTimestamp
      let absolute = null
      try {
        const relativePath = safeRelativePath(frame.path, `${runtime} frame path`)
        absolute = path.resolve(outputRoot, relativePath)
        if (!pathWithin(outputRoot, absolute)) throw new Error('frame escapes output root')
        const bytes = await readFile(absolute)
        const actualHash = await hashFile(absolute)
        if (frame.sha256 !== actualHash) issues.push(issue('frame-hash', `${runtime} frame hash does not match its PNG bytes`, { checkpoint: checkpoint.id, index }))
        const dimensions = pngDimensions(bytes, absolute)
        const expectedDimensions = {
          width: scenario.viewport.width,
          height: scenario.viewport.height
        }
        if (dimensions.width !== expectedDimensions.width || dimensions.height !== expectedDimensions.height) {
          issues.push(issue('frame-viewport-mismatch', `${runtime} frame dimensions do not match the scenario logical viewport`, {
            checkpoint: checkpoint.id,
            index,
            expected: expectedDimensions,
            actual: dimensions
          }))
        }
        let sourcePath = frame.sourcePath
        if (typeof sourcePath !== 'string' || !path.isAbsolute(sourcePath)) throw new Error('sourcePath must be absolute')
        sourcePath = await realpath(sourcePath)
        if (!pathWithin(retainedArtifactRoot, sourcePath)) {
          issues.push(issue('source-path-escape', `${runtime} frame sourcePath is outside the retained artifact root`, {
            checkpoint: checkpoint.id,
            index,
            sourcePath,
            artifactRoot: retainedArtifactRoot
          }))
        }
        frames.push({ checkpoint: checkpoint.id, index, relativeMs: expectedFrames[index], rawRelativeMs: frame.relativeMs, path: absolute, sourcePath, sha256: actualHash, dimensions })
        normalizedCheckpoint.frames.push({ index, relativeMs: expectedFrames[index], sha256: actualHash, dimensions })
      } catch (error) {
        issues.push(issue('frame-missing', `${runtime} checkpoint ${checkpoint.id} frame ${index} is invalid: ${error.message}`))
      }
    }
    normalized.checkpoints.push(normalizedCheckpoint)
  }
  for (const id of actualCheckpointIds) {
    if (!expectedCheckpointList.some((checkpoint) => checkpoint.id === id)) issues.push(issue('checkpoint-extra', `${runtime} capture contains an unexpected checkpoint`, { checkpoint: id }))
  }
  return { issues, normalized, frames, provenance }
}

export function compareMetadata (tauri, freya, scenario) {
  const issues = []
  if (!exact(tauri.normalized.viewport, freya.normalized.viewport)) issues.push(issue('viewport-mismatch', 'Tauri and Freya viewport/geometry metadata differ'))
  if (!exact(tauri.normalized.actions, freya.normalized.actions)) issues.push(issue('action-mismatch', 'Tauri and Freya action sequences differ'))
  const observableCheckpoints = (checkpoints) => checkpoints.map(({ id, afterAction, state, vault }) => ({
    id,
    afterAction,
    state: parityState(state),
    vault: parityVault(vault)
  }))
  if (!exact(observableCheckpoints(tauri.normalized.checkpoints), observableCheckpoints(freya.normalized.checkpoints))) {
    issues.push(issue('state-mismatch', 'Tauri and Freya normalized checkpoint state or vault snapshots differ'))
  }
  const tauriFrames = tauri.normalized.checkpoints.flatMap((checkpoint) => checkpoint.frames.map((frame) => ({ checkpoint: checkpoint.id, ...frame })))
  const freyaFrames = freya.normalized.checkpoints.flatMap((checkpoint) => checkpoint.frames.map((frame) => ({ checkpoint: checkpoint.id, ...frame })))
  if (!exact(tauriFrames.map(({ sha256, dimensions, ...frame }) => frame), freyaFrames.map(({ sha256, dimensions, ...frame }) => frame))) {
    issues.push(issue('temporal-mismatch', 'Tauri and Freya frame indexes or normalized action-relative timestamps differ'))
  }
  return issues
}

export async function detectCopiedEvidence (tauri, freya) {
  const issues = []
  if (tauri.provenance?.runId === freya.provenance?.runId || tauri.provenance?.captureId === freya.provenance?.captureId) {
    issues.push(issue('provenance', 'Tauri and Freya reuse the same run or capture id'))
  }
  if (tauri.provenance?.sourceEvidenceId && tauri.provenance.sourceEvidenceId === freya.provenance.sourceEvidenceId) {
    issues.push(issue('copied-evidence', 'Tauri and Freya declare the same source evidence id'))
  }
  if (tauri.provenance?.sourceArtifactRoot && freya.provenance?.sourceArtifactRoot && path.resolve(tauri.provenance.sourceArtifactRoot) === path.resolve(freya.provenance.sourceArtifactRoot)) {
    issues.push(issue('copied-evidence', 'Tauri and Freya declare the same source artifact root'))
  }
  const freyaByKey = new Map(freya.frames.map((frame) => [`${frame.checkpoint}:${frame.index}`, frame]))
  for (const frame of tauri.frames) {
    const candidate = freyaByKey.get(`${frame.checkpoint}:${frame.index}`)
    if (!candidate) continue
    const [tauriStat, freyaStat] = await Promise.all([stat(frame.sourcePath), stat(candidate.sourcePath)])
    if (tauriStat.dev === freyaStat.dev && tauriStat.ino === freyaStat.ino) {
      issues.push(issue('copied-evidence', `Frame ${frame.checkpoint}/${frame.index} points to the same source file in both runtimes`))
      break
    }
  }
  if (tauri.provenance?.evidenceHash && tauri.provenance.evidenceHash === freya.provenance?.evidenceHash) {
    issues.push(issue('copied-evidence', 'Tauri and Freya declare the same evidence hash'))
  }
  return issues
}
