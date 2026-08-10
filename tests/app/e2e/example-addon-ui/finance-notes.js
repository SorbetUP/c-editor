'use strict'

const fs = require('node:fs')
const path = require('node:path')

const addonId = 'com.elephantnote.examples.finance-notes'

const run = async ({ app, fixture, harness }) => {
  await harness.openInstalledAddon(app, addonId)
  await harness.assertPermissionDenied(app, addonId, 'Outside/Finance-denied.md')
  await app.command('click', '.en-addon-detail-commands button')
  const firstFeedback = await harness.retry(
    () => harness.readDom(app, '.en-addons-feedback'),
    (result) => result?.visible && (result.text.includes('completed') || result.text.includes('Finance provider returned HTTP')),
    60000
  )

  if (firstFeedback.text.includes('Finance provider returned HTTP')) {
    const notePath = path.join(fixture.vaultRoot, 'Finance', 'AAPL.md')
    if (fs.existsSync(notePath)) throw new Error('Finance provider failure unexpectedly wrote a note')
    await app.restart()
    await harness.waitForAddon(app, addonId, (addon) => addon?.enabled === true && addon.status === 'enabled')
    await harness.openInstalledAddon(app, addonId)
    await app.command('click', '.en-addon-detail-commands button')
    const afterReload = await harness.retry(
      () => harness.readDom(app, '.en-addons-feedback'),
      (result) => result?.visible && result.text.includes('Finance provider returned HTTP'),
      60000
    )
    return { outcome: 'provider-error-visible', firstFeedback, afterReload }
  }

  const notePath = path.join(fixture.vaultRoot, 'Finance', 'AAPL.md')
  await harness.retry(
    () => fs.existsSync(notePath) && fs.readFileSync(notePath, 'utf8'),
    (content) => typeof content === 'string' && content.includes('Yahoo Finance chart API'),
    60000
  )
  const firstRun = fs.readFileSync(notePath, 'utf8')
  const storagePath = path.join(fixture.vaultRoot, '.elephantnote', 'addons', 'data', addonId, 'storage.json')
  if (!fs.existsSync(storagePath)) throw new Error('Finance addon did not persist private storage')
  const storage = JSON.parse(fs.readFileSync(storagePath, 'utf8'))
  if (storage.lastRun?.path !== 'Finance/AAPL.md') throw new Error(`Unexpected finance storage: ${JSON.stringify(storage)}`)

  await app.restart()
  await harness.waitForAddon(app, addonId, (addon) => addon?.enabled === true && addon.status === 'enabled')
  await harness.openInstalledAddon(app, addonId)
  await app.command('click', '.en-addon-detail-commands button')
  await harness.waitForText(app, '.en-addons-feedback', 'completed', 60000)
  await harness.retry(() => fs.existsSync(notePath) && fs.readFileSync(notePath, 'utf8'),
    (content) => typeof content === 'string' && content.includes('Fetched:'), 60000)

  return { firstRun, afterReload: fs.readFileSync(notePath, 'utf8'), storage }
}

module.exports = { addonId, hidden: false, run }
