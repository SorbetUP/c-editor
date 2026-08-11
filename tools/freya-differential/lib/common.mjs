import { createHash } from 'node:crypto'
import { readFile, writeFile, mkdir } from 'node:fs/promises'
import path from 'node:path'

export const PNG_SIGNATURE = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10])

export function sha256 (bytes) {
  return createHash('sha256').update(bytes).digest('hex')
}

export async function hashFile (file) {
  return sha256(await readFile(file))
}

export function stableJson (value) {
  return JSON.stringify(value)
}

export function normalizeValue (value, key = '') {
  if (Array.isArray(value)) return value.map((item) => normalizeValue(item, key))
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value)
      .filter(([name]) => !/(?:runtime|request|correlation|trace)[_-]?id/i.test(name))
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([name, item]) => [name, normalizeValue(item, name)]))
  }
  if (typeof value === 'string') {
    const normalized = value.replaceAll('\\', '/').trim().replace(/\s+/g, ' ')
    return /(?:^|[-_])(x|y|width|height|top|left|right|bottom|scroll(?:Top|Height)?)[-_]*/i.test(key) &&
      /^-?\d+(?:\.\d+)?$/.test(normalized)
      ? Number(Number(normalized).toFixed(3))
      : normalized
  }
  if (typeof value === 'number' && Number.isFinite(value) &&
      /(?:^|[-_])(x|y|width|height|top|left|right|bottom|scroll(?:Top|Height)?)[-_]*/i.test(key)) {
    return Number(value.toFixed(3))
  }
  return value
}

export function safeRelativePath (value, label) {
  if (typeof value !== 'string' || !value || path.isAbsolute(value)) {
    throw new Error(`${label} must be a non-empty relative path`)
  }
  const normalized = value.split('\\').join('/')
  const resolved = path.posix.normalize(normalized)
  if (resolved === '..' || resolved.startsWith('../') || resolved.includes('/../')) {
    throw new Error(`${label} escapes its artifact root: ${value}`)
  }
  return resolved
}

export function pathWithin (root, candidate) {
  const relative = path.relative(root, candidate)
  return relative === '' || (relative !== '..' && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative))
}

export async function readJson (file) {
  return JSON.parse(await readFile(file, 'utf8'))
}

export async function writeJson (file, value) {
  await mkdir(path.dirname(file), { recursive: true })
  await writeFile(file, `${JSON.stringify(value, null, 2)}\n`, 'utf8')
}

export function pngDimensions (buffer, source) {
  if (buffer.length < 24 || !buffer.subarray(0, 8).equals(PNG_SIGNATURE)) {
    throw new Error(`${source} is not a valid PNG`)
  }
  const width = buffer.readUInt32BE(16)
  const height = buffer.readUInt32BE(20)
  if (!width || !height) throw new Error(`${source} has invalid PNG dimensions`)
  return { width, height }
}
