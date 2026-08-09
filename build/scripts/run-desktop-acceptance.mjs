#!/usr/bin/env node
import { mkdtempSync, rmSync, writeFileSync, mkdirSync } from 'node:fs'
import { join, resolve } from 'node:path'
import { spawn, execFileSync } from 'node:child_process'
import { tmpdir } from 'node:os'

const root = resolve(import.meta.dirname, '../..')
if (process.env.ELEPHANT_ACCEPTANCE_SKIP_BUILD !== '1') {
  console.log('[acceptance-runner] building the current Tauri renderer')
  execFileSync('pnpm', ['tauri:web:build'], { cwd: root, stdio: 'inherit', env: { ...process.env, ELEPHANT_ACCEPTANCE_BUILD: '1' } })
}

const fixtureRoot = mkdtempSync(join(tmpdir(), 'elephant-tauri-acceptance-'))
const vaultRoot = join(fixtureRoot, 'vault')
const configRoot = join(fixtureRoot, 'config')
const artifactRoot = join(root, 'test-results', 'acceptance')
mkdirSync(artifactRoot, { recursive: true })
mkdirSync(join(vaultRoot, '.elephantnote'), { recursive: true })
mkdirSync(join(vaultRoot, 'Getting Started'), { recursive: true })
mkdirSync(join(vaultRoot, 'Sites'), { recursive: true })
mkdirSync(configRoot, { recursive: true })
writeFileSync(join(vaultRoot, 'Acceptance.md'), '# Acceptance\n\nInitial\n', 'utf8')
writeFileSync(join(vaultRoot, 'Getting Started', 'Welcome.md'), '# Welcome\n\nElephant live rendering fixture.\n', 'utf8')
writeFileSync(join(vaultRoot, 'Sites', 'Home.md'), '# Home\n\nDesktop site acceptance fixture.\n', 'utf8')
writeFileSync(join(vaultRoot, 'Sites', 'index.html'), '<!doctype html><title>Acceptance site</title><h1>Acceptance site</h1>', 'utf8')
writeFileSync(join(vaultRoot, '.elephantnote', 'workspace.json'), JSON.stringify({ version: 1, vaultName: 'Acceptance', sidebar: [] }))
writeFileSync(join(configRoot, 'elephantnote.json'), JSON.stringify({ vaults: [], activeVaultId: null }))

let child
let endpoint
let output = ''
const { ELEPHANT_E2E_VAULT_ROOT: _ignoredE2EVaultRoot, ...processEnvWithoutE2E } = process.env
const originalHome = process.env.HOME || '/Users/sorbet'
const appPath = process.env.ELEPHANT_ACCEPTANCE_APP_PATH || './build/scripts/build_dev.sh'
const collect = (prefix, chunk) => {
  const text = chunk.toString()
  output += text
  process.stdout.write(`${prefix}${text}`)
}
const startChild = async() => {
  const outputOffset = output.length
  console.log(`[acceptance-runner] launching ${appPath}`)
  child = spawn(appPath, [], {
    cwd: root,
    env: { ...processEnvWithoutE2E, HOME: fixtureRoot, PNPM_HOME: process.env.PNPM_HOME || `${originalHome}/Library/pnpm`, RUSTUP_HOME: process.env.RUSTUP_HOME || `${originalHome}/.rustup`, CARGO_HOME: process.env.CARGO_HOME || `${originalHome}/.cargo`, ELEPHANTNOTE_CONFIG_DIR: configRoot, ELEPHANT_ACCEPTANCE_TAURI_PORT: '0' },
    detached: true,
    stdio: ['ignore', 'pipe', 'pipe']
  })
  child.stdout.on('data', (chunk) => collect('[tauri-app] ', chunk))
  child.stderr.on('data', (chunk) => collect('[tauri-app:error] ', chunk))
  const deadline = Date.now() + 120000
  while (Date.now() < deadline) {
    const match = output.slice(outputOffset).match(/ELEPHANT_ACCEPTANCE_TAURI_PORT=(\d+)/)
    if (match) {
      endpoint = `http://127.0.0.1:${Number(match[1])}`
      return endpoint
    }
    if (child.exitCode !== null) throw new Error(`Tauri exited before acceptance server started (${child.exitCode})`)
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 250))
  }
  throw new Error('Timed out waiting for the Tauri acceptance server')
}

await startChild()
const command = async(commandName, ...args) => {
  const startedAt = Date.now()
  const response = await fetch(`${endpoint}/command`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ command: commandName, args })
  })
  const body = await response.json()
  const durationMs = Date.now() - startedAt
  console.log(`[acceptance-runner] command ${commandName} ${response.ok ? 'ok' : 'failed'} ${JSON.stringify({ durationMs, requestId: body.requestId })}`)
  if (!response.ok || !body.ok) throw new Error(`${commandName} failed: ${body.error || response.status}`)
  return body.result
}

const expectCommandFailure = async(commandName, ...args) => {
  const startedAt = Date.now()
  const response = await fetch(`${endpoint}/command`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ command: commandName, args })
  })
  const body = await response.json()
  console.log(`[acceptance-runner] command ${commandName} expected-failure ${JSON.stringify({ durationMs: Date.now() - startedAt, requestId: body.requestId })}`)
  if (response.ok || body.ok || !body.error) throw new Error(`Expected ${commandName} to fail: ${JSON.stringify(body)}`)
  return body
}

const waitForDomText = async(selector, expectedText, timeoutMs = 10000) => {
  const deadline = Date.now() + timeoutMs
  let last = null
  while (Date.now() <= deadline) {
    last = await command('readDom', selector)
    if (last.exists && last.text.includes(expectedText)) return last
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 50))
  }
  throw new Error(`Timed out waiting for ${JSON.stringify(expectedText)} in ${selector}: ${JSON.stringify(last)}`)
}

