import path from 'path'
import os from 'os'
import {
  assertPathInsideVault,
  isIgnoredPath,
  isMarkdownFile
} from 'main_renderer/elephantnote/search/pathSafety'

describe('pathSafety', () => {
  it('accepts a path inside the vault', () => {
    const root = path.join(os.tmpdir(), 'elephantnote-vault')
    const target = path.join(root, 'Notes', 'test.md')
    expect(() => assertPathInsideVault(root, target)).not.to.throw()
  })

  it('rejects a path outside the vault', () => {
    const root = path.join(os.tmpdir(), 'elephantnote-vault')
    const target = path.join(os.tmpdir(), 'outside.md')
    expect(() => assertPathInsideVault(root, target)).to.throw('Path must stay inside the active vault.')
  })

  it('rejects path traversal', () => {
    const root = path.join(os.tmpdir(), 'elephantnote-vault')
    const target = path.join(root, '..', 'outside.md')
    expect(() => assertPathInsideVault(root, target)).to.throw()
  })

  it('ignores internal folders', () => {
    expect(isIgnoredPath('.elephantnote/search/index.json')).to.equal(true)
    expect(isIgnoredPath('.git/config')).to.equal(true)
    expect(isIgnoredPath('node_modules/package/index.md')).to.equal(true)
  })

  it('accepts markdown files and ignores other extensions', async() => {
    expect(isMarkdownFile('note.md')).to.equal(true)
    expect(isMarkdownFile('note.markdown')).to.equal(true)
    expect(isMarkdownFile('note.pdf')).to.equal(false)
    expect(isMarkdownFile('note.txt')).to.equal(false)
  })
})
