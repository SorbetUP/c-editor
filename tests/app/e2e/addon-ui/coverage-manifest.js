const scenarioFileByAddon = Object.freeze({
  'elephant.dashboard': 'dashboard-recent-calendar-sites.js',
  'elephant.recently-edited': 'dashboard-recent-calendar-sites.js',
  'elephant.calendar': 'dashboard-recent-calendar-sites.js',
  'elephant.sites': 'dashboard-recent-calendar-sites.js',
  'elephant.ai': 'ai-family.js',
  'elephant.ai-chat': 'ai-family.js',
  'elephant.ai-search': 'ai-family.js',
  'elephant.knowledge': 'knowledge-wiki-graph.js',
  'elephant.wiki': 'knowledge-wiki-graph.js',
  'elephant.graph': 'knowledge-wiki-graph.js',
  'elephant.open-models': 'native-services.js',
  'elephant.codex-connection': 'native-services.js',
  'elephant.sync': 'native-services.js',
  'elephant.ai-ocr': 'ocr-code-execution.js',
  'elephant.code-execution': 'ocr-code-execution.js',
  'elephant.google-keep-import': 'google-keep-import.js'
})

const validateCoverage = (catalog, scenarios) => {
  const expected = new Set(catalog.addons.map((addon) => addon.id))
  const actual = scenarios.map((scenario) => scenario.addonId)
  const duplicates = actual.filter((id, index) => actual.indexOf(id) !== index)
  const missing = [...expected].filter((id) => !actual.includes(id))
  const unexpected = actual.filter((id) => !expected.has(id))
  if (duplicates.length || missing.length || unexpected.length) {
    throw new Error([
      duplicates.length ? `duplicate UI scenarios: ${[...new Set(duplicates)].join(', ')}` : '',
      missing.length ? `missing UI scenarios: ${missing.join(', ')}` : '',
      unexpected.length ? `unexpected UI scenarios: ${[...new Set(unexpected)].join(', ')}` : ''
    ].filter(Boolean).join('; '))
  }
  return true
}

module.exports = { scenarioFileByAddon, validateCoverage }
