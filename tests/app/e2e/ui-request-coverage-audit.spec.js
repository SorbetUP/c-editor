const fs = require('node:fs')
const path = require('node:path')
const { test, expect } = require('playwright/test')
const { createSeededVaultFixture, launchElectron } = require('./helpers')

const launchAuditApp = async ({ prepareFixture, env = {} } = {}) => {
  const fixture = await createSeededVaultFixture()
  await prepareFixture?.(fixture)
  const launch = await launchElectron([], {
    userDataPath: fixture.userDataPath,
    env: {
      ELEPHANTNOTE_CONFIG_DIR: fixture.configRoot,
      ELEPHANT_E2E_VAULT_ROOT: fixture.vaultRoot,
      ELEPHANTNOTE_MUYA_RUNTIME: 'rust',
      ...env
    }
  })
  await launch.page.setViewportSize({ width: 1366, height: 900 })
  await launch.page.waitForSelector('.en-library-grid', { state: 'visible', timeout: 30000 })
  return { ...launch, fixture }
}

const closeAuditApp = async ({ app, fixture }) => {
  await app.close().catch(() => {})
  fs.rmSync(fixture.root, { recursive: true, force: true })
}

const card = (page, title) => page.locator('.en-note-card').filter({ hasText: title }).first()

const selectRenderedText = async (page, text) => {
  await page.evaluate((value) => {
    const host = document.querySelector('.en-editor-host')
    const textNode = [...(host?.querySelectorAll('*') || [])]
      .map((element) => [...element.childNodes].find((node) => (
        node.nodeType === Node.TEXT_NODE && node.textContent.includes(value)
      )))
      .find(Boolean)
    if (!textNode) throw new Error(`Rendered text was not found: ${value}`)
    const start = textNode.textContent.indexOf(value)
    const range = document.createRange()
    range.setStart(textNode, start)
    range.setEnd(textNode, start + value.length)
    const selection = window.getSelection()
    selection.removeAllRanges()
    selection.addRange(range)
    document.dispatchEvent(new Event('selectionchange', { bubbles: true }))
  }, text)
}

test.describe('real UI request coverage gaps', () => {
  test('applies a selected citation in another note and persists the result', async () => {
    const context = await launchAuditApp({
      prepareFixture: async (fixture) => {
        await fs.promises.writeFile(
          path.join(fixture.vaultRoot, 'Source.md'),
          '# Source\n\nA passage that must be cited.\n',
          'utf8'
        )
        await fs.promises.writeFile(
          path.join(fixture.vaultRoot, 'Target.md'),
          '# Target\n\nWrite the conclusion here.\n',
          'utf8'
        )
      }
    })
    try {
      const { page, fixture } = context
      const rendererErrors = []
      page.on('pageerror', (error) => rendererErrors.push(error?.message || String(error)))
      await card(page, 'Source').click()
      await expect(page.getByTestId('muya-runtime-editor')).toBeVisible()
      await selectRenderedText(page, 'A passage that must be cited.')

      const selectionAction = page.locator('[data-elephant-citation-selection-action="true"]')
      await expect(selectionAction).toBeVisible()
      await selectionAction.click()

      const pendingCitation = page.locator('[data-elephant-citation-buffer-item]').first()
      await expect(pendingCitation).toBeVisible()
      await expect(pendingCitation).toHaveAttribute('data-lucide', 'quote')
      await page.getByRole('button', { name: 'Close note' }).click()

      await card(page, 'Target').click()
      await expect(page.getByTestId('muya-runtime-editor')).toBeVisible()
      await expect(page.locator('[data-elephant-citation-buffer-item]').first()).toBeVisible()
      await page.locator('[data-elephant-citation-buffer-item]').first().click()

      await expect
        .poll(() => fs.readFileSync(path.join(fixture.vaultRoot, 'Target.md'), 'utf8'))
        .toContain('> A passage that must be cited.')
      await expect
        .poll(() => fs.readFileSync(path.join(fixture.vaultRoot, 'Target.md'), 'utf8'))
        .toMatch(/Source\.md#quote=/)
      await expect(page.getByTestId('muya-runtime-editor')).toContainText(
        'A passage that must be cited.'
      )
      expect(rendererErrors).toEqual([])
    } finally {
      await closeAuditApp(context)
    }
  })

  test('collapses and restores the desktop sidebar through the rail control', async () => {
    const context = await launchAuditApp()
    try {
      const { page } = context
      const railToggle = page.locator('.en-rail-sidebar-toggle')
      await expect(railToggle).toHaveAttribute('aria-label', 'Hide sidebar')
      await railToggle.click()
      await expect(railToggle).toHaveAttribute('aria-label', 'Show sidebar')
      await expect(page.locator('.en-body')).toHaveClass(/en-sidebar-hidden/)
      await expect(page.locator('.en-sidebar-resizer')).toBeHidden()

      await railToggle.click()
      await expect(railToggle).toHaveAttribute('aria-label', 'Hide sidebar')
      await expect(page.locator('.en-body')).not.toHaveClass(/en-sidebar-hidden/)
      await expect(page.locator('.en-sidebar')).toBeVisible()
    } finally {
      await closeAuditApp(context)
    }
  })

  test('application topbar stays flat instead of rendering a decorative border', async () => {
    const context = await launchAuditApp()
    try {
      const { page } = context
      const topbar = page.locator('.en-topstrip')
      await expect(topbar).toBeVisible()
      const chrome = await topbar.evaluate((element) => {
        const style = getComputedStyle(element)
        const after = getComputedStyle(element, '::after')
        return {
          height: Number.parseFloat(style.height),
          borderWidth: style.borderWidth,
          borderStyle: style.borderStyle,
          boxShadow: style.boxShadow,
          afterDisplay: after.display,
          afterContent: after.content
        }
      })
      expect(chrome.height).toBeLessThanOrEqual(32)
      expect(chrome.borderWidth).toBe('0px')
      expect(chrome.borderStyle).toBe('none')
      expect(chrome.boxShadow).toBe('none')
      expect(chrome.afterDisplay).toBe('none')
      expect(chrome.afterContent).toBe('none')
    } finally {
      await closeAuditApp(context)
    }
  })

  test('shows installed addon entries in the first catalogue view before addon selection', async () => {
    const context = await launchAuditApp({
      env: {
        ELEPHANT_E2E_OFFICIAL_ADDONS: 'all'
      }
    })
    try {
      const { page } = context
      await page.getByRole('button', { name: 'Settings' }).click()
      await page.locator('.en-settings-nav button').filter({ hasText: 'Addons' }).click()
      await expect(page.locator('.en-addon-catalogue')).toBeVisible()
      await expect(page.locator('.en-addon-tile.installed').first()).toBeVisible()
      await expect(page.locator('.en-addon-tile.installed').first()).toContainText(/Installed|Enabled/)
      await expect(page.getByRole('button', { name: 'Install addon from file' })).toBeEnabled()
    } finally {
      await closeAuditApp(context)
    }
  })
})
