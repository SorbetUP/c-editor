import fs from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import { createDomainClients } from 'elephant-front/services/elephantnoteClient/domainClients'
import { ELEPHANTNOTE_API_ACTIONS as API } from 'common/elephantnote/apiActions'

const root = process.cwd()
const read = (file) => fs.readFileSync(path.join(root, file), 'utf8')

describe('package-owned chat boundary', () => {
  it('does not expose RAG chat or semantic initialization through core clients', () => {
    const clients = createDomainClients(async() => ({}), () => ({}))

    expect(clients.rag).toBeUndefined()
    expect(clients.chat).toBeUndefined()
    expect(API.RAG_CHAT).toBeUndefined()
    expect(API.SEARCH_INIT_VAULT).toBeUndefined()
    expect(API.SEARCH_REBUILD).toBeUndefined()
  })

  it('orchestrates chat from the physical AI Chat package', () => {
    const base = read('addons/official/ai-chat/main.js')
    const chat = read('addons/official/ai-chat/main.v2.js')

    expect(base).toContain("getContributions?.('ai.providers')")
    expect(chat).toContain('retrieveContext(')
    expect(base).toContain("typeof option.provider.chat !== 'function'")
    expect(chat).not.toContain("this.call('rag.chat'")
    expect(chat).not.toContain('SEARCH_INIT_VAULT')
  })

  it('renders citations as accessible actions that open their source note', () => {
    const chat = read('addons/official/ai-chat/main.js')

    expect(chat).toContain('openNote(citation)')
    expect(chat).toContain("store?.openNote")
    expect(chat).toContain("node(documentRef, 'button', 'en-chat-citation elephant-chat-citation',")
    expect(chat).toContain("button.setAttribute('aria-label'")
    expect(chat).toContain("'[ai-chat] citation opened'")
  })

  it('keeps provider implementations in separate packages', () => {
    const openModels = read('addons/official/open-models/main.js')
    const codex = read('addons/official/codex-connection/main.js')

    expect(openModels).toContain("registerContribution('ai.providers'")
    expect(codex).toContain("registerContribution('ai.providers'")
    expect(openModels).toContain('chat: (request) => this.chat(request)')
    expect(codex).toContain('chat: (request) => this.chat(request)')
  })
})
