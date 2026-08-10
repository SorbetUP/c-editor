const fs = require('node:fs')
const path = require('node:path')
const {
  closeSettings,
  ensureAddon,
  openAddonView,
  openSettingsSection,
  readAddonHttpTrace
} = require('./usage-harness')

const AI_ID = 'elephant.ai'
const PROVIDER = {
  label: 'E2E intercepted provider',
  endpoint: 'https://api.openai.com/v1',
  chatModel: 'e2e-chat-model',
  embeddingModel: 'e2e-embedding-model',
  enabled: true
}

const persistedAddonValue = (fixture, addonId, key) => {
  try {
    const state = JSON.parse(
      fs.readFileSync(path.join(fixture.configRoot, 'official-addons-e2e-state.json'), 'utf8')
    )
    return state.storage?.[`${addonId}:${key}`]
  } catch {
    return undefined
  }
}

const providerField = (provider, label) => provider
  .locator('label')
  .filter({ hasText: new RegExp(`^\\s*${label}\\s*$`) })
  .locator('input')
  .first()

const configureInterceptedProvider = async ({ page, fixture, expect }) => {
  await ensureAddon(page, AI_ID)
  await openSettingsSection(page, 'AI')

  const settings = page.locator('.elephant-ai-settings')
  await expect(settings).toBeVisible()
  await settings.getByRole('button', { name: 'Providers', exact: true }).click()
  await settings.getByRole('button', { name: 'Add provider', exact: true }).click()

  const provider = settings.locator('.elephant-ai-provider').last()
  await providerField(provider, 'Name').fill(PROVIDER.label)
  await providerField(provider, 'Base URL').fill(PROVIDER.endpoint)
  await providerField(provider, 'Chat model').fill(PROVIDER.chatModel)
  await providerField(provider, 'Embedding model').fill(PROVIDER.embeddingModel)
  await providerField(provider, 'Embedding model').blur()

  await expect
    .poll(() => persistedAddonValue(fixture, AI_ID, 'provider-config'), { timeout: 5000 })
    .toEqual(expect.objectContaining({
      providers: expect.objectContaining({
        list: expect.arrayContaining([expect.objectContaining(PROVIDER)])
      })
    }))

  const config = persistedAddonValue(fixture, AI_ID, 'provider-config')
  const configured = config.providers.list.find((entry) => entry.label === PROVIDER.label)
  if (!configured?.id) throw new Error('The intercepted E2E provider did not receive a persisted id')
  await closeSettings(page)
  return { ...PROVIDER, id: configured.id }
}

const configureChatRoute = async ({ page, provider, expect }) => {
  await openSettingsSection(page, 'AI')
  const settings = page.locator('.elephant-ai-settings')
  await settings.getByRole('button', { name: 'Chat', exact: true }).click()
  const chatSettings = settings.locator('.elephant-chat-settings')
  await expect(chatSettings).toBeVisible()

  const source = chatSettings.locator('select').first()
  await source.selectOption(provider.id)
  const models = chatSettings.locator('select').nth(1)
  await expect(models.locator('option[value="e2e-chat-model"]')).toHaveCount(1)
  await models.selectOption('e2e-chat-model')
  await chatSettings.getByRole('button', { name: 'Enregistrer la route Chat', exact: true }).click()
  await expect(chatSettings.locator('.elephant-chat-feedback').last()).toHaveText('Route Chat enregistrée.')
  await closeSettings(page)
}

const ai = {
  addonId: AI_ID,
  async run({ page, fixture, expect, context }) {
    const provider = await configureInterceptedProvider({ page, fixture, expect })
    const inference = await page.evaluate(async (providerId) => {
      const resource = window.__ELEPHANT_ADDON_HOST__?.get('ai.inference')
      if (!resource) throw new Error('ai.inference resource is unavailable')
      return {
        providers: await resource.listProviders(),
        models: await resource.listModels({ providerId, route: 'chat' }),
        completion: await resource.complete(
          [{ role: 'user', content: 'Which notes mention alpha?' }],
          { providerId, model: 'e2e-chat-model' }
        ),
        embeddings: await resource.embed(
          ['Alpha note', 'Beta project'],
          { providerId, model: 'e2e-embedding-model' }
        )
      }
    }, provider.id)

    expect(inference.providers).toEqual(expect.arrayContaining([
      expect.objectContaining({ id: provider.id, endpoint: provider.endpoint })
    ]))
    expect(inference.models.models).toEqual([
      expect.objectContaining({ id: 'e2e-chat-model', ownedBy: 'elephant-e2e' })
    ])
    expect(inference.completion).toEqual(expect.objectContaining({
      text: 'E2E provider response: Alpha note contains the requested alpha context.',
      model: 'e2e-chat-model',
      providerId: provider.id
    }))
    expect(inference.embeddings.vectors).toHaveLength(2)
    expect(inference.embeddings.dimensions).toBe(4)

    const trace = readAddonHttpTrace(fixture)
    expect(trace.map((entry) => `${entry.method} ${entry.path}`)).toEqual(expect.arrayContaining([
      'GET /v1/models',
      'POST /v1/chat/completions',
      'POST /v1/embeddings'
    ]))
    await context.checkpoint('ai-intercepted-http', { provider, inference, trace })
  }
}

