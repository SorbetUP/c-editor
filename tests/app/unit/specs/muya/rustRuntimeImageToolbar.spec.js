import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApp, h, nextTick } from 'vue'

import RuntimeImageToolbar from '../../../../../Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeImageToolbar.vue'

const image = {
  image: 42,
  source: 'old.png',
  alt: 'old',
  title: 'Old title',
  rect: { left: 40, top: 240 }
}

const mountToolbar = async (listeners = {}) => {
  const host = document.createElement('div')
  document.body.appendChild(host)
  const app = createApp({
    setup: () => () => h(RuntimeImageToolbar, { image, ...listeners })
  })
  app.mount(host)
  await nextTick()
  return { app, host }
}

const input = (host, index, value) => {
  const element = host.querySelectorAll('input')[index]
  element.value = value
  element.dispatchEvent(new Event('input', { bubbles: true }))
}

describe('Rust runtime image toolbar', () => {
  afterEach(() => document.body.replaceChildren())

  it('emits a complete semantic replacement payload', async () => {
    const onApply = vi.fn()
    const { app, host } = await mountToolbar({ onApply })
    input(host, 0, 'new image.png')
    input(host, 1, 'new alt')
    input(host, 2, 'New title')

    host.querySelector('form').dispatchEvent(new Event('submit', {
      bubbles: true,
      cancelable: true
    }))
    await nextTick()

    expect(onApply).toHaveBeenCalledWith({
      image: 42,
      source: 'new image.png',
      alt: 'new alt',
      title: 'New title'
    })
    app.unmount()
  })

  it('emits choose, delete and close actions for the active image', async () => {
    const onChooseFile = vi.fn()
    const onDelete = vi.fn()
    const onClose = vi.fn()
    const { app, host } = await mountToolbar({ onChooseFile, onDelete, onClose })
    const buttons = Array.from(host.querySelectorAll('button'))

    buttons.find((button) => button.textContent.trim() === 'Choose file').click()
    buttons.find((button) => button.textContent.trim() === 'Delete').click()
    buttons.find((button) => button.getAttribute('aria-label')).click()

    expect(onChooseFile).toHaveBeenCalledWith(image)
    expect(onDelete).toHaveBeenCalledWith(image)
    expect(onClose).toHaveBeenCalledTimes(1)
    app.unmount()
  })
})