const waitForDisplayedText = async(expectedText, timeoutMs = 10000) => {
  const deadline = Date.now() + timeoutMs
  let last = null
  while (Date.now() <= deadline) {
    last = await command('readDisplayed')
    if (last.displayedText.includes(expectedText)) return last
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 50))
  }
  throw new Error(`Timed out waiting for displayed text ${JSON.stringify(expectedText)}: ${JSON.stringify(last)}`)
}

const health = await fetch(`${endpoint}/health`).then((response) => response.json())
if (health.transport !== 'tauri') throw new Error(`Acceptance transport is not Tauri: ${JSON.stringify(health)}`)

const officialAddonIds = [
  'elephant.ai', 'elephant.ai-chat', 'elephant.ai-ocr', 'elephant.ai-search',
  'elephant.calendar', 'elephant.code-execution', 'elephant.codex-connection',
  'elephant.dashboard', 'elephant.google-keep-import', 'elephant.graph',
  'elephant.knowledge', 'elephant.open-models', 'elephant.recently-edited',
  'elephant.sites', 'elephant.sync', 'elephant.wiki'
]

const stopChild = () => new Promise((resolvePromise) => {
  if (!child) return resolvePromise()
  if (child.exitCode !== null) return resolvePromise()
  const finish = () => resolvePromise()
  child.once('close', finish)
  try {
    process.kill(-child.pid, 'SIGTERM')
  } catch {
    child.kill('SIGTERM')
  }
  setTimeout(finish, 5000)
})

