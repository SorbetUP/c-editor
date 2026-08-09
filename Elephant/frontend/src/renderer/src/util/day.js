const pad2 = (value) => String(value).padStart(2, '0')

const getParts = (date = new Date()) => ({
  year: date.getFullYear(),
  month: pad2(date.getMonth() + 1),
  day: pad2(date.getDate()),
  hour: pad2(date.getHours()),
  minute: pad2(date.getMinutes()),
  second: pad2(date.getSeconds())
})

const formatDate = (date, pattern) => {
  const { year, month, day, hour, minute, second } = getParts(date)
  return String(pattern || '')
    .replace(/YYYY/g, String(year))
    .replace(/MM/g, month)
    .replace(/DD/g, day)
    .replace(/HH/g, hour)
    .replace(/mm/g, minute)
    .replace(/ss/g, second)
}

export default function dayjs(date = new Date()) {
  const value = date instanceof Date ? date : new Date(date)
  return {
    format: (pattern) => formatDate(value, pattern)
  }
}
