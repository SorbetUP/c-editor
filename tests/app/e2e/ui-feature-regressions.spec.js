const fs = require('node:fs')
const path = require('node:path')
const { test, expect } = require('playwright/test')
const { createSeededVaultFixture, launchElectron } = require('./helpers')

const launchFeatureApp = async ({ prepareFixture, extraEnv = {} } = {}) => {
  const fixture = await createSeededVaultFixture()
  await prepareFixture?.(fixture)
  const resolvedExtraEnv = typeof extraEnv === 'function' ? await extraEnv(fixture) : extraEnv
  const launch = await launchElectron([], {
    userDataPath: fixture.userDataPath,
    env: {
      ELEPHANTNOTE_CONFIG_DIR: fixture.configRoot,
      ELEPHANT_E2E_VAULT_ROOT: fixture.vaultRoot,
      ELEPHANTNOTE_MUYA_RUNTIME: 'rust',
      ...resolvedExtraEnv
    }
  })
  await launch.page.setViewportSize({ width: 1366, height: 900 })
  await launch.page.waitForSelector('.en-library-grid', { state: 'visible', timeout: 30000 })
  return { ...launch, fixture }
}

const closeFeatureApp = async ({ app, fixture }) => {
  await app.close().catch(() => {})
  fs.rmSync(fixture.root, { recursive: true, force: true })
}

const card = (page, title) => page.locator('.en-note-card').filter({ hasText: title }).first()

