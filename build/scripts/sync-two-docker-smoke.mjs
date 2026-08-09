import { execFile } from 'child_process'
import { setTimeout as delay } from 'timers/promises'
import { promisify } from 'util'

const execFileAsync = promisify(execFile)
const image = process.env.ELEPHANTNOTE_DOCKER_IMAGE || 'elephantnote-web-sync-pair-smoke'
const basePort = Number(process.env.PORT || 19787 + Math.floor(Math.random() * 1000))
const prefix = `elephantnote-sync-pair-${process.pid}-${Date.now()}`
const network = `${prefix}-net`
const remoteVolume = `${prefix}-remote`
const vaultA = `${prefix}-vault-a`
const vaultB = `${prefix}-vault-b`
const deviceA = `${prefix}-a`
const deviceB = `${prefix}-b`
const remotePath = '/git/elephantnote.git'
const autoSyncIntervalMs = Number(process.env.ELEPHANTNOTE_SYNC_SMOKE_AUTO_INTERVAL_MS || 1000)
const maxContainerMemoryMiB = Number(process.env.ELEPHANTNOTE_SYNC_SMOKE_MAX_CONTAINER_MIB || 256)
const maxEndToEndMs = Number(process.env.ELEPHANTNOTE_SYNC_SMOKE_MAX_TOTAL_MS || 120000)
const startedAt = Date.now()

const resources = {
  containers: [deviceA, deviceB],
  volumes: [remoteVolume, vaultA, vaultB],
  network
}

const run = async(command, args, options = {}) => {
  try {
    const result = await execFileAsync(command, args, {
      maxBuffer: 1024 * 1024 * 20,
      ...options
    })
    return result.stdout
  } catch (error) {
    const details = [
      `${command} ${args.join(' ')} failed`,
      error.stdout ? `stdout:\n${error.stdout}` : '',
      error.stderr ? `stderr:\n${error.stderr}` : '',
      error.message || ''
    ].filter(Boolean).join('\n')
    throw new Error(details)
  }
}

const docker = (args, options) => run('docker', args, options)

const request = async(port, pathname, options = {}) => {
  const response = await fetch(`http://127.0.0.1:${port}${pathname}`, {
    headers: { 'content-type': 'application/json' },
    ...options,
    body: options.body ? JSON.stringify(options.body) : undefined
  })
  const body = await response.json()
  if (!response.ok) throw new Error(`${pathname} failed with HTTP ${response.status}: ${JSON.stringify(body)}`)
  return body
}

const waitForServer = async(port, label) => {
  const deadline = Date.now() + 45000
  while (Date.now() < deadline) {
    try {
      await request(port, '/api/sync/status')
      return
    } catch {
      await delay(500)
    }
  }
  throw new Error(`${label} did not become ready.`)
}

const waitForNote = async(port, notePath, label, timeoutMs = 45000) => {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    const notes = await request(port, '/api/notes').catch(() => [])
    if (notes.some((note) => note.path === notePath)) return notes
    await delay(500)
  }
  throw new Error(`${label} did not auto-detect ${notePath} before timeout.`)
}

const startDevice = async(name, port, vaultVolume, env = {}) => {
  const envArgs = Object.entries(env).flatMap(([key, value]) => ['-e', `${key}=${value}`])
  await docker([
    'run',
    '--rm',
    '-d',
    '--name',
    name,
    '--network',
    network,
    '-p',
    `${port}:8787`,
    '-v',
    `${vaultVolume}:/data/vault`,
    '-v',
    `${remoteVolume}:/git`,
    ...envArgs,
    image
  ])
}

const startAutoPullDevice = async(name, port, vaultVolume, branch = '') => startDevice(name, port, vaultVolume, {
  ELEPHANTNOTE_SYNC_REMOTE: remotePath,
  ELEPHANTNOTE_SYNC_BRANCH: branch,
  ELEPHANTNOTE_SYNC_AUTO_INTERVAL_MS: autoSyncIntervalMs,
  ELEPHANTNOTE_SYNC_AUTO_MODE: 'pull'
})

const stopDevice = async(name) => {
  await execFileAsync('docker', ['rm', '-f', name]).catch(() => {})
}

const restartDeviceWithoutAutoSync = async(name, port, vaultVolume) => {
  await stopDevice(name)
  await startDevice(name, port, vaultVolume)
  await waitForServer(port, `${name} manual restart`)
}

const assertNoteListed = (notes, notePath, label) => {
  if (!notes.some((note) => note.path === notePath)) {
    throw new Error(`${label} did not list ${notePath}. Notes: ${JSON.stringify(notes)}`)
  }
}

const assertHistory = (status, operation, label) => {
  if (!status.history?.some((item) => item.operation === operation && item.status === 'done')) {
    throw new Error(`${label} expected completed ${operation}. History: ${JSON.stringify(status.history)}`)
  }
}

