import { spawnSync } from 'node:child_process'
import { existsSync, mkdirSync, readdirSync, writeFileSync } from 'node:fs'
import { basename, join, resolve } from 'node:path'
import { tmpdir } from 'node:os'

const root = resolve(import.meta.dirname, '../..')
const targetTriple = process.env.TAURI_MACOS_TARGET || 'aarch64-apple-darwin'
const bundleRoots = [
  resolve(root, `Elephant/backend/tauri/target/${targetTriple}/release/bundle/macos`),
  resolve(root, 'Elephant/backend/tauri/target/release/bundle/macos')
]
const timeoutMs = Number.parseInt(process.env.TAURI_MACOS_SMOKE_TIMEOUT_MS || '90000', 10)
const pollMs = Number.parseInt(process.env.TAURI_MACOS_SMOKE_POLL_MS || '2000', 10)
const diagnosticsDir = resolve(root, 'build/tauri-macos-smoke')

const fail = (message) => {
  console.error(`[tauri-macos-smoke] ${message}`)
  process.exit(1)
}

const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms))

const run = (command, args, options = {}) => spawnSync(command, args, {
  encoding: 'utf8',
  timeout: 20000,
  ...options
})

const appleScriptString = value => `"${String(value).replaceAll('\\', '\\\\').replaceAll('"', '\\"')}"`

const readPlistValue = (plistPath, key) => {
  const result = run('/usr/libexec/PlistBuddy', ['-c', `Print :${key}`, plistPath])
  if (result.status !== 0) return null
  return result.stdout.trim() || null
}

const findAppBundle = () => {
  if (process.env.TAURI_APP_BUNDLE) return resolve(process.env.TAURI_APP_BUNDLE)

  for (const bundleRoot of bundleRoots) {
    if (!existsSync(bundleRoot)) continue
    const candidates = readdirSync(bundleRoot)
      .filter(entry => entry.endsWith('.app'))
      .map(entry => join(bundleRoot, entry))
    if (candidates.length === 0) continue
    const preferred = candidates.find(candidate => basename(candidate) === 'Elephant.app')
    return preferred || candidates[0]
  }

  fail(`no .app bundle found in ${bundleRoots.join(', ')}`)
}

const parsePids = value => String(value || '')
  .split(/\s+/)
  .map(item => Number.parseInt(item, 10))
  .filter(Number.isFinite)

const processIds = (candidateNames, executablePath) => {
  const ids = new Set()
  for (const candidate of candidateNames) {
    if (!candidate) continue
    const byName = run('/usr/bin/pgrep', ['-x', candidate])
    if (byName.status === 0) parsePids(byName.stdout).forEach(pid => ids.add(pid))
  }
  const byPath = run('/usr/bin/pgrep', ['-f', executablePath])
  if (byPath.status === 0) parsePids(byPath.stdout).forEach(pid => ids.add(pid))
  return [...ids]
}

const countWindowsWithAppleScript = (candidateNames) => {
  for (const processName of candidateNames) {
    if (!processName) continue
    const script = `
tell application "System Events"
  set targetName to ${appleScriptString(processName)}
  set matchingProcesses to application processes whose name is targetName
  if (count of matchingProcesses) is 0 then return "0"
  set targetProcess to first item of matchingProcesses
  return ((count of windows of targetProcess) as text)
end tell
`
    const result = run('/usr/bin/osascript', ['-e', script])
    if (result.status !== 0) continue
    const count = Number.parseInt(result.stdout.trim(), 10)
    if (Number.isFinite(count) && count > 0) return count
  }
  return 0
}

const swiftWindowProbePath = () => {
  const dir = join(tmpdir(), 'elephantnote-tauri-smoke')
  mkdirSync(dir, { recursive: true })
  const scriptPath = join(dir, 'count-visible-windows.swift')
  writeFileSync(scriptPath, `
import CoreGraphics
import Foundation

let requestedPids = Set(CommandLine.arguments.dropFirst().compactMap { Int($0) })

func number(_ value: Any?) -> Double {
  if let number = value as? NSNumber { return number.doubleValue }
  if let double = value as? Double { return double }
  if let int = value as? Int { return Double(int) }
  return 0
}

guard let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] else {
  print("0")
  exit(0)
}

let count = windows.filter { window in
  let ownerPid = Int(number(window[kCGWindowOwnerPID as String]))
  let layer = Int(number(window[kCGWindowLayer as String]))
  guard requestedPids.contains(ownerPid) && layer == 0 else { return false }
  guard let bounds = window[kCGWindowBounds as String] as? [String: Any] else { return false }
  let width = number(bounds["Width"])
  let height = number(bounds["Height"])
  return width > 100 && height > 100
}.count

print(count)
`)
  return scriptPath
}

