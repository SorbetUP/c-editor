import assert from 'node:assert/strict'
import { chmod, mkdir, mkdtemp, readFile, readdir, rm, stat, writeFile } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import { spawn, spawnSync } from 'node:child_process'
import { createInterface } from 'node:readline'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { JSDOM } from 'jsdom'

const scriptPath = fileURLToPath(import.meta.url)
const repoRoot = path.resolve(path.dirname(scriptPath), '../..')

// The addon sources use the same extensionless relative imports that the
// renderer bundler resolves. Keep the developer command short while using
// Node to load the extracted package itself.
if (!process.execArgv.includes('--experimental-specifier-resolution=node')) {
  const result = spawnSync(process.execPath, [
    '--experimental-specifier-resolution=node',
    scriptPath,
    ...process.argv.slice(2)
  ], { cwd: repoRoot, stdio: 'inherit', env: process.env })
  process.exit(result.status ?? 1)
}

const catalogPath = path.join(repoRoot, 'addons', 'catalog.json')
const packsRoot = path.join(repoRoot, 'addons', 'packs')
const packageBuilder = path.join(repoRoot, 'build', 'scripts', 'package-addon.mjs')
const catalogValidator = path.join(repoRoot, 'build', 'scripts', 'validate-addon-catalog.mjs')
const packsValidator = path.join(repoRoot, 'build', 'scripts', 'validate-integrated-addon-packs.mjs')
const releaseRoot = process.env.ELEPHANT_E2E_ADDON_RELEASE_ROOT
  ? path.resolve(repoRoot, process.env.ELEPHANT_E2E_ADDON_RELEASE_ROOT)
  : ''
const platformKey = () => {
  const os = process.platform === 'darwin' ? 'macos' : process.platform === 'win32' ? 'windows' : process.platform
  const arch = process.arch === 'arm64' ? 'aarch64' : process.arch === 'x64' ? 'x86_64' : process.arch
  return `${os}-${arch}`
}

const fail = (message) => {
  throw new Error(`[official-addon-packs] ${message}`)
}

const run = (command, args, options = {}) => {
  const result = spawnSync(command, args, {
    cwd: options.cwd || repoRoot,
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
    env: { ...process.env, ...(options.env || {}) },
    stdio: options.inherit ? 'inherit' : ['ignore', 'pipe', 'pipe']
  })
  if (result.error) throw result.error
  if (result.status !== 0) {
    fail(`${command} ${args.join(' ')} failed with ${result.status}\n${result.stdout || ''}\n${result.stderr || ''}`)
  }
  return { stdout: String(result.stdout || ''), stderr: String(result.stderr || '') }
}

const readJson = async (file) => JSON.parse(await readFile(file, 'utf8'))

const listFiles = async (root, prefix = '') => {
  const directory = path.join(root, prefix)
  const entries = await readdir(directory, { withFileTypes: true })
  const files = []
  for (const entry of entries) {
    const relativePath = prefix ? `${prefix}/${entry.name}` : entry.name
    if (entry.isDirectory()) files.push(...await listFiles(root, relativePath))
    else if (entry.isFile()) files.push(relativePath)
  }
  return files
}

