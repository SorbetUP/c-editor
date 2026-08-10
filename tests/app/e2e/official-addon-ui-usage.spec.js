const { test, expect } = require('playwright/test')
const { catalog, ensureAddon, launchUiApp } = require('./addon-ui/usage-harness')
const { validateCoverage, scenarioFileByAddon } = require('./addon-ui/coverage-manifest')

const scenarioModules = [
  require('./addon-ui/dashboard-recent-calendar-sites'),
  require('./addon-ui/ai-family'),
  require('./addon-ui/knowledge-wiki-graph'),
  require('./addon-ui/native-services'),
  require('./addon-ui/ocr-code-execution'),
  require('./addon-ui/google-keep-import')
]
const normalizeModule = (module) => Array.isArray(module) ? module : Object.values(module)
const scenarios = scenarioModules.flatMap(normalizeModule)
validateCoverage(catalog, scenarios)
for (const scenario of scenarios) {
  if (scenarioFileByAddon[scenario.addonId] === undefined) {
    throw new Error(`No declared UI scenario file for ${scenario.addonId}`)
  }
}

for (const scenario of scenarios) {
  test(`[official-addon-ui:${scenario.addonId}] real user workflow`, async ({ page: _page }, testInfo) => {
    void _page
    const context = await launchUiApp(testInfo, scenario.prepareFixture)
    try {
      await ensureAddon(context.page, scenario.addonId)
      await scenario.run({
        page: context.page,
        fixture: context.fixture,
        expect,
        context
      })
      await context.checkpoint(`${scenario.addonId}-ui-usage`, async () => ({
        addonId: scenario.addonId,
        snapshot: await context.page.evaluate((id) => window.__ELEPHANT_ADDONS__?.get?.(id) || null, scenario.addonId),
        visibleText: (await context.page.locator('body').innerText()).slice(0, 16000),
        errors: context.errors
      }))
      expect(context.errors.filter((entry) => entry.startsWith('pageerror:')), context.errors.join('\n')).toEqual([])
    } finally {
      await context.close()
    }
  })
}
