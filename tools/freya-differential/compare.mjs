#!/usr/bin/env node

import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { deflateSync, inflateSync } from 'node:zlib'
import { fileURLToPath, pathToFileURL } from 'node:url'

const PNG_SIGNATURE = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10])
const DEFAULT_REPORT = 'differential-report.json'
const DEFAULT_DIFF_DIR = 'differential-diffs'
const PROJECT_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')

function usage () {
  return `Usage:
  node tools/freya-differential/compare.mjs <reference-dir> <candidate-dir> [options]

Options:
  --reference-label <name>       Label for the first directory (default: reference)
  --candidate-label <name>       Label for the second directory (default: candidate)
  --threshold <0..1>             Per-channel tolerance, normalized to 0..255 (default: 0)
  --max-different-ratio <0..1>   Allowed proportion of pixels over threshold (default: 0)
  --report <file>                JSON report path (default: ${DEFAULT_REPORT})
  --diff-dir <directory>         PNG diff directory (default: ${DEFAULT_DIFF_DIR})
  --no-diff                      Do not write diff PNGs
  --help                         Show this help

Capture layout:
  <root>/<checkpoint>/static.png
  <root>/<checkpoint>/frames/frame-000.png
  <root>/<checkpoint>/frames/frame-001.png
`
}

function fail (message) {
  throw new Error(message)
}

function parseNormalizedNumber (name, value) {
  const number = Number(value)
  if (!Number.isFinite(number) || number < 0 || number > 1) {
    fail(`${name} must be a finite number between 0 and 1, received ${value}`)
  }
  return number
}

function parseArgs (argv) {
  const positionals = []
  const options = {
    referenceLabel: 'reference',
    candidateLabel: 'candidate',
    threshold: 0,
    maxDifferentRatio: 0,
    report: DEFAULT_REPORT,
    diffDir: DEFAULT_DIFF_DIR,
    writeDiffs: true
  }

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index]
    if (argument === '--help' || argument === '-h') {
      return { help: true }
    }
    if (argument === '--no-diff') {
      options.writeDiffs = false
      continue
    }
    if (argument.startsWith('--')) {
      const name = argument.slice(2)
      const value = argv[index + 1]
      if (!value || value.startsWith('--')) fail(`Missing value for --${name}`)
      index += 1
      if (name === 'reference-label') options.referenceLabel = value
      else if (name === 'candidate-label') options.candidateLabel = value
      else if (name === 'threshold') options.threshold = parseNormalizedNumber('--threshold', value)
      else if (name === 'max-different-ratio') options.maxDifferentRatio = parseNormalizedNumber('--max-different-ratio', value)
      else if (name === 'report') options.report = value
      else if (name === 'diff-dir') options.diffDir = value
      else fail(`Unknown option --${name}`)
      continue
    }
    positionals.push(argument)
  }

  if (positionals.length !== 2) fail('Exactly two capture directories are required')
  return { ...options, referenceDir: positionals[0], candidateDir: positionals[1] }
}

async function listPngs (root) {
  const files = []
  async function visit (directory, relativeDirectory) {
    let entries
    try {
      entries = await readdir(directory, { withFileTypes: true })
    } catch (error) {
      fail(`Cannot read capture directory ${root}: ${error.message}`)
    }
    for (const entry of entries) {
      const absolute = path.join(directory, entry.name)
      const relative = path.join(relativeDirectory, entry.name)
      if (entry.isDirectory()) await visit(absolute, relative)
      else if (entry.isFile() && entry.name.toLowerCase().endsWith('.png')) files.push(relative)
    }
  }
  await visit(root, '')
  return files.sort((left, right) => left.localeCompare(right))
}

function classifyCapture (relativePath) {
  const normalized = relativePath.split(path.sep).join('/')
  const parts = normalized.split('/')
  const file = parts.at(-1)
  const stem = file.replace(/\.png$/i, '')
  const isTemporal = parts.slice(0, -1).some((part) => /^(frames?|temporal|motion)$/i.test(part)) ||
    /^(frame|snapshot|t)[-_\.]?\d+$/i.test(stem)
  return {
    kind: isTemporal ? 'temporal' : 'static',
    checkpoint: parts[0] || stem,
    frame: isTemporal ? stem : 'static',
    relativePath: normalized
  }
}

