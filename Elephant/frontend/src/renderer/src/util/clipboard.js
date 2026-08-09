import { isLinux, isOsx, isWindows } from './index'
import { clipboard as remoteClipboard } from '@/platform/tauriRemoteShim'

const hasClipboardFiles = () => {
  return typeof remoteClipboard?.has === 'function' && remoteClipboard.has('NSFilenamesPboardType')
}

const getClipboardFiles = () => {
  if (!hasClipboardFiles()) {
    return []
  }
  const rawValue = typeof remoteClipboard?.read === 'function'
    ? remoteClipboard.read('NSFilenamesPboardType')
    : ''
  if (Array.isArray(rawValue)) return rawValue
  if (typeof rawValue !== 'string' || !rawValue.trim()) return []
  return rawValue
    .split(/\r?\n/)
    .map((value) => value.trim())
    .filter(Boolean)
}

export const guessClipboardFilePath = () => {
  if (isLinux) return ''
  if (isOsx) {
    const result = getClipboardFiles()
    return Array.isArray(result) && result.length ? result[0] : ''
  } else if (isWindows) {
    const rawFilePath = remoteClipboard.read('FileNameW')
    const filePath = rawFilePath.replace(new RegExp(String.fromCharCode(0), 'g'), '')
    return filePath && typeof filePath === 'string' ? filePath : ''
  } else {
    return ''
  }
}
