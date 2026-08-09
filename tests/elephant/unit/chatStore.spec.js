import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useChatStore } from '@/elephantnote/stores/chatStore'
import {
  buildChatContextPanel,
  formatChatTimestamp,
  shapeToolCallsForAssistant
} from '@/elephantnote/components/views/chatViewHelpers'

const flushPromises = () => new Promise((resolve) => setTimeout(resolve, 0))

describe('ElephantNote chat store', () => {
  beforeEach(() => {
    window.localStorage.clear()
    setActivePinia(createPinia())
  })

  it('creates a new conversation and marks it active', () => {
    const chatStore = useChatStore()
    const conversation = chatStore.createConversation({ title: 'Test chat' })

    expect(conversation.id).toBeTruthy()
    expect(conversation.title).toBe('Test chat')
    expect(chatStore.activeConversationId).toBe(conversation.id)
    expect(chatStore.activeConversation).toEqual(conversation)
    expect(chatStore.conversations).toHaveLength(1)
  })

  it('ensures an active conversation exists without duplicating', () => {
    const chatStore = useChatStore()
    const first = chatStore.ensureActiveConversation()
    const second = chatStore.ensureActiveConversation()

    expect(first.id).toBe(second.id)
    expect(chatStore.conversations).toHaveLength(1)
  })

  it('adds a user message and derives a title from the content', () => {
    const chatStore = useChatStore()
    chatStore.createConversation()
    chatStore.addMessage({ role: 'user', content: 'How do I center a div in CSS?' })

    const conversation = chatStore.activeConversation
    expect(conversation.messages).toHaveLength(1)
    expect(conversation.messages[0].role).toBe('user')
    expect(conversation.title).toBe('How do I center a div in CSS?')
  })

  it('truncates long user content for the conversation title', () => {
    const chatStore = useChatStore()
    chatStore.createConversation()
    const longContent = 'A'.repeat(120)
    chatStore.addMessage({ role: 'user', content: longContent })

    expect(chatStore.activeConversation.title.length).toBeLessThanOrEqual(43)
  })

  it('links assistant messages with tool calls and citations', () => {
    const chatStore = useChatStore()
    chatStore.createConversation()
    chatStore.addMessage({ role: 'user', content: 'Summarize the graph' })
    chatStore.addMessage({
      role: 'assistant',
      content: 'Here is a summary.',
      citations: [{ path: 'a.md', title: 'Alpha' }],
      wikiContext: { source: { path: 'a.md', title: 'Alpha' } },
      toolCalls: [{ id: 't1', name: 'rag.search', label: 'Search', status: 'done' }]
    })

    const assistant = chatStore.activeMessages.find((m) => m.role === 'assistant')
    expect(assistant.toolCalls).toHaveLength(1)
    expect(assistant.citations).toEqual([{ path: 'a.md', title: 'Alpha' }])
    expect(assistant.wikiContext?.source?.path).toBe('a.md')
  })

  it('deletes a conversation and keeps the remaining ones', () => {
    const chatStore = useChatStore()
    const first = chatStore.createConversation()
    const second = chatStore.createConversation()
    chatStore.deleteConversation(second.id)

    expect(chatStore.conversations).toHaveLength(1)
    expect(chatStore.conversations[0].id).toBe(first.id)
    expect(chatStore.activeConversationId).toBe(first.id)
  })

  it('groups conversations by relative date buckets', () => {
    const chatStore = useChatStore()
    const now = Date.now()
    const day = 86400000

    chatStore.conversations = [
      { id: 'c1', title: 'Today', createdAt: now, updatedAt: now, messages: [] },
      {
        id: 'c2',
        title: 'Yesterday',
        createdAt: now - day,
        updatedAt: now - day,
        messages: []
      },
      {
        id: 'c3',
        title: 'Old',
        createdAt: now - 60 * day,
        updatedAt: now - 60 * day,
        messages: []
      }
    ]

    const groups = chatStore.groupedConversations
    expect(groups.map((g) => g.id)).toEqual(['today', 'yesterday', 'older'])
    expect(groups[0].items[0].title).toBe('Today')
  })

  it('filters conversations by search query across title and messages', () => {
    const chatStore = useChatStore()
    chatStore.conversations = [
      { id: 'c1', title: 'Graph analysis', createdAt: 1, updatedAt: 1, messages: [] },
      {
        id: 'c2',
        title: 'Random',
        createdAt: 2,
        updatedAt: 2,
        messages: [{ id: 'm1', content: 'tell me about semantic links' }]
      }
    ]

    chatStore.setSearchQuery('graph')
    expect(chatStore.filteredConversations.map((c) => c.id)).toEqual(['c1'])

    chatStore.setSearchQuery('semantic')
    expect(chatStore.filteredConversations.map((c) => c.id)).toEqual(['c2'])
  })

  it('persists and restores conversations across store instances', async() => {
    const first = useChatStore()
    first.createConversation({ title: 'Persistent chat' })
    first.addMessage({ role: 'user', content: 'hello' })

    setActivePinia(createPinia())
    const second = useChatStore()
    await flushPromises()

    expect(second.conversations).toHaveLength(1)
    expect(second.conversations[0].title).toBe('Persistent chat')
    expect(second.activeConversationId).toBe(first.activeConversationId)
  })
})

