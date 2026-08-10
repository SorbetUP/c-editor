const FLAG = '__ELEPHANTNOTE_MOBILE_LIBRARY_CHROME__'

const isMobile = (target = globalThis) => Boolean(
  target.matchMedia?.('(max-width: 760px), (hover: none) and (pointer: coarse)').matches
)

const svg = (kind) => {
  if (kind === 'list') {
    return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/></svg>'
  }
  if (kind === 'sort') {
    return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m3 6 3-3 3 3M6 3v14M21 18l-3 3-3-3M18 21V7"/></svg>'
  }
  if (kind === 'sort-newest') {
    return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m8 7 4-4 4 4M12 3v12M4 19h16M7 15h10"/></svg>'
  }
  if (kind === 'sort-oldest') {
    return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m8 17 4 4 4-4M12 21V9M4 5h16M7 9h10"/></svg>'
  }
  if (kind === 'sort-title-az') {
    return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h7M3 12h7M3 18h7M14 17l3-3 3 3M17 14v7"/></svg>'
  }
  if (kind === 'sort-title-za') {
    return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h7M3 12h7M3 18h7M14 7l3 3 3-3M17 10V3"/></svg>'
  }
  return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>'
}

const currentView = (target) => {
  return target.document.querySelector('.en-view-cycle')?.dataset.viewMode || 'grid'
}

const setView = (target, mode) => {
  const button = target.document.querySelector('.en-view-cycle')
  if (button && currentView(target) !== mode) button.click()
}

const currentSort = (target) => target.document.querySelector('.en-sort-cycle')?.dataset.sort || 'updated-newest'

const setSort = (target, value, onDone) => {
  const button = target.document.querySelector('.en-sort-cycle')
  if (!button) return
  const advance = () => {
    if (currentSort(target) === value) {
      onDone?.()
      return
    }
    button.click()
    target.requestAnimationFrame(advance)
  }
  advance()
}

const openSortSheet = (target) => {
  const active = currentSort(target)
  const options = [
    ['updated-newest', 'Updated newest', 'sort-newest'],
    ['updated-oldest', 'Updated oldest', 'sort-oldest'],
    ['title-az', 'Title A–Z', 'sort-title-az'],
    ['title-za', 'Title Z–A', 'sort-title-za']
  ]
  const backdrop = target.document.createElement('div')
  backdrop.className = 'en-mobile-sort-sheet-backdrop'
  backdrop.innerHTML = `
    <section class="en-mobile-sort-sheet" role="dialog" aria-modal="true" aria-label="Sort notes">
      <header><strong>Sort notes</strong></header>
      <div>${options.map(([value, label, icon]) => `
        <button type="button" data-sort="${value}" aria-label="${label}" title="${label}" class="${value === active ? 'active' : ''}">
          <span class="check">${value === active ? '✓' : ''}</span>
          ${svg(icon)}
        </button>`).join('')}
      </div>
    </section>
  `
  const close = () => backdrop.remove()
  backdrop.addEventListener('click', (event) => {
    if (event.target === backdrop) close()
    const value = event.target.closest('[data-sort]')?.dataset.sort
    if (!value) return
    setSort(target, value, close)
  })
  target.document.body.appendChild(backdrop)
}

const createControls = (target) => {
  const controls = target.document.createElement('div')
  controls.className = 'en-mobile-library-controls'
  controls.innerHTML = `
    <button type="button" class="en-mobile-library-view" aria-label="Switch note layout"></button>
    <button type="button" class="en-mobile-library-sort" aria-label="Sort notes">${svg('sort')}</button>
  `
  controls.querySelector('.en-mobile-library-view').addEventListener('click', () => {
    setView(target, currentView(target) === 'grid' ? 'list' : 'grid')
    target.requestAnimationFrame(() => refreshControls(target))
  })
  controls.querySelector('.en-mobile-library-sort').addEventListener('click', () => openSortSheet(target))
  return controls
}

const refreshControls = (target) => {
  const topbar = target.document.querySelector('.en-mobile-topbar')
  const libraryOpen = isMobile(target) && !!target.document.querySelector('.en-main:not(.has-editor-open) .en-library')
  const existing = target.document.querySelector('.en-mobile-library-controls')
  if (!topbar || !libraryOpen) {
    existing?.remove()
    topbar?.classList.remove('has-library-controls')
    return
  }
  const controls = existing || createControls(target)
  if (!existing) {
    const settings = topbar.querySelector('[aria-label="Settings"]')
    topbar.insertBefore(controls, settings || null)
  }
  topbar.classList.add('has-library-controls')
  const view = currentView(target)
  const desiredIcon = view === 'grid' ? 'list' : 'grid'
  const desiredLabel = view === 'grid' ? 'Show notes as list' : 'Show notes as grid'
  const viewButton = controls.querySelector('.en-mobile-library-view')
  if (viewButton.dataset.icon !== desiredIcon) {
    viewButton.dataset.icon = desiredIcon
    viewButton.innerHTML = svg(desiredIcon)
  }
  if (viewButton.getAttribute('aria-label') !== desiredLabel) {
    viewButton.setAttribute('aria-label', desiredLabel)
  }
}

export const installMobileLibraryChromeRuntime = (target = globalThis) => {
  if (!target?.document || target[FLAG]) return false
  target[FLAG] = true
  let pending = false
  const schedule = () => {
    if (pending) return
    pending = true
    target.requestAnimationFrame(() => {
      pending = false
      refreshControls(target)
    })
  }
  const observer = new target.MutationObserver(schedule)
  observer.observe(target.document.documentElement, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['class']
  })
  target.addEventListener('resize', schedule)
  schedule()
  target.__ELEPHANTNOTE_MOBILE_LIBRARY_CHROME_DISPOSE__ = () => {
    observer.disconnect()
    target.removeEventListener('resize', schedule)
    target.document.querySelector('.en-mobile-library-controls')?.remove()
    target[FLAG] = false
  }
  return true
}

installMobileLibraryChromeRuntime()