const normalizeNodeImports = async (packageRoot) => {
  // The production bundler resolves these imports. Node 22 no longer honors
  // --experimental-specifier-resolution=node, so only the extracted test copy
  // gets explicit .js suffixes when the target file is present.
  for (const relativePath of await listFiles(packageRoot)) {
    if (!relativePath.endsWith('.js')) continue
    const file = path.join(packageRoot, relativePath)
    const source = await readFile(file, 'utf8')
    const normalized = source.replace(/((?:from|import)\s*["'])(\.\/[A-Za-z0-9._/-]+)(["'])/g, (match, prefix, specifier, suffix) => {
      if (path.extname(specifier)) return match
      const target = path.join(path.dirname(file), `${specifier.slice(2)}.js`)
      return existsSync(target) ? `${prefix}${specifier}.js${suffix}` : match
    })
    if (normalized !== source) await writeFile(file, normalized)
  }
}

const safeRelative = (value, label) => {
  const normalized = String(value || '').replaceAll('\\', '/')
  if (!normalized || normalized.startsWith('/') || normalized.split('/').some((part) => !part || part === '..')) {
    fail(`${label} is not a safe relative path: ${value}`)
  }
  return normalized.replace(/^\.\//, '')
}

const listPackFiles = async () => (await listFiles(packsRoot))
  .filter((relativePath) => relativePath.endsWith('.enaddonpack'))
  .map((relativePath) => path.join(packsRoot, relativePath))
  .sort()

const validatePack = (pack, fileName, catalogById) => {
  if (pack?.format !== 'elephantnote-addon-pack' || pack?.version !== 1) {
    fail(`${fileName}: unsupported pack format`)
  }
  if (!Array.isArray(pack.addons) || pack.addons.length === 0) fail(`${fileName}: empty addon list`)

  const positions = new Map()
  for (const [index, item] of pack.addons.entries()) {
    if (!item || typeof item.id !== 'string') fail(`${fileName}: addon ${index} has no id`)
    if (positions.has(item.id)) fail(`${fileName}: duplicate addon ${item.id}`)
    positions.set(item.id, index)
    if (item.source !== 'official' || item.enabled !== true) {
      fail(`${fileName}/${item.id}: integrated addons must be official and enabled`)
    }
    const entry = catalogById.get(item.id)
    if (!entry?.official) fail(`${fileName}/${item.id}: missing official catalogue entry`)
    if (item.version !== entry.version) {
      fail(`${fileName}/${item.id}: obsolete or mismatched version ${item.version}; expected ${entry.version}`)
    }
  }

  for (const item of pack.addons) {
    const entry = catalogById.get(item.id)
    const manifest = entry.manifest
    for (const dependencyId of Object.keys(manifest.requires || {})) {
      if (!positions.has(dependencyId)) fail(`${fileName}/${item.id}: missing dependency ${dependencyId}`)
      if (positions.get(dependencyId) > positions.get(item.id)) {
        fail(`${fileName}/${dependencyId}: dependency is after ${item.id}`)
      }
    }
  }
  return new Set(pack.addons.map((item) => item.id))
}

const runExistingValidators = () => {
  console.log('[official-addon-packs] reusing catalogue validator')
  run(process.execPath, [catalogValidator, path.join(repoRoot, 'addons')], { inherit: true })
  console.log('[official-addon-packs] reusing integrated-pack validator')
  run(process.execPath, [packsValidator], { inherit: true })
}

const findBuiltArchive = async (entry) => {
  if (!releaseRoot) return null
  const expected = `${entry.slug}-${entry.version}-${platformKey()}.enaddon`
  const archive = path.join(releaseRoot, entry.slug, expected)
  return existsSync(archive) ? archive : null
}

const packageArchive = async (entry, outputRoot) => {
  const sourceRoot = path.join(repoRoot, 'addons', 'official', entry.slug)
  const builtArchive = await findBuiltArchive(entry)
  const archive = builtArchive || path.join(outputRoot, `${entry.slug}-${entry.version}.enaddon`)
  if (builtArchive) {
    console.log(`[official-addon-packs] using platform-built archive=${builtArchive}`)
  } else {
    run(process.execPath, [packageBuilder, sourceRoot, archive], { inherit: true })
  }
  run('unzip', ['-tq', archive])

  const extracted = path.join(outputRoot, 'installed', entry.slug)
  await mkdir(extracted, { recursive: true })
  run('unzip', ['-q', archive, '-d', extracted])
  await writeFile(path.join(extracted, 'package.json'), '{"type":"module"}\n')
  await normalizeNodeImports(extracted)

  const manifest = await readJson(path.join(extracted, 'manifest.json'))
  assert.equal(manifest.id, entry.id, `${entry.id}: installed manifest id`)
  assert.equal(manifest.version, entry.version, `${entry.id}: installed manifest version`)
  const entryPath = safeRelative(manifest.runtime?.entry, `${entry.id}.runtime.entry`)
  const installedEntry = path.join(extracted, entryPath)
  assert.equal((await stat(installedEntry)).isFile(), true, `${entry.id}: installed runtime entry`)

  const sidecar = manifest.native?.sidecars?.[platformKey()]
  if (manifest.permissions?.native === true) {
    if (!sidecar) fail(`${entry.id}: no native sidecar for ${platformKey()}`)
    const sidecarPath = path.join(extracted, safeRelative(sidecar, `${entry.id}.native.sidecar`))
    const sidecarStat = await stat(sidecarPath).catch(() => null)
    if (!sidecarStat?.isFile() || sidecarStat.size === 0) fail(`${entry.id}: native sidecar is not physically installable at ${sidecar}`)
    if (process.platform !== 'win32') await chmod(sidecarPath, 0o755)
  }

  return { archive, extracted, manifest, entryPath, installedEntry }
}

const notePath = (root, relativePath) => {
  const normalized = safeRelative(relativePath, 'note path')
  const absolute = path.resolve(root, normalized)
  if (!absolute.startsWith(`${path.resolve(root)}${path.sep}`)) fail(`note path escaped fixture: ${relativePath}`)
  return { normalized, absolute }
}

const walkNotes = async (root, prefix = '') => {
  const directory = path.join(root, prefix)
  const entries = await readdir(directory, { withFileTypes: true }).catch(() => [])
  const output = []
  for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
    if (entry.name.startsWith('.')) continue
    const relativePath = prefix ? `${prefix}/${entry.name}` : entry.name
    if (entry.isDirectory()) output.push(...await walkNotes(root, relativePath))
    else if (entry.isFile() && /\.(md|markdown)$/i.test(entry.name)) output.push(relativePath)
  }
  return output
}

const createNativeRuntime = ({ addonId, manifest, packageDir, vaultDir, dataDir }) => {
  const runner = manifest.native?.runner
  const sidecar = manifest.native?.sidecars?.[platformKey()]
  const executable = sidecar ? path.join(packageDir, safeRelative(sidecar, `${addonId}.native.sidecar`)) : ''
  const environment = {
    ...process.env,
    ELEPHANT_VAULT_DIR: vaultDir,
    ELEPHANT_ADDON_DATA_DIR: dataDir,
    ELEPHANT_ADDON_PACKAGE_DIR: packageDir
  }
  let child = null
  let lines = null
  let sequence = 0

  const start = async () => {
    if (!runner) return { running: false, available: false, reason: 'addon has no native runner' }
    if (!existsSync(executable)) return { running: false, available: false, reason: `missing ${executable}` }
    if (runner === 'process') return { running: true, available: true, executable }
    if (child) return { running: true, available: true, executable }
    child = spawn(executable, [], {
      cwd: dataDir,
      env: environment,
      stdio: ['pipe', 'pipe', 'pipe']
    })
    lines = createInterface({ input: child.stdout })
    child.stderr.on('data', (chunk) => {
      const text = String(chunk).trim()
      if (text) console.log(`[official-addon-packs][${addonId}][native-stderr] ${text.slice(0, 500)}`)
    })
    await new Promise((resolve, reject) => {
      const onError = (error) => { child?.off('spawn', onSpawn); reject(error) }
      const onSpawn = () => { child?.off('error', onError); resolve() }
      child.once('error', onError)
      child.once('spawn', onSpawn)
    })
    return { running: true, available: true, executable }
  }

  const stop = async () => {
    if (!child) return { stopped: true, running: false }
    const current = child
    child = null
    lines?.close()
    lines = null
    if (!current.killed) current.kill('SIGTERM')
    await new Promise((resolve) => current.once('close', resolve))
    return { stopped: true, running: false }
  }

  const oneShot = async (method, params) => new Promise((resolve, reject) => {
    const processRef = spawn(executable, [], { cwd: dataDir, env: environment, stdio: ['pipe', 'pipe', 'pipe'] })
    let stdout = ''
    let stderr = ''
    processRef.stdout.on('data', (chunk) => { stdout += String(chunk) })
    processRef.stderr.on('data', (chunk) => { stderr += String(chunk) })
    processRef.once('error', reject)
    processRef.once('close', (code) => {
      if (code !== 0 && !stdout.trim()) return reject(new Error(`${addonId} sidecar exited ${code}: ${stderr.trim()}`))
      let response
      try { response = JSON.parse(stdout.trim().split(/\r?\n/).filter(Boolean).at(-1) || '') } catch (error) {
        return reject(new Error(`${addonId} sidecar returned invalid JSON: ${error.message}`))
      }
      if (response.ok === false) return reject(new Error(response.error?.message || `${addonId} sidecar request failed`))
      resolve(response.result ?? response)
    })
    processRef.stdin.end(JSON.stringify({ protocol: 'elephant-addon-sidecar-v1', addonId, method, params }))
  })

  const serviceCall = async (method, params) => {
    await start()
    if (!child || !lines) throw new Error(`${addonId} native service is unavailable`)
    const id = ++sequence
    const response = new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error(`${addonId} native service timed out on ${method}`)), 120_000)
      const onLine = (line) => {
        clearTimeout(timer)
        lines?.off('line', onLine)
        try { resolve(JSON.parse(line)) } catch (error) { reject(new Error(`${addonId} native service returned invalid JSON: ${error.message}`)) }
      }
      lines.on('line', onLine)
    })
    child.stdin.write(`${JSON.stringify({ protocol: 'elephant-addon-service-v1', id, addonId, method, params })}\n`)
    const result = await response
    if (result.ok === false) throw new Error(result.error?.message || `${addonId} native service rejected ${method}`)
    return result.result ?? result
  }

  return {
    status: async () => runner === 'process' ? oneShot('status', {}) : serviceCall('service.status', {}).catch((error) => ({ running: Boolean(child), available: Boolean(child), error: error.message })),
    call: async (method, params = {}) => runner === 'process' ? oneShot(method, params) : serviceCall(method, params),
    service: { start, status: async () => serviceCall('service.status', {}), call: serviceCall, stop },
    stop
  }
}

