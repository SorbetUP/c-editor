import bus from '@/bus'
import '@/i18n/editorUiFallbacks'

const COMMAND_LOG_PREFIX = '[writing-command]'
const DUPLICATE_COMMAND_WINDOW_MS = 350
const QUICK_INSERT_QUERY_RE = /^\/[\p{L}\p{N}_-]*$/u

let lastCommand = { name: '', at: 0 }

const normalizeCommand = (command = '') => String(command || '').trim().toLowerCase()

const shouldSkipDuplicateCommand = (command) => {
  const now = Date.now()
  const duplicate = lastCommand.name === command && now - lastCommand.at < DUPLICATE_COMMAND_WINDOW_MS
  lastCommand = { name: command, at: now }
  return duplicate
}

const textNodeBeforeSelection = (range) => {
  const node = range?.startContainer
  if (!node) return null
  if (node.nodeType === Node.TEXT_NODE) return { node, offset: range.startOffset }
  if (node.nodeType !== Node.ELEMENT_NODE) return null
  const child = node.childNodes[Math.max(0, range.startOffset - 1)]
  if (child?.nodeType === Node.TEXT_NODE) return { node: child, offset: child.textContent.length }
  const walker = document.createTreeWalker(node, NodeFilter.SHOW_TEXT)
  let current = null
  let last = null
  while ((current = walker.nextNode())) last = current
  return last ? { node: last, offset: last.textContent.length } : null
}

const removeActiveQuickInsertQuery = () => {
  const selection = window.getSelection?.()
  if (!selection?.rangeCount) return false
  const range = selection.getRangeAt(0)
  if (!range.collapsed) return false
  const textInfo = textNodeBeforeSelection(range)
  if (!textInfo?.node) return false

  const text = String(textInfo.node.textContent || '')
  const beforeCursor = text.slice(0, textInfo.offset)
  const slashIndex = beforeCursor.lastIndexOf('/')
  if (slashIndex < 0) return false
  const query = beforeCursor.slice(slashIndex)
  if (!QUICK_INSERT_QUERY_RE.test(query)) return false

  const deleteRange = document.createRange()
  deleteRange.setStart(textInfo.node, slashIndex)
  deleteRange.setEnd(textInfo.node, textInfo.offset)
  selection.removeAllRanges()
  selection.addRange(deleteRange)

  let removed = false
  try {
    removed = document.execCommand('delete') === true
  } catch {
    removed = false
  }
  if (!removed) {
    textInfo.node.textContent = `${text.slice(0, slashIndex)}${text.slice(textInfo.offset)}`
    const nextRange = document.createRange()
    nextRange.setStart(textInfo.node, slashIndex)
    nextRange.collapse(true)
    selection.removeAllRanges()
    selection.addRange(nextRange)
    removed = true
  }
  console.info(`${COMMAND_LOG_PREFIX} removed quick insert query`, { query })
  return removed
}

const addonWritingCommands = () => {
  const manager = globalThis?.__ELEPHANT_ADDONS__
  if (typeof manager?.getContributions !== 'function') return []
  return manager.getContributions('editor.extensions')
    .flatMap((entry) => Array.isArray(entry?.contribution?.writingCommands)
      ? entry.contribution.writingCommands.map((command) => ({ addonId: entry.addonId, command }))
      : [])
}

const runAddonWritingCommand = (command, payload) => {
  const entry = addonWritingCommands().find(({ command: candidate }) => normalizeCommand(candidate?.id) === command)
  if (!entry || typeof entry.command.run !== 'function') return false
  try {
    entry.command.run(payload, {
      addonId: entry.addonId,
      commandId: command,
      addons: globalThis.__ELEPHANT_ADDONS__
    })
    return true
  } catch (error) {
    console.error(`${COMMAND_LOG_PREFIX} addon command failed`, {
      addonId: entry.addonId,
      command,
      error
    })
    return false
  }
}

const normalizeRequest = (request, payload) => {
  if (request && typeof request === 'object') {
    return {
      command: normalizeCommand(request.command),
      payload: request.payload
    }
  }
  return { command: normalizeCommand(request), payload }
}

const runWritingCommand = (request, payload) => {
  const normalized = normalizeRequest(request, payload)
  if (!normalized.command) return false
  if (shouldSkipDuplicateCommand(normalized.command)) {
    console.info(`${COMMAND_LOG_PREFIX} duplicate ignored`, { command: normalized.command })
    return true
  }
  removeActiveQuickInsertQuery()
  console.info(`${COMMAND_LOG_PREFIX} run`, { command: normalized.command })
  if (runAddonWritingCommand(normalized.command, normalized.payload)) return true

  switch (normalized.command) {
    case 'heading-2':
      bus.emit('paragraph', 'heading 2')
      return true
    case 'bold':
      bus.emit('format', 'strong')
      return true
    case 'italic':
      bus.emit('format', 'em')
      return true
    case 'strike':
      bus.emit('format', 'del')
      return true
    case 'link':
      bus.emit('format', 'link')
      return true
    case 'bullets':
      bus.emit('paragraph', 'ul-bullet')
      return true
    case 'numbers':
      bus.emit('paragraph', 'ol-order')
      return true
    case 'tasks':
      bus.emit('paragraph', 'ul-task')
      return true
    case 'code':
      bus.emit('format', 'inline_code')
      return true
    case 'quote':
      bus.emit('paragraph', 'blockquote')
      return true
    case 'table':
      bus.emit('paragraph', 'table')
      return true
    case 'horizontal-rule':
      bus.emit('insert-horizontal-rule')
      return true
    default:
      console.warn(`${COMMAND_LOG_PREFIX} unknown`, { command: normalized.command })
      return false
  }
}

export const installWritingCommandBridge = (target = globalThis) => {
  if (target.__ELEPHANT_WRITING_COMMAND_BRIDGE__) return false
  target.__ELEPHANT_WRITING_COMMAND_BRIDGE__ = true
  bus.on('elephantnote-writing-command', runWritingCommand)
  console.info(`${COMMAND_LOG_PREFIX} installed`)
  return true
}
