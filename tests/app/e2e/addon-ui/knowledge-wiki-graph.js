const fs = require('node:fs')
const path = require('node:path')

const knowledge = {
  addonId: 'elephant.knowledge',

  async run({ page, fixture, expect, context }) {
    void fixture
    void context

    await page.getByRole('button', { name: 'Settings', exact: true }).last().click()
    const search = page.getByRole('searchbox', { name: 'Search all settings' })
    await expect(search).toBeVisible()
    await search.fill('Knowledge index')

    const result = page.locator('.en-settings-search-results button').filter({
      hasText: 'Knowledge index'
    })
    await expect(result).toHaveCount(1)
    await result.click()

    const settings = page.locator('.elephant-knowledge-settings')
    await expect(settings).toBeVisible()
    const status = settings.locator('p').first()
    const rebuild = settings.getByRole('button', {
      name: 'Rebuild knowledge index',
      exact: true
    })
    await expect(rebuild).toBeVisible()
    await rebuild.click()
    await expect(status).toHaveText('Indexed 2; unchanged 0; removed 0.')
  }
}

const wiki = {
  addonId: 'elephant.wiki',

  async run({ page, fixture, expect, context }) {
    void context

    const wikiButton = page.getByRole('button', { name: 'Wiki', exact: true })
    await expect(wikiButton).toBeVisible()
    await wikiButton.click()

    const view = page.locator('.elephant-wiki-package')
    await expect(view).toBeVisible()
    await expect(view.locator('h2')).toHaveText('Wiki')

    const propose = view.getByRole('button', { name: 'Propose with AI', exact: true })
    await expect(propose).toBeVisible()
    await propose.click()

    const proposal = view.locator('.elephant-wiki-record').filter({ hasText: /E2e/i }).first()
    await expect(proposal).toBeVisible()
    await expect(proposal).toContainText('2 sources')
    await proposal.getByRole('button', { name: 'Approve', exact: true }).click()
    await expect(proposal).toHaveAttribute('data-status', 'accepted')

    const wikiPath = path.join(fixture.vaultRoot, 'Wiki', 'e2e.md')
    await expect.poll(() => fs.existsSync(wikiPath)).toBe(true)
    const markdown = fs.readFileSync(wikiPath, 'utf8')
    expect(markdown).toContain('[[Alpha]]')
    expect(markdown).toContain('[[Projects/Beta]]')
  }
}

const graph = {
  addonId: 'elephant.graph',

  async run({ page, fixture, expect, context }) {
    void fixture
    void context

    const graphButton = page.getByRole('button', { name: 'Graph', exact: true })
    await expect(graphButton).toBeVisible()
    await graphButton.click()

    const view = page.locator('.elephant-graph-package')
    await expect(view).toBeVisible()
    await expect(view.locator('h2')).toHaveText('Graph')
    await expect(view.locator('header p')).toHaveText(/2 nodes · \d+ edges/)
    await expect(view.locator('.elephant-graph-node')).toHaveCount(2)
    await expect(view.locator('.elephant-graph-node').filter({ hasText: 'Alpha note' })).toHaveCount(1)
    await expect(view.locator('.elephant-graph-node').filter({ hasText: 'Beta project' })).toHaveCount(1)
  }
}

module.exports = Object.freeze([knowledge, wiki, graph])
