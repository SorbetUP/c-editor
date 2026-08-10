'use strict'

const fs = require('node:fs')
const path = require('node:path')
const { closeSettings, ensureAddon, openSettingsSection } = require('./usage-harness')

const OCR_ID = 'elephant.ai-ocr'
const CODE_ID = 'elephant.code-execution'

const fixtureImagePath = (fixture, context) => {
  const values = [
    fixture?.ocrImagePath,
    fixture?.imagePath,
    fixture?.ocrImage,
    fixture?.assets?.ocrImagePath,
    fixture?.assets?.ocrImage,
    fixture?.assets?.ocr,
    fixture?.files?.ocrImage,
    context?.ocrImagePath,
    context?.fixture?.ocrImagePath
  ]
  const candidates = values
    .filter((value) => typeof value === 'string' && value.trim())
    .map((value) => {
      const trimmed = value.trim()
      return path.isAbsolute(trimmed)
        ? trimmed
        : path.resolve(fixture?.vaultRoot || process.cwd(), trimmed)
    })
  return (
    candidates.find((candidate) => fs.existsSync(candidate) && fs.statSync(candidate).isFile()) ||
    ''
  )
}

const findMarkdownContaining = (root, marker) => {
  if (!root || !fs.existsSync(root)) return null
  const visit = (directory) => {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      if (entry.name.startsWith('.')) continue
      const candidate = path.join(directory, entry.name)
      if (entry.isDirectory()) {
        const nested = visit(candidate)
        if (nested) return nested
      } else if (entry.isFile() && entry.name.endsWith('.md')) {
        const content = fs.readFileSync(candidate, 'utf8')
        if (content.includes(marker)) return { path: candidate, content }
      }
    }
    return null
  }
  return visit(root)
}

const waitForOutcome = async (expect, locator) => {
  const transient = /^(Checking|Starting|Running|Stopping|Running OCR)/
  await expect
    .poll(async () => ((await locator.textContent()) || '').trim(), { timeout: 120000 })
    .toMatch(/\S/)
  await expect
    .poll(async () => ((await locator.textContent()) || '').trim(), { timeout: 120000 })
    .not.toMatch(transient)
  return ((await locator.textContent()) || '').trim()
}

const ocr = {
  addonId: OCR_ID,

  async run({ page, fixture, expect, context }) {
    await ensureAddon(page, this.addonId)
    await openSettingsSection(page, 'AI')

    const aiSettings = page.locator('.elephant-ai-settings')
    await expect(aiSettings).toBeVisible()
    await aiSettings.getByRole('button', { name: 'OCR', exact: true }).click()
    const settings = page.locator('.elephant-ocr-addon-settings')
    await expect(settings).toBeVisible()
    const languages = settings.locator('input[placeholder="eng,fra"]')
    const output = settings.locator('select')
    await expect(languages).toBeVisible()
    await expect(output).toBeVisible()

    await languages.fill('eng')
    await languages.dispatchEvent('change')
    await output.selectOption('markdown')
    await openSettingsSection(page, 'Editor')
    await openSettingsSection(page, 'AI')
    await expect(aiSettings).toBeVisible()
    await aiSettings.getByRole('button', { name: 'OCR', exact: true }).click()
    await expect(languages).toHaveValue('eng')
    await expect(output).toHaveValue('markdown')

    const image = settings.locator('input[placeholder="/path/to/image.png"]')
    const imagePath = fixtureImagePath(fixture, context)
    const feedback = settings.locator('.elephant-ocr-addon-feedback')
    await image.fill(imagePath)
    await settings.getByRole('button', { name: 'Run OCR', exact: true }).click()
    const message = await waitForOutcome(expect, feedback)

    if (imagePath) {
      const badge = await settings.locator('.elephant-ocr-addon-badge').textContent()
      expect(`${badge || ''} ${message}`).toMatch(/\S/)
      if (!message) expect(badge || '').toMatch(/unavailable|not found|error|failed/i)
    } else {
      expect(message).toMatch(/image path|required|not found|unavailable|error/i)
    }

    await closeSettings(page)
    return { imagePath: imagePath || null, message, persistedSettings: true }
  }
}

