import { spawn } from 'node:child_process'
import { createServer } from 'node:net'
import path from 'node:path'

const SERVICE_PROTOCOL = 'elephant-addon-service-v1'
const SIDECAR_PROTOCOL = 'elephant-addon-sidecar-v1'

const sleep = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds))

const tail = (value, limit = 4_000) => String(value || '').slice(-limit)

export class JsonLineService {
  constructor({ addonId, executable, env = {}, cwd = process.cwd() }) {
    this.addonId = addonId
    this.executable = executable
    this.stderr = ''
    this.pending = new Map()
    this.nextId = 1
    this.buffer = ''
    this.child = spawn(executable, [], {
      cwd,
      env: { ...process.env, ...env },
      stdio: ['pipe', 'pipe', 'pipe']
    })
    this.exitPromise = new Promise((resolve) => {
      this.child.once('close', (code, signal) => {
        this.closed = true
        const error = new Error(
          `[REAL_RUNTIME_BLOCKED][${this.addonId}] service exited before responding (code=${code}, signal=${signal}, stderr=${tail(this.stderr)})`
        )
        for (const { reject, timer } of this.pending.values()) {
          clearTimeout(timer)
          reject(error)
        }
        this.pending.clear()
        resolve({ code, signal })
      })
    })
    this.child.stdout.setEncoding('utf8')
    this.child.stdout.on('data', (chunk) => this.consume(chunk))
    this.child.stderr.setEncoding('utf8')
    this.child.stderr.on('data', (chunk) => { this.stderr += chunk })
  }

  consume(chunk) {
    this.buffer += chunk
    while (true) {
      const newline = this.buffer.indexOf('\n')
      if (newline < 0) return
      const raw = this.buffer.slice(0, newline).trim()
      this.buffer = this.buffer.slice(newline + 1)
      if (!raw) continue
      let response
      try {
        response = JSON.parse(raw)
      } catch (error) {
        for (const { reject, timer } of this.pending.values()) {
          clearTimeout(timer)
          reject(new Error(`[REAL_RUNTIME_BLOCKED][${this.addonId}] invalid JSON from service: ${error.message}; raw=${raw}`))
        }
        this.pending.clear()
        continue
      }
      const pending = this.pending.get(Number(response?.id))
      if (!pending) continue
      this.pending.delete(Number(response.id))
      clearTimeout(pending.timer)
      if (response.protocol !== SERVICE_PROTOCOL) {
        pending.reject(new Error(`[REAL_RUNTIME_BLOCKED][${this.addonId}] invalid service protocol: ${response.protocol}`))
      } else if (response.ok !== true) {
        const message = response?.error?.message || JSON.stringify(response.error || response)
        pending.reject(new Error(`[REAL_RUNTIME_BLOCKED][${this.addonId}] ${pending.method} failed: ${message}; stderr=${tail(this.stderr)}`))
      } else {
        pending.resolve(response.result)
      }
    }
  }

  call(method, params = {}, timeoutMs = 30_000) {
    if (this.closed || this.child.exitCode !== null) {
      return Promise.reject(new Error(`[REAL_RUNTIME_BLOCKED][${this.addonId}] cannot call ${method}: service is not running`))
    }
    const id = this.nextId++
    const request = `${JSON.stringify({
      protocol: SERVICE_PROTOCOL,
      id,
      addonId: this.addonId,
      method,
      params
    })}\n`
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(id)
        reject(new Error(`[REAL_RUNTIME_BLOCKED][${this.addonId}] ${method} timed out after ${timeoutMs} ms; stderr=${tail(this.stderr)}`))
      }, timeoutMs)
      this.pending.set(id, { method, resolve, reject, timer })
      this.child.stdin.write(request, (error) => {
        if (!error) return
        clearTimeout(timer)
        this.pending.delete(id)
        reject(new Error(`[REAL_RUNTIME_BLOCKED][${this.addonId}] failed to write ${method}: ${error.message}`))
      })
    })
  }

  async stop() {
    let stopError = null
    if (!this.closed && this.child.exitCode === null) {
      try {
        await this.call('service.stop', {}, 5_000)
      } catch (error) {
        stopError = error
      }
    }
    if (!this.closed && this.child.exitCode === null) {
      this.child.kill('SIGTERM')
    }
    await Promise.race([this.exitPromise, sleep(6_000)])
    if (!this.closed && this.child.exitCode === null) this.child.kill('SIGKILL')
    if (stopError) throw stopError
  }
}