const aiChat = {
  addonId: 'elephant.ai-chat',
  async run({ page, fixture, expect, context }) {
    const provider = await configureInterceptedProvider({ page, fixture, expect })
    await configureChatRoute({ page, provider, expect })

    const chat = await openAddonView(
      page,
      'Assistant IA connecté au vault',
      '.elephant-chat-package'
    )
    await expect(chat.locator('.elephant-chat-empty')).toBeVisible()

    const question = 'Which notes mention alpha?'
    await chat.locator('textarea[placeholder="Ask"]').fill(question)
    await chat.locator('form button[type="submit"]').press('Enter')

    const assistant = chat.locator('.en-chat-message.assistant').last()
    const answer = 'E2E provider response: Alpha note contains the requested alpha context.'
    await expect(assistant.locator('.en-chat-message-body')).toHaveText(answer, { timeout: 15000 })
    await expect(assistant.locator('.en-chat-message-head small')).toContainText('e2e-chat-model')

    await chat.locator('textarea[placeholder="Ask"]').fill('[e2e-error] Trigger provider error')
    await chat.locator('form button[type="submit"]').press('Enter')
    const failedAssistant = chat.locator('.en-chat-message.assistant').last()
    await expect(failedAssistant.locator('.en-chat-message-body')).toHaveText(
      'E2E provider rejected this synthetic request.',
      { timeout: 15000 }
    )

    const readMessages = () => {
      const state = persistedAddonValue(fixture, this.addonId, 'chat-state-v2')
      return Object.values(state?.vaults || {})
        .flatMap((vault) => vault.conversations || [])
        .flatMap((conversation) => conversation.messages || [])
    }
    await expect
      .poll(readMessages, { timeout: 5000 })
      .toEqual(expect.arrayContaining([
        expect.objectContaining({ role: 'user', content: question }),
        expect.objectContaining({ role: 'assistant', content: answer })
      ]))
    const messages = readMessages()
    expect(messages.map((message) => message.content)).toEqual(expect.arrayContaining([question, answer]))
    expect(messages).toEqual(expect.arrayContaining([
      expect.objectContaining({ error: 'E2E provider rejected this synthetic request.' })
    ]))

    await page.reload()
    await page.waitForSelector('.en-library-grid', { state: 'visible' })
    await ensureAddon(page, this.addonId)
    const reloaded = await openAddonView(
      page,
      'Assistant IA connecté au vault',
      '.elephant-chat-package'
    )
    await expect(reloaded.locator('.en-chat-message.assistant').first().locator('.en-chat-message-body'))
      .toHaveText(answer)

    const trace = readAddonHttpTrace(fixture)
    expect(trace.filter((entry) => entry.path === '/v1/chat/completions').length).toBeGreaterThanOrEqual(2)
    await context.checkpoint('ai-chat-intercepted-usage', { provider, answer, trace, messages })
    await reloaded.locator('button[title="Fermer le chat"]').click()
    await expect(page.locator('.elephant-chat-package')).toHaveCount(0)
  }
}

const aiSearch = {
  addonId: 'elephant.ai-search',
  async run({ page, fixture, expect, context }) {
    const provider = await configureInterceptedProvider({ page, fixture, expect })
    await openSettingsSection(page, 'AI')

    const settings = page.locator('.elephant-ai-settings')
    await settings.getByRole('button', { name: 'Search', exact: true }).click()
    const search = settings.locator('.elephant-search-settings')
    await expect(search).toBeVisible()
    await search.getByRole('button', { name: 'Rebuild index and vectors', exact: true }).click()
    await expect(search.locator('.elephant-search-status')).toContainText('2 semantic vectors', { timeout: 15000 })

    const hits = await page.evaluate(async () => {
      const resource = window.__ELEPHANT_ADDON_HOST__?.get('search.provider')
      return resource?.query?.('alpha', { limit: 5 }) || []
    })
    expect(hits).toEqual(expect.arrayContaining([
      expect.objectContaining({ title: 'Alpha note', path: 'Alpha.md', engine: 'knowledge-provider' })
    ]))

    const resultLimit = search
      .locator('.elephant-search-field')
      .filter({ hasText: /^\s*Result limit\s*$/ })
      .locator('input')
    await resultLimit.fill('5')
    await resultLimit.blur()
    await search.getByRole('button', { name: 'Clear lexical fallback', exact: true }).click()
    await expect(search.locator('.elephant-search-status')).toContainText('0 local lexical notes')

    await expect
      .poll(() => persistedAddonValue(fixture, this.addonId, 'search-config-v3'), { timeout: 5000 })
      .toEqual(expect.objectContaining({ limit: 5 }))
    await expect
      .poll(() => persistedAddonValue(fixture, this.addonId, 'search-index-v2'), { timeout: 5000 })
      .toBeUndefined()

    const resourceStatus = await page.evaluate(async () => {
      const resource = window.__ELEPHANT_ADDON_HOST__?.get('search.provider')
      return resource?.status?.() || null
    })
    expect(resourceStatus).toMatchObject({ enabled: true, notesIndexed: 0, semanticVectors: 2 })
    const trace = readAddonHttpTrace(fixture)
    expect(trace.some((entry) => entry.path === '/v1/embeddings')).toBe(true)
    await context.checkpoint('ai-search-intercepted-usage', { provider, hits, resourceStatus, trace })

    await page.reload()
    await page.waitForSelector('.en-library-grid', { state: 'visible' })
    await ensureAddon(page, this.addonId)
    const reloadedSettings = await openSettingsSection(page, 'AI')
    const reloadedAiSettings = reloadedSettings.locator('.elephant-ai-settings')
    await reloadedAiSettings.getByRole('button', { name: 'Search', exact: true }).click()
    await expect(
      reloadedAiSettings
        .locator('.elephant-search-settings')
        .locator('.elephant-search-field')
        .filter({ hasText: /^\s*Result limit\s*$/ })
        .locator('input')
    ).toHaveValue('5')
    await closeSettings(page)
  }
}

module.exports = [ai, aiChat, aiSearch]