const codeExecution = {
  addonId: CODE_ID,

  async run({ page, fixture, expect, context }) {
    void context
    await ensureAddon(page, this.addonId)
    await openSettingsSection(page, 'Editor')
    await expect(page.locator('.elephant-code-settings')).toBeVisible()
    await closeSettings(page)

    const filename = fixture?.codeExecutionNoteName || `Code execution UI ${Date.now()}.md`
    const marker = `code-execution-ui-${Date.now()}`
    const markdown = `# Code execution UI\n\n\`\`\`python\nprint(${JSON.stringify(marker)})\n\`\`\`\n`
    const bridgeAvailable = await page.evaluate(() =>
      Boolean(
        window.__ELEPHANT_ACCEPTANCE_TEST__?.createNote &&
        window.__ELEPHANT_ACCEPTANCE_TEST__?.openNote &&
        window.__ELEPHANT_ACCEPTANCE_TEST__?.setMarkdown &&
        window.__ELEPHANT_ACCEPTANCE_TEST__?.save
      )
    )

    if (bridgeAvailable) {
      await page.evaluate(
        async ({ filename, markdown }) => {
          const acceptance = window.__ELEPHANT_ACCEPTANCE_TEST__
          await acceptance.createNote('', filename)
          await acceptance.openNote(filename)
          acceptance.setMarkdown(markdown)
          await acceptance.save()
        },
        { filename, markdown }
      )
    } else {
      await page.getByRole('button', { name: 'Create', exact: true }).click()
      await page
        .getByRole('menu', { name: 'Create' })
        .getByRole('menuitem', { name: /Note/ })
        .click()
      const editor = page.locator('.editor-component[contenteditable="true"]').first()
      await expect(editor).toBeVisible()
      await editor.click()
      await page.keyboard.type('/')
      const quickInsert = page.locator('.ag-quick-insert')
      await expect(quickInsert).toBeVisible()
      await page.keyboard.type('code')
      const codeItem = quickInsert.locator('.item[data-label="pre"]')
      await expect(codeItem).toBeVisible()
      await codeItem.click()
      const codeContent = page.locator('pre.ag-fence-code code').first()
      await expect(codeContent).toBeVisible()
      await codeContent.click()
      await page.keyboard.type(`print(${JSON.stringify(marker)})`)
      const language = page.locator('pre.ag-fence-code.ag-active .ag-language-input').first()
      await expect(language).toBeVisible()
      await language.click()
      await page.keyboard.type('python')
      await page.keyboard.press('Enter')
    }

    const block = page.locator('[data-elephant-editor-kind="code_block"], .ag-fence-code').first()
    await expect(block).toBeVisible({ timeout: 30000 })
    const copy = block.locator('button.elephant-physical-code-copy')
    const run = block.locator('button.elephant-physical-code-run')
    const output = block.locator('pre.elephant-physical-code-output')
    await expect(copy).toBeVisible()
    await expect(run).toBeVisible()

    await copy.click()
    await expect(copy).toHaveText('Copied')
    const copied = await page.evaluate(() => navigator.clipboard?.readText?.() || '')
    expect(copied).toContain(marker)

    await run.click()
    const result = await waitForOutcome(expect, output)
    const exitCode = await output.getAttribute('data-exit-code')
    if (!result.includes(marker)) {
      expect(`${result} ${exitCode || ''}`).toMatch(
        /interpreter|configuration|unavailable|error|traceback|exception|execution service|no execution id|timed out|stopped/i
      )
    }

    if (fixture?.vaultRoot) {
      await expect
        .poll(() => findMarkdownContaining(fixture.vaultRoot, marker)?.content || '', {
          timeout: 30000
        })
        .toMatch(/```python/)
      const persisted = findMarkdownContaining(fixture.vaultRoot, marker)
      expect(persisted?.content).toContain(marker)
    }

    return { note: bridgeAvailable ? filename : 'Untitled.md', marker, output: result, exitCode }
  }
}

module.exports = Object.freeze([ocr, codeExecution])
