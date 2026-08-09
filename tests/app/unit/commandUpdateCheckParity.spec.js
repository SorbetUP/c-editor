import { afterEach, describe, expect, it } from 'vitest'

import { isUpdatable } from '../../../Elephant/frontend/src/renderer/src/commands/utils.js'

const originalResourcesPath = process.resourcesPath
const originalPlatform = process.platform
const originalAppImage = process.env.APPIMAGE

const setProcessValue = (key, value) => {
  Object.defineProperty(process, key, { value, configurable: true })
}

const installFileUtils = (files = new Set()) => {
  window.path = { join: (...parts) => parts.join('/') }
  window.fileUtils = { isFile: (pathname) => files.has(pathname) }
  setProcessValue('resourcesPath', '/resources')
}

afterEach(() => {
  delete window.fileUtils
  delete window.path
  if (originalResourcesPath === undefined) {
    delete process.resourcesPath
  } else {
    setProcessValue('resourcesPath', originalResourcesPath)
  }
  setProcessValue('platform', originalPlatform)
  if (originalAppImage === undefined) {
    delete process.env.APPIMAGE
  } else {
    process.env.APPIMAGE = originalAppImage
  }
})

describe('command update check parity', () => {
  it('returns false when fileUtils is missing in jsdom or Tauri-compatible renderer', () => {
    delete window.fileUtils
    expect(isUpdatable()).toBe(false)
  })

  it('returns false when resources path is not available', () => {
    window.path = { join: (...parts) => parts.join('/') }
    window.fileUtils = { isFile: () => true }
    delete process.resourcesPath
    expect(isUpdatable()).toBe(false)
  })

  it('returns false when update resource does not exist', () => {
    installFileUtils(new Set())
    expect(isUpdatable()).toBe(false)
  })

  it('returns true for AppImage when update resource exists', () => {
    installFileUtils(new Set(['/resources/app-update.yml']))
    process.env.APPIMAGE = '/tmp/Elephant.AppImage'
    expect(isUpdatable()).toBe(true)
  })

  it('returns true for Windows installer when icon marker exists', () => {
    installFileUtils(new Set(['/resources/app-update.yml', '/resources/md.ico']))
    setProcessValue('platform', 'win32')
    expect(isUpdatable()).toBe(true)
  })

  it('returns false for unpacked desktop builds without package marker', () => {
    installFileUtils(new Set(['/resources/app-update.yml']))
    setProcessValue('platform', 'darwin')
    expect(isUpdatable()).toBe(false)
  })
})
