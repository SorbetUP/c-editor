import { createHash } from 'node:crypto'
import { existsSync, readFileSync, statSync } from 'node:fs'
import { isAbsolute, relative, resolve } from 'node:path'

export const CAPTURE_SCHEMA = 'elephant.tauri-visual-capture/v1'

const pngSignature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10])

export const sha256File = (filename) => createHash('sha256').update(readFileSync(filename)).digest('hex')

export const readPngDimensions = (filename) => {
  const bytes = readFileSync(filename)
  if (bytes.length < 24 || !bytes.subarray(0, 8).equals(pngSignature)) throw new Error(`not a PNG: ${filename}`)
  return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) }
}

const add = (errors, message) => errors.push(message)

export const validateCaptureManifest = (manifest, { outputRoot } = {}) => {
  const errors = []
  if (!manifest || typeof manifest !== 'object') return { ok: false, errors: ['manifest is not an object'] }
  if (manifest.schema !== CAPTURE_SCHEMA) add(errors, `schema must be ${CAPTURE_SCHEMA}`)
  if (manifest.runtime !== 'tauri-native-window') add(errors, 'runtime must be tauri-native-window')
  if (manifest.control !== 'acceptance-http-bridge') add(errors, 'control must be acceptance-http-bridge')
  if (!manifest.window || Number.isInteger(manifest.window.id) !== true || manifest.window.width < 1 || manifest.window.height < 1) {
    add(errors, 'window id and positive dimensions are required')
  }
  if (!Array.isArray(manifest.actions) || manifest.actions.length === 0) add(errors, 'at least one source-mapped action is required')
  if (!Array.isArray(manifest.frames) || manifest.frames.length === 0) add(errors, 'at least one frame is required')
  const framePaths = new Set()
  const frameIds = new Set()
  let temporalCount = 0
  const timestamps = []
  for (const frame of manifest.frames || []) {
    if (!frame?.id || frameIds.has(frame.id)) add(errors, `frame id is missing or duplicated: ${frame?.id || '<empty>'}`)
    frameIds.add(frame?.id)
    if (frame.kind === 'temporal') temporalCount += 1
    if (!frame.path || isAbsolute(frame.path) || frame.path.split('/').includes('..')) {
      add(errors, `frame path must be relative and contained: ${frame.path || '<empty>'}`)
      continue
    }
    if (framePaths.has(frame.path)) add(errors, `frame path is duplicated: ${frame.path}`)
    framePaths.add(frame.path)
    if (!outputRoot) continue
    const absolute = resolve(outputRoot, frame.path)
    if (relative(resolve(outputRoot), absolute).startsWith('..') || !existsSync(absolute)) {
      add(errors, `missing frame artifact: ${frame.path}`)
      continue
    }
    try {
      const stat = statSync(absolute)
      const actualDimensions = readPngDimensions(absolute)
      if (stat.size !== frame.bytes) add(errors, `byte count mismatch for ${frame.path}`)
      if (actualDimensions.width !== frame.width || actualDimensions.height !== frame.height) add(errors, `dimension mismatch for ${frame.path}`)
      if (sha256File(absolute) !== frame.sha256) add(errors, `sha256 mismatch for ${frame.path}`)
    } catch (error) {
      add(errors, `${frame.path}: ${error.message}`)
    }
    if (Number.isFinite(frame.capturedAtMs)) timestamps.push(frame.capturedAtMs)
  }
  if (manifest.capturePlan?.temporalMinFrames > temporalCount) add(errors, `temporal frame count ${temporalCount} is below ${manifest.capturePlan.temporalMinFrames}`)
  if (manifest.capturePlan?.requireDistinctTemporalFrames) {
    const hashes = new Set((manifest.frames || []).filter((frame) => frame.kind === 'temporal').map((frame) => frame.sha256))
    if (hashes.size < 2) add(errors, 'temporal capture contains no distinct frames')
  }
  for (let index = 1; index < timestamps.length; index += 1) {
    if (timestamps[index] < timestamps[index - 1]) add(errors, 'frame timestamps are not monotonic')
  }
  return { ok: errors.length === 0, errors }
}

