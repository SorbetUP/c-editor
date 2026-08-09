import { useEditorStore } from '@/store/editor'

const QUICK_INSERT_SUFFIX_RE = /^\s*\/[\p{L}\p{N}_-]*\s*$/u

const normalizeMarkdown = (value = '') => String(value || '').replace(/\r\n?/g, '\n')

const isTransientQuickInsertRollback = (previousMarkdown = '', nextMarkdown = '') => {
  const previous = normalizeMarkdown(previousMarkdown)
  const next = normalizeMarkdown(nextMarkdown)
  if (!previous || previous.length <= next.length) return false
  if (!previous.startsWith(next)) return false
  const removedSuffix = previous.slice(next.length)
  return QUICK_INSERT_SUFFIX_RE.test(removedSuffix)
}

const isFreshInsertedDrawingRollback = (previousMarkdown = '', nextMarkdown = '') => {
  const previous = normalizeMarkdown(previousMarkdown)
  const next = normalizeMarkdown(nextMarkdown)
  if (!previous || previous.length <= next.length) return false
  return /\.assets\/excalidraw-[^)]+\.png/i.test(previous) && !/\.assets\/excalidraw-[^)]+\.png/i.test(next)
}

export const installStaleContentChangeGuard = () => {
  const editorStore = useEditorStore()
  if (editorStore.__ELEPHANT_STALE_CONTENT_CHANGE_GUARD__) return false
  const original = editorStore.LISTEN_FOR_CONTENT_CHANGE?.bind(editorStore)
  if (typeof original !== 'function') return false

  editorStore.__ELEPHANT_STALE_CONTENT_CHANGE_GUARD__ = true
  editorStore.LISTEN_FOR_CONTENT_CHANGE = (change = {}) => {
    const id = change.id
    const tab = id ? editorStore.tabs?.find?.((item) => item.id === id) : null
    const previousMarkdown = typeof tab?.markdown === 'string' ? tab.markdown : ''
    const nextMarkdown = typeof change.markdown === 'string' ? change.markdown : ''

    if (
      nextMarkdown &&
      (isTransientQuickInsertRollback(previousMarkdown, nextMarkdown) || isFreshInsertedDrawingRollback(previousMarkdown, nextMarkdown))
    ) {
      console.warn('[editor-state] ignored stale content change', {
        id,
        pathname: tab?.pathname || '',
        previousLength: previousMarkdown.length,
        nextLength: nextMarkdown.length,
        reason: isTransientQuickInsertRollback(previousMarkdown, nextMarkdown)
          ? 'quick-insert-query-rollback'
          : 'excalidraw-image-rollback'
      })
      return
    }

    return original(change)
  }
  console.info('[editor-state] stale content change guard installed')
  return true
}
