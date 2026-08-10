const assert = require('node:assert/strict')
const crypto = require('node:crypto')
const fs = require('node:fs')
const os = require('node:os')
const path = require('node:path')
const readline = require('node:readline')
const { spawn, spawnSync } = require('node:child_process')

const ADDON_ID = 'elephant.sync'
const SERVICE_PROTOCOL = 'elephant-addon-service-v1'
const repoRoot = path.resolve(__dirname, '../../../../../')
const nativeRoot = path.join(
  repoRoot,
  'addons/official/sync/native'
)
const packageRoot = path.dirname(nativeRoot)

const platformKey = () => {
  const platform = { darwin: 'macos', win32: 'windows', linux: 'linux' }[process.platform]
  const architecture = { arm64: 'aarch64', x64: 'x86_64', arm: 'armv7' }[process.arch]
  if (!platform || !architecture) throw new Error(`Unsupported Sync test platform: ${process.platform}/${process.arch}`)
  return `${platform}-${architecture}`
}

const binaryName = () => process.platform === 'win32' ? 'elephant-sync-service.exe' : 'elephant-sync-service'

const resolveServiceBinary = () => {
  const packaged = path.join(nativeRoot, platformKey(), binaryName())
  if (fs.existsSync(packaged)) return { path: packaged, source: 'official-addon-sidecar' }

  const built = path.join(nativeRoot, 'target', 'release', binaryName())
  if (!fs.existsSync(built)) {
    const result = spawnSync('cargo', [
      'build',
      '--release',
      '--manifest-path',
      path.join(nativeRoot, 'Cargo.toml')
    ], { cwd: repoRoot, encoding: 'utf8' })
    if (result.status !== 0 || !fs.existsSync(built)) {
      throw new Error([
        `Unable to build the real Sync service for ${platformKey()}.`,
        result.stdout || '',
        result.stderr || ''
      ].join('\n'))
    }
  }
  return { path: built, source: 'cargo-release-build' }
}

const sha256 = (filePath) => crypto.createHash('sha256').update(fs.readFileSync(filePath)).digest('hex')

const summarize = (value) => {
  if (!value || typeof value !== 'object') return value
  const result = {}
  for (const key of [
    'ok', 'running', 'stopped', 'state', 'runtime', 'transport', 'pairing',
    'endpointId', 'folderId', 'folderLabel', 'transferredFiles', 'transferredBytes',
    'sessions', 'conflicts', 'peers', 'pendingInvites', 'networkTransfersReady'
  ]) {
    if (value[key] !== undefined) result[key] = value[key]
  }
  if (value.error) result.error = typeof value.error === 'string' ? value.error : value.error.message
  return result
}

class SyncServiceProcess {
  constructor(label, executable, fixture) {
    this.label = label
    this.fixture = fixture
    this.nextId = 1
    this.pending = null
    this.stderr = []
    this.unexpectedStdout = []
    this.child = spawn(executable, [], {
      cwd: nativeRoot,
      env: {
        ...process.env,
        ELEPHANT_ADDON_ID: ADDON_ID,
        ELEPHANT_ADDON_PACKAGE_DIR: packageRoot,
        ELEPHANT_ADDON_DATA_DIR: fixture.dataDir,
        ELEPHANT_VAULT_DIR: fixture.vaultDir,
        ELEPHANT_ADDON_SERVICE_PROTOCOL: SERVICE_PROTOCOL
      },
      stdio: ['pipe', 'pipe', 'pipe'],
      windowsHide: true
    })
    this.exit = new Promise((resolve) => {
      this.child.once('exit', (code, signal) => {
        this.exitState = { code, signal }
        if (this.pending) {
          this.pending.reject(new Error(`${this.label} exited before replying (code=${code}, signal=${signal || 'none'})`))
          this.pending = null
        }
        resolve(this.exitState)
      })
    })
    this.child.once('error', (error) => {
      if (this.pending) {
        this.pending.reject(error)
        this.pending = null
      }
    })
    const stderrReader = readline.createInterface({ input: this.child.stderr })
    stderrReader.on('line', (line) => this.stderr.push(line))
    this.stderrReader = stderrReader
    const stdoutReader = readline.createInterface({ input: this.child.stdout })
    stdoutReader.on('line', (line) => this.handleResponse(line))
    this.stdoutReader = stdoutReader
  }

