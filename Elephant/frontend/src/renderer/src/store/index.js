import { createPinia, defineStore } from 'pinia'
import { isPortableRuntime } from '../platform/preferenceStorage'

const pinia = createPinia()

// Main store for global states
export const useMainStore = defineStore('main', {
  state: () => ({
    platform: isPortableRuntime()
      ? window.navigator.platform || 'portable'
    : window.tauri?.process?.platform || window.navigator.platform || 'portable', // platform of system `darwin` | `win32` | `linux`
    appVersion: window.__MARKTEXT_VERSION_STRING__ ||
      window.tauri?.process?.env?.MARKTEXT_VERSION_STRING || '', // MarkText version string
    windowActive: true, // whether current window is active or focused
    init: false // whether MarkText is initialized
  }),

  getters: {
    // Add any getters here if needed
  },

  actions: {
    SET_WIN_STATUS(status) {
      this.windowActive = status
    },

    SET_INITIALIZED() {
      this.init = true
    },

    LISTEN_WIN_STATUS() {
      window.tauri?.ipcRenderer?.on('mt::window-active-status', (e, { status }) => {
        this.windowActive = status
      })
    }
  }
})

export default pinia
