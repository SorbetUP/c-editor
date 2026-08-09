const getInvoke = (target = globalThis) => target?.__TAURI__?.core?.invoke

const MOBILE_VAULT_CHOICE_KEY = 'elephantnote:mobile-vault-choice-v4'
const MOBILE_ADVANCED_VAULT_KEY = 'elephantnote:mobile-advanced-vault-v1'
const MOBILE_ADVANCED_DIRTY_KEY = 'elephantnote:mobile-advanced-vault-dirty-v1'
let syncTimer = null
let lifecycleInstalled = false

const invoke = (target, command, payload = {}) => {
  const invokeCommand = getInvoke(target)
  if (typeof invokeCommand !== 'function') {
    throw new Error(`Tauri command API is unavailable for ${command}`)
  }
  return invokeCommand(command, payload)
}

const normalizePath = (value = '') => String(value || '')
  .replace(/\\/g, '/')
  .replace(/\/+$/g, '')

const readFlag = (target, key) => {
  try {
    return target?.localStorage?.getItem(key) === '1'
  } catch {
    return false
  }
}

const writeFlag = (target, key, enabled) => {
  try {
    if (enabled) target?.localStorage?.setItem(key, '1')
    else target?.localStorage?.removeItem(key)
  } catch {
    // Rust and Android retain the actual vault state if WebView storage is unavailable.
  }
}

const hasExplicitVaultChoice = (target) => readFlag(target, MOBILE_VAULT_CHOICE_KEY)
const usesAdvancedVault = (target) => readFlag(target, MOBILE_ADVANCED_VAULT_KEY)
const hasPendingAdvancedWrites = (target) => readFlag(target, MOBILE_ADVANCED_DIRTY_KEY)

const rememberExplicitVaultChoice = (target, advanced = false) => {
  writeFlag(target, MOBILE_VAULT_CHOICE_KEY, true)
  writeFlag(target, MOBILE_ADVANCED_VAULT_KEY, advanced)
  writeFlag(target, MOBILE_ADVANCED_DIRTY_KEY, false)
}

const isAndroid = async (target) => {
  try {
    return (await invoke(target, 'tauri_platform_info'))?.android === true
  } catch {
    return false
  }
}

export const defaultMobileVaultPath = async () => {
  const { appDataDir } = await import('@tauri-apps/api/path')
  const base = await appDataDir()
  return `${String(base || '').replace(/[\\/]+$/g, '')}/vaults/Personal`
}

const hideLegacyAutomaticVaultUntilChoice = async (payload, target) => {
  if (!payload || hasExplicitVaultChoice(target)) return payload
  const privatePath = normalizePath(await defaultMobileVaultPath())
  const activePath = normalizePath(payload?.activeVault?.path)
  if (!activePath || activePath !== privatePath) return payload
  return {
    ...payload,
    activeVaultId: null,
    activeVault: null,
    workspace: null,
    entries: [],
    requiresVaultChoice: true
  }
}

const syncAdvancedVault = async (target) => {
  if (!usesAdvancedVault(target) || !(await isAndroid(target))) return null
  const result = await invoke(target, 'tauri_android_vault_sync')
  writeFlag(target, MOBILE_ADVANCED_DIRTY_KEY, false)
  return result
}

const restoreAdvancedVault = async (target) => {
  if (!usesAdvancedVault(target) || !(await isAndroid(target))) return null
  if (hasPendingAdvancedWrites(target)) await syncAdvancedVault(target)
  return invoke(target, 'tauri_android_vault_restore')
}

const scheduleAdvancedSync = (target, immediate = false) => {
  if (!usesAdvancedVault(target)) return
  writeFlag(target, MOBILE_ADVANCED_DIRTY_KEY, true)
  target.clearTimeout?.(syncTimer)
  syncTimer = target.setTimeout?.(() => {
    syncAdvancedVault(target).catch((error) => {
      console.warn('[mobile-vault] unable to synchronize Android document tree', error)
    })
  }, immediate ? 0 : 350)
}

const installLifecycleSync = (target) => {
  if (lifecycleInstalled || !target?.document) return
  lifecycleInstalled = true
  target.addEventListener?.('elephantnote:vault-mutated', () => scheduleAdvancedSync(target))
  target.addEventListener?.('pagehide', () => scheduleAdvancedSync(target, true))
  target.document.addEventListener('visibilitychange', async () => {
    if (target.document.visibilityState === 'hidden') {
      if (hasPendingAdvancedWrites(target)) scheduleAdvancedSync(target, true)
      return
    }
    if (!usesAdvancedVault(target)) return
    try {
      await restoreAdvancedVault(target)
      target.dispatchEvent?.(new CustomEvent('elephantnote:vault-files-changed'))
    } catch (error) {
      console.warn('[mobile-vault] unable to refresh Android document tree', error)
    }
  })
}

export const patchMobileVaultBridge = (bridge, target = globalThis) => {
  if (!bridge || typeof bridge !== 'object' || bridge.__elephantnoteMobileVaultBridge) return bridge

  const originalGetVaults = typeof bridge.getVaults === 'function'
    ? bridge.getVaults.bind(bridge)
    : null
  const originalSelectVault = typeof bridge.selectVault === 'function'
    ? bridge.selectVault.bind(bridge)
    : null

  if (originalGetVaults) {
    bridge.getVaults = async () => {
      await restoreAdvancedVault(target)
      return hideLegacyAutomaticVaultUntilChoice(await originalGetVaults(), target)
    }
  }

  bridge.createLocalVault = async () => {
    try {
      await invoke(target, 'tauri_android_vault_clear')
    } catch {
      // Desktop and older Android builds do not have a persisted tree to clear.
    }
    const result = await invoke(target, 'tauri_vaults_select_path', {
      vaultPath: await defaultMobileVaultPath()
    })
    rememberExplicitVaultChoice(target, false)
    return result
  }

  bridge.selectVault = async () => {
    if (!(await isAndroid(target))) {
      if (!originalSelectVault) throw new Error('The folder picker is unavailable in this build.')
      return originalSelectVault()
    }

    try {
      const selected = await invoke(target, 'tauri_android_vault_pick')
      if (!selected?.configured || !selected?.shadowPath) return { canceled: true }
      const result = await invoke(target, 'tauri_vaults_select_path', {
        vaultPath: selected.shadowPath
      })
      const vaultId = result?.activeVault?.id
      if (vaultId && selected.displayName) {
        try {
          const renamed = await invoke(target, 'tauri_vaults_set_name', {
            vaultId,
            name: selected.displayName
          })
          if (renamed?.activeVault) Object.assign(result, renamed)
        } catch {
          // The selected tree remains functional even if its display label cannot be updated.
        }
      }
      rememberExplicitVaultChoice(target, true)
      return result
    } catch (error) {
      if (/cancel/i.test(String(error?.message || error))) return { canceled: true }
      throw error
    }
  }

  Object.defineProperty(bridge, '__elephantnoteMobileVaultBridge', {
    value: true,
    enumerable: false
  })
  return bridge
}

export const installMobileVaultBridge = (target = globalThis) => {
  let currentBridge = patchMobileVaultBridge(target.elephantnote, target)
  const descriptor = Object.getOwnPropertyDescriptor(target, 'elephantnote')
  installLifecycleSync(target)

  if (descriptor?.configurable === false) return currentBridge

  Object.defineProperty(target, 'elephantnote', {
    configurable: true,
    enumerable: true,
    get: () => currentBridge,
    set: (bridge) => {
      currentBridge = patchMobileVaultBridge(bridge, target)
    }
  })

  return currentBridge
}

installMobileVaultBridge()