const assertClean = (status, label) => {
  if (status.queued !== 0) throw new Error(`${label} expected empty queue, got ${status.queued}.`)
  if (status.dirty) throw new Error(`${label} expected clean git tree.`)
  if (!status.deviceId || !status.folderId) throw new Error(`${label} expected sync identity fields.`)
}

const assertPeerIdentity = (statusA, statusB) => {
  if (!statusA.deviceId || !statusB.deviceId) throw new Error('Expected both devices to expose a deviceId.')
  if (statusA.deviceId === statusB.deviceId) throw new Error(`Expected distinct device IDs, got ${statusA.deviceId}.`)
  if (!statusA.folderId || !statusB.folderId) throw new Error('Expected both devices to expose a folderId.')
  if (statusA.folderId !== statusB.folderId) {
    throw new Error(`Expected both devices to detect the same logical vault folder, got ${statusA.folderId} and ${statusB.folderId}.`)
  }
}

const waitForAutoSyncSuccess = async(port, label, minSuccesses = 1, timeoutMs = 45000) => {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    const state = await request(port, '/api/sync/auto/status').catch(() => null)
    if (state?.successes >= minSuccesses) return state
    await delay(500)
  }
  throw new Error(`${label} auto sync did not report ${minSuccesses} successful run(s).`)
}

const catFromContainer = async(container, absolutePath) => docker(['exec', container, 'cat', absolutePath])

const trackedSyncMetadata = async(container) => {
  const output = await docker(['exec', container, 'git', '-C', '/data/vault', 'ls-files', '.elephantnote'])
  return output.split('\n').map((line) => line.trim()).filter(Boolean)
}

const assertNoTrackedSyncMetadata = async(container, label) => {
  const tracked = await trackedSyncMetadata(container)
  const leaked = tracked.filter((file) => /(^|\/)sync-(config|log|queue|state)\.json$/.test(file))
  if (leaked.length) {
    throw new Error(`${label} leaked local sync metadata into git: ${JSON.stringify(leaked)}`)
  }
}

const parseMemoryMiB = (value = '') => {
  const match = String(value).match(/([0-9.]+)\s*([KMGT]?i?B)/i)
  if (!match) return Number.NaN
  const amount = Number(match[1])
  const unit = match[2].toLowerCase()
  if (unit.startsWith('k')) return amount / 1024
  if (unit.startsWith('g')) return amount * 1024
  if (unit.startsWith('t')) return amount * 1024 * 1024
  return amount
}

const containerMemoryMiB = async(container) => {
  const output = await docker(['stats', '--no-stream', '--format', '{{.MemUsage}}', container])
  return parseMemoryMiB(output.split('/')[0])
}

const assertResourceBudget = async(containers) => {
  const samples = []
  for (const container of containers) {
    const memoryMiB = await containerMemoryMiB(container)
    samples.push({ container, memoryMiB })
    if (!Number.isFinite(memoryMiB)) throw new Error(`Could not parse Docker memory usage for ${container}.`)
    if (memoryMiB > maxContainerMemoryMiB) {
      throw new Error(`${container} used ${memoryMiB.toFixed(1)} MiB, above ${maxContainerMemoryMiB} MiB budget.`)
    }
  }
  const elapsedMs = Date.now() - startedAt
  if (elapsedMs > maxEndToEndMs) {
    throw new Error(`Docker sync smoke took ${elapsedMs} ms, above ${maxEndToEndMs} ms budget.`)
  }
  return { samples, elapsedMs }
}

