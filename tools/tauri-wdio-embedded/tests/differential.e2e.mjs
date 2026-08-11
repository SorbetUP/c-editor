import assert from 'node:assert/strict'
import path from 'node:path'
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises'
import { browser } from '@wdio/globals'

import { expectedActions, expectedCheckpoints, loadScenario } from '../../../tools/freya-differential/lib/scenario.mjs'
import { byText, captureFrame, displayed, railOrder, requireElement, sleep, snapshotVault, stateSnapshot, visibleByText, writeJson } from '../runtime.mjs'

const projectRoot = path.resolve(import.meta.dirname, '../../..')
const scenarioPath = path.resolve(process.env.DIFFERENTIAL_SCENARIO_PATH ?? path.join(projectRoot, 'migration/freya/differential-scenarios.json'))
const outputRoot = path.resolve(process.env.DIFFERENTIAL_OUTPUT_DIR ?? path.join(projectRoot, 'test-results/tauri-wdio-embedded/standalone'))
const fixtureRoot = path.resolve(process.env.DIFFERENTIAL_FIXTURE_ROOT ?? path.join(outputRoot, 'fixture'))
const vaultRoot = path.join(fixtureRoot, 'vault')
const scenario = await loadScenario(scenarioPath)
const actions = expectedActions(scenario)
const checkpoints = expectedCheckpoints(scenario)
const manifest = {
  schemaVersion: 1,
  scenarioId: scenario.id,
  runtime: 'tauri-wdio-embedded',
  status: 'running',
  viewport: scenario.viewport,
  scaleFactor: scenario.viewport.scaleFactor,
  deviceScaleFactor: scenario.viewport.deviceScaleFactor,
  visualSurface: 'application-content-only',
  fixture: null,
  provenance: {
    driver: 'webdriver',
    controlPlane: 'webdriver',
    provider: 'embedded',
    service: '@wdio/tauri-service@1.3.0',
    embeddedPlugin: 'tauri-plugin-wdio-webdriver@1.3.0',
    runId: process.env.DIFFERENTIAL_RUN_ID ?? `standalone-${process.pid}`,
    captureId: `tauri-wdio-embedded:${process.env.DIFFERENTIAL_RUN_ID ?? `standalone-${process.pid}`}`,
    commandSha256: process.env.DIFFERENTIAL_COMMAND_SHA256 ?? null,
    captureNonce: process.env.DIFFERENTIAL_CAPTURE_NONCE ?? null,
    real: false,
    synthetic: false,
    captureMode: 'real-tauri-wdio-embedded',
    artifactRoot: outputRoot,
    sourceEvidenceId: `tauri-wdio-embedded:${process.pid}:${Date.now()}`
  },
  comparison: { ready: false, reason: 'pending real 14-action run' },
  actions: [],
  checkpoints: [],
  frames: [],
  hardFailures: [],
  logs: {
    run: 'run-log.json',
    service: 'wdio.log',
    frontend: 'frontend.log',
    backend: 'backend.log',
    fixtureVault: 'fixture-vault.json'
  }
}
const runLog = []
const frameHashes = (frames) => new Set(frames.map((frame) => frame.sha256)).size

const selectorFor = (target) => {
  if (target?.strategy === 'placeholder') return `[placeholder="${target.value}"]`
  if (target?.strategy === 'testid') return `[data-testid="${target.value}"]`
  if (target?.strategy === 'role') return `[role="${target.role}"][aria-label="${target.name}"]`
  return target?.selector
}

const actionTarget = (action) => action.target?.tauri ?? {}

const clickTimeline = async (action, element, afterClick = async () => {}) => {
  let clicked = false
  const frames = []
  for (const [index, relativeMs] of action.frameTimes.entries()) {
    if (index) await sleep(relativeMs - action.frameTimes[index - 1])
    if (index === 1 && !clicked) {
      await element.click()
      clicked = true
      await afterClick()
    }
    frames.push(await captureFrame({ outputRoot, checkpoint: action.checkpoint, index, relativeMs, kind: index === 0 ? 'before' : index === action.frameTimes.length - 1 ? 'after' : 'during' }))
  }
  return frames
}

