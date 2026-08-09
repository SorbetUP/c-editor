import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { basename, dirname, join, resolve } from 'node:path'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const __filename = fileURLToPath(import.meta.url)
const repoRoot = resolve(dirname(__filename), '../..')
const mobilePlatforms = new Set([
  'android-aarch64',
  'android-x86_64',
  'ios-aarch64',
  'ios-x86_64'
])

const fail = (message) => {
  console.error(`[mobile-addon] ${message}`)
  process.exit(1)
}

const run = (command, args) => {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    stdio: ['ignore', 'pipe', 'inherit'],
    encoding: 'utf8'
  })
  if (result.error) fail(result.error.message)
  if (result.status !== 0) fail(`${command} exited with ${result.status}`)
  return String(result.stdout || '').trim()
}

const addonArg = process.argv[2]
const platform = String(process.argv[3] || '').trim()
if (!addonArg || !mobilePlatforms.has(platform)) {
  fail('usage: node build/scripts/build-mobile-addon-package.mjs <addon-directory> <android-aarch64|android-x86_64|ios-aarch64|ios-x86_64>')
}

const addonDir = resolve(repoRoot, addonArg)
const manifestPath = join(addonDir, 'manifest.json')
const mobileEntry = 'main.mobile.js'
if (!existsSync(manifestPath)) fail(`missing ${manifestPath}`)
if (!existsSync(join(addonDir, mobileEntry))) fail(`missing mobile runtime ${mobileEntry}`)
const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'))
if (manifest.id !== 'elephant.code-execution') {
  fail('only the Code execution mobile package is currently supported')
}

const mobileManifest = structuredClone(manifest)
if (mobileManifest.permissions) delete mobileManifest.permissions.native
delete mobileManifest.native
mobileManifest.runtime.entry = mobileEntry
mobileManifest.description = 'Runs JavaScript code blocks in an isolated Web Worker on Android and iOS.'
mobileManifest.mobileRuntime = { kind: 'web-worker', platform }

const safeId = String(manifest.id).replace(/[^a-z0-9._-]/gi, '-')
const stagingDir = resolve(repoRoot, 'build/out/addons/mobile-staging', `${safeId}-${platform}`)
const releaseDir = resolve(repoRoot, 'build/out/addons/releases', safeId)
rmSync(stagingDir, { recursive: true, force: true })
mkdirSync(stagingDir, { recursive: true })
mkdirSync(releaseDir, { recursive: true })
writeFileSync(join(stagingDir, 'manifest.json'), `${JSON.stringify(mobileManifest, null, 2)}\n`)
cpSync(join(addonDir, mobileEntry), join(stagingDir, mobileEntry))
if (existsSync(join(addonDir, 'assets'))) {
  cpSync(join(addonDir, 'assets'), join(stagingDir, 'assets'), { recursive: true })
}

const outputName = `${safeId}-${manifest.version}-${platform}.enaddon`
const outputPath = join(releaseDir, outputName)
const stdout = run('cargo', [
  'run',
  '--quiet',
  '--manifest-path',
  resolve(repoRoot, 'build/tools/enaddon-packager/Cargo.toml'),
  '--',
  stagingDir,
  outputPath
])
const metadata = JSON.parse(stdout.split(/\r?\n/).filter(Boolean).at(-1))
writeFileSync(`${outputPath}.json`, `${JSON.stringify({
  ...metadata,
  id: manifest.id,
  version: manifest.version,
  platform,
  file: outputName,
  catalogPath: `addons/${basename(addonDir)}/releases/${outputName}`,
  runtime: 'web-worker'
}, null, 2)}\n`)

console.log(`[mobile-addon] package=${outputPath}`)
console.log(`[mobile-addon] blake3=${metadata.blake3}`)
