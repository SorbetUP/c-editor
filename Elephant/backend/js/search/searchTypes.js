export const SEARCH_STATUSES = Object.freeze({
  DISABLED: 'disabled',
  NOT_INITIALIZED: 'not_initialized',
  MODEL_MISSING: 'model_missing',
  MODEL_LOADING: 'model_loading',
  INDEXING: 'indexing',
  READY: 'ready',
  ERROR: 'error'
})

export const SEARCH_MODES = Object.freeze({
  SMART: 'smart',
  EXACT: 'exact',
  SEMANTIC: 'semantic'
})

export const SEARCH_MATCH_TYPES = Object.freeze({
  SEMANTIC: 'semantic',
  KEYWORD: 'keyword',
  HYBRID: 'hybrid'
})
