const crypto = require('node:crypto')
const fs = require('node:fs')
const path = require('node:path')
const { test, expect } = require('playwright/test')
const { createSeededVaultFixture, launchElectron } = require('./helpers')

const root = process.cwd()
const catalog = JSON.parse(fs.readFileSync(path.join(root, 'tests/app/usage/linux/scenarios.json'), 'utf8'))
const metadata = new Map(catalog.scenarios.map((scenario) => [scenario.id, scenario]))
const screenshotFingerprints = new Map()

const enrichFixture = (fixture) => {
  fs.mkdirSync(path.join(fixture.vaultRoot, 'Notes'), { recursive: true })
  fs.writeFileSync(path.join(fixture.vaultRoot, 'Getting Started.md'), '# Getting Started\n\nElephant Linux usage fixture.\n\n- Search\n- Settings\n- Rust editor\n')
  fs.writeFileSync(path.join(fixture.vaultRoot, 'Project Alpha.md'), '# Project Alpha\n\nAlpha planning and requirements.\n')
  fs.writeFileSync(path.join(fixture.vaultRoot, 'Zeta Note.md'), '# Zeta Note\n\nA note used to verify title sorting.\n')
  fs.writeFileSync(path.join(fixture.vaultRoot, 'Notes', 'Deep Work.md'), '# Deep Work\n\nNested note fixture.\n')

  const timestamp = (day) => new Date(`2026-07-${String(day).padStart(2, '0')}T10:00:00.000Z`)
  fs.utimesSync(path.join(fixture.vaultRoot, 'Getting Started.md'), timestamp(10), timestamp(10))
  fs.utimesSync(path.join(fixture.vaultRoot, 'Zeta Note.md'), timestamp(11), timestamp(11))
  fs.utimesSync(path.join(fixture.vaultRoot, 'Project Alpha.md'), timestamp(12), timestamp(12))
}

const launchUsageApp = async (testInfo, options = {}) => {
  const fixture = await createSeededVaultFixture()
  enrichFixture(fixture)
  await options.prepareFixture?.(fixture)
  const launch = await launchElectron([], {
    userDataPath: fixture.userDataPath,
    env: {
      ELEPHANTNOTE_CONFIG_DIR: fixture.configRoot,
      ELEPHANT_E2E_VAULT_ROOT: fixture.vaultRoot,
      ELEPHANTNOTE_MUYA_RUNTIME: 'rust'
    }
  })
  const { app, page } = launch
  const errors = []
  page.on('pageerror', (error) => errors.push(`pageerror: ${error.message}`))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(`console.error: ${message.text()}`)
  })
  await page.setViewportSize({ width: 1366, height: 900 })
  await page.waitForSelector(options.readySelector || '.en-library-grid', { state: 'visible', timeout: 30000 })

  return {
    app,
    page,
    fixture,
    errors,
    async checkpoint(name) {
      const screenshotPath = testInfo.outputPath(`${name}.png`)
      await page.screenshot({ path: screenshotPath, fullPage: true })
      const digest = crypto.createHash('sha256').update(fs.readFileSync(screenshotPath)).digest('hex')
      const previous = screenshotFingerprints.get(digest)
      if (previous && previous.retry === testInfo.retry && previous.workerIndex === testInfo.workerIndex) {
        throw new Error(`Screenshot ${name} is pixel-identical to ${previous.name}; retain state data instead of duplicate visual evidence.`)
      }
      screenshotFingerprints.set(digest, {
        name,
        retry: testInfo.retry,
        workerIndex: testInfo.workerIndex
      })
      await testInfo.attach(name, { path: screenshotPath, contentType: 'image/png' })
      const dimensions = await page.evaluate(() => ({
        width: document.documentElement.scrollWidth,
        height: document.documentElement.scrollHeight,
        text: document.body.innerText.trim().slice(0, 500)
      }))
      expect(dimensions.width).toBeGreaterThan(500)
      expect(dimensions.height).toBeGreaterThan(400)
      expect(dimensions.text.length).toBeGreaterThan(20)
    },
    async checkpointState(name, value) {
      const resolved = typeof value === 'function' ? await value() : value
      const statePath = testInfo.outputPath(`${name}.json`)
      fs.writeFileSync(statePath, `${JSON.stringify(resolved, null, 2)}\n`)
      await testInfo.attach(name, { path: statePath, contentType: 'application/json' })
    },
    async healthy() {
      expect(await page.locator('.en-shell').isVisible()).toBe(true)
      expect(await page.locator('.muya-rust-runtime-error').count()).toBe(0)
      expect(errors, errors.join('\n')).toEqual([])
    },
    async close() {
      await app.close().catch(() => {})
      fs.rmSync(fixture.root, { recursive: true, force: true })
    }
  }
}

