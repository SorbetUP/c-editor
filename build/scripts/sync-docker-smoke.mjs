import { execFile } from 'child_process'
import { setTimeout as delay } from 'timers/promises'
import { promisify } from 'util'

const execFileAsync = promisify(execFile)
const image = process.env.ELEPHANTNOTE_DOCKER_IMAGE || 'elephantnote-web-sync-smoke'
const port = Number(process.env.PORT || 18787 + Math.floor(Math.random() * 1000))
const name = `elephantnote-sync-smoke-${process.pid}`

const run = async(command, args, options = {}) => {
  const result = await execFileAsync(command, args, {
    maxBuffer: 1024 * 1024 * 20,
    ...options
  })
  return result.stdout
}

const request = async(pathname, options = {}) => {
  const response = await fetch(`http://127.0.0.1:${port}${pathname}`, {
    headers: { 'content-type': 'application/json' },
    ...options,
    body: options.body ? JSON.stringify(options.body) : undefined
  })
  const body = await response.json()
  if (!response.ok) throw new Error(`${pathname} failed with HTTP ${response.status}: ${JSON.stringify(body)}`)
  return body
}

const waitForServer = async() => {
  const deadline = Date.now() + 30000
  while (Date.now() < deadline) {
    try {
      await request('/api/sync/status')
      return
    } catch {
      await delay(500)
    }
  }
  throw new Error('Docker smoke server did not become ready.')
}

try {
  await run('docker', ['build', '-t', image, '.'])
  await run('docker', [
    'run',
    '--rm',
    '-d',
    '--name',
    name,
    '-p',
    `${port}:8787`,
    image
  ])
  await waitForServer()

  await request('/api/notes', {
    method: 'POST',
    body: { title: 'Docker Sync Smoke', body: 'Created by the sync Docker smoke test.' }
  })
  const status = await request('/api/sync/run', {
    method: 'POST',
    body: {
      snapshot: {
        message: 'Docker sync smoke snapshot'
      }
    }
  })
  const notes = await request('/api/notes')

  if (status.queued !== 0) throw new Error(`Expected an empty queue, got ${status.queued}.`)
  if (status.dirty) throw new Error('Expected the vault to be clean after snapshot.')
  if (!status.deviceId || !status.folderId) throw new Error('Expected sync identity fields in Docker status.')
  if (!status.history?.some((item) => item.operation === 'snapshot' && item.status === 'done')) {
    throw new Error('Expected a completed snapshot operation in Docker history.')
  }
  if (!notes.some((note) => note.path === 'Docker Sync Smoke.md')) {
    throw new Error('Expected the smoke note to be listed after sync.')
  }

  console.log(JSON.stringify({
    ok: true,
    image,
    port,
    deviceId: status.deviceId,
    folderId: status.folderId,
    history: status.history.map((item) => `${item.operation}:${item.status}`)
  }, null, 2))
} finally {
  await execFileAsync('docker', ['rm', '-f', name]).catch(() => {})
}

