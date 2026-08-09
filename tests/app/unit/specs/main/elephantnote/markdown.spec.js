import { describe, expect, it } from 'vitest'
import { updateMarkdownTitle } from '../../../../../../Elephant/backend/js/markdown.js'

describe('updateMarkdownTitle', () => {
  it('updates the frontmatter title and first heading', () => {
    const markdown =
      '---\n' +
      'title: Untitled\n' +
      'type: note\n' +
      '---\n\n' +
      '# Untitled\n\n' +
      'Body'
    const result = updateMarkdownTitle(markdown, 'Welcome')

    expect(result).toContain('title: "Welcome"')
    expect(result).toContain('# Welcome')
    expect(result).not.toContain('Untitled\n\nBody')
  })

  it('adds a heading when there is no frontmatter', () => {
    const result = updateMarkdownTitle('Body only', 'Renamed')

    expect(result.startsWith('# Renamed')).toBe(true)
    expect(result).toContain('Body only')
  })
})
