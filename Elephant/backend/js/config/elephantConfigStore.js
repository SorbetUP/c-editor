import fs from 'fs-extra'
import path from 'path'
import { app } from 'electron'
import Store from 'electron-store'

import { normalizeAiConfig } from 'common/elephantnote/aiProviders'
import { createDefaultModelSelection, createDefaultPluginState, createDefaultTaskState } from 'common/elephantnote/atomicWorkspace'
import { normalizeFeatureFlags } from 'common/elephantnote/featureFlags'
import { normalizeGoogleCalendarConfig } from 'common/elephantnote/googleCalendar'
import { WORKSPACE_DIR } from 'common/elephantnote/workspace'
import { normalizeProgramEnvironments } from '../programRuntime'

const resolveElectronAppDataPath = () => {
  if (app && typeof app.getPath === 'function') return app.getPath('appData')
  return path.join(process.cwd(), '.elephantnote-test-appdata')
}

const resolveElephantNoteConfigDir = () => process.env.ELEPHANTNOTE_CONFIG_DIR || path.join(resolveElectronAppDataPath(), 'ElephantNote')

export const ELEPHANTNOTE_CONFIG_DIR = resolveElephantNoteConfigDir()
export const ELEPHANTNOTE_APP_DIRS = Object.freeze({
  configs: path.join(ELEPHANTNOTE_CONFIG_DIR, 'configs'),
  chatHistory: path.join(ELEPHANTNOTE_CONFIG_DIR, 'chat-history'),
  providers: path.join(ELEPHANTNOTE_CONFIG_DIR, 'providers'),
  runtime: path.join(ELEPHANTNOTE_CONFIG_DIR, 'runtime'),
  cache: path.join(ELEPHANTNOTE_CONFIG_DIR, 'cache')
})
export const ELEPHANTNOTE_AI_SETTINGS_FILE = path.join(ELEPHANTNOTE_APP_DIRS.configs, 'ai-settings.json')
export const ELEPHANTNOTE_COMPATIBILITY_AI_SETTINGS_FILE = path.join(ELEPHANTNOTE_CONFIG_DIR, 'ai-settings.json')

const VAULT_HIDDEN_DATA_DIRS = Object.freeze(['embeddings', 'wiki', 'dashboard'])

const ensureAppDataDirsSync = () => {
  try {
    fs.ensureDirSync(ELEPHANTNOTE_CONFIG_DIR)
    for (const directory of Object.values(ELEPHANTNOTE_APP_DIRS)) {
      fs.ensureDirSync(directory)
    }
  } catch (error) {
    console.warn('[app-config] ensure app data folders failed:', error)
  }
}

const migrateCompatibilityAiSettingsFileSync = () => {
  try {
    if (
      fs.pathExistsSync(ELEPHANTNOTE_COMPATIBILITY_AI_SETTINGS_FILE) &&
      !fs.pathExistsSync(ELEPHANTNOTE_AI_SETTINGS_FILE)
    ) {
      fs.copySync(ELEPHANTNOTE_COMPATIBILITY_AI_SETTINGS_FILE, ELEPHANTNOTE_AI_SETTINGS_FILE)
    }
  } catch (error) {
    console.warn('[ai-config] migrate compatibility ai-settings.json failed:', error)
  }
}

const ensureVaultHiddenDirsSync = (vaults = []) => {
  for (const vault of Array.isArray(vaults) ? vaults : []) {
    const vaultPath = String(vault?.path || '').trim()
    if (!vaultPath) continue
    try {
      const metaDir = path.join(path.resolve(vaultPath), WORKSPACE_DIR)
      fs.ensureDirSync(metaDir)
      for (const directory of VAULT_HIDDEN_DATA_DIRS) {
        fs.ensureDirSync(path.join(metaDir, directory))
      }
    } catch (error) {
      console.warn('[vault-config] ensure hidden vault folders failed:', { vaultPath, error })
    }
  }
}

ensureAppDataDirsSync()
migrateCompatibilityAiSettingsFileSync()

