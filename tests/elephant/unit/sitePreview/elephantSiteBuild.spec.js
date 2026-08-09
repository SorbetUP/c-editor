/* @vitest-environment node */

import fs from 'fs-extra'
import os from 'os'
import path from 'path'
import { ElephantSiteManager } from 'main_renderer/elephantnote/sitePreview/ElephantSiteManager'

describe('Elephant site build integration', () => {
  let root

  beforeEach(async() => {
    root = await fs.mkdtemp(path.join(os.tmpdir(), 'en-site-build-'))
  })

  afterEach(async() => {
    await fs.remove(root)
  })

  it('builds a static site from a Markdown folder without modifying notes', async() => {
    const vaultRoot = path.join(root, 'Vault')
    const sourceFolder = path.join(vaultRoot, 'Docs')
    const assetsFolder = path.join(sourceFolder, 'assets')
    await fs.ensureDir(assetsFolder)
    await fs.writeFile(path.join(sourceFolder, 'index.md'), '# Home\n\n[Guide](./guide.md)\n\n![Schema](./assets/schema.png)\n', 'utf8')
    await fs.writeFile(path.join(sourceFolder, 'guide.md'), '# Guide\n', 'utf8')
    await fs.writeFile(path.join(assetsFolder, 'schema.png'), Buffer.from('89504e470d0a1a0a', 'hex'))
    const originalIndex = await fs.readFile(path.join(sourceFolder, 'index.md'), 'utf8')

    const info = await new ElephantSiteManager().buildStaticSite({
      vaultRoot,
      folderPath: sourceFolder
    })

    expect(info.status).to.equal('ready')
    expect(await fs.pathExists(path.join(info.outputDir, 'index.html'))).to.equal(true)
    expect(await fs.pathExists(path.join(info.outputDir, 'guide', 'index.html'))).to.equal(true)
    expect(await fs.pathExists(path.join(info.outputDir, 'assets', 'schema.png'))).to.equal(true)
    const homeHtml = await fs.readFile(path.join(info.outputDir, 'index.html'), 'utf8')
    expect(homeHtml).to.contain('href="guide/"')
    expect(homeHtml).to.contain('src="assets/schema.png"')
    expect(homeHtml).not.to.match(/built with/i)
    expect(await fs.readFile(path.join(sourceFolder, 'index.md'), 'utf8')).to.equal(originalIndex)
  })

  it('creates a preview home page when the folder has notes but no index.md', async() => {
    const vaultRoot = path.join(root, 'Vault')
    const sourceFolder = path.join(vaultRoot, 'Notes')
    await fs.ensureDir(sourceFolder)
    await fs.writeFile(path.join(sourceFolder, 'Untitled.md'), '# Untitled\n\nBody', 'utf8')

    const info = await new ElephantSiteManager().preparePreview({
      vaultRoot,
      folderPath: sourceFolder
    })

    expect(info.status).not.to.equal('error')
    const indexPath = path.join(info.outputDir, 'index.html')
    const indexHtml = await fs.readFile(indexPath, 'utf8')
    expect(await fs.pathExists(indexPath)).to.equal(true)
    expect(indexHtml).to.contain('Untitled')
    expect(indexHtml).not.to.contain('404')
    expect(indexHtml).not.to.match(/built with/i)
  })
})
