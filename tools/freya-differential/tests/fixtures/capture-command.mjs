#!/usr/bin/env node

import { createHash } from 'node:crypto'
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { deflateSync } from 'node:zlib'

function crc32 (buffer) {
  let crc = 0xffffffff
  for (const byte of buffer) {
    crc ^= byte
    for (let bit = 0; bit < 8; bit += 1) crc = (crc >>> 1) ^ ((crc & 1) ? 0xedb88320 : 0)
  }
  return (crc ^ 0xffffffff) >>> 0
}

function pngChunk (type, data) {
  const typeBuffer = Buffer.from(type, 'ascii')
  const chunk = Buffer.alloc(12 + data.length)
  chunk.writeUInt32BE(data.length, 0)
  typeBuffer.copy(chunk, 4)
  data.copy(chunk, 8)
  chunk.writeUInt32BE(crc32(Buffer.concat([typeBuffer, data])), 8 + data.length)
  return chunk
}

function pixelPng (red, green, blue) {
  const header = Buffer.alloc(13)
  header.writeUInt32BE(1, 0)
  header.writeUInt32BE(1, 4)
  header[8] = 8
  header[9] = 6
  const raw = Buffer.from([0, red, green, blue, 255])
  return Buffer.concat([
    Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]),
    pngChunk('IHDR', header),
    pngChunk('IDAT', deflateSync(raw)),
    pngChunk('IEND', Buffer.alloc(0))
  ])
}

const PNG_WHITE = pixelPng(255, 255, 255)
const PNG_BLACK = pixelPng(0, 0, 0)

function argument (name) {
  const index = process.argv.indexOf(name)
  return index === -1 ? 'valid' : process.argv[index + 1]
}

function sha256 (bytes) {
  return createHash('sha256').update(bytes).digest('hex')
}

async function vaultFiles (root) {
  const files = []
  async function visit (directory, relative = '') {
    for (const entry of await readdir(directory, { withFileTypes: true })) {
      const entryRelative = path.posix.join(relative, entry.name)
      const absolute = path.join(directory, entry.name)
      if (entry.isDirectory()) await visit(absolute, entryRelative)
      else if (entry.isFile()) files.push({ path: entryRelative, sha256: sha256(await readFile(absolute)) })
    }
  }
  await visit(root)
  return files.sort((left, right) => left.path.localeCompare(right.path))
}

const mutation = argument('--mutation')
const runtime = process.env.DIFFERENTIAL_RUNTIME
const output = process.env.DIFFERENTIAL_OUTPUT_DIR
const scenarioPath = process.env.DIFFERENTIAL_SCENARIO_PATH
const fixtureRoot = process.env.DIFFERENTIAL_FIXTURE_ROOT
const runId = process.env.DIFFERENTIAL_RUN_ID
const commandSha256 = process.env.DIFFERENTIAL_COMMAND_SHA256
const captureNonce = process.env.DIFFERENTIAL_CAPTURE_NONCE
const expectedArtifactRoot = process.env.DIFFERENTIAL_OUTPUT_DIR

if (!runtime || !output || !scenarioPath || !fixtureRoot || !runId || !commandSha256 || !captureNonce) {
  throw new Error('capture command environment is incomplete')
}

const scenario = JSON.parse(await readFile(scenarioPath, 'utf8'))
const fixture = {
  id: scenario.fixture.id,
  files: await vaultFiles(path.join(fixtureRoot, 'vault'))
}
if (mutation === 'vault-mismatch' && runtime === 'freya') fixture.files.push({ path: 'fake.md', sha256: 'not-a-real-fixture-hash' })
const checkpoints = []
const actions = []

for (const [index, action] of scenario.actions.entries()) {
  if (mutation === 'missing-action' && index === scenario.actions.length - 1 && runtime === 'freya') continue
  actions.push({
    index,
    id: action.id,
    logical: action.logical,
    status: 'passed',
    relativeStartMs: 0,
    relativeDoneMs: 250
  })
  const frameTimes = Array.isArray(action.frames) ? action.frames : [0]
  const frames = []
  for (let frameIndex = 0; frameIndex < frameTimes.length; frameIndex += 1) {
    const actualIndex = mutation === 'misaligned-frame' && runtime === 'tauri' && frameIndex === 1
      ? frameIndex + 1
      : frameIndex
    const relativePath = path.posix.join(action.checkpoint, 'frames', `frame-${actualIndex.toString().padStart(3, '0')}.png`)
    const absolutePath = path.join(output, relativePath)
    await mkdir(path.dirname(absolutePath), { recursive: true })
    const png = mutation === 'pixel-difference' && runtime === 'freya' ? PNG_BLACK : PNG_WHITE
    await writeFile(absolutePath, png)
    frames.push({
      index: actualIndex,
      relativeMs: frameTimes[frameIndex],
      path: relativePath,
      sha256: sha256(png),
      sourcePath: mutation === 'copied-evidence' && runtime === 'freya'
        ? path.join(output, '..', 'tauri', relativePath)
        : absolutePath
    })
  }
  checkpoints.push({
    id: action.checkpoint,
    afterAction: action.id,
    state: mutation === 'state-mismatch' && runtime === 'freya'
      ? { ...scenario.checkpoints.find((checkpoint) => checkpoint.id === action.checkpoint)?.state, orchestratorMutation: true }
      : scenario.checkpoints.find((checkpoint) => checkpoint.id === action.checkpoint)?.state ?? {},
    vault: fixture.files,
    frames
  })
}

const provenance = {
  runtime,
  driver: runtime === 'tauri' ? 'webdriver' : 'freya-testing',
  controlPlane: runtime === 'tauri' ? 'webdriver' : 'testing-runner',
  runId,
  captureId: `${runtime}:${runId}`,
  commandSha256,
  captureNonce,
  artifactRoot: expectedArtifactRoot,
  sourceEvidenceId: mutation === 'copied-evidence' ? 'shared-source-evidence' : `${runtime}:${runId}`,
  sourceArtifactRoot: mutation === 'copied-evidence' && runtime === 'freya'
    ? path.join(output, '..', 'tauri')
    : expectedArtifactRoot,
  real: mutation !== 'faked-evidence',
  synthetic: mutation === 'faked-evidence'
}

const viewport = mutation === 'viewport-mismatch' && runtime === 'tauri'
  ? { ...scenario.viewport, width: scenario.viewport.width + 1 }
  : scenario.viewport

await writeFile(path.join(output, 'manifest.json'), `${JSON.stringify({
  schemaVersion: 1,
  scenarioId: scenario.id,
  runtime,
  provenance,
  viewport,
  scaleFactor: viewport.scaleFactor,
  deviceScaleFactor: viewport.deviceScaleFactor,
  fixture,
  actions,
  checkpoints
}, null, 2)}\n`)
