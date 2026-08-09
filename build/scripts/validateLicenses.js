'use strict'

const { spawnSync } = require('child_process')
const fs = require('fs')
const path = require('path')

const rootDir = path.resolve(__dirname, '../..')
const executable = path.join(
  rootDir,
  'Elephant',
  'node_modules',
  '.bin',
  process.platform === 'win32' ? 'license-checker.cmd' : 'license-checker'
)

if (!fs.existsSync(executable)) {
  console.error(`[ERROR] Installed license-checker binary is missing: ${executable}`)
  process.exit(1)
}

const result = spawnSync(
  executable,
  ['--production', '--json', '--start', rootDir],
  {
    cwd: rootDir,
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024
  }
)

if (result.error) {
  console.error(`[ERROR] Unable to execute license-checker: ${result.error.message}`)
  process.exit(1)
}

if (result.status !== 0) {
  process.stderr.write(result.stderr || '')
  process.stdout.write(result.stdout || '')
  process.exit(result.status || 1)
}

let report
try {
  report = JSON.parse(result.stdout || '{}')
} catch (error) {
  console.error(`[ERROR] license-checker returned invalid JSON: ${error.message}`)
  process.exit(1)
}

const packages = Object.entries(report || {})
if (!packages.length) {
  console.error('[ERROR] No production dependency license metadata was returned.')
  process.exit(1)
}

const unusable = []
const groups = new Map()
for (const [packageId, metadata] of packages) {
  const rawLicense = Array.isArray(metadata?.licenses)
    ? metadata.licenses.join(' OR ')
    : String(metadata?.licenses || '').trim()
  const normalizedLicense = rawLicense.replace(/^\(|\)$/g, '').trim()
  if (!normalizedLicense || /^(UNKNOWN|UNLICENSED|UNDEFINED)$/i.test(normalizedLicense)) {
    unusable.push({ packageId, license: normalizedLicense || 'missing' })
    continue
  }
  groups.set(normalizedLicense, (groups.get(normalizedLicense) || 0) + 1)
}

if (unusable.length) {
  console.error('[ERROR] Dependencies without usable license metadata:')
  for (const item of unusable) {
    console.error(`- ${item.packageId}: ${item.license}`)
  }
  process.exit(1)
}

console.log(`[licenses] valid groups=${groups.size} packages=${packages.length}`)
for (const [license, count] of [...groups.entries()].sort(([left], [right]) => left.localeCompare(right))) {
  console.log(`${license}: ${count}`)
}