function readPngDimensions (buffer, source) {
  if (buffer.length < 24 || !buffer.subarray(0, 8).equals(PNG_SIGNATURE)) {
    fail(`${source} is not a valid PNG (signature missing)`)
  }
  const width = buffer.readUInt32BE(16)
  const height = buffer.readUInt32BE(20)
  if (width === 0 || height === 0) fail(`${source} has invalid dimensions ${width}x${height}`)
  return { width, height }
}

async function loadSharp () {
  try {
    const module = await import('sharp')
    return module.default ?? module
  } catch (error) {
    const packageRoots = [
      path.join(PROJECT_ROOT, 'node_modules', 'sharp'),
      path.join(PROJECT_ROOT, 'Elephant', 'node_modules', 'sharp'),
      path.join(PROJECT_ROOT, 'node_modules', '.pnpm', 'node_modules', 'sharp')
    ]
    try {
      const pnpmEntries = await readdir(path.join(PROJECT_ROOT, 'node_modules', '.pnpm'), { withFileTypes: true })
      packageRoots.push(...pnpmEntries
        .filter((entry) => entry.isDirectory() && entry.name.startsWith('sharp@'))
        .map((entry) => path.join(PROJECT_ROOT, 'node_modules', '.pnpm', entry.name, 'node_modules', 'sharp')))
    } catch {
      // The direct import error below remains the actionable diagnostic.
    }
    for (const packageRoot of packageRoots) {
      try {
        const module = await import(pathToFileURL(path.join(packageRoot, 'lib', 'index.js')).href)
        return module.default ?? module
      } catch {
        // Try the next already-installed package location.
      }
    }
    return { unavailable: error.message }
  }
}

function pngChunks (buffer, source) {
  if (buffer.length < 8 || !buffer.subarray(0, 8).equals(PNG_SIGNATURE)) {
    fail(`${source} is not a valid PNG (signature missing)`)
  }
  const chunks = []
  let offset = 8
  while (offset + 12 <= buffer.length) {
    const length = buffer.readUInt32BE(offset)
    const type = buffer.toString('ascii', offset + 4, offset + 8)
    const start = offset + 8
    const end = start + length
    if (end + 4 > buffer.length) fail(`${source} contains a truncated ${type} chunk`)
    chunks.push({ type, data: buffer.subarray(start, end) })
    offset = end + 4
    if (type === 'IEND') break
  }
  return chunks
}

function decodePngRgba (buffer, source) {
  const chunks = pngChunks(buffer, source)
  const header = chunks.find((chunk) => chunk.type === 'IHDR')?.data
  if (!header || header.length !== 13) fail(`${source} has no valid IHDR chunk`)
  const width = header.readUInt32BE(0)
  const height = header.readUInt32BE(4)
  const bitDepth = header[8]
  const colorType = header[9]
  const compression = header[10]
  const filterMethod = header[11]
  const interlace = header[12]
  if (bitDepth !== 8 || compression !== 0 || filterMethod !== 0 || interlace !== 0) {
    fail(`${source} uses unsupported PNG encoding (8-bit non-interlaced PNG required)`)
  }
  const channelsByColorType = { 0: 1, 2: 3, 4: 2, 6: 4 }
  const channels = channelsByColorType[colorType]
  if (!channels) fail(`${source} uses unsupported PNG color type ${colorType}`)
  const compressed = Buffer.concat(chunks.filter((chunk) => chunk.type === 'IDAT').map((chunk) => chunk.data))
  const decoded = inflateSync(compressed)
  const rowBytes = width * channels
  const expected = height * (rowBytes + 1)
  if (decoded.length !== expected) fail(`${source} has unexpected scanline length ${decoded.length}, expected ${expected}`)
  const scanlines = Buffer.alloc(height * rowBytes)
  const previous = Buffer.alloc(rowBytes)
  let sourceOffset = 0
  for (let row = 0; row < height; row += 1) {
    const filter = decoded[sourceOffset++]
    const current = scanlines.subarray(row * rowBytes, (row + 1) * rowBytes)
    for (let column = 0; column < rowBytes; column += 1) {
      const left = column >= channels ? current[column - channels] : 0
      const above = previous[column]
      const aboveLeft = column >= channels ? previous[column - channels] : 0
      const value = decoded[sourceOffset + column]
      if (filter === 0) current[column] = value
      else if (filter === 1) current[column] = (value + left) & 0xff
      else if (filter === 2) current[column] = (value + above) & 0xff
      else if (filter === 3) current[column] = (value + Math.floor((left + above) / 2)) & 0xff
      else if (filter === 4) {
        const predictor = left + above - aboveLeft
        const pa = Math.abs(predictor - left)
        const pb = Math.abs(predictor - above)
        const pc = Math.abs(predictor - aboveLeft)
        const nearest = pa <= pb && pa <= pc ? left : pb <= pc ? above : aboveLeft
        current[column] = (value + nearest) & 0xff
      } else fail(`${source} uses unsupported PNG filter ${filter}`)
    }
    current.copy(previous)
    sourceOffset += rowBytes
  }
  const rgba = Buffer.alloc(width * height * 4)
  for (let row = 0, sourceOffset = 0, targetOffset = 0; row < height; row += 1) {
    for (let column = 0; column < width; column += 1, targetOffset += 4) {
      if (colorType === 6) {
        rgba[targetOffset] = scanlines[sourceOffset++]
        rgba[targetOffset + 1] = scanlines[sourceOffset++]
        rgba[targetOffset + 2] = scanlines[sourceOffset++]
        rgba[targetOffset + 3] = scanlines[sourceOffset++]
      } else if (colorType === 2) {
        rgba[targetOffset] = scanlines[sourceOffset++]
        rgba[targetOffset + 1] = scanlines[sourceOffset++]
        rgba[targetOffset + 2] = scanlines[sourceOffset++]
        rgba[targetOffset + 3] = 255
      } else if (colorType === 4) {
        const gray = scanlines[sourceOffset++]
        rgba[targetOffset] = gray
        rgba[targetOffset + 1] = gray
        rgba[targetOffset + 2] = gray
        rgba[targetOffset + 3] = scanlines[sourceOffset++]
      } else {
        const gray = scanlines[sourceOffset++]
        rgba[targetOffset] = gray
        rgba[targetOffset + 1] = gray
        rgba[targetOffset + 2] = gray
        rgba[targetOffset + 3] = 255
      }
    }
  }
  return { data: rgba, width, height }
}

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

