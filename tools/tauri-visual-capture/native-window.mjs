import { execFileSync, spawnSync } from 'node:child_process'
import { mkdtempSync, readFileSync, renameSync, rmSync, writeFileSync } from 'node:fs'
import { basename, join } from 'node:path'
import { tmpdir } from 'node:os'
import { pngNonBackgroundBounds } from './png-geometry.mjs'

const swiftWindowProbe = `
import CoreGraphics
import Foundation

let requestedPids = Set(CommandLine.arguments.dropFirst().compactMap { Int($0) })
func number(_ value: Any?) -> Double {
  if let value = value as? NSNumber { return value.doubleValue }
  return 0
}
guard let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] else {
  print("[]")
  exit(0)
}
let result: [[String: Any]] = windows.compactMap { window in
  let ownerPid = Int(number(window[kCGWindowOwnerPID as String]))
  let layer = Int(number(window[kCGWindowLayer as String]))
  guard requestedPids.isEmpty || requestedPids.contains(ownerPid), layer == 0 else { return nil }
  guard let bounds = window[kCGWindowBounds as String] as? [String: Any] else { return nil }
  return [
    "windowId": Int(number(window[kCGWindowNumber as String])),
    "ownerPid": ownerPid,
    "ownerName": window[kCGWindowOwnerName as String] as? String ?? "",
    "title": window[kCGWindowName as String] as? String ?? "",
    "layer": layer,
    "onScreen": window[kCGWindowIsOnscreen as String] as? Bool ?? true,
    "bounds": [
      "x": number(bounds["X"]), "y": number(bounds["Y"]),
      "width": number(bounds["Width"]), "height": number(bounds["Height"])
    ]
  ]
}
let data = try JSONSerialization.data(withJSONObject: result, options: [])
print(String(data: data, encoding: .utf8)!)
`

const run = (command, args, options = {}) => spawnSync(command, args, { encoding: 'utf8', ...options })
let captureWorkspace
let captureBinary
let accessibilityWorkspace
let accessibilityBinary
let actionsWorkspace
let actionsBinary

const ensureCaptureBinary = () => {
  if (captureBinary) return captureBinary
  captureWorkspace = mkdtempSync(join(tmpdir(), 'elephant-tauri-window-capture-'))
  const source = join(captureWorkspace, 'capture.swift')
  const binary = join(captureWorkspace, 'capture')
  writeFileSync(source, readFileSync(join(import.meta.dirname, 'native-window-capture.swift')), 'utf8')
  const result = run('/usr/bin/swiftc', ['-parse-as-library', source, '-o', binary], { timeout: 30000 })
  if (result.status !== 0) return null
  captureBinary = binary
  return captureBinary
}

export const listMatchingProcessIds = ({ appPath = '', names = [] } = {}) => {
  const result = run('/bin/ps', ['-axo', 'pid=,ppid=,comm=,args='])
  if (result.status !== 0) return []
  const normalizedNames = names.map((name) => name.toLowerCase()).filter(Boolean)
  const pathNeedles = [appPath, 'target/debug/Elephant', 'target/debug/elephantnote-tauri', 'Contents/MacOS/Elephant']
    .map((value) => value.toLowerCase()).filter(Boolean)
  return result.stdout.split('\n').flatMap((line) => {
    const match = line.trim().match(/^(\d+)\s+(\d+)\s+(\S+)\s+(.*)$/)
    if (!match || Number(match[1]) === process.pid) return []
    const haystack = `${match[3]} ${match[4]}`.toLowerCase()
    if (!normalizedNames.some((name) => haystack.includes(name)) && !pathNeedles.some((needle) => haystack.includes(needle))) return []
    return [Number(match[1])]
  })
}

export const readProcessTable = () => {
  const result = run('/bin/ps', ['-axo', 'pid=,ppid=,comm=,args='])
  if (result.status !== 0) throw new Error(result.stderr || 'ps process table failed')
  return result.stdout.split('\n').flatMap((line) => {
    const match = line.trim().match(/^(\d+)\s+(\d+)\s+(\S+)\s+(.*)$/)
    if (!match) return []
    return [{ pid: Number(match[1]), ppid: Number(match[2]), comm: match[3], args: match[4] }]
  })
}

