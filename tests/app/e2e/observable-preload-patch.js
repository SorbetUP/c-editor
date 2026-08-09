'use strict'

const INVOKE_DECLARATION = 'const invoke = async (command, payload = {}) => {'
const CALLBACK_BOUNDARY = '\nlet callbackId = 0\n'
const UNHANDLED_FALLBACK = `      console.warn('[e2e-tauri] unhandled invoke', command, params)
      return null`

const wrapperSource = String.raw`
const redactObservableValue = (value, key = '', seen = new WeakSet()) => {
  const secretPattern = /(token|secret|password|passwd|passphrase|private.?key|api.?key|cookie|authorization|credential|session|dsn)/i
  if (secretPattern.test(String(key || ''))) return '[REDACTED_SECRET_VALUE]'
  if (value === null || value === undefined) return value
  if (typeof value === 'bigint') return String(value)
  if (typeof value !== 'object') return value
  if (seen.has(value)) return '[CIRCULAR]'
  seen.add(value)
  if (Array.isArray(value)) return value.map((entry, index) => redactObservableValue(entry, String(index), seen))
  const output = {}
  for (const [entryKey, entryValue] of Object.entries(value)) {
    output[entryKey] = redactObservableValue(entryValue, entryKey, seen)
  }
  return output
}

let observableInvokeSequence = 0
const invoke = async (command, payload = {}) => {
  const sequence = ++observableInvokeSequence
  const startedAt = Date.now()
  const safePayload = redactObservableValue(payload)
  console.log('[e2e-tauri:invoke:start] ' + JSON.stringify({ sequence, command, payload: safePayload }))
  try {
    const result = await invokeImplementation(command, payload)
    console.log('[e2e-tauri:invoke:done] ' + JSON.stringify({
      sequence,
      command,
      durationMs: Date.now() - startedAt,
      result: redactObservableValue(result)
    }))
    return result
  } catch (error) {
    console.error('[e2e-tauri:invoke:error] ' + JSON.stringify({
      sequence,
      command,
      durationMs: Date.now() - startedAt,
      error: {
        name: error?.name || 'Error',
        message: error?.message || String(error),
        stack: error?.stack || ''
      }
    }))
    throw error
  }
}
`

const strictUnhandledSource = String.raw`      const error = new Error('Unhandled E2E Tauri invoke: ' + command + ' ' + JSON.stringify(redactObservableValue(params)))
      console.error('[e2e-tauri] unhandled invoke', command, redactObservableValue(params))
      if (process.env.ELEPHANT_E2E_STRICT_TAURI_COMMANDS !== '0') throw error
      return null`

module.exports = (source) => {
  let patched = String(source)
  if (!patched.includes(INVOKE_DECLARATION)) throw new Error('Unable to locate E2E Tauri invoke implementation')
  patched = patched.replace(INVOKE_DECLARATION, 'const invokeImplementation = async (command, payload = {}) => {')
  if (!patched.includes(UNHANDLED_FALLBACK)) throw new Error('Unable to locate E2E Tauri unhandled fallback')
  patched = patched.replace(UNHANDLED_FALLBACK, strictUnhandledSource)
  if (!patched.includes(CALLBACK_BOUNDARY)) throw new Error('Unable to locate E2E callback boundary')
  patched = patched.replace(CALLBACK_BOUNDARY, `\n${wrapperSource}\nlet callbackId = 0\n`)
  return patched
}
