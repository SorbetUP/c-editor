import { ElMessage } from 'element-plus'

export const invokeTauri = (command, payload = {}) => {
  const invoke = globalThis?.__TAURI__?.core?.invoke
  if (typeof invoke !== 'function') {
    throw new Error(`Tauri command API is unavailable for ${command}`)
  }
  return invoke(command, payload)
}

const pad = (value, length = 2) => String(value).padStart(length, '0')

export const localDateParts = (date = new Date()) => ({
  date: `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`,
  time: `${pad(date.getHours())}-${pad(date.getMinutes())}-${pad(date.getSeconds())}`,
  milliseconds: pad(date.getMilliseconds(), 3),
  weekday: new Intl.DateTimeFormat(undefined, { weekday: 'long' }).format(date),
  iso: date.toISOString()
})

export const offsetLocalDate = (days, date = new Date()) => {
  const shifted = new Date(date)
  shifted.setDate(shifted.getDate() + days)
  return localDateParts(shifted)
}

export const yamlString = (value) => JSON.stringify(String(value))

export const readNote = (path) => invokeTauri('tauri_notes_read', { relativePath: path })

export const writeNote = (path, content) => invokeTauri('tauri_notes_write', {
  relativePath: path,
  content
})

export const createNoteIfMissing = async (path, content) => {
  try {
    const existing = await readNote(path)
    return { path, created: false, existing }
  } catch {
    const written = await writeNote(path, content)
    return { path, created: true, written }
  }
}

export const notifyCreated = (label, result) => {
  ElMessage.success(result.created
    ? `${label} created: ${result.path}`
    : `${label} already exists: ${result.path}`)
}

export const notifySuccess = (message) => ElMessage.success(message)

export const logAction = (ctx, phase, payload = {}) => {
  ctx.logger?.info?.(`[addons] builtin:${phase}`, payload)
}

export const markdownLink = (path, title) => {
  const target = String(path || '').replace(/\.md$/i, '').replaceAll('|', '\\|')
  const label = String(title || target || 'Untitled').replaceAll('|', '\\|')
  return `[[${target}|${label}]]`
}

export const sortByPath = (left, right) => {
  const leftPath = String(left?.relativePath || left?.path || '')
  const rightPath = String(right?.relativePath || right?.path || '')
  return leftPath.localeCompare(rightPath, undefined, { numeric: true, sensitivity: 'base' })
}
