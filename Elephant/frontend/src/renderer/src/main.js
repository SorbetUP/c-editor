import './platform/bootstrapGlobals'
import elephantLogoUrl from '../../../../assets/static/icon.png'
import './mobile-shell-layout.css'
import './mobile-android.css'
import './mobile-native-ux.css'
import './mobile-editor-round2.css'
import './mobile-library-chrome.css'
import './platform/mobileVaultBridge'
import './platform/mobileEditorRuntime'
import './platform/mobileInteractionRuntime'
import './platform/mobileLibraryChromeRuntime'
import './addons/officialAddonCatalogBridge'
import { createApp, nextTick } from 'vue'
import { createRouter, createWebHashHistory } from 'vue-router'
import bootstrapRenderer from './bootstrap'
import axios from './axios'
import pinia, { useMainStore } from './store'
import './assets/symbolIcon'
import { installTauriRuntimeBridge } from './platform/tauriRuntimeBridge'
import { ensureRendererPathFacade } from './platform/rendererPathFacade'
import { installTauriFileUtilsPathGuards } from './platform/tauriFileUtilsPathGuards'
import { installTauriElephantNoteBridge } from './platform/tauriElephantNoteBridge'
import { installTauriSearchLifecycleBridge } from './platform/tauriSearchLifecycleBridge'
import { installTauriSearchRuntimeGuards } from './platform/tauriSearchRuntimeGuards'
import { installTauriSearchConceptFallback } from './platform/tauriSearchConceptFallback'
import { installPiProviderBridge } from './platform/piProviderInterface'
import { installTauriMarkTextSaveBridge } from './platform/tauriMarkTextSaveBridge'
import { installTauriLocalIpcBridge } from './platform/tauriLocalIpcBridge'
import { installSlashMenuDiagnostics } from './platform/slashMenuDiagnostics'
import { installWritingCommandBridge } from './platform/writingCommandBridge'
import { installNoteCitationRuntime } from './platform/noteCitationRuntime'
import { installNoteCitationSelectionGuard } from './platform/noteCitationSelectionGuard'
import { restorePortableWindowState, savePortableWindowState } from './platform/windowState'
import { installRendererDiagnostics, pushDiagnosticLog } from './platform/rendererDiagnostics'
import { installStoreDiagnostics } from './platform/storeDiagnostics'
import { installAcceptanceTestBridge } from './platform/acceptanceTestBridge'
import { installAddonSystem } from './addons'
import { activateCoreFeature } from './addons/coreFeatures'
import { addonPacksCoreFeature } from './addons/builtin/addonProfiles'
import { excalidrawCoreFeature } from './addons/builtin/excalidraw'
import { installAddonPermissionConsentGuard } from './addons/permissionConsentGuard'
import { appDataDir } from '@tauri-apps/api/path'

import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import en from 'element-plus/es/locale/lang/en'

import i18nPlugin from './i18n'

import services from './services/index'
import { elephantnoteClient } from 'elephant-front/services/elephantnoteClient'
import createRoutes from './router'
import { resolveRendererRoutes } from './router/resolveRendererRoutes'
import Main from './Main.vue'

import './assets/styles/index.css'
import './assets/styles/printService.css'
import 'elephant-front/styles/runtime-layout-fixes.css'

const clearBootstrapFileUtilsFallbackForTauri = () => {
  if (window.__TAURI__ && window.fileUtils?.__elephantnoteBootstrapFallback) delete window.fileUtils
}

const renderStartupFailure = (error) => {
  const root = document.getElementById('app')
  if (!root) return
  const message = error?.stack || error?.message || String(error || 'Unknown renderer startup failure')
  root.replaceChildren()
  const surface = document.createElement('main')
  surface.id = 'elephant-startup'
  surface.dataset.error = 'true'
  surface.setAttribute('role', 'alert')

  const mark = document.createElement('div')
  mark.className = 'elephant-startup-mark'
  mark.setAttribute('aria-hidden', 'true')
  installStartupLogo(mark)

  const title = document.createElement('strong')
  title.textContent = 'Elephant n’a pas pu démarrer'

  const details = document.createElement('span')
  details.textContent = message

  surface.append(mark, title, details)
  root.append(surface)
}

const installStartupLogo = (mark = document.querySelector('.elephant-startup-mark')) => {
  if (!mark) return
  const logo = document.createElement('img')
  logo.src = elephantLogoUrl
  logo.alt = ''
  mark.replaceChildren(logo)
}

installStartupLogo()
installRendererDiagnostics()
globalThis.marktext = {}
clearBootstrapFileUtilsFallbackForTauri()
installTauriRuntimeBridge()
ensureRendererPathFacade()
installTauriFileUtilsPathGuards()
installTauriElephantNoteBridge()
installTauriSearchLifecycleBridge({ target: globalThis, client: elephantnoteClient })
installTauriSearchRuntimeGuards()
installTauriSearchConceptFallback()
installPiProviderBridge()
installTauriMarkTextSaveBridge()
installTauriLocalIpcBridge()
installSlashMenuDiagnostics()
installWritingCommandBridge()

