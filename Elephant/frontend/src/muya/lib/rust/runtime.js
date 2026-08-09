import { MuyaRustBridge } from './bridge'
import { createDomPatchAdapter } from './domRenderer'
import { createRustInputController } from './inputController'
import { createBundledMuyaRustEngine } from './wasmFactory'

const noop = () => {}

const composeCallbacks = (...callbacks) => {
  const active = callbacks.filter((callback) => typeof callback === 'function')
  if (!active.length) return noop
  return async (...args) => {
    for (const callback of active) await callback(...args)
  }
}

const resolveFactory = (config) => {
  if (typeof config.factory === 'function') return config.factory
  if (config.useBundledWasm) return createBundledMuyaRustEngine
  throw new TypeError(
    'experimentalRustEditor requires factory(markdown) or useBundledWasm: true.'
  )
}

const resolveDomContainer = (muya, config) => {
  const candidate =
    typeof config.domContainer === 'function' ? config.domContainer(muya) : config.domContainer
  if (candidate == null) return null
  if (!candidate?.replaceChildren || !candidate?.addEventListener) {
    throw new TypeError('experimentalRustEditor.domContainer must resolve to an Element.')
  }
  return candidate
}

export const initializeExperimentalRustRuntime = async (muya, config, reportError) => {
  const factory = resolveFactory(config)
  const engine = await factory(muya.markdown)
  const domContainer = resolveDomContainer(muya, config)
  const adapter = domContainer
    ? createDomPatchAdapter(domContainer, { onRender: config.onRender })
    : null

  const bridge = new MuyaRustBridge(engine, {
    applyPatches: composeCallbacks(adapter?.applyPatches, config.applyPatches),
    applySnapshot: composeCallbacks(adapter?.applySnapshot, config.applySnapshot),
    onSelection: composeCallbacks(adapter?.onSelection, config.onSelection),
    onError: reportError
  })

  await bridge.snapshot()

  let inputController = null
  if (adapter && config.captureInput) {
    if (!domContainer.hasAttribute('contenteditable')) {
      domContainer.setAttribute('contenteditable', 'true')
    }
    inputController = createRustInputController(domContainer, bridge, adapter.renderer, {
      onError: reportError,
      onFileDrop: config.onFileDrop,
      onUriDrop: config.onUriDrop,
      onImageClick: config.onImageClick
    })
  }

  return {
    bridge,
    renderer: adapter?.renderer || null,
    inputController,
    domContainer,
    shadow: !adapter && typeof config.applyPatches !== 'function',
    destroy() {
      inputController?.detach()
    }
  }
}
