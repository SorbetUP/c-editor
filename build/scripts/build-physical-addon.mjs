import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { basename, dirname, extname, isAbsolute, join, normalize, relative, resolve } from 'node:path'
import { spawnSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const __filename = fileURLToPath(import.meta.url)
const repoRoot = resolve(dirname(__filename), '../..')

const fail = (message) => {
  console.error(`[physical-addon] ${message}`)
  process.exit(1)
}

const run = (command, args, options = {}) => {
  console.log(`[physical-addon] ${command} ${args.join(' ')}`)
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    stdio: options.capture ? ['ignore', 'pipe', 'inherit'] : 'inherit',
    encoding: 'utf8',
    env: { ...process.env, ...(options.env || {}) }
  })
  if (result.error) fail(result.error.message)
  if (result.status !== 0) fail(`${command} exited with ${result.status}`)
  return options.capture ? String(result.stdout || '').trim() : ''
}

const DESKTOP_NATIVE_PLATFORMS = new Set([
  'macos-aarch64',
  'macos-x86_64',
  'linux-aarch64',
  'linux-x86_64',
  'windows-aarch64',
  'windows-x86_64'
])
const NATIVE_RUNNERS = new Set(['process', 'service'])

const platformKey = () => {
  const os = process.platform === 'darwin'
    ? 'macos'
    : process.platform === 'win32'
      ? 'windows'
      : process.platform
  const arch = process.arch === 'arm64'
    ? 'aarch64'
    : process.arch === 'x64'
      ? 'x86_64'
      : process.arch
  return `${os}-${arch}`
}

