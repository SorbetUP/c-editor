export const CURRENT_WORKSPACE_SCHEMA_VERSION = 2

const flattenCompatibilitySidebar = (sidebar = []) => {
  const flattened = []
  for (const item of sidebar) {
    if ((item.type === 'note' || item.type === 'folder') && item.path) {
      flattened.push(item)
      continue
    }
    for (const child of item.items || []) {
      if (child?.path) {
        flattened.push({
          ...child,
          type: child.type === 'note' ? 'note' : 'folder',
          collapsed: Boolean(child.collapsed)
        })
      }
    }
  }
  return flattened
}

const migrations = [
  {
    from: 0,
    to: 1,
    migrate: (workspace) => ({
      ...workspace,
      schemaVersion: 1,
      sidebar: flattenCompatibilitySidebar(workspace.sidebar || [])
    })
  },
  {
    from: 1,
    to: 2,
    migrate: (workspace) => ({
      ...workspace,
      schemaVersion: 2,
      sync: {
        enabled: false,
        provider: 'git',
        lastRunAt: '',
        ...(workspace.sync || {})
      },
      features: {
        ...(workspace.features || {})
      }
    })
  }
]

export const migrateWorkspace = (workspace = {}) => {
  let nextWorkspace = {
    ...workspace,
    schemaVersion: Number.isInteger(workspace.schemaVersion) ? workspace.schemaVersion : 0
  }

  for (const migration of migrations) {
    if (nextWorkspace.schemaVersion === migration.from) {
      nextWorkspace = migration.migrate(nextWorkspace)
    }
  }

  if (nextWorkspace.schemaVersion !== CURRENT_WORKSPACE_SCHEMA_VERSION) {
    nextWorkspace = {
      ...nextWorkspace,
      schemaVersion: CURRENT_WORKSPACE_SCHEMA_VERSION
    }
  }

  return nextWorkspace
}
