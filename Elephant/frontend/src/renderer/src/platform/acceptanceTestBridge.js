import bus from '@/bus'
import { useEditorStore } from '@/store/editor'
import { useVaultStore } from 'elephant-front/stores/vaultStore'
import { listen } from '@tauri-apps/api/event'

const PREFIX = '[acceptance-test]'
const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms))

const compactForTerminal = (value, depth = 0) => {
  if (value === null || typeof value === 'boolean' || typeof value === 'number') return value
  if (typeof value === 'string') return value.length > 240 ? `${value.slice(0, 237)}...` : value
  if (depth >= 2) return Array.isArray(value) ? `[${value.length} items]` : '{...}'
  if (Array.isArray(value)) return value.slice(0, 12).map((item) => compactForTerminal(item, depth + 1))
  if (typeof value === 'object') {
    return Object.fromEntries(Object.entries(value).slice(0, 24).map(([key, item]) => [key, compactForTerminal(item, depth + 1)]))
  }
  return String(value)
}

const log = (target, event, data = {}) => {
  const entry = { at: new Date().toISOString(), event, ...data }
  // Keep the terminal stream readable and bounded. The unbounded structured
  // archive below remains the authoritative log consumed by the acceptance runner.
  console.info(`${PREFIX} ${event} ${JSON.stringify(compactForTerminal(data))}`)
  const logs = target.__ELEPHANT_DEBUG_LOGS__ = Array.isArray(target.__ELEPHANT_DEBUG_LOGS__)
    ? target.__ELEPHANT_DEBUG_LOGS__
    : []
  logs.push(entry)
  if (logs.length > 1000) logs.splice(0, logs.length - 1000)
  const acceptanceLogs = target.__ELEPHANT_ACCEPTANCE_LOGS__ = Array.isArray(target.__ELEPHANT_ACCEPTANCE_LOGS__)
    ? target.__ELEPHANT_ACCEPTANCE_LOGS__
    : []
  acceptanceLogs.push(entry)
  return entry
}

const displayedSurface = (documentObject) => documentObject?.querySelector?.(
  '[contenteditable="true"], [data-muya-editor="true"], .muya-rust-runtime-editor, .cm-content, .muya-editor, .en-editor-host'
)

const isVisible = (target, element) => {
  if (!element || element.hidden || element.getAttribute('aria-hidden') === 'true') return false
  if ((target.document?.defaultView?.getComputedStyle?.(element)?.display || '') === 'none') return false
  return typeof element.getClientRects !== 'function' || element.getClientRects().length > 0
}

const snapshot = (target, editorStore, vaultStore) => {
  const file = editorStore.currentFile || {}
  const surface = displayedSurface(target.document)
  return {
    notePath: vaultStore.openedNotePath || (surface ? file.pathname || '' : ''),
    markdown: typeof file.markdown === 'string' ? file.markdown : '',
    isSaved: file.isSaved !== false,
    displayedText: surface?.innerText || surface?.textContent || '',
    displayedHtml: surface?.innerHTML || '',
    sourceCode: false,
    rustEditorPresent: Boolean(target.document?.querySelector?.('[data-testid="muya-rust-runtime-editor"]')),
    codeMirrorPresent: Boolean(target.document?.querySelector?.('.CodeMirror, .cm-editor')),
    activeVault: vaultStore.activeVault?.path || null
  }
}

const findEntry = (vaultStore, path) => {
  const entries = [...(vaultStore.entries || []), ...(vaultStore.rootEntries || [])]
  return entries.find((entry) => entry?.path === path) || {
    path,
    title: path.split('/').pop()?.replace(/\.md$/i, '') || 'Acceptance test note',
    type: 'file'
  }
}

const invokeApplicationCommand = async(target, command, payload = {}) => {
  const invoke = target.__TAURI__?.core?.invoke
  if (typeof invoke !== 'function') throw new Error(`${command} requires the Tauri command bridge`)
  log(target, 'tauri:invoke:start', { command })
  try {
    const result = await invoke(command, payload)
    log(target, 'tauri:invoke:done', { command })
    return result
  } catch (error) {
    log(target, 'tauri:invoke:error', { command, error: error?.message || String(error) })
    throw error
  }
}

