const { expect, test } = require('playwright/test')
const { launchElectronWithSeededVault } = require('../helpers')

const getBodyText = async(page) => (await page.locator('body').innerText()).trim()

const getVisibleCardTexts = async(page) => {
  const cards = page.locator('.en-note-card')
  const count = await cards.count()
  const texts = []
  for (let index = 0; index < count; index += 1) {
    const card = cards.nth(index)
    if (await card.isVisible().catch(() => false)) texts.push(await card.innerText())
  }
  return texts
}

test.describe('Native app UI parity contract baseline', () => {
  test('Electron baseline opens seeded vault without a blank page', async() => {
    const { app, page } = await launchElectronWithSeededVault()
    try {
      await expect(page.locator('body')).toBeVisible()
      const bodyText = await getBodyText(page)
      expect(bodyText.length).toBeGreaterThan(0)
      expect(bodyText).toContain('Alpha note')
      await expect(page.locator('#elephant-diagnostic-overlay')).toHaveCount(0)
    } finally {
      await app.close()
    }
  })

  test('Electron baseline does not stay on first vault selection screen', async() => {
    const { app, page } = await launchElectronWithSeededVault()
    try {
      const bodyText = await getBodyText(page)
      expect(bodyText).not.toMatch(/select.*vault|choose.*vault|premi[eè]re.*vault|s[eé]lection/i)
      expect(bodyText).toContain('All notes')
      expect(bodyText).toContain('Alpha note')
    } finally {
      await app.close()
    }
  })

  test('Electron baseline note cards do not expose raw metadata when cards are present', async() => {
    const { app, page } = await launchElectronWithSeededVault()
    try {
      const texts = await getVisibleCardTexts(page)
      expect(texts.join('\n')).toContain('Alpha note')
      for (const text of texts) {
        expect(text).not.toContain('---')
        expect(text.toLowerCase()).not.toContain('type:')
      }
    } finally {
      await app.close()
    }
  })

  test('Electron baseline visible note cards have titles', async() => {
    const { app, page } = await launchElectronWithSeededVault()
    try {
      const cards = page.locator('.en-note-card')
      const count = await cards.count()
      expect(count).toBeGreaterThan(0)
      for (let index = 0; index < count; index += 1) {
        const card = cards.nth(index)
        if (await card.isVisible().catch(() => false)) {
          const title = card.locator('h3')
          await expect(title).toBeVisible()
          expect((await title.innerText()).trim().length).toBeGreaterThan(0)
        }
      }
    } finally {
      await app.close()
    }
  })

  test('Electron baseline can open the seeded note content', async() => {
    const { app, page } = await launchElectronWithSeededVault()
    try {
      const card = page.locator('.en-note-card').filter({ hasText: 'Alpha note' }).first()
      await expect(card).toBeVisible()
      const title = card.locator('h3')
      await expect(title).toHaveText('Alpha note')
      await title.dispatchEvent('click')
      await expect(page.locator('.en-editor-layer')).toBeVisible({ timeout: 10000 })
      await expect(page.getByText('Visible alpha body line.').first()).toBeVisible({ timeout: 10000 })
    } finally {
      await app.close()
    }
  })

  test('Electron baseline never enables experimental Muya active mode by default', async() => {
    const { app, page } = await launchElectronWithSeededVault()
    try {
      const mode = await page.evaluate(() => window.__ELEPHANT_MUYA_RUNTIME__?.mode?.() || window.__ELEPHANT_MUYA_RUNTIME_MODE__ || null)
      expect(mode).not.toBe('active')
    } finally {
      await app.close()
    }
  })
})