const safeModulePath = (value, label = 'module path') => {
  const input = String(value || '').replaceAll('\\', '/')
  if (!input || isAbsolute(input)) fail(`${label} must be a relative package path`)
  const normalized = normalize(input).replaceAll('\\', '/')
  if (normalized === '..' || normalized.startsWith('../') || normalized.split('/').includes('..')) {
    fail(`${label} escapes the addon package: ${value}`)
  }
  return normalized.replace(/^\.\//, '')
}

const staticImportSpecifiers = (source) => {
  const matches = []
  const pattern = /(?:^|[;\n])\s*(?:import|export)\s+(?:[^'";]+?\s+from\s+)?['"]([^'"]+)['"]/gm
  for (const match of String(source || '').matchAll(pattern)) matches.push(match[1])
  return matches
}

const addonArg = process.argv[2]
if (!addonArg) fail('usage: node build/scripts/build-physical-addon.mjs <addon-directory>')

const addonDir = resolve(repoRoot, addonArg)
const addonRelativeDir = relative(repoRoot, addonDir).replaceAll('\\', '/')
if (!addonRelativeDir.startsWith('addons/official/') || addonRelativeDir.split('/').includes('..')) {
  fail(`physical addon sources must live under addons/official: ${addonArg}`)
}
const manifestPath = join(addonDir, 'manifest.json')
const buildConfigPath = join(addonDir, 'addon.build.json')
if (!existsSync(manifestPath)) fail(`missing ${manifestPath}`)
if (!existsSync(buildConfigPath)) fail(`missing ${buildConfigPath}`)

const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'))
const buildConfig = JSON.parse(readFileSync(buildConfigPath, 'utf8'))
const platform = String(process.env.ELEPHANT_ADDON_PLATFORM || platformKey()).trim()
const runner = String(buildConfig.runner || '').trim()
if (!NATIVE_RUNNERS.has(runner)) fail('addon.build.json runner must be process or service')
if (manifest?.native?.runner !== runner) fail(`manifest native.runner must match addon.build.json runner (${runner})`)
const expectedProtocol = runner === 'service' ? 'elephant-addon-service-v1' : 'elephant-addon-sidecar-v1'
if (manifest?.native?.protocol !== expectedProtocol) {
  fail(`${runner} packages must use native protocol ${expectedProtocol}`)
}
if (!DESKTOP_NATIVE_PLATFORMS.has(platform)) {
  fail(`${runner} native packages cannot be built for ${platform}; Android and iOS require package-owned mobile host adapters`)
}
if (!Array.isArray(buildConfig.supportedPlatforms) || !buildConfig.supportedPlatforms.includes(platform)) {
  fail(`addon.build.json does not support platform ${platform}`)
}
const sidecarRelativePath = manifest?.native?.sidecars?.[platform]
if (!sidecarRelativePath) fail(`manifest has no native executable path for ${platform}`)

const nativeManifest = resolve(addonDir, buildConfig.nativeManifest)
const binaryName = String(buildConfig.binaryName || '').trim()
if (!binaryName) fail('addon.build.json requires binaryName')
if (!existsSync(nativeManifest)) fail(`missing native manifest ${nativeManifest}`)

const safeId = String(manifest.id || basename(addonDir)).replace(/[^a-z0-9._-]/gi, '-')
const targetDir = resolve(repoRoot, 'build/out/addon-target', safeId)
const stagingDir = resolve(repoRoot, 'build/out/addons/staging', safeId)
const releaseDir = resolve(repoRoot, 'build/out/addons/releases', safeId)
const targetTriple = String(process.env.CARGO_BUILD_TARGET || '').trim()
const cargoCommand = String(process.env.CARGO_COMMAND || 'cargo').trim()

rmSync(stagingDir, { recursive: true, force: true })
mkdirSync(stagingDir, { recursive: true })
mkdirSync(releaseDir, { recursive: true })

const cargoArgs = ['build', '--release', '--manifest-path', nativeManifest]
if (targetTriple) cargoArgs.push('--target', targetTriple)
run(cargoCommand, cargoArgs, { env: { CARGO_TARGET_DIR: targetDir } })

const binaryFileName = platform.startsWith('windows-') ? `${binaryName}.exe` : binaryName
const binaryPath = targetTriple
  ? join(targetDir, targetTriple, 'release', binaryFileName)
  : join(targetDir, 'release', binaryFileName)
if (!existsSync(binaryPath)) fail(`built native executable not found at ${binaryPath}`)

cpSync(manifestPath, join(stagingDir, 'manifest.json'))
const copiedModules = new Set()
const copyTrustedModule = (requestedPath) => {
  const modulePath = safeModulePath(requestedPath)
  if (copiedModules.has(modulePath)) return
  copiedModules.add(modulePath)
  const sourcePath = resolve(addonDir, modulePath)
  const packageRelative = relative(addonDir, sourcePath).replaceAll('\\', '/')
  if (!packageRelative || packageRelative === '..' || packageRelative.startsWith('../')) {
    fail(`trusted module escapes the addon package: ${requestedPath}`)
  }
  if (!existsSync(sourcePath)) fail(`trusted module is missing: ${modulePath}`)
  const source = readFileSync(sourcePath, 'utf8')
  const destination = join(stagingDir, modulePath)
  mkdirSync(dirname(destination), { recursive: true })
  writeFileSync(destination, source)

  for (const specifier of staticImportSpecifiers(source)) {
    if (!specifier.startsWith('.')) fail(`trusted module imports an external dependency: ${specifier}`)
    let dependency = normalize(join(dirname(modulePath), specifier)).replaceAll('\\', '/')
    if (!extname(dependency)) dependency += '.js'
    copyTrustedModule(dependency)
  }
}
copyTrustedModule(manifest.runtime.entry)

if (existsSync(join(addonDir, 'assets'))) {
  cpSync(join(addonDir, 'assets'), join(stagingDir, 'assets'), { recursive: true })
}

const stagedSidecar = join(stagingDir, safeModulePath(sidecarRelativePath, 'native executable path'))
mkdirSync(dirname(stagedSidecar), { recursive: true })
cpSync(binaryPath, stagedSidecar)

if (process.env.ELEPHANT_ADDON_MATERIALIZE_SOURCE === '1') {
  const sourceSidecar = resolve(addonDir, safeModulePath(sidecarRelativePath, 'native executable path'))
  mkdirSync(dirname(sourceSidecar), { recursive: true })
  cpSync(binaryPath, sourceSidecar)
  console.log(`[physical-addon] materialized=${sourceSidecar}`)
}

if (process.env.ELEPHANT_ADDON_MATERIALIZE_ONLY === '1') {
  console.log(`[physical-addon] runner=${runner} modules=${copiedModules.size}`)
  process.exit(0)
}

const outputName = `${safeId}-${manifest.version}-${platform}.enaddon`
const outputPath = join(releaseDir, outputName)
const packagerOutput = run('cargo', [
  'run',
  '--quiet',
  '--manifest-path',
  resolve(repoRoot, 'build/tools/enaddon-packager/Cargo.toml'),
  '--',
  stagingDir,
  outputPath
], { capture: true })

const metadata = JSON.parse(packagerOutput.split(/\r?\n/).filter(Boolean).at(-1))
const metadataPath = `${outputPath}.json`
writeFileSync(metadataPath, `${JSON.stringify({
  ...metadata,
  id: manifest.id,
  version: manifest.version,
  platform,
  runner,
  modules: copiedModules.size,
  file: outputName,
  catalogPath: `${addonRelativeDir}/releases/${outputName}`
}, null, 2)}\n`)

console.log(`[physical-addon] package=${outputPath}`)
console.log(`[physical-addon] runner=${runner} modules=${copiedModules.size}`)
console.log(`[physical-addon] blake3=${metadata.blake3}`)
console.log(`[physical-addon] metadata=${metadataPath}`)
