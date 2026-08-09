const unwrapQuote = (contentState, quoteBlock) => {
  const children = quoteBlock.children
  for (const child of children) contentState.insertBefore(child, quoteBlock)
  contentState.removeBlock(quoteBlock)
}

const wrapSingleBlock = (contentState, startBlock) => {
  if (startBlock.type === 'span') startBlock = contentState.getParent(startBlock)
  const quoteBlock = contentState.createBlock('blockquote')
  contentState.insertAfter(quoteBlock, startBlock)
  contentState.removeBlock(startBlock)
  contentState.appendChild(quoteBlock, startBlock)
}

const wrapMultipleBlocks = (contentState, parent, startIndex, endIndex) => {
  const children = parent ? parent.children : contentState.blocks
  const referBlock = children[endIndex]
  const quoteBlock = contentState.createBlock('blockquote')
  children.slice(startIndex, endIndex + 1).forEach(child => {
    if (child !== referBlock) {
      contentState.removeBlock(child, children)
    } else {
      contentState.insertAfter(quoteBlock, child)
      contentState.removeBlock(child, children)
    }
    contentState.appendChild(quoteBlock, child)
  })
}

const recordQuoteMutation = (contentState, start, end) => {
  contentState.cursor = {
    start: { key: start.key, offset: start.offset },
    end: { key: end.key, offset: end.offset },
    isEdit: true
  }
}

const paragraphQuote = ContentState => {
  ContentState.prototype.handleQuoteMenu = function(insertMode) {
    const { start, end, affiliation } = this.selectionChange(this.cursor)
    const startBlock = this.getBlock(start.key)
    const blockquotes = affiliation
      .slice(0, 2)
      .filter(block => /blockquote/.test(block.type))
    if (blockquotes.length && !insertMode) {
      unwrapQuote(this, blockquotes[0])
    } else if (start.key === end.key) {
      wrapSingleBlock(this, startBlock)
    } else {
      const { parent, startIndex, endIndex } = this.getCommonParent()
      wrapMultipleBlocks(this, parent, startIndex, endIndex)
    }
    recordQuoteMutation(this, start, end)
  }
}

export default paragraphQuote
