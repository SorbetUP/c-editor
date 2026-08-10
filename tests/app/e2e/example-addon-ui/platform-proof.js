'use strict'

const fs = require('node:fs')
const path = require('node:path')

const addonId = 'com.elephantnote.examples.platform-proof'
const noteRelativePath = 'Addon Proof/External Addon Platform Proof.md'

const run = async ({ app, fixture, harness }) => {
  await harness.openInstalledAddon(app, addonId)
  await harness.assertPermissionDenied(app, addonId, 'Outside/Platform-denied.md')
  await app.command('click', '.en-addon-detail-commands button')
  await harness.waitForText(app, '.en-addons-feedback', 'completed')

  const notePath = path.join(fixture.vaultRoot, ...noteRelativePath.split('/'))
  await harness.retry(() => fs.existsSync(notePath) && fs.readFileSync(notePath, 'utf8'),
    (content) => typeof content === 'string' && content.includes('ELEPHANT_ADDON_PROOF:1:'), 30000)
  const firstRun = fs.readFileSync(notePath, 'utf8')

  await app.restart()
  await harness.waitForAddon(app, addonId, (addon) => addon?.enabled === true && addon.status === 'enabled')
  await harness.openInstalledAddon(app, addonId)
  await app.command('click', '.en-addon-detail-commands button')
  await harness.waitForText(app, '.en-addons-feedback', 'completed')
  await harness.retry(() => fs.existsSync(notePath) && fs.readFileSync(notePath, 'utf8'),
    (content) => typeof content === 'string' && content.includes('ELEPHANT_ADDON_PROOF:2:'), 30000)

  const storagePath = path.join(fixture.vaultRoot, '.elephantnote', 'addons', 'data', addonId, 'storage.json')
  const storage = JSON.parse(fs.readFileSync(storagePath, 'utf8'))
  if (storage.runCount !== 2) throw new Error(`Platform proof storage did not survive reload: ${JSON.stringify(storage)}`)
  return { firstRun, afterReload: fs.readFileSync(notePath, 'utf8'), storage }
}

module.exports = { addonId, hidden: false, run }
