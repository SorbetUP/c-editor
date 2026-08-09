'use strict'

const fs = require('fs')
const http = require('http')
const path = require('path')
const { app, BrowserWindow } = require('electron')

const projectRoot = path.resolve(__dirname, '../../..')
const rendererRoot = path.join(projectRoot, 'build', 'out', 'renderer')
const preload = path.join(__dirname, 'tauri-preload-entry.js')

const mimeType = (pathname) => {
  switch (path.extname(pathname).toLowerCase()) {
    case '.html': return 'text/html; charset=utf-8'
    case '.js': return 'text/javascript; charset=utf-8'
    case '.css': return 'text/css; charset=utf-8'
    case '.json': return 'application/json; charset=utf-8'
    case '.svg': return 'image/svg+xml'
    case '.png': return 'image/png'
    case '.jpg':
    case '.jpeg': return 'image/jpeg'
    case '.webp': return 'image/webp'
    case '.woff': return 'font/woff'
    case '.woff2': return 'font/woff2'
    case '.wasm': return 'application/wasm'
    default: return 'application/octet-stream'
  }
}

const resolveRequest = (requestUrl = '/') => {
  const url = new URL(requestUrl, 'http://127.0.0.1')
  const relative = decodeURIComponent(url.pathname).replace(/^\/+/, '') || 'index.html'
  const candidate = path.resolve(rendererRoot, relative)
  if (!candidate.startsWith(`${rendererRoot}${path.sep}`) && candidate !== rendererRoot) return null
  return candidate
}

const startRendererServer = () => new Promise((resolve, reject) => {
  if (!fs.existsSync(path.join(rendererRoot, 'index.html'))) {
    reject(new Error(`Built Tauri renderer is missing: ${rendererRoot}`))
    return
  }

  const server = http.createServer((request, response) => {
    const pathname = resolveRequest(request.url)
    if (!pathname || !fs.existsSync(pathname) || !fs.statSync(pathname).isFile()) {
      response.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' })
      response.end('Not found')
      return
    }
    response.writeHead(200, {
      'Content-Type': mimeType(pathname),
      'Cache-Control': 'no-store',
      'Cross-Origin-Opener-Policy': 'same-origin'
    })
    fs.createReadStream(pathname).pipe(response)
  })
  server.once('error', reject)
  server.listen(0, '127.0.0.1', () => resolve(server))
})

let rendererServer = null

const createWindow = async () => {
  rendererServer = await startRendererServer()
  const address = rendererServer.address()
  const window = new BrowserWindow({
    width: 1280,
    height: 720,
    show: true,
    backgroundColor: '#ffffff',
    webPreferences: {
      preload,
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
      webSecurity: true,
      additionalArguments: process.argv.slice(2).map((argument) => `--elephant-e2e-arg=${encodeURIComponent(argument)}`)
    }
  })
  await window.loadURL(`http://127.0.0.1:${address.port}/index.html`)
  return window
}

app.commandLine.appendSwitch('disable-gpu')
app.commandLine.appendSwitch('disable-dev-shm-usage')
if (process.env.ELEPHANT_ACCEPTANCE_CDP_PORT) {
  app.commandLine.appendSwitch('remote-debugging-port', String(process.env.ELEPHANT_ACCEPTANCE_CDP_PORT))
  console.info(`[acceptance-runner] CDP listening on ${process.env.ELEPHANT_ACCEPTANCE_CDP_PORT}`)
}
app.whenReady().then(createWindow).catch((error) => {
  console.error('[e2e-shell] failed to start', error)
  app.exit(1)
})

app.on('window-all-closed', () => app.quit())
app.on('before-quit', () => {
  rendererServer?.close()
  rendererServer = null
})
