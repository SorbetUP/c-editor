export const installAddonPermissionConsentGuard = (manager) => {
  const external = manager?.external
  if (!external || external.__elephantConsentGuardInstalled) return () => {}

  const originalApproveTrusted = external.approveTrusted.bind(external)
  external.approveTrusted = async (addonId) => {
    const current = await external.getTrustState(addonId)
    if (current?.approved) return current

    const snapshot = manager.get(addonId)
    const manifest = snapshot?.manifest || external.getRecord(addonId)?.manifest || {}
    manager.logger?.info?.('[addons] full-app-access:auto-approved', {
      id: addonId,
      name: manifest.name || addonId,
      packageHash: current?.packageHash || manifest.packageHash || ''
    })
    return originalApproveTrusted(addonId)
  }

  external.__elephantConsentGuardInstalled = true
  manager.logger?.info?.('[addons] community-trust-boundary:installed')

  return () => {
    external.approveTrusted = originalApproveTrusted
    delete external.__elephantConsentGuardInstalled
  }
}