  handleResponse(line) {
    if (!this.pending) {
      this.unexpectedStdout.push(line)
      return
    }
    let response
    try {
      response = JSON.parse(line)
    } catch (error) {
      this.pending.reject(new Error(`${this.label} returned invalid JSON: ${error.message}`))
      this.pending = null
      return
    }
    if (response.id !== this.pending.id || response.protocol !== SERVICE_PROTOCOL) {
      this.pending.reject(new Error(`${this.label} returned an invalid service envelope`))
      this.pending = null
      return
    }
    const pending = this.pending
    this.pending = null
    clearTimeout(pending.timer)
    pending.resolve(response)
  }

  call(method, params = {}, timeoutMs = 120_000) {
    if (this.exitState) return Promise.reject(new Error(`${this.label} is not running`))
    if (this.pending) return Promise.reject(new Error(`${this.label} only supports one in-flight request`))
    const id = this.nextId++
    const request = JSON.stringify({
      protocol: SERVICE_PROTOCOL,
      id,
      addonId: ADDON_ID,
      method,
      params
    })
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending = null
        reject(new Error(`${this.label} timed out in ${method} after ${timeoutMs}ms`))
      }, timeoutMs)
      this.pending = { id, timer, resolve, reject }
      this.child.stdin.write(`${request}\n`, (error) => {
        if (!error) return
        clearTimeout(timer)
        this.pending = null
        reject(error)
      })
    })
  }

  async callOk(method, params = {}, timeoutMs = 120_000) {
    const response = await this.call(method, params, timeoutMs)
    assert.equal(response.ok, true, `${this.label} ${method}: ${response.error?.message || 'service call failed'}`)
    return response.result
  }

  async stop() {
    if (this.exitState) return { ...this.exitState, clean: this.exitState.code === 0 && !this.exitState.signal }
    let stopError = null
    try {
      await this.callOk('service.stop', {}, 10_000)
    } catch (error) {
      stopError = error
    }
    const exitState = await Promise.race([
      this.exit,
      new Promise((resolve) => setTimeout(() => resolve(null), 10_000))
    ])
    if (!exitState && this.exitState === undefined) {
      this.child.kill('SIGTERM')
      await Promise.race([this.exit, new Promise((resolve) => setTimeout(resolve, 2_000))])
    }
    const finalState = this.exitState || { code: null, signal: 'timeout' }
    return {
      ...finalState,
      clean: !stopError && finalState.code === 0 && !finalState.signal,
      error: stopError?.message || null
    }
  }

  async forceStop() {
    if (!this.exitState) this.child.kill('SIGKILL')
    await Promise.race([this.exit, new Promise((resolve) => setTimeout(resolve, 2_000))])
    return this.exitState || { code: null, signal: 'timeout' }
  }

  snapshot() {
    return {
      label: this.label,
      pid: this.child.pid,
      exit: this.exitState || null,
      stderr: this.stderr,
      unexpectedStdout: this.unexpectedStdout
    }
  }
}

const createFixture = () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'elephant-real-iroh-sync-'))
  const a = path.join(root, 'device-a')
  const b = path.join(root, 'device-b')
  fs.mkdirSync(a, { recursive: true })
  fs.mkdirSync(b, { recursive: true })
  fs.writeFileSync(path.join(a, 'From A.md'), '# From A\n\nCreated on device A.\n', 'utf8')
  fs.writeFileSync(path.join(b, 'From B.md'), '# From B\n\nCreated on device B.\n', 'utf8')
  return {
    root,
    a: { vaultDir: a, dataDir: path.join(root, 'data-a') },
    b: { vaultDir: b, dataDir: path.join(root, 'data-b') }
  }
}

const readJson = (filePath) => JSON.parse(fs.readFileSync(filePath, 'utf8'))

