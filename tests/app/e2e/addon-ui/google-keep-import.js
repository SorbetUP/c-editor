const fs = require('node:fs')
const path = require('node:path')
const { closeSettings, ensureAddon, openSettingsSection } = require('./usage-harness')

const NOTE_TITLE = 'Elephant imported UI note'
const NOTE_BODY = 'Imported through the real Google Keep addon UI.'

const cardWithExactTitle = (page, selector, title) => page
  .locator(selector)
  .filter({ has: page.locator('h3', { hasText: title }) })
  .first()
const noteCard = (page, title) => cardWithExactTitle(page, '.en-note-card:not(.is-folder)', title)
const folderCard = (page, title) => cardWithExactTitle(page, '.en-note-card.is-folder', title)

const googleKeep = {
  addonId: 'elephant.google-keep-import',
  async prepareFixture(fixture) {
    const keepDirectory = path.join(fixture.vaultRoot, 'tmp', 'Takeout', 'Keep')
    const keepPath = path.join(keepDirectory, `${NOTE_TITLE}.json`)
    fs.mkdirSync(keepDirectory, { recursive: true })
    fs.writeFileSync(
      keepPath,
      `${JSON.stringify(
        {
          title: NOTE_TITLE,
          textContent: NOTE_BODY,
          listContent: [{ text: 'Verify on disk', isChecked: true }],
          labels: ['ui-test'],
          isPinned: true
        },
        null,
        2
      )}\n`
    )
    fixture.keepPath = keepPath
  },
  async run({ page, fixture, expect, context }) {
    void context
    await ensureAddon(page, this.addonId)
    await openSettingsSection(page, 'Import')
    const settings = page.locator('.elephant-keep-import')
    await expect(settings).toBeVisible()
    const input = settings.locator('input[type="file"]')
    await expect(input).toHaveAttribute('accept', 'application/json,.json')
    await input.setInputFiles(fixture.keepPath)
    const importButton = settings.getByRole('button', {
      name: 'Import selected files',
      exact: true
    })
    await expect(importButton).toBeEnabled()
    await importButton.click()
    const status = settings.locator('.elephant-keep-status')
    const importedPath = path.join(fixture.vaultRoot, 'Imported', 'Google Keep', `${NOTE_TITLE}.md`)

    await expect(status).toContainText(/"imported": 1|"failed": 1/)
    const statusText = await status.textContent()
    if (statusText?.includes('Unhandled E2E Tauri invoke: tauri_addons_notes_write')) {
      await expect(status).toContainText('"failed": 1')
      expect(fs.existsSync(importedPath)).toBe(false)
      await closeSettings(page)
      return { imported: false, visibleError: true }
    }

    await expect(status).toContainText('"imported": 1')
    await expect(status).toContainText('"failed": 0')
    await expect.poll(() => fs.existsSync(importedPath)).toBe(true)
    const markdown = fs.readFileSync(importedPath, 'utf8')
    expect(markdown).toContain(`# ${NOTE_TITLE}`)
    expect(markdown).toContain(NOTE_BODY)
    expect(markdown).toContain('- [x] Verify on disk')

    await closeSettings(page)
    await page.getByRole('button', { name: 'All notes', exact: true }).click()
    await expect(folderCard(page, 'Imported')).toBeVisible()
    await folderCard(page, 'Imported').click()
    await expect(folderCard(page, 'Google Keep')).toBeVisible()
    await folderCard(page, 'Google Keep').click()
    const importedNote = noteCard(page, NOTE_TITLE)
    await expect(importedNote).toBeVisible()
    await importedNote.click()
    await expect(page.getByTestId('muya-runtime-editor')).toBeVisible()
    await expect(page.locator('.en-editor-host')).toContainText(NOTE_BODY)
  }
}

module.exports = [googleKeep]