let cachedSwiftProbePath = null
const countWindowsWithCoreGraphics = (pids) => {
  if (!existsSync('/usr/bin/swift') || pids.length === 0) return 0
  cachedSwiftProbePath ||= swiftWindowProbePath()
  const result = run('/usr/bin/swift', [cachedSwiftProbePath, ...pids.map(String)], { timeout: 30000 })
  if (result.status !== 0) return 0
  const count = Number.parseInt(result.stdout.trim(), 10)
  return Number.isFinite(count) ? count : 0
}

const quitApplication = (bundleId, candidateNames) => {
  if (bundleId) {
    run('/usr/bin/osascript', ['-e', `tell application id ${appleScriptString(bundleId)} to quit`], { timeout: 5000 })
  }
  for (const target of candidateNames) {
    if (!target) continue
    run('/usr/bin/osascript', ['-e', `tell application ${appleScriptString(target)} to quit`], { timeout: 5000 })
  }
}

const collectDiagnostics = (candidateNames, executablePath, bundleId, lastProbe) => {
  const ps = run('/bin/ps', ['-axo', 'pid,comm,args'])
  const matchingProcesses = ps.stdout
    .split('\n')
    .filter(line => candidateNames.some(name => name && line.includes(name)) || line.includes(executablePath))
    .join('\n')
  const predicateParts = candidateNames
    .filter(Boolean)
    .map(name => `process == ${appleScriptString(name)}`)
  const logResult = predicateParts.length > 0
    ? run('/usr/bin/log', ['show', '--last', '5m', '--style', 'compact', '--predicate', predicateParts.join(' OR ')], { timeout: 30000 })
    : { stdout: '', stderr: '' }

  const report = [
    `last probe: ${lastProbe}`,
    `bundle id: ${bundleId || '<unknown>'}`,
    `executable path: ${executablePath}`,
    '',
    'matching processes:',
    matchingProcesses || '<none>',
    '',
    'recent unified log:',
    logResult.stdout || logResult.stderr || '<none>'
  ].join('\n')

  mkdirSync(diagnosticsDir, { recursive: true })
  writeFileSync(join(diagnosticsDir, 'diagnostics.txt'), `${report}\n`)
  return report
}

if (process.platform !== 'darwin') {
  fail(`this smoke test must run on macOS, got ${process.platform}`)
}

const appBundle = findAppBundle()
const infoPlist = join(appBundle, 'Contents/Info.plist')
if (!existsSync(infoPlist)) fail(`missing Info.plist in ${appBundle}`)

const executableName = readPlistValue(infoPlist, 'CFBundleExecutable') || basename(appBundle, '.app')
const bundleName = readPlistValue(infoPlist, 'CFBundleDisplayName') || readPlistValue(infoPlist, 'CFBundleName') || basename(appBundle, '.app')
const bundleId = readPlistValue(infoPlist, 'CFBundleIdentifier')
const candidateNames = [...new Set([executableName, bundleName, basename(appBundle, '.app')].filter(Boolean))]
const executablePath = join(appBundle, 'Contents/MacOS', executableName)
if (!existsSync(executablePath)) fail(`missing app executable: ${executablePath}`)

console.log(`[tauri-macos-smoke] launching packaged app: ${appBundle}`)
console.log(`[tauri-macos-smoke] executable: ${executablePath}`)
console.log(`[tauri-macos-smoke] process names: ${candidateNames.join(', ')}`)

quitApplication(bundleId, candidateNames)

const launchResult = run('/usr/bin/open', ['-n', appBundle], { timeout: 30000 })
if (launchResult.status !== 0) {
  console.error(launchResult.stderr || launchResult.stdout)
  fail('macOS could not launch the packaged .app bundle')
}
if (bundleId) {
  run('/usr/bin/osascript', ['-e', `tell application id ${appleScriptString(bundleId)} to activate`], { timeout: 5000 })
}

const deadline = Date.now() + timeoutMs
let lastProbe = '<not started>'
let opened = false

try {
  while (Date.now() < deadline) {
    const pids = processIds(candidateNames, executablePath)
    const appleWindowCount = countWindowsWithAppleScript(candidateNames)
    const coreGraphicsWindowCount = countWindowsWithCoreGraphics(pids)
    lastProbe = `pids=${pids.join(',') || 'none'} appleScriptWindows=${appleWindowCount} coreGraphicsWindows=${coreGraphicsWindowCount}`
    console.log(`[tauri-macos-smoke] probe: ${lastProbe}`)

    if (pids.length > 0 && (appleWindowCount > 0 || coreGraphicsWindowCount > 0)) {
      opened = true
      break
    }

    await sleep(pollMs)
  }
} finally {
  quitApplication(bundleId, candidateNames)
}

if (!opened) {
  console.error(collectDiagnostics(candidateNames, executablePath, bundleId, lastProbe))
  fail(`packaged Tauri app did not expose a visible macOS window within ${timeoutMs}ms`)
}

console.log('[tauri-macos-smoke] packaged Tauri app opened a visible macOS window')
