const { createReadStream, existsSync, statSync } = require('node:fs')
const { createServer } = require('node:http')
const { extname, join, normalize, resolve } = require('node:path')
const { test, expect } = require('playwright/test')

const repositoryRoot = resolve(process.cwd())
const tauriRenderer = resolve(repositoryRoot, 'build/out/renderer')
const avaloniaRenderer = resolve(
  repositoryRoot,
  'Elephant/avalonia/src/ElephantNote.Avalonia/Assets/Renderer'
)

const contentTypes = {
  '.css': 'text/css; charset=utf-8',
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.png': 'image/png',
  '.svg': 'image/svg+xml',
  '.wasm': 'application/wasm',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2'
}

const mockBridge = ({ populated = false } = {}) => {
  const entries = populated
    ? [{ path: 'Acceptance/Parity.md', title: 'Parity note', preview: 'Initial body.', excerpt: 'Initial body.', type: 'note', kind: 'note', isDirectory: false, updatedAt: '2026-01-01T00:00:00.000Z' }]
    : []
  const valueFor = (command) => command === 'tauri_vaults_get'
    ? {
      vaults: populated ? [{ id: 'parity', name: 'Parity vault', path: '/tmp/parity-vault', icon: '' }] : [],
      activeVaultId: populated ? 'parity' : null,
      activeVault: populated ? { id: 'parity', name: 'Parity vault', path: '/tmp/parity-vault', icon: '' } : null,
      workspace: populated ? { version: 1, vaultName: 'Parity vault', sidebar: [] } : null,
      entries
    }
    : command === 'tauri_directory_list'
      ? entries
      : command === 'tauri_notes_read'
        ? { path: 'Acceptance/Parity.md', content: '# Parity note\n\nInitial body.\n' }
    : command === 'tauri_platform_info'
      ? { os: 'macos', family: 'unix', arch: 'arm64', desktop: true, mobile: false, macos: true }
      : command === 'tauri_features_get'
        ? { features: [] }
        : command === 'tauri_search_status'
          ? { ready: true, indexed: false }
          : command === 'plugin:path|resolve_directory'
            ? '/tmp/elephant-renderer-parity'
            : command === 'plugin:event|listen'
              ? 0
              : command === 'tauri_acceptance_enabled'
          ? false
                : command === 'tauri_prefs_get' || command === 'tauri_prefs_all' || command === 'tauri_prefs_set' || command === 'tauri_prefs_set_many'
                  ? {}
                : command.startsWith('plugin:') || command.endsWith('_enabled')
                  ? true
                  : []
  const invoke = (command) => Promise.resolve(valueFor(command))
  window.__TAURI_INTERNALS__ = { invoke, transformCallback: () => 0, unregisterCallback: () => {} }
  window.__TAURI__ = {
    core: { invoke },
    dialog: { open: () => invoke('plugin:dialog|open') },
    event: { listen: async() => ({ unlisten: async() => {} }) },
    clipboardManager: { writeText: () => invoke('plugin:clipboard-manager|write_text'), readText: () => invoke('plugin:clipboard-manager|read_text') },
    opener: { openUrl: () => invoke('plugin:opener|open_url'), openPath: () => invoke('plugin:opener|open_path'), revealItemInDir: () => invoke('plugin:opener|reveal_item_in_dir') },
    fs: {}
  }
  window.chrome = {
    webview: {
      postMessage(raw) {
        const request = JSON.parse(raw)
        if (request.type !== 'invoke') return
        const command = request.command
        const value = valueFor(command)
        queueMicrotask(() => window.__avaloniaResolve(request.id, true, value, null))
      }
    }
  }
}

const serve = async() => {
  const server = createServer((request, response) => {
    const url = new URL(request.url || '/', 'http://127.0.0.1')
    const root = url.pathname.startsWith('/avalonia/') ? avaloniaRenderer : tauriRenderer
    const relative = decodeURIComponent(url.pathname.replace(/^\/(?:avalonia|tauri)\//, '') || 'index.html')
    const file = resolve(root, relative)
    const normalizedRoot = normalize(`${root}/`)
    if (!normalize(file).startsWith(normalizedRoot) || !existsSync(file) || !statSync(file).isFile()) {
      response.writeHead(404)
      response.end('not found')
      return
    }
    response.setHeader('Content-Type', contentTypes[extname(file)] || 'application/octet-stream')
    createReadStream(file).pipe(response)
  })
  await new Promise((resolveServer) => server.listen(0, '127.0.0.1', resolveServer))
  const address = server.address()
  return { server, origin: `http://127.0.0.1:${address.port}` }
}

test('Avalonia and Tauri render the same shared shell asset', async({ browser }) => {
  test.skip(!existsSync(join(tauriRenderer, 'index.html')) || !existsSync(join(avaloniaRenderer, 'index.html')), 'renderer assets are not built')
  const server = await serve()
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 840 }, deviceScaleFactor: 1 })
    const screenshots = []
    for (const runtime of ['tauri', 'avalonia']) {
      const page = await context.newPage()
      await page.addInitScript(mockBridge)
      const errors = []
      page.on('pageerror', (error) => errors.push(error.message))
      await page.goto(`${server.origin}/${runtime}/index.html`)
      await expect(page.locator('.en-empty')).toBeVisible({ timeout: 15000 })
      await expect(page.locator('h1')).toHaveText('Choose your first vault')
      screenshots.push(await page.screenshot())
      expect(errors, `${runtime} renderer errors`).toEqual([])
      await page.close()
    }
    expect(screenshots[1].equals(screenshots[0])).toBe(true)
    await context.close()
  } finally {
    await new Promise((resolveServer) => server.server.close(resolveServer))
  }
})

test('Avalonia and Tauri render the same populated library state', async({ browser }) => {
  test.skip(!existsSync(join(tauriRenderer, 'index.html')) || !existsSync(join(avaloniaRenderer, 'index.html')), 'renderer assets are not built')
  const server = await serve()
  try {
    const context = await browser.newContext({ viewport: { width: 1280, height: 840 }, deviceScaleFactor: 1 })
    const screenshots = []
    for (const runtime of ['tauri', 'avalonia']) {
      const page = await context.newPage()
      await page.addInitScript(mockBridge, { populated: true })
      await page.goto(`${server.origin}/${runtime}/index.html`)
      await expect(page.locator('.en-library-grid')).toBeVisible({ timeout: 15000 })
      await expect(page.locator('.en-note-card').first()).toContainText('Parity note')
      screenshots.push(await page.screenshot())
      await page.close()
    }
    expect(screenshots[1].equals(screenshots[0])).toBe(true)
    await context.close()
  } finally {
    await new Promise((resolveServer) => server.server.close(resolveServer))
  }
})