const createStorageApi = async (dataDir, defaults = {}) => {
  const storageFile = path.join(dataDir, 'storage.json')
  let storage = await readJson(storageFile).catch(() => ({ ...defaults }))
  const saveStorage = async () => {
    await mkdir(path.dirname(storageFile), { recursive: true })
    await writeFile(storageFile, `${JSON.stringify(storage, null, 2)}\n`)
  }
  return {
    get: async (key) => storage[key] ?? null,
    set: async (key, value) => { storage[key] = value; await saveStorage(); return value },
    remove: async (key) => { delete storage[key]; await saveStorage(); return true },
    entries: async () => ({ ...storage })
  }
}

const createRuntime = async ({ packName, fixtureRoot, addonId, packageDir, manifest }) => {
  const vaultDir = path.join(fixtureRoot, 'vault')
  const dataDir = path.join(fixtureRoot, 'addon-data', addonId)
  await mkdir(path.join(vaultDir, 'Sites'), { recursive: true })
  await mkdir(dataDir, { recursive: true })
  const dom = new JSDOM('<!doctype html><html><body></body></html>', { url: 'https://elephant.test/' })
  const browserWindow = dom.window
  const state = {
    resources: new Map(),
    registrations: { commands: [], views: [], layouts: [], settings: [], styles: [], contributions: [] },
    openedViews: [],
    events: [],
    files: [],
    native: null,
    store: {
      activeVaultId: `integration-${packName}`,
      activeVault: { id: `integration-${packName}`, name: 'Pack integration vault', path: vaultDir },
      workspaceStats: { notes: 2, folders: 1 },
      recentNoteEntries: [{ path: 'Integration.md', title: 'Integration', kind: 'note', type: 'note', updatedAt: Date.now() }],
      openedNotePath: '',
      chatSidebarOpen: false,
      openNote(note) { this.openedNotePath = note?.path || '' },
      toggleChatSidebar() { this.chatSidebarOpen = !this.chatSidebarOpen }
    }
  }
  state.native = createNativeRuntime({ addonId, manifest, packageDir, vaultDir, dataDir })
  const nativeRuntimes = [state.native]

  const previousGlobals = new Map([
    ['window', globalThis.window],
    ['document', globalThis.document],
    ['CustomEvent', globalThis.CustomEvent],
    ['dispatchEvent', globalThis.dispatchEvent],
    ['MutationObserver', globalThis.MutationObserver]
  ])
  browserWindow.__ELEPHANT_ADDON_VUE__ = {
    createDomComponent: ({ name, mount, className = '' }) => ({ name, className, __mount: mount }),
    getStore: (_pinia, name) => name === 'elephantnoteVaults' ? state.store : null
  }
  browserWindow.__TAURI__ = {
    core: {
      convertFileSrc: (value) => `asset://localhost/${String(value).replaceAll('\\', '/')}`,
      invoke: async (command, params = {}) => {
        if (command === 'tauri_notes_read') {
          const target = notePath(vaultDir, params.relativePath)
          if (!existsSync(target.absolute)) throw new Error(`Note does not exist: ${target.normalized}`)
          return { path: target.normalized, fullPath: target.absolute, title: path.basename(target.normalized, path.extname(target.normalized)) }
        }
        if (command === 'tauri_notes_create') {
          const target = notePath(vaultDir, path.join(params.relativePath || '', params.filename || 'Untitled.md'))
          await mkdir(path.dirname(target.absolute), { recursive: true })
          await writeFile(target.absolute, `# ${params.title || 'Untitled'}\n`)
          return { path: target.normalized, fullPath: target.absolute, title: params.title || 'Untitled' }
        }
        if (command === 'tauri_addons_notes_list') return await noteEntries(vaultDir, params.prefix || '.')
        if (command === 'tauri_addons_notes_read') return { markdown: await readFile(notePath(vaultDir, params.path).absolute, 'utf8') }
        if (command === 'tauri_addons_notes_write') return await writeNote(vaultDir, params.path, params.markdown)
        if (command === 'tauri_addons_assets_allow_directory') {
          const target = notePath(vaultDir, params.relativePath)
          const details = await stat(target.absolute)
          if (!details.isDirectory()) throw new Error(`${target.normalized} is not a directory`)
          return { relativePath: target.normalized, path: target.absolute }
        }
        if (command === 'tauri_vault_remove_path') {
          const target = notePath(vaultDir, params.pathname)
          await rm(target.absolute, { recursive: true, force: true })
          return { ok: true, path: target.normalized }
        }
        if (command === 'tauri_vault_read_binary') return { dataBase64: '' }
        if (command === 'tauri_vault_write_binary') return { ok: true }
        if (command === 'tauri_addons_call') {
          if (params.method === 'notes.list') return await noteEntries(vaultDir, params.params?.prefix || '.')
          if (params.method === 'notes.read') return await readFile(notePath(vaultDir, params.params?.path).absolute, 'utf8')
          if (params.method === 'notes.write') return await writeNote(vaultDir, params.params?.path, params.params?.content)
        }
        throw new Error(`Unsupported integration Tauri command: ${command}`)
      }
    }
  }
  browserWindow.console = console
  browserWindow.setTimeout = setTimeout
  browserWindow.clearTimeout = clearTimeout
  globalThis.window = browserWindow
  globalThis.document = browserWindow.document
  globalThis.CustomEvent = browserWindow.CustomEvent
  globalThis.dispatchEvent = browserWindow.dispatchEvent.bind(browserWindow)
  globalThis.MutationObserver = browserWindow.MutationObserver

  const storageApi = await createStorageApi(dataDir)
  const track = (collection, value) => {
    collection.push(value)
    return () => { const index = collection.indexOf(value); if (index >= 0) collection.splice(index, 1) }
  }
  const baseApi = {
    manifest,
    experimental: { window: browserWindow },
    logger: { info: (message, payload) => console.log(`[official-addon-packs][${addonId}] ${message}`, payload || '') },
    app: {
      pinia: { _s: new Map([['elephantnoteVaults', state.store]]) },
      runtime: 'integration',
      addons: {
        getContributions: (kind) => state.registrations.contributions.filter((item) => item.kind === kind),
        get: () => null,
        list: () => [],
        on: () => () => {}
      },
      emit: (name, payload) => state.events.push({ name, payload }),
      host: {},
      services: {},
      router: {}
    },
    storage: storageApi,
    notes: {
      list: async (prefix = '.') => noteEntries(vaultDir, prefix),
      read: async (relativePath) => ({ path: safeRelative(relativePath, 'note path'), content: await readFile(notePath(vaultDir, relativePath).absolute, 'utf8') }),
      write: async (relativePath, content) => writeNote(vaultDir, relativePath, content)
    },
    native: state.native,
    resources: {
      get: (name) => state.resources.get(name),
      has: (name) => state.resources.has(name),
      list: () => [...state.resources.keys()],
      provide: (name, value) => { state.resources.set(name, value); return () => { if (state.resources.get(name) === value) state.resources.delete(name) } },
      watch: (_name, listener, options = {}) => { if (options.immediate !== false) listener({ value: state.resources.get(_name), previous: undefined }); return () => {} }
    },
    commands: { register: (definition) => track(state.registrations.commands, definition) },
    workspace: {
      registerView: (definition) => track(state.registrations.views, definition),
      registerSidebarItem: (definition) => track(state.registrations.layouts, definition),
      registerContribution: (kind, contribution) => { const entry = { kind, contribution }; state.registrations.contributions.push(entry); return () => { const index = state.registrations.contributions.indexOf(entry); if (index >= 0) state.registrations.contributions.splice(index, 1) } },
      openView: (id) => { state.openedViews.push(id); return id }
    },
    settings: { registerSection: (definition) => track(state.registrations.settings, definition), registerPage: (definition) => track(state.registrations.settings, definition) },
    layout: { registerZone: (definition) => track(state.registrations.layouts, definition), registerItem: (definition) => track(state.registrations.layouts, definition) },
    ui: { registerStyle: (cssText, id) => track(state.registrations.styles, { cssText, id }), mount: (_host, render) => render?.(browserWindow.document.body), on: () => () => {}, observe: () => () => {} },
    editor: { active: null, watch: () => () => {}, registerExtension: () => () => {}, registerBlockType: () => () => {}, registerInlineType: () => () => {}, registerInputRule: () => () => {}, registerToolbarItem: () => () => {}, registerPasteHandler: () => () => {} },
    markdown: { registerPostProcessor: () => () => {}, registerCodeBlockProcessor: () => () => {}, registerEmbedRenderer: () => () => {} },
    router: { addRoute: () => () => {}, beforeEach: () => () => {}, afterEach: () => () => {} },
    vue: { component: () => () => {}, directive: () => () => {}, provide: () => () => {} },
    patch: { method: () => () => {}, property: () => () => {}, hook: () => () => {}, runHook: async (_name, value) => value },
    http: { request: async () => { throw new Error('Network is intentionally unavailable in the pack fixture') } }
  }
  let addonApi = { ...baseApi }

  return {
    get api() { return addonApi },
    state,
    browserWindow,
    async setAddon({ addonId: nextAddonId, packageDir: nextPackageDir, manifest: nextManifest }) {
      const nextDataDir = path.join(fixtureRoot, 'addon-data', nextAddonId)
      await mkdir(nextDataDir, { recursive: true })
      const nextNative = createNativeRuntime({
        addonId: nextAddonId,
        manifest: nextManifest,
        packageDir: nextPackageDir,
        vaultDir,
        dataDir: nextDataDir
      })
      const nextStorage = await createStorageApi(nextDataDir, nextAddonId === 'elephant.ai-search'
        ? { 'search-config-v3': { enabled: false, autoRebuild: false } }
        : {})
      state.native = nextNative
      nativeRuntimes.push(nextNative)
      addonApi = {
        ...baseApi,
        manifest: nextManifest,
        native: nextNative,
        storage: nextStorage,
        logger: { info: (message, payload) => console.log(`[official-addon-packs][${nextAddonId}] ${message}`, payload || '') }
      }
      return addonApi
    },
    async dispose() {
      await Promise.all(nativeRuntimes.map((native) => native.stop().catch(() => {})))
      dom.window.close()
      for (const [name, value] of previousGlobals) {
        if (value === undefined) delete globalThis[name]
        else globalThis[name] = value
      }
    }
  }
}

