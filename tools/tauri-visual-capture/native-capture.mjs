import { mkdirSync, statSync } from 'node:fs'
import path from 'node:path'

import { readPngDimensions, sha256File } from './manifest.mjs'
import {
  captureNativeWindow,
  listNativeWindows,
  measureNativeWindowChrome,
  readProcessTable,
  selectNativeWindow,
  selectTauriChildProcess
} from './native-window.mjs'

const directTauriPath = (appPath) => appPath.toLowerCase().endsWith('/elephant') || appPath.toLowerCase().endsWith('/elephantnote-tauri')

export const waitForTauriWindow = async (child, appPath, delay, log) => {
  const deadline = Date.now() + 120000
  let last = 'no actual Tauri child/window candidate'
  while (Date.now() < deadline) {
    try {
      const table = readProcessTable()
      const process = selectTauriChildProcess({ launcherPid: child.pid, appPath, includeLauncher: directTauriPath(appPath), table })
      const windows = listNativeWindows([process.pid])
      const window = selectNativeWindow({ windows, pid: process.pid, names: ['Elephant'] })
      log({ type: 'window:selected', launcherPid: child.pid, tauriPid: process.pid, executable: process.comm, args: process.args, windowId: window.windowId, ownerPid: window.ownerPid, ownerName: window.ownerName, title: window.title, bounds: window.bounds })
      return { process, window }
    } catch (error) {
      last = error.message
    }
    await delay(500)
  }
  throw new Error(`timed out waiting for a visible native Tauri child window: ${last}`)
}

export const refreshWindow = (runtime) => {
  const table = readProcessTable()
  const process = selectTauriChildProcess({ launcherPid: runtime.launcherPid, appPath: runtime.appPath, includeLauncher: directTauriPath(runtime.appPath), table })
  if (process.pid !== runtime.pid) throw new Error(`Tauri child changed from ${runtime.pid} to ${process.pid}`)
  runtime.window = selectNativeWindow({ windows: listNativeWindows([runtime.pid]), pid: runtime.pid, names: ['Elephant'] })
  runtime.process = process
  return runtime.window
}

export const measureContent = (runtime, accessibility) => {
  const windowBounds = runtime.window.bounds
  const webAreas = accessibility.elements.filter((element) => element.role === 'AXWebArea' && element.rect && element.rect.width > 100 && element.rect.height > 100)
  const nativeControls = accessibility.elements.filter((element) => ['AXCloseButton', 'AXMinimizeButton', 'AXFullScreenButton', 'AXToolbarButton'].includes(element.subrole) && element.rect && element.rect.width > 0 && element.rect.height > 0)
  if (webAreas.length === 1) {
    const contentBounds = webAreas[0].rect
    const chrome = accessibility.elements.filter((element) => ['AXTitleBar', 'AXToolbar'].includes(element.role) && element.rect && element.rect.width > 0 && element.rect.height > 0)
    const overlapsChrome = chrome.some((element) => element.rect.y <= windowBounds.y + 2 && element.rect.y + element.rect.height > contentBounds.y + 1)
    if (overlapsChrome) throw new Error(`native chrome overlaps the measured AXWebArea; content crop cannot be proven: ${JSON.stringify(chrome)}`)
    return { contentBounds, chrome, nativeWindowDecorated: chrome.length > 0, nativeChromeExcluded: true, measured: true, measuredBy: 'macOS Accessibility AXWebArea' }
  }
  if (nativeControls.length > 0) {
    const chromeBottom = Math.max(...nativeControls.map((element) => element.rect.y + element.rect.height))
    const contentBounds = { x: windowBounds.x, y: chromeBottom, width: windowBounds.width, height: Math.max(0, windowBounds.height - (chromeBottom - windowBounds.y)) }
    if (contentBounds.height < 1) throw new Error(`measured native controls consume the whole native window: ${JSON.stringify(nativeControls)}`)
    return { contentBounds, chrome: nativeControls.map((element) => ({ subrole: element.subrole, rect: element.rect })), nativeWindowDecorated: true, nativeChromeExcluded: true, measured: true, measuredBy: 'macOS Accessibility native window controls' }
  }
  const hierarchy = measureNativeWindowChrome({ processName: runtime.window.ownerName })
  if (hierarchy.windowBounds.width !== windowBounds.width || hierarchy.windowBounds.height !== windowBounds.height) throw new Error(`System Events and CGWindow bounds disagree: ${JSON.stringify({ cgWindow: windowBounds, systemEvents: hierarchy.windowBounds })}`)
  if (!hierarchy.titleBar) return { contentBounds: windowBounds, chrome: [], nativeWindowDecorated: false, nativeChromeExcluded: true, measured: true, measuredBy: hierarchy.measuredBy }
  const titleBarBottom = hierarchy.titleBar.y + hierarchy.titleBar.height
  const contentBounds = { x: windowBounds.x, y: titleBarBottom, width: windowBounds.width, height: Math.max(0, windowBounds.height - (titleBarBottom - windowBounds.y)) }
  if (contentBounds.height < 1) throw new Error(`measured title bar consumes the whole native window: ${JSON.stringify(hierarchy)}`)
  return { contentBounds, chrome: [hierarchy.titleBar], nativeWindowDecorated: true, nativeChromeExcluded: true, measured: true, measuredBy: hierarchy.measuredBy }
}

export const createFrameCapture = ({ runtime, content, outputRoot, manifest, log }) => (action, index, relativeMs, kind) => {
  refreshWindow(runtime)
  const relativePath = path.join(action.checkpoint, 'frames', `${String(action.index).padStart(2, '0')}-${action.id}-${String(index).padStart(2, '0')}-${relativeMs}ms.png`)
  const filename = path.join(outputRoot, relativePath)
  mkdirSync(path.dirname(filename), { recursive: true })
  const captured = captureNativeWindow({ windowId: runtime.window.windowId, filename, outputWidth: runtime.window.bounds.width, outputHeight: runtime.window.bounds.height, contentBounds: content.contentBounds, windowBounds: runtime.window.bounds })
  const dimensions = readPngDimensions(filename)
  const frame = { index, relativeMs, kind, path: relativePath, sourcePath: filename, bytes: statSync(filename).size, width: dimensions.width, height: dimensions.height, sha256: sha256File(filename), contentOnly: true, physicalCapture: true, captureMethod: captured.method, contentCrop: captured.content?.crop || null, captureBounds: captured.content?.captureBounds || null, captureFallbackError: captured.fallbackError || null, capturedAtMs: Date.now() }
  manifest.frames.push({ ...frame, actionId: action.id, checkpoint: action.checkpoint })
  log({ type: 'frame:capture', action: action.id, index, relativeMs, kind, width: frame.width, height: frame.height, method: frame.captureMethod, fallbackError: frame.captureFallbackError })
  return frame
}
