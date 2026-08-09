const DETACHED_TTL_MS = 1000
const RUN_BUTTON_SELECTOR = '.en-code-native-run'
const OUTPUT_SELECTOR = 'elephant-code-output'

const buttonFor = (element) => element?.closest?.('pre')?.querySelector?.(`:scope > ${RUN_BUTTON_SELECTOR}`) || null

const stateFor = (runtime, element) => {
  if (!element) return null
  const direct = [...runtime.states.values()].find((state) => state.element === element || state.output === element)
  if (direct) return direct
  const blockKey = element.dataset?.blockKey || element.closest?.('pre')?.id || ''
  return [...runtime.states.values()].find((state) => state.key.endsWith(`:${blockKey}`)) || null
}

const syncRunButton = (state) => {
  if (!state) return
  const button = state.element?.matches?.(RUN_BUTTON_SELECTOR)
    ? state.element
    : buttonFor(state.output)
  if (!button) return

  state.element = button
  button.__elephantCodeStateKey = state.key
  const running = state.status === 'running' || state.status === 'stopping'
  button.classList.toggle('is-running', running)
  button.setAttribute('aria-label', running ? 'Stop code execution' : 'Run code block')
  button.setAttribute('title', running ? 'Stop code execution' : 'Run code block')
  button.disabled = state.status === 'stopping'
}

export const installExecutableCodeNativeLifecycle = (runtime, target = globalThis) => {
  if (!runtime || runtime.__nativeLifecycleInstalled) return runtime
  runtime.__nativeLifecycleInstalled = true

  const documentTarget = target.document || document
  const clearDetachTimer = (state) => {
    if (!state?.detachTimer) return
    clearTimeout(state.detachTimer)
    state.detachTimer = null
  }
  const stopDetachedExecution = async (state) => {
    if (!state?.executionId) return
    try {
      await target.elephantnote?.programs?.stop?.({ executionId: state.executionId })
    } catch (error) {
      console.warn('[Code:UI] detached execution stop failed', {
        executionId: state.executionId,
        error: error?.message || String(error)
      })
    }
  }

  const originalRegisterRunButton = runtime.registerRunButton.bind(runtime)
  runtime.registerRunButton = (element) => {
    originalRegisterRunButton(element)
    const state = stateFor(runtime, element)
    if (state) {
      element.__elephantCodeStateKey = state.key
      clearDetachTimer(state)
    }
    syncRunButton(state)
  }

  const originalUnregisterRunButton = runtime.unregisterRunButton.bind(runtime)
  runtime.unregisterRunButton = (element) => {
    originalUnregisterRunButton(element)
    const state = stateFor(runtime, element)
    if (state?.element === element) state.element = null
  }

  const originalRegisterOutput = runtime.registerOutput.bind(runtime)
  runtime.registerOutput = (element) => {
    originalRegisterOutput(element)
    const button = buttonFor(element)
    if (button) originalRegisterRunButton(button)
    const state = stateFor(runtime, element) || stateFor(runtime, button)
    if (!state) return
    state.output = element
    element.__elephantCodeStateKey = state.key
    if (button) {
      state.element = button
      button.__elephantCodeStateKey = state.key
    }
    clearDetachTimer(state)
    syncRunButton(state)
    element.renderState?.(state, runtime)
  }

  const originalUnregisterOutput = runtime.unregisterOutput.bind(runtime)
  runtime.unregisterOutput = (element) => {
    const key = element.__elephantCodeStateKey
    originalUnregisterOutput(element)
    if (!key) return
    const state = runtime.states.get(key)
    if (!state) return

    if (state.output === element) state.output = null
    clearDetachTimer(state)
    state.detachTimer = setTimeout(() => {
      if (state.output?.isConnected || state.element?.isConnected) return
      void stopDetachedExecution(state)
      runtime.states.delete(key)
    }, DETACHED_TTL_MS)
  }

  const syncAfter = (operation, element) => {
    const task = operation(element)
    syncRunButton(stateFor(runtime, element))
    Promise.resolve(task).finally(() => syncRunButton(stateFor(runtime, element)))
    return task
  }

  const originalRun = runtime.run.bind(runtime)
  runtime.run = (element) => syncAfter(originalRun, element)
  const originalStop = runtime.stop.bind(runtime)
  runtime.stop = (element) => syncAfter(originalStop, element)

  const originalClear = runtime.clear.bind(runtime)
  runtime.clear = (state) => {
    const result = originalClear(state)
    syncRunButton(state)
    return result
  }

  const activateButton = (button) => {
    runtime.registerRunButton(button)
    const state = stateFor(runtime, button)
    if (state?.status === 'running' || state?.status === 'stopping') return runtime.stop(button)
    return runtime.run(button)
  }

  const onClick = (event) => {
    const button = event.target?.closest?.(RUN_BUTTON_SELECTOR)
    if (!button || !documentTarget.contains(button)) return
    event.preventDefault()
    event.stopPropagation()
    void activateButton(button)
  }

  const onKeydown = (event) => {
    if (event.key !== 'Enter' || !(event.metaKey || event.ctrlKey)) return
    const pre = event.target?.closest?.('pre.ag-fence-code')
    const button = pre?.querySelector?.(`:scope > ${RUN_BUTTON_SELECTOR}`)
    if (!button) return
    event.preventDefault()
    event.stopPropagation()
    void activateButton(button)
  }

  documentTarget.addEventListener('click', onClick)
  documentTarget.addEventListener('keydown', onKeydown)

  for (const output of documentTarget.querySelectorAll(OUTPUT_SELECTOR)) {
    runtime.registerOutput(output)
  }

  const originalDispose = runtime.dispose.bind(runtime)
  runtime.dispose = () => {
    documentTarget.removeEventListener('click', onClick)
    documentTarget.removeEventListener('keydown', onKeydown)
    for (const state of runtime.states.values()) clearDetachTimer(state)
    originalDispose()
  }

  return runtime
}
