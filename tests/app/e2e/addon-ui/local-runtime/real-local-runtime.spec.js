const { test, expect } = require('playwright/test')
const assert = require('node:assert/strict')
const { access, mkdtemp, rm, stat, writeFile } = require('node:fs/promises')
const os = require('node:os')
const path = require('node:path')

const runtimeModules = Promise.all([
  import('../../../../../build/scripts/ensure-test-model.mjs'),
  import('./protocol-client.mjs'),
  import('./ocr-fixture.mjs')
]).then(([model, protocol, fixture]) => ({ ...model, ...protocol, ...fixture }))

const loadRuntime = () => runtimeModules

test.setTimeout(20 * 60 * 1000)

const HERE = __dirname
const ROOT = path.resolve(HERE, '../../../../..')
const NATIVE_ROOT = path.resolve(
  process.env.ELEPHANT_E2E_NATIVE_ROOT ||
  path.join(ROOT, 'addons/official')
)
const BIN = path.resolve(
  process.env.ELEPHANT_E2E_LLAMA_SERVER ||
  path.join(ROOT, 'Elephant/backend/tauri/bin/llama-server')
)
const platformKey = `${process.platform === 'darwin' ? 'macos' : process.platform}-${process.arch === 'arm64' ? 'aarch64' : process.arch}`

const executable = (addon, binary) => path.join(
  NATIVE_ROOT,
  addon,
  'native',
  platformKey,
  binary
)

const OPEN_MODELS_SERVICE = process.env.ELEPHANT_E2E_OPEN_MODELS_SERVICE || executable('open-models', 'elephant-open-models-service')
const OCR_SIDECAR = process.env.ELEPHANT_E2E_OCR_SIDECAR || executable('ai-ocr', 'elephant-ai-ocr')
const KNOWLEDGE_SERVICE = process.env.ELEPHANT_E2E_KNOWLEDGE_SERVICE || executable('knowledge', 'elephant-knowledge-service')

const assertExecutable = async (label, filePath) => {
  try {
    const info = await stat(filePath)
    assert.equal(info.isFile(), true, `[REAL_RUNTIME_BLOCKED][${label}] is not a regular file: ${filePath}`)
  } catch (error) {
    throw new Error(`[REAL_RUNTIME_BLOCKED][${label}] executable is unavailable: ${filePath}; ${error.message}`)
  }
}

const writeEvidence = async (testInfo, evidence) => {
  const outputPath = testInfo.outputPath('real-runtime-evidence.json')
  await writeFile(outputPath, `${JSON.stringify(evidence, null, 2)}\n`)
  return outputPath
}

const ensureSharedModel = async (service, runtime) => runtime.ensureTestModel({
  cacheRoot: runtime.modelCacheRoot(),
  listModels: () => service.call('models.list', {}, 30_000).then((result) => result.models || []),
  download: (params) => service.call('models.download', params, 15 * 60 * 1000)
})

const startOpenModelsService = async ({ port, cacheRoot, runtime } = {}) => {
  const service = new runtime.JsonLineService({
    addonId: 'elephant.open-models',
    executable: OPEN_MODELS_SERVICE,
    env: {
      ELEPHANT_ADDON_DATA_DIR: runtime.openModelsDataDir(cacheRoot || runtime.modelCacheRoot()),
      ELEPHANT_LLAMA_SERVER_PATH: BIN,
      ELEPHANT_LLAMA_BASE_URL: `http://127.0.0.1:${port}/v1`
    },
    cwd: ROOT
  })
  await service.call('service.start', {}, 30_000)
  return service
}

