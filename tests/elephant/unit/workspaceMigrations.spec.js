import { describe, expect, it } from 'vitest'
import {
  CURRENT_WORKSPACE_SCHEMA_VERSION,
  migrateWorkspace
} from 'main_renderer/elephantnote/workspaceMigrations'

describe('ElephantNote workspace migrations', () => {
  it('upgrades compatibility nested sidebars and adds sync metadata', () => {
    const migrated = migrateWorkspace({
      sidebar: [
        {
          id: 'compatibility',
          title: 'Compatibility',
          items: [
            { id: 'n1', title: 'Note', type: 'note', path: 'Note.md' }
          ]
        }
      ]
    })

    expect(migrated.schemaVersion).toBe(CURRENT_WORKSPACE_SCHEMA_VERSION)
    expect(migrated.sidebar).toEqual([
      {
        id: 'n1',
        title: 'Note',
        type: 'note',
        path: 'Note.md',
        collapsed: false
      }
    ])
    expect(migrated.sync).toMatchObject({
      enabled: false,
      provider: 'git'
    })
  })
})