const inputTimeline = async (action, element, text) => {
  let typed = false
  const frames = []
  for (const [index, relativeMs] of action.frameTimes.entries()) {
    if (index) await sleep(relativeMs - action.frameTimes[index - 1])
    if (index === 1 && !typed) {
      await element.click()
      await element.setValue(text)
      typed = true
    }
    frames.push(await captureFrame({ outputRoot, checkpoint: action.checkpoint, index, relativeMs, kind: index === 0 ? 'before' : index === action.frameTimes.length - 1 ? 'after' : 'during' }))
  }
  return frames
}

const keyboardTimeline = async (action, keys) => {
  let pressed = false
  const frames = []
  for (const [index, relativeMs] of action.frameTimes.entries()) {
    if (index) await sleep(relativeMs - action.frameTimes[index - 1])
    if (index === 1 && !pressed) {
      await browser.keys(keys)
      pressed = true
    }
    frames.push(await captureFrame({ outputRoot, checkpoint: action.checkpoint, index, relativeMs, kind: index === 0 ? 'before' : index === action.frameTimes.length - 1 ? 'after' : 'during' }))
  }
  return frames
}

const editorInputTimeline = async (action, editor) => {
  let edited = false
  const frames = []
  for (const [index, relativeMs] of action.frameTimes.entries()) {
    if (index) await sleep(relativeMs - action.frameTimes[index - 1])
    if (index === 1 && !edited) {
      await editor.click()
      await browser.keys(['Meta', 'End'])
      await browser.keys('Enter')
      await browser.keys('Differential edit marker 2026-06-22.')
      edited = true
    }
    frames.push(await captureFrame({ outputRoot, checkpoint: action.checkpoint, index, relativeMs, kind: index === 0 ? 'before' : index === action.frameTimes.length - 1 ? 'after' : 'during' }))
  }
  return frames
}

const pointerTimeline = async (action, element) => {
  const frames = []
  for (const [index, relativeMs] of action.frameTimes.entries()) {
    if (index) await sleep(relativeMs - action.frameTimes[index - 1])
    if (index) await element.moveTo({ xOffset: index % 2 ? 20 : 0, yOffset: 0, duration: 25 })
    frames.push(await captureFrame({ outputRoot, checkpoint: action.checkpoint, index, relativeMs, kind: index === 0 ? 'before' : 'during' }))
  }
  await element.moveTo()
  return frames
}

const scrollTimeline = async (action, element) => {
  const before = Number(await element.getProperty('scrollTop'))
  const frames = []
  let scrolled = false
  for (const [index, relativeMs] of action.frameTimes.entries()) {
    if (index) await sleep(relativeMs - action.frameTimes[index - 1])
    if (index === 1 && !scrolled) {
      await browser.performActions([{
        type: 'wheel',
        id: 'differential-wheel',
        actions: [{ type: 'scroll', x: 0, y: 0, deltaX: 0, deltaY: action.delta?.y ?? 560, duration: action.durationMs ?? 400 }]
      }])
      await browser.releaseActions()
      scrolled = true
    }
    frames.push(await captureFrame({ outputRoot, checkpoint: action.checkpoint, index, relativeMs, kind: index === 0 ? 'before' : 'during' }))
  }
  const after = Number(await element.getProperty('scrollTop'))
  action.scrollChanged = before !== after
  assert.notEqual(after, before, `real scrollTop must change for ${action.id}`)
  return frames
}

const dragTimeline = async (action, source, target, beforeOrder) => {
  const frames = []
  for (const [index, relativeMs] of action.frameTimes.entries()) {
    if (index) await sleep(relativeMs - action.frameTimes[index - 1])
    if (index === 1) await source.dragAndDrop(target, { duration: action.durationMs ?? 400 })
    frames.push(await captureFrame({ outputRoot, checkpoint: action.checkpoint, index, relativeMs, kind: index === 0 ? 'before' : 'during' }))
  }
  const afterOrder = await railOrder()
  assert.notDeepEqual(afterOrder, beforeOrder, 'real rail drag/drop must change rail order')
  assert.ok(afterOrder.includes('search'), 'changed real rail order must retain Search')
  return frames
}

