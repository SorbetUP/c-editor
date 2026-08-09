import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import {
  ELEPHANTNOTE_API_ACTIONS,
  ELEPHANTNOTE_API_DOMAINS,
  listApiContracts
} from '../../../../Elephant/shared/apiContracts.js'
import { COMPATIBILITY_CALLS } from '../../../../Elephant/frontend/app/services/elephantnoteClient/compatibilityCalls.js'

const root = process.cwd()
const absolute = (file) => path.join(root, file)
const read = (file) => fs.readFileSync(absolute(file), 'utf8')

const OPTIONAL_ACTIONS = Object.freeze([
  'ai.config.get',
  'ai.config.set',
  'ai.config.test',
  'models.selection.get',
  'models.selection.set',
  'models.local.list',
  'models.download',
  'ocr.extract',
  'search.inspect',
  'search.rebuild',
  'search.concepts',
  'search.initVault',
  'rag.chat',
  'sync.status',
  'sync.plan',
  'sync.enqueue',
  'sync.run',
  'wiki.list',
  'wiki.propose',
  'graph.rebuild',
  'sites.list',
  'calendar.list',
  'programs.run'
])

describe('minimal core API physical boundary', () => {
  it('advertises exactly the generic core domains', () => {
    expect(Object.keys(ELEPHANTNOTE_API_DOMAINS)).toEqual([
      'system',
      'vaults',
      'documents',
      'search',
      'coreFeatures'
    ])
    const actions = listApiContracts().map(({ name }) => name)
    expect(actions).toContain(ELEPHANTNOTE_API_ACTIONS.VAULTS_GET)
    expect(actions).toContain(ELEPHANTNOTE_API_ACTIONS.SEARCH_QUERY)
    expect(actions).toContain(ELEPHANTNOTE_API_ACTIONS.FEATURES_GET)
    for (const action of OPTIONAL_ACTIONS) expect(actions).not.toContain(action)
  })

  it('routes only generic core actions through compatibility calls', () => {
    const actions = Object.keys(COMPATIBILITY_CALLS)
    expect(actions).toContain('vaults.get')
    expect(actions).toContain('directory.list')
    expect(actions).toContain('search.query')
    expect(actions).toContain('features.get')
    for (const action of OPTIONAL_ACTIONS) expect(actions).not.toContain(action)
    expect(actions).not.toContain('notes.read')
    expect(actions).not.toContain('notes.write')
  })

  it('does not construct optional addon domain clients in the core renderer', () => {
    const domains = read('Elephant/frontend/app/services/elephantnoteClient/domainClients.js')
    for (const domain of [
      'sources:',
      'wiki:',
      'models:',
      'ai:',
      'ocr:',
      'sites:',
      'calendar:',
      'sync:',
      'programs:',
      'automation:',
      'plugins:'
    ]) expect(domains).not.toContain(domain)
    expect(domains).toContain('vaults:')
    expect(domains).toContain('notes:')
    expect(domains).toContain('search:')
  })

  it('keeps the atomic fallback generic instead of implementing extracted features', () => {
    const atomic = read('Elephant/frontend/app/services/elephantnoteClient/atomicFeatureApi.js')
    expect(atomic).toContain('describeApi:')
    expect(atomic).toContain('callApi:')
    expect(atomic).toContain('providers:')
    for (const method of [
      'semanticSearch:',
      'suggestTags:',
      'summarize:',
      'graphCluster:',
      'wikiPropose:',
      'wikiGenerate:',
      'localModelInstall:',
      'localModelList:'
    ]) expect(atomic).not.toContain(method)
  })

  it('compiles only generic core commands and stores AI configuration in its package', () => {
    const lib = read('Elephant/backend/tauri/src/lib_min.rs')
    const commands = read('Elephant/backend/tauri/src/core_commands.rs')
    const aiPackage = read('addons/official/ai/main.js')

    expect(fs.existsSync(absolute('Elephant/backend/tauri/src/tauri_extra_commands.rs'))).toBe(false)
    expect(lib).toContain('#[path = "core_commands.rs"]')
    for (const marker of [
      'tauri_ai_config_',
      'tauri_models_',
      'tauri_search_inspect',
      'tauri_search_rebuild',
      'iroh_sync_'
    ]) {
      expect(lib).not.toContain(marker)
      expect(commands).not.toContain(marker)
    }
    expect(aiPackage).toContain("const CONFIG_KEY = 'provider-config'")
    expect(aiPackage).toContain('this.api.storage.get(CONFIG_KEY)')
    expect(aiPackage).toContain('this.api.storage.set(CONFIG_KEY, payload)')
    expect(aiPackage).not.toContain('tauri_ai_config_')
  })
})
