import fs from 'node:fs'
import path from 'node:path'

const root = process.cwd()
const rendererDir = path.join(root, 'build', 'out', 'renderer')
const reportPath = path.join(root, 'build', 'out', 'renderer-bundle-report.json')

const OPTIONAL_BUILTIN_CHUNK_PATTERNS = Object.freeze({
  ai: /(?:^|[-_.])ai(?:[-_.]|$)/i,
  calendar: /calendar/i,
  codeExecution: /code[-_.]?execution/i,
  codexConnection: /codex[-_.]?connection/i,
  googleKeepImport: /google[-_.]?keep[-_.]?import/i,
  openModels: /open[-_.]?models/i,
  recentlyEdited: /recently[-_.]?edited/i,
  sites: /(?:^|[-_.])sites?(?:[-_.]|$)/i,
  sync: /(?:^|[-_.])sync(?:[-_.]|$)/i
})

const walk = (directory) => {
  if (!fs.existsSync(directory)) return []
  const files = []
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const pathname = path.join(directory, entry.name)
    if (entry.isDirectory()) files.push(...walk(pathname))
    else if (entry.isFile()) files.push(pathname)
  }
  return files
}

const formatBytes = (value) => {
  const bytes = Number(value) || 0
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`
  return `${(bytes / (1024 * 1024)).toFixed(2)} MiB`
}

if (!fs.existsSync(rendererDir)) {
  console.error(`[bundle-report] renderer output does not exist: ${rendererDir}`)
  process.exitCode = 1
} else {
  const files = walk(rendererDir).map((pathname) => {
    const relativePath = path.relative(rendererDir, pathname).split(path.sep).join('/')
    return {
      path: relativePath,
      bytes: fs.statSync(pathname).size
    }
  })

  const optionalChunks = {}
  const optionalPaths = new Set()
  for (const [id, pattern] of Object.entries(OPTIONAL_BUILTIN_CHUNK_PATTERNS)) {
    const matches = files.filter((file) => pattern.test(path.basename(file.path)))
    optionalChunks[id] = {
      bytes: matches.reduce((sum, file) => sum + file.bytes, 0),
      files: matches
    }
    for (const file of matches) optionalPaths.add(file.path)
  }

  const totalBytes = files.reduce((sum, file) => sum + file.bytes, 0)
  const optionalBuiltinBytes = files
    .filter((file) => optionalPaths.has(file.path))
    .reduce((sum, file) => sum + file.bytes, 0)
  const coreAndSharedBytes = totalBytes - optionalBuiltinBytes

  const report = {
    generatedAt: new Date().toISOString(),
    rendererDir: path.relative(root, rendererDir),
    totalBytes,
    coreAndSharedBytes,
    optionalBuiltinBytes,
    optionalBuiltinChunks: optionalChunks,
    files: [...files].sort((left, right) => right.bytes - left.bytes)
  }

  fs.mkdirSync(path.dirname(reportPath), { recursive: true })
  fs.writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`)

  console.log(`[bundle-report] renderer total: ${formatBytes(totalBytes)}`)
  console.log(`[bundle-report] core/shared: ${formatBytes(coreAndSharedBytes)}`)
  console.log(`[bundle-report] optional built-in chunks still packaged: ${formatBytes(optionalBuiltinBytes)}`)
  for (const [id, chunk] of Object.entries(optionalChunks)) {
    if (chunk.bytes) console.log(`[bundle-report]   ${id}: ${formatBytes(chunk.bytes)}`)
  }
  console.log(`[bundle-report] JSON: ${path.relative(root, reportPath)}`)
  console.log('[bundle-report] Lazy chunks reduce startup work, not installer size. Real size reduction requires downloadable .enaddon packages outside the renderer import graph.')
}
