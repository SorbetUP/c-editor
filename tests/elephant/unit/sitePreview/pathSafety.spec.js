/* @vitest-environment node */

import fs from 'fs-extra'
import os from 'os'
import path from 'path'
import {
  assertPathInsideVault,
  isAllowedSiteFile,
  isIgnoredForSite,
  isValidSiteSourceFolder
} from 'main_renderer/elephantnote/sitePreview/pathSafety'

describe('sitePreview pathSafety', () => {
  let vaultRoot

  beforeEach(async() => {
    vaultRoot = await fs.mkdtemp(path.join(os.tmpdir(), 'en-site-vault-'))
    await fs.ensureDir(path.join(vaultRoot, 'Docs'))
    await fs.writeFile(path.join(vaultRoot, 'Docs', 'index.md'), '# Docs\n', 'utf8')
  })

  afterEach(async() => {
    await fs.remove(vaultRoot)
  })

  it('accepts a normal folder inside the vault', async() => {
    await expect(isValidSiteSourceFolder(vaultRoot, path.join(vaultRoot, 'Docs'))).resolves.to.equal(true)
  })

  it('rejects a folder outside the vault', () => {
    const outside = path.join(os.tmpdir(), 'outside')
    expect(() => assertPathInsideVault(vaultRoot, outside)).to.throw('outside the active vault')
  })

  it('rejects path traversal', () => {
    expect(() => assertPathInsideVault(vaultRoot, path.join(vaultRoot, '..', 'outside'))).to.throw()
  })

  it('rejects ignored site folders', async() => {
    for (const ignored of ['.elephantnote', '.git', 'node_modules']) {
      const folder = path.join(vaultRoot, ignored)
      await fs.ensureDir(folder)
      await fs.writeFile(path.join(folder, 'index.md'), '# Hidden\n', 'utf8')
      await expect(isValidSiteSourceFolder(vaultRoot, folder)).resolves.to.equal(false)
    }
  })

  it('detects ignored paths and allowed file extensions', () => {
    expect(isIgnoredForSite('.elephantnote/site-previews/x')).to.equal(true)
    expect(isIgnoredForSite('Notes/.git/config')).to.equal(true)
    expect(isIgnoredForSite('Notes/node_modules/pkg/index.md')).to.equal(true)
    expect(isAllowedSiteFile('image.webp')).to.equal(true)
    expect(isAllowedSiteFile('script.sh')).to.equal(false)
  })
})
