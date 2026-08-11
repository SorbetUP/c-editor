import { mkdir, utimes, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { readJson, safeRelativePath, stableJson, sha256 } from './common.mjs'

export async function loadScenario (file) {
  const scenario = await readJson(file)
  const issues = validateScenario(scenario)
  if (issues.length) throw new Error(issues.map((issue) => issue.message).join('\n'))
  return scenario
}

export function validateScenario (scenario) {
  const issues = []
  if (!scenario || typeof scenario !== 'object') return [{ type: 'scenario', message: 'Scenario must be an object' }]
  if (!scenario.id || !scenario.viewport || !Array.isArray(scenario.actions) || !Array.isArray(scenario.checkpoints)) {
    issues.push({ type: 'scenario', message: 'Scenario needs id, viewport, actions and checkpoints' })
    return issues
  }
  const actionIds = new Set()
  for (const [index, action] of scenario.actions.entries()) {
    if (!action.id || actionIds.has(action.id)) issues.push({ type: 'action-schema', action: action.id, message: 'Action ids must be unique and non-empty' })
    actionIds.add(action.id)
    if (!action.checkpoint) issues.push({ type: 'action-schema', action: action.id, message: 'Action must name its checkpoint' })
    const freya = action.target?.freya
    if (freya?.sourceLabel && freya.label !== freya.sourceLabel) {
      issues.push({
        type: 'source-label-mismatch',
        action: action.id,
        message: `Freya mapping for ${action.id} does not match its proven source label`
      })
    }
    const hasProvenMapping = Boolean(freya && (
      freya.provenMapping ||
      (freya.strategy && freya.label && freya.source && freya.test)
    ))
    if (action.requiredForParity && freya?.status === 'not-exposed' && hasProvenMapping) {
      issues.push({
        type: 'stale-mapping',
        action: action.id,
        message: `Required action ${action.id} still marks a proven Freya mapping as not-exposed`
      })
    }
    if (Array.isArray(action.frames) && action.frames.some((frame) => !Number.isFinite(frame) || frame < 0)) {
      issues.push({ type: 'timeline-schema', action: action.id, message: 'Frame timestamps must be finite non-negative numbers' })
    }
    if (index !== scenario.actions.findIndex((candidate) => candidate.id === action.id)) {
      issues.push({ type: 'action-schema', action: action.id, message: 'Action order is not deterministic' })
    }
  }
  const checkpointIds = new Set()
  for (const checkpoint of scenario.checkpoints) {
    if (!checkpoint.id || checkpointIds.has(checkpoint.id)) issues.push({ type: 'checkpoint-schema', checkpoint: checkpoint.id, message: 'Checkpoint ids must be unique and non-empty' })
    checkpointIds.add(checkpoint.id)
    if (!actionIds.has(checkpoint.afterAction)) issues.push({ type: 'checkpoint-schema', checkpoint: checkpoint.id, message: `Checkpoint ${checkpoint.id} points at an unknown action` })
  }
  return issues
}

function fixtureRoots (scenario) {
  return scenario.fixture?.roots ?? { vault: 'vault', config: 'config', userData: 'user-data' }
}

function fixtureContent (value) {
  if (typeof value === 'string') return value
  if (!value || typeof value !== 'object') return `${JSON.stringify(value, null, 2)}\n`
  if (typeof value.prefix === 'string' && value.generatedLines) {
    const from = Number(value.generatedLines.from ?? 0)
    const to = Number(value.generatedLines.to ?? from)
    const template = String(value.generatedLines.template ?? '')
    const generated = []
    for (let index = from; index <= to; index += 1) generated.push(template.replaceAll('{index}', String(index)))
    return `${value.prefix}${generated.join('\n')}${value.suffix ?? ''}`
  }
  return `${JSON.stringify(value, null, 2)}\n`
}

function lookupJsonRef (scenario, reference) {
  const parts = String(reference).split('.')
  let current = scenario
  for (const part of parts) current = current?.[part]
  if (current === undefined) throw new Error(`Unknown fixture jsonRef ${reference}`)
  return current
}

export async function materializeFixture (scenario, root) {
  const roots = fixtureRoots(scenario)
  for (const relativeRoot of Object.values(roots)) await mkdir(path.join(root, relativeRoot), { recursive: true })
  const files = []
  const mtime = scenario.fixture?.fileDefaults?.mtime
    ? new Date(scenario.fixture.fileDefaults.mtime)
    : null
  for (const file of scenario.fixture?.files ?? []) {
    const fixtureRoot = roots[file.root]
    if (!fixtureRoot) throw new Error(`Unknown fixture root ${file.root}`)
    const relative = safeRelativePath(file.path, `fixture file ${file.path}`)
    const contentValue = file.jsonRef ? lookupJsonRef(scenario, file.jsonRef) : file.content ?? file.json
    const bytes = Buffer.from(fixtureContent(contentValue), 'utf8')
    const destination = path.join(root, fixtureRoot, relative)
    await mkdir(path.dirname(destination), { recursive: true })
    await writeFile(destination, bytes)
    if (mtime && !Number.isNaN(mtime.getTime())) await utimes(destination, mtime, mtime)
    if (file.root === 'vault') files.push({ path: relative, sha256: sha256(bytes) })
  }
  files.sort((left, right) => left.path.localeCompare(right.path))
  return { id: scenario.fixture?.id, roots, files }
}

export function expectedActions (scenario) {
  return scenario.actions.map((action, index) => ({
    index,
    id: action.id,
    logical: action.logical,
    checkpoint: action.checkpoint,
    frameTimes: Array.isArray(action.frames) ? action.frames : [0]
  }))
}

export function expectedCheckpoints (scenario) {
  return scenario.checkpoints.map((checkpoint) => ({ ...checkpoint }))
}

export function fixtureDigest (fixture) {
  return sha256(Buffer.from(stableJson(fixture.files), 'utf8'))
}
