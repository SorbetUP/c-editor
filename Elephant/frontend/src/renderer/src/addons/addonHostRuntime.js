const requireName = (value, label = 'resource name') => {
  const name = typeof value === 'string' ? value.trim() : ''
  if (!name) throw new TypeError(`${label} must be a non-empty string`)
  return name
}

const requireObject = (value, label) => {
  if ((typeof value !== 'object' && typeof value !== 'function') || value === null) {
    throw new TypeError(`${label} must be an object or function`)
  }
  return value
}

const requireFunction = (value, label) => {
  if (typeof value !== 'function') throw new TypeError(`${label} must be a function`)
  return value
}

const callSafely = (listener, payload, logger) => {
  try {
    listener(payload)
  } catch (error) {
    logger?.warn?.('[addons:host] listener failed', {
      error: error?.message || String(error)
    })
  }
}

export const createAddonHostRuntime = (options = {}, target = globalThis) => {
  const resources = new Map()
  const listeners = new Map()
  const hooks = new Map()
  const logger = options.logger

  const notify = (name, value, previous) => {
    const payload = Object.freeze({ name, value, previous })
    for (const listener of listeners.get(name) || []) callSafely(listener, payload, logger)
    for (const listener of listeners.get('*') || []) callSafely(listener, payload, logger)
  }

  const provide = (name, value) => {
    const normalized = requireName(name)
    const previous = resources.get(normalized)
    resources.set(normalized, value)
    notify(normalized, value, previous)
    let active = true
    return () => {
      if (!active || resources.get(normalized) !== value) return
      active = false
      resources.delete(normalized)
      notify(normalized, undefined, value)
    }
  }

  const get = (name) => {
    const normalized = requireName(name)
    if (resources.has(normalized)) return resources.get(normalized)
    return target?.[normalized]
  }

  const watch = (name, listener, options = {}) => {
    const normalized = requireName(name)
    requireFunction(listener, 'resource listener')
    if (!listeners.has(normalized)) listeners.set(normalized, new Set())
    listeners.get(normalized).add(listener)
    if (options.immediate !== false) {
      callSafely(listener, Object.freeze({ name: normalized, value: get(normalized), previous: undefined }), logger)
    }
    return () => listeners.get(normalized)?.delete(listener)
  }

  const patchMethod = (object, methodName, wrapper) => {
    const targetObject = requireObject(object, 'patch target')
    const name = requireName(methodName, 'method name')
    const wrap = requireFunction(wrapper, 'method wrapper')
    const original = targetObject[name]
    requireFunction(original, `${name}`)

    const patched = function(...args) {
      const callOriginal = (...nextArgs) => original.apply(this, nextArgs.length ? nextArgs : args)
      return wrap.call(this, callOriginal, ...args)
    }
    Object.defineProperty(patched, 'name', { value: `${name}WithAddonPatch`, configurable: true })
    targetObject[name] = patched

    let active = true
    return () => {
      if (!active) return
      active = false
      if (targetObject[name] === patched) targetObject[name] = original
    }
  }

  const patchProperty = (object, propertyName, value) => {
    const targetObject = requireObject(object, 'patch target')
    const name = requireName(propertyName, 'property name')
    const previousDescriptor = Object.getOwnPropertyDescriptor(targetObject, name)
    const nextDescriptor = value && typeof value === 'object' && (
      Object.prototype.hasOwnProperty.call(value, 'get') ||
      Object.prototype.hasOwnProperty.call(value, 'set') ||
      Object.prototype.hasOwnProperty.call(value, 'value')
    )
      ? { configurable: true, enumerable: previousDescriptor?.enumerable ?? true, ...value }
      : { configurable: true, enumerable: previousDescriptor?.enumerable ?? true, writable: true, value }

    Object.defineProperty(targetObject, name, nextDescriptor)
    let active = true
    return () => {
      if (!active) return
      active = false
      if (previousDescriptor) Object.defineProperty(targetObject, name, previousDescriptor)
      else delete targetObject[name]
    }
  }

  const registerHook = (name, handler) => {
    const normalized = requireName(name, 'hook name')
    requireFunction(handler, 'hook handler')
    if (!hooks.has(normalized)) hooks.set(normalized, new Set())
    hooks.get(normalized).add(handler)
    return () => hooks.get(normalized)?.delete(handler)
  }

  const runHook = async (name, payload) => {
    const normalized = requireName(name, 'hook name')
    let current = payload
    for (const handler of hooks.get(normalized) || []) {
      const result = await handler(current)
      if (result !== undefined) current = result
    }
    return current
  }

  const mount = (selectorOrElement, renderer) => {
    const documentRef = target?.document
    if (!documentRef) throw new Error('Document is unavailable')
    const host = typeof selectorOrElement === 'string'
      ? documentRef.querySelector(selectorOrElement)
      : selectorOrElement
    if (!host?.appendChild) throw new Error(`Addon mount target was not found: ${String(selectorOrElement)}`)
    requireFunction(renderer, 'mount renderer')

    const result = renderer(host)
    const nodes = []
    if (result?.nodeType) {
      host.appendChild(result)
      nodes.push(result)
    } else if (Array.isArray(result)) {
      for (const node of result) {
        if (!node?.nodeType) continue
        host.appendChild(node)
        nodes.push(node)
      }
    }

    let active = true
    return () => {
      if (!active) return
      active = false
      if (typeof result === 'function') result()
      else if (typeof result?.dispose === 'function') result.dispose()
      for (const node of nodes) node.remove?.()
    }
  }

  const host = Object.freeze({
    get,
    has: (name) => resources.has(requireName(name)),
    list: () => [...resources.keys()].sort(),
    provide,
    watch,
    patchMethod,
    patchProperty,
    registerHook,
    runHook,
    mount
  })

  const initialResources = {
    window: target,
    document: target?.document,
    tauri: target?.__TAURI__,
    marktext: target?.marktext,
    elephantnote: target?.elephantnote,
    fileUtils: target?.fileUtils,
    router: options.router,
    pinia: options.pinia,
    services: options.services,
    vueApp: options.vueApp,
    runtime: options.runtime
  }
  for (const [name, value] of Object.entries(initialResources)) {
    if (value !== undefined && value !== null) provide(name, value)
  }

  return host
}
