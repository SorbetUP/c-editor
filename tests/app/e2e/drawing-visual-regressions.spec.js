const fs = require('node:fs')
const path = require('node:path')
const { test, expect } = require('playwright/test')
const { createSeededVaultFixture, launchElectron } = require('./helpers')

const evidenceRoot = '/tmp/elephant-visual-evidence'

const card = (page, title) => page.locator('.en-note-card').filter({ hasText: title }).first()

const capture = async (page, testInfo, name) => {
  fs.mkdirSync(evidenceRoot, { recursive: true })
  const screenshotPath = path.join(evidenceRoot, `${name}.png`)
  await page.screenshot({ path: screenshotPath, fullPage: false })
  await testInfo.attach(name, { path: screenshotPath, contentType: 'image/png' })
  expect(fs.statSync(screenshotPath).size).toBeGreaterThan(1000)
  return screenshotPath
}

const launchVisualApp = async (fixture) => {
  const launch = await launchElectron([], {
    userDataPath: fixture.userDataPath,
    env: {
      ELEPHANTNOTE_CONFIG_DIR: fixture.configRoot,
      ELEPHANT_E2E_VAULT_ROOT: fixture.vaultRoot,
      ELEPHANTNOTE_MUYA_RUNTIME: 'rust'
    }
  })
  await launch.page.setViewportSize({ width: 1366, height: 900 })
  await launch.page.waitForSelector('.en-library-grid', { state: 'visible', timeout: 30000 })
  return launch
}

const closeVisualApp = async ({ app, fixture }) => {
  await app.close().catch(() => {})
  fs.rmSync(fixture.root, { recursive: true, force: true })
}

