import { defineStore } from 'pinia'

const STORAGE_KEY = 'elephantnote:chat:conversations'
const ACTIVE_KEY = 'elephantnote:chat:active'

const createId = () => `chat-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`

const safeRead = (key, fallback) => {
  try {
    const raw = window.localStorage.getItem(key)
    if (!raw) return fallback
    return JSON.parse(raw)
  } catch {
    return fallback
  }
}

const safeWrite = (key, value) => {
  try {
    window.localStorage.setItem(key, JSON.stringify(value))
  } catch {
    /* ignore quota errors */
  }
}

const deriveTitle = (content) => {
  if (!content) return 'New chat'
  const text = String(content).replace(/\s+/g, ' ').trim()
  if (!text) return 'New chat'
  return text.length > 42 ? `${text.slice(0, 42)}\u2026` : text
}

const bucketKeyFor = (timestamp) => {
  const now = new Date()
  const date = new Date(timestamp)
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime()
  const dayMs = 86400000
  if (date.getTime() >= startOfToday) return 'today'
  if (date.getTime() >= startOfToday - dayMs) return 'yesterday'
  if (date.getTime() >= startOfToday - 7 * dayMs) return 'previous7'
  if (date.getTime() >= startOfToday - 30 * dayMs) return 'previous30'
  return 'older'
}

export const CHAT_HISTORY_BUCKETS = Object.freeze([
  { id: 'today', label: 'Today' },
  { id: 'yesterday', label: 'Yesterday' },
  { id: 'previous7', label: 'Previous 7 days' },
  { id: 'previous30', label: 'Previous 30 days' },
  { id: 'older', label: 'Older' }
])

export const useChatStore = defineStore('elephantnoteChat', {
  state: () => ({
    conversations: safeRead(STORAGE_KEY, []),
    activeConversationId: safeRead(ACTIVE_KEY, '') || '',
    searchQuery: '',
    pendingToolCalls: [],
    isSending: false,
    runtimeMessage: ''
  }),
  getters: {
    activeConversation(state) {
      return state.conversations.find((c) => c.id === state.activeConversationId) || null
    },
    activeMessages(state) {
      const conversation = state.conversations.find((c) => c.id === state.activeConversationId)
      return conversation?.messages || []
    },
    filteredConversations(state) {
      const query = state.searchQuery.trim().toLowerCase()
      if (!query) return state.conversations
      return state.conversations.filter((conversation) => {
        if (conversation.title?.toLowerCase().includes(query)) return true
        return conversation.messages?.some((message) =>
          String(message.content || '').toLowerCase().includes(query)
        )
      })
    },
    groupedConversations(state) {
      const query = state.searchQuery.trim().toLowerCase()
      const list = query
        ? state.conversations.filter((conversation) => {
          if (conversation.title?.toLowerCase().includes(query)) return true
          return conversation.messages?.some((message) =>
            String(message.content || '').toLowerCase().includes(query)
          )
        })
        : state.conversations
      const groups = {}
      for (const conversation of list) {
        const bucket = bucketKeyFor(conversation.updatedAt || conversation.createdAt)
        if (!groups[bucket]) groups[bucket] = []
        groups[bucket].push(conversation)
      }
      return CHAT_HISTORY_BUCKETS.map((bucket) => ({
        ...bucket,
        items: (groups[bucket.id] || [])
          .slice()
          .sort((a, b) => (b.updatedAt || 0) - (a.updatedAt || 0))
      })).filter((bucket) => bucket.items.length)
    }
  },
  actions: {
    persist() {
      safeWrite(STORAGE_KEY, this.conversations)
      safeWrite(ACTIVE_KEY, this.activeConversationId)
    },

    createConversation({ title = '' } = {}) {
      const now = Date.now()
      const conversation = {
        id: createId(),
        title: title || 'New chat',
        createdAt: now,
        updatedAt: now,
        pinned: false,
        messages: []
      }
      this.conversations.unshift(conversation)
      this.activeConversationId = conversation.id
      this.persist()
      return conversation
    },

    ensureActiveConversation() {
      if (this.activeConversation) return this.activeConversation
      return this.createConversation()
    },

    selectConversation(id) {
      if (!id) return
      this.activeConversationId = id
      this.pendingToolCalls = []
      this.runtimeMessage = ''
      this.persist()
    },

    deleteConversation(id) {
      const index = this.conversations.findIndex((c) => c.id === id)
      if (index < 0) return
      this.conversations.splice(index, 1)
      if (this.activeConversationId === id) {
        this.activeConversationId = this.conversations[0]?.id || ''
      }
      this.persist()
    },

    renameConversation(id, title) {
      const conversation = this.conversations.find((c) => c.id === id)
      if (!conversation) return
      conversation.title = title?.trim() || 'New chat'
      conversation.updatedAt = Date.now()
      this.persist()
    },

    togglePinned(id) {
      const conversation = this.conversations.find((c) => c.id === id)
      if (!conversation) return
      conversation.pinned = !conversation.pinned
      conversation.updatedAt = Date.now()
      this.persist()
    },

    addMessage(message) {
      const conversation = this.ensureActiveConversation()
      const entry = {
        id: `msg-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        createdAt: Date.now(),
        ...message
      }
      conversation.messages.push(entry)
      conversation.updatedAt = entry.createdAt
      if (message.role === 'user' && (!conversation.title || conversation.title === 'New chat')) {
        conversation.title = deriveTitle(message.content)
      }
      this.persist()
      return entry
    },

    updateMessage(messageId, patch) {
      const conversation = this.activeConversation
      if (!conversation) return
      const message = conversation.messages.find((m) => m.id === messageId)
      if (!message) return
      Object.assign(message, patch)
      conversation.updatedAt = Date.now()
      this.persist()
    },

    addToolCall(toolCall) {
      this.pendingToolCalls.push({
        id: `tool-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`,
        createdAt: Date.now(),
        ...toolCall
      })
    },

    consumePendingToolCalls() {
      const calls = [...this.pendingToolCalls]
      this.pendingToolCalls = []
      return calls
    },

    clearActive() {
      const conversation = this.activeConversation
      if (!conversation) return
      conversation.messages = []
      conversation.title = 'New chat'
      conversation.updatedAt = Date.now()
      this.persist()
    },

    setRuntimeMessage(message) {
      this.runtimeMessage = message || ''
    },

    setSending(value) {
      this.isSending = !!value
    },

    setSearchQuery(query) {
      this.searchQuery = String(query || '')
    }
  }
})
