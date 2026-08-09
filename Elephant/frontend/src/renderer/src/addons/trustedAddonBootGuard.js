const TRUSTED_ACTIVATION_MARKER_KEY = 'elephantnote:addons:trusted-activation-marker'
const TRUSTED_SAFE_MODE_PREF_KEY = 'addons.trustedSafeMode'
const TRUSTED_SAFE_MODE_LOCAL_KEY = 'elephantnote:addons:trusted-safe-mode'

const safeString = (value) => typeof value === 'string' ? value.trim() : ''

const setPreference = (key, value, target = globalThis) => {
  const invoke = target?.__TAURI__?.core?.invoke
  if (typeof invoke !== 'function') return Promise.resolve(null)
  return invoke('tauri_prefs_set', { key, value })
}

export const readTrustedActivationMarker = (target = globalThis) => {
  const raw = target?.localStorage?.getItem?.(TRUSTED_ACTIVATION_MARKER_KEY)
  if (!raw) return null
  try {
    const marker = JSON.parse(raw)
    const addonId = safeString(marker?.addonId)
    const packageHash = safeString(marker?.packageHash)
    if (!addonId) return null
    return {
      addonId,
      packageHash,
      startedAt: safeString(marker?.startedAt)
    }
  } catch {
    return null
  }
}

export const beginTrustedActivation = (record, target = globalThis) => {
  const marker = {
    addonId: safeString(record?.manifest?.id),
    packageHash: safeString(record?.packageHash || record?.manifest?.packageHash),
    startedAt: new Date().toISOString()
  }
  if (!marker.addonId) throw new Error('Trusted activation marker requires an addon id')
  target?.localStorage?.setItem?.(TRUSTED_ACTIVATION_MARKER_KEY, JSON.stringify(marker))
  return marker
}

export const clearTrustedActivationMarker = (target = globalThis) => {
  target?.localStorage?.removeItem?.(TRUSTED_ACTIVATION_MARKER_KEY)
}

export const recoverTrustedActivationCrash = async (target = globalThis) => {
  const marker = readTrustedActivationMarker(target)
  if (!marker) return null
  clearTrustedActivationMarker(target)
  target?.localStorage?.setItem?.(TRUSTED_SAFE_MODE_LOCAL_KEY, 'true')
  await setPreference(TRUSTED_SAFE_MODE_PREF_KEY, true, target).catch(() => {})
  return marker
}

// Run before the external-addon controller starts. A leftover marker means the
// previous renderer stopped during trusted addon activation.
export const trustedActivationRecovery = recoverTrustedActivationCrash()
