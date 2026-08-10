#!/usr/bin/env node

/**
 * Real Avalonia acceptance harness for the native Markdown vertical.
 *
 * This script deliberately uses macOS Accessibility/System Events and the
 * native screencapture utility. It never opens the Muya bundle in Chromium,
 * never evaluates JavaScript in the WebView, and never mutates the DOM.
 */

import { closeSync, cpSync, existsSync, mkdirSync, mkdtempSync, openSync, readFileSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { spawn, spawnSync } from 'node:child_process'
import { createConnection } from 'node:net'
import { setTimeout as delay } from 'node:timers/promises'

const scriptDirectory = dirname(new URL(import.meta.url).pathname)
const repositoryRoot = resolve(scriptDirectory, '../../../..')
const appProject = resolve(repositoryRoot, 'Elephant/avalonia/src/ElephantNote.Avalonia/ElephantNote.Avalonia.csproj')
const publishedExecutable = resolve(repositoryRoot, 'Elephant/avalonia/build/artifacts/publish/macos-arm64-aot/ElephantNote.Avalonia')
const noteName = 'acceptance-native.md'
const noteTitle = 'Native acceptance note'
const initialMarkdown = '# Native acceptance note\n\nInitial paragraph from the Avalonia acceptance harness.\n'

const PASS = 0
const FAIL = 1
const NOT_RUN = 77

const args = new Set(process.argv.slice(2))
const keepArtifacts = args.has('--keep-artifacts')
const timeoutMs = Number(process.env.AVALONIA_ACCEPTANCE_TIMEOUT_MS || 45000)
const expectedProcessName = process.env.AVALONIA_ACCEPTANCE_PROCESS || 'ElephantNote.Avalonia'
const artifactRoot = process.env.AVALONIA_ACCEPTANCE_ARTIFACT_DIR
  ? resolve(process.env.AVALONIA_ACCEPTANCE_ARTIFACT_DIR)
  : mkdtempSync(join(tmpdir(), 'elephant-avalonia-native-'))

mkdirSync(artifactRoot, { recursive: true })

const report = {
  status: 'NOT_RUN',
  reason: null,
  platform: process.platform,
  startedAt: new Date().toISOString(),
  artifactRoot,
  evidence: {},
  events: []
}

const log = (event, details = {}) => {
  const item = { at: new Date().toISOString(), event, ...details }
  report.events.push(item)
  process.stdout.write(`[avalonia-native] ${event}${Object.keys(details).length ? ` ${JSON.stringify(details)}` : ''}\n`)
}

const finish = (status, reason = null, exitCode = status === 'PASS' ? PASS : status === 'FAIL' ? FAIL : NOT_RUN) => {
  report.status = status
  report.reason = reason
  report.finishedAt = new Date().toISOString()
  report.evidence.result = resolve(artifactRoot, 'result.json')
  writeFileSync(report.evidence.result, `${JSON.stringify(report, null, 2)}\n`)
  process.stdout.write(`\nAVALONIA_NATIVE_ACCEPTANCE=${status}\n`)
  if (reason) process.stdout.write(`reason=${reason}\n`)
  process.stdout.write(`artifacts=${artifactRoot}\n`)
  process.exitCode = exitCode
}

const appleScriptString = (value) => `"${String(value).replaceAll('\\', '\\\\').replaceAll('"', '\\"')}"`

const runAppleScript = (script) => {
  const result = spawnSync('osascript', ['-e', script], { encoding: 'utf8' })
  return {
    ok: result.status === 0,
    status: result.status,
    stdout: (result.stdout || '').trim(),
    stderr: (result.stderr || '').trim()
  }
}

class NativeAcceptanceClient {
  constructor(socket) {
    this.socket = socket
    this.buffer = ''
    this.pending = []
    socket.setEncoding('utf8')
    socket.on('data', (chunk) => {
      this.buffer += chunk
      let newline
      while ((newline = this.buffer.indexOf('\n')) >= 0) {
        const line = this.buffer.slice(0, newline)
        this.buffer = this.buffer.slice(newline + 1)
        const request = this.pending.shift()
        if (!request) continue
        try { request.resolve(JSON.parse(line)) }
        catch (error) { request.reject(error) }
      }
    })
    socket.on('error', (error) => {
      while (this.pending.length) this.pending.shift().reject(error)
    })
  }

  command(command, payload = {}) {
    return new Promise((resolve, reject) => {
      this.pending.push({
        resolve: (response) => {
          if (!response.ok) reject(new Error(response.error || `Native acceptance command failed: ${command}`))
          else resolve(response.result)
        },
        reject
      })
      this.socket.write(`${JSON.stringify({ command, payload })}\n`)
    })
  }

  close() { this.socket.end() }
}

const connectNativeAcceptance = async (socketPath) => waitFor(() => new Promise((resolve) => {
  const socket = createConnection(socketPath)
  let connected = false
  socket.once('connect', () => {
    connected = true
    resolve(new NativeAcceptanceClient(socket))
  })
  socket.once('error', () => {
    if (!connected) {
      socket.destroy()
      resolve(false)
    }
  })
}), 'native acceptance control channel')

const waitFor = async (predicate, label, maxWait = timeoutMs) => {
  const started = Date.now()
  let lastError = null
  while (Date.now() - started < maxWait) {
    try {
      const value = await predicate()
      if (value) return value
    } catch (error) {
      lastError = error
    }
    await delay(250)
  }
  throw new Error(`${label} timed out${lastError ? `: ${lastError.message}` : ''}`)
}

const commandExists = (command) => spawnSync('sh', ['-c', `command -v ${command}`], { encoding: 'utf8' }).status === 0

const prepareFixture = () => {
  const configDirectory = join(artifactRoot, 'config')
  const vaultDirectory = join(artifactRoot, 'vault')
  const socketPath = join(artifactRoot, 'acceptance.sock')
  mkdirSync(configDirectory, { recursive: true })
  mkdirSync(vaultDirectory, { recursive: true })
  writeFileSync(join(vaultDirectory, noteName), initialMarkdown)
  writeFileSync(
    join(configDirectory, 'tauri-vaults.json'),
    `${JSON.stringify({
      vaults: [{ id: 'avalonia-acceptance', name: 'Acceptance vault', path: vaultDirectory.replaceAll('\\', '/') }],
      activeVaultId: 'avalonia-acceptance'
    }, null, 2)}\n`
  )
  writeFileSync(join(configDirectory, 'preferences.json'), `${JSON.stringify({ theme: 'light', autoSave: false }, null, 2)}\n`)
  report.evidence.fixtureVault = vaultDirectory
  report.evidence.note = join(vaultDirectory, noteName)
  report.evidence.config = configDirectory
  report.evidence.controlSocket = socketPath
  log('fixture.prepared', { vaultDirectory, note: noteName })
  return { configDirectory, vaultDirectory, notePath: join(vaultDirectory, noteName), socketPath }
}

const findRuntime = () => {
  const requested = process.env.AVALONIA_ACCEPTANCE_EXECUTABLE
  if (requested) {
    if (!existsSync(requested)) throw new Error(`AVALONIA_ACCEPTANCE_EXECUTABLE does not exist: ${requested}`)
    return { command: resolve(requested), args: [], kind: 'explicit-executable' }
  }
  const useDotnet = process.env.AVALONIA_ACCEPTANCE_USE_DOTNET === '1'
  if (process.env.AVALONIA_ACCEPTANCE_BACKGROUND === '1' && existsSync(publishedExecutable)) {
    return { command: publishedExecutable, args: [], kind: 'macos-background-bundle' }
  }
  if (!useDotnet && existsSync(publishedExecutable)) return { command: publishedExecutable, args: [], kind: 'macos-arm64-aot' }
  if (commandExists('dotnet')) {
    return {
      command: 'dotnet',
      args: ['run', '--project', appProject, '--configuration', 'Release', '--no-build', '--no-restore'],
      kind: 'dotnet-no-build'
    }
  }
  if (existsSync(publishedExecutable)) return { command: publishedExecutable, args: [], kind: 'macos-arm64-aot' }
  throw new Error(`No Avalonia runtime found. Build the app or set AVALONIA_ACCEPTANCE_EXECUTABLE. Expected ${publishedExecutable}`)
}

const prepareBackgroundBundle = (configDirectory, socketPath, stdoutPath, stderrPath) => {
  const bundleDirectory = join(artifactRoot, 'ElephantNote.Avalonia.app')
  const contentsDirectory = join(bundleDirectory, 'Contents')
  const executableDirectory = join(contentsDirectory, 'MacOS')
  const plistPath = configDirectory.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;')
  const plistSocket = socketPath.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;')
  mkdirSync(executableDirectory, { recursive: true })
  cpSync(resolve(publishedExecutable, '..'), executableDirectory, { recursive: true, force: true })
  writeFileSync(join(contentsDirectory, 'Info.plist'), `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleExecutable</key><string>ElephantNote.Avalonia</string>
  <key>CFBundleIdentifier</key><string>com.elephantnote.acceptance.avalonia</string>
  <key>CFBundleName</key><string>ElephantNote.Avalonia</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>13.0</string>
  <key>LSUIElement</key><true/>
  <key>NSHighResolutionCapable</key><true/>
  <key>LSEnvironment</key><dict>
    <key>ELEPHANTNOTE_BACKGROUND</key><string>1</string>
    <key>ELEPHANTNOTE_ACCESSIBILITY_AUTOMATION</key><string>1</string>
    <key>ELEPHANTNOTE_CONFIG_DIR</key><string>${plistPath}</string>
    <key>ELEPHANTNOTE_ACCEPTANCE_SOCKET</key><string>${plistSocket}</string>
  </dict>
</dict></plist>
`)
  return bundleDirectory
}

const launchRuntime = (runtime, configDirectory, socketPath) => {
  const stdoutPath = join(artifactRoot, 'avalonia.stdout.log')
  const stderrPath = join(artifactRoot, 'avalonia.stderr.log')
  if (runtime.kind === 'macos-background-bundle') {
    const bundleDirectory = prepareBackgroundBundle(configDirectory, socketPath, stdoutPath, stderrPath)
    writeFileSync(stdoutPath, '')
    writeFileSync(stderrPath, '')
    const openResult = spawnSync('open', ['-g', '-n', bundleDirectory], {
      encoding: 'utf8',
      env: {
        ...process.env,
        ELEPHANTNOTE_CONFIG_DIR: configDirectory,
        ELEPHANTNOTE_BACKGROUND: '1',
        ELEPHANTNOTE_ACCEPTANCE_SOCKET: socketPath,
        AVALONIA_TELEMETRY_OPTOUT: '1',
        DOTNET_CLI_TELEMETRY_OPTOUT: '1'
      }
    })
    if (openResult.status !== 0) {
      throw new Error(`Could not launch background Avalonia bundle: ${(openResult.stderr || '').trim()}`)
    }
    report.evidence.stdout = stdoutPath
    report.evidence.stderr = stderrPath
    report.evidence.runtime = { ...runtime, bundleDirectory }
    log('app.launched.background', { runtime: runtime.kind, bundleDirectory })
    return { child: null, stdout: null, stderr: null, bundleDirectory }
  }
  const stdout = openSync(stdoutPath, 'w')
  const stderr = openSync(stderrPath, 'w')
  const child = spawn(runtime.command, runtime.args, {
    cwd: repositoryRoot,
    env: {
      ...process.env,
      AVALONIA_TELEMETRY_OPTOUT: '1',
      DOTNET_CLI_TELEMETRY_OPTOUT: '1',
      ELEPHANTNOTE_CONFIG_DIR: configDirectory,
      ELEPHANTNOTE_ACCEPTANCE_SOCKET: socketPath
    },
    stdio: ['ignore', stdout, stderr]
  })
  report.evidence.stdout = stdoutPath
  report.evidence.stderr = stderrPath
  report.evidence.runtime = runtime
  log('app.launched', { runtime: runtime.kind, pid: child.pid })
  return { child, stdout, stderr }
}

const appWindowScript = `
tell application "System Events"
  tell process ${appleScriptString(expectedProcessName)}
    repeat with candidateWindowRef in (windows)
      set candidateWindow to contents of candidateWindowRef
      try
        if (name of candidateWindow as text) starts with "Elephant" then
          return ${appleScriptString(expectedProcessName)} & linefeed & (name of candidateWindow as text)
        end if
      end try
    end repeat
  end tell
  return ""
end tell
`

const getAppWindow = () => {
  const result = runAppleScript(appWindowScript)
  if (!result.ok) throw new Error(result.stderr || 'System Events could not inspect application windows.')
  const [processName, windowName] = result.stdout.split(/\r?\n/)
  return processName && windowName ? { processName, windowName } : null
}

const moveAppWindowOffscreen = ({ processName, windowName }) => {
  const script = `
tell application "System Events"
  tell process ${appleScriptString(processName)}
    repeat with windowRef in (windows)
      set currentWindow to contents of windowRef
      try
        if (name of currentWindow as text) is ${appleScriptString(windowName)} then
          set position of currentWindow to {-4000, -4000}
          set frontmost to false
          return "background"
        end if
      end try
    end repeat
  end tell
end tell
return "not-found"
`
  const result = runAppleScript(script)
  if (!result.ok || result.stdout !== 'background') {
    throw new Error(`Could not move native window to background: ${(result.stderr || result.stdout || 'not-found').trim()}`)
  }
}

const accessibilitySnapshot = ({ processName, windowName }, phase) => {
  moveAppWindowOffscreen({ processName, windowName })
  const script = `
tell application "System Events"
  tell process ${appleScriptString(processName)}
    set targetWindow to window ${appleScriptString(windowName)}
      set resultRows to {}
      repeat with itemRef in (entire contents of targetWindow)
        try
          set elementValue to contents of itemRef
          set roleValue to role of elementValue as text
          set descriptionValue to ""
          set titleValue to ""
          set valueValue to ""
          try
            set descriptionValue to description of elementValue as text
          end try
          try
            set titleValue to title of elementValue as text
          end try
          try
            set valueValue to value of elementValue as text
          end try
          set end of resultRows to role of elementValue as text & tab & descriptionValue & tab & titleValue & tab & valueValue
        end try
      end repeat
      return resultRows as text
  end tell
end tell
`
  const result = runAppleScript(script)
  const outputPath = join(artifactRoot, `accessibility-${phase}.txt`)
  writeFileSync(outputPath, result.ok ? `${result.stdout}\n` : `${result.stderr}\n`)
  report.evidence[`accessibility${phase[0].toUpperCase()}${phase.slice(1)}`] = outputPath
  if (!result.ok) throw new Error(`Accessibility snapshot failed: ${result.stderr}`)
  log('accessibility.snapshot', { phase, path: outputPath })
  return result.stdout
}

const pressAccessibleElement = ({ processName, windowName }, expectedText) => {
  moveAppWindowOffscreen({ processName, windowName })
  const target = appleScriptString(expectedText)
  const directAction = expectedText === noteTitle
    ? `click row 1 of group 1 of group 1 of group 1 of group 1 of scroll area 1 of group 1 of list 1 of group 1 of group 3 of group 2 of group 1 of group 1 of group 4 of group 2 of window ${appleScriptString(windowName)}`
    : expectedText === 'Back to notes'
      ? `click button "Back to notes" of group 1 of group 1 of group 3 of group 2 of group 1 of group 1 of group 4 of group 2 of window ${appleScriptString(windowName)}`
      : null
  if (directAction) {
  const directScript = `
tell application "System Events"
  tell process ${appleScriptString(processName)}
    ${directAction}
    return "pressed"
  end tell
end tell
`
    const directResult = runAppleScript(directScript)
    if (!directResult.ok) throw new Error(`Accessibility action failed for ${expectedText}: ${directResult.stderr}`)
    return directResult.stdout === 'pressed'
  }
  const script = `
tell application "System Events"
  tell process ${appleScriptString(processName)}
    set targetWindow to window ${appleScriptString(windowName)}
      repeat with itemRef in (entire contents of targetWindow)
        try
          set elementValue to contents of itemRef
          set descriptionValue to ""
          set titleValue to ""
          set valueValue to ""
          try
            set descriptionValue to description of elementValue as text
          end try
          try
            set titleValue to title of elementValue as text
          end try
          try
            set valueValue to value of elementValue as text
          end try
          if descriptionValue is ${target} or titleValue is ${target} or valueValue is ${target} then
            perform action "AXPress" of elementValue
            return "pressed"
          end if
        end try
      end repeat
      return "not-found"
  end tell
end tell
`
  const result = runAppleScript(script)
  if (!result.ok) throw new Error(`Accessibility action failed for ${expectedText}: ${result.stderr}`)
  return result.stdout === 'pressed'
}

const focusAndType = ({ processName, windowName }, text) => {
  const script = `
tell application "System Events"
  tell process ${appleScriptString(processName)}
    set targetWindow to window ${appleScriptString(windowName)}
    set focused of targetWindow to true
    set editorControl to group "Muya Markdown editor" of group 1 of group 2 of group 1 of group 3 of group 2 of group 1 of group 1 of group 4 of group 2 of targetWindow
    perform action "AXPress" of editorControl
    try
      set focused of editorControl to true
    end try
    delay 0.4
    key code 119
    key code 36
    keystroke ${appleScriptString(text)}
  end tell
end tell
`
  const result = runAppleScript(script)
  if (!result.ok) throw new Error(`Native keyboard edit failed: ${result.stderr}`)
}

const saveWithKeyboard = ({ processName }) => {
  const modifier = process.platform === 'darwin' ? 'command down' : 'control down'
  const script = `
tell application "System Events"
  tell process ${appleScriptString(processName)}
    keystroke "s" using {${modifier}}
  end tell
end tell
`
  const result = runAppleScript(script)
  if (!result.ok) throw new Error(`Native save shortcut failed: ${result.stderr}`)
}

const captureNativeWindow = ({ processName, windowName }, phase) => {
  const path = join(artifactRoot, `native-window-${phase}.png`)
  const result = spawnSync('screencapture', ['-x', '-o', path], { encoding: 'utf8' })
  if (result.status !== 0 || !existsSync(path)) {
    throw new Error(`Native screenshot failed: ${(result.stderr || '').trim() || `exit ${result.status}`}`)
  }
  report.evidence[`screenshot${phase[0].toUpperCase()}${phase.slice(1)}`] = path
  log('screenshot.captured', { phase, path })
}

const waitForFileContent = async (notePath, expected, label) => {
  await waitFor(async () => {
    try {
      return readFileSync(notePath, 'utf8').includes(expected)
    } catch {
      return false
    }
  }, label)
}

const terminate = async (child) => {
  if (!child || child.exitCode !== null) return
  child.kill('SIGTERM')
  await Promise.race([
    new Promise((resolve) => child.once('exit', resolve)),
    delay(4000)
  ])
  if (child.exitCode === null) child.kill('SIGKILL')
}

const terminateBackgroundBundle = (bundleDirectory) => {
  if (!bundleDirectory) return
  const executable = join(bundleDirectory, 'Contents', 'MacOS', 'ElephantNote.Avalonia')
  spawnSync('pkill', ['-TERM', '-f', executable], { encoding: 'utf8' })
}

const requireNativeState = (state, label) => {
  if (!state || state.error) throw new Error(`${label} failed: ${state?.error || 'no state returned'}`)
  return state
}

const runNativeControlAcceptance = async (appWindow, fixture, editToken) => {
  const client = await connectNativeAcceptance(fixture.socketPath)
  report.evidence.controlMode = 'native-ui-thread-command-channel'
  log('acceptance.channel.connected', { socket: fixture.socketPath })
  try {
    const initial = requireNativeState(await client.command('state'), 'Initial native state')
    if (!initial.muyaReady) throw new Error('Muya was not ready on the native UI thread.')
    if (appWindow) captureNativeWindow(appWindow, 'initial')

    const opened = requireNativeState(
      await client.command('open-note', { path: noteName }),
      'Native note open')
    if (opened.openedNotePath !== noteName) throw new Error(`Native note path mismatch: ${opened.openedNotePath}`)
    log('note.opened.native-command', { note: noteName })

    const edited = requireNativeState(
      await client.command('append-text', { text: editToken }),
      'Native Muya edit')
    if (!edited.muyaReady || edited.contentLength <= initial.contentLength) {
      throw new Error('Native Muya edit did not change the document snapshot.')
    }
    log('keyboard.edit.native-command', { token: editToken })

    requireNativeState(await client.command('save-note'), 'Native note save')
    log('keyboard.save.native-command', { command: 'save-note' })
    await waitForFileContent(fixture.notePath, editToken, 'saved Markdown content')
    report.evidence.savedContent = readFileSync(fixture.notePath, 'utf8')
    log('filesystem.save.verified', { notePath: fixture.notePath })

    const backed = requireNativeState(await client.command('back-to-notes'), 'Native return to notes')
    if (backed.openedNotePath !== '') throw new Error('Native back action did not close the note.')
    log('note.closed.native-command', { action: 'back-to-notes' })

    const reopened = requireNativeState(
      await client.command('open-note', { path: noteName }),
      'Native note reopen')
    if (reopened.openedNotePath !== noteName || !reopened.muyaReady) {
      throw new Error('Native reopen did not restore the Muya document.')
    }
    await waitForFileContent(fixture.notePath, editToken, 'persisted Markdown content after reopen')
    report.evidence.reopenedContent = readFileSync(fixture.notePath, 'utf8')
    log('note.reopened.native-command', { contentVerifiedFromFilesystem: true })
    if (appWindow) captureNativeWindow(appWindow, 'reopened')
  } finally {
    client.close()
  }
}

const run = async () => {
  if (process.platform !== 'darwin') {
    finish('NOT_RUN', `GUI_UNAVAILABLE: native Avalonia window automation is implemented for macOS only (platform=${process.platform})`)
    return
  }
  for (const command of ['osascript', 'screencapture']) {
    if (!commandExists(command)) {
      finish('NOT_RUN', `GUI_UNAVAILABLE: required macOS command is missing: ${command}`)
      return
    }
  }

  const accessibilityProbe = runAppleScript(`tell application "System Events" to exists process ${appleScriptString(expectedProcessName)}`)
  if (!accessibilityProbe.ok) {
    finish('NOT_RUN', `GUI_UNAVAILABLE: macOS Accessibility/System Events is unavailable: ${accessibilityProbe.stderr}`)
    return
  }

  const fixture = prepareFixture()
  let runtime
  try {
    runtime = findRuntime()
  } catch (error) {
    finish('NOT_RUN', `RUNTIME_UNAVAILABLE: ${error instanceof Error ? error.message : String(error)}`)
    return
  }
  const childState = process.env.AVALONIA_ACCEPTANCE_ATTACH_ONLY === '1'
    ? { child: null, stdout: null, stderr: null, attachOnly: true }
    : launchRuntime(runtime, fixture.configDirectory)
  let appWindow = null
  const editToken = `Native keyboard edit ${Date.now()}`
  try {
    appWindow = await waitFor(async () => {
      if (childState.child && childState.child.exitCode !== null) {
        const stderr = readFileSync(report.evidence.stderr, 'utf8')
        throw new Error(`Avalonia exited with code ${childState.child.exitCode}: ${stderr.trim()}`)
      }
      return getAppWindow()
    }, 'Avalonia native window')
    report.evidence.processName = appWindow.processName
    report.evidence.windowName = appWindow.windowName
    moveAppWindowOffscreen(appWindow)
    log('window.detected.backgrounded', { ...appWindow, position: [-4000, -4000] })
    captureNativeWindow(appWindow, 'initial')
    const initialAccessibility = accessibilitySnapshot(appWindow, 'initial')
    report.evidence.nativeWebViewVisibleInitially = initialAccessibility.includes('Muya Markdown editor')

    if (!pressAccessibleElement(appWindow, noteTitle)) {
      throw new Error(`Could not activate the Markdown note through macOS Accessibility: ${noteTitle}`)
    }
    log('note.opened', { note: noteName })
    await waitFor(() => accessibilitySnapshot(appWindow, 'editor').includes('Muya Markdown editor'), 'Muya NativeWebView accessibility surface')
    report.evidence.nativeWebViewVisibleInEditor = true
    captureNativeWindow(appWindow, 'editor')

    focusAndType(appWindow, editToken)
    log('keyboard.edit.sent', { token: editToken })
    saveWithKeyboard(appWindow)
    log('keyboard.save.sent', { shortcut: 'Meta+S' })
    await waitForFileContent(fixture.notePath, editToken, 'saved Markdown content')
    report.evidence.savedContent = readFileSync(fixture.notePath, 'utf8')
    log('filesystem.save.verified', { notePath: fixture.notePath })

    if (!pressAccessibleElement(appWindow, 'Back to notes')) {
      throw new Error('Could not close the note through the native Back to notes action')
    }
    log('note.closed', { action: 'Back to notes' })
    if (!pressAccessibleElement(appWindow, noteTitle)) {
      throw new Error(`Could not reopen the Markdown note: ${noteTitle}`)
    }
    await waitFor(() => accessibilitySnapshot(appWindow, 'reopened').includes('Muya Markdown editor'), 'Muya NativeWebView after reopen')
    await waitForFileContent(fixture.notePath, editToken, 'persisted Markdown content after reopen')
    captureNativeWindow(appWindow, 'reopened')
    report.evidence.reopenedContent = readFileSync(fixture.notePath, 'utf8')
    log('note.reopened', { contentVerifiedFromFilesystem: true })
    finish('PASS')
  } catch (error) {
    const stderr = readFileSync(report.evidence.stderr, 'utf8')
    const message = error instanceof Error ? error.message : String(error)
    if (message.includes('Native error code is: -6661') || message.includes('RenderTimer')) {
      finish('NOT_RUN', `GUI_UNAVAILABLE: Avalonia.Native could not start its render timer: ${message}`)
    } else if (childState.child && childState.child.exitCode !== null && /Avalonia exited/.test(message)) {
      finish('NOT_RUN', `GUI_UNAVAILABLE_OR_RUNTIME_FAILED: ${message}${stderr ? `; stderr=${stderr.trim()}` : ''}`)
    } else {
      finish('FAIL', message)
    }
  } finally {
    await terminate(childState.child)
    if (childState.bundleDirectory) terminateBackgroundBundle(childState.bundleDirectory)
    if (childState.stdout) closeSync(childState.stdout)
    if (childState.stderr) closeSync(childState.stderr)
    if (!keepArtifacts && report.status === 'PASS') {
      log('artifacts.retained', { reason: 'pass results are retained for auditability' })
    }
  }
}

run().catch((error) => finish('FAIL', error instanceof Error ? error.message : String(error)))
