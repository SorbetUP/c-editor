import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import {
  getNoteCardExcerpt,
  getNoteCardTitle,
  getNoteCardTypeLabel
} from '@/elephantnote/utils/noteCardView'
import { ensureRendererPathFacade } from '@/platform/rendererPathFacade'

const makePath = () => ensureRendererPathFacade({})

describe('note card view contracts', () => {
  it.each([
    [{ title: '  Explicit title  ', name: 'fallback.md' }, 'Explicit title'],
    [{ name: 'alpha.md' }, 'alpha'],
    [{ filename: 'BETA.MD' }, 'BETA'],
    [{ name: 'plain-name' }, 'plain-name'],
    [{ title: '   ', name: 'fallback.md' }, 'fallback'],
    [{}, 'Untitled'],
    [null, 'Untitled']
  ])('derives a stable title from %#', (entry, expected) => {
    expect(getNoteCardTitle(entry)).toBe(expected)
  })

  it.each([
    [{ type: '  Meeting  ' }, 'Meeting'],
    [{ type: '' }, 'Note'],
    [{}, 'Note'],
    [null, 'Note']
  ])('derives a stable type label from %#', (entry, expected) => {
    expect(getNoteCardTypeLabel(entry)).toBe(expected)
  })

  it.each([
    [{ excerpt: '# Title\nUseful preview' }, 'Useful preview'],
    [{ markdown: '## Heading only' }, 'Heading only'],
    [{ content: 'Plain content' }, 'Plain content'],
    [{ excerpt: '' }, 'No preview yet.'],
    [{}, 'No preview yet.'],
    [null, 'No preview yet.']
  ])('selects and cleans preview content from %#', (entry, expected) => {
    expect(getNoteCardExcerpt(entry)).toBe(expected)
  })

  it('removes multiline LF frontmatter before the document title', () => {
    const markdown = '---\ntitle: Example\ntags: [one, two]\n---\n# Example\nBody'
    expect(getNoteCardExcerpt({ markdown })).toBe('Body')
  })

  it('removes multiline CRLF frontmatter without leaking carriage returns', () => {
    const markdown = '---\r\ntitle: Example\r\nupdated: 2026-07-10\r\n---\r\n# Example\r\nBody'
    expect(getNoteCardExcerpt({ markdown })).toBe('Body')
  })

  it('removes closed inline frontmatter', () => {
    expect(getNoteCardExcerpt({ excerpt: '--- title: Example tags: [one, two] --- Preview' })).toBe('Preview')
  })

  it('removes open inline frontmatter containing known metadata pairs', () => {
    expect(getNoteCardExcerpt({
      excerpt: '--- title: "A title" tags: [one, two] updated: 2026-07-10 Actual preview'
    })).toBe('Actual preview')
  })

  it('does not treat unknown inline keys as Elephant frontmatter', () => {
    const value = '--- custom: value Actual preview'
    expect(getNoteCardExcerpt({ excerpt: value })).toBe(value)
  })

  it('handles a frontmatter-only note as an empty preview', () => {
    expect(getNoteCardExcerpt({ markdown: '---\ntitle: Example\n---' })).toBe('No preview yet.')
  })

  it('handles a two-megabyte note without changing its result', () => {
    const body = `First line\n${'x'.repeat(2_000_000)}`
    const markdown = `---\ntitle: Large\n---\n# Large\n${body}`
    expect(getNoteCardExcerpt({ markdown })).toBe(body)
  })

  it('keeps the note-card parser on bounded-prefix algorithms', () => {
    const source = fs.readFileSync(path.resolve(process.cwd(), 'Elephant/frontend/app/utils/noteCardView.js'), 'utf8')
    expect(source).toContain('FRONTMATTER_BLOCK_PATTERN')
    expect(source).not.toContain("raw.split(/\\r?\\n/)")
  })
})

