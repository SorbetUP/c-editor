import { chmodSync, copyFileSync, createWriteStream, existsSync, mkdirSync, readdirSync, rmSync, statSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { basename, dirname, extname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
const binDir = join(root, 'Elephant/backend/tauri', 'bin')
const binaryName = process.platform === 'win32' ? 'llama-server.exe' : 'llama-server'
const targetPath = join(binDir, binaryName)
const optionalInstall = process.env.ELEPHANTNOTE_LLAMA_INSTALL_REQUIRED !== '1'

const log = (...args) => console.log('[tauri-llama]', ...args)
const warn = (...args) => console.warn('[tauri-llama]', ...args)

const exitSoft = (message) => {
  if (message) warn(message)
  if (optionalInstall) {
    warn('continuing without bundled llama-server; configure an existing path in Preferences > Local AI runtime if needed')
    process.exit(0)
  }
  process.exit(1)
}

if (process.env.ELEPHANTNOTE_SKIP_LLAMA_BUNDLE === '1') {
  log('skip requested through ELEPHANTNOTE_SKIP_LLAMA_BUNDLE=1')
  process.exit(0)
}

mkdirSync(binDir, { recursive: true })

const requiredSupportFiles = () => {
  if (process.platform === 'darwin') return ['libllama-server-impl.dylib']
  return []
}

const hasRequiredRuntimeFiles = () => {
  if (!existsSync(targetPath)) return false
  return requiredSupportFiles().every((name) => existsSync(join(binDir, name)))
}

if (hasRequiredRuntimeFiles()) {
  log(`already installed: ${targetPath}`)
  process.exit(0)
}

if (existsSync(targetPath)) {
  warn(`existing ${targetPath} is incomplete; reinstalling bundled llama-server runtime`)
  rmSync(targetPath, { force: true })
}

const platformTokens = {
  darwin: ['macos', 'darwin', 'apple'],
  linux: ['ubuntu', 'linux'],
  win32: ['win', 'windows']
}[process.platform] || [process.platform]

const archTokens = {
  arm64: ['arm64', 'aarch64'],
  x64: ['x64', 'x86_64', 'amd64']
}[process.arch] || [process.arch]

const badTokens = ['cuda', 'vulkan', 'kompute', 'rocm', 'hip', 'sycl', 'opencl', 'android', 'server-cuda']

const scoreAsset = (asset) => {
  const name = asset.name.toLowerCase()
  if (!/\.(zip|tar\.gz|tgz)$/.test(name)) return -1000
  if (!name.includes('bin') && !name.includes('llama')) return -100
  if (badTokens.some((token) => name.includes(token))) return -50
  let score = 0
  for (const token of platformTokens) if (name.includes(token)) score += 20
  for (const token of archTokens) if (name.includes(token)) score += 20
  if (name.includes('server')) score += 5
  if (name.endsWith('.zip')) score += 2
  return score
}

const requestJson = async(url) => {
  const response = await fetch(url, {
    headers: {
      accept: 'application/vnd.github+json',
      'user-agent': 'ElephantNote llama-server installer'
    }
  })
  if (!response.ok) throw new Error(`${url} returned HTTP ${response.status}`)
  return response.json()
}

const download = async(url, destination) => {
  const response = await fetch(url, { headers: { 'user-agent': 'ElephantNote llama-server installer' } })
  if (!response.ok) throw new Error(`${url} returned HTTP ${response.status}`)
  if (!response.body) throw new Error('download response body is empty')
  await new Promise((resolvePromise, reject) => {
    const file = createWriteStream(destination)
    file.on('error', reject)
    response.body.pipeTo(new WritableStream({
      write(chunk) { file.write(Buffer.from(chunk)) },
      close() { file.end(resolvePromise) },
      abort(error) { file.destroy(); reject(error) }
    })).catch(reject)
  })
}

const extractArchive = (archive, destination) => {
  mkdirSync(destination, { recursive: true })
  const lower = archive.toLowerCase()
  if (lower.endsWith('.zip')) {
    if (process.platform === 'win32') {
      const result = spawnSync('powershell.exe', ['-NoProfile', '-Command', `Expand-Archive -Force ${JSON.stringify(archive)} ${JSON.stringify(destination)}`], { stdio: 'inherit' })
      if (result.status !== 0) throw new Error('PowerShell Expand-Archive failed')
      return
    }
    const unzip = spawnSync('unzip', ['-q', archive, '-d', destination], { stdio: 'inherit' })
    if (unzip.status !== 0) throw new Error('unzip failed; install unzip or place llama-server manually in Elephant/backend/tauri/bin')
    return
  }
  const tar = spawnSync('tar', ['-xzf', archive, '-C', destination], { stdio: 'inherit' })
  if (tar.status !== 0) throw new Error('tar extraction failed')
}

const findBinary = (dir) => {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry)
    const stat = statSync(path)
    if (stat.isDirectory()) {
      const found = findBinary(path)
      if (found) return found
    } else if (basename(path).toLowerCase() === binaryName.toLowerCase()) {
      return path
    }
  }
  return null
}

const supportExtensions = new Set(process.platform === 'win32' ? ['.dll'] : process.platform === 'darwin' ? ['.dylib'] : ['.so'])
const collectSupportLibraries = (dir, out = []) => {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry)
    const stat = statSync(path)
    if (stat.isDirectory()) {
      collectSupportLibraries(path, out)
    } else if (supportExtensions.has(extname(path).toLowerCase()) || /\.so(?:\.\d+)*$/i.test(path)) {
      out.push(path)
    }
  }
  return out
}

const copyRuntimeSupport = (extractDir) => {
  const libraries = collectSupportLibraries(extractDir)
  for (const source of libraries) {
    const destination = join(binDir, basename(source))
    copyFileSync(source, destination)
    if (process.platform !== 'win32') chmodSync(destination, 0o755)
    log(`installed support library ${destination}`)
  }
}

const main = async() => {
  log(`installing ${binaryName} for ${process.platform}/${process.arch}`)
  const release = await requestJson('https://api.github.com/repos/ggerganov/llama.cpp/releases/latest')
  const asset = [...(release.assets || [])]
    .map((asset) => ({ asset, score: scoreAsset(asset) }))
    .sort((a, b) => b.score - a.score)[0]

  if (!asset || asset.score <= 0) {
    throw new Error(`No suitable llama.cpp release asset found for ${process.platform}/${process.arch}`)
  }

  log(`selected asset: ${asset.asset.name}`)
  const workDir = join(tmpdir(), `elephantnote-llama-${Date.now()}`)
  const archive = join(workDir, asset.asset.name)
  const extractDir = join(workDir, 'extract')
  mkdirSync(workDir, { recursive: true })
  try {
    await download(asset.asset.browser_download_url, archive)
    extractArchive(archive, extractDir)
    const extracted = findBinary(extractDir)
    if (!extracted) throw new Error(`Downloaded archive does not contain ${binaryName}`)
    copyFileSync(extracted, targetPath)
    if (process.platform !== 'win32') chmodSync(targetPath, 0o755)
    copyRuntimeSupport(extractDir)
    const missing = requiredSupportFiles().filter((name) => !existsSync(join(binDir, name)))
    if (missing.length) throw new Error(`Downloaded llama-server is missing required runtime libraries: ${missing.join(', ')}`)
    log(`installed ${targetPath}`)
  } finally {
    rmSync(workDir, { recursive: true, force: true })
  }
}

main().catch((error) => {
  exitSoft(error?.message || String(error))
})
