import { describe, expect, it } from 'vitest'
import { hasSparseGraph, hasSubstantialVault, shouldRepairSparseGraph } from 'elephant-front/runtime/graphRuntimeFixes'

describe('graph runtime repair guard', () => {
  it('detects sparse semantic graphs', () => {
    expect(hasSparseGraph({ indexInspection: { graph: { nodes: [] } } })).toBe(true)
    expect(hasSparseGraph({ indexInspection: { graph: { nodes: [{ id: 'a' }] } } })).toBe(true)
    expect(hasSparseGraph({ indexInspection: { graph: { nodes: [{ id: 'a' }, { id: 'b' }] } } })).toBe(false)
  })

  it('detects vaults large enough to justify repair', () => {
    expect(hasSubstantialVault({ rootEntries: Array.from({ length: 9 }, (_, index) => ({ path: `${index}.md`, type: 'note' })) })).toBe(true)
    expect(hasSubstantialVault({ rootEntries: [{ path: 'Folder', type: 'folder' }] })).toBe(true)
    expect(hasSubstantialVault({ rootEntries: [{ path: 'Note.md', type: 'note' }] })).toBe(false)
  })

  it('repairs only active graph views and never repeats the same vault repair', () => {
    const vaultStore = {
      activeWorkspaceView: 'graph',
      rootEntries: Array.from({ length: 10 }, (_, index) => ({ path: `${index}.md`, type: 'note' }))
    }
    const searchStore = { indexInspection: { graph: { nodes: [] } } }

    expect(shouldRepairSparseGraph({ vaultStore, searchStore, vaultPath: '/vault' })).toBe(true)
    expect(shouldRepairSparseGraph({ vaultStore, searchStore, vaultPath: '/vault', lastRepairPath: '/vault' })).toBe(false)
    expect(shouldRepairSparseGraph({ vaultStore, searchStore, vaultPath: '/vault', inFlight: true })).toBe(false)
    expect(shouldRepairSparseGraph({ vaultStore: { ...vaultStore, activeWorkspaceView: 'notes' }, searchStore, vaultPath: '/vault' })).toBe(false)
  })
})
