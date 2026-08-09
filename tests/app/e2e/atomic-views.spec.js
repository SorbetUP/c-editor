const { expect, test } = require('playwright/test')
const { launchElectron } = require('./helpers')

test.describe('Core workspace visual smoke', () => {
  let app = null
  let page = null

  test.beforeEach(async() => {
    const launched = await launchElectron()
    app = launched.app
    page = launched.page
  })

  test.afterEach(async() => {
    await app?.close()
  })

  test('does not expose optional Chat, Canvas or Graph surfaces without their packages', async() => {
    await expect(page.locator('body')).toBeVisible()
    await expect(page.getByRole('button', { name: 'Chat', exact: true })).toHaveCount(0)
    await expect(page.getByRole('button', { name: 'Canvas', exact: true })).toHaveCount(0)
    await expect(page.getByRole('button', { name: 'Graph', exact: true })).toHaveCount(0)
    await expect(page.locator('.en-canvas-stage')).toHaveCount(0)
    await expect(page.locator('.elephant-chat-package')).toHaveCount(0)
    await expect(page.locator('.elephant-graph-package')).toHaveCount(0)

    const bodyText = (await page.locator('body').innerText()).trim()
    expect(bodyText.length).toBeGreaterThan(20)
    expect(bodyText).toMatch(/Choose your first vault|All notes|Stockage privé/i)
  })
})
