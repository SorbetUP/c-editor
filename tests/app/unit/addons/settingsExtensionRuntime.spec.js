import { afterEach, describe, expect, it, vi } from 'vitest'

import { ElephantAddonManager } from '../../../../Elephant/frontend/src/renderer/src/addons/AddonManager.js'
import { installSettingsContributionRuntime } from '../../../../Elephant/frontend/src/renderer/src/addons/settingsContributionRuntime.js'

const flush = async () => {
  await Promise.resolve()
  await Promise.resolve()
  await new Promise((resolve) => setTimeout(resolve, 0))
}

const mountSettingsPage = (section, body = '<section class="en-settings-group" data-native-settings="true"></section>') => {
  document.body.innerHTML = `
    <section class="en-settings-panel" data-v-settings-test>
      <main class="en-settings-content" data-v-settings-test data-active-section="${section}">
        <div class="en-settings-page-title" data-v-settings-test><h1>${section[0].toUpperCase()}${section.slice(1)}</h1></div>
        ${body}
      </main>
    </section>
  `
  return document.querySelector('.en-settings-content')
}

afterEach(() => {
  delete globalThis.elephantnote
  document.body.innerHTML = ''
  vi.restoreAllMocks()
})

describe('addon settings contribution runtime', () => {
  it('renders a contribution only in its target section and removes it on disable', async () => {
    const content = mountSettingsPage('appearance')
    const cleanup = vi.fn()
    const manager = new ElephantAddonManager({
      logger: { info: vi.fn(), warn: vi.fn(), error: vi.fn() }
    })
    manager.host = { list: () => ['router', 'services'] }
    manager.register({
      manifest: {
        id: 'com.example.settings',
        name: 'Settings extension',
        version: '1.0.0'
      },
      activate(context) {
        context.addSettingsSection({
          id: 'com.example.settings.panel',
          title: 'Addon-owned settings',
          description: 'Rendered in the real Addons category.',
          section: 'addons',
          render(container) {
            const marker = document.createElement('button')
            marker.type = 'button'
            marker.textContent = 'Real addon control'
            container.append(marker)
            return cleanup
          }
        })
      }
    })

    const runtime = installSettingsContributionRuntime(manager)
    await manager.enable('com.example.settings')
    await flush()

    expect(content.querySelector('[data-elephant-addon-settings-host]')).toBeNull()

    content.dataset.activeSection = 'addons'
    content.querySelector('h1').textContent = 'Addons'
    runtime.sync()
    await flush()

    const extension = content.querySelector('[data-elephant-addon-settings-host]')
    expect(extension).not.toBeNull()
    expect(extension.dataset.addonId).toBe('com.example.settings')
    expect(extension.textContent).toContain('Addon-owned settings')
    expect(extension.textContent).toContain('Real addon control')

    await manager.disable('com.example.settings')
    await flush()

    expect(content.querySelector('[data-elephant-addon-settings-host]')).toBeNull()
    expect(cleanup).toHaveBeenCalledOnce()
    runtime.dispose()
  })

  it('mounts a bare contribution in a named slot without adding duplicate chrome', async () => {
    const content = mountSettingsPage(
      'ai',
      '<section class="en-ai-card">External providers</section><div data-elephant-addon-settings-slot="ai.providers.after-external"></div>'
    )
    const manager = new ElephantAddonManager()
    manager.host = { list: () => [] }
    manager.register({
      manifest: { id: 'com.example.codex', name: 'Codex', version: '1.0.0' },
      activate(context) {
        context.addSettingsSection({
          id: 'com.example.codex.settings',
          title: 'Should not be rendered as duplicate chrome',
          description: 'Should not be rendered as duplicate chrome.',
          section: 'ai',
          slot: 'ai.providers.after-external',
          chrome: false,
          render(container) {
            const card = document.createElement('section')
            card.className = 'en-ai-card'
            card.textContent = 'ChatGPT subscription'
            container.append(card)
          }
        })
      }
    })

    const runtime = installSettingsContributionRuntime(manager)
    await manager.enable('com.example.codex')
    await flush()

    const slot = content.querySelector('[data-elephant-addon-settings-slot="ai.providers.after-external"]')
    const host = slot.querySelector('[data-elephant-addon-settings-host]')
    const card = slot.querySelector('.en-ai-card')
    expect(host).not.toBeNull()
    expect(host.classList.contains('en-addon-settings-bare-host')).toBe(true)
    expect(host.textContent).toBe('ChatGPT subscription')
    expect(host.textContent).not.toContain('Should not be rendered as duplicate chrome')
    expect(card.hasAttribute('data-v-settings-test')).toBe(true)

    await manager.disable('com.example.codex')
    await flush()
    expect(slot.querySelector('[data-elephant-addon-settings-host]')).toBeNull()
    runtime.dispose()
  })

  it('waits for a named slot instead of appending the contribution to the wrong page location', async () => {
    const content = mountSettingsPage('ai', '<section class="en-ai-card">Chat route</section>')
    const manager = new ElephantAddonManager()
    manager.host = { list: () => [] }
    manager.register({
      manifest: { id: 'com.example.slot', name: 'Slot extension', version: '1.0.0' },
      activate(context) {
        context.addSettingsSection({
          id: 'com.example.slot.panel',
          section: 'ai',
          slot: 'ai.providers.after-external',
          chrome: false,
          render(container) {
            container.textContent = 'Provider-only control'
          }
        })
      }
    })

    const runtime = installSettingsContributionRuntime(manager)
    await manager.enable('com.example.slot')
    await flush()
    expect(content.textContent).not.toContain('Provider-only control')

    const slot = document.createElement('div')
    slot.setAttribute('data-elephant-addon-settings-slot', 'ai.providers.after-external')
    content.append(slot)
    await flush()
    expect(slot.textContent).toContain('Provider-only control')
    runtime.dispose()
  })

  it('keeps a root addon settings page mounted while a nested addon slot appears', async () => {
    const content = mountSettingsPage('ai', '')
    const manager = new ElephantAddonManager()
    manager.host = { list: () => [] }
    const rootRender = vi.fn((container) => {
      const slot = document.createElement('div')
      slot.setAttribute('data-elephant-addon-settings-slot', 'ai.providers.after-external')
      container.append(slot)
    })
    const nestedRender = vi.fn((container) => {
      container.textContent = 'Codex account'
    })

    manager.register({
      manifest: { id: 'com.example.ai', name: 'AI', version: '1.0.0' },
      activate(context) {
        context.addSettingsSection({ id: 'com.example.ai.settings', section: 'ai', chrome: false, render: rootRender })
      }
    })
    manager.register({
      manifest: { id: 'com.example.codex-nested', name: 'Codex', version: '1.0.0' },
      activate(context) {
        context.addSettingsSection({
          id: 'com.example.codex-nested.settings',
          section: 'ai',
          slot: 'ai.providers.after-external',
          chrome: false,
          render: nestedRender
        })
      }
    })

    const runtime = installSettingsContributionRuntime(manager)
    await manager.enable('com.example.ai')
    await manager.enable('com.example.codex-nested')
    await flush()
    await flush()
    await flush()

    expect(rootRender).toHaveBeenCalledTimes(1)
    expect(nestedRender).toHaveBeenCalledTimes(1)
    expect(content.querySelectorAll('[data-elephant-addon-settings-host]')).toHaveLength(2)
    runtime.dispose()
  })

  it('moves contributions when the active settings category changes', async () => {
    const content = mountSettingsPage('addons')
    const manager = new ElephantAddonManager()
    manager.register({
      manifest: { id: 'com.example.editor-settings', name: 'Editor settings', version: '1.0.0' },
      activate(context) {
        context.addSettingsSection({
          id: 'com.example.editor-settings.panel',
          title: 'Editor-only addon setting',
          section: 'editor',
          fields: [{ id: 'enabled', label: 'Enabled', type: 'boolean', value: true }]
        })
      }
    })

    const runtime = installSettingsContributionRuntime(manager)
    await manager.enable('com.example.editor-settings')
    await flush()
    expect(content.textContent).not.toContain('Editor-only addon setting')

    content.dataset.activeSection = 'editor'
    content.querySelector('h1').textContent = 'Editor'
    runtime.sync()
    await flush()
    expect(content.textContent).toContain('Editor-only addon setting')

    content.dataset.activeSection = 'sync'
    content.querySelector('h1').textContent = 'Sync'
    runtime.sync()
    await flush()
    expect(content.textContent).not.toContain('Editor-only addon setting')
    runtime.dispose()
  })
})
