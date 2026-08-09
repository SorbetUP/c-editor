const clampPercent = (value) => Math.max(0, Math.min(100, Math.round(Number(value) || 0)))
const approximately = (value, target) => Math.abs(value - target) <= target * 0.05

export const formatCodexWindowLabel = (window, isSecondary = false) => {
  const minutes = Number(window?.windowDurationMins)
  if (Number.isFinite(minutes) && minutes > 0) {
    if (approximately(minutes, 5 * 60)) return '5-hour limit'
    if (approximately(minutes, 24 * 60)) return 'Daily limit'
    if (approximately(minutes, 7 * 24 * 60)) return 'Weekly limit'
    if (approximately(minutes, 30 * 24 * 60)) return 'Monthly limit'
    if (minutes % (24 * 60) === 0) return `${Math.round(minutes / (24 * 60))}-day limit`
    if (minutes % 60 === 0) return `${Math.round(minutes / 60)}-hour limit`
    return `${minutes}-minute limit`
  }
  return isSecondary ? 'Secondary usage limit' : 'Usage limit'
}

const snapshotsFromPayload = (payload = {}) => {
  const byId = payload?.rateLimitsByLimitId
  const entries = byId && typeof byId === 'object' && !Array.isArray(byId)
    ? Object.entries(byId).filter(([, snapshot]) => snapshot && typeof snapshot === 'object')
    : []
  entries.sort(([left], [right]) => {
    if (left === 'codex') return -1
    if (right === 'codex') return 1
    return left.localeCompare(right)
  })
  if (entries.length) return entries
  const fallback = payload?.rateLimits
  return fallback && typeof fallback === 'object' ? [[fallback.limitId || 'codex', fallback]] : []
}

export const buildCodexRateLimitRows = (payload = {}) => {
  const snapshots = snapshotsFromPayload(payload)
  const showBucket = snapshots.length > 1
  return snapshots.flatMap(([bucketId, snapshot]) => {
    const bucketLabel = snapshot.limitName || bucketId
    return ['primary', 'secondary'].flatMap((kind, index) => {
      const window = snapshot?.[kind]
      if (!window || typeof window !== 'object') return []
      const usedPercent = clampPercent(window.usedPercent)
      return [{
        id: `${bucketId}-${kind}`,
        label: formatCodexWindowLabel(window, index === 1),
        bucketLabel: showBucket && bucketLabel !== 'codex' ? bucketLabel : '',
        usedPercent,
        remainingPercent: 100 - usedPercent,
        resetsAt: Number.isFinite(Number(window.resetsAt)) ? Number(window.resetsAt) : null,
        windowDurationMins: Number.isFinite(Number(window.windowDurationMins)) ? Number(window.windowDurationMins) : null
      }]
    })
  })
}


const resetCreditsSummary = (payload = {}) => payload?.rateLimitResetCredits && typeof payload.rateLimitResetCredits === 'object'
  ? payload.rateLimitResetCredits
  : {}

export const buildCodexResetCredits = (payload = {}) => {
  const credits = resetCreditsSummary(payload).credits
  if (!Array.isArray(credits)) return []
  return credits
    .filter((credit) => credit && credit.status === 'available' && (!credit.resetType || credit.resetType === 'codexRateLimits'))
    .map((credit) => ({
      id: String(credit.id || ''),
      title: String(credit.title || ''),
      description: String(credit.description || ''),
      grantedAt: Number.isFinite(Number(credit.grantedAt)) ? Number(credit.grantedAt) : null,
      expiresAt: Number.isFinite(Number(credit.expiresAt)) ? Number(credit.expiresAt) : null
    }))
    .filter((credit) => credit.id)
    .sort((left, right) => (left.expiresAt ?? Number.MAX_SAFE_INTEGER) - (right.expiresAt ?? Number.MAX_SAFE_INTEGER))
}

export const getCodexResetAvailableCount = (payload = {}) => {
  const summary = resetCreditsSummary(payload)
  const count = Number(summary.availableCount)
  return Number.isFinite(count) && count >= 0 ? Math.floor(count) : buildCodexResetCredits(payload).length
}
