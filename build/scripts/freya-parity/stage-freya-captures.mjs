#!/usr/bin/env node
import { copyFile, mkdir, readdir, readFile, writeFile } from 'node:fs/promises'
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
if (manifest.runtime !== 'freya' || manifest.status !== 'passed' || manifest.provenance?.real !== true) {
  throw new Error(`INFRA_ERROR: Freya manifest is not real passed evidence: ${manifestPath}`)
}
const staged = []
for (const checkpoint of config.checkpoints) {
  const kind = checkpoint.source ?? 'journey'
  let staticSource
  let stabilitySource
  let sourceFrame = null
  let stabilityFrame = null
  if (kind === 'journey') {
    const framesDir = path.join(source, 'checkpoints', checkpoint.id, 'frames')
    const frames = (await readdir(framesDir)).filter((name) => name.endsWith('.png')).sort()
    if (!frames.length) throw new Error(`INFRA_ERROR: no Freya PNG frames for ${checkpoint.id}`)
    sourceFrame = frames.at(-1)
    stabilityFrame = frames.length > 1 ? frames.at(-2) : sourceFrame
    staticSource = path.join(framesDir, sourceFrame)
    stabilitySource = path.join(framesDir, stabilityFrame)
  } else if (kind === 'surface') {
    if (!surfaceSource) throw new Error(`INFRA_ERROR: ${checkpoint.id} requires --surface-source`)
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
  staged,
}, null, 2)}\n`)
console.log(`[freya-parity] staged ${staged.length} Freya checkpoints plus stability frames`)
