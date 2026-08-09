import ExportMarkdown from '../utils/exportMarkdown'
import ExportHtml from '../utils/exportHtml'
import { wordCount } from '../utils'

export default (Muya) => {
  Object.assign(Muya.prototype, {
    getMarkdown() {
      if (this._markdownBlockCache) return this.getIncrementalMarkdown()
      const blocks = this.contentState.getBlocks()
      const { isGitlabCompatibilityEnabled, listIndentation } = this.contentState
      return new ExportMarkdown(blocks, listIndentation, isGitlabCompatibilityEnabled).generate()
    },

    getIncrementalMarkdown() {
      const blocks = this.contentState.getBlocks()
      const { isGitlabCompatibilityEnabled, listIndentation } = this.contentState
      const nextCache = new Map()
      const chunks = []
      let previousBlock = null

      for (const block of blocks) {
        const signature = this.getMarkdownBlockSignature(block)
        const cached = this._markdownBlockCache.get(block.key)
        const markdown = cached && cached.signature === signature
          ? cached.markdown
          : new ExportMarkdown([block], listIndentation, isGitlabCompatibilityEnabled).generate()
        nextCache.set(block.key, { signature, markdown })
        if (chunks.length) chunks.push(this.getMarkdownBlockSeparator(previousBlock, block))
        chunks.push(markdown)
        previousBlock = block
      }

      this._markdownBlockCache = nextCache
      return chunks.join('')
    },

    getMarkdownBlockSeparator(previousBlock, block) {
      const previousMarker = previousBlock?.children?.[0]?.bulletMarkerOrDelimiter
      const marker = block?.children?.[0]?.bulletMarkerOrDelimiter
      if (
        previousBlock &&
        block &&
        /ul|ol/.test(previousBlock.type) &&
        /ul|ol/.test(block.type) &&
        previousMarker &&
        marker &&
        previousMarker !== marker
      ) {
        return ''
      }
      return '\n'
    },

    getMarkdownBlockSignature(block) {
      const walk = (item) => {
        if (!item) return null
        return [
          item.key,
          item.type,
          item.functionType || '',
          item.text || '',
          item.lang || '',
          item.marker || '',
          item.headingStyle || '',
          item.listItemType || '',
          item.bulletMarkerOrDelimiter || '',
          item.isLooseListItem ? '1' : '0',
          item.checked === true ? '1' : item.checked === false ? '0' : '',
          item.start ?? '',
          (item.children || []).map(walk)
        ]
      }
      return JSON.stringify(walk(block))
    },

    setMarkdown(markdown, cursor, isRenderCursor = true, muyaIndexCursor, blocks) {
      let finalCursor = null
      if (blocks && cursor) {
        finalCursor = cursor
        this.contentState.setBlocks(JSON.parse(JSON.stringify(blocks)))
      } else if (muyaIndexCursor && muyaIndexCursor.anchor && muyaIndexCursor.focus) {
        const cursorInfo = this.contentState.addCursorToMarkdown(markdown, muyaIndexCursor)
        this.contentState.importMarkdown(cursorInfo.markdown, true)
        finalCursor = this.contentState.convertMuyaIndexCursortoCursor(muyaIndexCursor)
      } else {
        this.contentState.importMarkdown(markdown)
      }

      this._markdownBlockCache = new Map()
      this.contentState.importCursor(finalCursor)
      this.contentState.render(isRenderCursor)
      setTimeout(() => this.dispatchChange(), 0)
    },

    exportStyledHTML(options) {
      return new ExportHtml(this.markdown, this).generate(options)
    },

    exportHtml() {
      return new ExportHtml(this.markdown, this).renderHtml()
    },

    getWordCount(markdown) {
      return wordCount(markdown)
    }
  })
}