describe('chatViewHelpers integration with chat store', () => {
  beforeEach(() => {
    window.localStorage.clear()
    setActivePinia(createPinia())
  })

  it('shapes tool calls from a RAG chat result for the assistant message', () => {
    const result = {
      answer: 'Here is the answer.',
      citations: [
        { path: 'note-a.md', title: 'Note A', score: 0.9 },
        { path: 'note-b.md', title: 'Note B', score: 0.7 }
      ],
      wikiContext: {
        source: { path: 'note-a.md', title: 'Note A' },
        graphSummary: { nodes: 5, semanticLinks: 3, clusters: 2 }
      }
    }

    const toolCalls = shapeToolCallsForAssistant(result)
    expect(toolCalls).toHaveLength(2)
    expect(toolCalls[0].name).toBe('rag.search')
    expect(toolCalls[0].sources).toHaveLength(2)
    expect(toolCalls[1].name).toBe('wiki.context')
    expect(toolCalls[1].summary).toContain('5 nodes')
  })

  it('returns an empty array when the RAG result has no citations or wiki context', () => {
    expect(shapeToolCallsForAssistant({})).toEqual([])
    expect(shapeToolCallsForAssistant({ citations: [], wikiContext: null })).toEqual([])
  })

  it('formats chat timestamps as time only for today, date+time otherwise', () => {
    const now = new Date()
    const today = formatChatTimestamp(now.getTime())
    expect(today).toMatch(/^\d{1,2}:\d{2}/)

    const past = new Date(2020, 0, 15, 10, 30).getTime()
    const formatted = formatChatTimestamp(past)
    expect(formatted).toContain('/')
    expect(formatted).toContain(':')
  })

  it('builds quick prompts with icons for the empty chat state', () => {
    const panel = buildChatContextPanel({ graph: null })
    expect(panel.quickPrompts).toHaveLength(4)
    panel.quickPrompts.forEach((prompt) => {
      expect(prompt.icon).toBeTruthy()
      expect(prompt.prompt).toBeTruthy()
    })
  })

  it('links the chat store workflow with shaped tool calls end to end', () => {
    const chatStore = useChatStore()
    chatStore.createConversation()
    chatStore.addMessage({ role: 'user', content: 'What links Alpha and Beta?' })

    const ragResult = {
      answer: 'Alpha and Beta are linked by a semantic edge.',
      citations: [
        { path: 'alpha.md', title: 'Alpha' },
        { path: 'beta.md', title: 'Beta' }
      ],
      wikiContext: {
        source: { path: 'alpha.md', title: 'Alpha' },
        graphSummary: { nodes: 2, semanticLinks: 1, clusters: 1 }
      }
    }

    chatStore.addMessage({
      role: 'assistant',
      content: ragResult.answer,
      citations: ragResult.citations,
      wikiContext: ragResult.wikiContext,
      toolCalls: shapeToolCallsForAssistant(ragResult)
    })

    const assistant = chatStore.activeMessages.find((m) => m.role === 'assistant')
    expect(assistant.toolCalls).toHaveLength(2)
    expect(assistant.citations).toHaveLength(2)
    expect(assistant.content).toBe(ragResult.answer)
    expect(chatStore.activeConversation.messages).toHaveLength(2)
  })
})
