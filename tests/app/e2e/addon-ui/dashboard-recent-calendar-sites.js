const fs = require('node:fs')
const path = require('node:path')

const DASHBOARD_PATH = path.join('.elephantnote', 'Dashboard.md')
const SITE_DIRECTORY = 'Sites'
const SITE_INDEX = path.join(SITE_DIRECTORY, 'index.html')

const assertAddonEnabled = async (page, expect, addonId, contribution) => {
  const state = await page.evaluate((id) => {
    const manager = window.__ELEPHANT_ADDONS__
    const record = manager?.get?.(id)
    const contributionMap = manager?.getContributionMap?.() || {}
    const contributions = Object.entries(contributionMap).flatMap(([area, entries]) =>
      (entries || [])
        .filter((entry) => entry?.addonId === id)
        .map((entry) => ({ area, id: entry.contribution?.id || '' }))
    )
    return {
      enabled: record?.enabled === true,
      status: record?.status || '',
      error: record?.error?.message || record?.error || null,
      contributions
    }
  }, addonId)

  expect(state.enabled, `${addonId} must be enabled in the renderer`).toBe(true)
  expect(state.status, `${addonId} must be active in the renderer`).toBe('enabled')
  expect(state.error, `${addonId} must not have an activation error`).toBeNull()
  expect(state.contributions).toContainEqual(contribution)
  return state
}

const railButton = (page, label) =>
  page.locator(`.en-rail button.en-rail-icon[aria-label="${label}"]`)

const noteCard = (page, title) => page.locator('.en-note-card').filter({ hasText: title }).first()

const dashboard = {
  addonId: 'elephant.dashboard',

  async run({ page, fixture, expect, context }) {
    void context
    await assertAddonEnabled(page, expect, 'elephant.dashboard', {
      area: 'sidebar.items',
      id: 'elephant.dashboard.sidebar'
    })

    const dashboardButton = railButton(page, 'Open Dashboard')
    await expect(dashboardButton).toBeVisible()
    await dashboardButton.click()

    await expect(page.locator('.en-note-title-input')).toHaveValue('Dashboard')
    await expect(page.locator('.en-note-editor-shell')).toBeVisible()
    await expect.poll(() => fs.existsSync(path.join(fixture.vaultRoot, DASHBOARD_PATH))).toBe(true)

    return {
      route: 'note',
      notePath: DASHBOARD_PATH,
      persisted: true
    }
  }
}

const recentlyEdited = {
  addonId: 'elephant.recently-edited',

  async run({ page, fixture, expect, context }) {
    void context
    await assertAddonEnabled(page, expect, 'elephant.recently-edited', {
      area: 'layout.zones',
      id: 'elephant.recently-edited.sidebar-section'
    })

    const openNote = page.getByRole('button', { name: 'Close note' }).first()
    if (await openNote.count()) await openNote.click()
    await noteCard(page, 'Alpha note').click()
    await expect(page.getByTestId('muya-runtime-editor')).toBeVisible()
    await expect(page.locator('.elephant-physical-recently-edited-host')).toBeVisible()
    await expect(
      page.locator('.elephant-recent-note').filter({ hasText: 'Alpha note' })
    ).toBeVisible()

    await page.getByRole('button', { name: 'Close note' }).first().click()
    await page.locator('.en-sidebar-tree-row').filter({ hasText: 'Projects' }).first().click()
    await page.locator('.en-sidebar-tree-note').filter({ hasText: 'Beta project' }).first().click()
    await expect(page.getByTestId('muya-runtime-editor')).toBeVisible()

    const recentAlpha = page.locator('.elephant-recent-note').filter({ hasText: 'Alpha note' })
    await expect(recentAlpha).toBeVisible()
    await recentAlpha.click()
    await expect(page.locator('.en-note-title-input')).toHaveValue('Alpha note')
    await expect(page.getByTestId('muya-runtime-editor')).toBeVisible()

    return {
      visibleSection: '.elephant-recent-notes',
      clickedNote: 'Alpha.md',
      route: 'note'
    }
  }
}

