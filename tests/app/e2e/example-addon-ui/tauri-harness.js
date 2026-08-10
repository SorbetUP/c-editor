'use strict'

const fs = require('node:fs')
const os = require('node:os')
const path = require('node:path')
const { spawn, spawnSync } = require('node:child_process')

const root = path.resolve(__dirname, '../../../..')

const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const VITE_READY_TIMEOUT_MS = 120000

const retry = async (read, predicate, timeoutMs = 30000) => {
  const deadline = Date.now() + timeoutMs
  let last
  while (Date.now() <= deadline) {
    last = await read()
    if (predicate(last)) return last
    await wait(100)
  }
  throw new Error(`Acceptance condition timed out: ${JSON.stringify(last)}`)
}

const createFixture = () => {
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'elephant-example-addon-ui-'))
  const vaultRoot = path.join(fixtureRoot, 'vault')
  const configRoot = path.join(fixtureRoot, 'config')
  fs.mkdirSync(path.join(vaultRoot, '.elephantnote'), { recursive: true })
  fs.writeFileSync(path.join(vaultRoot, 'Acceptance.md'), '# Acceptance\n\nExample addon UI fixture.\n')
  fs.writeFileSync(path.join(vaultRoot, '.elephantnote', 'workspace.json'), JSON.stringify({
    version: 1,
    vaultName: 'Example Addon UI',
    sidebar: []
  }))
  fs.mkdirSync(configRoot, { recursive: true })
  fs.writeFileSync(path.join(configRoot, 'elephantnote.json'), JSON.stringify({
    vaults: [],
    activeVaultId: null
  }))
  const identifier = `com.elephantnote.examplee2e.${path.basename(fixtureRoot).replace(/[^a-z0-9]/gi, '').toLowerCase()}`
  const tauriConfigPath = path.join(fixtureRoot, 'tauri-e2e-config.json')
  fs.writeFileSync(tauriConfigPath, JSON.stringify({
    identifier,
    build: {
      beforeDevCommand: { script: 'true', cwd: '../../..', wait: true },
      devUrl: 'http://127.0.0.1:1420'
    }
  }))
  return { fixtureRoot, vaultRoot, configRoot, tauriConfigPath }
}

const packageAddon = (slug, outputRoot) => {
  const outputPath = path.join(outputRoot, `${slug}.enaddon`)
  const sourcePath = path.join(root, 'examples', 'addons', slug)
  const result = spawnSync(process.execPath, [
    path.join(root, 'build', 'scripts', 'package-addon.mjs'),
    sourcePath,
    outputPath
  ], { cwd: root, encoding: 'utf8' })
  if (result.status !== 0) throw new Error(`Failed to package ${slug}: ${result.stderr || result.stdout}`)
  return outputPath
}