function encodePngRgba (rgba, width, height) {
  const scanlines = Buffer.alloc(height * (width * 4 + 1))
  for (let row = 0; row < height; row += 1) {
    const destinationOffset = row * (width * 4 + 1)
    scanlines[destinationOffset] = 0
    rgba.copy(scanlines, destinationOffset + 1, row * width * 4, (row + 1) * width * 4)
  }
  const header = Buffer.alloc(13)
  header.writeUInt32BE(width, 0)
  header.writeUInt32BE(height, 4)
  header[8] = 8
  header[9] = 6
  return Buffer.concat([
    PNG_SIGNATURE,
    pngChunk('IHDR', header),
    pngChunk('IDAT', deflateSync(scanlines)),
    pngChunk('IEND', Buffer.alloc(0))
  ])
}

async function decodeRgba (sharp, buffer, source) {
  if (sharp.unavailable) return decodePngRgba(buffer, source)
  const result = await sharp(buffer, { failOn: 'error' })
    .ensureAlpha()
    .raw({ depth: 'uchar' })
    .toBuffer({ resolveWithObject: true })
  const channels = result.info.channels
  if (channels === 4) return { data: result.data, width: result.info.width, height: result.info.height }

  const rgba = Buffer.alloc(result.info.width * result.info.height * 4)
  for (let sourceIndex = 0, targetIndex = 0; targetIndex < rgba.length; targetIndex += 4) {
    if (channels === 1) {
      rgba[targetIndex] = result.data[sourceIndex]
      rgba[targetIndex + 1] = result.data[sourceIndex]
      rgba[targetIndex + 2] = result.data[sourceIndex]
      rgba[targetIndex + 3] = 255
      sourceIndex += 1
    } else if (channels === 2) {
      rgba[targetIndex] = result.data[sourceIndex]
      rgba[targetIndex + 1] = result.data[sourceIndex]
      rgba[targetIndex + 2] = result.data[sourceIndex]
      rgba[targetIndex + 3] = result.data[sourceIndex + 1]
      sourceIndex += 2
    } else if (channels === 3) {
      rgba[targetIndex] = result.data[sourceIndex]
      rgba[targetIndex + 1] = result.data[sourceIndex + 1]
      rgba[targetIndex + 2] = result.data[sourceIndex + 2]
      rgba[targetIndex + 3] = 255
      sourceIndex += 3
    } else {
      fail(`${source} decoded with unsupported channel count ${channels}`)
    }
  }
  return { data: rgba, width: result.info.width, height: result.info.height }
}

