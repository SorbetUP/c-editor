#!/usr/bin/env node
import { execFileSync, spawn } from 'node:child_process'
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import { loadScenario, materializeFixture } from '../../../tools/freya-differential/lib/scenario.mjs'

const args = process.argv.slice(2)
const stringArg = (flag, fallback = null) => {
  const index = args.indexOf(flag)
  return index >= 0 ? args[index + 1] : fallback
}
const harnessRoot = path.resolve(import.meta.dirname, '../../..')
const repoRoot = path.resolve(stringArg('--repo-root', harnessRoot))
const output = path.resolve(stringArg('--output', 'test-results/freya-parity/tauri'))
const scenarioPath = path.resolve(stringArg('--scenario', path.join(harnessRoot, 'migration/freya/differential-scenarios.json')))
const configPath = path.resolve(stringArg('--config', path.join(import.meta.dirname, 'checkpoints.json'))
const fixtureRoot = path.resolve(stringArg('--fixture-root', path.join(output, '_fixture'))
const expectedSha = stringArg('--expected-sha', process.env.TAURI_REFERENCE_SHA || '')
const appPath = stringArg('--app-path', './build/scripts/build_dev.sh')
const startupTimeoutMs = Number(process.env.FREYA_PARITY_TAURI_STARTUP_TIMEOUT_MS || 900000)
const config = JSON.parse(await readFile(configPath, 'utf8'))
const scenario = await loadScenario(scenarioPath)
const viewport = scenario.viewport
const originalHome = process.env.HOME || '/tmp'

if (!Number.isFinite(startupTimeoutMs) || startupTimeoutMs < 30000) {
  throw new Error(`INFRA_ERROR: invalid FREYA_PARITY_TAURI_STARTUP_TIMEOUT_MS=${process.env.FREYA_PARITY_TAURI_STARTUP_TIMEOUT_MS}`)
}
if (expectedSha) {
  const actual = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: repoRoot, encoding: 'utf8' }).trim()
  if (actual !== expectedSha) throw new Error(`INFRA_ERROR: Tauri reference SHA mismatch: expected ${expectedSha}, got ${actual}`)
}
for (const binary of ['cargo', 'xdotool', 'import', 'identify', 'xwininfo']) {
  try { execFileSync('which', [binary], { stdio: 'ignore' }) } catch { throw new Error(`INFRA_ERROR: required capture tool ${binary} is missing`) }
}
try { execFileSync('cargo', ['tauri', '--version'], { cwd: repoRoot, stdio: 'inherit' }) } catch { throw new Error('INFRA_ERROR: cargo-tauri CLI is not available') }

await rm(output, { recursive: true, force: true })
await mkdir(output, { recursive: true })
await rm(fixtureRoot, { recursive: true, force: true })
await mkdir(fixtureRoot, { recursive: true })
const fixture = await materializeFixture(scenario, fixtureRoot)
const vaultRoot = path.join(fixtureRoot, fixture.roots.vault)
const configRoot = path.join(fixtureRoot, fixture.roots.config)

