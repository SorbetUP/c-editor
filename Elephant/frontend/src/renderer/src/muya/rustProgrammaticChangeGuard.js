const DEFAULT_BURST = 1
const DEFAULT_TTL_MS = 750

export const createProgrammaticChangeGuard = ({
  now = () => Date.now(),
  burst = DEFAULT_BURST,
  ttlMs = DEFAULT_TTL_MS
} = {}) => {
  let pending = 0
  let deadline = 0

  const currentTime = () => Number(now()) || 0
  const ttl = () => Number(ttlMs) || DEFAULT_TTL_MS
  const slotsPerRender = () => Math.max(1, Math.floor(Number(burst) || DEFAULT_BURST))
  const clearIfExpired = () => {
    if (deadline > 0 && currentTime() > deadline) {
      pending = 0
      deadline = 0
    }
  }

  return {
    run (render) {
      if (typeof render !== 'function') throw new TypeError('A programmatic Muya render callback is required.')
      clearIfExpired()
      const previousPending = pending
      const previousDeadline = deadline
      pending += slotsPerRender()
      deadline = currentTime() + ttl()
      try {
        return render()
      } catch (error) {
        pending = previousPending
        deadline = previousDeadline
        throw error
      }
    },

    consume () {
      clearIfExpired()
      if (pending <= 0) return false
      pending -= 1
      if (pending === 0) deadline = 0
      return true
    },

    clear () {
      pending = 0
      deadline = 0
    },

    get pending () {
      clearIfExpired()
      return pending
    }
  }
}

