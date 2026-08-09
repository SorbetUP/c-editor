import { describe, expect, it } from 'vitest'

const imports = [
  ['App page', () => import('../../../Elephant/frontend/src/renderer/src/pages/app.vue')],
  ['App shell', () => import('../../../Elephant/frontend/app/components/shell/AppShell.vue')],
  ['Addon workspace router', () => import('../../../Elephant/frontend/app/components/views/AddonWorkspaceRouter.vue')],
  ['Search modal', () => import('../../../Elephant/frontend/app/search/SearchModal.vue')],
  ['Settings panel', () => import('../../../Elephant/frontend/app/components/settings/SettingsPanel.vue')]
]

describe('real core component import smoke', () => {
  for (const [name, load] of imports) {
    const timeout = name === 'App page' ? 15000 : 5000
    it(`${name} imports without renderer bootstrap errors`, async() => {
      const module = await load()
      expect(module.default).toBeTruthy()
    }, timeout)
  }
})