const launchTauriProcess = async (fixture) => {
  if (!fixture.vite) {
    const existing = await fetch('http://127.0.0.1:1420/index.html').then((response) => response.ok).catch(() => false)
    if (!existing) {
      fixture.vite = spawn(path.join(root, 'Elephant', 'node_modules', 'vite', 'bin', 'vite.js'), [
        '--config', 'vite.tauri.config.mjs', '--host', '127.0.0.1', '--port', '1420', '--strictPort'
      ], { cwd: root, stdio: ['ignore', 'pipe', 'pipe'] })
      await retry(
        () => fetch('http://127.0.0.1:1420/index.html').then((response) => response.ok).catch(() => false),
        (ready) => ready,
        VITE_READY_TIMEOUT_MS
      )
    }
  }
  const child = spawn('cargo', ['tauri', 'dev', '--no-watch', '--config', fixture.tauriConfigPath], {
    cwd: path.join(root, 'Elephant', 'backend', 'tauri'),
    detached: true,
    stdio: ['ignore', 'pipe', 'pipe'],
    env: {
      ...process.env,
      HOME: fixture.fixtureRoot,
      RUSTUP_HOME: process.env.RUSTUP_HOME || path.join(os.homedir(), '.rustup'),
      CARGO_HOME: process.env.CARGO_HOME || path.join(os.homedir(), '.cargo'),
      TAURI_FRONTEND_PATH: root,
      ELEPHANTNOTE_CONFIG_DIR: fixture.configRoot,
      ELEPHANT_ACCEPTANCE_TAURI_PORT: '0',
      ELEPHANT_ACCEPTANCE_HIDE_WINDOW: '1'
    }
  })
  let output = ''
  const append = (chunk) => { output += chunk.toString() }
  child.stdout.on('data', append)
  child.stderr.on('data', append)
  const port = await new Promise((resolve, reject) => {
    const deadline = Date.now() + 180000
    const poll = () => {
      const match = output.match(/ELEPHANT_ACCEPTANCE_TAURI_PORT=(\d+)/)
      if (match) return resolve(Number(match[1]))
      if (child.exitCode !== null) return reject(new Error(`Tauri exited (${child.exitCode})\n${output.slice(-12000)}`))
      if (Date.now() > deadline) return reject(new Error(`Timed out starting Tauri\n${output.slice(-12000)}`))
      setTimeout(poll, 100)
    }
    poll()
  })
  const endpoint = `http://127.0.0.1:${port}`
  const request = async (command, args = [], allowFailure = false) => {
    let response
    try {
      response = await fetch(`${endpoint}/command`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ command, args }),
        signal: AbortSignal.timeout(60000)
      })
    } catch (error) {
      throw new Error(`${command} transport failed: ${error.message}\n${output.slice(-16000)}`)
    }
    const body = await response.json()
    if (!allowFailure && (!response.ok || !body.ok)) {
      throw new Error(`${command} failed: ${body.error || response.status}`)
    }
    return body
  }
  const command = async (name, ...args) => (await request(name, args)).result
  const stop = async () => {
    if (child.exitCode !== null) return
    try { process.kill(-child.pid, 'SIGTERM') } catch {}
    await Promise.race([
      new Promise((resolve) => child.once('close', resolve)),
      wait(15000)
    ])
    if (child.exitCode === null) {
      try { process.kill(-child.pid, 'SIGKILL') } catch {}
    }
  }
  const selectVault = async () => {
    await command('selectVault', fixture.vaultRoot)
    await command('waitFor', '.en-library-grid', 30000)
  }
  try {
    // The Rust acceptance port is announced before the renderer finishes
    // installing its event listener and calls tauri_acceptance_ready.
    await wait(10000)
    await command('capabilities')
    await selectVault()
  } catch (error) {
    await stop()
    throw error
  }
  return {
    child,
    fixture,
    endpoint,
    command,
    request,
    output: () => output,
    stop,
  }
}

const startTauri = async (fixture) => {
  let current = await launchTauriProcess(fixture)
  const api = {
    fixture,
    command: (...args) => current.command(...args),
    request: (...args) => current.request(...args),
    output: () => current.output(),
    stop: () => current.stop(),
    async close() {
      await current.stop()
      if (fixture.vite && fixture.vite.exitCode === null) {
        try { fixture.vite.kill('SIGTERM') } catch {}
        await Promise.race([
          new Promise((resolve) => fixture.vite.once('close', resolve)),
          wait(5000)
        ])
      }
    },
    async restart() {
      await current.stop()
      current = await launchTauriProcess(fixture)
      await current.command('selectVault', fixture.vaultRoot)
      await current.command('waitFor', '.en-library-grid', 30000)
      return api
    }
  }
  return api
}

const readDom = (app, selector) => app.command('readDom', selector)

const waitForText = (app, selector, text, timeoutMs = 30000) => retry(
  () => readDom(app, selector),
  (result) => result?.exists && result?.visible && String(result.text).includes(text),
  timeoutMs
)

const addonState = (app) => app.command('addonState')

