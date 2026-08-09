export const buildRagChatPrompt = ({
  message = '',
  messages = [],
  citations = [],
  wikiContext = null
} = {}) => {
  const normalizedMessage = String(message || '').trim()
  const conversationHistory = Array.isArray(messages)
    ? messages
        .map((entry) => ({
          role: String(entry?.role || 'user'),
          content: String(entry?.content || '').trim()
        }))
        .filter((entry, index, list) => {
          if (!entry.content) return false
          const isFinalUserTurn =
            index === list.length - 1 &&
            entry.role === 'user' &&
            entry.content === normalizedMessage
          return !isFinalUserTurn
        })
        .slice(-8)
        .map((entry) => `${entry.role === 'assistant' ? 'Assistant' : 'User'}: ${entry.content}`)
        .join('\n')
    : ''
  const contextBlock = citations
    .map(
      (citation, index) =>
        `[${index + 1}] ${citation.title} (${citation.path})\n${citation.snippet}`
    )
    .join('\n\n')
  const wikiBlock = wikiContext
    ? [
        'Wiki context:',
        `- Source: ${wikiContext.source?.title || wikiContext.source?.path || 'unknown'}`,
        `- Graph: ${wikiContext.graphSummary?.nodes || 0} nodes, ${wikiContext.graphSummary?.semanticLinks || 0} semantic links, ${wikiContext.graphSummary?.clusters || 0} clusters`,
        wikiContext.cluster
          ? `- Cluster: ${wikiContext.cluster.label} (${wikiContext.cluster.nodeCount || 0} notes)`
          : '',
        wikiContext.relatedNodes?.length
          ? `- Related notes: ${wikiContext.relatedNodes
              .slice(0, 5)
              .map((node, index) => `${index + 1}. ${node.title} [${node.linkType}]`)
              .join('; ')}`
          : ''
      ]
        .filter(Boolean)
        .join('\n')
    : ''

  const conversationBlock = conversationHistory
    ? ['Conversation history:', conversationHistory].join('\n')
    : ''

  const hasLocalContext = citations.length > 0 || Boolean(wikiBlock) || Boolean(conversationBlock)
  const sections = hasLocalContext
    ? [
        `Question: ${message}`,
        conversationBlock,
        'Local context:',
        contextBlock || 'No direct note citation matched.',
        wikiBlock ? `\n${wikiBlock}` : ''
      ].filter(Boolean)
    : [
        `Question: ${message}`,
        'No local note citation matched this message. Answer normally as the ElephantNote chat assistant. Do not claim that a local note matched unless citations are provided.'
      ]

  return {
    systemMessage: hasLocalContext
      ? 'You are a private local notes assistant. Use the provided citations, semantic graph, wiki context, and conversation history for grounded factual claims. Cite relevant sources with markers like [1]. If the user asks for general help, you may also answer normally while keeping citations for note-derived claims.'
      : 'You are the ElephantNote chat assistant. No local note citation matched the user message, so answer normally without pretending to have local-note evidence.',
    prompt: sections.join('\n\n')
  }
}