const writeNote = async (root, relativePath, content) => {
  const target = notePath(root, relativePath)
  await mkdir(path.dirname(target.absolute), { recursive: true })
  const value = String(content ?? '')
  await writeFile(target.absolute, value)
  return { ok: true, path: target.normalized, bytes: Buffer.byteLength(value) }
}

const noteEntries = async (root, prefix = '.') => {
  const normalizedPrefix = prefix === '.' ? '' : safeRelative(prefix, 'note prefix')
  const paths = await walkNotes(root, normalizedPrefix)
  return Promise.all(paths.map(async (relativePath) => ({
    path: relativePath,
    size: (await stat(path.join(root, relativePath))).size,
    modifiedAt: (await stat(path.join(root, relativePath))).mtimeMs
  })))
}

const findCommand = (state, addonId, suffix) => state.registrations.commands.find((command) => command.id === `${addonId}${suffix}` || command.id.endsWith(`${addonId}${suffix}`))

const exercise = async (entry, runtime, instance) => {
  const { id } = entry
  const { state, api, browserWindow } = runtime
  if (id === 'elephant.dashboard') {
    const command = findCommand(state, id, '.open')
    assert.ok(command, `${id}: Dashboard command was not registered`)
    const result = await command.run()
    assert.equal(result.path, '.elephantnote/Dashboard.md')
    assert.equal(existsSync(path.join(state.store.activeVault.path, result.path)), true)
    return 'created and opened the persisted Dashboard note'
  }
  if (id === 'elephant.ai') {
    const resource = state.resources.get('ai.config')
    assert.ok(resource?.set && resource?.get, `${id}: ai.config resource missing`)
    const config = { providers: { list: [{ id: 'pack-provider', type: 'openai-compatible', endpoint: 'https://api.openai.com/v1', enabled: true }] }, routes: {} }
    await resource.set(config)
    assert.equal((await resource.get()).providers.list[0].id, 'pack-provider')
    return 'persisted and reloaded ai.config'
  }
  if (id === 'elephant.ai-chat') {
    const command = findCommand(state, id, '.toggle') || findCommand(state, id, '.open')
    assert.ok(command, `${id}: Chat action was not registered`)
    const result = await command.run()
    assert.equal(result.open, true)
    return 'toggled the real chat sidebar state'
  }
  if (id === 'elephant.ai-search') {
    const provider = state.resources.get('search.provider')
    assert.ok(provider?.clear && provider?.status, `${id}: search provider missing`)
    await provider.clear()
    assert.equal((await provider.status()).notesIndexed, 0)
    return 'cleared and inspected the persisted lexical search index'
  }
  if (id === 'elephant.ai-ocr') {
    const result = await state.resources.get('ocr')?.status?.()
    assert.ok(result && typeof result === 'object', `${id}: OCR status did not return a result`)
    return `called the installed OCR sidecar status (${result.available ? 'available' : 'unavailable'})`
  }
  if (id === 'elephant.wiki') {
    const result = await state.resources.get('wiki.provider')?.status?.()
    assert.equal(result?.engine, 'package-owned-wiki')
    return `inspected Wiki state (${result.records} records)`
  }
  if (id === 'elephant.graph') {
    const command = findCommand(state, id, '.open')
    assert.ok(command, `${id}: graph action was not registered`)
    await command.run()
    assert.equal(state.openedViews.at(-1), 'elephant.graph.workspace')
    return 'opened the package-owned graph workspace'
  }
  if (id === 'elephant.knowledge') {
    const provider = state.resources.get('knowledge.provider')
    assert.ok(provider?.status, `${id}: knowledge provider missing`)
    const result = await provider.status()
    assert.equal(typeof result, 'object')
    assert.equal(typeof result.documents, 'number')
    return `queried the real Knowledge service (${result.documents} documents)`
  }
  if (id === 'elephant.open-models') {
    const provider = state.resources.get('models.provider')
    assert.ok(provider?.status && provider?.list, `${id}: model provider missing`)
    const status = await provider.status()
    const models = await provider.list()
    assert.equal(Array.isArray(models), true)
    return `queried the real model service (${status.serverRunning ? 'service running' : 'service ready'}, ${models.length} models)`
  }
  if (id === 'elephant.codex-connection') {
    const provider = state.registrations.contributions.find((item) => item.kind === 'ai.providers')?.contribution
    assert.ok(provider?.getModels, `${id}: Codex provider was not registered`)
    try {
      const models = await provider.getModels()
      assert.equal(Array.isArray(models), true)
      return `queried Codex models (${models.length} returned)`
    } catch (error) {
      assert.match(String(error?.message || error), /Codex|codex|app-server|auth|found/i)
      return `surfaced the expected unavailable Codex runtime: ${error.message}`
    }
  }
  if (id === 'elephant.sync') {
    const service = state.resources.get('sync.native-service')
    assert.ok(service?.status, `${id}: Sync service resource missing`)
    const result = await service.status()
    console.log(`[official-addon-packs] sync-status=${JSON.stringify(result)}`)
    assert.equal(result?.owner, 'elephant.sync')
    assert.equal(result?.stableIdentity, true)
    return `queried the live Iroh service (${result.endpointId})`
  }
  if (id === 'elephant.calendar') {
    const provider = state.resources.get('calendar.provider')
    const result = await provider.importIcs('BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:pack-integration\nDTSTART:20260810T100000Z\nDTEND:20260810T110000Z\nSUMMARY:Pack integration\nEND:VEVENT\nEND:VCALENDAR', 'pack.ics')
    assert.equal(result.imported, 1)
    assert.equal((await provider.list()).length, 1)
    return 'imported and reloaded one ICS event'
  }
  if (id === 'elephant.sites') {
    await writeNote(state.store.activeVault.path, 'Sites/Home.md', '# Pack site\n\nA generated page.\n')
    const provider = state.resources.get('sites.provider')
    const generated = await instance.generate({ sourceDirectory: 'Sites', mode: 'preview' })
    assert.equal(generated.pages, 1)
    assert.equal(existsSync(path.join(state.store.activeVault.path, generated.indexPath)), true)
    await provider.stop(generated.siteId)
    assert.equal(existsSync(path.join(state.store.activeVault.path, generated.relativePath)), false)
    return 'generated, persisted, and cleaned a real static-site preview'
  }
  if (id === 'elephant.code-execution') {
    const status = await instance.service('interpreter.status', { executable: 'python3', args: ['-'] })
    assert.equal(typeof status.available, 'boolean')
    if (!status.available) return `surfaced unavailable Python interpreter: ${status.error || 'not installed'}`
    const started = await instance.service('execute', { executable: 'python3', args: ['-'], code: 'print(2 + 2)', timeoutMs: 15_000, outputLineLimit: 20 })
    assert.ok(started.executionId, `${id}: execution service returned no execution id`)
    let snapshot = await instance.service('execution.status', { executionId: started.executionId })
    for (let index = 0; index < 100 && snapshot.running; index += 1) {
      await new Promise((resolve) => setTimeout(resolve, 20))
      snapshot = await instance.service('execution.status', { executionId: started.executionId })
    }
    assert.equal(snapshot.running, false)
    assert.match(snapshot.result?.stdout || '', /4/)
    return 'executed Python through the package-owned service'
  }
  if (id === 'elephant.google-keep-import') {
    const provider = state.resources.get('import.google-keep')
    const result = await provider.importDocuments([{ name: 'Pack note.json', text: JSON.stringify({ title: 'Pack note', text: 'Imported from Keep' }) }])
    assert.equal(result.imported, 1)
    assert.equal(existsSync(path.join(state.store.activeVault.path, 'Imported/Google Keep/Pack note.md')), true)
    return 'imported a Keep document into a persisted Markdown note'
  }
  if (id === 'elephant.recently-edited') {
    const zone = state.registrations.layouts.find((item) => item.id === 'elephant.recently-edited.sidebar-section')
    assert.ok(zone?.component?.__mount, `${id}: sidebar component was not registered`)
    const host = browserWindow.document.createElement('div')
    browserWindow.document.body.append(host)
    const dispose = zone.component.__mount(host)
    assert.match(host.textContent, /Recently edited/)
    assert.match(host.textContent, /Integration/)
    dispose?.()
    host.remove()
    return 'mounted the real Recently edited sidebar contribution'
  }
  fail(`${id}: no significant action scenario is defined`)
}