let child
let endpoint
let spawnError
let logs = ''
const collect = (prefix, chunk) => {
  const text = chunk.toString(); logs += text; process.stdout.write(`${prefix}${text}`)
}
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms))
const stop = async () => {
  if (!child || child.exitCode !== null) return
  try { process.kill(-child.pid, 'SIGTERM') } catch { child.kill('SIGTERM') }
  await Promise.race([new Promise((resolve) => child.once('close', resolve)), sleep(5000)])
}
async function command (name, ...commandArgs) {
  const response = await fetch(`${endpoint}/command`, {
    method: 'POST', headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ command: name, args: commandArgs })
  })
  const body = await response.json()
  if (!response.ok || !body.ok) throw new Error(`INFRA_ERROR: Tauri command ${name} failed: ${body.error || response.status}`)
  return body.result
}
async function waitFor (selector, timeout = 15000) {
  const deadline = Date.now() + timeout; let state
  while (Date.now() < deadline) {
    state = await command('readDom', selector)
    if (state?.exists && state?.visible !== false) return state
    await sleep(75)
  }
  throw new Error(`INFRA_ERROR: timeout waiting for ${selector}: ${JSON.stringify(state)}`)
}
async function waitGone (selector, timeout = 15000) {
  const deadline = Date.now() + timeout; let state
  while (Date.now() < deadline) {
    state = await command('readDom', selector)
    if (!state?.exists || state?.visible === false) return state
    await sleep(75)
  }
  throw new Error(`INFRA_ERROR: timeout waiting for ${selector} to disappear: ${JSON.stringify(state)}`)
}
async function waitText (text, timeout = 15000) {
  const deadline = Date.now() + timeout; let state
  while (Date.now() < deadline) {
    state = await command('readDisplayed')
    if (state?.displayedText?.includes(text)) return state
    await sleep(75)
  }
  throw new Error(`INFRA_ERROR: timeout waiting for visible text ${JSON.stringify(text)}: ${JSON.stringify(state)}`)
}
function windowId () {
  const raw = execFileSync('xdotool', ['search', '--onlyvisible', '--name', '^Elephant$'], { encoding: 'utf8' }).trim().split(/\s+/).filter(Boolean)
  if (!raw.length) {
    let tree = ''
    try { tree = execFileSync('xwininfo', ['-root', '-tree'], { encoding: 'utf8' }) } catch {}
    throw new Error(`INFRA_ERROR: visible Tauri window named Elephant was not found; X11 tree=${JSON.stringify(tree)}`)
  }
  return raw.at(-1)
}
function pngDimensions (png) {
  return execFileSync('identify', ['-format', '%w %h', png], { encoding: 'utf8' }).trim().split(/\s+/).map(Number)
}
async function capturePng (id, png) {
  execFileSync('import', ['-window', id, png])
  const dimensions = pngDimensions(png)
  if (dimensions[0] !== viewport.width || dimensions[1] !== viewport.height) {
    const xinfo = execFileSync('xwininfo', ['-id', id], { encoding: 'utf8' })
    throw new Error(`INFRA_ERROR: Tauri capture is ${dimensions.join('x')}, expected ${viewport.width}x${viewport.height}; xwininfo=${JSON.stringify(xinfo)}`)
  }
  return dimensions
}
async function capture (checkpoint) {
  await sleep(350)
  const id = windowId()
  execFileSync('xdotool', ['windowsize', '--sync', id, String(viewport.width), String(viewport.height)])
  await sleep(150)
  const dir = path.join(output, checkpoint); await mkdir(dir, { recursive: true })
  const png = path.join(dir, 'static.png')
  const dimensions = await capturePng(id, png)
  await sleep(250)
  const stability = path.join(dir, 'stability.png')
  await capturePng(id, stability)
  return { checkpoint, path: path.relative(output, png), stabilityPath: path.relative(output, stability), width: dimensions[0], height: dimensions[1], windowId: id }
}

