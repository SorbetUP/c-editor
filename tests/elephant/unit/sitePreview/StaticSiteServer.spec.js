/* @vitest-environment node */

import fs from 'fs-extra'
import http from 'node:http'
import net from 'node:net'
import os from 'os'
import path from 'path'
import { StaticSiteServer } from 'main_renderer/elephantnote/sitePreview/StaticSiteServer'

const get = (url) => new Promise((resolve, reject) => {
  http.get(url, (res) => {
    let body = ''
    res.setEncoding('utf8')
    res.on('data', chunk => {
      body += chunk
    })
    res.on('end', () => resolve({ statusCode: res.statusCode, body }))
  }).on('error', reject)
})

const getRawPath = (port, rawPath) => new Promise((resolve, reject) => {
  const socket = net.createConnection({ host: '127.0.0.1', port }, () => {
    socket.write(`GET ${rawPath} HTTP/1.1\r\nHost: 127.0.0.1:${port}\r\nConnection: close\r\n\r\n`)
  })
  let response = ''
  socket.setEncoding('utf8')
  socket.on('data', chunk => {
    response += chunk
  })
  socket.on('error', reject)
  socket.on('end', () => {
    const statusCode = Number(response.match(/^HTTP\/1\.1\s+(\d+)/)?.[1] || 0)
    resolve({ statusCode, response })
  })
})

describe('StaticSiteServer', () => {
  let root
  let server

  beforeEach(async() => {
    root = await fs.mkdtemp(path.join(os.tmpdir(), 'en-site-server-'))
    await fs.writeFile(path.join(root, 'index.html'), '<h1>Home</h1>', 'utf8')
    await fs.ensureDir(path.join(root, 'assets'))
    await fs.writeFile(path.join(root, 'assets', 'app.js'), 'window.ok = true', 'utf8')
    await fs.writeFile(path.join(os.tmpdir(), 'en-outside.txt'), 'outside', 'utf8')
    server = new StaticSiteServer()
  })

  afterEach(async() => {
    await server.stop()
    await fs.remove(root)
    await fs.remove(path.join(os.tmpdir(), 'en-outside.txt'))
  })

  it('serves index and assets from 127.0.0.1', async() => {
    const { url, port } = await server.start(root)
    expect(url).to.equal(`http://127.0.0.1:${port}/`)
    expect((await get(url)).body).to.contain('Home')
    expect((await get(`${url}assets/app.js`)).body).to.contain('window.ok')
  })

  it('blocks path traversal outside outputDir', async() => {
    const { port } = await server.start(root)
    const response = await getRawPath(port, '/%2e%2e/en-outside.txt')
    expect(response.statusCode).to.equal(403)
  })

  it('serves a fallback home page when the build has pages but no root index', async() => {
    await fs.remove(path.join(root, 'index.html'))
    await fs.ensureDir(path.join(root, 'untitled'))
    await fs.writeFile(path.join(root, 'untitled', 'index.html'), '<h1>Untitled</h1>', 'utf8')
    await fs.writeFile(path.join(root, '404.html'), '<h1>404</h1>', 'utf8')

    const { url } = await server.start(root)
    const response = await get(url)

    expect(response.statusCode).to.equal(200)
    expect(response.body).to.contain('Preview home')
    expect(response.body).to.contain('/untitled/index.html')
    expect(response.body).not.to.contain('/404.html')
  })
})
