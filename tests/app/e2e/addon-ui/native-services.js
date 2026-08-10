const {
  ensureAddon,
  openAddonView,
  openSettingsSection
} = require('./usage-harness')

const invokeTauri = (page, command, payload = {}) => page.evaluate(async ({ command: name, payload: args }) => {
  const invoke = globalThis.__TAURI__?.core?.invoke
  if (typeof invoke !== 'function') return { ok: false, error: 'Tauri command API is unavailable' }
  try {
    return { ok: true, value: await invoke(name, args) }
  } catch (error) {
    return { ok: false, error: error?.message || String(error) }
  }
}, { command, payload })

const callResource = (page, name, method, payload) => page.evaluate(async ({ name: resourceName, method: resourceMethod, payload: args }) => {
  const resource = globalThis.__ELEPHANT_ADDONS__?.host?.get?.(resourceName)
  if (!resource || typeof resource[resourceMethod] !== 'function') {
    return { ok: false, error: `Addon resource method is unavailable: ${resourceName}.${resourceMethod}` }
  }
  try {
    return { ok: true, value: await resource[resourceMethod](args) }
  } catch (error) {
    return { ok: false, error: error?.message || String(error) }
  }
}, { name, method, payload })

const checkpoint = async (context, fixture, name, value) => {
  if (typeof context?.checkpoint !== 'function') return
  await context.checkpoint(name, {
    vaultRoot: fixture?.vaultRoot || '',
    ...value
  })
}

const serviceState = async (page, addonId) => {
  const response = await invokeTauri(page, 'tauri_addons_service_status', { addonId })
  if (!response.ok) return { ...response, mocked: false }
  const value = response.value || {}
  return {
    ...response,
    addonId: value.addonId,
    running: value.running === true,
    mocked: value.mocked === true,
    error: value.error || ''
  }
}

const assertServiceState = (expect, state, addonId) => {
  if (!state.ok) {
    expect(state.error).toBeTruthy()
    return
  }
  expect(state.addonId).toBe(addonId)
  if (state.error) expect(state.running).toBe(false)
  else expect(state.running).toBe(true)
}

const errorMessage = (response) => response.ok
  ? String(response.value?.error || '').trim()
  : String(response.error || '').trim()

const openModels = {
  addonId: 'elephant.open-models',
  async run({ page, fixture, expect, context }) {
    await ensureAddon(page, this.addonId)
    const view = await openAddonView(page, 'Models', '.elephant-models-package')
    const service = await serviceState(page, this.addonId)
    assertServiceState(expect, service, this.addonId)

    const status = await callResource(page, 'models.provider', 'status')
    const mocked = service.mocked || status.value?.mocked === true
    if (!status.ok || errorMessage(status)) {
      const message = errorMessage(status)
      const visibleError = view.locator('.elephant-package-error')
      await expect(visibleError).toBeVisible()
      if (message) await expect(visibleError).toContainText(message)
    } else if (mocked) {
      // The renderer fixture is deliberately not inference evidence.
      await expect(view).toContainText('No local model installed.')
      await expect(view.locator('input[placeholder="Hugging Face repository or direct GGUF URL"]')).toBeVisible()
      await expect(view.getByRole('button', { name: 'Download', exact: true })).toBeVisible()
    } else {
      expect(status.value?.owner).toBe(this.addonId)
      expect(status.value?.running).toBe(true)
      await expect(view.locator('.elephant-model-list')).toBeVisible()

      // Exercise the real error path without downloading a model or claiming inference.
      const input = view.locator('input[placeholder="Hugging Face repository or direct GGUF URL"]')
      await input.fill('https://127.0.0.1:1/elephant-ui-missing.gguf')
      await view.getByRole('button', { name: 'Download', exact: true }).click()
      await expect(view.locator('.elephant-package-error')).toBeVisible()
    }

    const config = await callResource(page, 'ai.config', 'get')
    expect(config.ok).toBe(true)
    expect(config.value?.localAi?.enabled).toBe(true)
    expect(config.value?.localAi?.allowHuggingFaceDownloads).toBe(true)
    await checkpoint(context, fixture, 'open-models-native-state', {
      serviceRunning: service.running,
      serviceMocked: mocked,
      modelStatusError: errorMessage(status),
      localAiEnabled: config.value?.localAi?.enabled === true
    })

    await page.reload()
    await page.waitForSelector('.en-library-grid', { state: 'visible' })
    await ensureAddon(page, this.addonId)
    const reloadedConfig = await callResource(page, 'ai.config', 'get')
    expect(reloadedConfig.ok).toBe(true)
    expect(reloadedConfig.value?.localAi?.enabled).toBe(true)
    const reloadedView = await openAddonView(page, 'Models', '.elephant-models-package')
    await expect(reloadedView).toBeVisible()
  }
}

