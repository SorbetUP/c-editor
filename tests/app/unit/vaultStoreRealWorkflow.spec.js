import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

const sentMessages = []

vi.mock('electron-log', () => ({
  default: { info: vi.fn(), warn: vi.fn(), error: vi.fn(), debug: vi.fn() }
}))

const workspace = {
  sidebar: [
    { id: 'alpha', title: 'Alpha', type: 'note', path: 'Alpha.md' },
    { id: 'projects', title: 'Projects', type: 'folder', path: 'Projects' }
  ]
}

const makeEntries = () => [
  { kind: 'note', type: 'note', path: 'Alpha.md', title: 'Alpha', updatedAt: '2026-06-01T10:00:00.000Z' },
  { kind: 'folder', type: 'folder', path: 'Projects', title: 'Projects', updatedAt: '2026-06-01T11:00:00.000Z' },
  { kind: 'note', type: 'note', path: 'Projects/Beta.md', title: 'Beta', updatedAt: '2026-06-01T12:00:00.000Z' }
]

let entries

vi.mock('../../../Elephant/frontend/app/services/elephantnoteClient.js', () => ({
  elephantnoteClient: {
    vaults: {
      get: vi.fn(async() => ({
        vaults: [{ id: 'v1', name: 'Vault', path: '/vault' }],
        activeVaultId: 'v1',
        activeVault: { id: 'v1', name: 'Vault', path: '/vault' },
        workspace,
        entries
      })),
      select: vi.fn(async() => ({
        vaults: [{ id: 'v1', name: 'Vault', path: '/vault' }],
        activeVaultId: 'v1',
        activeVault: { id: 'v1', name: 'Vault', path: '/vault' },
        workspace,
        entries
      })),
      setActive: vi.fn(async() => ({
        vaults: [{ id: 'v1', name: 'Vault', path: '/vault' }],
        activeVaultId: 'v1',
        activeVault: { id: 'v1', name: 'Vault', path: '/vault' },
        workspace,
        entries
      })),
      setIcon: vi.fn(async() => ({ activeVault: { id: 'v1', name: 'Vault', path: '/vault', icon: 'book' }, vaults: [{ id: 'v1', name: 'Vault', path: '/vault', icon: 'book' }], workspace, entries })),
      setName: vi.fn(async(_id, name) => ({ activeVault: { id: 'v1', name, path: '/vault' }, vaults: [{ id: 'v1', name, path: '/vault' }], workspace, entries })),
      remove: vi.fn(async() => ({ activeVault: null, activeVaultId: null, vaults: [], workspace: null, entries: [] }))
    },
    directory: {
      list: vi.fn(async(relativePath = '') => relativePath ? entries.filter((entry) => entry.path.startsWith(`${relativePath}/`)) : entries)
    },
    notes: {
      create: vi.fn(async(currentPath = '') => {
        const path = currentPath ? `${currentPath}/Untitled.md` : 'Untitled.md'
        const note = { kind: 'note', type: 'note', path, title: 'Untitled', fullPath: `/vault/${path}`, updatedAt: '2026-06-02T10:00:00.000Z' }
        entries = [note, ...entries]
        return { note, entries }
      })
    },
    folders: {
      create: vi.fn(async(currentPath = '') => {
        const path = currentPath ? `${currentPath}/New Folder` : 'New Folder'
        const folder = { kind: 'folder', type: 'folder', path, title: 'New Folder', updatedAt: '2026-06-02T11:00:00.000Z' }
        entries = [folder, ...entries]
        return { entries }
      })
    },
    entries: {
      rename: vi.fn(async({ relativePath, title }) => {
        const parent = relativePath.includes('/') ? `${relativePath.split('/').slice(0, -1).join('/')}/` : ''
        const nextPath = `${parent}${title}`
        entries = entries.map((entry) => entry.path === relativePath ? { ...entry, path: nextPath, title: title.replace(/\.md$/i, '') } : entry)
        return { workspace, entries }
      }),
      move: vi.fn(async({ relativePath, targetDirectoryPath }) => {
        const basename = relativePath.split('/').pop()
        const nextPath = targetDirectoryPath ? `${targetDirectoryPath}/${basename}` : basename
        entries = entries.map((entry) => entry.path === relativePath ? { ...entry, path: nextPath } : entry)
        return { workspace, entries }
      }),
      delete: vi.fn(async(relativePath) => {
        entries = entries.filter((entry) => entry.path !== relativePath && !entry.path.startsWith(`${relativePath}/`))
        return { deleted: true, path: relativePath }
      })
    },
    sidebar: {
      attach: vi.fn(async(payload) => ({ sidebar: [...workspace.sidebar, { id: payload.relativePath, path: payload.relativePath, title: payload.title, type: payload.type }] })),
      detach: vi.fn(async(pathname) => ({ sidebar: workspace.sidebar.filter((item) => item.path !== pathname) }))
    },
    wiki: {
      list: vi.fn(async() => ({ records: [] })),
      propose: vi.fn(async() => ({ records: [] })),
      accept: vi.fn(async() => ({ wiki: { records: [] }, entries })),
      dismiss: vi.fn(async() => ({ records: [] }))
    }
  }
}))

