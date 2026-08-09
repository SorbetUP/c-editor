import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import vm from 'node:vm'

const catalogueRoot = path.resolve(process.argv[2] || '')
if (!process.argv[2]) {
  console.error('Usage: node build/scripts/validate-addon-catalog.mjs <catalogue-directory>')
  process.exit(2)
}

const fail = (message) => {
  throw new Error(`[addon-catalog] ${message}`)
}

const safePath = (value, label) => {
  if (typeof value !== 'string' || !value.trim()) fail(`${label} must be a non-empty path`)
  const normalized = value.replaceAll('\\', '/')
  if (normalized.startsWith('/') || normalized.split('/').some((part) => part === '..' || !part)) {
    fail(`${label} is not a safe relative path: ${value}`)
  }
  return normalized
}

const readJson = async (relativePath) => {
  const raw = await fs.readFile(path.join(catalogueRoot, relativePath), 'utf8')
  try {
    return JSON.parse(raw)
  } catch (error) {
    fail(`${relativePath} is invalid JSON: ${error.message}`)
  }
}

const isTrustedManifest = (manifest = {}) => {
  const mode = String(manifest.runtime?.mode || manifest.contributes?.runtimeMode || manifest.contributes?.security?.access || '').toLowerCase()
  return ['trusted', 'full', 'full-app', 'full-app-access'].includes(mode)
}

const validateMobileNativeSupport = (entry, runner, mobilePlatform, support) => {
  if (support?.supported === true) {
    const mobileRunner = String(support.runner || '')
    if (!['embedded-rust', 'javascript-worker', 'web-worker'].includes(mobileRunner)) {
      fail(`${entry.id} declares ${mobilePlatform} support without a recognized mobile runner`)
    }
    if (mobileRunner === 'embedded-rust') {
      if (runner !== 'service') fail(`${entry.id} embedded Rust mobile host requires the service runner`)
      if (typeof support.host !== 'string' || !support.host.trim()) {
        fail(`${entry.id} embedded Rust ${mobilePlatform} support must declare a versioned host`)
      }
    }
    return
  }

  if (support?.supported !== false || typeof support?.reason !== 'string' || !support.reason.trim()) {
    fail(`${entry.id} must declare either supported mobile execution or an explicit ${mobilePlatform} incompatibility reason`)
  }
}

const validateNativeContract = (entry, manifest) => {
  if (manifest.permissions?.native !== true) return
  if (!manifest.native?.protocol) fail(`${entry.id} requests native access without declaring a native protocol`)

  const runner = manifest.native?.runner
  if (!['process', 'service'].includes(runner)) {
    fail(`${entry.id} native package must declare process or service runner explicitly`)
  }
  const expectedProtocol = runner === 'service' ? 'elephant-addon-service-v1' : 'elephant-addon-sidecar-v1'
  if (manifest.native.protocol !== expectedProtocol) {
    fail(`${entry.id} ${runner} runner must use ${expectedProtocol}`)
  }

  const sidecars = manifest.native?.sidecars
  if (!sidecars || typeof sidecars !== 'object' || Array.isArray(sidecars) || Object.keys(sidecars).length === 0) {
    fail(`${entry.id} ${runner} package must declare desktop sidecars`)
  }
  for (const [platform, relativePath] of Object.entries(sidecars)) {
    if (/^(android|ios)-/.test(platform)) fail(`${entry.id} cannot advertise a ${runner} sidecar on ${platform}`)
    if (!/^(macos|linux|windows)-(aarch64|x86_64)$/.test(platform)) {
      fail(`${entry.id} uses unsupported ${runner}-sidecar platform key ${platform}`)
    }
    safePath(relativePath, `${entry.id}.native.sidecars.${platform}`)
  }

  for (const mobilePlatform of ['android', 'ios']) {
    validateMobileNativeSupport(entry, runner, mobilePlatform, manifest.native?.mobile?.[mobilePlatform])
  }
}

