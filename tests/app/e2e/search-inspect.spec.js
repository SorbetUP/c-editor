const fs = require('fs-extra')
const { expect, test } = require('playwright/test')
const { launchElectronWithSeededVault } = require('./helpers')

test.describe('Elephant optional Search boundary', () => {
  let app = null
  let page = null
  let fixture = null

  test.beforeAll(async() => {
    const launched = await launchElectronWithSeededVault()
    app = launched.app
    page = launched.page
    fixture = launched.fixture
  })

  test.afterAll(async() => {
    await app?.close()
    await fs.remove(fixture?.root)
  })

  test('does not expose semantic inspection without the Search addon', async() => {
    const result = await page.evaluate(async() => {
      try {
        const response = await window.elephantnote.api.call('search.inspect')
        return {
          ok: response?.ok !== false,
          error: response?.error?.message || ''
        }
      } catch (error) {
        return { ok: false, error: error?.message || String(error) }
      }
    })

    expect(result.ok).toBe(false)
    expect(result.error).toMatch(/unknown|unsupported|unavailable|not implemented|undefined/i)
  })
})
