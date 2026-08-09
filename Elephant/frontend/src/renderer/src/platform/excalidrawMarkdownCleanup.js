import bus from '@/bus'
import { useEditorStore } from '@/store/editor'

const QUICK_INSERT_QUERY_LINE_RE = /^\/[\p{L}\p{N}_-]*$/u
const QUICK_INSERT_SUFFIX_RE = /^\s*\/[\p{L}\p{N}_-]*\s*$/u
const EXCALIDRAW_IMAGE_RE = /!\[[^\]]*\]\([^)]*\.assets\/excalidraw-[^)]+\.png[^)]*\)/i

const cleanedTabs = new Map()

const normalizeMarkdown = (value = '') => String(value || '').replace(/\r\n?/g, '\n')
const isTransientRollback = (previousMarkdown = '', nextMarkdown = '') => {
  const previous = normalizeMarkdown(previousMarkdown)
  const next = normalizeMarkdown(nextMarkdown)
  if (!previous || previous.length <= next.length) return false
  if (previous.startsWith(next) && QUICK_INSERT_SUFFIX_RE.test(previous.slice(next.length))) return true
  return EXCALIDRAW_IMAGE_RE.test(previous) && !EXCALIDRAW_IMAGE_RE.test(next)
}

const installStaleContentGuard = (editorStore) => {
  if (editorStore.__ELEPHANT_EXCALIDRAW_STALE_CONTENT_GUARD__?.dispose) {
    return editorStore.__ELEPHANT_EXCALIDRAW_STALE_CONTENT_GUARD__
  }
  const original = editorStore.LISTEN_FOR_CONTENT_CHANGE
  if (typeof original !== 'function') return { dispose() {} }

  const runtime = {
    dispose() {
      if (editorStore.LISTEN_FOR_CONTENT_CHANGE === guarded) {
        editorStore.LISTEN_FOR_CONTENT_CHANGE = original
      }
      if (editorStore.__ELEPHANT_EXCALIDRAW_STALE_CONTENT_GUARD__ === runtime) {
        delete editorStore.__ELEPHANT_EXCALIDRAW_STALE_CONTENT_GUARD__
      }
    }
  }

  const guarded = (change = {}) => {
    const tab = change.id ? editorStore.tabs.find((item) => item.id === change.id) : null
    const previousMarkdown = typeof tab?.markdown === 'string' ? tab.markdown : ''
    const nextMarkdown = typeof change.markdown === 'string' ? change.markdown : ''
    if (nextMarkdown && isTransientRollback(previousMarkdown, nextMarkdown)) {
      console.warn('[excalidraw-cleanup] ignored stale editor rollback', {
        id: change.id || '',
        pathname: tab?.pathname || '',
        previousLength: previousMarkdown.length,
        nextLength: nextMarkdown.length
      })
      return
    }
    return original.call(editorStore, change)
  }

  editorStore.LISTEN_FOR_CONTENT_CHANGE = guarded
  editorStore.__ELEPHANT_EXCALIDRAW_STALE_CONTENT_GUARD__ = runtime
  return runtime
}

const nextMeaningfulLine = (lines, startIndex) => {
  for (let index = startIndex + 1; index < lines.length; index += 1) {
    if (lines[index].trim()) return lines[index]
  }
  return ''
}

const removeQueriesBeforeDrawings = (markdown = '') => {
  if (typeof markdown !== 'string' || !EXCALIDRAW_IMAGE_RE.test(markdown)) {
    return { markdown, removed: 0 }
  }
  const lines = markdown.split('\n')
  const kept = []
  let removed = 0
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index]
    if (QUICK_INSERT_QUERY_LINE_RE.test(line.trim()) && EXCALIDRAW_IMAGE_RE.test(nextMeaningfulLine(lines, index))) {
      removed += 1
      continue
    }
    kept.push(line)
  }
  if (!removed) return { markdown, removed: 0 }
  return {
    markdown: kept.join('\n').replace(/\n{3,}/g, '\n\n').trimEnd(),
    removed
  }
}

const applyCleanMarkdown = (tab, nextMarkdown, removed) => {
  if (!tab?.id || typeof nextMarkdown !== 'string' || tab.markdown === nextMarkdown) return false
  const cacheKey = `${tab.id}:${nextMarkdown}`
  if (cleanedTabs.get(tab.id) === cacheKey) return false
  cleanedTabs.set(tab.id, cacheKey)
  tab.markdown = nextMarkdown
  tab.isSaved = false
  bus.emit('file-changed', {
    id: tab.id,
    markdown: nextMarkdown,
    cursor: tab.cursor || null,
    muyaIndexCursor: tab.muyaIndexCursor || null,
    renderCursor: false,
    history: tab.history,
    blocks: tab.blocks
  })
  bus.emit('invalidate-image-cache')
  console.info('[excalidraw-cleanup] removed stale quick insert query', {
    id: tab.id,
    pathname: tab.pathname || null,
    removed,
    markdownLength: nextMarkdown.length
  })
  return true
}

const cleanCurrentTab = (editorStore) => {
  const currentId = editorStore.currentFile?.id
  const tab = currentId ? editorStore.tabs.find((item) => item.id === currentId) : editorStore.currentFile
  if (!tab?.id || typeof tab.markdown !== 'string') return false
  const result = removeQueriesBeforeDrawings(tab.markdown)
  if (!result.removed) return false
  return applyCleanMarkdown(tab, result.markdown, result.removed)
}

export const installExcalidrawMarkdownCleanup = () => {
  const editorStore = useEditorStore()
  const existing = editorStore.__ELEPHANT_EXCALIDRAW_MARKDOWN_CLEANUP__
  if (existing?.dispose) return existing

  const staleGuard = installStaleContentGuard(editorStore)
  const handleInvalidate = () => cleanCurrentTab(editorStore)
  cleanCurrentTab(editorStore)
  const unsubscribe = editorStore.$subscribe(handleInvalidate)
  bus.on('invalidate-image-cache', handleInvalidate)

  const runtime = {
    dispose() {
      unsubscribe?.()
      bus.off?.('invalidate-image-cache', handleInvalidate)
      staleGuard?.dispose?.()
      cleanedTabs.clear()
      if (editorStore.__ELEPHANT_EXCALIDRAW_MARKDOWN_CLEANUP__ === runtime) {
        delete editorStore.__ELEPHANT_EXCALIDRAW_MARKDOWN_CLEANUP__
      }
    }
  }
  editorStore.__ELEPHANT_EXCALIDRAW_MARKDOWN_CLEANUP__ = runtime
  console.info('[excalidraw-cleanup] installed')
  return runtime
}
