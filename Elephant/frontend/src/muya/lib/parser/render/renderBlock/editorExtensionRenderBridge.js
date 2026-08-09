const EDITOR_EXTENSION_AREA = 'editor.extensions'

const getEditorExtensions = () => {
  const manager = globalThis?.__ELEPHANT_ADDONS__
  if (typeof manager?.getContributions !== 'function') return []
  return manager.getContributions(EDITOR_EXTENSION_AREA)
    .map((entry) => entry?.contribution)
    .filter(Boolean)
}

export const applyEditorContainerExtensions = ({ parent, block, children, h, t }) => {
  let nextChildren = Array.isArray(children) ? children : []

  for (const extension of getEditorExtensions()) {
    if (typeof extension.decorateContainer !== 'function') continue
    try {
      const result = extension.decorateContainer({ parent, block, children: nextChildren, h, t })
      if (Array.isArray(result)) nextChildren = result
    } catch (error) {
      console.error('[editor-extension] container decoration failed', {
        id: extension.id || '',
        blockKey: block?.key || '',
        error
      })
    }
  }

  return nextChildren
}

const normalizeToolbarItem = (extension, item, index) => {
  if (!item || typeof item !== 'object') return null
  const itemId = String(item.id || `${index}`).trim()
  if (!itemId || typeof item.run !== 'function') return null
  const extensionId = String(extension.id || 'anonymous').trim()
  return {
    ...item,
    type: `addon-${extensionId}-${itemId}`.replace(/[^a-z0-9_-]/gi, '-'),
    addonExtensionId: extensionId,
    addonToolbarItem: true
  }
}

export const getEditorImageToolbarItems = ({ imageInfo, muya, isLocalImage }) => {
  const result = []

  for (const extension of getEditorExtensions()) {
    const items = Array.isArray(extension.imageToolbarItems) ? extension.imageToolbarItems : []
    items.forEach((rawItem, index) => {
      const item = normalizeToolbarItem(extension, rawItem, index)
      if (!item || (item.localOnly && !isLocalImage)) return
      try {
        if (typeof item.when === 'function' && item.when({ imageInfo, muya, isLocalImage }) !== true) return
        result.push(item)
      } catch (error) {
        console.error('[editor-extension] image toolbar predicate failed', {
          id: extension.id || '',
          itemId: item.id || '',
          error
        })
      }
    })
  }

  return result
}

export const runEditorImageToolbarItem = (item, context) => {
  if (!item?.addonToolbarItem || typeof item.run !== 'function') return false
  try {
    item.run(context)
    return true
  } catch (error) {
    console.error('[editor-extension] image toolbar action failed', {
      id: item.addonExtensionId || '',
      itemId: item.id || '',
      error
    })
    return false
  }
}
