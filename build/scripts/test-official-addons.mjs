import assert from 'node:assert/strict'
import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import vm from 'node:vm'

const root = path.resolve(process.argv[2] || '')
if (!process.argv[2]) {
  console.error('Usage: node build/scripts/test-official-addons.mjs <catalogue-directory>')
  process.exit(2)
}

const loadAddon = async (slug) => {
  const source = await fs.readFile(path.join(root, 'addons', slug, 'main.js'), 'utf8')
  const sandbox = { self: {}, Intl }
  vm.runInNewContext(source, sandbox, { filename: `${slug}/main.js`, timeout: 1_000 })
  return sandbox.self.elephantAddon
}

const activateCommand = async (definition, api) => {
  let command = null
  const wrappedApi = {
    ...api,
    commands: {
      register(value) {
        command = value
        return () => { command = null }
      }
    }
  }
  await definition.activate(wrappedApi)
  assert.ok(command?.id)
  return command
}

const createStorage = () => {
  const values = new Map()
  return {
    values,
    api: {
      get: async (key) => values.get(key) ?? null,
      set: async (key, value) => { values.set(key, value); return { ok: true } },
      remove: async (key) => values.delete(key),
      entries: async () => Object.fromEntries(values)
    }
  }
}

const noteApi = (sourceNotes = new Map(), written = new Map()) => ({
  list: async () => [...sourceNotes.entries()].map(([notePath, content], index) => ({
    path: notePath,
    size: content.length,
    modifiedAt: 1_700_000_000_000 + index
  })),
  read: async (notePath) => {
    if (sourceNotes.has(notePath)) return { path: notePath, content: sourceNotes.get(notePath) }
    if (written.has(notePath)) return { path: notePath, content: written.get(notePath) }
    throw new Error(`missing note: ${notePath}`)
  },
  write: async (notePath, content) => {
    written.set(notePath, content)
    return { ok: true, path: notePath }
  }
})

const testPlatformProof = async () => {
  const definition = await loadAddon('platform-proof')
  const storage = createStorage()
  const notes = new Map()
  const command = await activateCommand(definition, {
    app: { info: async () => ({ name: 'ElephantNote', version: '0.1.0', addonApiVersion: 1 }) },
    notes: {
      write: async (notePath, content) => { notes.set(notePath, content); return { ok: true, path: notePath } },
      read: async (notePath) => ({ path: notePath, content: notes.get(notePath) }),
      list: async (prefix) => [...notes.keys()]
        .filter((notePath) => notePath === prefix || notePath.startsWith(`${prefix}/`))
        .map((notePath) => ({ path: notePath, size: notes.get(notePath).length, modifiedAt: Date.now() }))
    },
    http: { request: async () => { throw new Error('network not expected') } },
    storage: storage.api
  })

  const first = await command.run()
  const second = await command.run()
  assert.equal(first.verified, true)
  assert.equal(second.verified, true)
  assert.equal(second.runCount, 2)
  assert.match(notes.get(second.path), /ELEPHANT_ADDON_PROOF:2:/)
  console.log('[official-addons] platform-proof ok runs=2')
}

const financePayload = (symbol, latest, previous) => JSON.stringify({
  chart: {
    result: [{
      meta: {
        symbol,
        longName: `${symbol} Asset`,
        currency: symbol === 'BTC-USD' ? 'USD' : 'EUR',
        exchangeName: 'TEST',
        instrumentType: 'EQUITY',
        chartPreviousClose: previous
      },
      indicators: { quote: [{ close: [previous, latest] }] }
    }],
    error: null
  }
})

const testFinanceNotes = async () => {
  const definition = await loadAddon('finance-notes')
  const storage = createStorage()
  const notes = new Map()
  let failNvda = false
  const command = await activateCommand(definition, {
    app: { info: async () => ({}) },
    notes: {
      list: async () => [],
      read: async () => { throw new Error('read not expected') },
      write: async (notePath, content) => { notes.set(notePath, content); return { ok: true, path: notePath } }
    },
    http: {
      request: async ({ url }) => {
        const symbol = decodeURIComponent(url.match(/chart\/([^?]+)/)?.[1] || '')
        if (symbol === 'NVDA' && failNvda) throw new Error('simulated provider outage')
        const seed = [...symbol].reduce((sum, character) => sum + character.charCodeAt(0), 100)
        return { ok: true, status: 200, body: financePayload(symbol, seed + 5, seed) }
      }
    },
    storage: storage.api
  })

  const first = await command.run({ symbols: ['AAPL', 'NVDA'] })
  assert.equal(first.live, 2)
  assert.match(notes.get(first.path), /AAPL Asset/)
  assert.match(notes.get(first.path), /NVDA Asset/)

  failNvda = true
  const second = await command.run({ symbols: ['AAPL', 'NVDA'] })
  assert.equal(second.live, 1)
  assert.equal(second.cached, 1)
  assert.match(notes.get(second.path), /\| Cached \|/)
  assert.match(notes.get('Finance/NVDA.md'), /Data status:\*\* Cached fallback/)
  console.log('[official-addons] finance-notes ok live=1 cached=1')
}