test.describe('visual drawing regressions', () => {
  // The Electron harness owns the browser process; Playwright's page fixture is intentionally unused.
  // eslint-disable-next-line no-empty-pattern
  test('visually proves the floating toolbar has its own space above a direct drawing', async ({}, testInfo) => {
    const fixture = await createSeededVaultFixture()
    await fs.promises.copyFile(
      path.join(process.cwd(), 'Elephant/frontend/src/muya/lib/assets/pngicon/image/2.png'),
      path.join(fixture.vaultRoot, 'Visual direct drawing.png')
    )
    await fs.promises.writeFile(
      path.join(fixture.vaultRoot, 'Visual direct drawing.excalidraw'),
      JSON.stringify({
        type: 'excalidraw',
        version: 2,
        source: 'elephantnote-visual-e2e',
        elements: [],
        appState: {},
        files: {}
      }),
      'utf8'
    )

    const context = await launchVisualApp(fixture)
    try {
      const { page } = context
      const toolbarPath = await capture(page, testInfo, 'library-toolbar-hit-area')
      const metrics = await page.evaluate(() => {
        const library = document.querySelector('.en-library')
        const toolbar = document.querySelector('.en-library-toolbar')
        const firstCard = document.querySelector('.en-note-card')
        const sort = document.querySelector('.en-sort-cycle')
        const view = document.querySelector('.en-view-cycle')
        const hitTarget = (element) => {
          const rect = element.getBoundingClientRect()
          return document.elementFromPoint(rect.left + rect.width / 2, rect.top + rect.height / 2)?.closest('button')?.className || ''
        }
        return {
          toolbarTop: toolbar.getBoundingClientRect().top,
          toolbarBottom: toolbar.getBoundingClientRect().bottom,
          firstCardTop: firstCard.getBoundingClientRect().top,
          libraryTop: library.getBoundingClientRect().top,
          sortHit: hitTarget(sort),
          viewHit: hitTarget(view)
        }
      })
      expect(metrics.firstCardTop).toBeGreaterThanOrEqual(metrics.toolbarBottom)
      expect(metrics.firstCardTop - metrics.libraryTop).toBeGreaterThanOrEqual(64)
      expect(metrics.sortHit).toBe('en-sort-cycle')
      expect(metrics.viewHit).toBe('en-view-cycle')
      expect(fs.existsSync(toolbarPath)).toBe(true)

      const folder = page.locator('.en-note-card.is-folder').filter({ hasText: 'Projects' }).first()
      await expect(folder).toBeVisible()
      await expect(folder.locator('[aria-label="Folder contents preview"]')).toBeVisible()
      await expect(folder.locator('.en-folder-preview')).toContainText('Beta')
      const cardHeights = await page.evaluate(() => {
        const cardHeight = (title) => [...document.querySelectorAll('.en-note-card')]
          .find((element) => element.querySelector('h3')?.textContent?.trim() === title)
          ?.getBoundingClientRect().height
        const note = cardHeight('Alpha note')
        const folder = cardHeight('Projects')
        return { note, folder }
      })
      expect(cardHeights.note).toBeGreaterThan(0)
      expect(cardHeights.folder).toBeGreaterThan(0)
      expect(Math.abs(cardHeights.note - cardHeights.folder)).toBeLessThanOrEqual(2)

      const drawing = card(page, 'Visual direct drawing')
      await expect(drawing).toBeVisible()
      await expect(drawing.locator('h3')).not.toContainText('.excalidraw')
      const preview = drawing.locator('img[data-elephant-excalidraw-preview="true"]')
      await expect(preview).toBeVisible()
      await expect.poll(() => preview.evaluate((image) => image.naturalWidth)).toBeGreaterThan(0)
      await capture(page, testInfo, 'direct-drawing-library-preview')
      await drawing.click()
      await expect(page.getByTestId('excalidraw-dialog')).toBeVisible()
      await expect(page.getByTestId('muya-runtime-editor')).toHaveCount(0)
      const excalidrawControls = await page.evaluate(() => {
        const rect = (selector) => document.querySelector(selector)?.getBoundingClientRect()
        const library = [...document.querySelectorAll('button, [role="button"], label')]
          .find((button) => /library/i.test([
            button.textContent,
            button.getAttribute('aria-label'),
            button.getAttribute('title')
          ].filter(Boolean).join(' ')))
        const libraryRect = library?.getBoundingClientRect()
        const closeRect = rect('[data-testid="excalidraw-close"]')
        const saveRect = rect('[data-testid="excalidraw-save"]')
        return {
          closeTop: closeRect?.top ?? null,
          saveTop: saveRect?.top ?? null,
          closeRight: closeRect?.right ?? null,
          saveRight: saveRect?.right ?? null,
          libraryTop: libraryRect?.top ?? null,
          libraryLeft: libraryRect?.left ?? null
        }
      })
      expect(excalidrawControls.libraryTop).not.toBeNull()
      expect(Math.abs(excalidrawControls.closeTop - excalidrawControls.libraryTop)).toBeLessThan(14)
      expect(Math.abs(excalidrawControls.saveTop - excalidrawControls.libraryTop)).toBeLessThan(14)
      expect(excalidrawControls.closeRight).toBeLessThan(excalidrawControls.libraryLeft - 8)
      expect(excalidrawControls.saveRight).toBeLessThan(excalidrawControls.libraryLeft - 8)
      const drawingPath = await capture(page, testInfo, 'direct-drawing-excalidraw-dialog')
      expect(fs.existsSync(drawingPath)).toBe(true)
      await expect(page.getByTestId('excalidraw-dialog').locator('canvas.interactive')).toBeVisible()
    } finally {
      await closeVisualApp({ ...context, fixture })
    }
  })

  // eslint-disable-next-line no-empty-pattern
  test('visually proves a Markdown drawing opens as Excalidraw instead of a note editor', async ({}, testInfo) => {
    const fixture = await createSeededVaultFixture()
    await fs.promises.mkdir(path.join(fixture.vaultRoot, '.assets'), { recursive: true })
    await fs.promises.copyFile(
      path.join(process.cwd(), 'Elephant/frontend/src/muya/lib/assets/pngicon/image/2.png'),
      path.join(fixture.vaultRoot, '.assets', 'visual-markdown-drawing.png')
    )
    await fs.promises.writeFile(
      path.join(fixture.vaultRoot, '.assets', 'visual-markdown-drawing.excalidraw'),
      JSON.stringify({
        type: 'excalidraw',
        version: 2,
        source: 'elephantnote-visual-e2e',
        elements: [],
        appState: {},
        files: {}
      }),
      'utf8'
    )
    await fs.promises.writeFile(
      path.join(fixture.vaultRoot, 'Visual Markdown drawing.md'),
      [
        '---',
        'title: "Visual Markdown drawing"',
        'type: "drawing"',
        '---',
        '',
        '# Visual Markdown drawing',
        '',
        '![Excalidraw: Visual Markdown drawing](.assets/visual-markdown-drawing.png)',
        ''
      ].join('\n'),
      'utf8'
    )

    const context = await launchVisualApp(fixture)
    try {
      const { page } = context
      const drawing = card(page, 'Visual Markdown drawing')
      const preview = drawing.locator('img[data-elephant-excalidraw-preview="true"]')
      await expect(preview).toBeVisible()
      await expect.poll(() => preview.evaluate((image) => image.naturalWidth)).toBeGreaterThan(0)
      await capture(page, testInfo, 'markdown-drawing-library-preview')
      await drawing.click()
      await expect(page.getByTestId('excalidraw-dialog')).toBeVisible()
      await expect(page.getByTestId('muya-runtime-editor')).toHaveCount(0)
      await capture(page, testInfo, 'markdown-drawing-excalidraw-dialog')
      await expect(page.getByTestId('excalidraw-dialog').locator('canvas.interactive')).toBeVisible()
    } finally {
      await closeVisualApp({ ...context, fixture })
    }
  })
})
