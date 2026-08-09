import { describe, expect, it } from 'vitest'
import path from 'path'
import {
  createWorkspace,
  nextAvailableName,
  parseMarkdownMeta,
  resolveInsideVault
} from 'main_renderer/elephantnote/core'
import {
  normalizeRelativePath,
  normalizeWorkspaceSidebar
} from 'common/elephantnote/workspace'

describe('ElephantNote core', () => {
  it('creates the default workspace sidebar', () => {
    const workspace = createWorkspace('/tmp/Personal')

    expect(workspace.vaultName).toBe('Personal')
    expect(workspace.sidebar[0].title).toBe('Getting started')
    expect(workspace.sidebar[0].items[0].title).toBe('Welcome')
    expect(workspace.sidebar[0].items[0].path).toBe('Getting Started')
  })

  it('extracts note metadata from markdown frontmatter', () => {
    const meta = parseMarkdownMeta(`---
title: "Research"
type: "article"
tags: ["llm", "notes"]
---

# Ignored title
Useful body text.
`)

    expect(meta.title).toBe('Research')
    expect(meta.type).toBe('article')
    expect(meta.tags).toEqual(['llm', 'notes'])
    expect(meta.excerpt).toContain('Useful body text')
  })

  it('extracts tags from CRLF frontmatter and quoted comma values', () => {
    const meta = parseMarkdownMeta([
      '---',
      'title: "Research"',
      'tags: ["ideas", "needs, review"]',
      '---',
      '',
      '# Ignored title',
      'Useful body text.'
    ].join('\r\n'))

    expect(meta.tags).toEqual(['ideas', 'needs, review'])
  })

  it('extracts tags from YAML block frontmatter', () => {
    const meta = parseMarkdownMeta(`---
title: "Research"
tags:
  - "#ideas"
  - "needs, review"
createdAt: "2026-05-17T23:43:04.008Z"
---

# Ignored title
Useful body text.
`)

    expect(meta.tags).toEqual(['ideas', 'needs, review'])
  })

  it('falls back to the first H1 when frontmatter has no title', () => {
    const meta = parseMarkdownMeta('# Hello\n\nBody', 'Fallback.md')

    expect(meta.title).toBe('Hello')
  })

  it('blocks path traversal outside the vault', () => {
    const root = path.resolve('/tmp/vault')

    expect(() => resolveInsideVault(root, '../outside.md')).not.toThrow()
    expect(resolveInsideVault(root, '../outside.md')).toBe(path.join(root, 'outside.md'))
  })

  it('normalizes workspace paths and compatibility nested sidebar structures without platform APIs', () => {
    expect(normalizeRelativePath('\\Inbox/../Ideas//Note.md')).toBe('Inbox/Ideas/Note.md')

    const workspace = normalizeWorkspaceSidebar({
      version: 1,
      sidebar: [
        {
          id: 'compatibility',
          title: 'Compatibility',
          items: [
            { type: 'note', path: 'Folder/Note.md' },
            { type: 'folder', path: 'Folder' }
          ]
        }
      ]
    })

    expect(workspace.sidebar).toEqual([
      {
        id: 'note-folder-note-md',
        title: 'Note',
        type: 'note',
        path: 'Folder/Note.md',
        collapsed: false
      },
      {
        id: 'folder-folder',
        title: 'Folder',
        type: 'folder',
        path: 'Folder',
        collapsed: false
      }
    ])
  })

  it('generates the next available note or folder name', () => {
    const existing = new Set(['Untitled.md', 'Untitled 2.md'])

    expect(nextAvailableName('Untitled.md', (name) => existing.has(name))).toBe('Untitled 3.md')
    expect(nextAvailableName('New Folder', (name) => name === 'New Folder')).toBe('New Folder 2')
  })
})