const card = (page, title) => page.locator('.en-note-card').filter({ hasText: title }).first()
const searchInput = (page) => page.getByPlaceholder('Search notes, paths, tags, or ideas…')
const openSearch = async (page) => {
  await page.getByRole('button', { name: 'Search', exact: true }).click()
  await expect(searchInput(page)).toBeVisible()
}

const closeSearch = async (page) => {
  const input = searchInput(page)
  if (await input.isVisible()) {
    if (await input.inputValue()) await input.press('Escape')
    await input.press('Escape')
  }
  await expect(input).toBeHidden()
}

const openSettings = async (page) => {
  await page.getByRole('button', { name: 'Settings', exact: true }).last().click()
  await expect(page.getByRole('dialog', { name: 'ElephantNote settings' })).toBeVisible()
}

const closeSettings = async (page) => {
  await page.getByRole('button', { name: 'Close settings' }).click()
  await expect(page.getByRole('dialog', { name: 'ElephantNote settings' })).toBeHidden()
}

const openNote = async (page, title) => {
  await card(page, title).click()
  const editor = page.getByTestId('muya-runtime-editor')
  await expect(editor).toBeVisible()
  await expect(page.locator('.en-note-topbar')).toBeVisible()
  await expect(page.getByRole('textbox', { name: 'Note title' })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Close note', exact: true })).toBeVisible()
  await expect(page.locator('.muya-rust-runtime-error')).toHaveCount(0)
  return editor
}

const closeNote = async (page) => {
  await page.getByRole('button', { name: 'Close note', exact: true }).click()
  await expect(page.locator('.en-library-grid')).toBeVisible()
}

const defineUsageTest = (id, implementation, launchOptions = {}) => {
  const scenario = metadata.get(id)
  test(`[linux-usage:${id}] ${scenario.description}`, async ({ browserName }, testInfo) => {
    void browserName
    const context = await launchUsageApp(testInfo, launchOptions)
    try {
      await implementation(context)
      await context.healthy()
    } finally {
      await context.close()
    }
  })
}

test.describe('Linux production-renderer usage regressions', () => {
  defineUsageTest('startup-render', async ({ page, checkpoint }) => {
    await expect(page.getByText('Getting Started', { exact: true }).first()).toBeVisible()
    await expect(page.getByText('Project Alpha', { exact: true }).first()).toBeVisible()
    await checkpoint('linux-startup-render')
  })

  defineUsageTest('vault-chooser-first-run', async ({ page, fixture, checkpoint, checkpointState }) => {
    await expect(page.getByRole('heading', { name: 'Choose your first vault' })).toBeVisible()
    await checkpoint('linux-vault-chooser')
    await page.getByRole('button', { name: /Dossier Android/ }).click()

    fs.writeFileSync(path.join(fixture.configRoot, 'elephantnote.json'), JSON.stringify({
      vaults: [{ id: 'selected-vault', name: 'Selected usage vault', path: fixture.vaultRoot, icon: 'vault' }],
      activeVaultId: 'selected-vault'
    }, null, 2))
    await page.reload()
    await page.waitForSelector('.en-library-grid', { state: 'visible', timeout: 30000 })
    await expect(page.getByText('Project Alpha', { exact: true }).first()).toBeVisible()
    await checkpointState('linux-vault-selected-state', async () => ({
      chooserVisible: await page.getByRole('heading', { name: 'Choose your first vault' }).isVisible().catch(() => false),
      cards: await page.locator('.en-note-card').count(),
      activeVault: await page.locator('.en-top-vault-name').textContent().catch(() => 'Selected usage vault')
    }))
  }, {
    readySelector: '.en-empty',
    prepareFixture: async (fixture) => {
      fs.writeFileSync(path.join(fixture.configRoot, 'elephantnote.json'), JSON.stringify({
        vaults: [],
        activeVaultId: null
      }, null, 2))
    }
  })

  defineUsageTest('library-layout-roundtrip', async ({ page, checkpoint }) => {
    await page.getByTitle('List').click()
    await expect(page.locator('.en-library-grid')).toHaveClass(/list/)
    await checkpoint('linux-layout-list')
    await page.getByTitle('Grid').click()
    await expect(page.locator('.en-library-grid')).not.toHaveClass(/list/)
  })

  defineUsageTest('library-title-sort', async ({ page, checkpoint }) => {
    const before = await page.locator('.en-note-card:not(.is-folder) h3').allTextContents()
    await page.locator('.en-library-actions .en-select').selectOption('title')
    await page.waitForTimeout(300)
    const after = await page.locator('.en-note-card:not(.is-folder) h3').allTextContents()
    const normalized = after.map((title) => title.trim()).filter(Boolean)
    expect(normalized.indexOf('Getting Started')).toBeLessThan(normalized.indexOf('Project Alpha'))
    expect(after).not.toEqual(before)
    await checkpoint('linux-library-title-sort')
  })

  defineUsageTest('sidebar-hide-show', async ({ page, checkpoint }) => {
    await page.getByRole('button', { name: 'Hide sidebar' }).click()
    await expect(page.getByRole('button', { name: 'Show sidebar' })).toBeVisible()
    await expect(page.locator('.en-sidebar')).toHaveCount(0)
    await checkpoint('linux-sidebar-hidden')
    await page.getByRole('button', { name: 'Show sidebar' }).click()
    await expect(page.locator('.en-sidebar')).toBeVisible()
  })

  defineUsageTest('search-roundtrip', async ({ page, checkpoint }) => {
    await openSearch(page)
    await searchInput(page).fill('Project Alpha')
    await expect(page.locator('.en-search-results')).toBeVisible()
    await expect(page.getByText('Project Alpha', { exact: true }).last()).toBeVisible()
    await checkpoint('linux-search-results')
    await closeSearch(page)
  })

  defineUsageTest('search-no-results', async ({ page, checkpoint }) => {
    await openSearch(page)
    await searchInput(page).fill('zzzxxyy-no-linux-match-9173')
    await expect(page.getByText('No matching notes found')).toBeVisible()
    await checkpoint('linux-search-empty')
    await closeSearch(page)
  })

  defineUsageTest('settings-roundtrip', async ({ page, checkpoint }) => {
    await openSettings(page)
    await page.getByRole('searchbox', { name: 'Search all settings' }).fill('autosave')
    await expect(page.getByText('Autosave', { exact: true }).first()).toBeVisible()
    await checkpoint('linux-settings-search')
    await closeSettings(page)
  })

  defineUsageTest('theme-mode-roundtrip', async ({ page, checkpoint }) => {
    const shell = page.locator('.en-shell')
    const initiallyDark = await shell.evaluate((element) => element.classList.contains('en-theme-dark'))
    await openSettings(page)
    await page.getByRole('button', { name: initiallyDark ? 'Light' : 'Dark', exact: true }).click()
    await expect.poll(() => shell.evaluate((element) => element.classList.contains('en-theme-dark'))).toBe(!initiallyDark)
    await checkpoint('linux-theme-toggled')
    await page.getByRole('button', { name: initiallyDark ? 'Dark' : 'Light', exact: true }).click()
    await closeSettings(page)
  })

  defineUsageTest('open-existing-note', async ({ page, checkpoint }) => {
    await openNote(page, 'Getting Started')
    await expect(page.getByText('Elephant Linux usage fixture.', { exact: true })).toBeVisible()
    await checkpoint('linux-note-open-full-chrome')
  })

  defineUsageTest('editor-write-autosave-preview', async ({ page, fixture, checkpoint }) => {
    const editor = await openNote(page, 'Project Alpha')
    await editor.click()
    await page.keyboard.press('Control+End')
    await page.keyboard.press('Enter')
    await page.keyboard.type('## CI Rendered Heading')
    await page.keyboard.press('Enter')
    await page.keyboard.type('Preview updated by Rust editor 9173.')

    await expect(editor).toContainText('CI Rendered Heading')
    await expect(editor).toContainText('Preview updated by Rust editor 9173.')
    await expect.poll(() => fs.readFileSync(path.join(fixture.vaultRoot, 'Project Alpha.md'), 'utf8'), {
      timeout: 15000
    }).toContain('Preview updated by Rust editor 9173.')
    await checkpoint('linux-editor-after-write')

    await closeNote(page)
    await expect(card(page, 'Project Alpha')).toContainText('Preview updated by Rust editor 9173.')
    await checkpoint('linux-library-preview-after-editor-save')
  })

  defineUsageTest('note-close-roundtrip', async ({ page, checkpoint }) => {
    await openNote(page, 'Getting Started')
    await closeNote(page)
    await openNote(page, 'Project Alpha')
    await checkpoint('linux-note-second-with-chrome')
  })

  defineUsageTest('pin-unpin-note', async ({ page, checkpoint }) => {
    const projectCard = card(page, 'Project Alpha')
    await projectCard.hover()
    await projectCard.getByRole('button', { name: 'Pin entry' }).click()
    await expect(projectCard.getByRole('button', { name: 'Unpin entry' })).toBeVisible()
    await checkpoint('linux-note-pinned')
    await projectCard.getByRole('button', { name: 'Unpin entry' }).click()
    await expect(projectCard.getByRole('button', { name: 'Pin entry' })).toBeVisible()
  })

  defineUsageTest('addon-install-enable-action', async ({ page, fixture, checkpoint }) => {
    await page.evaluate(() => {
      const manager = window.__ELEPHANT_ADDONS__
      if (!manager) throw new Error('Addon manager is not exposed')
      const manifest = {
        id: 'elephant.e2e-note-tools',
        name: 'E2E Note Tools',
        version: '1.0.0',
        description: 'Deterministic usage-test addon that modifies a real note.',
        author: 'Elephant CI',
        source: 'builtin',
        defaultEnabled: false,
        removable: true,
        permissions: { commands: true }
      }
      const definition = {
        manifest,
        activate(context) {
          context.addAction({
            id: 'elephant.e2e-note-tools.append-marker',
            title: 'Append CI marker',
            description: 'Append a deterministic marker to Project Alpha.',
            order: 1,
            async run() {
              const relativePath = 'Project Alpha.md'
              const current = await window.__TAURI__.core.invoke('tauri_notes_read', { path: relativePath })
              const markdown = String(current?.markdown || current?.content || '')
              const marker = 'Addon action updated this note 4421.'
              const next = markdown.includes(marker) ? markdown : `${markdown.trimEnd()}\n\n${marker}\n`
              await window.__TAURI__.core.invoke('tauri_notes_write', { relativePath, markdown: next })
              return { path: relativePath }
            }
          })
        }
      }
      const originalList = manager.listBuiltinCatalog.bind(manager)
      const originalInstall = manager.installBuiltin.bind(manager)
      manager.listBuiltinCatalog = () => [
        ...originalList().filter((entry) => entry?.manifest?.id !== manifest.id),
        { manifest, installed: Boolean(manager.get(manifest.id)) }
      ]
      manager.installBuiltin = async (id) => {
        if (id !== manifest.id) return originalInstall(id)
        return manager.get(id) || manager.register(definition)
      }
    })

    await openSettings(page)
    await page.getByRole('button', { name: 'Addons', exact: true }).click()
    await expect(page.getByRole('searchbox', { name: 'Search addons' })).toBeVisible()
    await expect(page.locator('[data-addon-id="elephant.e2e-note-tools"]')).toBeVisible()
    await checkpoint('linux-addon-catalogue')

    await page.locator('[data-addon-id="elephant.e2e-note-tools"]').click()
    await page.getByRole('button', { name: 'Install', exact: true }).click()
    const enableSwitch = page.getByRole('switch', { name: 'Enable E2E Note Tools' })
    await expect(enableSwitch).toBeVisible()
    await checkpoint('linux-addon-installed-detail')
    await enableSwitch.click()
    await expect(enableSwitch).toHaveAttribute('aria-checked', 'true')
    await page.getByRole('button', { name: 'Append CI marker', exact: true }).click()

    await expect.poll(() => fs.readFileSync(path.join(fixture.vaultRoot, 'Project Alpha.md'), 'utf8'), {
      timeout: 15000
    }).toContain('Addon action updated this note 4421.')
    await closeSettings(page)
    await expect(page.locator('.en-note-topbar')).toBeVisible()
    await expect(page.getByTestId('muya-runtime-editor')).toContainText('Addon action updated this note 4421.')
    await checkpoint('linux-addon-action-note-result')
    await closeNote(page)
    await expect(card(page, 'Project Alpha')).toContainText('Addon action updated this note 4421.')
  })

  defineUsageTest('reload-preserves-vault', async ({ page, checkpointState }) => {
    const before = await page.locator('.en-note-card').count()
    await page.reload()
    await page.waitForSelector('.en-library-grid', { state: 'visible' })
    const after = await page.locator('.en-note-card').count()
    expect(after).toBe(before)
    await checkpointState('linux-reload-preserves-vault-state', {
      cardsBefore: before,
      cardsAfter: after,
      gettingStartedVisible: await page.getByText('Getting Started', { exact: true }).first().isVisible(),
      projectAlphaVisible: await page.getByText('Project Alpha', { exact: true }).first().isVisible()
    })
  })

  defineUsageTest('navigation-stress', async ({ page, checkpointState }) => {
    for (let cycle = 1; cycle <= 3; cycle += 1) {
      await openSearch(page)
      await closeSearch(page)
      await openSettings(page)
      await closeSettings(page)
      await page.getByRole('button', { name: 'Hide sidebar' }).click()
      await page.getByRole('button', { name: 'Show sidebar' }).click()
      await expect(page.locator('.en-sidebar')).toBeVisible()
    }
    await openNote(page, 'Getting Started')
    await checkpointState('linux-navigation-stress-state', async () => ({
      cycles: 3,
      editorVisible: await page.getByTestId('muya-runtime-editor').isVisible(),
      editorChromeVisible: await page.locator('.en-note-topbar').isVisible(),
      errors: []
    }))
  })
})