function comparePixels (reference, candidate, threshold) {
  const thresholdChannel = Math.ceil(threshold * 255)
  let differentPixels = 0
  let totalChannelDelta = 0
  let maxChannelDelta = 0
  const diff = Buffer.alloc(reference.length)

  for (let index = 0; index < reference.length; index += 4) {
    let pixelDelta = 0
    for (let channel = 0; channel < 4; channel += 1) {
      const delta = Math.abs(reference[index + channel] - candidate[index + channel])
      pixelDelta = Math.max(pixelDelta, delta)
      maxChannelDelta = Math.max(maxChannelDelta, delta)
      totalChannelDelta += delta
    }
    if (pixelDelta > thresholdChannel) {
      differentPixels += 1
      diff[index] = 255
      diff[index + 1] = Math.max(0, 255 - pixelDelta)
      diff[index + 2] = 0
      diff[index + 3] = 255
    }
  }

  return {
    differentPixels,
    totalPixels: reference.length / 4,
    differentRatio: differentPixels / (reference.length / 4),
    maxChannelDelta,
    meanChannelDelta: totalChannelDelta / reference.length,
    thresholdChannel,
    diff
  }
}

async function writeDiff (sharp, diff, dimensions, destination) {
  await mkdir(path.dirname(destination), { recursive: true })
  if (sharp.unavailable) {
    await writeFile(destination, encodePngRgba(diff, dimensions.width, dimensions.height))
    return
  }
  await sharp(diff, { raw: { width: dimensions.width, height: dimensions.height, channels: 4 } }).png().toFile(destination)
}

function emptyGroup () {
  return { files: 0, compared: 0, passed: 0, failed: 0, missing: 0 }
}

function makeReport (args) {
  return {
    schemaVersion: 1,
    status: 'invalid-input',
    reference: { label: args.referenceLabel, directory: args.referenceDir },
    candidate: { label: args.candidateLabel, directory: args.candidateDir },
    options: {
      threshold: args.threshold,
      thresholdChannel: Math.ceil(args.threshold * 255),
      maxDifferentRatio: args.maxDifferentRatio,
      writeDiffs: args.writeDiffs,
      diffDirectory: args.diffDir
    },
    summary: {
      files: { reference: 0, candidate: 0, compared: 0, missing: 0, extra: 0 },
      static: emptyGroup(),
      temporal: emptyGroup()
    },
    comparisons: [],
    issues: []
  }
}