const waitForAddon = (app, addonId, predicate = () => true) => retry(
  () => addonState(app),
  (state) => predicate(state?.addons?.find((addon) => addon.id === addonId)),
  30000
)

const openAddonSettings = async (app) => {
  const settings = await readDom(app, '.en-settings-panel')
  if (!settings.visible) {
    await app.command('click', '.en-rail-icon[aria-label="Settings"]')
    await app.command('waitFor', '.en-settings-panel', 10000)
  }
  const addons = await readDom(app, '.en-addons-panel')
  if (!addons.visible) {
    await app.command('click', '.en-settings-nav button:nth-child(4)')
    await app.command('waitFor', '.en-addons-panel', 10000)
  }
}

const openInstalledAddon = async (app, addonId) => {
  await openAddonSettings(app)
  const detailSelector = `.en-addon-browser-detail[data-selected-addon-id="${addonId}"]`
  if (!(await readDom(app, detailSelector)).visible) {
    const tileSelector = `.en-addon-tile.installed[data-addon-id="${addonId}"]`
    const sidebarSelector = `.en-addon-browser-item.installed[data-addon-id="${addonId}"]`
    const tile = await readDom(app, tileSelector)
    await app.command('click', tile.visible ? tileSelector : sidebarSelector)
    await app.command('waitFor', detailSelector, 10000)
  }
}

const runVisibleCommand = async (app, title) => {
  await app.command('click', `.en-addon-detail-commands button`)
  await waitForText(app, '.en-addons-feedback', 'completed', 60000)
  return title
}

const setCommunityEnabled = (app, enabled) => app.command('invokeTauri', 'tauri_prefs_set', {
  key: 'addons.communityEnabled',
  value: enabled === true
})

const setLocalStorage = (app, key, value) => app.command('setLocalStorage', key, value)

const installPackage = async (app, packagePath, addonId) => {
  const record = await app.command('invokeTauri', 'tauri_addons_install', { packagePath })
  if (record?.manifest?.id !== addonId) throw new Error(`Installed package id mismatch: ${JSON.stringify(record)}`)
  return record
}

const assertPermissionDenied = async (app, addonId, relativePath) => {
  const response = await app.request('invokeTauri', [
    'tauri_addons_notes_write',
    { addonId, path: relativePath, markdown: 'must be denied', overwrite: true }
  ], true)
  if (response.ok || !/permitted|permission|scope/i.test(response.error || '')) {
    throw new Error(`Expected addon permission failure for ${addonId}: ${JSON.stringify(response)}`)
  }
  return response.error
}

const cleanup = async (app, addonId, hidden = false) => {
  const current = (await addonState(app)).addons.find((addon) => addon.id === addonId)
  if (current?.enabled) await app.command('disableAddon', addonId)
  if (hidden) await app.command('invokeTauri', 'tauri_addons_uninstall', { addonId })
  else {
    await openInstalledAddon(app, addonId)
    await app.command('click', '.en-addon-detail-actions .en-danger-button')
    await retry(() => readDom(app, `.en-addon-tile[data-addon-id="${addonId}"]`), (result) => !result.exists, 15000)
  }
  await app.restart()
  const after = await addonState(app)
  if (after.addons.some((addon) => addon.id === addonId)) throw new Error(`Addon remained after cleanup: ${addonId}`)
}

const writeEvidence = (testInfo, name, value) => {
  const filename = testInfo.outputPath(`${name}.json`)
  fs.writeFileSync(filename, `${JSON.stringify(value, null, 2)}\n`)
  return filename
}

module.exports = {
  addonState,
  assertPermissionDenied,
  cleanup,
  createFixture,
  installPackage,
  openAddonSettings,
  openInstalledAddon,
  packageAddon,
  readDom,
  retry,
  setCommunityEnabled,
  setLocalStorage,
  startTauri,
  waitForAddon,
  waitForText,
  writeEvidence
}
