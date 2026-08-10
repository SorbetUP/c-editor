import { createHash } from 'node:crypto'
import { open, readFile, stat, writeFile, rename, rm, mkdir } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'

export const TEST_MODEL = Object.freeze({
  repoId: 'bartowski/SmolLM2-135M-Instruct-GGUF',
  fileName: 'SmolLM2-135M-Instruct-Q4_K_M.gguf',
  url: 'https://huggingface.co/bartowski/SmolLM2-135M-Instruct-GGUF/resolve/main/SmolLM2-135M-Instruct-Q4_K_M.gguf?download=1'
})

const MIN_MODEL_BYTES = 1_000_000
const LOCK_WAIT_MS = 250
const LOCK_TIMEOUT_MS = 15 * 60 * 1000
const STALE_LOCK_MS = 30 * 60 * 1000

export const modelCacheRoot = () => (
  process.env.ELEPHANT_E2E_LOCAL_RUNTIME_CACHE ||
  path.join(process.env.XDG_CACHE_HOME || path.join(os.homedir(), '.cache'), 'elephant-e2e', 'local-runtime')
)

export const openModelsDataDir = (cacheRoot = modelCacheRoot()) => path.join(cacheRoot, 'open-models')

const modelManifestPath = (cacheRoot) => path.join(cacheRoot, 'model-cache.json')
const modelLockPath = (cacheRoot) => path.join(cacheRoot, '.model-download.lock')

const matchingModel = (models = []) => models.find((model) => (
  String(model?.fileName || '').toLowerCase() === TEST_MODEL.fileName.toLowerCase() &&
  (!model?.repoId || model.repoId === TEST_MODEL.repoId)
))

const isGguf = async (modelPath) => {
  const info = await stat(modelPath)
  if (!info.isFile() || info.size < MIN_MODEL_BYTES) return false
  const handle = await open(modelPath, 'r')
  try {
    const header = Buffer.alloc(4)
    await handle.read(header, 0, header.length, 0)
    return header.toString('ascii') === 'GGUF'
  } finally {
    await handle.close()
  }
}

const sha256 = async (modelPath) => {
  const bytes = await readFile(modelPath)
  return createHash('sha256').update(bytes).digest('hex')
}

const validateModel = async (model, stage) => {
  const modelPath = path.resolve(String(model?.path || ''))
  if (!modelPath || !(await isGguf(modelPath).catch(() => false))) {
    throw new Error(
      `[REAL_RUNTIME_BLOCKED][open-models][${stage}] service returned a model that is not a valid GGUF file: ${modelPath}`
    )
  }
  return {
    ...model,
    path: modelPath,
    size: (await stat(modelPath)).size,
    sha256: await sha256(modelPath)
  }
}

const readManifest = async (cacheRoot) => {
  try {
    return JSON.parse(await readFile(modelManifestPath(cacheRoot), 'utf8'))
  } catch {
    return {}
  }
}

const writeManifest = async (cacheRoot, value) => {
  const target = modelManifestPath(cacheRoot)
  const temporary = `${target}.${process.pid}.tmp`
  await writeFile(temporary, `${JSON.stringify(value, null, 2)}\n`, { mode: 0o600 })
  await rename(temporary, target)
}

const sleep = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds))

const acquireDownloadLock = async (cacheRoot) => {
  const lockPath = modelLockPath(cacheRoot)
  const startedAt = Date.now()
  await mkdir(cacheRoot, { recursive: true })
  while (Date.now() - startedAt < LOCK_TIMEOUT_MS) {
    try {
      const handle = await open(lockPath, 'wx')
      await handle.writeFile(JSON.stringify({ pid: process.pid, startedAt: new Date().toISOString() }))
      await handle.close()
      return async () => { await rm(lockPath, { force: true }) }
    } catch (error) {
      if (error?.code !== 'EEXIST') throw error
      const lockInfo = await stat(lockPath).catch(() => null)
      if (lockInfo && Date.now() - lockInfo.mtimeMs > STALE_LOCK_MS) {
        await rm(lockPath, { force: true })
        continue
      }
      await sleep(LOCK_WAIT_MS)
    }
  }
  throw new Error(`[REAL_RUNTIME_BLOCKED][open-models] model download lock was not released after ${LOCK_TIMEOUT_MS} ms: ${lockPath}`)
}

const inspectCachedModel = async (listModels, stage) => {
  const model = matchingModel(await listModels())
  return model ? validateModel(model, stage) : null
}

export const ensureTestModel = async ({
  cacheRoot = modelCacheRoot(),
  listModels,
  download
} = {}) => {
  if (typeof listModels !== 'function' || typeof download !== 'function') {
    throw new TypeError('ensureTestModel requires the real Open Models service listModels and download callbacks')
  }
  await mkdir(openModelsDataDir(cacheRoot), { recursive: true })

  const beforeLock = await inspectCachedModel(listModels, 'before-cache-lock')
  if (beforeLock) {
    const manifest = await readManifest(cacheRoot)
    return {
      model: beforeLock,
      downloaded: false,
      reused: true,
      cacheRoot,
      manifestPath: modelManifestPath(cacheRoot),
      downloadCount: Number(manifest.downloadCount || 0)
    }
  }

  const release = await acquireDownloadLock(cacheRoot)
  try {
    const existing = await inspectCachedModel(listModels, 'after-cache-lock')
    if (existing) {
      const manifest = await readManifest(cacheRoot)
      return {
        model: existing,
        downloaded: false,
        reused: true,
        cacheRoot,
        manifestPath: modelManifestPath(cacheRoot),
        downloadCount: Number(manifest.downloadCount || 0)
      }
    }

    const downloaded = await download({ id: TEST_MODEL.url })
    const model = await inspectCachedModel(listModels, 'after-download')
    if (!model) {
      throw new Error(
        `[REAL_RUNTIME_BLOCKED][open-models][after-download] models.download completed without discoverable ${TEST_MODEL.repoId}/${TEST_MODEL.fileName}; response=${JSON.stringify(downloaded)}`
      )
    }
    const previous = await readManifest(cacheRoot)
    const nextManifest = {
      ...previous,
      source: TEST_MODEL,
      path: model.path,
      size: model.size,
      sha256: model.sha256,
      downloadedAt: new Date().toISOString(),
      downloadCount: Number(previous.downloadCount || 0) + 1
    }
    await writeManifest(cacheRoot, nextManifest)
    return {
      model,
      downloaded: true,
      reused: false,
      cacheRoot,
      manifestPath: modelManifestPath(cacheRoot),
      downloadCount: nextManifest.downloadCount
    }
  } finally {
    await release()
  }
}