const readAiSettingsFile = () => {
  try {
    if (!fs.pathExistsSync(ELEPHANTNOTE_AI_SETTINGS_FILE)) return null
    const data = fs.readJsonSync(ELEPHANTNOTE_AI_SETTINGS_FILE)
    return data && typeof data === 'object' && !Array.isArray(data) ? data : null
  } catch (error) {
    console.warn('[ai-config] read ai-settings.json failed:', error)
    return null
  }
}

const writeAiSettingsFile = (aiConfig = {}) => {
  try {
    ensureAppDataDirsSync()
    fs.writeJsonSync(
      ELEPHANTNOTE_AI_SETTINGS_FILE,
      {
        version: 1,
        updatedAt: new Date().toISOString(),
        aiConfig: normalizeAiConfig(aiConfig)
      },
      { spaces: 2 }
    )
  } catch (error) {
    console.warn('[ai-config] write ai-settings.json failed:', error)
  }
}

const readPersistedAiConfig = () => {
  const file = readAiSettingsFile()
  const fromFile = file?.aiConfig || null
  const fromStore = elephantConfigStore.get('aiConfig') || {}
  return normalizeAiConfig({ ...fromStore, ...(fromFile || {}) })
}

export const createElephantConfigDefaults = () => ({
  vaults: [],
  activeVaultId: null,
  featureFlags: normalizeFeatureFlags(),
  aiConfig: normalizeAiConfig(),
  atomicModelSelection: createDefaultModelSelection(),
  atomicPluginState: createDefaultPluginState(),
  atomicTaskState: createDefaultTaskState(),
  googleCalendarConfig: normalizeGoogleCalendarConfig(),
  programEnvironments: normalizeProgramEnvironments()
})

export const elephantConfigStore = new Store({
  name: 'elephantnote',
  cwd: ELEPHANTNOTE_CONFIG_DIR,
  defaults: createElephantConfigDefaults()
})

export const getConfig = () => {
  const vaults = elephantConfigStore.get('vaults') || []
  ensureAppDataDirsSync()
  ensureVaultHiddenDirsSync(vaults)
  return {
    vaults,
    activeVaultId: elephantConfigStore.get('activeVaultId') || null,
    featureFlags: normalizeFeatureFlags(elephantConfigStore.get('featureFlags') || {}),
    aiConfig: readPersistedAiConfig(),
    atomicModelSelection: {
      ...createDefaultModelSelection(),
      ...(elephantConfigStore.get('atomicModelSelection') || {})
    },
    atomicPluginState: {
      ...createDefaultPluginState(),
      ...(elephantConfigStore.get('atomicPluginState') || {})
    },
    atomicTaskState: {
      ...createDefaultTaskState(),
      ...(elephantConfigStore.get('atomicTaskState') || {})
    },
    googleCalendarConfig: normalizeGoogleCalendarConfig(elephantConfigStore.get('googleCalendarConfig') || {}),
    programEnvironments: normalizeProgramEnvironments(elephantConfigStore.get('programEnvironments') || {})
  }
}

export const setConfig = (config) => {
  const aiConfig = normalizeAiConfig(config.aiConfig || {})
  const vaults = Array.isArray(config.vaults) ? config.vaults : []
  ensureAppDataDirsSync()
  ensureVaultHiddenDirsSync(vaults)
  elephantConfigStore.set('vaults', vaults)
  elephantConfigStore.set('activeVaultId', config.activeVaultId)
  elephantConfigStore.set('featureFlags', normalizeFeatureFlags(config.featureFlags || {}))
  elephantConfigStore.set('aiConfig', aiConfig)
  writeAiSettingsFile(aiConfig)
  elephantConfigStore.set('atomicModelSelection', {
    ...createDefaultModelSelection(),
    ...(config.atomicModelSelection || {})
  })
  elephantConfigStore.set('atomicPluginState', {
    ...createDefaultPluginState(),
    ...(config.atomicPluginState || {})
  })
  elephantConfigStore.set('atomicTaskState', {
    ...createDefaultTaskState(),
    ...(config.atomicTaskState || {})
  })
  elephantConfigStore.set('googleCalendarConfig', normalizeGoogleCalendarConfig(config.googleCalendarConfig || {}))
  elephantConfigStore.set('programEnvironments', normalizeProgramEnvironments(config.programEnvironments || {}))
}