describe('renderer path facade contracts', () => {
  it('installs every required path operation', () => {
    const pathFacade = makePath()
    for (const operation of ['normalize', 'join', 'resolve', 'basename', 'dirname', 'isAbsolute', 'relative']) {
      expect(pathFacade[operation]).toBeTypeOf('function')
    }
  })

  it('installs on target.window when a window object is supplied', () => {
    const target = { window: {} }
    expect(ensureRendererPathFacade(target)).toBe(target.window.path)
  })

  it('preserves existing host implementations', () => {
    const normalize = () => 'host-normalized'
    const target = { path: { normalize } }
    const installed = ensureRendererPathFacade(target)
    expect(installed.normalize).toBe(normalize)
    expect(installed.normalize('anything')).toBe('host-normalized')
  })

  it('does not replace any complete host path facade operation', () => {
    const host = {
      normalize: () => 'normalize',
      join: () => 'join',
      resolve: () => 'resolve',
      basename: () => 'basename',
      dirname: () => 'dirname',
      isAbsolute: () => true,
      relative: () => 'relative'
    }
    const target = { path: host }
    const installed = ensureRendererPathFacade(target)
    expect(installed).toBe(host)
    expect(installed.normalize()).toBe('normalize')
    expect(installed.join()).toBe('join')
    expect(installed.resolve()).toBe('resolve')
    expect(installed.basename()).toBe('basename')
    expect(installed.dirname()).toBe('dirname')
    expect(installed.isAbsolute()).toBe(true)
    expect(installed.relative()).toBe('relative')
  })

  it.each([
    ['folder\\note.md', 'folder/note.md'],
    ['/vault/note.md', '/vault/note.md'],
    ['', ''],
    [null, '']
  ])('normalizes %# to a renderer-safe path', (input, expected) => {
    expect(makePath().normalize(input)).toBe(expected)
  })

  it.each([
    [['vault', 'notes', 'one.md'], 'vault/notes/one.md'],
    [['vault', '', null, 'one.md'], 'vault/one.md'],
    [['vault\\notes', 'one.md'], 'vault/notes/one.md']
  ])('joins path parts from %#', (parts, expected) => {
    expect(makePath().join(...parts)).toBe(expected)
  })

  it.each([
    ['/vault/notes/one.md', 'one.md'],
    ['vault\\notes\\one.md', 'one.md'],
    ['one.md', 'one.md'],
    ['/', '']
  ])('returns basename for %#', (input, expected) => {
    expect(makePath().basename(input)).toBe(expected)
  })

  it.each([
    ['/vault/notes/one.md', '/vault/notes'],
    ['vault/notes/one.md', 'vault/notes'],
    ['/one.md', '/'],
    ['one.md', '.']
  ])('returns dirname for %#', (input, expected) => {
    expect(makePath().dirname(input)).toBe(expected)
  })

  it.each([
    ['/vault', true],
    ['vault', false],
    ['C:\\vault', false],
    ['', false]
  ])('detects renderer absolute paths for %#', (input, expected) => {
    expect(makePath().isAbsolute(input)).toBe(expected)
  })

  it.each([
    ['/vault/notes', '/vault/assets/image.png', '../assets/image.png'],
    ['/vault/notes', '/vault/notes', ''],
    ['vault/a/b', 'vault/a/c/d', '../c/d'],
    ['', 'vault/note.md', 'vault/note.md'],
    ['vault/note.md', '', '../..']
  ])('computes a relative path from %#', (from, to, expected) => {
    expect(makePath().relative(from, to)).toBe(expected)
  })

  it('handles deep paths without a quadratic Array.shift implementation', () => {
    const common = Array.from({ length: 10_000 }, (_, index) => `segment-${index}`)
    const from = `/${[...common, 'from'].join('/')}`
    const to = `/${[...common, 'to', 'note.md'].join('/')}`
    expect(makePath().relative(from, to)).toBe('../to/note.md')
    const source = fs.readFileSync(
      path.resolve(process.cwd(), 'Elephant/frontend/src/renderer/src/platform/rendererPathFacade.js'),
      'utf8'
    )
    expect(source).not.toContain('.shift()')
  })
})