async function writeReport (report, destination) {
  if (destination === '-') {
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`)
    return
  }
  await mkdir(path.dirname(path.resolve(destination)), { recursive: true })
  await writeFile(destination, `${JSON.stringify(report, null, 2)}\n`, 'utf8')
}

async function compare (args) {
  const report = makeReport(args)
  let referenceFiles
  let candidateFiles
  try {
    referenceFiles = await listPngs(args.referenceDir)
    candidateFiles = await listPngs(args.candidateDir)
  } catch (error) {
    report.issues.push({ type: 'input', message: error.message })
    report.status = 'invalid-input'
    await writeReport(report, args.report)
    return 2
  }

  report.summary.files.reference = referenceFiles.length
  report.summary.files.candidate = candidateFiles.length
  if (referenceFiles.length === 0 || candidateFiles.length === 0) {
    report.issues.push({
      type: 'sequence-missing',
      message: 'Both capture directories must contain at least one PNG',
      referenceFiles: referenceFiles.length,
      candidateFiles: candidateFiles.length
    })
  }

  const referenceSet = new Set(referenceFiles)
  const candidateSet = new Set(candidateFiles)
  const missing = referenceFiles.filter((file) => !candidateSet.has(file))
  const extra = candidateFiles.filter((file) => !referenceSet.has(file))
  report.summary.files.missing = missing.length
  report.summary.files.extra = extra.length
  report.summary.files.compared = referenceFiles.length - missing.length
  if (missing.length > 0) {
    report.issues.push({ type: 'sequence-missing', side: args.candidateLabel, files: missing })
  }
  if (extra.length > 0) {
    report.issues.push({ type: 'sequence-extra', side: args.candidateLabel, files: extra })
  }

  const sharp = await loadSharp()
  report.capabilities = {
    pixelDecoder: sharp.unavailable ? 'built-in-png-8bit' : 'sharp',
    diffEncoder: sharp.unavailable ? 'built-in-png-8bit' : 'sharp'
  }

  for (const relativePath of referenceFiles.filter((file) => candidateSet.has(file))) {
    const capture = classifyCapture(relativePath)
    const group = report.summary[capture.kind]
    group.files += 1
    group.compared += 1
    const comparison = {
      checkpoint: capture.checkpoint,
      frame: capture.frame,
      kind: capture.kind,
      path: relativePath,
      referencePath: path.join(args.referenceDir, relativePath),
      candidatePath: path.join(args.candidateDir, relativePath),
      status: 'not-compared',
      dimensions: null,
      pixels: null,
      diffPath: null,
      issues: []
    }
    report.comparisons.push(comparison)

    let referenceBuffer
    let candidateBuffer
    try {
      referenceBuffer = await readFile(comparison.referencePath)
      candidateBuffer = await readFile(comparison.candidatePath)
      const referenceDimensions = readPngDimensions(referenceBuffer, comparison.referencePath)
      const candidateDimensions = readPngDimensions(candidateBuffer, comparison.candidatePath)
      comparison.dimensions = { reference: referenceDimensions, candidate: candidateDimensions }
      if (referenceDimensions.width !== candidateDimensions.width || referenceDimensions.height !== candidateDimensions.height) {
        comparison.status = 'dimension-mismatch'
        comparison.issues.push({ type: 'dimension-mismatch', reference: referenceDimensions, candidate: candidateDimensions })
        group.failed += 1
        continue
      }
      const referencePixels = await decodeRgba(sharp, referenceBuffer, comparison.referencePath)
      const candidatePixels = await decodeRgba(sharp, candidateBuffer, comparison.candidatePath)
      if (referencePixels.width !== candidatePixels.width || referencePixels.height !== candidatePixels.height) {
        comparison.status = 'dimension-mismatch'
        comparison.issues.push({
          type: 'decoded-dimension-mismatch',
          reference: { width: referencePixels.width, height: referencePixels.height },
          candidate: { width: candidatePixels.width, height: candidatePixels.height }
        })
        group.failed += 1
        continue
      }
      const pixels = comparePixels(referencePixels.data, candidatePixels.data, args.threshold)
      const withinRatio = pixels.differentRatio <= args.maxDifferentRatio
      comparison.pixels = {
        total: pixels.totalPixels,
        different: pixels.differentPixels,
        differentRatio: pixels.differentRatio,
        maxChannelDelta: pixels.maxChannelDelta,
        meanChannelDelta: pixels.meanChannelDelta,
        thresholdChannel: pixels.thresholdChannel,
        maxDifferentRatio: args.maxDifferentRatio
      }
      comparison.status = withinRatio ? 'passed' : 'pixel-mismatch'
      if (!withinRatio) comparison.issues.push({ type: 'pixel-mismatch', message: 'Different pixel ratio exceeds the configured limit' })
      if (args.writeDiffs && pixels.differentPixels > 0) {
        const diffPath = path.join(args.diffDir, `${relativePath.slice(0, -4)}.diff.png`)
        try {
          await writeDiff(sharp, pixels.diff, referenceDimensions, diffPath)
          comparison.diffPath = diffPath
        } catch (error) {
          comparison.issues.push({ type: 'diff-write-failed', message: error.message })
        }
      }
      group[pixels.differentRatio <= args.maxDifferentRatio ? 'passed' : 'failed'] += 1
    } catch (error) {
      comparison.status = 'decode-or-read-error'
      comparison.issues.push({ type: 'comparison-error', message: error.message })
      group.failed += 1
    }
  }

  report.summary.static.missing = missing.filter((file) => classifyCapture(file).kind === 'static').length
  report.summary.temporal.missing = missing.filter((file) => classifyCapture(file).kind === 'temporal').length
  report.summary.static.files += extra.filter((file) => classifyCapture(file).kind === 'static').length
  report.summary.temporal.files += extra.filter((file) => classifyCapture(file).kind === 'temporal').length
  const hasFailure = report.issues.length > 0 || report.comparisons.some((item) => item.status !== 'passed' && item.status !== 'passed-byte-identical')
  const notComparable = report.comparisons.some((item) => item.status === 'decode-or-read-error')
  report.status = notComparable ? 'not-comparable' : hasFailure ? 'mismatch' : 'ok'
  await writeReport(report, args.report)
  return notComparable ? 2 : hasFailure ? 1 : 0
}

let exitCode = 0
try {
  const args = parseArgs(process.argv.slice(2))
  if (args.help) process.stdout.write(usage())
  else exitCode = await compare(args)
} catch (error) {
  process.stderr.write(`freya-differential: ${error.message}\n\n${usage()}`)
  exitCode = 2
}
process.exitCode = exitCode
