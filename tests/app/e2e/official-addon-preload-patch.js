'use strict'

const INSERT_BEFORE_UPDATE_SIDEBAR = 'const updateSidebar = (params, attach) => {'

const fixtureSource = String.raw`
const crypto = require('crypto')
const childProcess = require('child_process')
const readline = require('readline')
const projectRoot = path.resolve(__dirname, '../../..')
const officialAddonsRoot = path.join(projectRoot, 'addons')
const officialAddonStateFile = path.join(configRoot, 'official-addons-e2e-state.json')
const officialAddonRequested = String(process.env.ELEPHANT_E2E_OFFICIAL_ADDONS || '').trim()

const officialAddonFixture = (() => {
  const catalogPath = path.join(officialAddonsRoot, 'catalog.json')
  if (!fs.existsSync(catalogPath)) {
    throw new Error('Official addon E2E fixture requires pnpm addons:sync before launch')
  }
  const catalog = readJson(catalogPath, { addons: [] })
  const entries = new Map((catalog.addons || []).map((entry) => [entry.id, entry]))
  const requestedIds = officialAddonRequested === 'all'
    ? [...entries.keys()]
    : officialAddonRequested.split(',').map((value) => value.trim()).filter(Boolean)
  const persisted = readJson(officialAddonStateFile, null)
  const state = persisted && Array.isArray(persisted.installed)
    ? persisted
    : { installed: requestedIds, enabled: [], storage: {} }
  if (!state.storage || typeof state.storage !== 'object') state.storage = {}
  const storage = new Map(Object.entries(state.storage))
  const secrets = new Map()
  const runningServices = new Set()
  const nativeServices = new Map()
  const realNativeEnabled = process.env.ELEPHANT_E2E_REAL_NATIVE === '1'
  const interceptedExternalAi = process.env.ELEPHANT_E2E_INTERCEPT_EXTERNAL_AI === '1'
  const httpTracePath = path.join(configRoot, 'official-addons-http-trace.json')
  const httpTrace = []
  let embeddingState = { documents: 0, modelId: '' }

  const recordHttpTrace = (entry) => {
    httpTrace.push({
      ...entry,
      at: new Date().toISOString()
    })
    fs.mkdirSync(path.dirname(httpTracePath), { recursive: true })
    fs.writeFileSync(httpTracePath, JSON.stringify(httpTrace, null, 2))
  }

  const interceptedAiRequest = (addonId, params = {}) => {
    if (!interceptedExternalAi || addonId !== 'elephant.ai' || params.method === 'HEAD') return null
    let url
    try { url = new URL(String(params.url || '')) } catch { return null }
    if (url.protocol !== 'https:' || url.hostname !== 'api.openai.com' || !url.pathname.startsWith('/v1/')) return null

    let body = null
    try { body = params.body ? JSON.parse(String(params.body)) : null } catch { body = null }
    const requestId = 'e2e-http-' + String(httpTrace.length + 1).padStart(4, '0')
    const request = {
      requestId,
      addonId,
      method: String(params.method || 'GET').toUpperCase(),
      path: url.pathname,
      model: body?.model || '',
      messageCount: Array.isArray(body?.messages) ? body.messages.length : 0,
      inputCount: Array.isArray(body?.input) ? body.input.length : body?.input ? 1 : 0
    }
    const userText = Array.isArray(body?.messages)
      ? body.messages.filter((message) => message?.role === 'user').map((message) => String(message.content || '')).join('\n')
      : ''
    const finish = (status, payload) => {
      recordHttpTrace({ ...request, status, responseKind: status >= 200 && status < 300 ? 'json' : 'error' })
      return {
        ok: status >= 200 && status < 300,
        status,
        headers: { 'content-type': 'application/json', 'x-request-id': requestId },
        body: JSON.stringify(payload)
      }
    }

    if (url.pathname === '/v1/models' && request.method === 'GET') {
      return finish(200, {
        object: 'list',
        data: [{ id: 'e2e-chat-model', object: 'model', owned_by: 'elephant-e2e', context_window: 4096 }]
      })
    }
    if (url.pathname === '/v1/embeddings' && request.method === 'POST') {
      const values = Array.isArray(body?.input) ? body.input : [body?.input]
      return finish(200, {
        object: 'list',
        model: String(body?.model || 'e2e-embedding-model'),
        data: values.map((_, index) => ({ object: 'embedding', index, embedding: [0.11, 0.22, 0.33, Number(index) / 100] })),
        usage: { prompt_tokens: values.length * 4, total_tokens: values.length * 4 }
      })
    }
    if (url.pathname === '/v1/chat/completions' && request.method === 'POST') {
      if (userText.includes('[e2e-error]')) {
        return finish(400, { error: { message: 'E2E provider rejected this synthetic request.' } })
      }
      const answer = userText.toLowerCase().includes('alpha')
        ? 'E2E provider response: Alpha note contains the requested alpha context.'
        : 'E2E provider response: automatic answer for “' + userText.trim().slice(0, 80) + '”.'
      return finish(200, {
        id: requestId,
        object: 'chat.completion',
        model: String(body?.model || 'e2e-chat-model'),
        choices: [{ index: 0, message: { role: 'assistant', content: answer }, finish_reason: 'stop' }],
        usage: { prompt_tokens: Math.max(1, request.messageCount * 8), completion_tokens: answer.length, total_tokens: answer.length + request.messageCount * 8 }
      })
    }
    return finish(404, { error: { message: 'E2E provider has no route for ' + request.method + ' ' + request.path + '.' } })
  }

  const nativePlatformKey = () => {
    const os = process.platform === 'darwin' ? 'macos' : process.platform === 'win32' ? 'windows' : process.platform
    const arch = process.arch === 'arm64' ? 'aarch64' : process.arch === 'x64' ? 'x86_64' : process.arch
    return os + '-' + arch
  }

  const nativeResolution = (addonId) => {
    const entry = addonEntry(addonId)
    const manifest = manifestFor(entry)
    const platform = nativePlatformKey()
    const relativePath = manifest.native?.sidecars?.[platform]
    if (!relativePath) throw new Error('No native package for ' + addonId + ' on ' + platform)
    const packageRoot = path.resolve(addonRoot(entry))
    const executable = path.resolve(packageRoot, relativePath)
    if (executable !== packageRoot && !executable.startsWith(packageRoot + path.sep)) {
      throw new Error('Native addon executable escaped its package: ' + addonId)
    }
    if (!fs.existsSync(executable) || !fs.statSync(executable).isFile()) {
      throw new Error('Native addon executable is unavailable: ' + executable)
    }
    return { entry, manifest, packageRoot, executable, relativePath, platform }
  }

  const nativeServiceRequest = (record, method, params = {}, timeoutMs = 120000) => new Promise((resolve, reject) => {
    const id = ++record.nextId
    const timer = setTimeout(() => {
      record.pending.delete(id)
      reject(new Error('Native addon service timed out: ' + record.addonId + '.' + method))
    }, timeoutMs)
    record.pending.set(id, { resolve, reject, timer })
    record.child.stdin.write(JSON.stringify({
      protocol: 'elephant-addon-service-v1',
      id,
      addonId: record.addonId,
      method,
      params
    }) + '\\n', (error) => {
      if (!error) return
      clearTimeout(timer)
      record.pending.delete(id)
      reject(error)
    })
  })

  const startNativeService = (addonId) => {
    const resolved = nativeResolution(addonId)
    const dataDir = path.join(configRoot, 'native-addon-data', addonId)
    fs.mkdirSync(dataDir, { recursive: true })
    const child = childProcess.spawn(resolved.executable, [], {
      cwd: resolved.packageRoot,
      env: {
        ...process.env,
        ELEPHANT_ADDON_ID: addonId,
        ELEPHANT_ADDON_PACKAGE_DIR: resolved.packageRoot,
        ELEPHANT_ADDON_DATA_DIR: dataDir,
        ELEPHANT_VAULT_DIR: vaultRoot(),
        ELEPHANT_ADDON_SERVICE_PROTOCOL: 'elephant-addon-service-v1'
      },
      stdio: ['pipe', 'pipe', 'pipe']
    })
    const record = { addonId, child, nextId: 0, pending: new Map(), resolved }
    const output = readline.createInterface({ input: child.stdout })
    output.on('line', (line) => {
      let response
      try { response = JSON.parse(line) } catch (error) {
        console.error('[e2e-native-addon] invalid response ' + addonId + ': ' + error.message)
        return
      }
      const pending = record.pending.get(response.id)
      if (!pending) return
      record.pending.delete(response.id)
      clearTimeout(pending.timer)
      if (response.protocol !== 'elephant-addon-service-v1') {
        pending.reject(new Error('Invalid native service protocol for ' + addonId))
      } else if (response.ok === true) {
        pending.resolve(response.result)
      } else {
        pending.reject(new Error(response.error?.message || 'Native addon service call failed'))
      }
    })
    child.stderr.on('data', (chunk) => console.warn('[e2e-native-addon:' + addonId + '] ' + String(chunk).trim()))
    const rejectPending = (error) => {
      for (const pending of record.pending.values()) {
        clearTimeout(pending.timer)
        pending.reject(error)
      }
      record.pending.clear()
    }
    child.on('error', (error) => rejectPending(error))
    child.on('exit', (code, signal) => {
      rejectPending(new Error('Native addon service exited: ' + addonId + ' code=' + code + ' signal=' + signal))
      nativeServices.delete(addonId)
      runningServices.delete(addonId)
    })
    nativeServices.set(addonId, record)
    return record
  }

  const nativeService = async (addonId) => {
    let record = nativeServices.get(addonId)
    if (!record || record.child.exitCode !== null) record = startNativeService(addonId)
    if (!record.started) {
      await nativeServiceRequest(record, 'service.start')
      record.started = true
      runningServices.add(addonId)
    }
    return record
  }

  const stopNativeService = async (addonId) => {
    const record = nativeServices.get(addonId)
    if (!record) return { addonId, running: false, mocked: false }
    try { await nativeServiceRequest(record, 'service.stop', {}, 10000) } catch (error) { console.warn('[e2e-native-addon] stop failed ' + addonId + ': ' + error.message) }
    if (record.child.exitCode === null) record.child.kill()
    nativeServices.delete(addonId)
    runningServices.delete(addonId)
    return { addonId, running: false, mocked: false }
  }

  const runNativeSidecar = (addonId, method, params = {}, timeoutMs = 120000) => new Promise((resolve, reject) => {
    const resolved = nativeResolution(addonId)
    const dataDir = path.join(configRoot, 'native-addon-data', addonId)
    fs.mkdirSync(dataDir, { recursive: true })
    const child = childProcess.spawn(resolved.executable, [], {
      cwd: resolved.packageRoot,
      env: {
        ...process.env,
        ELEPHANT_ADDON_ID: addonId,
        ELEPHANT_ADDON_PACKAGE_DIR: resolved.packageRoot,
        ELEPHANT_ADDON_DATA_DIR: dataDir
      },
      stdio: ['pipe', 'pipe', 'pipe']
    })
    const stdout = []
    const stderr = []
    const timer = setTimeout(() => { child.kill(); reject(new Error('Native addon sidecar timed out: ' + addonId + '.' + method)) }, timeoutMs)
    child.stdout.on('data', (chunk) => stdout.push(chunk))
    child.stderr.on('data', (chunk) => stderr.push(chunk))
    child.on('error', (error) => { clearTimeout(timer); reject(error) })
    child.on('close', (code) => {
      clearTimeout(timer)
      if (code !== 0) {
        reject(new Error(String(Buffer.concat(stderr).toString() || 'Native addon sidecar exited with code ' + code).trim()))
        return
      }
      try {
        const response = JSON.parse(Buffer.concat(stdout).toString('utf8'))
        if (response.protocol !== 'elephant-addon-sidecar-v1') throw new Error('Invalid native sidecar protocol for ' + addonId)
        if (response.ok !== true) throw new Error(response.error?.message || 'Native addon sidecar call failed')
        resolve(response.result)
      } catch (error) { reject(error) }
    })
    child.stdin.end(JSON.stringify({
      protocol: 'elephant-addon-sidecar-v1',
      addonId,
      platform: resolved.platform,
      method,
      params
    }))
  })

  const save = () => {
    state.storage = Object.fromEntries(storage)
    fs.mkdirSync(path.dirname(officialAddonStateFile), { recursive: true })
    fs.writeFileSync(officialAddonStateFile, JSON.stringify(state, null, 2))
  }
  const addonEntry = (addonId) => {
    const entry = entries.get(addonId)
    if (!entry) throw new Error('Unknown official addon: ' + addonId)
    return entry
  }
  const addonRoot = (entry) => path.dirname(path.join(officialAddonsRoot, entry.manifestPath))
  const manifestFor = (entry) => {
    const manifest = readJson(path.join(officialAddonsRoot, entry.manifestPath), {})
    return {
      ...manifest,
      id: entry.id,
      name: manifest.name || entry.name,
      version: manifest.version || entry.version,
      source: 'official',
      official: true
    }
  }
  const recordFor = (addonId) => {
    const entry = addonEntry(addonId)
    const manifest = manifestFor(entry)
    const entryRelative = String(manifest.runtime?.entry || path.basename(entry.entryPath || 'main.js'))
    const entryPath = path.resolve(addonRoot(entry), entryRelative)
    const packageHash = crypto.createHash('sha256')
      .update(JSON.stringify(manifest))
      .update(fs.existsSync(entryPath) ? fs.readFileSync(entryPath) : Buffer.from('missing-entry'))
      .digest('hex')
    return {
      manifest,
      source: 'official',
      official: true,
      packageHash,
      installedAt: '2026-07-17T00:00:00.000Z',
      enabled: state.enabled.includes(addonId)
    }
  }
  const safeModulePath = (addonId, modulePath) => {
    const entry = addonEntry(addonId)
    const root = path.resolve(addonRoot(entry))
    const normalized = String(modulePath || '').replaceAll('\\', '/').replace(/^\/+/, '')
    const resolved = path.resolve(root, normalized)
    if (resolved !== root && !resolved.startsWith(root + path.sep)) {
      throw new Error('Official addon module escaped package: ' + addonId + '/' + modulePath)
    }
    return resolved
  }
  const setEnabled = (addonId, enabled) => {
    addonEntry(addonId)
    const next = new Set(state.enabled)
    if (enabled) next.add(addonId)
    else next.delete(addonId)
    state.enabled = [...next].sort()
    memory.enabledAddons.set(addonId, enabled === true)
    save()
    return { ok: true, addonId, enabled: enabled === true }
  }
  const normalizeInstallId = (value) => String(value || '').replace(/^official:/, '').trim()
  const install = (value) => {
    const addonId = normalizeInstallId(value)
    addonEntry(addonId)
    if (!state.installed.includes(addonId)) state.installed.push(addonId)
    state.installed.sort()
    save()
    return recordFor(addonId)
  }
  const uninstall = (addonId) => {
    state.installed = state.installed.filter((id) => id !== addonId)
    state.enabled = state.enabled.filter((id) => id !== addonId)
    runningServices.delete(addonId)
    save()
    return { ok: true, addonId }
  }
  const serviceStatus = (addonId) => ({
    addonId,
    available: true,
    running: runningServices.has(addonId),
    mocked: !realNativeEnabled || addonId === 'elephant.codex-connection',
    evidence: realNativeEnabled && addonId !== 'elephant.codex-connection'
      ? 'Real package-owned native service exercised by the E2E preload.'
      : 'Renderer E2E uses an explicit service mock for this addon.'
  })
  const callBroker = (addonId, method, params = {}) => {
    const intercepted = method === 'http.request' ? interceptedAiRequest(addonId, params) : null
    if (intercepted) return intercepted
    const key = addonId + ':' + String(params.key || '')
    const secretKey = addonId + ':' + String(params.name || '')
    if (method === 'secrets.get') return secrets.get(secretKey) ?? null
    if (method === 'secrets.set') { secrets.set(secretKey, params.value); return { ok: true } }
    if (method === 'secrets.remove') { secrets.delete(secretKey); return { ok: true } }
    if (method === 'storage.get') return storage.get(key) ?? null
    if (method === 'storage.set') { storage.set(key, params.value); save(); return params.value }
    if (method === 'storage.remove') { storage.delete(key); save(); return true }
    if (method === 'storage.entries') {
      return [...storage.entries()]
        .filter(([entryKey]) => entryKey.startsWith(addonId + ':'))
        .map(([entryKey, value]) => [entryKey.slice(addonId.length + 1), value])
    }
    if (method === 'notes.list') {
      const prefix = String(params.prefix || '').replaceAll('\\', '/').replace(/^[/]+|[/]+$/g, '')
      return allMarkdownEntries().filter((entry) => {
        if (!prefix || prefix === '.') return true
        return String(entry.path || '').startsWith(prefix + '/') || String(entry.path || '') === prefix
      })
    }
    if (method === 'notes.read') return readMarkdown(params.path)
    if (method === 'notes.write') return writeMarkdown(params.path, params.content)
    if (addonId === 'elephant.knowledge') {
      const entries = allMarkdownEntries()
      const links = entries.flatMap((entry) => [...readMarkdown(entry.path).matchAll(/\[\[([^\]|#]+)(?:[|#][^\]]*)?\]\]/g)]).length
      if (method === 'knowledge.status') {
        return { documents: entries.length, chunks: entries.length, explicit_links: links }
      }
      if (method === 'knowledge.rebuild') {
        return { indexed: entries.length, unchanged: 0, removed: 0 }
      }
      if (method === 'knowledge.search') {
        const query = String(params.query || '').toLowerCase().trim()
        const tokens = query.split(/[^\p{L}\p{N}_-]+/u).filter((token) => token.length > 1)
        return entries
          .map((entry) => {
            const markdown = readMarkdown(entry.path)
            const haystack = (entry.title + ' ' + markdown).toLowerCase()
            const score = tokens.reduce((total, token) => total + (haystack.includes(token) ? 1 : 0), 0)
            return {
              id: entry.path,
              path: entry.path,
              relativePath: entry.path,
              title: entry.title,
              excerpt: markdown.replace(/\s+/g, ' ').slice(0, 240),
              score,
              engine: 'e2e-knowledge-broker'
            }
          })
          .filter((entry) => entry.score > 0)
          .sort((left, right) => right.score - left.score || left.title.localeCompare(right.title))
          .slice(0, Math.max(1, Number(params.limit || 20)))
      }
      if (method === 'knowledge.embedding.pending') {
        return entries.map((entry) => ({ id: entry.path, relativePath: entry.path, text: readMarkdown(entry.path) }))
      }
      if (method === 'knowledge.embedding.save') {
        const rows = Array.isArray(params.rows) ? params.rows : []
        embeddingState = { documents: rows.length, modelId: String(params.modelId || '') }
        return { written: rows.length, status: embeddingState }
      }
      if (method === 'knowledge.embedding.status') return embeddingState
      if (method === 'knowledge.wiki.list') return []
      if (method === 'knowledge.graph') {
        const nodes = entries.map((entry) => ({
          id: String(entry.path).replace(/\.md$/i, '').toLowerCase(),
          label: entry.title,
          title: entry.title,
          path: entry.path,
          relativePath: entry.path,
          kind: 'note'
        }))
        const aliases = new Map(nodes.flatMap((node) => [
          [node.id, node.id],
          [String(node.title).toLowerCase(), node.id],
          [String(node.path).replace(/\.md$/i, '').toLowerCase(), node.id]
        ]))
        const edges = []
        for (const entry of entries) {
          const source = String(entry.path).replace(/\.md$/i, '').toLowerCase()
          for (const match of readMarkdown(entry.path).matchAll(/\[\[([^\]|#]+)(?:[|#][^\]]*)?\]\]/g)) {
            const target = aliases.get(String(match[1]).trim().replace(/\.md$/i, '').toLowerCase())
            if (target && target !== source) edges.push({
              id: 'link:' + source + '|' + target,
              source,
              target,
              kind: 'link',
              weight: 1
            })
          }
        }
        return { nodes, edges }
      }
    }
    if (method === 'app.info') return { name: 'Elephant', platform: process.platform, e2e: true }
    return { ok: true, method, params, mocked: true }
  }

  save()
  return {
    listInstalled: () => state.installed.map(recordFor),
    listCatalog: () => (catalog.addons || []).map((entry) => ({
      ...entry,
      manifest: manifestFor(entry),
      installed: state.installed.includes(entry.id)
    })),
    install,
    uninstall,
    setEnabled,
    readEntry: (addonId) => {
      const entry = addonEntry(addonId)
      const manifest = manifestFor(entry)
      const entryRelative = String(manifest.runtime?.entry || path.basename(entry.entryPath || 'main.js'))
      const filename = safeModulePath(addonId, entryRelative)
      return { source: fs.readFileSync(filename, 'utf8'), path: entryRelative }
    },
    readModule: (addonId, modulePath) => {
      const filename = safeModulePath(addonId, modulePath)
      return { source: fs.readFileSync(filename, 'utf8'), path: modulePath }
    },
    callBroker,
    serviceStatus,
    serviceStart: async (addonId) => {
      if (realNativeEnabled && addonId !== 'elephant.codex-connection') {
        await nativeService(addonId)
        return serviceStatus(addonId)
      }
      runningServices.add(addonId)
      console.warn('[e2e-addon-service-mock] start ' + addonId)
      return serviceStatus(addonId)
    },
    serviceStop: async (addonId) => {
      if (realNativeEnabled && addonId !== 'elephant.codex-connection') return stopNativeService(addonId)
      runningServices.delete(addonId)
      console.warn('[e2e-addon-service-mock] stop ' + addonId)
      return serviceStatus(addonId)
    },
    serviceCall: async (addonId, method, params) => {
      if (realNativeEnabled && addonId !== 'elephant.codex-connection') {
        const record = await nativeService(addonId)
        return nativeServiceRequest(record, method, params)
      }
      if (addonId === 'elephant.knowledge') return callBroker(addonId, method, params)
      return { ok: true, addonId, method, params, mocked: true }
    },
    sidecarStatus: (addonId) => {
      if (!realNativeEnabled || addonId !== 'elephant.ai-ocr') return { ...serviceStatus(addonId), available: true }
      try {
        const resolved = nativeResolution(addonId)
        return { addonId, available: true, mocked: false, platform: resolved.platform, relativePath: resolved.relativePath }
      } catch (error) {
        return { addonId, available: false, mocked: false, error: error.message }
      }
    },
    sidecarCall: (addonId, method, params) => {
      if (realNativeEnabled && addonId === 'elephant.ai-ocr') return runNativeSidecar(addonId, method, params)
      return { ok: true, addonId, method, params, mocked: true }
    }
  }
})()
`

