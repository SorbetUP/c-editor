'use strict'

const addonId = 'com.elephantnote.examples.trusted-workspace-lab'

const run = async ({ app, harness, installed }) => {
  // The current settings component intentionally hides this legacy bundled id.
  // Enable through the production acceptance bridge, then exercise the real UI
  // contribution and command registered by the trusted addon.
  await harness.setLocalStorage(app, 'elephantnote:addons:trusted-safe-mode', 'false')
  await app.command('invokeTauri', 'tauri_prefs_set', {
    key: 'addons.trustedSafeMode',
    value: false
  })
  await app.command('invokeTauri', 'tauri_prefs_set', {
    key: `addons.trustedApproval.${addonId}`,
    value: installed.packageHash
  })
  try {
    await app.command('enableAddon', addonId)
  } catch (error) {
    const state = await harness.addonState(app)
    const community = await app.command('invokeTauri', 'tauri_prefs_get', { key: 'addons.communityEnabled' })
    const safeMode = await app.command('invokeTauri', 'tauri_prefs_get', { key: 'addons.trustedSafeMode' })
    const approval = await app.command('invokeTauri', 'tauri_prefs_get', { key: `addons.trustedApproval.${addonId}` })
    throw new Error(`${error.message}; state=${JSON.stringify(state)} prefs=${JSON.stringify({ community, safeMode, approval, packageHash: installed.packageHash })}`)
  }
  await harness.waitForAddon(app, addonId, (addon) => addon?.enabled === true && addon.status === 'enabled')
  const beforePermission = await harness.assertPermissionDenied(app, addonId, 'Outside/Trusted-denied.md')
  await app.command('waitFor', '.en-rail button[aria-label="Toggle the trusted focus mode"]', 15000)
  await app.command('click', '.en-rail button[aria-label="Toggle the trusted focus mode"]')
  await harness.retry(
    () => harness.readDom(app, 'html'),
    (result) => result?.attributes?.class?.includes('elephant-trusted-focus'),
    10000
  )
  await new Promise((resolve) => setTimeout(resolve, 500))
  await app.command('click', '.en-rail button[aria-label="Toggle the trusted focus mode"]')
  await harness.retry(
    () => harness.readDom(app, 'html'),
    (result) => !result?.attributes?.class?.includes('elephant-trusted-focus'),
    10000
  )

  await app.restart()
  await harness.waitForAddon(app, addonId, (addon) => addon?.enabled === true && addon.status === 'enabled')
  await app.command('waitFor', '.en-rail button[aria-label="Toggle the trusted focus mode"]', 15000)
  await app.command('click', '.en-rail button[aria-label="Toggle the trusted focus mode"]')
  await harness.retry(
    () => harness.readDom(app, 'html'),
    (result) => result?.attributes?.class?.includes('elephant-trusted-focus'),
    10000
  )
  return { permissionError: beforePermission, reloadedCommand: 'toggle-focus' }
}

module.exports = { addonId, hidden: true, run }
