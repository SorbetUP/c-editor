import { defineStore } from 'pinia'
import { DEFAULT_THEME, CANVAS_THEME_DEFS, resolveCanvasTheme } from '../graph/graphThemes'

export const useCanvasStore = defineStore('canvas', {
  state: () => ({
    controller: null,
    localGraphOpen: false,
    localGraphCenter: null,
    localGraphDepth: 1,
    pendingFocusNoteId: null,
    pendingCamera: null,
    themeId: 'midnight',
    appMode: 'dark',
    theme: { ...DEFAULT_THEME },
    edgeThreshold: 0,
    canvasPositions: {},
  }),

  getters: {
    activeTheme(state) {
      const def = CANVAS_THEME_DEFS.find((t) => t.id === state.themeId) || CANVAS_THEME_DEFS.find((t) => t.id === 'midnight')
      return resolveCanvasTheme(def, state.appMode)
    },
  },

  actions: {
    registerController(ctrl) {
      this.controller = ctrl
    },
    unregisterController() {
      this.controller = null
    },
    setThemeId(id) {
      const def = CANVAS_THEME_DEFS.find((t) => t.id === id)
      if (!def) return
      this.themeId = id
      this.theme = resolveCanvasTheme(def, this.appMode)
    },
    setTheme(themeOrDef) {
      if (themeOrDef && themeOrDef.id) {
        this.setThemeId(themeOrDef.id)
        return
      }
      this.theme = { ...themeOrDef }
    },
    setAppMode(mode) {
      this.appMode = mode === 'light' ? 'light' : 'dark'
      const def = CANVAS_THEME_DEFS.find((t) => t.id === this.themeId) || CANVAS_THEME_DEFS.find((t) => t.id === 'midnight')
      this.theme = resolveCanvasTheme(def, this.appMode)
    },
    setEdgeThreshold(value) {
      this.edgeThreshold = value
    },
    openLocalGraph(noteId, depth = 1) {
      this.localGraphOpen = true
      this.localGraphCenter = noteId
      this.localGraphDepth = depth
    },
    closeLocalGraph() {
      this.localGraphOpen = false
      this.localGraphCenter = null
    },
    navigateLocalGraph(noteId) {
      this.localGraphCenter = noteId
    },
    setPendingFocusNoteId(id) {
      this.pendingFocusNoteId = id
    },
    setPendingCamera(cameraState) {
      this.pendingCamera = cameraState
    },
    clearPendingFocus() {
      this.pendingFocusNoteId = null
    },
    clearPendingCamera() {
      this.pendingCamera = null
    },
    persistPositions(vaultId, positions) {
      try {
        const key = `elephantnote:canvas:${vaultId}`
        window.localStorage.setItem(key, JSON.stringify(positions))
        this.canvasPositions = { ...positions }
      } catch { /* quota exceeded */ }
    },
    loadPositions(vaultId) {
      try {
        const key = `elephantnote:canvas:${vaultId}`
        const raw = window.localStorage.getItem(key)
        this.canvasPositions = raw ? JSON.parse(raw) : {}
      } catch {
        this.canvasPositions = {}
      }
    }
  }
})
