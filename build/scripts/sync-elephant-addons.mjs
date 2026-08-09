import fs from 'node:fs'
import path from 'node:path'
import { execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..')
const repository = 'https://github.com/SorbetUP/Elephant-Addons.git'
const pinnedRef = process.env.ELEPHANT_ADDONS_REF || '08b74d125c27cf07a8defa417a22dc708b586c12'
const cacheRoot = path.join(root, '.cache', 'elephant-addons')

const runGit = (args, cwd = root) => execFileSync('git', args, { cwd, stdio: 'inherit' })
const currentHead = () => {
  try {
    return execFileSync('git', ['rev-parse', 'HEAD'], { cwd: cacheRoot, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim()
  } catch {
    return ''
  }
}

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

if (currentHead() !== pinnedRef) {
  fs.rmSync(cacheRoot, { recursive: true, force: true })
  fs.mkdirSync(path.dirname(cacheRoot), { recursive: true })
  runGit(['clone', '--filter=blob:none', '--no-checkout', repository, cacheRoot])
  runGit(['fetch', '--depth', '1', 'origin', pinnedRef], cacheRoot)
  runGit(['checkout', '--detach', 'FETCH_HEAD'], cacheRoot)
  if (currentHead() !== pinnedRef) throw new Error(`Elephant-Addons checkout mismatch: expected ${pinnedRef}`)
}

for (const required of ['catalog.json', 'official', 'packs/base.enaddonpack', 'packs/develop-parity.enaddonpack']) {
  if (!fs.existsSync(path.join(cacheRoot, required))) throw new Error(`Elephant-Addons is missing ${required}`)
}

const ensureLink = (linkName, target) => {
  const linkPath = path.join(root, linkName)
  const pointsToTarget = () => {
    try {
      return fs.realpathSync(linkPath) === fs.realpathSync(target)
    } catch {
      return false
    }
  }
  if (pointsToTarget()) return
  try {
    fs.rmSync(linkPath, { recursive: true, force: true })
    fs.symlinkSync(target, linkPath, process.platform === 'win32' ? 'junction' : 'dir')
  } catch (error) {
    // Another concurrent addons:sync may have created the correct link between
    // rmSync and symlinkSync. Treat that race as success; surface all other
    // filesystem failures with their original error and path.
    if (error?.code === 'EEXIST' && pointsToTarget()) return
    throw error
  }
}

const materializeNativeServices = () => {
  const skippedExplicitly = process.env.ELEPHANT_SKIP_NATIVE_ADDON_BUILD === '1'
  if (skippedExplicitly) {
    console.log('[addons] native service materialization skipped explicitly')
    return
  }

  const platform = platformKey()
  const officialRoot = path.join(cacheRoot, 'official')
  for (const entry of fs.readdirSync(officialRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue

    const addonDir = path.join(officialRoot, entry.name)
    const buildConfigPath = path.join(addonDir, 'addon.build.json')
    const manifestPath = path.join(addonDir, 'manifest.json')
    if (!fs.existsSync(buildConfigPath) || !fs.existsSync(manifestPath)) continue

    const buildConfig = JSON.parse(fs.readFileSync(buildConfigPath, 'utf8'))
    if (!Array.isArray(buildConfig.supportedPlatforms) || !buildConfig.supportedPlatforms.includes(platform)) continue

    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'))
    const sidecar = manifest?.native?.sidecars?.[platform]
    if (!sidecar) continue

    const executable = path.join(addonDir, sidecar)
    if (fs.existsSync(executable) && fs.statSync(executable).isFile() && fs.statSync(executable).size > 0) continue

    console.log(`[addons] building missing native service ${manifest.id} for ${platform}`)
    execFileSync(process.execPath, [
      path.join(root, 'build/scripts/build-physical-addon.mjs'),
      `addons/official/${entry.name}`
    ], {
      cwd: root,
      stdio: 'inherit',
      env: {
        ...process.env,
        ELEPHANT_ADDON_PLATFORM: platform,
        ELEPHANT_ADDON_MATERIALIZE_SOURCE: '1',
        ELEPHANT_ADDON_MATERIALIZE_ONLY: '1'
      }
    })

    if (!fs.existsSync(executable) || !fs.statSync(executable).isFile() || fs.statSync(executable).size === 0) {
      throw new Error(`Native addon service was not materialized: ${manifest.id} (${sidecar})`)
    }
  }
}

ensureLink('addons', cacheRoot)
ensureLink('packs', path.join(cacheRoot, 'packs'))
materializeNativeServices()
console.log(`[addons] materialized Elephant-Addons ${pinnedRef}`)