const runAction = async (action) => {
  const target = actionTarget(action)
  if (action.id === 'launch') {
    await requireElement('.en-library-grid', 'library grid readiness')
    const frames = []
    for (const [index, relativeMs] of action.frameTimes.entries()) {
      if (index) await sleep(relativeMs - action.frameTimes[index - 1])
      frames.push(await captureFrame({ outputRoot, checkpoint: action.checkpoint, index, relativeMs, kind: index === 0 ? 'before' : 'during' }))
    }
    return frames
  }
  if (action.id === 'move-to-alpha-card') return pointerTimeline(action, await visibleByText('.en-note-card', 'Alpha note', 'Alpha note card'))
  if (action.id === 'open-search') return clickTimeline(action, await requireElement(selectorFor(target), 'Search button'))
  if (action.id === 'search-alpha') return inputTimeline(action, await requireElement(selectorFor(target), 'Search input'), 'Alpha note')
  if (action.id === 'close-search') return keyboardTimeline(action, ['Escape'])
  if (action.id === 'navigate-all-notes') return clickTimeline(action, await requireElement(selectorFor(target), 'All notes button'))
  if (action.id === 'open-alpha-note') return clickTimeline(action, await visibleByText('.en-note-card', 'Alpha note', 'Alpha note card'))
  if (action.id === 'edit-alpha-note') {
    const editor = await requireElement(selectorFor(target), 'Muya runtime editor')
    return editorInputTimeline(action, editor)
  }
  if (action.id === 'scroll-alpha-note') return scrollTimeline(action, await requireElement('.en-editor-host .editor-component, .en-note-editor-shell', 'real note scroll container'))
  if (action.id === 'close-alpha-note') return clickTimeline(action, await requireElement(selectorFor(target), 'Close note button'))
  if (action.id === 'open-create-menu') return clickTimeline(action, await requireElement(selectorFor(target), 'Create button'))
  if (action.id === 'move-through-create-menu') return pointerTimeline(action, await visibleByText('[role="menuitem"]', 'Note', 'Note menu item'))
  if (action.id === 'close-create-menu') return keyboardTimeline(action, ['Escape'])
  if (action.id === 'drag-search-rail-item') return dragTimeline(action, await requireElement('.en-rail-nav .en-rail-icon[aria-label="Search"]', 'Search rail item'), await requireElement('.en-rail-nav .en-rail-sidebar-toggle', 'sidebar rail drop target'), await railOrder())
  throw new Error(`No real WebDriver action adapter exists for ${action.id}`)
}

const validatePostcondition = async (action, frames, previousState) => {
  const current = await stateSnapshot(vaultRoot, action)
  if (action.id === 'launch') {
    assert.deepEqual(current.visibleEntries, ['Alpha note', 'Projects'])
    assert.equal(current.route, 'library')
  } else if (action.id === 'move-to-alpha-card') {
    assert.ok(frameHashes(frames) > 1, 'hover movement must produce distinct rendered frames')
  } else if (action.id === 'open-search') {
    assert.equal(current.searchVisible, true)
    assert.equal(current.query, '')
  } else if (action.id === 'search-alpha') {
    await browser.waitUntil(async () => (await stateSnapshot(vaultRoot, action)).resultTitles.includes('Alpha note'), { timeout: 20000, timeoutMsg: 'real search result did not appear' })
    const afterSearch = await stateSnapshot(vaultRoot, action)
    assert.equal(afterSearch.query, 'Alpha note')
    assert.deepEqual(afterSearch.resultTitles, ['Alpha note'])
  } else if (action.id === 'edit-alpha-note') {
    await browser.waitUntil(async () => (await snapshotVault(vaultRoot)).some((file) => file.path === 'Alpha.md'), { timeout: 20000 })
    const source = await stateSnapshot(vaultRoot, action)
    assert.equal(source.persistedFile.contains, true)
  } else if (action.id === 'scroll-alpha-note') {
    assert.equal(action.scrollChanged, true)
    assert.ok(frameHashes(frames) > 1, 'scroll movement must produce distinct rendered frames')
  } else if (action.id === 'drag-search-rail-item') {
    assert.notDeepEqual(current.railOrder, previousState.railOrder)
  }
  return current
}

