import { isPortableRuntime } from './preferenceStorage'

const isMobileRuntime = () =>
  typeof navigator !== 'undefined' && /android|iphone|ipad|ipod/i.test(navigator.userAgent)

export const restorePortableWindowState = async() => {
  if (!isPortableRuntime() || isMobileRuntime()) return false
  const { restoreStateCurrent, StateFlags } = await import('@tauri-apps/plugin-window-state')
  await restoreStateCurrent(StateFlags.ALL)
  return true
}

export const savePortableWindowState = async() => {
  if (!isPortableRuntime() || isMobileRuntime()) return false
  const { saveWindowState, StateFlags } = await import('@tauri-apps/plugin-window-state')
  await saveWindowState(StateFlags.ALL)
  return true
}
