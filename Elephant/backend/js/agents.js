import { ipcMain } from 'electron'
import {
  createAiRequestBody,
  extractAiResponseText,
  normalizeAiConfig,
  normalizeAiEndpoint
} from 'common/elephantnote/aiProviders'

const agents = new Map()

const normalizeAgent = (agent = {}) => {
  const id = String(agent.id || '').trim()
  if (!id) throw new Error('Agent id is required.')
  return {
    id,
    name: String(agent.name || id),
    transport: String(agent.transport || 'openai-compatible'),
    endpoint: String(agent.endpoint || ''),
    model: String(agent.model || ''),
    apiKey: String(agent.apiKey || ''),
    capabilities: Array.isArray(agent.capabilities)
      ? agent.capabilities.map((capability) => String(capability)).filter(Boolean)
      : []
  }
}

export const registerElephantNoteAgentIpc = () => {
  ipcMain.handle('en:agents:list', async() => listAgents())
  ipcMain.handle('en:agents:register', async(_event, payload) => {
    return registerAgent(payload)
  })
  ipcMain.handle('en:agents:unregister', async(_event, id) => {
    return unregisterAgent(id)
  })
  ipcMain.handle('en:agents:send', async(_event, payload = {}) => {
    return sendAgentMessage(payload)
  })
}

export const listAgents = () => [...agents.values()]

export const registerAgent = (payload) => {
  const agent = normalizeAgent({
    ...payload,
    endpoint: normalizeAiEndpoint(payload?.endpoint)
  })
  agents.set(agent.id, agent)
  return agent
}

export const unregisterAgent = (id) => agents.delete(String(id || '').trim())

export const sendAgentMessage = async(payload = {}) => {
  const id = String(payload.agentId || payload.id || '').trim()
  const agent = payload.agent
    ? normalizeAgent(payload.agent)
    : agents.get(id)
  if (!agent) throw new Error('Unknown agent.')
  if (!agent.endpoint) throw new Error('Agent endpoint is not configured.')
  const config = normalizeAiConfig(agent)
  const message = String(payload.message || '').trim()
  if (!message) throw new Error('Agent message is required.')
  if (!config.model) throw new Error('Agent model is not configured.')

  const response = await fetch(config.endpoint, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
      ...(config.apiKey ? { authorization: `Bearer ${config.apiKey}` } : {})
    },
    body: JSON.stringify(createAiRequestBody({
      transport: config.transport,
      model: config.model,
      messages: [
        {
          role: 'user',
          content: message
        }
      ]
    }))
  })

  const text = await response.text()
  let data = {}
  if (text) {
    try {
      data = JSON.parse(text)
    } catch {
      data = { message: text }
    }
  }
  if (!response.ok) {
    throw new Error(data?.error?.message || data?.message || `Agent endpoint returned HTTP ${response.status}.`)
  }

  return {
    id: agent.id,
    message: extractAiResponseText(data),
    raw: data
  }
}
