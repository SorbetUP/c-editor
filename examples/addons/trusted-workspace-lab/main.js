export default class TrustedWorkspaceLab {
  constructor(api) {
    this.api = api
    this.enabled = false
  }

  toggleFocus() {
    this.enabled = !this.enabled
    this.api.experimental.document.documentElement.classList.toggle('elephant-trusted-focus', this.enabled)
    return { enabled: this.enabled }
  }

  async onload(api) {
    this.api = api

    api.ui.registerStyle(`
      html.elephant-trusted-focus .en-sidebar-nav,
      html.elephant-trusted-focus .en-top-vault-bar {
        opacity: 0.22;
        filter: saturate(0.35);
        transition: opacity 160ms ease, filter 160ms ease;
      }

      html.elephant-trusted-focus .en-body-main {
        outline: 2px solid color-mix(in srgb, var(--en-primary, #2563eb) 45%, transparent);
        outline-offset: -2px;
      }

      .trusted-workspace-lab-card {
        display: grid;
        gap: 10px;
        padding: 12px;
        border: 1px solid var(--en-border, #c5cfdd);
        border-radius: 10px;
      }

      .trusted-workspace-lab-card p {
        margin: 0;
        color: var(--en-muted, #667085);
        font-size: 11px;
        line-height: 1.5;
      }
    `, 'focus-mode')

    api.commands.register({
      id: 'com.elephantnote.examples.trusted-workspace-lab.toggle-focus',
      title: 'Toggle trusted focus mode',
      description: 'Modify ElephantNote directly from a full app access addon.',
      run: () => this.toggleFocus()
    })

    api.workspace.registerSidebarItem({
      id: 'com.elephantnote.examples.trusted-workspace-lab.sidebar',
      title: 'Trusted Lab',
      tooltip: 'Toggle the trusted focus mode',
      actionId: 'com.elephantnote.examples.trusted-workspace-lab.toggle-focus',
      icon: 'shield-alert',
      order: 980
    })

    api.settings.registerSection({
      id: 'com.elephantnote.examples.trusted-workspace-lab.settings',
      title: 'Trusted Workspace Lab',
      description: 'This section is rendered by the addon inside the real Addons settings page.',
      section: 'addons',
      order: 980,
      render: (container) => {
        const documentRef = api.experimental.document
        const card = documentRef.createElement('div')
        card.className = 'trusted-workspace-lab-card'

        const description = documentRef.createElement('p')
        description.textContent = `Host resources available: ${api.resources.list().join(', ')}`

        const button = documentRef.createElement('button')
        button.type = 'button'
        button.className = 'en-secondary-button'
        const refreshLabel = () => {
          button.textContent = this.enabled ? 'Disable trusted focus mode' : 'Enable trusted focus mode'
        }
        refreshLabel()
        button.addEventListener('click', () => {
          this.toggleFocus()
          refreshLabel()
        })

        card.append(description, button)
        container.append(card)
        return () => card.remove()
      }
    })

    api.editor.registerExtension({
      id: 'com.elephantnote.examples.trusted-workspace-lab.editor-extension',
      type: 'trusted-reference',
      description: 'Proves that trusted addons can register editor extensions.'
    })

    api.layout.registerItem({
      id: 'com.elephantnote.examples.trusted-workspace-lab.layout-item',
      title: 'Trusted Lab',
      preferredZone: 'icon-rail',
      order: 980
    })

    api.resources.provide('trustedWorkspaceLab', this)

    api.ui.on(api.experimental.window, 'beforeunload', () => {
      api.experimental.document.documentElement.classList.remove('elephant-trusted-focus')
    })

    api.app.emit('elephantnote:trusted-addon-loaded', {
      addonId: api.manifest.id,
      routerReady: Boolean(api.app.router),
      piniaReady: Boolean(api.app.pinia),
      servicesReady: Boolean(api.app.services),
      resources: api.resources.list()
    })
  }

  async onunload() {
    this.api?.experimental?.document?.documentElement?.classList?.remove('elephant-trusted-focus')
  }
}