export const installAcceptanceTestBridge = ({
  target = globalThis,
  pinia,
  editorStore = useEditorStore(pinia),
  vaultStore = useVaultStore(pinia)
} = {}) => {
  if (target.__ELEPHANT_ACCEPTANCE_TEST__) return target.__ELEPHANT_ACCEPTANCE_TEST__
  target.__ELEPHANT_ACCEPTANCE_LOGS__ = []

  const api = {
    async invokeTauri(command, payload = {}) {
      if (!(command === 'healthcheck' || /^tauri_[a-z0-9_]+$/i.test(command)) || command.startsWith('tauri_acceptance_')) {
        throw new TypeError(`invokeTauri only accepts application Tauri commands: ${command}`)
      }
      return invokeApplicationCommand(target, command, payload)
    },

    setLocalStorage(key, value = null) {
      if (typeof key !== 'string' || !key) throw new TypeError('setLocalStorage requires a key')
      if (value === null || value === undefined) target.localStorage?.removeItem?.(key)
      else target.localStorage?.setItem?.(key, String(value))
      log(target, 'storage:local:set', { key, present: value !== null && value !== undefined })
      return { key, value: value === null || value === undefined ? null : String(value) }
    },

    readDom(selector) {
      if (!selector || typeof selector !== 'string') throw new TypeError('readDom requires a CSS selector')
      const element = target.document?.querySelector?.(selector)
      const attributes = {}
      if (element?.attributes) {
        for (const attribute of element.attributes) attributes[attribute.name] = attribute.value
      }
      const result = {
        selector,
        exists: Boolean(element),
        visible: isVisible(target, element),
        value: element && 'value' in element ? element.value : null,
        text: element?.innerText || element?.textContent || '',
        html: element?.innerHTML || '',
        attributes
      }
      log(target, 'dom:read', { selector, exists: result.exists, textLength: result.text.length, htmlLength: result.html.length })
      return result
    },

    click(selector) {
      if (!selector || typeof selector !== 'string') throw new TypeError('click requires a CSS selector')
      const element = target.document?.querySelector?.(selector)
      if (!element) throw new Error(`click target was not found: ${selector}`)
      element.click()
      log(target, 'dom:click', { selector })
      return api.readDom(selector)
    },

    fill(selector, value) {
      if (!selector || typeof selector !== 'string') throw new TypeError('fill requires a CSS selector')
      if (typeof value !== 'string') throw new TypeError('fill requires a string value')
      const element = target.document?.querySelector?.(selector)
      if (!element || !('value' in element)) throw new Error(`fill target was not found or is not an input: ${selector}`)
      element.value = value
      element.dispatchEvent(new target.Event('input', { bubbles: true }))
      element.dispatchEvent(new target.Event('change', { bubbles: true }))
      log(target, 'dom:fill', { selector, valueLength: value.length })
      return api.readDom(selector)
    },

    insertText(selector, value) {
      if (!selector || typeof selector !== 'string') throw new TypeError('insertText requires a CSS selector')
      if (typeof value !== 'string') throw new TypeError('insertText requires a string value')
      const element = target.document?.querySelector?.(selector)
      if (!element) throw new Error(`insertText target was not found: ${selector}`)
      const selectionBeforeFocus = target.getSelection?.() || target.window?.getSelection?.()
      const savedRange = selectionBeforeFocus?.rangeCount ? selectionBeforeFocus.getRangeAt(0).cloneRange() : null
      element.focus?.()
      if (savedRange && element.contains(savedRange.commonAncestorContainer)) {
        const selectionAfterFocus = target.getSelection?.() || target.window?.getSelection?.()
        selectionAfterFocus?.removeAllRanges()
        selectionAfterFocus?.addRange(savedRange)
      }
      const InputEventConstructor = target.InputEvent || target.window?.InputEvent
      if (typeof InputEventConstructor !== 'function') throw new Error('insertText requires InputEvent support')
      const inputEvent = new InputEventConstructor('beforeinput', {
        inputType: 'insertText',
        data: value,
        bubbles: true,
        cancelable: true,
        composed: true
      })
      const dispatched = element.dispatchEvent(inputEvent)
      const activeMuya = target.__ELEPHANT_ACTIVE_MUYA__
      // WebKit can drop a synthetic beforeinput event at the contenteditable
      // boundary. If it was not cancelled by Muya's canonical handler, invoke
      // that same handler explicitly so acceptance tests exercise the real
      // Rust command path instead of mutating the DOM themselves.
      if (dispatched && typeof activeMuya?.__beforeInput === 'function') {
        log(target, 'dom:insert-text:handler-fallback', { selector, valueLength: value.length })
        activeMuya.__beforeInput(inputEvent)
      }
      if (dispatched && !inputEvent.defaultPrevented && element.getAttribute?.('contenteditable') === 'true') {
        const selection = target.getSelection?.() || target.window?.getSelection?.()
        const range = selection?.rangeCount ? selection.getRangeAt(0) : null
        if (range && element.contains(range.commonAncestorContainer)) {
          range.deleteContents()
          const textNode = target.document.createTextNode(value)
          range.insertNode(textNode)
          range.setStartAfter(textNode)
          range.collapse(true)
          selection.removeAllRanges()
          selection.addRange(range)
      const InputEventForDocument = element.ownerDocument.defaultView?.InputEvent ||
        target.InputEvent ||
        target.window?.InputEvent
          const domInput = typeof InputEventForDocument === 'function'
            ? new InputEventForDocument('input', { inputType: 'insertText', data: value, bubbles: true, composed: true })
            : new element.ownerDocument.defaultView.Event('input', { bubbles: true, composed: true })
          element.dispatchEvent(domInput)
          log(target, 'dom:insert-text:contenteditable', { selector, valueLength: value.length })
        }
      }
      log(target, 'dom:insert-text', { selector, valueLength: value.length, dispatched, defaultPrevented: inputEvent.defaultPrevented })
      return api.readDom(selector)
    },

    async waitFor(selector, timeoutMs = 5000) {
      if (!selector || typeof selector !== 'string') throw new TypeError('waitFor requires a CSS selector')
      const deadline = Date.now() + Math.max(0, timeoutMs)
      while (Date.now() <= deadline) {
        const element = target.document?.querySelector?.(selector)
        if (element) {
          const result = api.readDom(selector)
          log(target, 'dom:wait:done', { selector })
          return result
        }
        await wait(25)
      }
      log(target, 'dom:wait:error', { selector, timeoutMs })
      throw new Error(`Timed out waiting for selector: ${selector}`)
    },

    async waitUntilGone(selector, timeoutMs = 5000) {
      if (!selector || typeof selector !== 'string') throw new TypeError('waitUntilGone requires a CSS selector')
      const deadline = Date.now() + Math.max(0, timeoutMs)
      while (Date.now() <= deadline) {
        const element = target.document?.querySelector?.(selector)
        const visible = isVisible(target, element)
        if (!element || !visible) {
          log(target, 'dom:wait-gone:done', { selector, exists: Boolean(element), visible })
          return { selector, exists: Boolean(element), visible }
        }
        await wait(25)
      }
      log(target, 'dom:wait-gone:error', { selector, timeoutMs })
      throw new Error(`Timed out waiting for selector to disappear: ${selector}`)
    },

    executeCommand(commandId) {
      if (!/^(file\.save|view\.toggle-sidebar|view\.toggle-tabbar|file\.close-tab)$/.test(commandId)) {
        throw new TypeError(`executeCommand does not allow this command: ${commandId}`)
      }
      log(target, 'app:command:start', { commandId })
      bus.emit('cmd::execute', commandId)
      log(target, 'app:command:done', { commandId })
      return { commandId }
    },

    press(selector, key, modifiers = {}) {
      if (!selector || typeof selector !== 'string') throw new TypeError('press requires a CSS selector')
      if (!key || typeof key !== 'string') throw new TypeError('press requires a key')
      const element = target.document?.querySelector?.(selector)
      if (!element) throw new Error(`press target was not found: ${selector}`)
      const KeyboardEventConstructor = target.KeyboardEvent || target.window?.KeyboardEvent
      if (typeof KeyboardEventConstructor !== 'function') throw new Error('press requires KeyboardEvent support')
      const eventInit = {
        key,
        bubbles: true,
        cancelable: true,
        altKey: !!modifiers.altKey,
        ctrlKey: !!modifiers.ctrlKey,
        metaKey: !!modifiers.metaKey,
        shiftKey: !!modifiers.shiftKey
      }
      element.dispatchEvent(new KeyboardEventConstructor('keydown', eventInit))
      element.dispatchEvent(new KeyboardEventConstructor('keyup', eventInit))
      log(target, 'dom:press', { selector, key, modifiers: eventInit })
      return api.readDom(selector)
    },

    selectText(selector, start = 0, end = undefined) {
      if (!selector || typeof selector !== 'string') throw new TypeError('selectText requires a CSS selector')
      const element = target.document?.querySelector?.(selector)
      if (!element) throw new Error(`selectText target was not found: ${selector}`)
      const text = element.textContent || ''
      const from = Math.max(0, Math.min(text.length, Number(start) || 0))
      const to = Math.max(from, Math.min(text.length, end === undefined ? text.length : Number(end) || 0))
      const createRange = () => {
        const walker = target.document.createTreeWalker(element, target.NodeFilter?.SHOW_TEXT || 4)
        const nodes = []
        let offset = 0
        let node
        while ((node = walker.nextNode())) {
          nodes.push({ node, start: offset, end: offset + node.textContent.length })
          offset += node.textContent.length
        }
        const locate = (position) => {
          const entry = nodes.find((candidate) => position >= candidate.start && position <= candidate.end) || nodes[nodes.length - 1]
          if (!entry) throw new Error(`selectText target has no text nodes: ${selector}`)
          return { node: entry.node, offset: Math.max(0, Math.min(entry.node.textContent.length, position - entry.start)) }
        }
        const range = target.document.createRange()
        const rangeStart = locate(from)
        const rangeEnd = locate(to)
        range.setStart(rangeStart.node, rangeStart.offset)
        range.setEnd(rangeEnd.node, rangeEnd.offset)
        return range
      }
      const range = createRange()
      const selection = target.getSelection?.() || target.document.defaultView?.getSelection?.()
      if (!selection) throw new Error('selectText requires Selection support')
      selection.removeAllRanges()
      selection.addRange(range)
      target.document.dispatchEvent(new target.Event('selectionchange', { bubbles: true }))
      element.dispatchEvent(new target.MouseEvent('mouseup', { bubbles: true, button: 0 }))
      const activeMuya = target.__ELEPHANT_ACTIVE_MUYA__
      if (activeMuya?.contentState) {
        const selectionChanges = activeMuya.contentState.selectionChange()
        log(target, 'muya:selection-sync', {
          start: selectionChanges?.start,
          end: selectionChanges?.end,
          textLength: selection.toString().length
        })
        if (selectionChanges?.start && selectionChanges?.end) {
          activeMuya.contentState.cursor = {
            start: { ...selectionChanges.start },
            end: { ...selectionChanges.end },
            isEdit: true
          }
        }
        activeMuya.dispatchSelectionChange?.(activeMuya.contentState.cursor)
      }
      // Muya synchronizes its cursor through the editor adapter and may clear
      // the browser selection while publishing that state. Keep the real DOM
      // range selected so the following user action (for example, citation)
      // observes the same selection that was just created.
      selection.removeAllRanges()
      const restoredRange = createRange()
      selection.addRange(restoredRange)
      const selectedText = selection.toString() || restoredRange.toString()
      const result = { selector, start: from, end: to, text: selectedText }
      log(target, 'dom:select-text', { selector, start: from, end: to, textLength: result.text.length })
      return result
    },

    contextClick(selector) {
      if (!selector || typeof selector !== 'string') throw new TypeError('contextClick requires a CSS selector')
      const element = target.document?.querySelector?.(selector)
      if (!element) throw new Error(`contextClick target was not found: ${selector}`)
      element.dispatchEvent(new target.MouseEvent('contextmenu', { bubbles: true, cancelable: true, button: 2, clientX: 20, clientY: 20 }))
      log(target, 'dom:context-click', { selector })
      return api.readDom(selector)
    },

    capabilities() {
      const result = {
        runtime: target.__MARKTEXT_RUNTIME__ || 'tauri',
        commands: Object.keys(api).filter((key) => key !== 'logs' && key !== 'capabilities'),
        activeVault: vaultStore.activeVault?.path || null,
        notePath: editorStore.currentFile?.pathname || vaultStore.openedNotePath || null
      }
      log(target, 'capabilities:read', { commandCount: result.commands.length })
      return result
    },

    addonState() {
      const manager = target.__ELEPHANT_ADDONS__
      if (!manager) throw new Error('Addon manager is unavailable')
      const resourceNames = manager.host?.list?.() || []
      const resourceMethods = Object.fromEntries(resourceNames.map((name) => {
        const resource = manager.host?.get?.(name)
        return [name, resource
          ? Object.keys(resource).filter((key) => typeof resource[key] === 'function').sort()
          : []]
      }))
      const result = {
        addons: manager.list().map((entry) => ({
          id: entry.manifest?.id || '',
          name: entry.manifest?.name || '',
          enabled: entry.enabled === true,
          status: entry.status || '',
          error: entry.error || null
        })),
        resources: resourceNames,
        resourceMethods,
        actions: manager.getActions?.().map((entry) => entry.contribution?.id).filter(Boolean) || []
      }
      log(target, 'addons:state', { addonCount: result.addons.length, resourceCount: result.resources.length, resourceMethodCount: Object.values(resourceMethods).reduce((total, methods) => total + methods.length, 0), actionCount: result.actions.length })
      return result
    },

    async enableAddon(id) {
      const manager = target.__ELEPHANT_ADDONS__
      if (!manager) throw new Error('Addon manager is unavailable')
      log(target, 'addons:enable:start', { id })
      try {
        const addon = manager.get(id)
        if (addon?.manifest?.source === 'external' || addon?.manifest?.source === 'official' || addon?.manifest?.official === true) {
          await invokeApplicationCommand(target, 'tauri_addons_set_enabled', { addonId: id, enabled: true })
        }
        const result = await manager.enable(id)
        log(target, 'addons:enable:done', { id })
        return result
      } catch (error) {
        await invokeApplicationCommand(target, 'tauri_addons_set_enabled', { addonId: id, enabled: false }).catch(() => {})
        log(target, 'addons:enable:error', { id, error: error?.message || String(error) })
        throw error
      }
    },

    async disableAddon(id) {
      const manager = target.__ELEPHANT_ADDONS__
      if (!manager) throw new Error('Addon manager is unavailable')
      log(target, 'addons:disable:start', { id })
      try {
        const result = await manager.disable(id)
        const addon = manager.get(id)
        if (addon?.manifest?.source === 'external' || addon?.manifest?.source === 'official' || addon?.manifest?.official === true) {
          await invokeApplicationCommand(target, 'tauri_addons_set_enabled', { addonId: id, enabled: false })
        }
        log(target, 'addons:disable:done', { id })
        return result
      } catch (error) {
        log(target, 'addons:disable:error', { id, error: error?.message || String(error) })
        throw error
      }
    },

    async runAddonAction(id, payload = undefined) {
      const manager = target.__ELEPHANT_ADDONS__
      if (!manager) throw new Error('Addon manager is unavailable')
      log(target, 'addons:action:start', { id })
      try {
        const result = await manager.runAction(id, payload)
        log(target, 'addons:action:done', { id })
        return result
      } catch (error) {
        log(target, 'addons:action:error', { id, error: error?.message || String(error) })
        throw error
      }
    },

    async installOfficialAddon(id) {
      if (typeof id !== 'string' || !id.trim()) throw new TypeError('installOfficialAddon requires an addon id')
      const result = await invokeApplicationCommand(target, 'tauri_official_addons_catalog_install', { addonId: id.trim() })
      await target.__ELEPHANT_ADDONS__?.external?.reload?.()
      log(target, 'addons:official-install:done', { id: id.trim() })
      return result
    },

    async invokeAddonResource(name, method, payload = undefined) {
      const manager = target.__ELEPHANT_ADDONS__
      const resource = manager?.host?.get?.(name)
      if (!resource || typeof resource[method] !== 'function') throw new Error(`Addon resource method is unavailable: ${name}.${method}`)
      log(target, 'addons:resource:start', { name, method })
      try {
        const result = await resource[method](payload)
        log(target, 'addons:resource:done', { name, method })
        return result
      } catch (error) {
        log(target, 'addons:resource:error', { name, method, error: error?.message || String(error) })
        throw error
      }
    },

    async addonNativeStatus(id, kind = 'sidecar') {
      if (typeof id !== 'string' || !id.trim()) throw new TypeError('addonNativeStatus requires an addon id')
      const command = kind === 'service' ? 'tauri_addons_service_status' : 'tauri_addons_sidecar_status'
      const result = await invokeApplicationCommand(target, command, { addonId: id.trim() })
      log(target, 'addons:native-status', { id: id.trim(), kind, available: result?.available, running: result?.running, error: result?.error || null })
      return result
    },

    async addonNativeCall(id, method, payload = {}, options = {}) {
      if (typeof id !== 'string' || !id.trim()) throw new TypeError('addonNativeCall requires an addon id')
      if (typeof method !== 'string' || !method.trim()) throw new TypeError('addonNativeCall requires a method')
      const command = options?.service === true ? 'tauri_addons_service_call' : 'tauri_addons_sidecar_call'
      const result = await invokeApplicationCommand(target, command, {
        addonId: id.trim(),
        method: method.trim(),
        params: payload,
        timeoutMs: Number.isFinite(options?.timeoutMs) ? Math.max(1, Math.trunc(options.timeoutMs)) : undefined
      })
      log(target, 'addons:native-call:done', { id: id.trim(), kind: options?.service === true ? 'service' : 'sidecar', method: method.trim() })
      return result
    },

    async listNotes(path = '') {
      const entries = await invokeApplicationCommand(target, 'tauri_directory_list', {
        relativePath: path,
        offset: 0,
        limit: 1000,
        includePreview: false
      })
      const files = Array.isArray(entries)
        ? entries.filter((entry) => entry?.type === 'note' || entry?.kind === 'note')
        : []
      log(target, 'notes:list', { path, count: files.length })
      return files
    },

    async selectVault(path) {
      if (!path || typeof path !== 'string') throw new TypeError('selectVault requires an absolute vault path')
      const result = await invokeApplicationCommand(target, 'tauri_vaults_select_path', { vaultPath: path })
      vaultStore.applyPayload(result)
      log(target, 'vault:select', { path, vaultId: result?.activeVaultId || null })
      return result
    },

    async readNote(path) {
      if (!path || typeof path !== 'string') throw new TypeError('readNote requires a relative Markdown path')
      const result = await invokeApplicationCommand(target, 'tauri_notes_read', { relativePath: path })
      const content = typeof result?.content === 'string' ? result.content : ''
      log(target, 'note:read', { path, markdownLength: content.length })
      return { ...result, content }
    },

    async createNote(path, filename = 'Acceptance-created.md') {
      const result = await invokeApplicationCommand(target, 'tauri_notes_create', { relativePath: path || null, filename, title: null })
      log(target, 'note:create', { path, filename })
      return result
    },

    async openNote(path) {
      if (!path || typeof path !== 'string') throw new TypeError('openNote requires a relative Markdown path')
      if (!vaultStore.activeVault?.path) throw new Error('openNote requires an active vault')
      log(target, 'open:start', { path })
      vaultStore.openNote(findEntry(vaultStore, path), { record: false })
      for (let attempt = 0; attempt < 100; attempt += 1) {
        if (editorStore.currentFile?.pathname?.endsWith(path)) {
const state = snapshot(target, editorStore, vaultStore)
          log(target, 'open:done', { path, markdownLength: state.markdown.length })
          return state
        }
        await wait(25)
      }
      throw new Error(`Timed out opening note: ${path}`)
    },

    setMarkdown(markdown) {
      if (typeof markdown !== 'string') throw new TypeError('setMarkdown requires a string')
      if (!editorStore.currentFile?.id) throw new Error('setMarkdown requires an open note')
      editorStore.currentFile.markdown = markdown
      editorStore.currentFile.isSaved = false
      bus.emit('file-changed', {
        id: editorStore.currentFile.id,
        markdown,
        cursor: editorStore.currentFile.cursor,
        renderCursor: true,
        history: editorStore.currentFile.history,
        scrollTop: editorStore.currentFile.scrollTop
      })
const state = snapshot(target, editorStore, vaultStore)
      log(target, 'edit:set-markdown', { notePath: state.notePath, markdownLength: markdown.length })
      return state
    },

    appendMarkdown(text) {
      if (typeof text !== 'string') throw new TypeError('appendMarkdown requires a string')
      return api.setMarkdown(`${editorStore.currentFile?.markdown || ''}${text}`)
    },

    async save() {
      if (!editorStore.currentFile?.id) throw new Error('save requires an open note')
      log(target, 'save:start', { notePath: editorStore.currentFile.pathname || '' })
      const pathname = editorStore.currentFile.pathname || ''
      const vaultPath = vaultStore.activeVault?.path || ''
      const normalizedPathname = pathname.replace(/^\/private\//, '/')
      const normalizedVaultPath = vaultPath.replace(/^\/private\//, '/')
      const relativePath = normalizedPathname.startsWith(`${normalizedVaultPath}/`)
        ? normalizedPathname.slice(normalizedVaultPath.length + 1)
        : pathname
      const expectedMarkdown = editorStore.currentFile.markdown || ''
      if (typeof target.__TAURI__?.core?.invoke === 'function') {
        log(target, 'save:write:start', { notePath: relativePath, markdownLength: expectedMarkdown.length })
        await invokeApplicationCommand(target, 'tauri_notes_write', { relativePath, content: expectedMarkdown, markdown: expectedMarkdown })
        editorStore.currentFile.isSaved = true
        log(target, 'save:write:done', { notePath: relativePath, markdownLength: expectedMarkdown.length })
      } else {
        editorStore.FILE_SAVE?.()
      }
      for (let attempt = 0; attempt < 100; attempt += 1) {
        const expectedMarkdown = editorStore.currentFile?.markdown || ''
        let persisted = true
        if (typeof target.__TAURI__?.core?.invoke === 'function' && editorStore.currentFile?.pathname) {
          try {
            const pathname = editorStore.currentFile.pathname
            const vaultPath = vaultStore.activeVault?.path || ''
            const normalizedPathname = pathname.replace(/^\/private\//, '/')
            const normalizedVaultPath = vaultPath.replace(/^\/private\//, '/')
            const relativePath = normalizedPathname.startsWith(`${normalizedVaultPath}/`)
              ? normalizedPathname.slice(normalizedVaultPath.length + 1)
              : pathname
            const result = await invokeApplicationCommand(target, 'tauri_notes_read', { relativePath })
            persisted = result?.content === expectedMarkdown || result?.markdown === expectedMarkdown
          } catch {
            persisted = false
          }
        }
        if (editorStore.currentFile?.isSaved === true && persisted) {
const state = snapshot(target, editorStore, vaultStore)
          log(target, 'save:done', { notePath: state.notePath, markdownLength: state.markdown.length })
          return state
        }
        await wait(25)
      }
      throw new Error(`Timed out saving note: ${editorStore.currentFile.pathname || ''}`)
    },

    readDisplayed() {
const state = snapshot(target, editorStore, vaultStore)
      log(target, 'read:displayed', { notePath: state.notePath, displayedLength: state.displayedText.length })
      return state
    },

    async openExcalidraw(fileName = 'acceptance.excalidraw.png') {
      bus.emit('ELEPHANT::open-excalidraw', { fileName, title: 'Acceptance Excalidraw', saveMode: 'png', insertOnSave: false })
      for (let attempt = 0; attempt < 100; attempt += 1) {
        const dialog = target.document?.querySelector?.('[data-testid="excalidraw-dialog"]')
        if (dialog) {
          const state = { open: true, hasCanvas: Boolean(dialog.querySelector('.en-excalidraw-canvas canvas')), hasError: Boolean(dialog.querySelector('[role="alert"]')) }
          if (state.hasError) throw new Error('Excalidraw dialog reported an initialization error')
          if (state.hasCanvas) {
            log(target, 'excalidraw:open', state)
            return state
          }
        }
        await wait(25)
      }
      throw new Error('Timed out opening Excalidraw')
    },

    async closeExcalidraw() {
      const close = target.document?.querySelector?.('[data-testid="excalidraw-close"]')
      if (!close) throw new Error('Excalidraw dialog is not open')
      close.click()
      for (let attempt = 0; attempt < 100; attempt += 1) {
        if (!target.document?.querySelector?.('[data-testid="excalidraw-dialog"]')) {
          const state = { open: false }
          log(target, 'excalidraw:close', state)
          return state
        }
        await wait(25)
      }
      const state = { open: true }
      log(target, 'excalidraw:close', state)
      return state
    },

    readExcalidraw() {
      const dialog = target.document?.querySelector?.('[data-testid="excalidraw-dialog"]')
      const state = { open: Boolean(dialog), hasCanvas: Boolean(dialog?.querySelector('.en-excalidraw-canvas')), hasError: Boolean(dialog?.querySelector('[role="alert"]')) }
      log(target, 'excalidraw:state', state)
      return state
    },

    readState() {
const state = snapshot(target, editorStore, vaultStore)
      log(target, 'read:state', { notePath: state.notePath, markdownLength: state.markdown.length })
      return state
    },

    logs() {
      return [...(target.__ELEPHANT_ACCEPTANCE_LOGS__ || target.__ELEPHANT_DEBUG_LOGS__ || [])]
    }
  }

  target.__ELEPHANT_ACCEPTANCE_TEST__ = api
  log(target, 'installed', { commands: Object.keys(api).filter((key) => key !== 'logs') })
  void listen('elephant:acceptance:command', async({ payload }) => {
    const requestId = payload?.request_id
    const command = payload?.command
    const args = Array.isArray(payload?.args) ? payload.args : []
    log(target, 'transport:command:start', { requestId, command, argsCount: args.length, args })
    try {
      if (!requestId || typeof api[command] !== 'function') throw new Error(`Unknown acceptance command: ${command}`)
      const result = await api[command](...args)
      await target.__TAURI__?.core?.invoke('tauri_acceptance_result', { requestId, result: result ?? null, error: null })
      log(target, 'transport:command:done', {
        requestId,
        command,
        result: command === 'logs' ? { logCount: Array.isArray(result) ? result.length : 0 } : result ?? null
      })
    } catch (error) {
      const message = error?.stack || error?.message || String(error)
      log(target, 'transport:command:error', { requestId, command, error: message })
      await target.__TAURI__?.core?.invoke('tauri_acceptance_result', { requestId, result: null, error: message })
    }
  }).then(() => target.__TAURI__?.core?.invoke('tauri_acceptance_ready'))
    .then(() => log(target, 'transport:ready'))
    .catch((error) => log(target, 'transport:listen:error', { error: error?.message || String(error) }))
  return api
}
