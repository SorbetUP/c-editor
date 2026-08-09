let modulePromise = null

const loadBundle = async (initInput) => {
  if (!modulePromise) {
    modulePromise = import('muya-rust-wasm-bundle')
      .then(async (module) => {
        await module.default(initInput)
        return module
      })
      .catch((error) => {
        modulePromise = null
        throw error
      })
  }
  return modulePromise
}

export const createBundledMuyaRustEngine = async (markdown, initInput) => {
  const module = await loadBundle(initInput)
  return new module.MuyaEditor(String(markdown || ''))
}