const appendRecord = async (action, frames, previousState) => {
  const state = await validatePostcondition(action, frames, previousState)
  const vault = await snapshotVault(vaultRoot)
  const checkpoint = { id: action.checkpoint, afterAction: action.id, state, vault, frames }
  manifest.actions.push({ index: action.index, id: action.id, logical: action.logical, status: 'passed', source: 'shared-scenario', postconditions: { measured: true } })
  manifest.checkpoints.push(checkpoint)
  manifest.frames.push(...frames.map((frame) => ({ ...frame, checkpoint: action.checkpoint })))
  await writeJson(path.join(outputRoot, 'checkpoints', `${action.checkpoint}.json`), checkpoint)
  return state
}

describe('Elephant Tauri embedded WebDriver differential journey', () => {
  let previousState = {}
  it('completes all shared actions through real selectors, gestures and persisted effects', async () => {
    assert.equal(actions.length, 14, 'shared differential contract must remain exactly fourteen actions')
    manifest.fixture = { id: scenario.fixture.id, roots: scenario.fixture.roots, files: await snapshotVault(vaultRoot) }
    for (const action of actions) {
      const started = Date.now()
      runLog.push({ event: 'action:start', index: action.index, id: action.id, logical: action.logical })
      try {
        const frames = await runAction(action)
        if (action.id === 'launch') manifest.provenance.real = true
        previousState = await appendRecord(action, frames, previousState)
        runLog.push({ event: 'action:done', index: action.index, id: action.id, status: 'passed', durationMs: Date.now() - started })
      } catch (error) {
        const reason = error.stack || String(error)
        manifest.actions.push({ index: action.index, id: action.id, logical: action.logical, status: 'blocked', hardFailure: reason })
        manifest.hardFailures.push({ action: action.id, checkpoint: action.checkpoint, reason })
        runLog.push({ event: 'action:failed', index: action.index, id: action.id, error: reason })
        throw error
      }
    }
    manifest.status = 'passed'
    manifest.comparison = { ready: true, reason: null, postconditions: 'measured-real-webdriver' }
  })

  after(async () => {
    manifest.status = manifest.hardFailures.length ? 'failed' : manifest.status
    await mkdir(outputRoot, { recursive: true })
    await writeJson(path.join(outputRoot, 'manifest.json'), manifest)
    await writeJson(path.join(outputRoot, 'run-log.json'), runLog)
    await writeJson(path.join(outputRoot, 'fixture-vault.json'), await snapshotVault(vaultRoot).catch(() => []))
    const serviceLogs = (await readdir(outputRoot).catch(() => [])).filter((name) => /^wdio(?:[-.].*)?\.log$/i.test(name))
    manifest.logs.service = serviceLogs
    const logText = (await Promise.all(serviceLogs.map((name) => readFile(path.join(outputRoot, name), 'utf8').catch(() => '')))).join('')
    const frontendLines = logText.split('\n').filter((line) => line.includes('[Tauri:Frontend]') || line.includes('[WDIO-FRONTEND]'))
    const backendLines = logText.split('\n').filter((line) => line.includes('[Tauri:Backend]'))
    await writeFile(path.join(outputRoot, 'frontend.log'), frontendLines.length ? `${frontendLines.join('\n')}\n` : 'No frontend log lines were emitted by the optional frontend log bridge for this run.\n', 'utf8')
    await writeFile(path.join(outputRoot, 'backend.log'), backendLines.length ? `${backendLines.join('\n')}\n` : 'No backend log lines were emitted before the WDIO session ended.\n', 'utf8')
    await writeJson(path.join(outputRoot, 'manifest.json'), manifest)
  })
})
