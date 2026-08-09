<template>
  <ChatgptSubscriptionCard
    :status="status"
    :rate-limit-rows="rateLimitRows"
    :reset-credits="resetCredits"
    :available-reset-count="availableResetCount"
    :busy="busy"
    :reset-busy="resetBusy"
    :message="message"
    :login-challenge="loginChallenge"
    @connect="connect"
    @disconnect="disconnect"
    @open-auth="openExternal"
    @consume-reset="consumeReset"
  />
</template>

<script setup>
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import ChatgptSubscriptionCard from 'elephant-front/components/settings/ChatgptSubscriptionCard.vue'
import { buildCodexRateLimitRows, buildCodexResetCredits, getCodexResetAvailableCount } from 'elephant-front/components/settings/codexRateLimits'

const busy = ref(false)
const resetBusy = ref(false)
const message = ref('')
const status = ref({ installed: false, running: false, connected: false, account: null, version: '', error: '' })
const rateLimits = ref(null)
const loginChallenge = ref({})
let unlistenCodex = null

const rateLimitRows = computed(() => buildCodexRateLimitRows(rateLimits.value || {}))
const resetCredits = computed(() => buildCodexResetCredits(rateLimits.value || {}))
const availableResetCount = computed(() => getCodexResetAvailableCount(rateLimits.value || {}))

const invoke = (command, payload = {}) => {
  const fn = globalThis.window?.__TAURI__?.core?.invoke
  if (typeof fn !== 'function') throw new Error(`Tauri command API is unavailable for ${command}`)
  return fn(command, payload)
}
const invokeCodex = (codexOperation, payload = {}) => invoke('tauri_rag_chat', { payload: { codexOperation, ...payload } })
const openExternal = async (url) => {
  if (!url) return
  const { openUrl } = await import('@tauri-apps/plugin-opener')
  await openUrl(url)
}

const refresh = async () => {
  busy.value = true
  try {
    status.value = await invokeCodex('status')
    message.value = status.value.error || (status.value.connected
      ? ''
      : status.value.installed
        ? 'Codex is installed but no ChatGPT account is connected.'
        : 'The bundled Codex runtime is missing from this build.')
    rateLimits.value = status.value.connected
      ? await invokeCodex('rateLimits').catch(() => null)
      : null
  } catch (error) {
    status.value = {
      installed: false,
      running: false,
      connected: false,
      error: error instanceof Error ? error.message : String(error)
    }
    message.value = status.value.error
  } finally {
    busy.value = false
  }
}

const connect = async () => {
  busy.value = true
  try {
    const challenge = await invokeCodex('login', { flow: 'browser' })
    loginChallenge.value = challenge || {}
    const url = challenge?.authUrl || challenge?.verificationUrl
    if (url) await openExternal(url)
    message.value = challenge?.userCode
      ? `Enter device code ${challenge.userCode}.`
      : 'Authentication opened in your browser.'
  } catch (error) {
    message.value = error instanceof Error ? error.message : String(error)
  } finally {
    busy.value = false
  }
}

const disconnect = async () => {
  busy.value = true
  try {
    await invokeCodex('logout')
    loginChallenge.value = {}
    await refresh()
    globalThis.dispatchEvent?.(new CustomEvent('elephantnote:codex-addon-disconnected'))
  } catch (error) {
    message.value = error instanceof Error ? error.message : String(error)
  } finally {
    busy.value = false
  }
}

const consumeReset = async (creditId) => {
  if (!creditId || resetBusy.value) return
  resetBusy.value = true
  message.value = ''
  try {
    const idempotencyKey = globalThis.crypto?.randomUUID?.() || `elephantnote-${Date.now()}-${Math.random().toString(16).slice(2)}`
    const result = await invokeCodex('consumeRateLimitReset', { creditId, idempotencyKey })
    const outcome = String(result?.outcome || '')
    message.value = ({
      reset: 'Usage limits were reset.',
      nothingToReset: 'No current usage window is eligible for a reset.',
      noCredit: 'No reset credit is available.',
      alreadyRedeemed: 'This reset was already applied.'
    })[outcome] || 'Reset request completed.'
    rateLimits.value = await invokeCodex('rateLimits')
  } catch (error) {
    message.value = error instanceof Error ? error.message : String(error)
  } finally {
    resetBusy.value = false
  }
}

onMounted(async () => {
  const { listen } = await import('@tauri-apps/api/event')
  unlistenCodex = await listen('elephantnote:codex:event', (event) => {
    const method = event?.payload?.method
    if (method === 'account/login/completed' || method === 'account/updated' || method === 'account/rateLimits/updated') {
      refresh()
    }
  }).catch(() => null)
  await refresh()
})

onBeforeUnmount(() => {
  if (typeof unlistenCodex === 'function') unlistenCodex()
})
</script>
