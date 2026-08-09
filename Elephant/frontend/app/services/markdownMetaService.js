const isDateLikeString = (value) => /^\d{4}-\d{2}-\d{2}(?:T|$)/.test(String(value || ''))
const isUnixSecondsString = (value) => /^\d{10}$/.test(String(value || ''))
const isUnixMillisecondsString = (value) => /^\d{13}$/.test(String(value || ''))

const toDateInput = (value) => {
  if (typeof value !== 'string') return value
  if (isDateLikeString(value)) return value
  if (isUnixSecondsString(value)) return Number(value) * 1000
  if (isUnixMillisecondsString(value)) return Number(value)
  return null
}

export const formatShortDate = (value) => {
  if (!value) return ''
  const dateInput = toDateInput(value)
  if (dateInput == null) return ''
  const date = new Date(dateInput)
  const timestamp = date.getTime()
  if (!Number.isFinite(timestamp)) return ''
  return new Intl.DateTimeFormat('en-CA', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit'
  }).format(date)
}

export const relativeParentPath = (entryPath) => {
  if (!entryPath || !entryPath.includes('/')) return ''
  return entryPath.split('/').slice(0, -1).join('/')
}
