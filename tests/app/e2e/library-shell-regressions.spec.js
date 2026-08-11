const fs = require('node:fs')
const path = require('node:path')
const { test, expect } = require('playwright/test')
const { createSeededVaultFixture, launchElectron } = require('./helpers')

const launchLibraryApp = async ({ prepareFixture } = {}) => {
  const fixture = await createSeededVaultFixture()
  await prepareFixture?.(fixture)
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
  return { ...launch, fixture }
}

const closeLibraryApp = async ({ app, fixture }) => {
  await app.close().catch(() => {})
  fs.rmSync(fixture.root, { recursive: true, force: true })
}

const card = (page, title) => page.locator('.en-note-card').filter({ hasText: title }).first()

test.describe('library and shell visual contracts', () => {
  test('note, drawing and folder cards do not render update dates and folders stay compact', async () => {
    const context = await launchLibraryApp({
      prepareFixture: async (fixture) => {
        await fs.promises.mkdir(path.join(fixture.vaultRoot, '.assets'), { recursive: true })
        await fs.promises.copyFile(
          path.join(process.cwd(), 'Elephant/frontend/src/muya/lib/assets/pngicon/image/2.png'),
          path.join(fixture.vaultRoot, '.assets', 'library-shell-drawing.png')
        )
        await fs.promises.writeFile(
          path.join(fixture.vaultRoot, 'Library shell drawing.md'),
          [
            '---',
            'title: "Library shell drawing"',
            'type: "drawing"',
            '---',
            '',
            '![Excalidraw: Library shell drawing](.assets/library-shell-drawing.png)',
            ''
          ].join('\n'),
          'utf8'
        )
      }
    })
    try {
      const { page } = context
      for (const title of ['Alpha note', 'Library shell drawing', 'Projects']) {
        const entryCard = card(page, title)
        await expect(entryCard).toBeVisible()
        await expect(entryCard.locator('footer')).toHaveCount(0)
        await expect(entryCard.locator('.en-updated')).toHaveCount(0)
      }

      const folderMetrics = await card(page, 'Projects').evaluate((element) => {
        const style = getComputedStyle(element)
        return {
          height: element.getBoundingClientRect().height,
          minHeight: style.minHeight
        }
      })
      const noteHeight = await card(page, 'Alpha note').evaluate((element) =>
        element.getBoundingClientRect().height
      )
      expect(folderMetrics.minHeight).toBe('176px')
      expect(Math.abs(folderMetrics.height - noteHeight)).toBeLessThanOrEqual(2)
    } finally {
      await closeLibraryApp(context)
    }
  })

  test('sort and view controls float in a reserved hit area above cards and sidebar resize has no artifact line', async () => {
    const context = await launchLibraryApp()
    try {
      const { page } = context
      const metrics = await page.evaluate(() => {
        const library = document.querySelector('.en-library')
        const toolbar = document.querySelector('.en-library-toolbar')
        const grid = document.querySelector('.en-library-grid')
        const firstCard = document.querySelector('.en-note-card')
        const resizer = document.querySelector('[data-sidebar-resizer]')
        const resizerStyle = resizer && getComputedStyle(resizer)
        const resizerHandleStyle = resizer && getComputedStyle(resizer, '::after')
        return {
          libraryPosition: library && getComputedStyle(library).position,
          toolbarPosition: toolbar && getComputedStyle(toolbar).position,
          toolbarZIndex: toolbar && getComputedStyle(toolbar).zIndex,
          gridPaddingTop: grid && Number.parseFloat(getComputedStyle(grid).paddingTop),
          firstCardOffset: library && firstCard &&
            firstCard.getBoundingClientRect().top - library.getBoundingClientRect().top,
          resizerBackground: resizerStyle?.backgroundColor,
          resizerBorder: resizerStyle?.borderRightStyle,
          resizerHandleOpacity: resizerHandleStyle?.opacity,
          resizerHandleDisplay: resizerHandleStyle?.display
        }
      })

      expect(metrics.libraryPosition).toBe('relative')
      expect(metrics.toolbarPosition).toBe('absolute')
      expect(Number(metrics.toolbarZIndex)).toBeGreaterThan(0)
      expect(metrics.gridPaddingTop).toBeGreaterThanOrEqual(64)
      expect(metrics.firstCardOffset).toBeGreaterThanOrEqual(metrics.gridPaddingTop - 1)
      expect(metrics.resizerBackground).toBe('rgba(0, 0, 0, 0)')
      expect(metrics.resizerBorder).toBe('none')
      expect(metrics.resizerHandleOpacity).toBe('0')

      const hitTargets = await page.evaluate(() => {
        const getHitTarget = (selector) => {
          const element = document.querySelector(selector)
          const rect = element?.getBoundingClientRect()
          if (!rect) return ''
          const hit = document.elementFromPoint(rect.left + rect.width / 2, rect.top + rect.height / 2)
          return hit?.closest(selector) ? selector : hit?.className || hit?.tagName || ''
        }
        return {
          sort: getHitTarget('.en-sort-cycle'),
          view: getHitTarget('.en-view-cycle')
        }
      })
      expect(hitTargets.sort).toBe('.en-sort-cycle')
      expect(hitTargets.view).toBe('.en-view-cycle')

      await page.locator('.en-sort-cycle').click()
      await expect(page.locator('.en-sort-cycle')).toHaveAttribute('data-sort', 'updated-oldest')
      await page.locator('.en-view-cycle').click()
      await expect(page.locator('.en-library-grid')).toHaveAttribute('data-view-mode', 'list')

      const resizer = page.locator('[data-sidebar-resizer]')
      await expect(resizer).toHaveAttribute('role', 'separator')
      await expect(resizer).toHaveAttribute('tabindex', '0')
      await resizer.focus()
      await expect(resizer).toHaveAttribute('aria-valuenow', '232')
      await page.keyboard.press('ArrowRight')
      await expect(resizer).toHaveAttribute('aria-valuenow', '248')
      await expect(page.locator('.en-shell')).toHaveAttribute('style', /--en-sidebar-runtime-width: 248px/)
    } finally {
      await closeLibraryApp(context)
    }
  })

  test('flat workspace surfaces keep the sidebar resize control embedded without a second border', async () => {
    const context = await launchLibraryApp()
    try {
      const { page } = context
      const metrics = await page.locator('[data-sidebar-resizer]').evaluate((element) => ({
        background: getComputedStyle(element).backgroundColor,
        border: getComputedStyle(element).borderRightStyle,
        handleOpacity: getComputedStyle(element, '::after').opacity,
        handleDisplay: getComputedStyle(element, '::after').display,
        sidebarBorder: getComputedStyle(document.querySelector('.en-sidebar')).borderRightStyle,
        bodyBorder: getComputedStyle(document.querySelector('.en-body-main')).borderRightStyle,
        railShadow: getComputedStyle(document.querySelector('.en-rail')).boxShadow,
        sidebarShadow: getComputedStyle(document.querySelector('.en-sidebar')).boxShadow,
        bodyShadow: getComputedStyle(document.querySelector('.en-body-main')).boxShadow
      }))
      expect(metrics.background).toBe('rgba(0, 0, 0, 0)')
      expect(metrics.border).toBe('none')
      expect(metrics.handleOpacity).toBe('0')
      expect(metrics.handleDisplay).toBe('none')
      expect(metrics.sidebarBorder).toBe('solid')
      expect(metrics.bodyBorder).toBe('none')
      expect(metrics.railShadow).toBe('none')
      expect(metrics.sidebarShadow).toBe('none')
      expect(metrics.bodyShadow).toBe('none')
    } finally {
      await closeLibraryApp(context)
    }
  })
})