beforeEach(() => {
  setActivePinia(createPinia())
  entries = makeEntries()
  sentMessages.length = 0
  window.localStorage.clear()
  window.path = {
    relative: (from, to) => String(to).replace(String(from).replace(/\/$/, '') + '/', ''),
    dirname: (pathname) => String(pathname).includes('/') ? String(pathname).split('/').slice(0, -1).join('/') : '.',
    isAbsolute: (pathname) => String(pathname).startsWith('/')
  }
  window.tauri = {
    ipcRenderer: {
      send: (...args) => sentMessages.push(args),
      on: vi.fn(),
      off: vi.fn(),
      removeListener: vi.fn()
    }
  }
})

describe('real vault store workflow build/coverage', () => {
  it('loads a seeded vault and exposes real active entries', async() => {
    const { useVaultStore } = await import('../../../Elephant/frontend/app/stores/vaultStore.js')
    const store = useVaultStore()
    await store.load()
    expect(store.hasVault).toBe(true)
    expect(store.activeVault.path).toBe('/vault')
    expect(store.activeNoteEntries.map((entry) => entry.path)).toContain('Alpha.md')
  })

  it('keeps entry getters safe when a legacy or partial response contains undefined arrays', async() => {
    const { useVaultStore } = await import('../../../Elephant/frontend/app/stores/vaultStore.js')
    const store = useVaultStore()
    await store.load()
    store.entries = undefined
    store.rootEntries = undefined
    store.openedNotes = undefined
    store.pinnedNotePaths = undefined
    expect(store.activeEntries).toEqual([])
    expect(store.rootSidebarEntries).toEqual([])
    expect(store.activeNoteEntries).toEqual([])
    expect(store.recentNoteEntries).toEqual([])
  })

  it('opens a note through the real store and sends the open-file IPC message', async() => {
    const { useVaultStore } = await import('../../../Elephant/frontend/app/stores/vaultStore.js')
    const store = useVaultStore()
    await store.load()
    store.openNote(entries[0])
    expect(store.openedNotePath).toBe('Alpha.md')
    expect(store.openedNotes[0].title).toBe('Alpha')
    expect(sentMessages[0][0]).toBe('mt::open-file')
    expect(sentMessages[0][1]).toBe('/vault/Alpha.md')
  })

  it('preserves the open note when the active vault payload is refreshed', async() => {
    const { useVaultStore } = await import('../../../Elephant/frontend/app/stores/vaultStore.js')
    const store = useVaultStore()
    await store.load()
    store.openNote(entries[0])
    const openedNotes = store.openedNotes
    store.applyPayload({
      vaults: [{ id: 'v1', name: 'Vault', path: '/vault' }],
      activeVaultId: 'v1',
      activeVault: { id: 'v1', name: 'Vault', path: '/vault' },
      workspace,
      entries
    })
    expect(store.openedNotePath).toBe('Alpha.md')
    expect(store.openedNotes).toBe(openedNotes)
  })

  it('creates a note in the current directory and opens it', async() => {
    const { useVaultStore } = await import('../../../Elephant/frontend/app/stores/vaultStore.js')
    const store = useVaultStore()
    await store.load()
    await store.openDirectory('Projects')
    await store.createNote()
    expect(store.openedNotePath).toBe('Projects/Untitled.md')
    expect(store.entries.map((entry) => entry.path)).toContain('Projects/Untitled.md')
    expect(sentMessages.at(-1)[0]).toBe('mt::open-file')
  })

  it('creates folders with the real folder action', async() => {
    const { useVaultStore } = await import('../../../Elephant/frontend/app/stores/vaultStore.js')
    const store = useVaultStore()
    await store.load()
    await store.createFolder()
    expect(store.entries.map((entry) => entry.path)).toContain('New Folder')
  })

  it('renames an entry and updates pinned/opened paths', async() => {
    const { useVaultStore } = await import('../../../Elephant/frontend/app/stores/vaultStore.js')
    const store = useVaultStore()
    await store.load()
    store.pinnedNotePaths = ['Alpha.md']
    store.openedNotePath = 'Alpha.md'
    await store.renameEntry(entries[0], 'Renamed.md')
    expect(store.entries.map((entry) => entry.path)).toContain('Renamed.md')
    expect(store.pinnedNotePaths).toContain('Renamed.md')
    expect(store.openedNotePath).toBe('Renamed.md')
  })

  it('moves a note into a folder and can move it back out', async() => {
    const { useVaultStore } = await import('../../../Elephant/frontend/app/stores/vaultStore.js')
    const store = useVaultStore()
    await store.load()
    store.openedNotePath = 'Alpha.md'
    await store.moveEntry(entries[0], 'Projects')
    expect(store.rootEntries.map((entry) => entry.path)).toContain('Projects/Alpha.md')
    await store.moveEntry(store.rootEntries.find((entry) => entry.path === 'Projects/Alpha.md'), '')
    expect(store.rootEntries.map((entry) => entry.path)).toContain('Alpha.md')
  })

  it('rejects moving a folder into itself', async() => {
    const { useVaultStore } = await import('../../../Elephant/frontend/app/stores/vaultStore.js')
    const store = useVaultStore()
    await store.load()
    const result = await store.moveEntry(entries.find((entry) => entry.path === 'Projects'), 'Projects/Sub')
    expect(result).toBe(false)
  })

  it('deletes notes and folders through the real delete action', async() => {
    const { useVaultStore } = await import('../../../Elephant/frontend/app/stores/vaultStore.js')
    const store = useVaultStore()
    await store.load()
    await store.deleteEntry(entries.find((entry) => entry.path === 'Projects'))
    expect(store.entries.map((entry) => entry.path)).not.toContain('Projects')
    expect(store.entries.map((entry) => entry.path)).not.toContain('Projects/Beta.md')
  })


  it('accepts the direct workspace object returned by Rust sidebar commands', async() => {
    const { useVaultStore } = await import('../../../Elephant/frontend/app/stores/vaultStore.js')
    const store = useVaultStore()
    await store.load()
    await store.attachEntryToSidebar({ path: 'Gamma.md', title: 'Gamma', type: 'note' })
    expect(store.workspace.sidebar.map((entry) => entry.path)).toContain('Gamma.md')
    await store.detachEntryFromSidebar('Alpha.md')
    expect(store.workspace.sidebar.map((entry) => entry.path)).not.toContain('Alpha.md')
  })

  it('switches real workspace views and falls back to notes for invalid views', async() => {
    const { useVaultStore } = await import('../../../Elephant/frontend/app/stores/vaultStore.js')
    const store = useVaultStore()
    await store.load()
    store.setWorkspaceView('wiki')
    expect(store.activeWorkspaceView).toBe('wiki')
    store.setWorkspaceView('graph')
    expect(store.activeWorkspaceView).toBe('graph')
    store.setWorkspaceView('invalid-view')
    expect(store.activeWorkspaceView).toBe('notes')
  })
})
