export const MUYA_RUNTIME_FLAGS = Object.freeze({
  rust: 'rust'
})

export const defaultMuyaRuntimeMode = () => MUYA_RUNTIME_FLAGS.rust

export const readMuyaRuntimeMode = () => MUYA_RUNTIME_FLAGS.rust

export const isMuyaRuntimeEnabled = () => true
export const isMuyaRuntimeActive = () => true
export const isMuyaRustRuntime = () => true