test('elephant.open-models starts the real service, discovers the cached GGUF and answers through llama-server', async ({ browserName }, testInfo) => {
  void browserName
  const runtime = await loadRuntime()
  const { TEST_MODEL } = runtime
  await assertExecutable('elephant.open-models service', OPEN_MODELS_SERVICE)
  await assertExecutable('llama-server', BIN)
  const port = await runtime.reservePort()
  const evidence = {
    addonId: 'elephant.open-models',
    executable: OPEN_MODELS_SERVICE,
    llamaServer: BIN,
    model: TEST_MODEL,
    port,
    cacheRoot: runtime.modelCacheRoot(),
    events: []
  }
  let service
  let baseUrl = ''
  try {
    service = await startOpenModelsService({ port, runtime })
    const initialStatus = await service.call('models.status')
    assert.equal(initialStatus.owner, 'elephant.open-models')
    assert.equal(initialStatus.serverRunning, false)
    await assert.rejects(
      service.call('models.download', { id: 'https://127.0.0.1:1/elephant-integrity-error.gguf' }, 10_000),
      /download request failed|Model download returned HTTP|error/i
    )
    const failedDownloadStatus = await service.call('models.status')
    assert.equal(failedDownloadStatus.owner, 'elephant.open-models')
    assert.equal(failedDownloadStatus.serverRunning, false)
    const modelsAfterFailedDownload = await service.call('models.list')
    assert.equal(
      (modelsAfterFailedDownload.models || []).some((model) => model.fileName === 'elephant-integrity-error.gguf'),
      false,
      'a failed model download must not materialize a GGUF in the persistent cache'
    )
    evidence.events.push({ event: 'download.failure-visible', result: failedDownloadStatus, models: modelsAfterFailedDownload })
    evidence.events.push({ event: 'status.before-model', result: initialStatus })

    const first = await ensureSharedModel(service, runtime)
    const second = await ensureSharedModel(service, runtime)
    assert.ok(first.model.path.endsWith('.gguf'))
    assert.equal(first.model.fileName, TEST_MODEL.fileName)
    assert.equal(second.downloaded, false, 'the second ensure call must reuse the persistent cache')
    assert.equal(second.model.sha256, first.model.sha256)
    assert.equal(second.model.size, first.model.size)
    evidence.events.push({ event: 'model.cache', first, second })

    const models = await service.call('models.list')
    const discovered = (models.models || []).find((model) => model.path === first.model.path)
    assert.ok(discovered, 'the real Open Models service must discover the downloaded GGUF')
    assert.equal(discovered.status, 'downloaded')
    evidence.events.push({ event: 'models.list', result: models })

    const active = await service.call('models.activate', { id: discovered.id })
    assert.equal(active.path, first.model.path)
    evidence.events.push({ event: 'models.activate', result: active })

    const chat = await service.call('models.chat', {
      model: discovered.id,
      messages: [{ role: 'user', content: 'Reply with a short greeting.' }],
      route: { contextWindow: 512, maxTokens: 24, temperature: 0 }
    }, 180_000)
    assert.equal(chat.provider, 'app-local')
    assert.equal(chat.model, discovered.fileName)
    assert.ok(String(chat.answer || '').trim().length > 0, 'real llama-server response must contain text')
    baseUrl = chat.baseUrl
    evidence.events.push({ event: 'models.chat', result: chat })

    const afterChat = await service.call('models.status')
    assert.equal(afterChat.serverRunning, true)
    assert.equal(afterChat.serverModelPath, first.model.path)
    const llamaModels = await runtime.fetchJson(`${baseUrl}/models`)
    assert.ok((llamaModels.payload?.data || []).some((entry) => entry.id === discovered.fileName))
    evidence.events.push({ event: 'status.after-chat', result: afterChat, llamaModels: llamaModels.payload })
  } catch (error) {
    evidence.error = error.message
    throw error
  } finally {
    if (service) await service.stop()
    if (baseUrl) await runtime.assertHttpUnavailable(`${baseUrl}/models`)
    evidence.cleanup = { serviceStopped: true, llamaServerStopped: Boolean(baseUrl) }
    evidence.artifact = await writeEvidence(testInfo, evidence)
  }
})

test('elephant.ai-ocr starts the real sidecar and recognizes text with the installed Tesseract runtime', async ({ browserName }, testInfo) => {
  void browserName
  const runtime = await loadRuntime()
  await assertExecutable('elephant.ai-ocr sidecar', OCR_SIDECAR)
  const fixturePath = testInfo.outputPath('elephant-ocr-fixture.png')
  await runtime.writeOcrFixture(fixturePath)
  await access(fixturePath)
  const evidence = {
    addonId: 'elephant.ai-ocr',
    executable: OCR_SIDECAR,
    fixturePath,
    model: { used: false, reason: 'manifest declares a Tesseract process sidecar; no GGUF model is part of the OCR production path' },
    events: []
  }
  try {
    const status = await runtime.oneShotSidecar({
      addonId: 'elephant.ai-ocr',
      executable: OCR_SIDECAR,
      method: 'status',
      cwd: ROOT
    })
    assert.equal(status.ok, true)
    assert.equal(status.result?.available, true, `[REAL_RUNTIME_BLOCKED][elephant.ai-ocr] Tesseract is unavailable: ${JSON.stringify(status.result)}`)
    assert.equal(status.result?.engine, 'tesseract')
    evidence.events.push({ event: 'sidecar.status', result: status.result })

    const recognition = await runtime.oneShotSidecar({
      addonId: 'elephant.ai-ocr',
      executable: OCR_SIDECAR,
      method: 'ocr.image',
      params: { path: fixturePath, languages: 'eng', output: 'plain-text' },
      cwd: ROOT
    })
    const text = String(recognition.result?.text || '').replace(/\s+/g, ' ').trim().toUpperCase()
    expect(text).toContain('ELEPHANT')
    expect(text.length).toBeGreaterThan(0)
    assert.equal(recognition.result?.source, 'tesseract')
    evidence.events.push({ event: 'sidecar.ocr.image', result: recognition.result })
  } catch (error) {
    evidence.error = error.message
    throw error
  } finally {
    evidence.artifact = await writeEvidence(testInfo, evidence)
  }
})