test.describe('production UI regression paths', () => {
  test('creation menu exposes the real Excalidraw asset and every entry type', async () => {
    const context = await launchFeatureApp()
    try {
      const { page } = context
      const create = page.getByRole('button', { name: 'Create', exact: true })
      const createBox = await create.boundingBox()
      expect(createBox).not.toBeNull()
      expect(createBox.width).toBeGreaterThanOrEqual(56)
      expect(Math.abs(createBox.width - createBox.height)).toBeLessThanOrEqual(1)
      await create.click()
      const menu = page.getByRole('menu', { name: 'Create' })
      await expect(menu).toBeVisible()
      await expect(menu.getByRole('menuitem', { name: /Note/ })).toBeVisible()
      await expect(menu.getByRole('menuitem', { name: /Folder/ })).toBeVisible()
      const drawing = menu.getByRole('menuitem', { name: /Drawing/ })
      await expect(drawing).toBeVisible()
      await expect(drawing.locator('img[data-testid="excalidraw-logo"]')).toHaveAttribute(
        'data-excalidraw-asset',
        'shared-muya-icon'
      )
      await expect(drawing.locator('img[data-testid="excalidraw-logo"]')).toHaveAttribute(
        'src',
        /^data:image\/svg\+xml/
      )
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('creation menu executes folder and note creation through the production path', async () => {
    const context = await launchFeatureApp()
    try {
      const { page, fixture } = context
      const create = page.getByRole('button', { name: 'Create', exact: true })
      const foldersBefore = fs.readdirSync(fixture.vaultRoot, { withFileTypes: true })
        .filter((entry) => entry.isDirectory() && !entry.name.startsWith('.'))
        .length

      await create.click()
      await page.getByRole('menu', { name: 'Create' }).getByRole('menuitem', { name: /Folder/ }).click()
      await expect.poll(() => fs.readdirSync(fixture.vaultRoot, { withFileTypes: true })
        .filter((entry) => entry.isDirectory() && !entry.name.startsWith('.'))
        .length).toBe(foldersBefore + 1)

      await create.click()
      await page.getByRole('menu', { name: 'Create' }).getByRole('menuitem', { name: /Note/ }).click()
      await expect(page.getByTestId('muya-runtime-editor')).toBeVisible()
      await expect.poll(() => fs.readdirSync(fixture.vaultRoot).some((name) => /\.md$/i.test(name) && name !== 'Alpha.md')).toBe(true)
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('library exposes icon-only sorting and distinct grid/list layouts', async () => {
      const context = await launchFeatureApp({
        prepareFixture: async (fixture) => {
          await fs.promises.writeFile(
            path.join(fixture.vaultRoot, 'Beta root.md'),
            '# Beta root\n\nBeta body.\n',
            'utf8'
          )
          await fs.promises.writeFile(
            path.join(fixture.vaultRoot, 'Zulu root.md'),
            '# Zulu root\n\nZulu body.\n',
            'utf8'
          )
          const now = Date.now()
          await fs.promises.utimes(path.join(fixture.vaultRoot, 'Alpha.md'), now / 1000, now / 1000)
          await fs.promises.utimes(path.join(fixture.vaultRoot, 'Zulu root.md'), (now - 60_000) / 1000, (now - 60_000) / 1000)
          await fs.promises.utimes(path.join(fixture.vaultRoot, 'Beta root.md'), (now - 120_000) / 1000, (now - 120_000) / 1000)
        }
      })
    try {
      const { page } = context
      const sortButton = page.locator('.en-sort-cycle')
      await expect(sortButton).toHaveCount(1)
      await expect(sortButton.locator('svg')).toBeVisible()
      await expect(sortButton).toHaveText('')
      await expect(sortButton).toHaveAttribute('data-sort', 'updated-newest')
      for (const expected of ['updated-oldest', 'title-az', 'title-za', 'updated-newest']) {
        await sortButton.click()
        await expect(sortButton).toHaveAttribute('data-sort', expected)
      }

      const library = page.locator('.en-library-grid')
      const viewButton = page.locator('.en-view-cycle')
      await expect(viewButton).toHaveCount(1)
      await expect(viewButton.locator('svg')).toBeVisible()
      await expect(viewButton).toHaveText('')

      const inspectLayout = () =>
        page.evaluate(() => {
          const grid = document.querySelector('.en-library-grid')
          const surface = grid?.querySelector('[data-layout]')
          const cards = [...document.querySelectorAll('.en-note-card')].map((element) => {
            const rect = element.getBoundingClientRect()
            return {
              x: rect.x,
              width: rect.width,
              height: rect.height,
              paragraph: element.querySelector('p')
                ? getComputedStyle(element.querySelector('p')).display
                : 'none',
              folder: element.classList.contains('is-folder')
            }
          })
          const style = getComputedStyle(surface || grid)
          return {
            display: style.display,
            width: (surface || grid).getBoundingClientRect().width,
            paddingLeft: Number.parseFloat(style.paddingLeft),
            paddingRight: Number.parseFloat(style.paddingRight),
            columns: style.gridTemplateColumns.split(' ').filter(Boolean).length,
            cards
          }
        })

      const gridLayout = await inspectLayout()
      expect(gridLayout.display).toBe('grid')
      expect(gridLayout.columns).toBeGreaterThan(1)
      expect(gridLayout.cards[0].x).not.toBe(gridLayout.cards[1].x)
      expect(gridLayout.cards.filter((card) => !card.folder).every((card) => card.height >= 160)).toBe(true)
      expect(gridLayout.cards.filter((card) => card.folder).every((card) => card.height >= 160)).toBe(true)
      expect(gridLayout.cards.filter((card) => !card.folder).every((card) => card.paragraph === 'block')).toBe(true)
      expect(gridLayout.cards.filter((card) => card.folder).every((card) => card.paragraph === 'none')).toBe(true)

      const nonFolderTitles = () => page.locator('.en-note-card:not(.is-folder) h3').allTextContents()
      await sortButton.click()
      await expect(sortButton).toHaveAttribute('data-sort', 'updated-oldest')
      await expect.poll(nonFolderTitles).toEqual(['Beta root', 'Zulu root', 'Alpha note'])
      await sortButton.click()
      await expect(sortButton).toHaveAttribute('data-sort', 'title-az')
      await expect.poll(nonFolderTitles).toEqual(['Alpha note', 'Beta root', 'Zulu root'])
      await sortButton.click()
      await expect(sortButton).toHaveAttribute('data-sort', 'title-za')
      await expect.poll(nonFolderTitles).toEqual(['Zulu root', 'Beta root', 'Alpha note'])
      await sortButton.click()
      await expect(sortButton).toHaveAttribute('data-sort', 'updated-newest')
      await expect.poll(nonFolderTitles).toEqual(['Alpha note', 'Zulu root', 'Beta root'])

      await viewButton.click()
      await expect(library).toHaveAttribute('data-view-mode', 'list')
      await expect(library).toHaveClass(/list/)
      const listLayout = await inspectLayout()
      const listContentWidth = listLayout.width - listLayout.paddingLeft - listLayout.paddingRight
      expect(listLayout.display).toBe('flex')
      expect(listLayout.cards.every((card) => card.width >= listContentWidth - 2)).toBe(true)
      expect(listLayout.cards.every((card) => card.height <= 100)).toBe(true)
      expect(listLayout.cards.every((card) => card.paragraph === 'none')).toBe(true)

      await viewButton.click()
      await expect(library).toHaveAttribute('data-view-mode', 'grid')
      await expect(library).not.toHaveClass(/list/)
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('drawing entries render a preview and open directly in Excalidraw', async () => {
    const context = await launchFeatureApp({
      prepareFixture: async (fixture) => {
        await fs.promises.mkdir(path.join(fixture.vaultRoot, '.assets'), { recursive: true })
        await fs.promises.copyFile(
          path.join(process.cwd(), 'Elephant/frontend/src/muya/lib/assets/pngicon/image/2.png'),
          path.join(fixture.vaultRoot, '.assets', 'excalidraw-drawing-preview.png')
        )
        await fs.promises.writeFile(
          path.join(fixture.vaultRoot, 'Drawing.md'),
          [
            '---',
            'title: "Drawing"',
            'type: "drawing"',
            '---',
            '',
            '# Drawing',
            '',
            '![Excalidraw: Plan](.assets/excalidraw-drawing-preview.png)',
            ''
          ].join('\n'),
          'utf8'
        )
      }
    })
    try {
      const { page } = context
      const drawing = card(page, 'Drawing')
      await expect(drawing).toBeVisible()
      const preview = drawing.locator('img[data-elephant-excalidraw-preview="true"]')
      await expect(preview).toBeVisible()
      await expect(preview).toHaveAttribute('alt', 'Drawing preview')
      await expect(preview).toHaveJSProperty('naturalWidth', 40)
      await drawing.click()
      await expect(page.getByTestId('excalidraw-dialog')).toBeVisible()
      await expect(page.getByTestId('muya-runtime-editor')).toHaveCount(0)
      await expect(page.getByTestId('excalidraw-dialog').locator('canvas.interactive')).toBeVisible()
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('creates, names, saves and reopens a drawing through its edit control', async () => {
    const context = await launchFeatureApp()
    try {
      const { page, fixture } = context
      const drawingTitle = 'E2E-Saved-Drawing'
      const rendererErrors = []
      page.on('pageerror', (error) => rendererErrors.push(error?.message || String(error)))
      const create = page.getByRole('button', { name: 'Create', exact: true })

      await create.click()
      const menu = page.getByRole('menu', { name: 'Create' })
      await menu.getByRole('menuitem', { name: /Drawing/ }).click()

      const dialog = page.getByTestId('excalidraw-dialog')
      await expect(dialog).toBeVisible()
      const actionBox = await dialog.locator('.en-excalidraw-actions').boundingBox()
      expect(actionBox).not.toBeNull()
      expect(actionBox.y).toBeGreaterThanOrEqual(0)
      await expect(dialog.locator('.en-excalidraw-canvas canvas.interactive')).toBeVisible()
      await expect(page.locator('#elephant-diagnostic-overlay')).toHaveCount(0)
      expect(rendererErrors).toEqual([])
      await dialog.getByTestId('excalidraw-close').click()

      const namePrompt = page.getByTestId('excalidraw-name-prompt')
      await expect(namePrompt).toBeVisible()
      await namePrompt.getByRole('textbox', { name: 'Drawing name' }).fill(drawingTitle)
      await namePrompt.getByRole('button', { name: 'Save', exact: true }).click()
      await expect(dialog).toBeHidden()
      await expect(page.locator('.en-library-grid')).toBeVisible()

      const notePath = path.join(fixture.vaultRoot, `${drawingTitle}.md`)
      const assetsPath = path.join(fixture.vaultRoot, '.assets')
      await expect.poll(() => fs.existsSync(notePath)).toBe(true)
      await expect
        .poll(() => fs.readFileSync(notePath, 'utf8'))
        .toContain(`title: "${drawingTitle}"`)
      await expect
        .poll(() => fs.readFileSync(notePath, 'utf8'))
        .toContain('type: "drawing"')
      await expect
        .poll(() => fs.readFileSync(notePath, 'utf8'))
        .toContain(`](./.assets/excalidraw-${drawingTitle.replaceAll(' ', '%20')}.png)`)
      await expect
        .poll(() =>
          fs
            .readdirSync(assetsPath)
            .some((name) => name.startsWith(`excalidraw-${drawingTitle}`) && name.endsWith('.png'))
        )
        .toBe(true)

      const savedDrawing = card(page, drawingTitle)
      await expect(savedDrawing).toBeVisible()
      await expect(savedDrawing.locator('.en-note-card-drawing-preview img')).toBeVisible()
      await savedDrawing.click()
      await expect(dialog).toBeVisible()
      await expect(page.getByTestId('muya-runtime-editor')).toHaveCount(0)
      await expect(dialog.locator('.en-excalidraw-canvas canvas.interactive')).toBeVisible()
      await expect(page.locator('#elephant-diagnostic-overlay')).toHaveCount(0)
      expect(rendererErrors).toEqual([])
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('opens a direct Excalidraw file from the library without mounting Muya', async () => {
    const context = await launchFeatureApp({
      prepareFixture: async (fixture) => {
        await fs.promises.copyFile(
          path.join(process.cwd(), 'Elephant/frontend/src/muya/lib/assets/pngicon/image/2.png'),
          path.join(fixture.vaultRoot, 'Direct drawing.png')
        )
        await fs.promises.writeFile(
          path.join(fixture.vaultRoot, 'Direct drawing.excalidraw'),
          JSON.stringify({
            type: 'excalidraw',
            version: 2,
            source: 'elephantnote-e2e',
            elements: [],
            appState: {},
            files: {}
          }),
          'utf8'
        )
      }
    })
    try {
      const { page } = context
      const drawing = card(page, 'Direct drawing')
      await expect(drawing).toBeVisible()
      await drawing.click()
      await expect(page.getByTestId('excalidraw-dialog')).toBeVisible()
      await expect(page.getByTestId('muya-runtime-editor')).toHaveCount(0)
      await expect(page.getByTestId('excalidraw-dialog').locator('canvas.interactive')).toBeVisible()
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('keeps the rail, workspace and note editor on flat structural surfaces', async () => {
    const context = await launchFeatureApp()
    try {
      const { page } = context
      const structuralSurfaceMetrics = (selector) => page.locator(selector).evaluate((element) => {
        const style = getComputedStyle(element)
        return {
          boxShadow: style.boxShadow,
          borderRadius: style.borderRadius,
          margin: style.margin
        }
      })

      for (const selector of ['.en-rail', '.en-sidebar', '.en-body-main']) {
        const metrics = await structuralSurfaceMetrics(selector)
        expect(metrics.boxShadow).toBe('none')
        expect(metrics.borderRadius).toBe('0px')
        expect(metrics.margin).toBe('0px')
      }

      await card(page, 'Alpha note').click()
      await expect(page.locator('.en-note-editor-shell')).toBeVisible()
      const noteMetrics = await structuralSurfaceMetrics('.en-note-editor-shell')
      expect(noteMetrics.boxShadow).toBe('none')
      expect(noteMetrics.borderRadius).toBe('0px')
      expect(noteMetrics.margin).toBe('0px')
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('vault control stays above Settings and supports opening, hiding and restoring the vault rail item', async () => {
    const context = await launchFeatureApp()
    try {
      const { page } = context
      const bottomRail = page.locator('.en-rail-bottom')
      const vaultButton = bottomRail.locator('.en-rail-vault')
      await expect(vaultButton).toBeVisible()
      await expect(bottomRail.locator('button')).toHaveCount(2)
      await expect(bottomRail.locator('button').nth(0)).toHaveClass(/en-rail-vault/)
      await expect(bottomRail.locator('button').nth(1)).toHaveAttribute('aria-label', 'Settings')

      await vaultButton.click()
      await expect(page.locator('.en-vault-menu')).toBeVisible()
      await expect(page.locator('.en-vault-menu-select')).toHaveText('E2E Vault')
      await expect(page.getByRole('button', { name: 'Add another vault' })).toBeVisible()

      await page.getByRole('button', { name: 'Settings' }).click()
      const hideVault = page.getByRole('button', { name: 'Hide Vault' })
      await hideVault.scrollIntoViewIfNeeded()
      await hideVault.click()
      await expect(page.locator('.en-rail-vault')).toBeHidden()
      const showVault = page.getByRole('button', { name: 'Show Vault' })
      await showVault.scrollIntoViewIfNeeded()
      await showVault.click()
      await expect(page.locator('.en-rail-vault')).toBeVisible()
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('vault menu can add, activate and remove a second vault through the real bridge', async () => {
    const context = await launchFeatureApp({
      prepareFixture: async (fixture) => {
        fixture.secondaryVault = path.join(fixture.root, 'Second Vault')
        await fs.promises.mkdir(fixture.secondaryVault, { recursive: true })
        await fs.promises.writeFile(
          path.join(fixture.secondaryVault, 'Second.md'),
          '# Second vault\n',
          'utf8'
        )
      },
      extraEnv: (fixture) => ({ ELEPHANT_E2E_VAULT_PICK_PATH: fixture.secondaryVault })
    })
    try {
      const { page, fixture } = context
      await page.locator('.en-rail-vault').click()
      await page.getByRole('button', { name: 'Add another vault' }).click()
      await expect(page.locator('.en-vault-menu-select')).toHaveCount(2)
      await expect(page.locator('.en-vault-menu-item.active')).toContainText('Second Vault')
      await expect(card(page, 'Second')).toBeVisible()

      await page.locator('.en-vault-menu-select').filter({ hasText: 'E2E Vault' }).click()
      await expect(card(page, 'Alpha note')).toBeVisible()

      await page.getByRole('button', { name: 'Settings' }).click()
      await page.locator('.en-settings-nav button').filter({ hasText: 'Vaults' }).click()
      const secondVaultRow = page.locator('.en-vault-row').filter({ hasText: 'Second Vault' })
      await expect(secondVaultRow).toBeVisible()
      page.once('dialog', (dialog) => dialog.accept())
      await secondVaultRow.getByRole('button', { name: 'Remove' }).click()
      await expect(secondVaultRow).toHaveCount(0)
      await expect
        .poll(
          () =>
            JSON.parse(fs.readFileSync(path.join(fixture.configRoot, 'elephantnote.json'), 'utf8'))
              .vaults
        )
        .toHaveLength(1)
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('addon catalogue is directly accessible without a title-checkbox gate', async () => {
    const context = await launchFeatureApp()
    try {
      const { page } = context
      await page.getByRole('button', { name: 'Settings' }).click()
      await page.locator('.en-settings-nav button').filter({ hasText: 'Addons' }).click()
      await expect(page.locator('.en-addon-catalogue')).toBeVisible()
      await expect(page.locator('#en-community-title-check')).toHaveCount(0)
      await expect(page.locator('.en-addons-panel [role="checkbox"]')).toHaveCount(0)
      await expect(page.getByRole('button', { name: 'Install addon from file' })).toBeEnabled()
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('addon catalogue keeps installed content visible and reports an offline remote catalogue', async () => {
    const context = await launchFeatureApp({ extraEnv: { ELEPHANT_E2E_CATALOG_OFFLINE: '1' } })
    try {
      const { page } = context
      await page.getByRole('button', { name: 'Settings' }).click()
      await page.locator('.en-settings-nav button').filter({ hasText: 'Addons' }).click()
      await expect(page.locator('.en-addon-catalogue')).toBeVisible()
      await expect(page.locator('.en-addons-empty.error')).toContainText(/unavailable|offline/i, {
        timeout: 12000
      })
      await expect(page.getByRole('button', { name: 'Install addon from file' })).toBeEnabled()
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('rail icons can be reordered directly with a drop on another rail icon', async () => {
    const context = await launchFeatureApp()
    try {
      const { page } = context
      const orderBefore = await page
        .locator('.en-rail-nav .en-rail-icon')
        .evaluateAll((buttons) => buttons.map((button) => button.getAttribute('aria-label')))
      expect(orderBefore).toContain('Search')
      const source = page.locator('.en-rail-nav .en-rail-icon[aria-label="Search"]')
      const target = page.locator('.en-rail-nav .en-rail-sidebar-toggle')
      await page.evaluate(() => {
        const source = document.querySelector('.en-rail-nav .en-rail-icon[aria-label="Search"]')
        const target = document.querySelector('.en-rail-nav .en-rail-sidebar-toggle')
        if (!source || !target) throw new Error('Rail drag source or target is missing')
        const transfer = new DataTransfer()
        transfer.effectAllowed = 'move'
        source.dispatchEvent(
          new DragEvent('dragstart', { bubbles: true, cancelable: true, dataTransfer: transfer })
        )
        target.dispatchEvent(
          new DragEvent('dragover', { bubbles: true, cancelable: true, dataTransfer: transfer })
        )
        target.dispatchEvent(
          new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: transfer })
        )
        source.dispatchEvent(
          new DragEvent('dragend', { bubbles: true, cancelable: true, dataTransfer: transfer })
        )
      })
      await expect
        .poll(async () =>
          page.locator('.en-rail-nav .en-rail-icon').evaluateAll((buttons) => {
            const order = buttons.map((button) => button.getAttribute('aria-label'))
            const searchIndex = order.indexOf('Search')
            const sidebarIndex = order.findIndex((label) => /sidebar/i.test(label || ''))
            return searchIndex !== -1 && sidebarIndex !== -1 && searchIndex > sidebarIndex
          })
        )
        .toBe(true)
      await expect(source).toBeVisible()
      await expect(target).toBeVisible()
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('folder rename is inline and supports Escape, outside click and Enter', async () => {
    const context = await launchFeatureApp()
    try {
      const { page } = context
      const projects = card(page, 'Projects')
      await projects.getByRole('button', { name: 'Folder actions' }).click()
      await projects.getByRole('button', { name: 'Rename' }).click()
      const input = page.locator('[data-entry-rename-input]')
      await expect(input).toBeFocused()
      await input.fill('Cancelled folder')
      await input.press('Escape')
      await expect(projects.locator('h3')).toHaveText('Projects')

      await projects.getByRole('button', { name: 'Folder actions' }).click()
      await projects.getByRole('button', { name: 'Rename' }).click()
      await input.fill('Renamed folder')
      // The toolbar is intentionally a pointer-transparent floating layer. Click
      // its empty area through the library surface to exercise outside-click
      // cancellation without bypassing browser hit testing.
      await page.locator('.en-library-grid').click({ position: { x: 100, y: 10 } })
      await expect(projects.locator('h3')).toHaveText('Projects')

      await projects.locator('h3').dblclick()
      await input.fill('Projects renamed')
      await input.press('Enter')
      await expect(page.locator('.en-note-card h3', { hasText: 'Projects renamed' })).toBeVisible()
      await expect(fs.existsSync(path.join(context.fixture.vaultRoot, 'Projects renamed'))).toBe(
        true
      )
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('folder visibility action removes and restores the folder in the sidebar', async () => {
    const context = await launchFeatureApp()
    try {
      const { page } = context
      const projects = card(page, 'Projects')
      await projects.getByRole('button', { name: 'Folder actions' }).click()
      const hide = projects.locator('[data-entry-action="sidebar"]')
      await expect(hide).toHaveAttribute('title', 'Hide from sidebar')
      await hide.click()
      await expect(
        page.locator('.en-sidebar-tree-row').filter({ hasText: 'Projects' })
      ).toHaveCount(0)

      await projects.getByRole('button', { name: 'Folder actions' }).click()
      const show = projects.locator('[data-entry-action="sidebar"]')
      await expect(show).toHaveAttribute('title', 'Show in sidebar')
      await show.click()
      await expect(
        page.locator('.en-sidebar-tree-row').filter({ hasText: 'Projects' })
      ).toHaveCount(1)
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('mobile navigation can be opened and minimized again from the same control', async () => {
    const context = await launchFeatureApp()
    try {
      const { page } = context
      await page.setViewportSize({ width: 720, height: 900 })
      const navigation = page.locator('.en-mobile-topbar button[aria-label="Open navigation"]')
      await expect(navigation).toBeVisible()
      await navigation.click()
      await expect(page.locator('.en-mobile-scrim.visible')).toBeVisible()
      const closeNavigation = page.locator(
        '.en-mobile-topbar button[aria-label="Close navigation"]'
      )
      await expect(closeNavigation).toBeVisible()
      await closeNavigation.click()
      await expect(page.locator('.en-mobile-scrim.visible')).toHaveCount(0)
      await page.locator('.en-mobile-topbar button[aria-label="Open navigation"]').click()
      await expect(page.locator('.en-mobile-scrim.visible')).toBeVisible()
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('a note can be moved from a folder to All notes through the production drop path', async () => {
    const context = await launchFeatureApp()
    try {
      const { page, fixture } = context
      await card(page, 'Projects').click()
      await expect(page.locator('.en-library-grid')).toHaveAttribute('data-view-mode', 'grid')
      const beta = card(page, 'Beta project')
      await expect(beta).toBeVisible()
      const sidebarBeta = page
        .locator('.en-sidebar-tree-note')
        .filter({ hasText: 'Beta project' })
        .first()
      await expect(sidebarBeta).toBeVisible()
      await page.evaluate(
        (entry) => {
          const source = [...document.querySelectorAll('.en-sidebar-tree-note')].find((element) =>
            element.textContent.includes(entry.title)
          )
          const target = document.querySelector('.en-all-notes')
          if (!source || !target)
            throw new Error('Sidebar drag source or All notes target is missing')
          const transfer = new DataTransfer()
          transfer.effectAllowed = 'move'
          transfer.setData('application/x-elephantnote-entry', JSON.stringify(entry))
          source.dispatchEvent(
            new DragEvent('dragstart', { bubbles: true, cancelable: true, dataTransfer: transfer })
          )
          target.dispatchEvent(
            new DragEvent('dragenter', { bubbles: true, cancelable: true, dataTransfer: transfer })
          )
          target.dispatchEvent(
            new DragEvent('dragover', { bubbles: true, cancelable: true, dataTransfer: transfer })
          )
          target.dispatchEvent(
            new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: transfer })
          )
          source.dispatchEvent(
            new DragEvent('dragend', { bubbles: true, cancelable: true, dataTransfer: transfer })
          )
        },
        {
          kind: 'note',
          type: 'note',
          path: 'Projects/Beta.md',
          title: 'Beta project',
          preview: 'Beta project Visible beta body line.'
        }
      )
      await expect
        .poll(() => fs.existsSync(path.join(fixture.vaultRoot, 'Projects', 'Beta.md')))
        .toBe(false)
      await expect.poll(() => fs.existsSync(path.join(fixture.vaultRoot, 'Beta.md'))).toBe(true)
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('editor keeps the writing surface narrow and exposes the citation action after selection', async () => {
    const context = await launchFeatureApp()
    try {
      const { page } = context
      await card(page, 'Alpha note').click()
      const editor = page.getByTestId('muya-runtime-editor')
      await expect(editor).toBeVisible()
      const padding = await page
        .locator('.en-editor-host .editor-component')
        .evaluate((element) => {
          const style = getComputedStyle(element)
          return {
            left: Number.parseFloat(style.paddingLeft),
            right: Number.parseFloat(style.paddingRight)
          }
        })
      expect(padding.left).toBeLessThanOrEqual(16)
      expect(padding.right).toBeLessThanOrEqual(16)
      const writingSurface = await page
        .locator('.en-editor-host #ag-editor-id')
        .evaluate((element) => ({
          width: element.getBoundingClientRect().width,
          hostWidth: element.parentElement?.getBoundingClientRect().width || 0,
          left: Number.parseFloat(getComputedStyle(element).paddingLeft),
          right: Number.parseFloat(getComputedStyle(element).paddingRight)
        }))
      expect(writingSurface.width).toBeGreaterThanOrEqual(writingSurface.hostWidth * 0.9)
      expect(writingSurface.left).toBeLessThanOrEqual(16)
      expect(writingSurface.right).toBeLessThanOrEqual(16)

      await page.evaluate(() => {
        const editorHost = document.querySelector('.en-editor-host')
        const textNode = [...editorHost.querySelectorAll('*')]
          .map((element) =>
            [...element.childNodes].find(
              (node) =>
                node.nodeType === Node.TEXT_NODE &&
                node.textContent.includes('Visible alpha body line.')
            )
          )
          .find(Boolean)
        if (!textNode) throw new Error('Fixture text was not rendered in the editor')
        const range = document.createRange()
        range.selectNodeContents(textNode)
        const selection = window.getSelection()
        selection.removeAllRanges()
        selection.addRange(range)
        document.dispatchEvent(new Event('selectionchange', { bubbles: true }))
      })
      await expect(page.locator('[data-elephant-citation-selection-action="true"]')).toBeVisible()
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('citation stays available after navigation and inserts at the last line without a target selection', async () => {
    const context = await launchFeatureApp()
    try {
      const { page, fixture } = context
      await page.evaluate(() => {
        Object.defineProperty(navigator, 'clipboard', {
          configurable: true,
          value: { writeText: async () => {} }
        })
      })

      await card(page, 'Alpha note').click()
      await expect(page.getByTestId('muya-runtime-editor')).toBeVisible()
      await page.evaluate(() => {
        const editorHost = document.querySelector('.en-editor-host')
        const textNode = [...editorHost.querySelectorAll('*')]
          .map((element) =>
            [...element.childNodes].find(
              (node) =>
                node.nodeType === Node.TEXT_NODE &&
                node.textContent.includes('Visible alpha body line.')
            )
          )
          .find(Boolean)
        if (!textNode) throw new Error('Citation source text was not rendered')
        const range = document.createRange()
        const start = textNode.textContent.indexOf('Visible alpha body line.')
        range.setStart(textNode, start)
        range.setEnd(textNode, start + 'Visible alpha body line.'.length)
        const selection = window.getSelection()
        selection.removeAllRanges()
        selection.addRange(range)
        document.dispatchEvent(new Event('selectionchange', { bubbles: true }))
      })

      const selectionAction = page.locator('[data-elephant-citation-selection-action="true"]')
      await expect(selectionAction).toBeVisible()
      await selectionAction.click()
      const bufferedCitation = page.locator('[data-elephant-citation-buffer-item]').first()
      await expect(bufferedCitation).toBeVisible()
      await expect(bufferedCitation).toHaveAttribute('aria-label', /Coller la citation Alpha note/)

      // Navigation clears the DOM selection. The retained citation icon must still work
      // and append to the destination note's last line.
      const projectsRow = page.locator('.en-sidebar-tree-row').filter({ hasText: 'Projects' }).first()
      if (!(await page.locator('.en-sidebar-tree-note').filter({ hasText: 'Beta project' }).count())) {
        await projectsRow.click()
      }
      await page.locator('.en-sidebar-tree-note').filter({ hasText: 'Beta project' }).first().click()
      await expect(page.getByTestId('muya-runtime-editor')).toBeVisible()
      await expect(page.locator('[data-elephant-citation-buffer-item]').first()).toBeVisible()
      await page.locator('[data-elephant-citation-buffer-item]').first().click()

      const destinationPath = path.join(fixture.vaultRoot, 'Projects', 'Beta.md')
      await expect
        .poll(() => fs.readFileSync(destinationPath, 'utf8'))
        .toContain('> Visible alpha body line.')
      await expect
        .poll(() => fs.readFileSync(destinationPath, 'utf8'))
        .toContain('> — [Alpha note](</Alpha.md#quote=')
      const destinationMarkdown = fs.readFileSync(destinationPath, 'utf8').trimEnd()
      expect(destinationMarkdown.endsWith(')')).toBe(true)
      expect(destinationMarkdown.lastIndexOf('> Visible alpha body line.')).toBeGreaterThan(
        destinationMarkdown.lastIndexOf('Visible beta body line.')
      )
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('note title bar compacts when the writing surface is scrolled', async () => {
    const context = await launchFeatureApp({
      prepareFixture: async (fixture) => {
        const body = Array.from({ length: 80 }, (_, index) => `Scrolling line ${index + 1}.`).join(
          '\n\n'
        )
        await fs.promises.writeFile(
          path.join(fixture.vaultRoot, 'Scrolling.md'),
          `# Scrolling\n\n${body}\n`,
          'utf8'
        )
      }
    })
    try {
      const { page } = context
      await card(page, 'Scrolling').click()
      const editor = page.locator('.en-editor-host .editor-component')
      await expect(editor).toBeVisible()
      await expect
        .poll(() => editor.evaluate((element) => element.scrollHeight))
        .toBeGreaterThan(200)
      await editor.evaluate((element) => {
        element.scrollTop = Math.min(320, element.scrollHeight - element.clientHeight)
        element.dispatchEvent(new Event('scroll', { bubbles: true }))
      })
      await expect(page.locator('.en-note-topbar')).toHaveClass(/is-compact/)
      const editorShellStyle = await page.locator('.en-note-editor-shell').evaluate((element) => {
        const style = getComputedStyle(element)
        return { borderTopWidth: style.borderTopWidth, borderTopStyle: style.borderTopStyle }
      })
      expect(editorShellStyle).toEqual({ borderTopWidth: '0px', borderTopStyle: 'none' })

      await editor.evaluate((element) => {
        element.scrollTop = 0
        element.dispatchEvent(new Event('scroll', { bubbles: true }))
      })
      await expect(page.locator('.en-note-topbar')).not.toHaveClass(/is-compact/)
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('editor scrolling does not add a white top border or shadow', async () => {
    const context = await launchFeatureApp({
      prepareFixture: async (fixture) => {
        const body = Array.from(
          { length: 80 },
          (_, index) => `Border check line ${index + 1}.`
        ).join('\n\n')
        await fs.promises.writeFile(
          path.join(fixture.vaultRoot, 'Border check.md'),
          `# Border check\n\n${body}\n`,
          'utf8'
        )
      }
    })
    try {
      const { page } = context
      await card(page, 'Border check').click()
      const editor = page.locator('.en-editor-host .editor-component')
      const editorShell = page.locator('.en-note-editor-shell')
      const topBar = page.locator('.en-note-topbar')
      await expect(editor).toBeVisible()
      await expect
        .poll(() => editor.evaluate((element) => element.scrollHeight))
        .toBeGreaterThan(200)

      await editor.evaluate((element) => {
        element.scrollTop = Math.min(320, element.scrollHeight - element.clientHeight)
        element.dispatchEvent(new Event('scroll', { bubbles: true }))
      })
      await expect(topBar).toHaveClass(/is-compact/)

      const chrome = await editorShell.evaluate((element) => {
        const shellStyle = getComputedStyle(element)
        const topBarElement = element.previousElementSibling
        const topBarStyle = topBarElement ? getComputedStyle(topBarElement) : null
        return {
          shellBorderTopWidth: shellStyle.borderTopWidth,
          shellBorderTopColor: shellStyle.borderTopColor,
          shellBoxShadow: shellStyle.boxShadow,
          topBarBorderTopWidth: topBarStyle?.borderTopWidth || null,
          topBarBorderTopColor: topBarStyle?.borderTopColor || null,
          topBarBoxShadow: topBarStyle?.boxShadow || null
        }
      })
      expect(chrome.shellBorderTopWidth).toBe('0px')
      expect(chrome.topBarBorderTopWidth).toBe('0px')
      expect(chrome.shellBoxShadow).toBe('none')
      expect(chrome.topBarBoxShadow).toBe('none')
      expect(chrome.shellBorderTopColor).not.toBe('rgb(255, 255, 255)')
      expect(chrome.topBarBorderTopColor).not.toBe('rgb(255, 255, 255)')
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('Rust task checkboxes update the note on disk through the editor', async () => {
    const context = await launchFeatureApp({
      prepareFixture: async (fixture) => {
        await fs.promises.writeFile(
          path.join(fixture.vaultRoot, 'Checklist.md'),
          '# Checklist\n\n- [ ] Persist this task\n',
          'utf8'
        )
      }
    })
    try {
      const { page, fixture } = context
      await card(page, 'Checklist').click()
      const checkbox = page
        .locator('[data-muya-rust-task-checkbox], input.ag-task-list-item-checkbox')
        .first()
      await expect(checkbox).toBeVisible()
      const checkboxStyle = await checkbox.evaluate((element) => {
        const style = getComputedStyle(element)
        const rect = element.getBoundingClientRect()
        return {
          appearance: style.appearance,
          width: rect.width,
          height: rect.height,
          borderRadius: style.borderRadius
        }
      })
      expect(checkboxStyle).toMatchObject({
        appearance: 'none',
        width: 16,
        height: 16,
        borderRadius: '3px'
      })
      await expect(checkbox).not.toBeChecked()
      await checkbox.click()
      await expect
        .poll(() => fs.readFileSync(path.join(fixture.vaultRoot, 'Checklist.md'), 'utf8'))
        .toContain('- [x] Persist this task')
      await expect(checkbox).toBeChecked()
    } finally {
      await closeFeatureApp(context)
    }
  })

  test('dragging an image resize handle persists its width in Markdown', async () => {
    const context = await launchFeatureApp({
      prepareFixture: async (fixture) => {
        await fs.promises.mkdir(path.join(fixture.vaultRoot, '.assets'), { recursive: true })
        await fs.promises.copyFile(
          path.join(process.cwd(), 'Elephant/frontend/src/muya/lib/assets/pngicon/image/2.png'),
          path.join(fixture.vaultRoot, '.assets', 'icon.png')
        )
        await fs.promises.writeFile(
          path.join(fixture.vaultRoot, 'Resizable.md'),
          '# Resizable\n\n![Elephant](.assets/icon.png)\n',
          'utf8'
        )
      }
    })
    try {
      const { page, fixture } = context
      await card(page, 'Resizable').click()
      const image = page
        .locator('.ag-inline-image .ag-image-container img, [data-muya-rust-kind="image"]')
        .first()
      await expect(image).toBeVisible()
      await image.click()
      const handle = page.locator('.ag-transformer .bottom-right')
      await expect(handle).toBeVisible()
      const imageBox = await image.boundingBox()
      const handleBox = await handle.boundingBox()
      if (!imageBox || !handleBox) throw new Error('Image resize geometry is unavailable')
      const startX = handleBox.x + handleBox.width / 2
      const startY = handleBox.y + handleBox.height / 2
      await handle.dispatchEvent('mousedown', {
        button: 0,
        clientX: startX,
        clientY: startY,
        bubbles: true
      })
      await page.evaluate(
        ({ clientX, clientY }) => {
          document.body.dispatchEvent(
            new MouseEvent('mousemove', { bubbles: true, clientX, clientY })
          )
          document.body.dispatchEvent(
            new MouseEvent('mouseup', { bubbles: true, clientX, clientY })
          )
        },
        { clientX: startX + 80, clientY: startY }
      )
      await expect
        .poll(() => fs.readFileSync(path.join(fixture.vaultRoot, 'Resizable.md'), 'utf8'))
        .toMatch(/!\[Elephant\]\(\.assets\/icon\.png\)\{width=\d+\}/)
      const persistedWidth = fs
        .readFileSync(path.join(fixture.vaultRoot, 'Resizable.md'), 'utf8')
        .match(/!\[Elephant\]\(\.assets\/icon\.png\)\{width=(\d+)\}/)?.[1]
      expect(persistedWidth).toBeTruthy()
      await page.locator('.en-sidebar-tree-note').filter({ hasText: 'Alpha note' }).first().click()
      await page.locator('.en-sidebar-tree-note').filter({ hasText: 'Resizable' }).first().click()
      await expect
        .poll(() =>
          page.locator('.ag-inline-image .ag-image-container img').first().getAttribute('width')
        )
        .toBe(persistedWidth)
    } finally {
      await closeFeatureApp(context)
    }
  })
})
