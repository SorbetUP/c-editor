import { afterEach, describe, expect, it } from 'vitest'
import { createApp, h, nextTick, ref } from 'vue'
import { JSDOM } from 'jsdom'

import { MuyaRuntimeEditor, isMuyaRuntimeActive, isMuyaRuntimeEnabled, readMuyaRuntimeMode } from '../../../Elephant/frontend/src/renderer/src/muya/index.js'

describe('Muya Vue runtime integration', () => {
  afterEach(() => {
    delete globalThis.document
    delete globalThis.window
    delete globalThis.getSelection
    delete globalThis.__ELEPHANT_MUYA_RUNTIME_MODE__
  })

  it('reads the immutable Rust runtime flags safely', () => {
    expect(readMuyaRuntimeMode({})).toBe('rust')
    expect(isMuyaRuntimeEnabled('shadow')).toBe(true)
    expect(isMuyaRuntimeActive('active')).toBe(true)
    expect(isMuyaRuntimeActive('shadow')).toBe(true)
  })

  it('mounts the Vue wrapper and emits ready', async() => {
    const dom = new JSDOM('<div id="app"></div>')
    globalThis.window = dom.window
    globalThis.document = dom.window.document
    globalThis.getSelection = dom.window.getSelection.bind(dom.window)
    const ready = ref(false)
    const markdown = ref('# Hello')
    const app = createApp({
      setup: () => () => h(MuyaRuntimeEditor, {
        modelValue: markdown.value,
        mode: 'active',
        'onUpdate:modelValue': (value) => { markdown.value = value },
        onReady: () => { ready.value = true }
      })
    })
    app.mount(dom.window.document.getElementById('app'))
    await nextTick()
    await nextTick()
    expect(dom.window.document.querySelector('[data-testid="muya-runtime-editor"]')).toBeTruthy()
    expect(dom.window.document.querySelector('h1')?.textContent).toBe('Hello')
    expect(ready.value).toBe(true)
    app.unmount()
  })

  it('keeps paste flow synchronized with v-model', async() => {
    const dom = new JSDOM('<div id="app"></div>')
    globalThis.window = dom.window
    globalThis.document = dom.window.document
    globalThis.getSelection = dom.window.getSelection.bind(dom.window)
    const markdown = ref('# Start')
    const app = createApp({
      setup: () => () => h(MuyaRuntimeEditor, {
        modelValue: markdown.value,
        mode: 'active',
        'onUpdate:modelValue': (value) => { markdown.value = value }
      })
    })
    app.mount(dom.window.document.getElementById('app'))
    await nextTick()
    const editor = dom.window.document.querySelector('[data-testid="muya-runtime-editor"]')
    const event = new dom.window.Event('paste', { bubbles: true, cancelable: true })
    event.clipboardData = {
      getData: (kind) => kind === 'text/html' ? '<h2>Paste</h2><p>Body</p>' : ''
    }
    editor.dispatchEvent(event)
    await nextTick()
    expect(markdown.value).toContain('## Paste')
    expect(markdown.value).toContain('Body')
    app.unmount()
  })
})