test('elephant.knowledge indexes the real vault and stores vectors returned by the cached local model', async ({ browserName }, testInfo) => {
  void browserName
  const runtime = await loadRuntime()
  await assertExecutable('elephant.open-models service', OPEN_MODELS_SERVICE)
  await assertExecutable('elephant.knowledge service', KNOWLEDGE_SERVICE)
  await assertExecutable('llama-server', BIN)
  const vault = await mkdtemp(path.join(os.tmpdir(), 'elephant-knowledge-local-runtime-'))
  const evidence = {
    addonId: 'elephant.knowledge',
    executable: KNOWLEDGE_SERVICE,
    llamaServer: BIN,
    vault,
    events: []
  }
  let knowledge
  let embeddingServer
  let modelService
  try {
    await writeFile(path.join(vault, 'Alpha.md'), '# Alpha\n\nAlpha contains the local runtime marker and links to [[Beta]].\n')
    await writeFile(path.join(vault, 'Beta.md'), '# Beta\n\nBeta is the second real Knowledge document.\n')
    knowledge = new runtime.JsonLineService({
      addonId: 'elephant.knowledge',
      executable: KNOWLEDGE_SERVICE,
      env: { ELEPHANT_VAULT_DIR: vault },
      cwd: ROOT
    })
    const before = await knowledge.call('service.start', {}, 30_000)
    assert.equal(before.documents, 0)
    evidence.events.push({ event: 'knowledge.status.before-rebuild', result: before })

    const rebuild = await knowledge.call('knowledge.rebuild', {}, 180_000)
    assert.equal(rebuild.scanned, 2)
    assert.equal(rebuild.indexed, 2)
    assert.deepEqual(rebuild.failed, [])
    const status = await knowledge.call('knowledge.status')
    assert.equal(status.documents, 2)
    assert.ok(status.chunks >= 2)
    assert.ok((await stat(path.join(vault, '.elephantnote/knowledge/knowledge.sqlite'))).isFile())
    evidence.events.push({ event: 'knowledge.rebuild', result: rebuild, status })

    const search = await knowledge.call('knowledge.search', { query: 'local runtime marker', limit: 10 })
    assert.ok(search.some((hit) => hit.relative_path === 'Alpha.md'))
    evidence.events.push({ event: 'knowledge.search', result: search })

    modelService = await startOpenModelsService({ port: await runtime.reservePort(), runtime })
    const cached = await ensureSharedModel(modelService, runtime)
    await modelService.stop()
    modelService = null
    evidence.model = cached

    const pending = await knowledge.call('knowledge.embedding.pending', {
      modelId: cached.model.fileName,
      limit: 10
    })
    assert.equal(pending.length, 2)

    const embeddingPort = await runtime.reservePort()
    embeddingServer = await runtime.spawnLlamaServer({
      executable: BIN,
      modelPath: cached.model.path,
      alias: cached.model.fileName,
      port: embeddingPort,
      pooling: 'mean',
      cwd: ROOT
    })
    const discovered = await runtime.fetchJson(`${embeddingServer.baseUrl}/models`)
    assert.ok((discovered.payload?.data || []).some((entry) => entry.id === cached.model.fileName))
    const embeddings = await runtime.fetchJson(`${embeddingServer.baseUrl}/embeddings`, {
      method: 'POST',
      body: { model: cached.model.fileName, input: pending.map((entry) => entry.text) },
      timeoutMs: 180_000
    })
    const vectorEntries = [...(embeddings.payload?.data || [])].sort((left, right) => Number(left.index || 0) - Number(right.index || 0))
    const vectors = vectorEntries.map((entry) => entry.embedding)
    assert.equal(vectors.length, pending.length, 'real llama-server must return one embedding per Knowledge input')
    assert.ok(vectors.every((vector) => Array.isArray(vector) && vector.length > 0 && vector.every(Number.isFinite)))
    evidence.events.push({ event: 'llama.embeddings', model: cached.model.fileName, discovery: discovered.payload, dimensions: vectors[0].length })

    const saved = await knowledge.call('knowledge.embedding.save', {
      modelId: cached.model.fileName,
      threshold: 0,
      rows: pending.map((input, index) => ({ input, vector: vectors[index] }))
    }, 180_000)
    assert.equal(saved.written, pending.length)
    const embeddingStatus = await knowledge.call('knowledge.embedding.status')
    assert.equal(embeddingStatus.documents, 2)
    assert.equal(embeddingStatus.modelId, cached.model.fileName)
    assert.ok(embeddingStatus.dimensions > 0)
    evidence.events.push({ event: 'knowledge.embedding.save', result: saved, status: embeddingStatus })
  } catch (error) {
    evidence.error = error.message
    throw error
  } finally {
    if (embeddingServer) await embeddingServer.stop()
    if (modelService) await modelService.stop()
    if (knowledge) await knowledge.stop()
    evidence.cleanup = { embeddingServerStopped: Boolean(embeddingServer), knowledgeStopped: Boolean(knowledge) }
    evidence.artifact = await writeEvidence(testInfo, evidence)
    await rm(vault, { recursive: true, force: true })
  }
})
