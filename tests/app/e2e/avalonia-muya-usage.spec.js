const fs = require('node:fs')
const http = require('node:http')
const path = require('node:path')
const { test, expect } = require('playwright/test')

const pagePath = path.resolve(
  __dirname,
  '../../../Elephant/avalonia/src/ElephantNote.Avalonia/Assets/Muya/index.html'
)
const assetRoot = path.dirname(pagePath)

const contentTypes = {
  '.css': 'text/css; charset=utf-8',
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.ttf': 'font/ttf',
  '.wasm': 'application/wasm',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2'
}

const startAssetServer = () => new Promise((resolve, reject) => {
  const server = http.createServer((request, response) => {
    const requestPath = decodeURIComponent(new URL(request.url || '/', 'http://127.0.0.1').pathname)
    const relativePath = requestPath === '/' ? 'index.html' : requestPath.replace(/^\/+/, '')
    const candidate = path.resolve(assetRoot, relativePath)
    if (candidate !== assetRoot && !candidate.startsWith(`${assetRoot}${path.sep}`)) {
      response.writeHead(400)
      response.end('invalid asset path')
      return
    }
    fs.readFile(candidate, (error, contents) => {
      if (error) {
        response.writeHead(error.code === 'ENOENT' ? 404 : 500)
        response.end(error.code === 'ENOENT' ? 'not found' : 'asset read failed')
        return
      }
      response.writeHead(200, {
        'Content-Type': contentTypes[path.extname(candidate)] || 'application/octet-stream',
        'Cache-Control': 'no-store'
      })
      response.end(contents)
    })
  })
  server.once('error', reject)
  server.listen(0, '127.0.0.1', () => {
    const address = server.address()
    resolve({ server, url: `http://127.0.0.1:${address.port}/` })
  })
})

test('Muya native bundle renders, edits, validates with Rust and emits a save request', async({ page }, testInfo) => {
  const messages = []
  const pageErrors = []
  const failedRequests = []
  page.on('pageerror', (error) => pageErrors.push(error.message))
  page.on('requestfailed', (request) => failedRequests.push({
    url: request.url(),
    error: request.failure()?.errorText
  }))
  await page.exposeFunction('receiveNativeWebViewMessage', (body) => {
    const message = JSON.parse(body)
    messages.push(message)
  })
  await page.addInitScript(() => {
    const webview = {
      postMessage: (body) => window.receiveNativeWebViewMessage(body)
    }
    Object.defineProperty(window, 'chrome', {
      configurable: true,
      value: { webview }
    })
  })

  const assetServer = await startAssetServer()
  try {
    await page.goto(assetServer.url)
    await page.locator('html[data-muya-native-ready="true"]').waitFor()
    expect(await page.locator('html').getAttribute('data-muya-engine')).toBe('muya-js+muya-rust')
    expect(await page.locator('html').getAttribute('data-muya-native-bridge')).toBe('chrome.webview')

    await page.evaluate(() => {
      window.__ELEPHANT_MUYA_HOST__.receive({
        type: 'open-document',
        documentId: 'e2e-note',
        content: '# Native Muya\n\nInitial paragraph'
      })
    })
    await page.waitForFunction(
      (expected) => window.__ELEPHANT_MUYA__.getRustSnapshot()?.markdown?.includes(expected),
      'Initial paragraph'
    )
    await expect(page.locator('#muya-host h1')).toHaveText('# Native Muya')

    await page.evaluate(() => window.__ELEPHANT_MUYA__.focus())
    await page.keyboard.press('End')
    await page.keyboard.press('Enter')
    await page.keyboard.type('Edited through the native Muya page')

    const latestContent = () => messages.filter((message) => message.type === 'content-changed').at(-1)?.content || ''
    await expect.poll(latestContent).toContain('Edited through the native Muya page')
    const changed = messages.filter((message) => message.type === 'content-changed').at(-1)
    const visibleMarkdown = await page.evaluate(() => window.__ELEPHANT_MUYA__.getMarkdown())
    expect(changed.documentId).toBe('e2e-note')
    expect(changed.content).toBe(visibleMarkdown)
    expect(changed.engine).toBe('muya-js+muya-rust')
    expect(Number.isInteger(changed.rustRevision)).toBe(true)

    await page.keyboard.press(process.platform === 'darwin' ? 'Meta+S' : 'Control+S')
    await expect.poll(() => messages.find((message) => message.type === 'save-request')?.content || '').toContain('Edited through the native Muya page')

    const rustSnapshot = await page.evaluate(() => window.__ELEPHANT_MUYA__.getRustSnapshot())
    expect(rustSnapshot.document.nodes.length).toBeGreaterThan(0)
    expect(rustSnapshot.markdown).toContain('Edited through the native Muya page')
    expect(rustSnapshot.markdown).toBe(visibleMarkdown)
    expect(rustSnapshot.document.nodes.some((node) => node.kind?.value?.type === 'heading')).toBe(true)
    expect(rustSnapshot.document.nodes.some((node) => node.kind?.value?.type === 'paragraph')).toBe(true)
    expect(pageErrors).toEqual([])
    expect(failedRequests).toEqual([])
    expect(messages.filter((message) => message.type === 'error')).toEqual([])

    const screenshotPath = testInfo.outputPath('avalonia-muya-render.png')
    await page.screenshot({ path: screenshotPath, fullPage: true })
    expect(fs.statSync(screenshotPath).size).toBeGreaterThan(1000)
    await testInfo.attach('avalonia-muya-render', { path: screenshotPath, contentType: 'image/png' })
  } finally {
    await new Promise((resolve) => assetServer.server.close(resolve))
  }
})