const executeIntegration = async () => {
  runExistingValidators()
  const catalog = await readJson(catalogPath)
  const catalogEntries = catalog.addons.filter((entry) => entry.official === true && entry.id.startsWith('elephant.'))
  const catalogById = new Map()
  for (const entry of catalogEntries) {
    const manifest = await readJson(path.join(repoRoot, 'addons', entry.manifestPath))
    catalogById.set(entry.id, { ...entry, manifest })
    assert.equal(manifest.version, entry.version, `${entry.id}: catalogue version drift`)
    assert.equal(manifest.id, entry.id, `${entry.id}: manifest id drift`)
  }
  const packFiles = await listPackFiles()
  if (!packFiles.length) fail('no .enaddonpack files found')

  const packs = []
  const covered = new Set()
  for (const file of packFiles) {
    const pack = await readJson(file)
    const ids = validatePack(pack, path.basename(file), catalogById)
    packs.push({ file, pack, ids })
    for (const id of ids) covered.add(id)
    console.log(`[official-addon-packs] pack=${path.basename(file)} order=${pack.addons.map((item) => item.id).join(' -> ')}`)
  }
  const missing = catalogEntries.map((entry) => entry.id).filter((id) => !covered.has(id))
  assert.deepEqual(missing, [], 'the union of all integrated packs must cover every official catalogue addon')

  const staleProbe = structuredClone(packs[0].pack)
  staleProbe.addons[0] = { ...staleProbe.addons[0], version: '0.0.0' }
  assert.throws(() => validatePack(staleProbe, `${path.basename(packs[0].file)}.obsolete-fixture`, catalogById), /obsolete or mismatched version/)
  console.log('[official-addon-packs] obsolete-version rejection probe passed')

  const fixtureRoot = await mkdtemp(path.join(tmpdir(), 'elephant-official-addon-packs-'))
  const packageRoot = path.join(fixtureRoot, 'packages')
  await mkdir(packageRoot, { recursive: true })
  const packages = new Map()
  try {
    for (const entry of catalogEntries) {
      const packaged = await packageArchive(entry, packageRoot)
      packages.set(entry.id, packaged)
      console.log(`[official-addon-packs] installable id=${entry.id} version=${entry.version} archive=${packaged.archive}`)
    }

    let actionCount = 0
    for (const { file, pack } of packs) {
      const packName = path.basename(file)
      const runtimeRoot = await mkdtemp(path.join(fixtureRoot, `${packName.replace(/\W/g, '-')}-`))
      const firstItem = pack.addons[0]
      const firstPackage = packages.get(firstItem.id)
      const runtime = await createRuntime({
        packName,
        fixtureRoot: runtimeRoot,
        addonId: firstItem.id,
        packageDir: firstPackage.extracted,
        manifest: firstPackage.manifest
      })
      const activated = []
      try {
        for (const item of pack.addons) {
          const entry = catalogById.get(item.id)
          const packaged = packages.get(item.id)
          await runtime.setAddon({ addonId: item.id, packageDir: packaged.extracted, manifest: packaged.manifest })
          const imported = await import(`${pathToFileURL(packaged.installedEntry).href}?pack=${encodeURIComponent(packName)}&version=${encodeURIComponent(entry.version)}`)
          const Addon = imported.default
          assert.equal(typeof Addon, 'function', `${item.id}: installed entry did not export an addon class`)
          console.log(`[official-addon-packs] activate pack=${packName} id=${item.id} entry=${packaged.entryPath} class=${Addon.name}`)
          const instance = Reflect.construct(Addon, [runtime.api])
          assert.equal(typeof instance.onload, 'function', `${item.id}: installed addon has no onload()`)
          await instance.onload(runtime.api)
          console.log(`[official-addon-packs] activated pack=${packName} id=${item.id} resources=${[...runtime.state.resources.keys()].join(',')}`)
          activated.push({ item, entry, instance, runtime })
          const action = await exercise(entry, runtime, instance)
          actionCount += 1
          console.log(`[official-addon-packs] action pack=${packName} id=${item.id} ${action}`)
        }
      } finally {
        for (const { instance } of [...activated].reverse()) {
          if (typeof instance.onunload === 'function') await instance.onunload()
        }
        await runtime.dispose()
        await rm(runtimeRoot, { recursive: true, force: true })
      }
    }
    assert.equal(actionCount, packs.reduce((sum, pack) => sum + pack.pack.addons.length, 0))
    console.log(`[official-addon-packs] PASS packs=${packs.length} official=${catalogEntries.length} actions=${actionCount} platform=${platformKey()}`)
  } finally {
    if (process.env.ELEPHANT_KEEP_ADDON_PACK_ARTIFACTS !== '1') await rm(fixtureRoot, { recursive: true, force: true })
    else console.log(`[official-addon-packs] artifacts=${fixtureRoot}`)
  }
}

executeIntegration().catch((error) => {
  console.error(error?.stack || error)
  process.exitCode = 1
})