const addonCases = `    case 'tauri_addons_list':
    case 'tauri_addons_list_full': return officialAddonFixture.listInstalled()
    case 'tauri_addons_catalog_list': return officialAddonFixture.listCatalog()
    case 'tauri_official_addons_catalog_list': return officialAddonFixture.listCatalog()
    case 'tauri_addons_install': return officialAddonFixture.install(params.packagePath)
    case 'tauri_addons_catalog_install':
    case 'tauri_addons_install_catalog':
    case 'tauri_addons_install_official': return officialAddonFixture.install(params.addonId || params.id)
    case 'tauri_addons_uninstall': return officialAddonFixture.uninstall(params.addonId)
    case 'tauri_addons_read_module': return officialAddonFixture.readModule(params.addonId, params.path)
    case 'tauri_addons_call': return officialAddonFixture.callBroker(params.addonId, params.method, params.params || {})
    case 'tauri_addons_notes_list': return officialAddonFixture.callBroker(params.addonId, 'notes.list', { prefix: params.prefix })
    case 'tauri_addons_notes_read': return { path: params.path, markdown: officialAddonFixture.callBroker(params.addonId, 'notes.read', { path: params.path }) }
    case 'tauri_addons_notes_write': return officialAddonFixture.callBroker(params.addonId, 'notes.write', { path: params.path, content: params.markdown })
    case 'tauri_addons_assets_allow_directory': {
      const relativePath = String(params.relativePath || '').replaceAll('\\\\', '/').replace(/^[/]+|[/]+$/g, '')
      const fullPath = resolveVaultPath(relativePath)
      const root = path.resolve(vaultRoot())
      if (!relativePath || (fullPath !== root && !fullPath.startsWith(root + path.sep))) throw new Error('Addon asset path escaped active vault')
      if (!fs.existsSync(fullPath) || !fs.statSync(fullPath).isDirectory()) throw new Error('Addon asset directory does not exist: ' + relativePath)
      return { relativePath, path: fullPath }
    }
    case 'tauri_vault_remove_path': {
      const relativePath = String(params.pathname || params.path || '').replaceAll('\\\\', '/').replace(/^[/]+|[/]+$/g, '')
      const fullPath = resolveVaultPath(relativePath)
      const root = path.resolve(vaultRoot())
      if (!relativePath || fullPath === root || !fullPath.startsWith(root + path.sep)) throw new Error('Addon removal path escaped active vault')
      fs.rmSync(fullPath, { recursive: true, force: true })
      return { ok: true, relativePath }
    }
    case 'tauri_addons_sidecar_status': return officialAddonFixture.sidecarStatus(params.addonId)
    case 'tauri_addons_service_status': return officialAddonFixture.serviceStatus(params.addonId)
    case 'tauri_addons_service_start': return officialAddonFixture.serviceStart(params.addonId)
    case 'tauri_addons_service_stop': return officialAddonFixture.serviceStop(params.addonId)
    case 'tauri_addons_sidecar_call': return officialAddonFixture.sidecarCall(params.addonId, params.method, params.params || {})
    case 'tauri_addons_service_call': return officialAddonFixture.serviceCall(params.addonId, params.method, params.params || {})`