const contentManifest = (manifest) => Object.fromEntries(
  Object.entries(manifest.files || {}).map(([relativePath, record]) => [relativePath, {
    size: record.size,
    hash: record.hash
  }])
)

const assertPairedConfig = (fixture) => {
  const configA = readJson(path.join(fixture.a.vaultDir, '.elephantnote/sync/config.json'))
  const configB = readJson(path.join(fixture.b.vaultDir, '.elephantnote/sync/config.json'))
  assert.equal(configA.peers.length, 1, 'device A must persist exactly one paired peer')
  assert.equal(configB.peers.length, 1, 'device B must persist exactly one paired peer')
  assert.equal(configA.folderId, configB.folderId, 'both devices must persist the same Sync folder id')
  assert.equal(configA.pendingInvites.length, 0, 'the one-time invitation must be consumed')
  return {
    folderIdShared: true,
    aPeers: configA.peers.length,
    bPeers: configB.peers.length,
    pendingInvitesA: configA.pendingInvites.length
  }
}

const assertRoundTripOnDisk = (fixture, expected) => {
  for (const [device, files] of Object.entries(expected)) {
    for (const [relativePath, content] of Object.entries(files)) {
      const fullPath = path.join(fixture[device].vaultDir, relativePath)
      assert.equal(fs.readFileSync(fullPath, 'utf8'), content, `${device} must contain ${relativePath}`)
    }
  }
}