const captures = []
const captureById = async (id) => { captures.push(await capture(id)) }
try {
  child = spawn(appPath, [], {
    cwd: repoRoot,
    env: {
      ...process.env,
      HOME: fixtureRoot,
      CARGO_HOME: process.env.CARGO_HOME || path.join(originalHome, '.cargo'),
      RUSTUP_HOME: process.env.RUSTUP_HOME || path.join(originalHome, '.rustup'),
      PNPM_HOME: process.env.PNPM_HOME || path.join(originalHome, '.local', 'share', 'pnpm'),
      ELEPHANTNOTE_CONFIG_DIR: configRoot,
      ELEPHANT_ACCEPTANCE_TAURI_PORT: '0',
      ELEPHANT_ACCEPTANCE_HIDE_WINDOW: '0',
      ELEPHANT_ACCEPTANCE_SHOW_WINDOW: '1'
    },
    detached: true,
    stdio: ['ignore', 'pipe', 'pipe']
  })
  child.once('error', (error) => { spawnError = error; collect('[tauri-reference:spawn-error] ', `${error.stack || error}\n`) })
  child.stdout.on('data', (chunk) => collect('[tauri-reference] ', chunk))
  child.stderr.on('data', (chunk) => collect('[tauri-reference:error] ', chunk))
  const deadline = Date.now() + startupTimeoutMs
  while (Date.now() < deadline) {
    const match = logs.match(/ELEPHANT_ACCEPTANCE_TAURI_PORT=(\d+)/)
    if (match) { endpoint = `http://127.0.0.1:${match[1]}`; break }
    if (spawnError) throw new Error(`INFRA_ERROR: failed to spawn Tauri reference: ${spawnError.message}`)
    if (child.exitCode !== null) throw new Error(`INFRA_ERROR: Tauri exited before acceptance server startup (${child.exitCode})`)
    await sleep(250)
  }
  if (!endpoint) throw new Error(`INFRA_ERROR: timed out after ${startupTimeoutMs}ms waiting for Tauri acceptance server`)
  const health = await fetch(`${endpoint}/health`).then((response) => response.json())
  if (health.transport !== 'tauri') throw new Error(`INFRA_ERROR: acceptance transport is not Tauri: ${JSON.stringify(health)}`)

  await command('selectVault', vaultRoot)
  await waitFor('.en-library-grid'); await waitText('Alpha note')
  await captureById('startup')

  await command('click', '[aria-label="Search"]')
  const searchInput = 'input[placeholder="Search notes, paths, tags, or ideas…"]'
  await waitFor(searchInput); await captureById('search-open')
  await command('fill', searchInput, 'Alpha note'); await waitFor('.en-search-result'); await captureById('search-results')
  await command('press', searchInput, 'Escape'); await command('press', 'body', 'Escape'); await waitGone('.en-search-overlay'); await waitFor('.en-library-grid'); await captureById('library-after-search')

  const allNotes = await command('readDom', '[aria-label="All notes"]')
  if (allNotes?.exists) await command('click', '[aria-label="All notes"]')
  await waitFor('.en-library-grid'); await waitText('Alpha note'); await captureById('root-library')

  await command('click', '.en-note-card:not(.is-folder)'); await waitFor('[data-testid="muya-runtime-editor"]'); await waitText('Visible alpha body line.'); await captureById('note-open')
  const editor = '[data-testid="muya-runtime-editor"]'
  await command('press', editor, 'Control+End'); await command('press', editor, 'Enter'); await command('insertText', editor, 'Differential edit marker 2026-06-22.'); await waitText('Differential edit marker 2026-06-22.'); await captureById('note-edited')

  await command('click', '[aria-label="Close note"]'); await waitFor('.en-library-grid'); await captureById('library-after-note')
  await command('click', '.en-create-button-primary'); await waitFor('.en-create-menu-popover'); await waitText('Drawing'); await captureById('create-menu-open')
  await command('press', 'body', 'Escape'); await waitGone('.en-create-menu-popover'); await captureById('library-after-menu')

  await command('click', '[aria-label="Settings"]'); await waitFor('.en-settings-panel'); await captureById('settings-general')
  await command('click', '.en-settings-nav button:nth-of-type(2)'); await waitFor('.en-settings-content[data-active-section="editor"]'); await captureById('settings-editor')
  await command('click', '[aria-label="Close settings"]'); await waitGone('.en-settings-panel')

  await command('installOfficialAddon', 'elephant.graph')
  await command('enableAddon', 'elephant.graph')
  await command('runAddonAction', 'elephant.graph.open'); await waitFor('.elephant-graph-package', 20000); await captureById('graph-default')

  const expected = config.checkpoints.map((entry) => entry.id)
  const actual = captures.map((entry) => entry.checkpoint)
  if (JSON.stringify(actual) !== JSON.stringify(expected)) throw new Error(`INFRA_ERROR: captured checkpoints ${actual} do not match contract ${expected}`)
  await writeFile(path.join(output, 'metadata.json'), `${JSON.stringify({
    schemaVersion: 2, runtime: 'tauri', referenceSha: expectedSha || null, scenarioId: scenario.id,
    viewport, fixture: { id: fixture.id, files: fixture.files },
    captureMethod: 'ImageMagick import of the live X11 Tauri window', captures
  }, null, 2)}\n`)
} finally {
  await writeFile(path.join(output, 'tauri.log'), logs).catch(() => {})
  await stop()
}
console.log(`[freya-parity] captured ${captures.length} real Tauri X11 checkpoints plus stability frames`)