const staticImportSpecifiers = (source) => {
  const matches = []
  const pattern = /(?:^|[;\n])\s*import\s+(?:[^'";]+?\s+from\s+)?['"]([^'"]+)['"]/gm
  for (const match of source.matchAll(pattern)) matches.push(match[1])
  return matches
}

const validateTrustedModuleGraph = async (entry, rootEntryPath) => {
  const packagePrefix = `${path.posix.dirname(entry.manifestPath)}/`
  const visited = new Set()

  const visit = async (modulePath) => {
    const normalized = path.posix.normalize(modulePath.replaceAll('\\', '/'))
    if (!normalized.startsWith(packagePrefix) || normalized.includes('/../')) {
      fail(`${entry.id} trusted module escapes its package directory: ${modulePath}`)
    }
    if (visited.has(normalized)) return
    visited.add(normalized)

    let source
    try {
      source = await fs.readFile(path.join(catalogueRoot, normalized), 'utf8')
    } catch (error) {
      fail(`${entry.id} trusted module is missing: ${normalized} (${error.message})`)
    }
    if (/\bimport\s*\(/.test(source)) {
      fail(`${entry.id} trusted modules cannot use dynamic import: ${normalized}`)
    }

    for (const specifier of staticImportSpecifiers(source)) {
      if (!specifier.startsWith('.')) {
        fail(`${entry.id} trusted module imports an external dependency: ${specifier}`)
      }
      let resolved = path.posix.normalize(path.posix.join(path.posix.dirname(normalized), specifier))
      if (!path.posix.extname(resolved)) resolved += '.js'
      await visit(resolved)
    }
  }

  await visit(rootEntryPath)
  return visited.size
}

const validateTrustedEntry = async (entry, manifest, source) => {
  if (!/export\s+default\s+class\s+[A-Za-z_$][\w$]*/.test(source)) {
    fail(`${entry.id} trusted entry must export one default addon class`)
  }
  if (!/\bonload\s*\(/.test(source)) fail(`${entry.id} trusted entry must implement onload(api)`)
  const moduleCount = await validateTrustedModuleGraph(entry, entry.entryPath)
  validateNativeContract(entry, manifest)
  console.log(`[addon-catalog] ok id=${entry.id} version=${entry.version} runtime=trusted modules=${moduleCount} official=${entry.official === true}`)
}

const validateIsolatedEntry = async (entry, manifest, source) => {
  const sandbox = { self: {}, Intl, Date, Math }
  vm.runInNewContext(source, sandbox, { filename: entry.entryPath, timeout: 1_000 })
  const definition = sandbox.self.elephantAddon
  if (!definition || typeof definition.activate !== 'function') {
    fail(`${entry.id} must assign self.elephantAddon.activate`)
  }

  const registeredCommands = []
  const registeredViews = []
  const unavailable = (name) => async () => fail(`${entry.id} called ${name} during activation`)
  const api = Object.freeze({
    app: Object.freeze({ info: unavailable('app.info') }),
    notes: Object.freeze({
      list: unavailable('notes.list'),
      read: unavailable('notes.read'),
      write: unavailable('notes.write')
    }),
    http: Object.freeze({ request: unavailable('http.request') }),
    storage: Object.freeze({
      get: async () => null,
      set: unavailable('storage.set'),
      remove: unavailable('storage.remove'),
      entries: unavailable('storage.entries')
    }),
    commands: Object.freeze({
      register(command) {
        if (!command?.id?.startsWith(`${entry.id}.`)) fail(`${entry.id} registered invalid command id ${command?.id}`)
        if (typeof command.run !== 'function') fail(`${command.id} has no run handler`)
        registeredCommands.push(command)
        return () => {}
      }
    }),
    views: Object.freeze({
      register(view) {
        if (!view?.id?.startsWith(`${entry.id}.`)) fail(`${entry.id} registered invalid view id ${view?.id}`)
        if (typeof view.kind !== 'string' || !view.kind) fail(`${view.id} has no declarative kind`)
        if (typeof view.getState !== 'function') fail(`${view.id} has no getState handler`)
        if (typeof view.dispatch !== 'function') fail(`${view.id} has no dispatch handler`)
        registeredViews.push(view)
        return () => {}
      }
    })
  })
  await definition.activate(api)

  const declaredCommands = Array.isArray(manifest.contributes?.commands) ? manifest.contributes.commands : []
  const registeredCommandIds = registeredCommands.map((command) => command.id).sort()
  const declaredCommandIds = declaredCommands.map((command) => command.id).sort()
  if (JSON.stringify(registeredCommandIds) !== JSON.stringify(declaredCommandIds)) {
    fail(`${entry.id} registered commands do not match manifest contributions`)
  }

  const declaredViews = Array.isArray(manifest.contributes?.views) ? manifest.contributes.views : []
  const registeredViewSignatures = registeredViews.map((view) => `${view.id}:${view.kind}`).sort()
  const declaredViewSignatures = declaredViews.map((view) => `${view.id}:${view.kind}`).sort()
  if (JSON.stringify(registeredViewSignatures) !== JSON.stringify(declaredViewSignatures)) {
    fail(`${entry.id} registered views do not match manifest contributions`)
  }

  console.log(`[addon-catalog] ok id=${entry.id} version=${entry.version} runtime=isolated commands=${registeredCommands.length} views=${registeredViews.length}`)
  return { commands: registeredCommands.length, views: registeredViews.length }
}

const catalog = await readJson('catalog.json')
if (catalog.version !== 1) fail(`unsupported catalogue version ${catalog.version}`)
if (!['addon-catalog', 'integrated', 'main'].includes(catalog.branch)) fail('branch marker must be addon-catalog, integrated or main')
if (!Array.isArray(catalog.addons) || catalog.addons.length === 0) fail('catalogue must contain at least one addon')
const packageRoot = safePath(catalog.packageRoot || 'addons', 'packageRoot')

const ids = new Set()
const slugs = new Set()
let commandCount = 0
let viewCount = 0
let trustedCount = 0
let officialCount = 0

for (const entry of catalog.addons) {
  for (const field of ['id', 'slug', 'name', 'version', 'manifestPath', 'entryPath']) {
    if (typeof entry[field] !== 'string' || !entry[field].trim()) fail(`${entry.id || entry.slug || 'entry'} is missing ${field}`)
  }
  if (!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(entry.slug)) fail(`invalid slug ${entry.slug}`)
  if (ids.has(entry.id)) fail(`duplicate addon id ${entry.id}`)
  if (slugs.has(entry.slug)) fail(`duplicate addon slug ${entry.slug}`)
  ids.add(entry.id)
  slugs.add(entry.slug)

  const firstPartyId = entry.id.startsWith('elephant.')
  if (firstPartyId && entry.official !== true) fail(`${entry.id} must be explicitly marked official`)
  if (!firstPartyId && entry.official === true) fail(`${entry.id} cannot use the official first-party marker`)
  if (entry.official === true) officialCount += 1

  const prefix = `${packageRoot}/${entry.slug}/`
  const manifestPath = safePath(entry.manifestPath, `${entry.id}.manifestPath`)
  const entryPath = safePath(entry.entryPath, `${entry.id}.entryPath`)
  entry.manifestPath = manifestPath
  entry.entryPath = entryPath
  if (!manifestPath.startsWith(prefix) || !entryPath.startsWith(prefix)) {
    fail(`${entry.id} files must stay under ${prefix}`)
  }

  const manifest = await readJson(manifestPath)
  if (manifest.id !== entry.id || manifest.name !== entry.name || manifest.version !== entry.version) {
    fail(`${entry.id} catalogue metadata does not match its manifest`)
  }
  if (manifest.apiVersion !== 1) fail(`${entry.id} must use apiVersion 1`)
  if (manifest.runtime?.type !== 'javascript-worker') fail(`${entry.id} must use javascript-worker`)
  if (manifest.runtime?.entry !== path.posix.basename(entryPath)) {
    fail(`${entry.id} runtime.entry must match entryPath`)
  }

  const source = await fs.readFile(path.join(catalogueRoot, entryPath), 'utf8')
  if (isTrustedManifest(manifest)) {
    await validateTrustedEntry(entry, manifest, source)
    trustedCount += 1
  } else {
    validateNativeContract(entry, manifest)
    const counts = await validateIsolatedEntry(entry, manifest, source)
    commandCount += counts.commands
    viewCount += counts.views
  }
}

console.log(`[addon-catalog] valid addons=${catalog.addons.length} official=${officialCount} trusted=${trustedCount} commands=${commandCount} views=${viewCount}`)
