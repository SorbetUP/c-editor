#!/usr/bin/env node
import { copyFile, mkdir, readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'

const args = process.argv.slice(2)
const value = (flag, required = true) => {
  const index = args.indexOf(flag)
  if (index < 0 || !args[index + 1]) {
    if (required) throw new Error(`Missing ${flag}`)
    return null
  }
  return path.resolve(args[index + 1])
}
const source = value('--source')
const surfaceSource = value('--surface-source', false)
const output = value('--output')
const config = JSON.parse(await readFile(value('--config'), 'utf8'))
const manifestPath = path.join(source, 'manifest.json')
const manifest = JSON.parse(await readFile(manifestPath, 'utf8'))
if (manifest.runtime !== 'freya' || manifest.status !== 'passed' || manifest.provenance?.real !== true || manifest.provenance?.synthetic !== false) {
  throw new Error(`INFRA_ERROR: Freya manifest is not real passed evidence: ${manifestPath}`)
}
if (manifest.visualSurface !== 'application-content-only') {
  throw new Error(`INFRA_ERROR: unexpected Freya visual surface ${JSON.stringify(manifest.visualSurface)}`)
}

const surfaceIds = config.checkpoints.filter((entry) => (entry.source ?? 'journey') === 'surface').map((entry) => entry.id)
let surfaceManifest = null
if (surfaceIds.length) {
  if (!surfaceSource) throw new Error('INFRA_ERROR: parity config requires --surface-source')
  const surfaceManifestPath = path.join(surfaceSource, 'surface-manifest.json')
  surfaceManifest = JSON.parse(await readFile(surfaceManifestPath, 'utf8'))
  if (surfaceManifest.runtime !== 'freya' || surfaceManifest.captureMethod !== 'TestingRunner.render_to_file') {
    throw new Error(`INFRA_ERROR: invalid Freya surface manifest: ${surfaceManifestPath}`)
  }
  const expectedSurfaceIds = JSON.stringify(surfaceIds)
  if (JSON.stringify(surfaceManifest.checkpoints) !== expectedSurfaceIds) {
    throw new Error(`INFRA_ERROR: Freya surface checkpoints ${JSON.stringify(surfaceManifest.checkpoints)} do not match ${expectedSurfaceIds}`)
  }
  for (const field of ['width', 'height', 'scaleFactor', 'deviceScaleFactor']) {
    if (surfaceManifest.viewport?.[field] !== manifest.viewport?.[field]) {
      throw new Error(`INFRA_ERROR: Freya surface viewport ${field}=${surfaceManifest.viewport?.[field]} does not match journey ${manifest.viewport?.[field]}`)
    }
  }
}

function sourcePathForFrame (frame, checkpoint) {
  if (!frame?.path || typeof frame.path !== 'string') {
    throw new Error(`INFRA_ERROR: Freya manifest frame path missing for ${checkpoint}`)
  }
  const resolved = path.resolve(source, frame.path)
  const relative = path.relative(source, resolved)
  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error(`INFRA_ERROR: Freya frame escapes evidence root for ${checkpoint}: ${frame.path}`)
  }
  return resolved
}

const staged = []
for (const checkpoint of config.checkpoints) {
  const kind = checkpoint.source ?? 'journey'
  let staticSource
  let stabilitySource
  let sourceFrame = null
  let stabilityFrame = null
  if (kind === 'journey') {
    const checkpointManifest = manifest.checkpoints?.find((entry) => entry.id === checkpoint.id)
    if (!checkpointManifest) throw new Error(`INFRA_ERROR: Freya manifest has no checkpoint ${checkpoint.id}`)
    const frames = checkpointManifest.frames
    if (!Array.isArray(frames) || !frames.length) throw new Error(`INFRA_ERROR: Freya manifest has no frames for ${checkpoint.id}`)
    const finalFrame = frames.at(-1)
    const stableFrame = frames.length > 1 ? frames.at(-2) : finalFrame
    if (finalFrame.kind !== 'after') {
      throw new Error(`INFRA_ERROR: final Freya frame for ${checkpoint.id} is ${JSON.stringify(finalFrame.kind)}, expected after`)
    }
    sourceFrame = finalFrame.path
    stabilityFrame = stableFrame.path
    staticSource = sourcePathForFrame(finalFrame, checkpoint.id)
    stabilitySource = sourcePathForFrame(stableFrame, checkpoint.id)
  } else if (kind === 'surface') {
    staticSource = path.join(surfaceSource, checkpoint.id, 'static.png')
    stabilitySource = path.join(surfaceSource, checkpoint.id, 'stability.png')
    sourceFrame = path.relative(surfaceSource, staticSource)
    stabilityFrame = path.relative(surfaceSource, stabilitySource)
  } else {
    throw new Error(`INFRA_ERROR: unknown Freya checkpoint source ${kind}`)
  }
  const targetDir = path.join(output, checkpoint.id)
  await mkdir(targetDir, { recursive: true })
  const target = path.join(targetDir, 'static.png')
  const stabilityTarget = path.join(targetDir, 'stability.png')
  await copyFile(staticSource, target)
  await copyFile(stabilitySource, stabilityTarget)
  staged.push({ checkpoint: checkpoint.id, source: kind, sourceFrame, stabilityFrame, path: path.relative(output, target) })
}
if (JSON.stringify(staged.map((entry) => entry.checkpoint)) !== JSON.stringify(config.checkpoints.map((entry) => entry.id))) {
  throw new Error('INFRA_ERROR: staged Freya checkpoints do not match parity config')
}
await writeFile(path.join(output, 'staging.json'), `${JSON.stringify({
  schemaVersion: 2,
  runtime: 'freya',
  scenarioId: manifest.scenarioId,
  viewport: manifest.viewport,
  fixture: manifest.fixture,
  provenance: manifest.provenance,
  surfaceManifest,
  staged,
}, null, 2)}\n`)
console.log(`[freya-parity] staged ${staged.length} Freya checkpoints plus stability frames`)
