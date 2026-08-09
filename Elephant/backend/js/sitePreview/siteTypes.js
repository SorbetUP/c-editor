export const SITE_PREVIEW_STATUS = Object.freeze({
  IDLE: 'idle',
  PREPARING: 'preparing',
  BUILDING: 'building',
  SERVING: 'serving',
  READY: 'ready',
  ERROR: 'error',
  STOPPED: 'stopped'
})

export const SITE_BUILD_MODE = Object.freeze({
  PREVIEW: 'preview',
  STATIC_EXPORT: 'static-export'
})