const testInboxDigest = async () => {
  const definition = await loadAddon('inbox-digest')
  const storage = createStorage()
  const sourceNotes = new Map([
    ['Inbox/Idea.md', '---\ntitle: "Addon signing"\nstatus: "unprocessed"\n---\n\n# Addon signing\n\nCompare publisher signatures.\n\n- [ ] Write a proposal\n'],
    ['Inbox/Meeting.md', '# Meeting notes\n\nFollow up with the team.\n\n- [ ] Send summary\n- [ ] Create issue\n']
  ])
  const written = new Map()
  const command = await activateCommand(definition, {
    app: { info: async () => ({}) },
    notes: noteApi(sourceNotes, written),
    http: { request: async () => { throw new Error('network not expected') } },
    storage: storage.api
  })

  const result = await command.run()
  const digest = written.get(result.path)
  assert.equal(result.listed, 2)
  assert.equal(result.reviewed, 2)
  assert.equal(result.openTasks, 3)
  assert.match(digest, /Addon signing/)
  assert.match(digest, /Meeting notes/)
  assert.match(digest, /Open tasks found: 3/)
  console.log('[official-addons] inbox-digest ok notes=2 tasks=3')
}

const testTaskDashboard = async () => {
  const definition = await loadAddon('task-dashboard')
  const sourceNotes = new Map([
    ['Projects/Alpha.md', '# Alpha\n\n- [ ] Design API\n- [x] Create branch\n'],
    ['Daily/2026-07-10.md', '# Today\n\n- [ ] Review pull request\n']
  ])
  const written = new Map()
  const command = await activateCommand(definition, {
    app: { info: async () => ({}) },
    notes: noteApi(sourceNotes, written),
    http: { request: async () => { throw new Error('network not expected') } },
    storage: createStorage().api
  })
  const result = await command.run()
  assert.equal(result.open, 2)
  assert.equal(result.completed, 1)
  assert.equal(result.sources, 2)
  assert.match(written.get(result.path), /Design API/)
  assert.match(written.get(result.path), /Review pull request/)
  console.log('[official-addons] task-dashboard ok open=2 completed=1')
}

const testBrokenLinks = async () => {
  const definition = await loadAddon('broken-links-auditor')
  const sourceNotes = new Map([
    ['Home.md', '# Home\n\n[[Existing]]\n[[Missing]]\n[[Shared]]\n'],
    ['Existing.md', '# Existing\n'],
    ['One/Shared.md', '# Shared one\n'],
    ['Two/Shared.md', '# Shared two\n']
  ])
  const written = new Map()
  const command = await activateCommand(definition, {
    app: { info: async () => ({}) },
    notes: noteApi(sourceNotes, written),
    http: { request: async () => { throw new Error('network not expected') } },
    storage: createStorage().api
  })
  const result = await command.run()
  assert.equal(result.broken, 1)
  assert.equal(result.ambiguous, 1)
  assert.equal(result.checked, 3)
  const report = written.get(result.path)
  assert.match(report, /\[\[Missing\]\]/)
  assert.match(report, /\[\[Shared\]\]/)
  console.log('[official-addons] broken-links-auditor ok broken=1 ambiguous=1')
}

const githubReleasePayload = (repository, tag) => JSON.stringify({
  tag_name: tag,
  name: `${repository} ${tag}`,
  html_url: `https://github.com/${repository}/releases/tag/${tag}`,
  published_at: '2026-07-10T00:00:00Z',
  prerelease: false,
  draft: false,
  body: 'Release notes.'
})

const testGithubReleaseWatcher = async () => {
  const definition = await loadAddon('github-release-watcher')
  const storage = createStorage()
  const sourceNotes = new Map()
  const written = new Map()
  let tag = 'v1.0.0'
  const notes = noteApi(sourceNotes, written)
  notes.read = async (notePath) => {
    if (written.has(notePath)) return { path: notePath, content: written.get(notePath) }
    throw new Error('configuration missing')
  }
  const command = await activateCommand(definition, {
    app: { info: async () => ({}) },
    notes,
    http: {
      request: async ({ url }) => {
        const match = url.match(/repos\/([^/]+\/[^/]+)\/releases\/latest/)
        const repository = match?.[1] || 'unknown/repository'
        return { ok: true, status: 200, body: githubReleasePayload(repository, tag) }
      }
    },
    storage: storage.api
  })

  const first = await command.run()
  assert.equal(first.configurationCreated, true)
  assert.equal(first.live, 3)
  assert.equal(first.newReleases, 0)
  tag = 'v1.1.0'
  const second = await command.run()
  assert.equal(second.newReleases, 3)
  assert.match(written.get(second.path), /New release/)
  console.log('[official-addons] github-release-watcher ok new=3')
}

await testPlatformProof()
await testFinanceNotes()
await testInboxDigest()
await testTaskDashboard()
await testBrokenLinks()
await testGithubReleaseWatcher()
console.log('[official-addons] all functional pipelines passed')
