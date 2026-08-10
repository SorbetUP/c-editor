(() => {
  const pending = new Map()
  let nextRequestId = 0

  const postMessage = (message) => {
    const chromeWebView = globalThis.chrome?.webview
    if (typeof chromeWebView?.postMessage === 'function') {
      chromeWebView.postMessage(JSON.stringify(message))
      return
    }

    const handler = globalThis.webkit?.messageHandlers?.avalonia ||
      globalThis.webkit?.messageHandlers?.invokeCSharpAction
    if (typeof handler?.postMessage === 'function') {
      handler.postMessage(JSON.stringify(message))
      return
    }

    throw new Error('Avalonia WebView bridge is unavailable.')
  }

  const invoke = (command, payload = {}) => new Promise((resolve, reject) => {
    const id = `avalonia-${Date.now()}-${nextRequestId++}`
    pending.set(id, { resolve, reject })
    try {
      postMessage({ type: 'invoke', id, command, payload })
    } catch (error) {
      pending.delete(id)
      reject(error)
    }
  })

  globalThis.__avaloniaResolve = (id, ok, value, error) => {
    const request = pending.get(id)
    if (!request) return
    pending.delete(id)
    if (ok) request.resolve(value)
    else {
      const failure = new Error(error?.message || String(error || 'Avalonia command failed.'))
      failure.code = error?.code || 'AVALONIA_BRIDGE_ERROR'
      request.reject(failure)
    }
  }

  const listen = async() => ({
    unlisten: async() => {}
  })

  const core = { invoke }
  globalThis.__TAURI_INTERNALS__ = {
    invoke,
    transformCallback: () => 0,
    unregisterCallback: () => {}
  }
  globalThis.__TAURI__ = {
    core,
    dialog: {
      open: (options = {}) => invoke('plugin:dialog|open', { options })
    },
    event: { listen },
    clipboardManager: {
      writeText: (text) => invoke('plugin:clipboard-manager|write_text', { text }),
      readText: () => invoke('plugin:clipboard-manager|read_text')
    },
    opener: {
      openUrl: (url) => invoke('plugin:opener|open_url', { url }),
      openPath: (path) => invoke('plugin:opener|open_path', { path }),
      revealItemInDir: (path) => invoke('plugin:opener|reveal_item_in_dir', { path })
    },
    fs: {}
  }
})()
