import { getCurrentWindow } from '@tauri-apps/api/window'

const MENU_CLASS = 'mt-tauri-menu'
let activeMenuLayer = null

const removeActiveMenu = () => {
  if (!activeMenuLayer) return
  activeMenuLayer.remove()
  activeMenuLayer = null
}

const createMenuLayer = (items) => {
  removeActiveMenu()

  const layer = document.createElement('div')
  layer.className = MENU_CLASS
  layer.style.position = 'fixed'
  layer.style.inset = '0'
  layer.style.zIndex = '2147483647'
  layer.style.background = 'transparent'
  layer.style.pointerEvents = 'auto'

  const menu = document.createElement('div')
  menu.style.position = 'absolute'
  menu.style.minWidth = '220px'
  menu.style.padding = '6px 0'
  menu.style.border = '1px solid rgba(0, 0, 0, 0.16)'
  menu.style.borderRadius = '8px'
  menu.style.background = 'var(--color-bg-1, #fff)'
  menu.style.boxShadow = '0 16px 32px rgba(0, 0, 0, 0.18)'
  menu.style.font = '13px system-ui, sans-serif'
  menu.style.color = 'var(--color-text-1, #111)'
  menu.style.backdropFilter = 'blur(12px)'
  menu.style.pointerEvents = 'auto'

  const close = () => {
    removeActiveMenu()
    document.removeEventListener('keydown', onKeyDown, true)
    layer.removeEventListener('click', onOutsideClick, true)
  }

  const onKeyDown = (event) => {
    if (event.key === 'Escape') close()
  }

  const onOutsideClick = (event) => {
    if (!menu.contains(event.target)) close()
  }

  items.forEach((item) => {
    if (item?.type === 'separator') {
      const separator = document.createElement('div')
      separator.style.height = '1px'
      separator.style.margin = '6px 0'
      separator.style.background = 'rgba(0, 0, 0, 0.08)'
      menu.append(separator)
      return
    }

    const entry = document.createElement('button')
    entry.type = 'button'
    entry.textContent = item?.label || item?.text || item?.title || ''
    entry.disabled = item?.enabled === false
    entry.style.display = 'block'
    entry.style.width = '100%'
    entry.style.border = '0'
    entry.style.padding = '8px 14px'
    entry.style.textAlign = 'left'
    entry.style.background = 'transparent'
    entry.style.color = 'inherit'
    entry.style.cursor = entry.disabled ? 'default' : 'pointer'
    entry.style.opacity = entry.disabled ? '0.45' : '1'

    entry.addEventListener('mouseenter', () => {
      if (!entry.disabled) {
        entry.style.background = 'rgba(0, 0, 0, 0.06)'
      }
    })
    entry.addEventListener('mouseleave', () => {
      entry.style.background = 'transparent'
    })
    entry.addEventListener('click', async(event) => {
      event.preventDefault()
      event.stopPropagation()
      if (entry.disabled) return
      try {
        await item?.click?.(item, getCurrentWindow())
      } finally {
        close()
      }
    })

    menu.append(entry)
  })

  layer.append(menu)
  document.body.append(layer)
  document.addEventListener('keydown', onKeyDown, true)
  layer.addEventListener('click', onOutsideClick, true)

  return { layer, menu, close }
}

const positionMenu = (menuElement, target) => {
  const x = Math.max(8, target?.x ?? target?.clientX ?? 8)
  const y = Math.max(8, target?.y ?? target?.clientY ?? 8)
  menuElement.style.left = `${x}px`
  menuElement.style.top = `${y}px`
}

export class MenuItem {
  constructor(options = {}) {
    Object.assign(this, options)
  }
}

export class Menu {
  static _applicationMenu = null

  constructor() {
    this.items = []
  }

  append(item) {
    this.items.push(item)
  }

  async popup(positionOrOptions = {}, maybeWindow) {
    const normalized = Array.isArray(positionOrOptions)
      ? positionOrOptions[0] || {}
      : positionOrOptions || {}

    if (typeof document === 'undefined') return

    const { menu, close } = createMenuLayer(this.items)
    positionMenu(menu, normalized)

    const viewportWidth = window.innerWidth || document.documentElement.clientWidth || 0
    const viewportHeight = window.innerHeight || document.documentElement.clientHeight || 0
    const rect = menu.getBoundingClientRect()

    if (rect.right > viewportWidth) {
      menu.style.left = `${Math.max(8, viewportWidth - rect.width - 8)}px`
    }
    if (rect.bottom > viewportHeight) {
      menu.style.top = `${Math.max(8, viewportHeight - rect.height - 8)}px`
    }

    activeMenuLayer = menu.parentElement

    return { close, window: maybeWindow || getCurrentWindow() }
  }

  static getApplicationMenu() {
    if (!Menu._applicationMenu) {
      Menu._applicationMenu = new Menu()
    }
    return Menu._applicationMenu
  }

  static setApplicationMenu(menu) {
    Menu._applicationMenu = menu
  }
}

export { getCurrentWindow }

export const clipboard = {
  has: () => false,
  read: () => '',
  writeText: async(text) => navigator.clipboard?.writeText?.(text),
  readText: async() => navigator.clipboard?.readText?.()
}