module.exports = (source) => {
  let patched = String(source)
  const helperBoundary = patched.indexOf(INSERT_BEFORE_UPDATE_SIDEBAR)
  if (helperBoundary < 0) throw new Error('Unable to locate initialized preload helper boundary')
  patched = `${patched.slice(0, helperBoundary)}${fixtureSource}\n${patched.slice(helperBoundary)}`
  const addonStart = patched.indexOf("    case 'tauri_addons_list':")
  const addonEnd = patched.indexOf("    case 'tauri_atomic_features_list':", addonStart)
  if (addonStart < 0 || addonEnd < 0) throw new Error('Unable to locate empty addon command cases')
  patched = `${patched.slice(0, addonStart)}${addonCases}\n${patched.slice(addonEnd)}`
  const enabledStart = patched.indexOf("    case 'tauri_addons_set_enabled':")
  const readEntryStart = patched.indexOf("    case 'tauri_addons_read_entry':", enabledStart)
  if (enabledStart < 0 || readEntryStart < 0) throw new Error('Unable to locate addon enabled command cases')
  patched = `${patched.slice(0, enabledStart)}    case 'tauri_addons_set_enabled':\n    case 'tauri_addons_set_enabled_checked': return officialAddonFixture.setEnabled(params.addonId, params.enabled === true)\n${patched.slice(readEntryStart)}`
  const normalizedReadEntryStart = patched.indexOf("    case 'tauri_addons_read_entry':")
  const readEntryEnd = patched.indexOf('    default:', normalizedReadEntryStart)
  if (normalizedReadEntryStart < 0 || readEntryEnd < 0) throw new Error('Unable to locate addon read-entry fallback')
  patched = `${patched.slice(0, normalizedReadEntryStart)}    case 'tauri_addons_read_entry': return officialAddonFixture.readEntry(params.addonId)\n${patched.slice(readEntryEnd)}`
  return patched
}