async function runRealSyncScenario({ artifactPath }) {
  const fixture = createFixture()
  const binary = resolveServiceBinary()
  const events = []
  const record = (event, details = {}) => events.push({
    at: new Date().toISOString(),
    event,
    ...details
  })
  const services = {
    a: new SyncServiceProcess('device-a', binary.path, fixture.a),
    b: new SyncServiceProcess('device-b', binary.path, fixture.b)
  }
  const expected = {
    a: {
      'From A.md': '# From A\n\nCreated on device A.\n',
      'From B.md': '# From B\n\nCreated on device B.\n'
    },
    b: {
      'From A.md': '# From A\n\nCreated on device A.\n',
      'From B.md': '# From B\n\nCreated on device B.\n'
    }
  }
  let scenarioError = null
  let cleanup = {}
  let pairing = null
  const transfers = []
  try {
    record('processes.spawned', {
      binary: binary.path,
      binarySource: binary.source,
      binarySha256: sha256(binary.path),
      pids: { a: services.a.child.pid, b: services.b.child.pid }
    })

    const statusA = await services.a.callOk('service.start')
    const statusB = await services.b.callOk('service.start')
    assert.equal(statusA.transport, 'iroh')
    assert.equal(statusB.transport, 'iroh')
    assert.equal(statusA.networkTransfersReady, true)
    assert.equal(statusB.networkTransfersReady, true)
    assert.notEqual(statusA.endpointId, statusB.endpointId)
    record('services.started', {
      a: summarize(statusA),
      b: summarize(statusB)
    })

    const invitation = await services.a.callOk('sync.create-invite', { deviceName: 'Device A' })
    assert.ok(invitation.invite?.inviteId, 'the real service must return a structured invite')
    assert.ok(invitation.qrPayload, 'the real service must return a QR/manual payload')
    record('pairing.invite-created', {
      inviteId: invitation.invite.inviteId,
      folderId: invitation.invite.folderId,
      endpointId: invitation.invite.endpointAddr?.id
    })

    const tampered = JSON.parse(JSON.stringify(invitation.invite))
    tampered.inviteToken = `${tampered.inviteToken}-tampered`
    const invalidPairing = await services.b.call('sync.accept-invite', { invite: tampered })
    assert.equal(invalidPairing.ok, false, 'tampered pairing must fail at the real service boundary')
    assert.match(invalidPairing.error?.message || '', /invalid|token|connect/i)
    record('pairing.invalid-invite-rejected', { error: invalidPairing.error?.message })

    const accepted = await services.b.callOk('sync.accept-invite', { invite: invitation.invite }, 60_000)
    assert.equal(accepted.pairing?.state, 'paired')
    pairing = assertPairedConfig(fixture)
    record('pairing.completed', { ...pairing })

    const firstRun = await services.b.callOk('sync.run', {}, 120_000)
    assert.equal(firstRun.runtime, 'physical-sync-service')
    assert.equal(firstRun.sessions?.length, 1)
    assert.equal(firstRun.transferredFiles, 2, 'first run must transfer one file in each direction')
    assert.equal(firstRun.conflicts?.length, 0)
    transfers.push(summarize(firstRun))
    assertRoundTripOnDisk(fixture, expected)
    record('transfer.round-1.completed', summarize(firstRun))

    fs.writeFileSync(path.join(fixture.a.vaultDir, 'Second A.md'), 'second round from A\n', 'utf8')
    fs.writeFileSync(path.join(fixture.b.vaultDir, 'Second B.md'), 'second round from B\n', 'utf8')
    expected.a['Second A.md'] = 'second round from A\n'
    expected.a['Second B.md'] = 'second round from B\n'
    expected.b['Second A.md'] = 'second round from A\n'
    expected.b['Second B.md'] = 'second round from B\n'

    const secondRun = await services.b.callOk('sync.run', {}, 120_000)
    assert.equal(secondRun.transferredFiles, 2, 'a paired service must transfer subsequent changes both ways')
    assert.equal(secondRun.conflicts?.length, 0)
    transfers.push(summarize(secondRun))
    assertRoundTripOnDisk(fixture, expected)
    record('transfer.round-2.completed', summarize(secondRun))

    const manifestA = await services.a.callOk('sync.scan')
    const manifestB = await services.b.callOk('sync.scan')
    assert.deepEqual(contentManifest(manifestA.manifest), contentManifest(manifestB.manifest))
    assert.equal(fs.existsSync(path.join(fixture.a.vaultDir, '.elephantnote/sync/sync-manifest.json')), true)
    assert.equal(fs.existsSync(path.join(fixture.b.vaultDir, '.elephantnote/sync/sync-manifest.json')), true)
    record('disk.verify', {
      files: Object.keys(manifestA.manifest.files).sort(),
      baselines: true,
      manifestsEqual: true
    })
  } catch (error) {
    scenarioError = error
    record('scenario.error', { message: error.message, stack: error.stack })
  } finally {
    cleanup.a = await services.a.stop()
    cleanup.b = await services.b.stop()
    if (!cleanup.a.clean && !services.a.exitState) cleanup.a.forced = await services.a.forceStop()
    if (!cleanup.b.clean && !services.b.exitState) cleanup.b.forced = await services.b.forceStop()
    cleanup.processesExited = Boolean(services.a.exitState && services.b.exitState)
    record('cleanup.services', { ...cleanup })
    const processSnapshots = { a: services.a.snapshot(), b: services.b.snapshot() }
    fs.rmSync(fixture.root, { recursive: true, force: true })
    cleanup.vaultsRemoved = !fs.existsSync(fixture.root)
    record('cleanup.vaults', { rootRemoved: cleanup.vaultsRemoved })
    const artifact = {
      success: !scenarioError && cleanup.a.clean && cleanup.b.clean && cleanup.vaultsRemoved,
      binary: { path: binary.path, source: binary.source, sha256: sha256(binary.path) },
      pairing,
      transfers,
      cleanup,
      events,
      processes: processSnapshots
    }
    fs.mkdirSync(path.dirname(artifactPath), { recursive: true })
    fs.writeFileSync(artifactPath, `${JSON.stringify(artifact, null, 2)}\n`, 'utf8')
    if (scenarioError) throw new Error(`${scenarioError.message} (artifact: ${artifactPath})`)
    assert.equal(cleanup.a.clean, true, 'device A service must exit cleanly after service.stop')
    assert.equal(cleanup.b.clean, true, 'device B service must exit cleanly after service.stop')
    assert.equal(cleanup.processesExited, true, 'both real service processes must exit')
    assert.equal(cleanup.vaultsRemoved, true, 'temporary vaults must be removed')
    return artifact
  }
}

module.exports = { runRealSyncScenario }
