const fail = (action, message) => {
  throw new Error(`source-playwright action ${action.id} postcondition failed: ${message}`)
}

const hasFrameMotion = (frames) => new Set(frames.map((frame) => frame.sha256)).size > 1
const has = (values, value) => Array.isArray(values) && values.includes(value)

export function assertActionPostcondition (action, { before, after, frames, dropObserved = false }) {
  switch (action.id) {
    case 'launch':
      if (after.route !== 'library' || !after.libraryStillMounted || !has(after.visibleEntries, 'Alpha note')) fail(action, 'library readiness effect is absent')
      return { kind: 'ready', effect: 'library-mounted' }
    case 'move-to-alpha-card':
      if (!hasFrameMotion(frames)) fail(action, 'hover produced no frame change')
      return { kind: 'pointer', effect: 'alpha-card-hover', frameMotion: true }
    case 'open-search':
      if (!after.searchVisible) fail(action, 'search did not open')
      return { kind: 'visibility', effect: 'search-open' }
    case 'search-alpha':
      if (after.query !== action.input || !has(after.resultTitles, 'Alpha note')) fail(action, 'query/result effect is absent')
      return { kind: 'query', effect: 'alpha-result-visible', query: after.query }
    case 'close-search':
      if (after.searchVisible) fail(action, 'search remained visible')
      return { kind: 'visibility', effect: 'search-closed' }
    case 'navigate-all-notes':
      if (after.route !== 'library' || !after.libraryStillMounted || after.currentPath !== '') fail(action, 'all-notes navigation had no effect')
      return { kind: 'navigation', effect: 'root-library' }
    case 'open-alpha-note':
      if (after.route !== 'note-editor' || after.openNote !== 'Alpha note' || !after.editorText) fail(action, 'note editor did not open')
      return { kind: 'navigation', effect: 'alpha-note-open' }
    case 'edit-alpha-note':
      if (!has(after.bodyContains, 'Differential edit marker 2026-06-22.') || !after.persistedFile?.contains) fail(action, 'edit did not reach editor and disk')
      return { kind: 'mutation', effect: 'alpha-note-persisted', path: after.persistedFile.path }
    case 'scroll-alpha-note': {
      const beforeTop = Number(before?.scroll?.scrollTop)
      const afterTop = Number(after?.scroll?.scrollTop)
      const deltaY = Number((afterTop - beforeTop).toFixed(3))
      if (!Number.isFinite(beforeTop) || !Number.isFinite(afterTop) || !after.scroll?.scrollable) fail(action, 'real editor scroll container is absent')
      if (Math.abs(deltaY) < 1) fail(action, `scrollTop did not change (${beforeTop} -> ${afterTop})`)
      if (!hasFrameMotion(frames)) fail(action, 'scroll produced no frame change')
      return { kind: 'scroll', target: after.scroll.target, beforeTop, afterTop, deltaY, requestedDeltaY: action.delta.y, mustChange: true, frameMotion: true }
    }
    case 'close-alpha-note':
      if (after.route !== 'library' || after.openNote !== null) fail(action, 'note remained open')
      return { kind: 'navigation', effect: 'alpha-note-closed' }
    case 'open-create-menu':
      if (!after.createMenuVisible || !has(after.menuItems, 'Note')) fail(action, 'create menu did not open')
      return { kind: 'menu', effect: 'create-menu-open' }
    case 'move-through-create-menu':
      if (!after.createMenuVisible || !hasFrameMotion(frames)) fail(action, 'menu hover produced no effect')
      return { kind: 'pointer', effect: 'create-note-hover', frameMotion: true }
    case 'close-create-menu':
      if (after.createMenuVisible) fail(action, 'create menu remained open')
      return { kind: 'menu', effect: 'create-menu-closed' }
    case 'drag-search-rail-item': {
      const beforeOrder = before?.railOrder || []
      const afterOrder = after?.railOrder || []
      const beforePersisted = before?.railPersistence?.persistedOrder || []
      const afterPersisted = after?.railPersistence?.persistedOrder || []
      if (JSON.stringify(beforeOrder) === JSON.stringify(afterOrder)) fail(action, `DOM rail order did not change (${JSON.stringify(afterOrder)})`)
      if (!afterPersisted.length || JSON.stringify(beforePersisted) === JSON.stringify(afterPersisted)) fail(action, 'preferences.json rail order did not change')
      if (!afterPersisted.includes('search') || !dropObserved) fail(action, 'HTML5 drop was not observed and persisted')
      if (!hasFrameMotion(frames)) fail(action, 'drag produced no frame change')
      return { kind: 'drag', effect: 'rail-reordered', beforeOrder, afterOrder, beforePersisted, afterPersisted, persistedPath: after.railPersistence.persistedPath, dropObserved: true, frameMotion: true }
    }
    default:
      fail(action, 'no postcondition is defined')
  }
}
