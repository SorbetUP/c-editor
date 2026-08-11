import { chromium } from 'playwright'
import { test } from 'node:test'
import assert from 'node:assert/strict'
import { optionalText } from '../observations.mjs'

test('an absent optional observation returns without the Playwright default timeout', async () => {
  const browser = await chromium.launch({ headless: true })
  try {
    const page = await browser.newPage()
    await page.setContent('<main aria-label="fixture without vault title"></main>')
    const started = performance.now()
    const value = await optionalText(page.locator('.en-top-vault-name'))
    const elapsedMs = performance.now() - started
    assert.equal(value, '')
    assert.ok(elapsedMs < 1000, `absent optional observation took ${elapsedMs.toFixed(1)}ms`)
  } finally {
    await browser.close()
  }
})