export const oneShotSidecar = ({ addonId, executable, method, params = {}, env = {}, cwd = process.cwd() }) => new Promise((resolve, reject) => {
  const child = spawn(executable, [], {
    cwd,
    env: { ...process.env, ...env },
    stdio: ['pipe', 'pipe', 'pipe']
  })
  let stdout = ''
  let stderr = ''
  child.stdout.setEncoding('utf8')
  child.stderr.setEncoding('utf8')
  child.stdout.on('data', (chunk) => { stdout += chunk })
  child.stderr.on('data', (chunk) => { stderr += chunk })
  child.once('error', reject)
  child.once('close', (code, signal) => {
    if (code !== 0) {
      reject(new Error(`[REAL_RUNTIME_BLOCKED][${addonId}] sidecar exited code=${code}, signal=${signal}, stderr=${tail(stderr)}`))
      return
    }
    try {
      const response = JSON.parse(stdout.trim())
      if (response.protocol !== SIDECAR_PROTOCOL) throw new Error(`invalid sidecar protocol ${response.protocol}`)
      if (response.ok !== true) {
        throw new Error(response?.error?.message || JSON.stringify(response.error || response))
      }
      resolve(response)
    } catch (error) {
      reject(new Error(`[REAL_RUNTIME_BLOCKED][${addonId}] ${method} failed: ${error.message}; stdout=${tail(stdout)}; stderr=${tail(stderr)}`))
    }
  })
  child.stdin.end(`${JSON.stringify({
    protocol: SIDECAR_PROTOCOL,
    addonId,
    method,
    params
  })}\n`)
})

export const reservePort = () => new Promise((resolve, reject) => {
  const server = createServer()
  server.once('error', reject)
  server.listen(0, '127.0.0.1', () => {
    const port = server.address().port
    server.close((error) => error ? reject(error) : resolve(port))
  })
})

export const fetchJson = async (url, { method = 'GET', body, timeoutMs = 10_000 } = {}) => {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), timeoutMs)
  try {
    const response = await fetch(url, {
      method,
      headers: body === undefined ? undefined : { 'content-type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body),
      signal: controller.signal
    })
    const text = await response.text()
    let payload = null
    try { payload = text ? JSON.parse(text) : null } catch { payload = text }
    if (!response.ok) {
      throw new Error(`[REAL_RUNTIME_BLOCKED] HTTP ${response.status} ${response.statusText} from ${url}: ${typeof payload === 'string' ? payload : JSON.stringify(payload)}`)
    }
    return { response, payload }
  } catch (error) {
    if (error?.name === 'AbortError') throw new Error(`[REAL_RUNTIME_BLOCKED] HTTP timeout after ${timeoutMs} ms: ${url}`)
    throw error
  } finally {
    clearTimeout(timer)
  }
}

export const waitForJson = async (url, timeoutMs = 30_000) => {
  const startedAt = Date.now()
  let lastError = null
  while (Date.now() - startedAt < timeoutMs) {
    try {
      return await fetchJson(url, { timeoutMs: 2_000 })
    } catch (error) {
      lastError = error
      await sleep(250)
    }
  }
  throw new Error(`[REAL_RUNTIME_BLOCKED] service did not expose ${url} within ${timeoutMs} ms: ${lastError?.message || 'unknown error'}`)
}

export const spawnLlamaServer = async ({ executable, modelPath, alias, port, pooling = 'mean', cwd = process.cwd() }) => {
  const args = [
    '-m', modelPath,
    '--host', '127.0.0.1',
    '--port', String(port),
    '-c', '512',
    '--alias', alias,
    '--pooling', pooling,
    '--embeddings'
  ]
  const child = spawn(executable, args, {
    cwd,
    env: process.env,
    stdio: ['ignore', 'ignore', 'pipe']
  })
  let stderr = ''
  child.stderr.setEncoding('utf8')
  child.stderr.on('data', (chunk) => { stderr += chunk })
  const baseUrl = `http://127.0.0.1:${port}/v1`
  try {
    await waitForJson(`${baseUrl}/models`, 120_000)
  } catch (error) {
    child.kill('SIGKILL')
    throw new Error(`${error.message}; llama-server=${executable}; model=${path.basename(modelPath)}; stderr=${tail(stderr)}`)
  }
  return {
    baseUrl,
    stderr: () => tail(stderr),
    async stop() {
      if (child.exitCode !== null) return
      child.kill('SIGTERM')
      await Promise.race([
        new Promise((resolve) => child.once('close', resolve)),
        sleep(5_000)
      ])
      if (child.exitCode === null) child.kill('SIGKILL')
    }
  }
}

export const assertHttpUnavailable = async (url) => {
  try {
    const response = await fetch(url, { signal: AbortSignal.timeout(2_000) })
    throw new Error(`[REAL_RUNTIME_BLOCKED] expected runtime shutdown at ${url}, received HTTP ${response.status}`)
  } catch (error) {
    if (error.message.startsWith('[REAL_RUNTIME_BLOCKED] expected runtime shutdown')) throw error
  }
}