const bootstrapTauriRuntime = async() => {
  pushDiagnosticLog('info', 'bootstrapTauriRuntime:start', { runtime: window.__MARKTEXT_RUNTIME__ })
  try {
    const userDataPath = await appDataDir()
    window.__MARKTEXT_USER_DATA_PATH__ = userDataPath
    window.__MARKTEXT_WINDOW_ID__ = 1
    window.__MARKTEXT_WINDOW_TYPE__ = 'editor'
    pushDiagnosticLog('info', 'bootstrapTauriRuntime:appDataDir', { userDataPath })
  } catch (error) {
    pushDiagnosticLog('warn', 'bootstrapTauriRuntime:appDataDir fallback', error)
    window.__MARKTEXT_USER_DATA_PATH__ = window.__MARKTEXT_USER_DATA_PATH__ || window.path.resolve('/tmp', 'elephantnote')
    window.__MARKTEXT_WINDOW_ID__ = 1
    window.__MARKTEXT_WINDOW_TYPE__ = 'editor'
  }

  bootstrapRenderer()
  pushDiagnosticLog('info', 'bootstrapRenderer:done', { env: globalThis.marktext?.env })
  restorePortableWindowState().catch((error) => pushDiagnosticLog('warn', 'window-state restore failed', error))
  window.addEventListener('beforeunload', () => {
    void savePortableWindowState().catch((error) => pushDiagnosticLog('warn', 'window-state save failed', error))
  })
}

const bootstrapForRuntime = async(runtime) => {
  if (runtime !== 'tauri') {
    const message = `ElephantNote renderer is Tauri-only; unsupported runtime "${runtime}"`
    pushDiagnosticLog('error', 'bootstrapTauriRuntime:unsupported-runtime', { runtime })
    throw new Error(message)
  }
  await bootstrapTauriRuntime()
  return globalThis.marktext?.env?.type || window.__MARKTEXT_WINDOW_TYPE__ || 'editor'
}

const installCoreFeatures = async(addonManager) => {
  for (const feature of [addonPacksCoreFeature, excalidrawCoreFeature]) {
    try {
      await activateCoreFeature(addonManager, feature)
      pushDiagnosticLog('info', '[core-feature] ready', { id: feature.id })
    } catch (error) {
      pushDiagnosticLog('error', '[core-feature] failed', {
        id: feature.id,
        error: error?.message || String(error)
      })
      throw error
    }
  }
}

const mountRendererApp = async(runtime, windowType) => {
  const router = createRouter({
    history: createWebHashHistory(),
    routes: resolveRendererRoutes(createRoutes, windowType)
  })

  const app = createApp(Main)
  installRendererDiagnostics(app)
  app.use(pinia)
  app.use(router)
  app.use(ElementPlus, { locale: en })
  app.use(i18nPlugin)
  app.config.globalProperties.$http = axios
  app.config.globalProperties.$services = services

  const mainStore = useMainStore(pinia)
  if (!mainStore.init) {
    mainStore.SET_INITIALIZED()
    pushDiagnosticLog('info', 'renderer shell initialized', { runtime })
  }

  const addonManager = installAddonSystem(app, {
    router,
    pinia,
    services,
    runtime,
    logger: {
      info: (message, payload) => pushDiagnosticLog('info', message, payload),
      warn: (message, payload) => pushDiagnosticLog('warn', message, payload),
      error: (message, payload) => pushDiagnosticLog('error', message, payload)
    }
  })
  installAddonPermissionConsentGuard(addonManager)
  await installCoreFeatures(addonManager)
  await router.isReady()
  app.mount('#app')
  await nextTick()
  document.documentElement.dataset.elephantMounted = 'true'
  document.getElementById('app')?.setAttribute('aria-label', 'Elephant application ready')
  pushDiagnosticLog('info', 'renderer Vue shell mounted', {
    route: router.currentRoute.value.fullPath,
    hasVisibleShell: Boolean(document.querySelector('.en-empty-vault, .en-shell'))
  })
  installStoreDiagnostics()
  const acceptanceEnabled = typeof window.__TAURI__?.core?.invoke === 'function'
    ? await window.__TAURI__.core.invoke('tauri_acceptance_enabled').catch(() => false)
    : false
  if (acceptanceEnabled === true) installAcceptanceTestBridge({ pinia })
  installNoteCitationRuntime({ pinia })
  installNoteCitationSelectionGuard(window)
}

const startRendererApp = async() => {
  const runtime = 'tauri'
  window.__MARKTEXT_RUNTIME__ = runtime
  const windowType = await bootstrapForRuntime(runtime)
  await mountRendererApp(runtime, windowType)
}

void startRendererApp().catch((error) => {
  pushDiagnosticLog('error', 'renderer startup failed', error)
  renderStartupFailure(error)
  console.error('[Elephant] renderer startup failed', error)
})

if (import.meta.hot) import.meta.hot.accept()