export const validateSharedCaptureManifest = (manifest, { outputRoot, scenario, fixture } = {}) => {
  const errors = []
  const expectedActions = scenario?.actions || []
  const expectedCheckpoints = scenario?.checkpoints || []
  if (!manifest || typeof manifest !== 'object') return { ok: false, errors: ['manifest is not an object'] }
  if (manifest.schemaVersion !== 1) add(errors, 'shared manifest schemaVersion must be 1')
  if (manifest.runtime !== 'tauri') add(errors, 'shared manifest runtime must be tauri')
  if (manifest.scenarioId !== scenario?.id) add(errors, 'shared manifest scenario id differs from the loaded scenario')
  if (manifest.visualSurface !== 'application-content-only') add(errors, 'application-content-only surface is not proven')
  if (manifest.window?.contentBounds?.measured !== true) add(errors, 'content bounds are not measured')
  if (manifest.physicalInput?.bridgeFallback === true) add(errors, 'DOM bridge fallback is forbidden')
  if (manifest.physicalInput?.controlPlane !== 'native-cg-event') add(errors, 'physical input is not native CGEvent control')
  if (!manifest.fixture || manifest.fixture.id !== fixture?.id) add(errors, 'shared fixture identity is missing or mismatched')
  const actions = Array.isArray(manifest.actions) ? manifest.actions : []
  if (actions.length !== expectedActions.length) add(errors, `shared action count ${actions.length} does not equal ${expectedActions.length}`)
  for (const [index, expected] of expectedActions.entries()) {
    const actual = actions[index]
    if (!actual) {
      add(errors, `missing shared action ${expected.id}`)
      continue
    }
    if (actual.index !== index || actual.id !== expected.id || actual.logical !== expected.logical) add(errors, `shared action ${expected.id} is not source-aligned`)
    if (actual.status !== 'passed') add(errors, `shared action ${expected.id} is ${actual.status || 'missing'}, not passed`)
    if ((expected.event === 'move-pointer' || expected.event === 'drag' || expected.event === 'scroll' || expected.id === 'open-create-menu' || expected.id === 'close-create-menu') && (!actual.physical || actual.physical.bridgeFallback === true)) {
      add(errors, `shared temporal action ${expected.id} has no physical action evidence`)
    }
  }
  const checkpoints = Array.isArray(manifest.checkpoints) ? manifest.checkpoints : []
  if (checkpoints.length !== expectedCheckpoints.length) add(errors, `shared checkpoint count ${checkpoints.length} does not equal ${expectedCheckpoints.length}`)
  const seenPaths = new Set()
  for (const expected of expectedCheckpoints) {
    const actual = checkpoints.find((candidate) => candidate?.id === expected.id)
    const expectedAction = expectedActions.find((candidate) => candidate.checkpoint === expected.id)
    if (!actual) {
      add(errors, `missing shared checkpoint ${expected.id}`)
      continue
    }
    if (actual.afterAction !== expectedAction?.id) add(errors, `checkpoint ${expected.id} is attached to the wrong action`)
    const frames = Array.isArray(actual.frames) ? actual.frames : []
    const expectedTimes = Array.isArray(expectedAction?.frames) ? expectedAction.frames : [0]
    if (frames.length !== expectedTimes.length) add(errors, `checkpoint ${expected.id} has ${frames.length} frames; expected ${expectedTimes.length}`)
    for (const [index, frame] of frames.entries()) {
      if (frame.index !== index || frame.relativeMs !== expectedTimes[index]) add(errors, `checkpoint ${expected.id} frame ${index} is not action-relative aligned`)
      if (frame.contentOnly !== true || frame.physicalCapture !== true) add(errors, `checkpoint ${expected.id} frame ${index} lacks physical content-only proof`)
      if (!frame.path || isAbsolute(frame.path) || frame.path.split('/').includes('..')) {
        add(errors, `checkpoint ${expected.id} frame ${index} path is not contained`)
        continue
      }
      if (seenPaths.has(frame.path)) add(errors, `duplicate frame path ${frame.path}`)
      seenPaths.add(frame.path)
      if (!outputRoot) continue
      const absolute = resolve(outputRoot, frame.path)
      if (!existsSync(absolute)) {
        add(errors, `missing shared frame ${frame.path}`)
        continue
      }
      try {
        const stat = statSync(absolute)
        const dimensions = readPngDimensions(absolute)
        if (stat.size !== frame.bytes) add(errors, `byte count mismatch for ${frame.path}`)
        if (dimensions.width !== frame.width || dimensions.height !== frame.height) add(errors, `dimension mismatch for ${frame.path}`)
        if (sha256File(absolute) !== frame.sha256) add(errors, `sha256 mismatch for ${frame.path}`)
      } catch (error) {
        add(errors, `${frame.path}: ${error.message}`)
      }
    }
    if (expectedAction && expectedAction.frames?.length >= 3) {
      if (frames[0]?.kind !== 'before' || frames.at(-1)?.kind !== 'after' || frames.slice(1, -1).some((frame) => frame.kind !== 'during')) add(errors, `checkpoint ${expected.id} does not contain explicit before/during/after frame phases`)
      const requiresDistinct = ['move-to-alpha-card', 'open-create-menu', 'move-through-create-menu', 'drag-search-rail-item', 'scroll-alpha-note'].includes(expectedAction.id)
      if (requiresDistinct && new Set(frames.map((frame) => frame.sha256)).size < 2) add(errors, `checkpoint ${expected.id} has no distinct temporal frames for source motion/menu transition`)
    }
  }
  return { ok: errors.length === 0, errors }
}
