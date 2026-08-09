import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('develop_next mobile experience', () => {
  it('keeps addon views while adding a touch-first drawer', () => {
    const shell = read('Elephant/frontend/app/components/shell/AppShell.vue')
    expect(shell).toContain("'en-mobile-shell': isMobileShell")
    expect(shell).toContain('v-if="sidebarVisible || isMobileShell"')
    expect(shell).toContain(':class="{ visible: drawerProgress > 0 }"')
    expect(shell).toContain('const drawerProgress = ref(1)')
    expect(shell).toContain('handleDrawerPointerMove')
    expect(shell).toContain('cubic-bezier(0.22, 1, 0.36, 1)')
    expect(shell).toContain('active-addon-view-id="activeAddonViewId"')
    expect(shell).toContain('shellRightZones')
    expect(shell).toContain("'elephantnote:vault-files-changed'")
  })

  it('offers explicit private and Android vault onboarding without Android-only visible copy on desktop', () => {
    const picker = read('Elephant/frontend/app/components/shell/EmptyVaultPicker.vue')
    expect(picker).toContain('Private storage')
    expect(picker).toContain('Choose folder')
    expect(picker).toContain('aria-label="Dossier Android — Choose a folder and authorize access"')
    expect(picker).toContain("emit('create-local')")
    expect(picker).toContain("emit('choose')")
  })

  it('normalizes direct Tauri create responses', () => {
    const clients = read('Elephant/frontend/app/services/elephantnoteClient/domainClients.js')
    expect(clients).toContain('normalizeCreatedNote')
    expect(clients).toContain('normalizeCreatedFolder')
    expect(clients).toContain('call(API.DIRECTORY_LIST')
  })
})