try {
  await docker(['build', '-t', image, '.'])
  await docker(['network', 'create', network])
  for (const volume of resources.volumes) await docker(['volume', 'create', volume])
  await docker([
    'run',
    '--rm',
    '--name',
    `${prefix}-remote-init`,
    '-v',
    `${remoteVolume}:/git`,
    image,
    'sh',
    '-lc',
    `git init --bare ${remotePath}`
  ])

  await startDevice(deviceA, basePort, vaultA)
  await startAutoPullDevice(deviceB, basePort + 1, vaultB)
  await waitForServer(basePort, 'device A')
  await waitForServer(basePort + 1, 'device B')

  await request(basePort, '/api/sync/run', {
    method: 'POST',
    body: { init: { remote: remotePath } }
  })
  await request(basePort + 1, '/api/sync/auto/run', { method: 'POST' }).catch(() => {})
  assertPeerIdentity(
    await request(basePort, '/api/sync/status'),
    await request(basePort + 1, '/api/sync/status')
  )

  await request(basePort, '/api/notes', {
    method: 'POST',
    body: { title: 'Two Device Sync A', body: 'Created on device A and expected on device B.' }
  })

  const pushedFromA = await request(basePort, '/api/sync/run', {
    method: 'POST',
    body: {
      snapshot: { message: 'Device A sync snapshot' },
      push: {}
    }
  })
  assertClean(pushedFromA, 'device A push')
  assertHistory(pushedFromA, 'push', 'device A push')
  await assertNoTrackedSyncMetadata(deviceA, 'device A initial push')
  const branch = pushedFromA.branch || 'master'

  const notesOnB = await waitForNote(basePort + 1, 'Two Device Sync A.md', 'device B online auto-pull')
  assertNoteListed(notesOnB, 'Two Device Sync A.md', 'device B')
  await waitForAutoSyncSuccess(basePort + 1, 'device B online auto-pull')
  await assertNoTrackedSyncMetadata(deviceB, 'device B online auto-pull')
  const contentOnB = await catFromContainer(deviceB, '/data/vault/Two Device Sync A.md')
  if (!contentOnB.includes('Created on device A and expected on device B.')) {
    throw new Error('Device B file content did not match the note created on device A.')
  }

  await stopDevice(deviceB)
  await request(basePort, '/api/notes', {
    method: 'POST',
    body: { title: 'Offline Reconnect Sync', body: 'Created while device B is outside the network.' }
  })
  const pushedWhileBOffline = await request(basePort, '/api/sync/run', {
    method: 'POST',
    body: {
      snapshot: { message: 'Device A offline peer snapshot' },
      push: { branch }
    }
  })
  assertClean(pushedWhileBOffline, 'device A push while B offline')
  assertHistory(pushedWhileBOffline, 'push', 'device A push while B offline')
  await assertNoTrackedSyncMetadata(deviceA, 'device A push while B offline')

  await startAutoPullDevice(deviceB, basePort + 1, vaultB, branch)
  await waitForServer(basePort + 1, 'device B reconnect')
  const notesAfterReconnect = await waitForNote(basePort + 1, 'Offline Reconnect Sync.md', 'device B reconnect auto-pull')
  assertNoteListed(notesAfterReconnect, 'Offline Reconnect Sync.md', 'device B reconnect')
  await waitForAutoSyncSuccess(basePort + 1, 'device B reconnect auto-pull')
  await assertNoTrackedSyncMetadata(deviceB, 'device B reconnect auto-pull')
  const reconnectedContentOnB = await catFromContainer(deviceB, '/data/vault/Offline Reconnect Sync.md')
  if (!reconnectedContentOnB.includes('Created while device B is outside the network.')) {
    throw new Error('Device B did not auto-pull the note created while it was offline.')
  }

  await restartDeviceWithoutAutoSync(deviceB, basePort + 1, vaultB)
  await request(basePort + 1, '/api/notes', {
    method: 'POST',
    body: { title: 'Two Device Sync B', body: 'Created on device B and expected on device A.' }
  })

  const pushedFromB = await request(basePort + 1, '/api/sync/run', {
    method: 'POST',
    body: {
      snapshot: { message: 'Device B sync snapshot' },
      push: { branch }
    }
  })
  assertClean(pushedFromB, 'device B push')
  assertHistory(pushedFromB, 'push', 'device B push')
  await assertNoTrackedSyncMetadata(deviceB, 'device B push')

  const pulledIntoA = await request(basePort, '/api/sync/run', {
    method: 'POST',
    body: {
      pull: { branch }
    }
  })
  assertClean(pulledIntoA, 'device A pull')
  assertHistory(pulledIntoA, 'pull', 'device A pull')
  await assertNoTrackedSyncMetadata(deviceA, 'device A pull')

  const notesOnA = await request(basePort, '/api/notes')
  assertNoteListed(notesOnA, 'Two Device Sync B.md', 'device A')
  const contentOnA = await catFromContainer(deviceA, '/data/vault/Two Device Sync B.md')
  if (!contentOnA.includes('Created on device B and expected on device A.')) {
    throw new Error('Device A file content did not match the note created on device B.')
  }

  const resourcesUsed = await assertResourceBudget([deviceA, deviceB])

  console.log(JSON.stringify({
    ok: true,
    image,
    branch,
    ports: { deviceA: basePort, deviceB: basePort + 1 },
    remotePath,
    autoSyncIntervalMs,
    memoryBudgetMiB: maxContainerMemoryMiB,
    resources: resourcesUsed,
    checks: [
      'device identities are distinct while the logical vault folder is detected as shared',
      'device A pushed a note to a shared bare git remote',
      'device B auto-pulled and exposed the note through the API',
      'device B left the network while device A created and pushed a note',
      'device B reconnected and auto-pulled the offline note without an explicit sync call',
      'device B restarted in manual mode before reverse sync to avoid auto-sync races',
      'device B pushed a second note',
      'device A pulled and exposed the second note through the API',
      'local sync metadata files stay untracked in each container git repository',
      'containers stayed under the configured memory and end-to-end runtime budgets'
    ]
  }, null, 2))
} finally {
  for (const container of resources.containers) await execFileAsync('docker', ['rm', '-f', container]).catch(() => {})
  await execFileAsync('docker', ['network', 'rm', resources.network]).catch(() => {})
  for (const volume of resources.volumes) await execFileAsync('docker', ['volume', 'rm', '-f', volume]).catch(() => {})
}
