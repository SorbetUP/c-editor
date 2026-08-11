const delay = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds))

export const waitForAcceptanceEndpoint = async (child, { timeoutMs = 120000, expectedPort = null, onOutput = () => {} } = {}) => {
  let output = ''
  const append = (stream, chunk) => {
    const text = chunk.toString()
    output += text
    onOutput(stream, text)
  }
  child.stdout?.on('data', (chunk) => append('stdout', chunk))
  child.stderr?.on('data', (chunk) => append('stderr', chunk))
  const deadline = Date.now() + timeoutMs
  while (Date.now() <= deadline) {
    if (expectedPort) {
      const endpoint = `http://127.0.0.1:${expectedPort}`
      try {
        const response = await fetch(`${endpoint}/health`)
        if (response.ok) return { endpoint, output, source: 'reserved-acceptance-port' }
      } catch {
        // The child may still be between setup and listener bind.
      }
    }
    const match = output.match(/ELEPHANT_ACCEPTANCE_TAURI_PORT=(\d+)/)
    if (match) return { endpoint: `http://127.0.0.1:${Number(match[1])}`, output }
    if (child.exitCode !== null) throw new Error(`Tauri exited before acceptance server started (${child.exitCode})`)
    await delay(250)
  }
  throw new Error(`timed out waiting for Tauri acceptance server; output=${output.slice(-2000)}`)
}
export const createAcceptanceClient = (endpoint, { log = () => {} } = {}) => ({
  async health() {
    const response = await fetch(`${endpoint}/health`)
    const body = await response.json()
    log({ type: 'health', ok: response.ok, body })
    return body
  },
  async command(command, ...args) {
    const startedAt = Date.now()
    log({ type: 'command:start', command, argsCount: args.length })
    const response = await fetch(`${endpoint}/command`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ command, args })
    })
    const body = await response.json()
    log({ type: response.ok && body.ok ? 'command:done' : 'command:error', command, requestId: body.requestId, durationMs: Date.now() - startedAt, error: body.error || null })
    if (!response.ok || !body.ok) throw new Error(`${command} failed: ${body.error || response.status}`)
    return body.result
  }
})

export const stopProcessTree = async (child) => {
  if (!child || child.exitCode !== null) return
  try {
    process.kill(-child.pid, 'SIGTERM')
  } catch {
    child.kill('SIGTERM')
  }
  await Promise.race([
    new Promise((resolve) => child.once('close', resolve)),
    delay(5000)
  ])
}