export const descendantProcessIds = (launcherPid, table = readProcessTable()) => {
  const descendants = new Set([Number(launcherPid)])
  let changed = true
  while (changed) {
    changed = false
    for (const process of table) {
      if (!descendants.has(process.ppid) || descendants.has(process.pid)) continue
      descendants.add(process.pid)
      changed = true
    }
  }
  descendants.delete(Number(launcherPid))
  return descendants
}

export const selectTauriChildProcess = ({ launcherPid, appPath = '', table = readProcessTable() } = {}) => {
  const descendants = descendantProcessIds(launcherPid, table)
  const appNeedles = [
    'target/debug/elephant',
    'target/debug/elephantnote-tauri',
    'contents/macos/elephant',
    appPath && appPath.toLowerCase().endsWith('/elephant') ? appPath.toLowerCase() : ''
  ].filter(Boolean)
  const candidates = table.filter((process) => {
    if (!descendants.has(process.pid)) return false
    const haystack = `${process.comm} ${process.args}`.toLowerCase()
    const basename = process.comm.split('/').pop()?.toLowerCase()
    const executableArgument = process.args.trim().split(/\s+/)[0]?.split('/').pop()?.toLowerCase()
    const executableName = basename === 'elephant' || basename === 'elephantnote-tauri' || executableArgument === 'elephant' || executableArgument === 'elephantnote-tauri'
    return executableName && appNeedles.some((needle) => haystack.includes(needle))
  })
  if (candidates.length !== 1) {
    throw new Error(`expected exactly one actual Tauri child of launcher ${launcherPid}, found ${candidates.map((candidate) => `${candidate.pid}:${candidate.comm}`).join(', ') || '<none>'}`)
  }
  return { ...candidates[0], launcherPid, descendant: true, selection: 'process-tree+tauri-executable' }
}

