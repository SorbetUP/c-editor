// @vitest-environment jsdom

import { afterEach, describe, expect, it } from 'vitest'
import Muya from '../../../../../../Elephant/frontend/src/muya/lib'

const markdown = [
  '# Notes',
  '',
  '## Tasks',
  '',
  '- [ ] Review the note',
  '',
  '```text',
  'literal code',
  '```',
  '',
  '![drawing](../assets/excalidraw.png)',
  '',
  '# ddd'
].join('\n')

const settle = async() => {
  await new Promise((resolve) => setTimeout(resolve, 0))
  await new Promise((resolve) => setTimeout(resolve, 80))
}

describe('Muya Markdown rendering regression', () => {
  let muya

  afterEach(() => {
    muya?.destroy?.()
    muya = null
    document.body.innerHTML = ''
  })

  it('renders headings, task lists, fenced code and images instead of raw Markdown', async() => {
    const host = document.createElement('div')
    host.className = 'muya-regression-host'
    document.body.appendChild(host)
    muya = new Muya(host, { markdown, t: (key) => key })
    await settle()
    const surface = muya.container

    expect(surface.querySelector('h1 .ag-plain-text')?.textContent).toBe('Notes')
    expect(surface.querySelector('h2 .ag-plain-text')?.textContent).toBe('Tasks')
    expect(surface.querySelector('input[type="checkbox"]')).not.toBeNull()
    expect(surface.querySelector('pre.ag-fence-code')?.textContent).toContain('literal code')
    const image = surface.querySelector('.ag-inline-image[data-raw="![drawing](../assets/excalidraw.png)"]')
    expect(image?.querySelector('.ag-image-container')).not.toBeNull()
    expect(surface.textContent).not.toContain('```text')
    expect(surface.textContent).not.toContain('![drawing]')
  })

  it('keeps semantic rendering when a note is replaced through the file-change path', async() => {
    const host = document.createElement('div')
    document.body.appendChild(host)
    muya = new Muya(host, { markdown: '# Initial', t: (key) => key })
    await settle()

    muya.setMarkdown(markdown, null, true, undefined, undefined)
    await settle()

    const surface = muya.container
    expect(surface.querySelector('h1 .ag-plain-text')?.textContent).toBe('Notes')
    expect(surface.querySelector('h2 .ag-plain-text')?.textContent).toBe('Tasks')
    expect(surface.querySelector('pre.ag-fence-code')?.textContent).toContain('literal code')
    expect(surface.textContent).not.toContain('```text')
    expect(surface.textContent).not.toContain('![drawing]')
  })
})