const calendar = {
  addonId: 'elephant.calendar',

  async run({ page, fixture, expect, context }) {
    void fixture
    void context
    await assertAddonEnabled(page, expect, 'elephant.calendar', {
      area: 'views',
      id: 'elephant.calendar.workspace'
    })

    const calendarButton = railButton(page, 'Calendar')
    await expect(calendarButton).toBeVisible()
    await calendarButton.click()
    await expect(page.locator('.elephant-calendar-package')).toBeVisible()

    const picker = page.locator('input.elephant-calendar-picker')
    await picker.setInputFiles({
      name: 'lot-calendar.ics',
      mimeType: 'text/calendar',
      buffer: Buffer.from(
        [
          'BEGIN:VCALENDAR',
          'VERSION:2.0',
          'BEGIN:VEVENT',
          'UID:lot-calendar-event',
          'SUMMARY:Addon UI review',
          'DTSTART:20260815T090000Z',
          'DTEND:20260815T100000Z',
          'END:VEVENT',
          'END:VCALENDAR',
          ''
        ].join('\n')
      )
    })

    await expect(page.getByRole('button', { name: 'Import ICS', exact: true })).toBeEnabled()
    await page.getByRole('button', { name: 'Import ICS', exact: true }).click()
    const event = page.locator('.elephant-calendar-event').filter({ hasText: 'Addon UI review' })
    await expect(event).toBeVisible()
    await expect(event).toContainText('2026-08-15T09:00:00Z')

    return {
      view: 'elephant.calendar.workspace',
      eventId: 'lot-calendar-event',
      visibleEvent: true
    }
  }
}

const sites = {
  addonId: 'elephant.sites',

  async run({ page, fixture, expect, context }) {
    void context
    await assertAddonEnabled(page, expect, 'elephant.sites', {
      area: 'settings.sections',
      id: 'elephant.sites.settings'
    })

    const siteDirectory = path.join(fixture.vaultRoot, SITE_DIRECTORY)
    fs.mkdirSync(siteDirectory, { recursive: true })
    fs.writeFileSync(
      path.join(fixture.vaultRoot, SITE_INDEX),
      '<!doctype html><title>Addon UI site</title><h1>Addon UI site</h1>\n',
      'utf8'
    )

    await page.getByRole('button', { name: 'Settings' }).click()
    await expect(page.locator('.en-settings-panel')).toBeVisible()
    await page.locator('.en-settings-nav button').filter({ hasText: 'Sites' }).click()
    await expect(page.locator('.en-settings-content')).toHaveAttribute(
      'data-active-section',
      'sites'
    )
    await expect(page.locator('.elephant-sites-package')).toBeVisible()

    await page.getByPlaceholder('Sites/my-site').fill(SITE_DIRECTORY)
    const rendererErrors = []
    const onPageError = (error) => rendererErrors.push(error?.message || String(error))
    page.on('pageerror', onPageError)
    try {
      await page.getByRole('button', { name: 'Preview directory', exact: true }).click()
      await expect(page.locator('.elephant-sites-card')).toBeVisible()
      await expect(page.locator('.elephant-sites-card')).toContainText(SITE_DIRECTORY)
      await expect(page.locator('.elephant-sites-frame')).toBeVisible()
    } catch (error) {
      const missingAssetBridge = rendererErrors.find((message) =>
        message.includes('tauri_addons_assets_allow_directory')
      )
      if (missingAssetBridge) {
        throw new Error(
          'Sites UI preview is blocked by the Playwright bridge: tauri_addons_assets_allow_directory is not implemented.'
        )
      }
      throw error
    } finally {
      page.off('pageerror', onPageError)
    }

    await page.getByRole('button', { name: 'Close preview', exact: true }).click()
    await expect.poll(() => fs.existsSync(path.join(fixture.vaultRoot, SITE_DIRECTORY))).toBe(false)

    return {
      settingsSection: 'sites',
      previewSource: SITE_INDEX,
      closed: true
    }
  }
}

module.exports = { dashboard, recentlyEdited, calendar, sites }