export const listNativeWindows = (pids = []) => {
  if (process.platform !== 'darwin') throw new Error(`native macOS capture requires darwin, got ${process.platform}`)
  const dir = mkdtempSync(join(tmpdir(), 'elephant-tauri-window-probe-'))
  const script = join(dir, 'probe.swift')
  writeFileSync(script, swiftWindowProbe, 'utf8')
  try {
    const result = run('/usr/bin/swift', [script, ...pids.map(String)], { timeout: 30000 })
    if (result.status !== 0) throw new Error(result.stderr || 'Swift window probe failed')
    return JSON.parse(result.stdout || '[]')
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
}

export const selectNativeWindow = ({ windows, pid, names = [] } = {}) => {
  const normalizedNames = names.map((name) => name.toLowerCase()).filter(Boolean)
  const candidates = (windows || []).filter((window) => {
    const bounds = window.bounds || {}
    const ownerMatches = pid === undefined || window.ownerPid === pid
    const nameMatches = normalizedNames.length === 0 || normalizedNames.some((name) => `${window.ownerName} ${window.title}`.toLowerCase().includes(name))
    return ownerMatches && nameMatches && window.layer === 0 && window.onScreen !== false && bounds.width > 100 && bounds.height > 100
  })
  if (candidates.length === 0) throw new Error(`no visible native window matched pid=${pid ?? '<any>'} names=${normalizedNames.join(',') || '<any>'}`)
  if (candidates.length > 1) throw new Error(`ambiguous native window selection: ${candidates.map((candidate) => `${candidate.ownerPid}:${candidate.windowId}:${candidate.title}`).join(', ')}`)
  return candidates[0]
}

export const setWindowGeometry = ({ processName, x, y, width, height }) => {
  const quote = (value) => JSON.stringify(String(value))
  const script = `tell application "System Events"\n  tell first application process whose name is ${quote(processName)}\n    set frontmost to true\n    set position of first window to {${Number(x)}, ${Number(y)}}\n    set size of first window to {${Number(width)}, ${Number(height)}}\n  end tell\nend tell`
  const result = run('/usr/bin/osascript', ['-e', script], { timeout: 10000 })
  return { ok: result.status === 0, stdout: result.stdout, stderr: result.stderr, status: result.status }
}

export const measureNativeWindowChrome = ({ processName }) => {
  const quote = (value) => JSON.stringify(String(value))
  const script = `tell application "System Events"
  tell first application process whose name is ${quote(processName)}
    set theWindow to first window
    set windowPosition to position of theWindow
    set windowSize to size of theWindow
    try
      set titleBarElement to first UI element of theWindow whose role is "AXTitleBar"
      set titleBarPosition to position of titleBarElement
      set titleBarSize to size of titleBarElement
      return (item 1 of windowPosition as text) & "," & (item 2 of windowPosition as text) & "," & (item 1 of windowSize as text) & "," & (item 2 of windowSize as text) & "|" & (item 1 of titleBarPosition as text) & "," & (item 2 of titleBarPosition as text) & "," & (item 1 of titleBarSize as text) & "," & (item 2 of titleBarSize as text)
    on error
      return (item 1 of windowPosition as text) & "," & (item 2 of windowPosition as text) & "," & (item 1 of windowSize as text) & "," & (item 2 of windowSize as text) & "|none"
    end try
  end tell
end tell`
  const result = run('/usr/bin/osascript', ['-e', script], { timeout: 10000 })
  if (result.status !== 0) throw new Error(result.stderr || 'System Events native chrome measurement failed')
  const [windowPart, titleBarPart] = result.stdout.trim().split('|')
  const numbers = (value) => value.split(',').map(Number)
  const [x, y, width, height] = numbers(windowPart || '')
  const windowBounds = { x, y, width, height }
  if (titleBarPart === 'none') return { windowBounds, titleBar: null, measured: true, measuredBy: 'System Events window UI hierarchy' }
  const [titleX, titleY, titleWidth, titleHeight] = numbers(titleBarPart)
  return { windowBounds, titleBar: { x: titleX, y: titleY, width: titleWidth, height: titleHeight }, measured: true, measuredBy: 'System Events window UI hierarchy' }
}

const ensureAccessibilityBinary = () => {
  if (accessibilityBinary) return accessibilityBinary
  accessibilityWorkspace = mkdtempSync(join(tmpdir(), 'elephant-tauri-accessibility-'))
  const source = join(accessibilityWorkspace, 'accessibility.swift')
  const binary = join(accessibilityWorkspace, 'accessibility')
  writeFileSync(source, readFileSync(join(import.meta.dirname, 'native-accessibility.swift')), 'utf8')
  const result = run('/usr/bin/swiftc', [source, '-o', binary], { timeout: 30000 })
  if (result.status !== 0) throw new Error(result.stderr || 'Swift accessibility probe failed to compile')
  accessibilityBinary = binary
  return accessibilityBinary
}

const ensureActionsBinary = () => {
  if (actionsBinary) return actionsBinary
  actionsWorkspace = mkdtempSync(join(tmpdir(), 'elephant-tauri-actions-'))
  const source = join(actionsWorkspace, 'actions.swift')
  const binary = join(actionsWorkspace, 'actions')
  writeFileSync(source, readFileSync(join(import.meta.dirname, 'native-actions.swift')), 'utf8')
  const result = run('/usr/bin/swiftc', [source, '-o', binary], { timeout: 30000 })
  if (result.status !== 0) throw new Error(result.stderr || 'Swift native action dispatcher failed to compile')
  actionsBinary = binary
  return actionsBinary
}

export const nativeActionsBinary = () => ensureActionsBinary()

export const inspectAccessibility = ({ pid }) => {
  const result = run(ensureAccessibilityBinary(), [String(pid)], { timeout: 30000 })
  if (result.status !== 0) throw new Error(result.stderr || 'Accessibility probe failed')
  return JSON.parse(result.stdout || '{"accessibilityTrusted":false,"elements":[]}')
}

export const dispatchNativeAction = ({ requestFile }) => {
  const result = run(ensureActionsBinary(), [requestFile], { timeout: 30000 })
  if (result.status !== 0) throw new Error(result.stderr || `native action dispatcher exited with ${result.status}`)
  return JSON.parse(result.stdout || '{}')
}

const readPngDimensions = (filename) => {
  const bytes = readFileSync(filename)
  if (bytes.length < 24 || bytes.readUInt32BE(0) !== 0x89504e47) throw new Error(`not a PNG: ${filename}`)
  return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) }
}