let result
try {
  const emptyVaultUi = await command('readDom', '.en-empty-card')
  if (!emptyVaultUi.exists || !emptyVaultUi.text.includes('Choose your first vault')) {
    throw new Error(`First-run vault picker is not visible: ${JSON.stringify(emptyVaultUi)}`)
  }
  await command('selectVault', vaultRoot)
  if (process.env.ELEPHANT_ACCEPTANCE_UI_ONLY === '1') {
    const createToolbar = await command('waitFor', '.en-library-toolbar', 10000)
    const createButton = await command('readDom', '.en-create-button-primary')
    if (!createToolbar.exists || !createButton.exists || !createButton.visible || createButton.text.trim() !== 'Create') {
      throw new Error(`Create trigger is not visible or has an unexpected label: ${JSON.stringify({ createToolbar, createButton })}`)
    }
    await command('click', '.en-create-button-primary')
    const createMenu = await command('waitFor', '.en-create-menu-popover', 10000)
    const createOptions = await command('readDom', '.en-create-menu-popover')
    const menuItemCount = (createOptions.html.match(/role="menuitem"/g) || []).length
    if (!createMenu.exists || createMenu.attributes.role !== 'menu' || !createOptions.text.includes('Note') || !createOptions.text.includes('Drawing') || !createOptions.text.includes('Folder') || menuItemCount !== 3) {
      throw new Error(`Create menu is incomplete: ${JSON.stringify({ createMenu, createOptions, menuItemCount })}`)
    }
    await command('click', '.en-create-menu-option:nth-of-type(2)')
    const drawingDialog = await command('waitFor', '[data-testid="excalidraw-dialog"]', 15000)
    const drawingCanvas = await command('waitFor', '.en-excalidraw-canvas canvas', 15000)
    if (!drawingDialog.exists || !drawingCanvas.exists) throw new Error(`Drawing option did not open the real Excalidraw canvas: ${JSON.stringify({ drawingDialog, drawingCanvas })}`)
    await command('click', '[data-testid="excalidraw-close"]')
    await command('waitUntilGone', '[data-testid="excalidraw-dialog"]', 10000)
    const uiResult = { createToolbar, createButton, createMenu, createOptions, menuItemCount, drawingDialog, drawingCanvas }
    result = { emptyVaultUi: null, uiResult }
    writeFileSync(join(artifactRoot, 'latest.json'), JSON.stringify({ at: new Date().toISOString(), runtime: 'tauri', result }, null, 2))
    writeFileSync(join(artifactRoot, 'latest-tauri.log'), output, 'utf8')
    console.log(`[acceptance-runner] create menu UI scenario passed ${JSON.stringify({ menuItems: 3, drawingCanvas: drawingCanvas.exists })}`)
    await stopChild()
    rmSync(fixtureRoot, { recursive: true, force: true })
    process.exit(0)
  }
  const capabilities = await command('capabilities')
  const addonState = await command('addonState')
  if (capabilities.runtime !== 'tauri' || !capabilities.commands.includes('invokeTauri')) throw new Error(`Acceptance capabilities are incomplete: ${JSON.stringify(capabilities)}`)
  if (!Array.isArray(addonState.addons) || !Array.isArray(addonState.resources) || !Array.isArray(addonState.actions)) throw new Error(`Addon acceptance surface is incomplete: ${JSON.stringify(addonState)}`)
  const installedOfficialAddons = {}
  for (const addonId of officialAddonIds) {
    installedOfficialAddons[addonId] = await command('installOfficialAddon', addonId)
  }
  const installedAddonState = await command('addonState')
  const dashboardAddon = installedAddonState.addons.find((entry) => entry.id === 'elephant.dashboard')
  const keepAddon = installedAddonState.addons.find((entry) => entry.id === 'elephant.google-keep-import')
  const sitesAddon = installedAddonState.addons.find((entry) => entry.id === 'elephant.sites')
  const syncAddon = installedAddonState.addons.find((entry) => entry.id === 'elephant.sync')
  if (!dashboardAddon || !keepAddon || !sitesAddon || !syncAddon || officialAddonIds.some((id) => !installedAddonState.addons.some((entry) => entry.id === id))) {
    throw new Error(`Official addon installation did not register all requested addons: ${JSON.stringify(installedAddonState)}`)
  }
  const enabledOfficialAddons = {}
  for (const addonId of officialAddonIds) {
    enabledOfficialAddons[addonId] = await command('enableAddon', addonId)
  }
  const enabledAddonState = await command('addonState')
  const addonCoverage = Object.fromEntries(officialAddonIds.map((id) => {
    const entry = enabledAddonState.addons.find((candidate) => candidate.id === id)
    return [id, { enabled: entry?.enabled === true, status: entry?.status || '', error: entry?.error || null }]
  }))
  if (officialAddonIds.some((id) => addonCoverage[id].enabled !== true || addonCoverage[id].error)) {
    throw new Error(`Official addon enablement matrix failed: ${JSON.stringify(addonCoverage)}`)
  }
  const nativeRuntimeProbes = {}
  nativeRuntimeProbes['elephant.ai-ocr.sidecar'] = await command('addonNativeStatus', 'elephant.ai-ocr', 'sidecar')
  for (const addonId of ['elephant.code-execution', 'elephant.codex-connection', 'elephant.knowledge', 'elephant.open-models', 'elephant.sync']) {
    nativeRuntimeProbes[`${addonId}.service`] = await command('addonNativeStatus', addonId, 'service')
  }
  if (nativeRuntimeProbes['elephant.ai-ocr.sidecar']?.available !== true || Object.entries(nativeRuntimeProbes).some(([key, probe]) => key.endsWith('.service') && (probe?.running !== true || probe?.error))) {
    throw new Error(`Native addon runtime probe failed: ${JSON.stringify(nativeRuntimeProbes)}`)
  }
  nativeRuntimeProbes['elephant.code-execution.service-call'] = await command('addonNativeCall', 'elephant.code-execution', 'interpreter.status', { executable: 'python3' }, { service: true })
  nativeRuntimeProbes['elephant.codex-connection.service-call'] = await command('addonNativeCall', 'elephant.codex-connection', 'codex.status', {}, { service: true })
  nativeRuntimeProbes['elephant.knowledge.service-call'] = await command('addonNativeCall', 'elephant.knowledge', 'knowledge.status', {}, { service: true })
  nativeRuntimeProbes['elephant.open-models.service-call'] = await command('addonNativeCall', 'elephant.open-models', 'models.status', {}, { service: true })
  nativeRuntimeProbes['elephant.sync.service-call'] = await command('addonNativeCall', 'elephant.sync', 'sync.status', {}, { service: true })
  if (Object.entries(nativeRuntimeProbes).filter(([key]) => key.endsWith('.service-call')).some(([, probe]) => !probe || typeof probe !== 'object')) {
    throw new Error(`Native addon service call probe returned an invalid result: ${JSON.stringify(nativeRuntimeProbes)}`)
  }
  const addonResourceProbes = {}
  for (const [name, method, payload] of [
    ['ai.config', 'get'],
    ['ai.inference', 'status'],
    ['ocr', 'status'],
    ['search.provider', 'status'],
    ['calendar.provider', 'status'],
    ['knowledge.provider', 'status'],
    ['models.provider', 'status'],
    ['wiki.provider', 'status'],
    ['import.google-keep', 'parse', { name: 'probe.json', content: { title: 'Resource probe', textContent: 'ok' } }]
  ]) {
    addonResourceProbes[`${name}.${method}`] = await command('invokeAddonResource', name, method, payload)
  }
  addonResourceProbes['ai.config.listProviders'] = await command('invokeAddonResource', 'ai.config', 'listProviders')
  addonResourceProbes['ai.inference.listProviders'] = await command('invokeAddonResource', 'ai.inference', 'listProviders')
  addonResourceProbes['search.provider.query'] = await command('invokeAddonResource', 'search.provider', 'query', 'Acceptance')
  addonResourceProbes['calendar.provider.importIcs'] = await command('invokeAddonResource', 'calendar.provider', 'importIcs', 'BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:acceptance-event\nSUMMARY:Acceptance event\nDTSTART:20260719T100000Z\nDTEND:20260719T110000Z\nEND:VEVENT\nEND:VCALENDAR', 'acceptance.ics')
  addonResourceProbes['calendar.provider.list'] = await command('invokeAddonResource', 'calendar.provider', 'list')
  addonResourceProbes['knowledge.provider.search'] = await command('invokeAddonResource', 'knowledge.provider', 'search', { query: 'Acceptance', limit: 10 })
  addonResourceProbes['knowledge.provider.graph'] = await command('invokeAddonResource', 'knowledge.provider', 'graph', { includeSuggestions: true })
  addonResourceProbes['models.provider.list'] = await command('invokeAddonResource', 'models.provider', 'list')
  addonResourceProbes['models.provider.active'] = await command('invokeAddonResource', 'models.provider', 'active')
  addonResourceProbes['wiki.provider.list'] = await command('invokeAddonResource', 'wiki.provider', 'list')
  const calendarCleared = await command('invokeAddonResource', 'calendar.provider', 'clear')
  addonResourceProbes['calendar.provider.clear'] = calendarCleared
  if (!Array.isArray(addonResourceProbes['ai.config.listProviders']) || !Array.isArray(addonResourceProbes['ai.inference.listProviders']) || !Array.isArray(addonResourceProbes['calendar.provider.list']) || !Array.isArray(addonResourceProbes['models.provider.list']) || !Array.isArray(addonResourceProbes['wiki.provider.list']) || !Array.isArray(calendarCleared)) {
    throw new Error(`Addon resource functional probe returned an unexpected shape: ${JSON.stringify(addonResourceProbes)}`)
  }
  const addonActionProbes = {
    graphRebuild: await command('runAddonAction', 'elephant.graph.rebuild'),
    knowledgeRebuild: await command('runAddonAction', 'elephant.knowledge.rebuild'),
    wikiGenerateProposals: await command('runAddonAction', 'elephant.wiki.generate-proposals')
  }
  addonActionProbes.chatToggle = await command('runAddonAction', 'elephant.ai-chat.toggle')
  const chatPanel = await command('waitFor', '.elephant-chat-package', 10000)
  addonActionProbes.calendarOpen = await command('runAddonAction', 'elephant.calendar.open')
  const calendarPanel = await command('waitFor', '.elephant-calendar-package', 10000)
  addonActionProbes.graphOpen = await command('runAddonAction', 'elephant.graph.open')
  const graphPanel = await command('waitFor', '.elephant-graph-package', 10000)
  if (!chatPanel.exists || !calendarPanel.exists || !graphPanel.exists) throw new Error(`Addon view actions did not render their functional surfaces: ${JSON.stringify({ chatPanel, calendarPanel, graphPanel })}`)
  const dashboardAction = await command('runAddonAction', 'elephant.dashboard.open')
  const dashboardNote = await command('readNote', '.elephantnote/Dashboard.md')
  const keepImport = await command('runAddonAction', 'elephant.google-keep-import.import', [{ name: 'acceptance.json', content: { title: 'Imported acceptance', textContent: 'Google Keep round trip', listContent: [{ text: 'done', isChecked: true }] } }])
  const keepNote = await command('readNote', 'Imported/Google Keep/Imported acceptance.md')
  const siteGenerated = await command('invokeAddonResource', 'sites.provider', 'previewFolder', { relativePath: 'Sites' })
  const siteStatus = await command('invokeAddonResource', 'sites.provider', 'status')
  const siteStopped = await command('invokeAddonResource', 'sites.provider', 'stop', siteStatus?.siteId)
  const syncStatus = await command('invokeAddonResource', 'sync.native-service', 'status')
  if (!dashboardNote.content || !keepImport.imported || !keepNote.content.includes('Google Keep round trip') || siteGenerated?.running !== true || siteStatus?.siteId !== siteGenerated.siteId || siteStopped?.stopped !== true || syncStatus?.state === 'error') {
    throw new Error(`Official addon functional scenario failed: ${JSON.stringify({ dashboardAction, dashboardNote, keepImport, keepNote, siteGenerated, siteStatus, siteStopped, syncStatus })}`)
  }
  const platform = await command('invokeTauri', 'tauri_platform_info')
  const vaults = await command('invokeTauri', 'tauri_vaults_get')
  const directory = await command('invokeTauri', 'tauri_directory_list', { relativePath: '', offset: 0, limit: 1000, includePreview: false })
  const attachments = await command('invokeTauri', 'tauri_attachments_list')
  const drawings = await command('invokeTauri', 'tauri_drawings_list')
  const features = await command('invokeTauri', 'tauri_features_get')
  const searchStatus = await command('invokeTauri', 'tauri_search_status')
  const atomicFeatures = await command('invokeTauri', 'tauri_atomic_features_list')
  const localBackendProbes = {}
  for (const backendCommand of ['healthcheck', 'tauri_prefs_all', 'tauri_user_data_all', 'tauri_recents_list', 'tauri_keybindings_get', 'tauri_sources_list', 'tauri_addons_list', 'tauri_addons_list_full', 'tauri_official_addons_catalog_list']) {
    localBackendProbes[backendCommand] = await command('invokeTauri', backendCommand)
  }
  if (platform?.desktop !== true || !Array.isArray(directory) || !Array.isArray(attachments) || !Array.isArray(drawings) || typeof features !== 'object' || typeof searchStatus !== 'object' || typeof atomicFeatures !== 'object') {
    throw new Error(`Backend inventory probe failed: ${JSON.stringify({ platform, vaults, directory, attachments, drawings, features, searchStatus, atomicFeatures })}`)
  }
  const liveDiskBeforeOpen = await command('readNote', 'Getting Started/Welcome.md')
  if (!liveDiskBeforeOpen.content.includes('Elephant live rendering fixture.')) {
    throw new Error(`Live rendering fixture is not readable from the backend: ${JSON.stringify(liveDiskBeforeOpen)}`)
  }
  const liveNote = await command('openNote', 'Getting Started/Welcome.md')
  if (!liveNote.markdown.includes('Elephant')) throw new Error(`Live rendering fixture did not open the expected body: ${JSON.stringify(liveNote)}`)
  const editorSelector = '[contenteditable="true"].editor-component'
  const liveEditorSelector = editorSelector
  await command('readDom', liveEditorSelector)
  const liveTextSelector = `${editorSelector} .ag-paragraph-content`
  const liveText = await command('readDom', liveTextSelector)
  await command('selectText', liveTextSelector, liveText.text.length, liveText.text.length)
  await command('insertText', liveEditorSelector, ' Live Tauri body edit 9173.')
  let liveState = null
  const liveEditDeadline = Date.now() + 5000
  while (Date.now() <= liveEditDeadline) {
    liveState = await command('readState')
    if (liveState.markdown.includes('Live Tauri body edit 9173.')) break
    await new Promise((resolve) => setTimeout(resolve, 50))
  }
  if (!liveState.markdown.includes('Live Tauri body edit 9173.')) {
    throw new Error(`Live note edit did not reach document Markdown: ${JSON.stringify(liveState)}`)
  }
  await command('save')
  const liveDisk = await command('readNote', 'Getting Started/Welcome.md')
  if (!liveDisk.content.includes('Live Tauri body edit 9173.')) {
    throw new Error(`Live note edit did not reach disk: ${JSON.stringify(liveDisk)}`)
  }
  const initial = await command('openNote', 'Acceptance.md')
  if (!initial.notePath.endsWith('Acceptance.md')) throw new Error(`Wrong opened note: ${initial.notePath}`)
  if (initial.sourceCode || initial.rustEditorPresent || initial.codeMirrorPresent || !initial.displayedHtml.includes('ag-editor-id')) {
    throw new Error(`Editor is not running the Muya JS production surface: ${JSON.stringify({ sourceCode: initial.sourceCode, rustEditorPresent: initial.rustEditorPresent, codeMirrorPresent: initial.codeMirrorPresent, hasMuyaDom: initial.displayedHtml.includes('ag-editor-id') })}`)
  }
  const renderMarkdown = [
    '# Acceptance',
    '',
    '# Notes',
    '',
    '## Tasks',
    '',
    '- [ ] Review the note',
    '',
    '```text',
    'literal code',
    '```',
    '',
    '![drawing](../assets/excalidraw.png)',
    '',
    '# ddd'
  ].join('\n')
  await command('setMarkdown', renderMarkdown)
  await waitForDomText(`${editorSelector} h2 .ag-plain-text`, 'Tasks')
  await waitForDomText(`${editorSelector} pre.ag-fence-code`, 'literal code')
  const renderedFirstHeading = await command('readDom', `${editorSelector} h1:nth-of-type(1) .ag-plain-text`)
  const renderedSecondHeading = await command('readDom', `${editorSelector} h1:nth-of-type(2) .ag-plain-text`)
  const renderedMarkdown = await command('readDom', editorSelector)
  const renderedTask = await command('readDom', `${editorSelector} input[type="checkbox"]`)
  const renderedImage = await command('readDom', `${editorSelector} .ag-inline-image[data-raw="![drawing](../assets/excalidraw.png)"]`)
  if (renderedFirstHeading.text !== 'Notes' || renderedSecondHeading.text !== 'ddd' || !renderedTask.exists || !renderedImage.exists || renderedMarkdown.text.includes('```text') || renderedMarkdown.text.includes('![drawing]')) {
    throw new Error(`Markdown rendered in the wrong order, as raw text, or lost semantic nodes: ${JSON.stringify({ renderedFirstHeading, renderedSecondHeading, renderedTask, renderedImage, text: renderedMarkdown.text, html: renderedMarkdown.html })}`)
  }
  const manualSaveMarkdown = '# Acceptance\n\nManual CmdOrCtrl+S save from the real Tauri command runner.'
  await command('setMarkdown', manualSaveMarkdown)
  await waitForDomText(`${editorSelector} .ag-paragraph-content`, 'Manual CmdOrCtrl+S save from the real Tauri command runner.')
  const editorDom = await command('readDom', editorSelector)
  if (!editorDom.exists) throw new Error(`Muya editor DOM is missing before keyboard probe: ${JSON.stringify(editorDom)}`)
  const editorText = await command('readDom', `${editorSelector} .ag-paragraph-content`)
  await command('selectText', `${editorSelector} .ag-paragraph-content`, editorText.text.length, editorText.text.length)
  await command('insertText', editorSelector, ' Tauri keyboard probe 9173.')
  let keyboardState = null
  const keyboardDeadline = Date.now() + 5000
  while (Date.now() <= keyboardDeadline) {
    keyboardState = await command('readState')
    if (keyboardState.markdown.includes('Tauri keyboard probe 9173.')) break
    await new Promise((resolve) => setTimeout(resolve, 50))
  }
  if (!keyboardState.markdown.includes('Tauri keyboard probe 9173.')) {
    throw new Error(`Muya editor keyboard input did not update Markdown state: ${JSON.stringify(keyboardState)}`)
  }
  await command('executeCommand', 'file.save')
  let savedFromDisk
  const saveDeadline = Date.now() + 15000
  while (Date.now() <= saveDeadline) {
    savedFromDisk = await command('readNote', 'Acceptance.md')
    if (savedFromDisk.content.includes('Tauri keyboard probe 9173.')) break
    await new Promise((resolvePromise) => setTimeout(resolvePromise, 100))
  }
  if (!savedFromDisk?.content.includes('Tauri keyboard probe 9173.')) {
    throw new Error(`Manual file.save did not persist keyboard input to disk: ${JSON.stringify(savedFromDisk)}`)
  }
  const saved = await command('readState')
  if (!saved.isSaved || !saved.markdown.includes('Manual CmdOrCtrl+S save')) throw new Error('Manual Markdown save state is incorrect')
  const displayed = await waitForDisplayedText('Manual CmdOrCtrl+S save from the real Tauri command runner.')
  if (!displayed.displayedText.includes('Manual CmdOrCtrl+S save')) throw new Error('Displayed Markdown does not contain manually saved content')
  await command('setMarkdown', '# Acceptance\n\nEdited by the real Tauri command runner.')
  const acceptanceSaved = await command('save')
  if (!acceptanceSaved.isSaved || !acceptanceSaved.markdown.includes('real Tauri command runner')) throw new Error('Acceptance save state is incorrect')
  await command('setMarkdown', '# Code acceptance\n\n```python\nprint("code execution acceptance")\n```')
  await waitForDomText(`${editorSelector} .ag-code-content`, 'print("code execution acceptance")')
  const codeRunButton = await command('waitFor', '.elephant-physical-code-run', 10000)
  if (!codeRunButton.exists) throw new Error(`Code execution addon did not decorate the code block: ${JSON.stringify(codeRunButton)}`)
  await command('click', '.elephant-physical-code-run')
  const codeOutput = await command('waitFor', '.elephant-physical-code-output[data-exit-code="0"]', 15000)
  if (!codeOutput.text.includes('code execution acceptance')) throw new Error(`Code execution output is incorrect: ${JSON.stringify(codeOutput)}`)
  const citationSourceText = 'Edited by the real Tauri command runner.'
  await command('setMarkdown', `# Acceptance\n\n${citationSourceText}`)
  await waitForDomText(`${editorSelector} .ag-paragraph-content`, citationSourceText)
  await command('save')
  const citationSelection = await command('selectText', `${editorSelector} .ag-paragraph-content`, 0, citationSourceText.length)
  const citationSelectionAction = await command('waitFor', '[data-elephant-citation-selection-action]', 10000)
  await command('click', '[data-elephant-citation-selection-action]')
  const citationFeedback = await command('waitFor', '[data-elephant-citation-feedback]', 10000)
  if (!citationSelection.text || !citationFeedback.text.includes('Citation copiée')) throw new Error(`Citation selection flow failed: ${JSON.stringify({ citationSelection, citationFeedback })}`)
  await command('createNote', 'Acceptance', 'Citation target.md')
  await command('openNote', 'Acceptance/Citation target.md')
  const citationBufferItem = await command('waitFor', '[data-elephant-citation-buffer-item]', 10000)
  await command('click', '[data-elephant-citation-buffer-item]')
  const citationPasted = await command('readState')
  const citationHasText = citationPasted.markdown.includes('Edited by the real Tauri command runner')
  const citationHasLink = citationPasted.markdown.includes('](</Acceptance.md#quote=')
  if (!citationHasText || !citationHasLink) {
    throw new Error(`Citation paste did not create a linked quote: ${JSON.stringify({ citationHasText, citationHasLink, citationPasted })}`)
  }
  await command('contextClick', '[data-elephant-citation-buffer-item]')
  const citationContext = await command('waitFor', '[data-elephant-citation-context]', 10000)
  await command('click', '[aria-label^="Supprimer la citation"]')
  await command('waitUntilGone', '[data-elephant-citation-buffer-item]', 10000)
  const dom = await command('readDom', editorSelector)
  if (!dom.exists || !dom.text.includes('Edited by the real Tauri command runner')) throw new Error(`Displayed editor DOM is incomplete: ${JSON.stringify({ exists: dom.exists, textLength: dom.text.length })}`)
  const sidebarInitial = await command('readDom', '.en-body')
  await command('click', '.en-rail-sidebar-toggle')
  const sidebarToggled = await command('readDom', '.en-body')
  await command('click', '.en-rail-sidebar-toggle')
  const sidebarRestored = await command('readDom', '.en-body')
  const initialHidden = sidebarInitial.attributes.class?.includes('en-sidebar-hidden')
  const toggledHidden = sidebarToggled.attributes.class?.includes('en-sidebar-hidden')
  const restoredHidden = sidebarRestored.attributes.class?.includes('en-sidebar-hidden')
  if (!sidebarInitial.exists || toggledHidden === initialHidden || restoredHidden !== initialHidden) throw new Error(`Sidebar toggle round-trip failed: ${JSON.stringify({ sidebarInitial, sidebarToggled, sidebarRestored })}`)
  await command('click', '.en-rail-icon[aria-label="Search"]')
  await command('waitFor', '.en-search-bar-input', 10000)
  await command('fill', '.en-search-bar-input', 'Edited by the real Tauri command runner')
  await command('press', '.en-search-bar-input', 'Enter')
  const searchUi = await command('waitFor', '.en-search-results', 10000)
  if (!searchUi.text.includes('Acceptance.md') && !searchUi.text.includes('Acceptance')) throw new Error(`Sidebar search did not find the edited note or its concept result: ${JSON.stringify(searchUi)}`)
  await command('fill', '.en-search-bar-input', 'no-such-acceptance-note-9173')
  await command('press', '.en-search-bar-input', 'Enter')
  const searchEmptyUi = await command('waitFor', '.en-search-empty', 10000)
  if (!searchEmptyUi.text.trim()) throw new Error(`Sidebar empty search state is blank: ${JSON.stringify(searchEmptyUi)}`)
  await command('press', '.en-search-bar-input', 'Escape')
  await command('press', '.en-search-bar-input', 'Escape')
  await command('waitUntilGone', '.en-search-bar-input', 10000)
  await command('click', '[aria-label="Close note"]')
  await command('waitFor', '.en-library-grid', 10000)
  const afterClose = await command('readState')
  if (afterClose.notePath?.endsWith('Acceptance.md')) throw new Error(`Closed note still active: ${JSON.stringify(afterClose)}`)
  await command('openNote', 'Acceptance.md')
  await command('click', '[aria-label="Pin note"]')
  const pinned = await command('readDom', '[aria-label="Unpin note"]')
  await command('click', '[aria-label="Unpin note"]')
  if (!pinned.exists || !pinned.visible) throw new Error(`Pin note action did not update the editor state: ${JSON.stringify(pinned)}`)
  await command('click', '[aria-label="Settings"]')
  await command('waitFor', '.en-settings-panel', 10000)
  await command('fill', '[aria-label="Search all settings"]', 'autosave')
  const settingsSearch = await command('waitFor', '.en-settings-search-results', 10000)
  if (!settingsSearch.text.includes('Autosave')) throw new Error(`Settings search did not find Autosave: ${JSON.stringify(settingsSearch)}`)
  await command('fill', '[aria-label="Search all settings"]', '')
  await command('click', '.en-settings-nav button:first-child')
  await command('waitFor', '.en-settings-content[data-active-section="appearance"]', 10000)
  const themeBefore = await command('readDom', '.en-shell')
  const initiallyDark = themeBefore.attributes.class?.includes('en-theme-dark')
  const themeToggleSelector = initiallyDark ? '.en-segmented button:nth-child(1)' : '.en-segmented button:nth-child(2)'
  const themeRestoreSelector = initiallyDark ? '.en-segmented button:nth-child(2)' : '.en-segmented button:nth-child(1)'
  await command('click', themeToggleSelector)
  const themeToggled = await command('readDom', '.en-shell')
  await command('click', themeRestoreSelector)
  const themeRestored = await command('readDom', '.en-shell')
  const toggledDark = themeToggled.attributes.class?.includes('en-theme-dark')
  if (toggledDark === initiallyDark || themeRestored.attributes.class !== themeBefore.attributes.class) throw new Error(`Theme round-trip failed: ${JSON.stringify({ themeBefore, themeToggled, themeRestored })}`)
  await command('click', '[aria-label="Close settings"]')
  await command('waitUntilGone', '.en-settings-panel', 10000)
  await command('click', '[aria-label="Close note"]')
  await command('waitFor', '.en-library-grid', 10000)
  const createToolbar = await command('readDom', '.en-library-toolbar')
  if (!createToolbar.exists || !createToolbar.visible) throw new Error(`Library create toolbar is not visible: ${JSON.stringify(createToolbar)}`)
  await command('click', '.en-create-button-primary')
  const createMenu = await command('waitFor', '.en-create-menu-popover', 10000)
  const createOptions = await command('readDom', '.en-create-menu-popover')
  if (!createMenu.exists || !createOptions.text.includes('Note') || !createOptions.text.includes('Drawing') || !createOptions.text.includes('Folder') || !createOptions.html.includes('role="menuitem"')) {
    throw new Error(`Create menu is incomplete: ${JSON.stringify({ createMenu, createOptions })}`)
  }
  await command('press', '.en-create-button-primary', 'Escape')
  await command('waitUntilGone', '.en-create-menu-popover', 10000)
  const listBefore = await command('readDom', '.en-library-grid')
  await command('click', '[aria-label="List view"]')
  const listView = await command('readDom', '.en-library-grid')
  await command('click', '[aria-label="Grid view"]')
  await command('fill', '.en-library-actions .en-select', 'title')
  const sortedLibrary = await command('readDom', '.en-library-grid')
  const sortControl = await command('readDom', '.en-library-actions .en-select')
  if (!listView.attributes.class?.includes('list') || !sortedLibrary.exists || sortControl.value !== 'title') throw new Error(`Library view/sort round-trip failed: ${JSON.stringify({ listBefore, listView, sortedLibrary, sortControl })}`)
  const navigationCycles = []
  for (let cycle = 1; cycle <= 3; cycle += 1) {
    await command('click', '.en-rail-sidebar-toggle')
    const hidden = await command('readDom', '.en-body')
    await command('click', '.en-rail-sidebar-toggle')
    const restored = await command('readDom', '.en-body')
    await command('click', '[aria-label="Settings"]')
    await command('waitFor', '.en-settings-panel', 10000)
    await command('click', '[aria-label="Close settings"]')
    await command('waitUntilGone', '.en-settings-panel', 10000)
    await command('click', '.en-rail-icon[aria-label="Search"]')
    await command('waitFor', '.en-search-bar-input', 10000)
    await command('press', '.en-search-bar-input', 'Escape')
    await command('waitUntilGone', '.en-search-bar-input', 10000)
    if (hidden.attributes.class?.includes('en-sidebar-hidden') !== true || restored.attributes.class?.includes('en-sidebar-hidden') !== false) throw new Error(`Navigation cycle ${cycle} did not restore sidebar: ${JSON.stringify({ hidden, restored })}`)
    navigationCycles.push({ cycle, sidebarRestored: true, settingsRestored: true, searchRestored: true })
  }
  const markdownProbe = '# Probe\n\n[link](Target.md)\n\n**body**'
  const renderedProbe = await command('invokeTauri', 'tauri_markdown_render_html', { markdown: markdownProbe })
  const plainProbe = await command('invokeTauri', 'tauri_markdown_to_text', { markdown: markdownProbe })
  const linksProbe = await command('invokeTauri', 'tauri_markdown_extract_links', { markdown: markdownProbe })
  if (!renderedProbe?.html?.includes('<h1>Probe</h1>') || !plainProbe?.text?.includes('body') || !linksProbe?.links?.some((link) => JSON.stringify(link).includes('Target.md'))) {
    throw new Error(`Markdown command probe failed: ${JSON.stringify({ renderedProbe, plainProbe, linksProbe })}`)
  }
  const excalidraw = await command('openExcalidraw', 'acceptance.excalidraw.png')
  if (!excalidraw.open || !excalidraw.hasCanvas || excalidraw.hasError) throw new Error(`Excalidraw did not open correctly: ${JSON.stringify(excalidraw)}`)
  const excalidrawClosed = await command('closeExcalidraw')
  if (excalidrawClosed.open) throw new Error('Excalidraw did not close correctly')
  const created = await command('createNote', 'Acceptance', 'Created.md')
  await command('openNote', 'Acceptance/Created.md')
  await command('setMarkdown', '# Created\n\nSecond real Tauri scenario.')
  const createdSaved = await command('save')
  const disk = await command('readNote', 'Acceptance/Created.md')
  const notes = await command('listNotes', 'Acceptance')
  if (!createdSaved.isSaved || !disk.content.includes('Second real Tauri scenario.') || !notes.some((entry) => entry.path === 'Acceptance/Created.md')) {
    throw new Error(`Created note assertion failed: ${JSON.stringify({ createdSaved, disk, notes })}`)
  }
  const folder = await command('invokeTauri', 'tauri_folders_create', { relativePath: 'Projects' })
  const lifecycle = await command('invokeTauri', 'tauri_notes_create', { relativePath: 'Projects', filename: 'Lifecycle.md', title: 'Lifecycle' })
  await command('invokeTauri', 'tauri_entries_rename', { relativePath: 'Projects/Lifecycle.md', title: 'Renamed' })
  await command('invokeTauri', 'tauri_entries_move', { relativePath: 'Projects/Renamed.md', targetDirectoryPath: 'Acceptance' })
  const moved = await command('readNote', 'Acceptance/Renamed.md')
  if (!folder?.path || !lifecycle?.path || !moved.content.includes('# Lifecycle')) throw new Error(`Vault structure lifecycle failed: ${JSON.stringify({ folder, lifecycle, moved })}`)
  const attachmentWrite = await command('invokeTauri', 'tauri_attachments_write_text', { relativePath: 'acceptance.txt', content: 'asset round trip' })
  const attachmentList = await command('invokeTauri', 'tauri_attachments_list')
  if (!attachmentWrite?.ok || !attachmentList.some((entry) => JSON.stringify(entry).includes('acceptance.txt'))) throw new Error(`Attachment round-trip failed: ${JSON.stringify({ attachmentWrite, attachmentList })}`)
  await command('invokeTauri', 'tauri_entries_delete', { relativePath: 'Acceptance/Renamed.md' })
  const afterDelete = await command('invokeTauri', 'tauri_directory_list', { relativePath: 'Acceptance', offset: 0, limit: 1000, includePreview: false })
  if (afterDelete.some((entry) => entry.path === 'Acceptance/Renamed.md')) throw new Error(`Deleted note is still listed: ${JSON.stringify(afterDelete)}`)
  const search = await command('invokeTauri', 'tauri_search_query', { params: { query: 'Second real Tauri scenario', limit: 10 } })
  if (!Array.isArray(search) || !search.some((entry) => JSON.stringify(entry).includes('Second real Tauri scenario'))) throw new Error(`Search command did not find the created note: ${JSON.stringify(search)}`)
  const drawing = await command('invokeTauri', 'tauri_drawings_create', { title: 'Acceptance drawing' })
  console.log(`[acceptance-runner] drawing-created ${JSON.stringify(drawing)}`)
  const drawingRead = await command('invokeTauri', 'tauri_drawings_read', { relativePath: drawing.path })
  if (drawingRead?.title !== 'Acceptance drawing' && drawingRead?.scene?.title !== 'Acceptance drawing') throw new Error(`Drawing round-trip failed: ${JSON.stringify({ drawing, drawingRead })}`)
  const drawingScene = { ...drawingRead, acceptanceMarker: 'written-and-read' }
  await command('invokeTauri', 'tauri_drawings_write', { relativePath: drawing.path, scene: drawingScene })
  const drawingWritten = await command('invokeTauri', 'tauri_drawings_read', { relativePath: drawing.path })
  if (drawingWritten?.acceptanceMarker !== 'written-and-read') throw new Error(`Drawing write round-trip failed: ${JSON.stringify(drawingWritten)}`)
  const expectedFailure = await expectCommandFailure('doesNotExist')
  const invalidPathFailure = await expectCommandFailure('readNote', '../outside.md')
  const missingResourceFailure = await expectCommandFailure('invokeAddonResource', 'missing.resource', 'status')
  if (!invalidPathFailure.error || !missingResourceFailure.error) throw new Error('Expected desktop error probes did not return structured errors')
  const logs = await command('logs')
  if (!logs.some((entry) => entry.event === 'transport:command:done') || !logs.some((entry) => entry.event === 'transport:command:error')) throw new Error('Acceptance command completion/error was not logged')

  await stopChild()
  await startChild()
  const restartedHealth = await fetch(`${endpoint}/health`).then((response) => response.json())
  if (restartedHealth.transport !== 'tauri') throw new Error(`Restarted acceptance transport is not Tauri: ${JSON.stringify(restartedHealth)}`)
  const restartedCapabilities = await command('capabilities')
  const restartedState = await command('readState')
  const restartedNotes = await command('listNotes')
  const restartedNote = await command('readNote', 'Acceptance.md')
  if (restartedState.activeVault !== vaultRoot || !restartedNotes.some((entry) => entry.path === 'Acceptance.md') || !restartedNote.content.includes('real Tauri command runner')) {
    throw new Error(`Vault/content did not persist after restart: ${JSON.stringify({ restartedState, restartedNotes, restartedNote })}`)
  }
  const restartPersistence = { health: restartedHealth, capabilities: restartedCapabilities, state: restartedState, notes: restartedNotes, note: restartedNote }
  const packagedRun = Boolean(process.env.ELEPHANT_ACCEPTANCE_APP_PATH)
  const catalogSource = output.includes('[official-addon-catalog] source=bundled') ? 'bundled' : 'local-or-remote'
  if (packagedRun && catalogSource !== 'bundled') throw new Error('Packaged acceptance did not use the bundled official addon catalogue')
  if (packagedRun && output.includes('Addon service executable is unavailable')) throw new Error('Packaged acceptance reproduced the missing addon service executable regression')
  result = { emptyVaultUi, initial, saved, created, createdSaved, disk, displayed, codeRunButton, codeOutput, citationSelection, citationSelectionAction, citationFeedback, citationBufferItem, citationPasted, citationContext, dom, chatPanel, calendarPanel, graphPanel, sidebarInitial, sidebarToggled, sidebarRestored, searchUi, searchEmptyUi, afterClose, createToolbar, createMenu, createOptions, pinned, settingsSearch, themeBefore, themeToggled, themeRestored, listBefore, listView, sortedLibrary, navigationCycles, capabilities, addonState, installedOfficialAddons, installedAddonState, enabledOfficialAddons, enabledAddonState, addonCoverage, nativeRuntimeProbes, addonResourceProbes, addonActionProbes, dashboardAction, dashboardNote, keepImport, keepNote, siteGenerated, siteStatus, siteStopped, syncStatus, platform, vaults, directory, drawings, attachments, features, searchStatus, atomicFeatures, localBackendProbes, search, folder, lifecycle, moved, attachmentWrite, attachmentList, drawing, drawingRead, drawingWritten, expectedFailure, invalidPathFailure, missingResourceFailure, logs, restartPersistence, packagedRun, catalogSource }
  writeFileSync(join(artifactRoot, 'latest.json'), JSON.stringify({ at: new Date().toISOString(), runtime: 'tauri', result }, null, 2))
  writeFileSync(join(artifactRoot, 'latest-tauri.log'), output, 'utf8')
  console.log(`[acceptance-runner] artifact ${join(artifactRoot, 'latest.json')}`)
  console.log(`[acceptance-runner] Tauri desktop scenario passed ${JSON.stringify({ notePath: saved.notePath, logCount: logs.length })}`)
} catch (error) {
  writeFileSync(join(artifactRoot, 'latest.json'), JSON.stringify({ at: new Date().toISOString(), runtime: 'tauri', ok: false, error: error?.stack || String(error) }, null, 2))
  writeFileSync(join(artifactRoot, 'latest-tauri.log'), output, 'utf8')
  throw error
} finally {
  await stopChild()
  writeFileSync(join(artifactRoot, 'latest-tauri.log'), output, 'utf8')
  rmSync(fixtureRoot, { recursive: true, force: true })
}

if (!result) process.exitCode = 1
