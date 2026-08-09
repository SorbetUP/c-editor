export const createRustAsyncMutationGate = ({ dispatch, onSuppressed = () => {} } = {}) => {
  if (typeof dispatch !== 'function') {
    throw new TypeError('A Muya dispatchChange callback is required.')
  }
  if (typeof onSuppressed !== 'function') {
    throw new TypeError('A suppressed-dispatch callback must be a function.')
  }

  let pending = 0
  let tail = Promise.resolve()

  const guardedDispatch = (...args) => {
    if (pending > 0) {
      onSuppressed(...args)
      return undefined
    }
    return dispatch(...args)
  }

  const enqueue = (operation) => {
    if (typeof operation !== 'function') {
      return Promise.reject(new TypeError('A Rust editor operation is required.'))
    }

    // Increment synchronously. Muya calls dispatchChange immediately after its
    // keyboard hook returns, before an asynchronous Rust operation gets a turn.
    pending += 1

    const result = tail
      .catch(() => undefined)
      .then(operation)
    const settled = result.finally(() => {
      pending = Math.max(0, pending - 1)
    })

    // A failed command must not poison the ordering of later commands.
    tail = settled.catch(() => undefined)
    return settled
  }

  return {
    dispatch: guardedDispatch,
    enqueue,
    get pending () {
      return pending
    }
  }
}