const codex = {
  addonId: 'elephant.codex-connection',
  async run({ page, fixture, expect, context }) {
    await ensureAddon(page, this.addonId)
    const settings = await openSettingsSection(page, 'AI')
    const card = settings.locator('.elephant-codex-settings')
    await expect(card).toBeVisible()
    const service = await serviceState(page, this.addonId)
    assertServiceState(expect, service, this.addonId)
    const status = await invokeTauri(page, 'tauri_addons_service_call', {
      addonId: this.addonId,
      method: 'codex.status',
      params: {}
    })
    const message = errorMessage(status)
    const primary = card.getByRole('button', { name: /^(Connect|Disconnect)$/ })
    await expect(primary).toHaveCount(1)

    if (message) {
      await expect(card.locator('.elephant-package-error')).toBeVisible()
      await expect(card.locator('.elephant-package-error')).toContainText(message)
      await expect(card).toContainText('Disconnected')
    } else if (service.mocked || status.value?.mocked === true) {
      // A mocked status is not authentication and must remain actionable.
      await expect(card).toContainText('Disconnected')
      await expect(primary).toHaveText('Connect')
    } else if (status.value?.connected === false) {
      await expect(card).toContainText('Disconnected')
      await expect(primary).toHaveText('Connect')
      if (status.value?.error) {
        await expect(card.locator('.elephant-package-error')).toContainText(status.value.error)
      }
    }

    await card.getByRole('button', { name: 'Refresh', exact: true }).click()
    await expect(card).toContainText('ChatGPT subscription')
    await checkpoint(context, fixture, 'codex-native-state', {
      serviceRunning: service.running,
      serviceMocked: service.mocked || status.value?.mocked === true,
      runtimeDetected: status.value?.detected === true,
      reportedConnectionState: status.value?.connected === true
        ? 'reported-connected-without-proof'
        : status.value?.connected === false ? 'reported-disconnected' : 'unknown',
      statusError: message
    })

    await page.reload()
    await page.waitForSelector('.en-library-grid', { state: 'visible' })
    await ensureAddon(page, this.addonId)
    const reloadedSettings = await openSettingsSection(page, 'AI')
    const reloadedCard = reloadedSettings.locator('.elephant-codex-settings')
    await expect(reloadedCard).toBeVisible()
    // Reload proves that the package UI/service state is restored, not auth success.
    await expect(reloadedCard.getByRole('button', { name: /^(Connect|Disconnect)$/ })).toHaveCount(1)
  }
}

const sync = {
  addonId: 'elephant.sync',
  async run({ page, fixture, expect, context }) {
    await ensureAddon(page, this.addonId)
    const settings = await openSettingsSection(page, 'Sync')
    const panel = settings.locator('.elephant-sync-settings')
    await expect(panel).toBeVisible()
    await expect(panel).toContainText('Invite another device')
    await expect(panel).toContainText('Join an existing vault')
    await expect(panel.getByRole('button', { name: 'Create invitation', exact: true })).toBeVisible()
    await expect(panel.getByRole('button', { name: 'Pair this device', exact: true })).toBeVisible()

    const service = await serviceState(page, this.addonId)
    assertServiceState(expect, service, this.addonId)
    const before = await callResource(page, 'sync.native-service', 'status')
    const mocked = service.mocked || before.value?.mocked === true
    await expect(panel).toContainText('State:')

    await panel.getByRole('button', { name: 'Create invitation', exact: true }).click()
    const feedback = panel.locator('.elephant-sync-feedback')
    await expect(feedback).toBeVisible()
    const outgoing = panel.locator('.elephant-sync-invite-output')
    const invitation = await outgoing.inputValue()
    if (mocked) {
      // The fixture returns no invitation payload; that is an actionable failure, not pairing.
      expect(invitation).toBe('')
      await expect(feedback).toHaveAttribute('data-kind', 'error')
      await expect(feedback).toContainText('empty invitation')
    } else if (invitation) {
      await expect(feedback).toHaveAttribute('data-kind', 'success')
      await expect(panel.getByRole('button', { name: 'Copy', exact: true })).toBeEnabled()
    } else {
      await expect(feedback).toHaveAttribute('data-kind', 'error')
    }

    const incoming = panel.locator('.elephant-sync-invite-input')
    await incoming.fill('too-short')
    await panel.getByRole('button', { name: 'Pair this device', exact: true }).click()
    await expect(feedback).toHaveAttribute('data-kind', 'error')
    await expect(feedback).toContainText('manual pairing code is too short')

    await checkpoint(context, fixture, 'sync-native-state', {
      serviceRunning: service.running,
      serviceMocked: mocked,
      invitationCreated: Boolean(invitation),
      pendingInvitesBefore: before.value?.pendingInvites ?? null
    })

    await page.reload()
    await page.waitForSelector('.en-library-grid', { state: 'visible' })
    await ensureAddon(page, this.addonId)
    const reloaded = await openSettingsSection(page, 'Sync')
    await expect(reloaded.locator('.elephant-sync-settings')).toContainText('Invite another device')
    // No peer was accepted; this deliberately does not claim a sync completed.
    const after = await callResource(page, 'sync.native-service', 'status')
    if (!mocked && invitation && Number.isFinite(Number(before.value?.pendingInvites))) {
      expect(Number(after.value?.pendingInvites || 0)).toBeGreaterThanOrEqual(Number(before.value.pendingInvites))
    }
  }
}

module.exports = [openModels, codex, sync]
