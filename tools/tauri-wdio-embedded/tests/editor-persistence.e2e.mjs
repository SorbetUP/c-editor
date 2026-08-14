import assert from 'node:assert/strict'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'

import { browser } from '@wdio/globals'

import { loadScenario } from '../../../tools/freya-differential/lib/scenario.mjs'
import { requireElement, sleep } from '../runtime.mjs'

const projectRoot = path.resolve(import.meta.dirname, '../../..')
const scenarioPath = path.resolve(
  process.env.DIFFERENTIAL_SCENARIO_PATH ??
    path.join(projectRoot, 'migration/freya/differential-scenarios.json')
)
const scenario = await loadScenario(scenarioPath)
const fixtureRoot = path.resolve(process.env.DIFFERENTIAL_FIXTURE_ROOT ?? '')
const vaultRoot = path.join(fixtureRoot, scenario.fixture.roots.vault)
const outputRoot = path.resolve(process.env.DIFFERENTIAL_OUTPUT_DIR ?? path.join(projectRoot, 'test-results/tauri-wdio-embedded/editor-persistence'))
const edit = scenario.actions.find((action) => action.id === 'edit-alpha-note')

if (!process.env.DIFFERENTIAL_FIXTURE_ROOT) {
  throw new Error('DIFFERENTIAL_FIXTURE_ROOT is required for the focused editor persistence run')
}

describe('Elephant Tauri embedded WebDriver editor persistence', () => {
  it('persists text sent through the real Muya contenteditable', async () => {
    await requireElement('.en-library-grid', 'library grid readiness')
    const card = await browser.$$('.en-note-card').then(async (cards) => {
      for (const candidate of cards) {
        if ((await candidate.getText()).includes('Alpha note')) return candidate
      }
      throw new Error('Alpha note card was not found')
    })
    await card.click()

    const editor = await requireElement('[data-testid="muya-runtime-editor"]', 'Muya runtime editor')
    const paragraph = await browser.$('.editor-component .ag-paragraph')
    await (await paragraph.isExisting().catch(() => false) ? paragraph : editor).click()
    for (const chord of edit.keysBeforeText ?? []) await browser.keys(chord.split('+'))
    await editor.addValue(edit.text)

    const filename = path.join(vaultRoot, 'Alpha.md')
    await browser.waitUntil(
      async () => (await readFile(filename, 'utf8').catch(() => '')).includes(edit.text),
      { timeout: 20000, timeoutMsg: 'Muya text was not persisted to Alpha.md' }
    )

    const persisted = await readFile(filename, 'utf8')
    assert.ok(persisted.includes(edit.text), 'persisted Alpha.md must contain the deterministic marker')
    await mkdir(outputRoot, { recursive: true })
    await browser.saveScreenshot(path.join(outputRoot, 'editor-persisted.png'))
    await writeFile(path.join(outputRoot, 'editor-persistence.json'), `${JSON.stringify({
      scenarioId: scenario.id,
      actionId: edit.id,
      inputTransport: 'webdriver-element-send-keys',
      marker: edit.text,
      persistedPath: filename,
      persisted: persisted.includes(edit.text)
    }, null, 2)}\n`, 'utf8')
    await sleep(50)
  })
})