const cropToMeasuredContent = ({ filename, contentBounds, windowBounds }) => {
  const dimensions = readPngDimensions(filename)
  const captureBounds = pngNonBackgroundBounds(filename)
  const scaleX = captureBounds.width / windowBounds.width
  const scaleY = captureBounds.height / windowBounds.height
  const crop = {
    x: Math.max(0, Math.round(captureBounds.x + (contentBounds.x - windowBounds.x) * scaleX)),
    y: Math.max(0, Math.round(captureBounds.y + (contentBounds.y - windowBounds.y) * scaleY)),
    width: Math.round(contentBounds.width * scaleX),
    height: Math.round(contentBounds.height * scaleY)
  }
  if (crop.width < 1 || crop.height < 1) throw new Error(`measured content bounds produced an empty crop: ${JSON.stringify(crop)}`)
  const rawFilename = `${filename}.raw.png`
  renameSync(filename, rawFilename)
  const cropWorkspace = mkdtempSync(join(tmpdir(), 'elephant-tauri-content-crop-'))
  const croppedFilename = join(cropWorkspace, 'content.png')
  const result = run('ffmpeg', [
    '-hide_banner', '-loglevel', 'error', '-y', '-i', rawFilename,
    '-vf', `crop=${crop.width}:${crop.height}:${crop.x}:${crop.y}`,
    '-frames:v', '1', croppedFilename
  ], { timeout: 30000 })
  if (result.status !== 0) {
    rmSync(cropWorkspace, { recursive: true, force: true })
    throw new Error(result.stderr || `ffmpeg measured content crop exited with ${result.status}`)
  }
  const outputDimensions = readPngDimensions(croppedFilename)
  if (outputDimensions.width !== crop.width || outputDimensions.height !== crop.height) {
    rmSync(cropWorkspace, { recursive: true, force: true })
    throw new Error(`measured content crop was not applied: expected=${JSON.stringify(crop)} actual=${JSON.stringify(outputDimensions)}`)
  }
  renameSync(croppedFilename, filename)
  rmSync(cropWorkspace, { recursive: true, force: true })
  return { crop, rawFilename, captureBounds, sourceDimensions: dimensions }
}

export const captureNativeWindow = ({ windowId, filename, outputWidth, outputHeight, contentBounds = null, windowBounds = null }) => {
  const rawFilename = contentBounds && windowBounds ? `${filename}.capture.png` : filename
  const binary = ensureCaptureBinary()
  const directResult = binary
    ? run(binary, [String(windowId), String(Math.round(outputWidth)), String(Math.round(outputHeight)), rawFilename], { timeout: 30000 })
    : { status: 1, stderr: 'swiftc unavailable' }
  if (directResult.status === 0) {
    const bytes = readFileSync(rawFilename)
    if (bytes.length === 0) throw new Error(`CoreGraphics capture created an empty file: ${basename(filename)}`)
    const content = contentBounds && windowBounds ? cropToMeasuredContent({ filename: rawFilename, contentBounds, windowBounds }) : null
    if (content) renameSync(rawFilename, filename)
    return { bytes: readFileSync(filename).length, method: content ? 'ScreenCaptureKit-normalized-content-cropped' : 'ScreenCaptureKit-normalized', content }
  }
  const fallback = run('/usr/sbin/screencapture', ['-x', '-l', String(windowId), '-t', 'png', rawFilename], { timeout: 30000 })
  if (fallback.status !== 0) throw new Error(fallback.stderr || directResult.stderr || `screencapture exited with ${fallback.status}`)
  const bytes = readFileSync(rawFilename)
  if (bytes.length === 0) throw new Error(`screencapture created an empty file: ${basename(filename)}`)
  const content = contentBounds && windowBounds ? cropToMeasuredContent({ filename: rawFilename, contentBounds, windowBounds }) : null
  if (content) renameSync(rawFilename, filename)
  return { bytes: readFileSync(filename).length, method: content ? 'screencapture-window-id-fallback-content-cropped' : 'screencapture-window-id-fallback', fallbackError: directResult.stderr || null, content }
}
