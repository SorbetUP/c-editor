import fs from 'fs-extra'
import http from 'node:http'
import path from 'path'

const MIME_TYPES = Object.freeze({
  '.html': 'text/html; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.js': 'application/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.webp': 'image/webp',
  '.gif': 'image/gif',
  '.svg': 'image/svg+xml; charset=utf-8',
  '.txt': 'text/plain; charset=utf-8'
})

const isInside = (root, target) => {
  const relative = path.relative(root, target)
  return relative === '' || (!relative.startsWith('..') && !path.isAbsolute(relative))
}

const escapeHtml = (value) => String(value || '')
  .replace(/&/g, '&amp;')
  .replace(/</g, '&lt;')
  .replace(/>/g, '&gt;')
  .replace(/"/g, '&quot;')
  .replace(/'/g, '&#39;')

const collectHtmlPages = async(outputDir) => {
  const pages = []

  const walk = async(directory) => {
    const entries = await fs.readdir(directory, { withFileTypes: true }).catch(() => [])
    for (const entry of entries) {
      const fullPath = path.join(directory, entry.name)
      if (!isInside(outputDir, fullPath)) continue
      if (entry.isDirectory()) {
        await walk(fullPath)
      } else if (entry.isFile() && entry.name.toLowerCase().endsWith('.html')) {
        const relativePath = path.relative(outputDir, fullPath).split(path.sep).join('/')
        if (relativePath !== 'index.html' && relativePath !== '404.html') pages.push(relativePath)
      }
    }
  }

  await walk(outputDir)
  return pages.sort((a, b) => a.localeCompare(b))
}

const renderFallbackIndex = async(outputDir) => {
  const pages = await collectHtmlPages(outputDir)
  const title = path.basename(path.dirname(outputDir)) || 'ElephantNote preview'
  const pageItems = pages.length
    ? pages.map((page) => {
      const label = page.replace(/\/index\.html$/i, '').replace(/\.html$/i, '') || page
      return `<li><a href="/${escapeHtml(page)}">${escapeHtml(label)}</a></li>`
    }).join('\n')
    : '<li><span>No generated pages were found. Add markdown notes to this folder and rebuild the preview.</span></li>'

  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${escapeHtml(title)}</title>
  <style>
    body { margin: 0; font: 16px/1.5 system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; color: #111827; background: #f8fafc; }
    main { max-width: 880px; margin: 0 auto; padding: 48px 24px; }
    h1 { margin: 0 0 12px; font-size: 34px; line-height: 1.1; }
    p { margin: 0 0 24px; color: #6b7280; }
    ul { display: grid; gap: 10px; list-style: none; margin: 0; padding: 0; }
    a, span { display: block; border: 1px solid #d1d5db; border-radius: 8px; padding: 14px 16px; background: #fff; }
    a { color: #1d4ed8; text-decoration: none; }
    a:hover { border-color: #93c5fd; background: #eff6ff; }
    span { color: #6b7280; }
  </style>
</head>
<body>
  <main>
    <h1>${escapeHtml(title)}</h1>
    <p>Preview home</p>
    <ul>
${pageItems}
    </ul>
  </main>
</body>
</html>`
}

export class StaticSiteServer {
  constructor() {
    this.server = null
    this.outputDir = ''
    this.port = null
  }

  async start(outputDir) {
    await this.stop()
    this.outputDir = path.resolve(outputDir)

    this.server = http.createServer((req, res) => {
      this.handleRequest(req, res).catch(() => {
        if (!res.headersSent) {
          res.writeHead(500, { 'content-type': 'text/plain; charset=utf-8' })
        }
        res.end('The local preview server could not serve this file.')
      })
    })

    await new Promise((resolve, reject) => {
      this.server.once('error', reject)
      this.server.listen(0, '127.0.0.1', () => {
        this.server.off('error', reject)
        this.port = this.server.address().port
        resolve()
      })
    })

    return {
      port: this.port,
      url: `http://127.0.0.1:${this.port}/`
    }
  }

  async stop() {
    if (!this.server) return
    const server = this.server
    this.server = null
    this.port = null
    await new Promise((resolve, reject) => {
      server.close((err) => err ? reject(err) : resolve())
    })
  }

  async handleRequest(req, res) {
    const rawPath = String(req.url || '/').split(/[?#]/)[0]
    let decodedPath = '/'
    try {
      decodedPath = decodeURIComponent(rawPath)
    } catch {
      res.writeHead(400, { 'content-type': 'text/plain; charset=utf-8' })
      res.end('Bad request')
      return
    }
    const requestPath = decodedPath === '/' ? 'index.html' : decodedPath.replace(/^\/+/, '')
    let filePath = path.resolve(this.outputDir, requestPath)

    if (!isInside(this.outputDir, filePath)) {
      res.writeHead(403, { 'content-type': 'text/plain; charset=utf-8' })
      res.end('Forbidden')
      return
    }

    const stats = await fs.stat(filePath).catch(() => null)
    if (stats?.isDirectory()) {
      filePath = path.join(filePath, 'index.html')
    } else if (!stats && !path.extname(filePath)) {
      filePath = path.join(filePath, 'index.html')
    }

    if (!isInside(this.outputDir, filePath) || !(await fs.pathExists(filePath))) {
      const fallback = path.join(this.outputDir, 'index.html')
      if (isInside(this.outputDir, fallback) && await fs.pathExists(fallback)) {
        filePath = fallback
      } else if (requestPath === 'index.html') {
        res.writeHead(200, {
          'content-type': MIME_TYPES['.html'],
          'x-content-type-options': 'nosniff'
        })
        res.end(await renderFallbackIndex(this.outputDir))
        return
      } else {
        res.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' })
        res.end('Not found')
        return
      }
    }

    const contentType = MIME_TYPES[path.extname(filePath).toLowerCase()] || 'application/octet-stream'
    res.writeHead(200, {
      'content-type': contentType,
      'x-content-type-options': 'nosniff'
    })
    fs.createReadStream(filePath).pipe(res)
  }
}
