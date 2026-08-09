import { debounce } from '../utils'

export default (Muya) => {
  Object.assign(Muya.prototype, {
    init() {
      const { container, contentState, eventCenter } = this
      contentState.stateRender.setContainer(container.children[0])
      eventCenter.subscribe('stateChange', this.dispatchChange)
      const { markdown } = this
      const { focusMode } = this.options

      if (this.i18nCSS) this.i18nCSS.updateCSSVariables()

      this.setMarkdown(markdown)
      this.setFocusMode(focusMode)
      this.mutationObserver()

      const handleScroll = debounce(() => {
        eventCenter.dispatch('scroll', { scrollTop: container.scrollTop })
      }, 100)

      eventCenter.attachDOMEvent(container, 'focus', () => eventCenter.dispatch('focus'))
      eventCenter.attachDOMEvent(container, 'blur', () => eventCenter.dispatch('blur'))
      eventCenter.attachDOMEvent(container, 'scroll', handleScroll)
    },

    mutationObserver() {
      const { container, eventCenter } = this
      const callback = (mutationsList) => {
        for (const mutation of mutationsList) {
          if (mutation.type !== 'childList') continue
          const { removedNodes, target } = mutation
          if (removedNodes && removedNodes.length) {
            const hasTable = Array.from(removedNodes).some(
              (node) => node.nodeType === 1 && node.closest('table.ag-paragraph')
            )
            if (hasTable) {
              eventCenter.dispatch('crashed')
              console.warn('There was a problem with the table deletion.')
            }
          }

          if (target.getAttribute('id') === 'ag-editor-id' && target.childElementCount === 0) {
            eventCenter.dispatch('crashed')
            console.warn('editor crashed, and can not be input any more.')
          }
        }
      }

      this._mutationObserver = new MutationObserver(callback)
      this._mutationObserver.observe(container, { childList: true, subtree: true })
    },

    destroy() {
      this._rustEditorDisposed = true
      this.rustEditorRuntime?.destroy()
      this.rustEditorRuntime = null
      this.rustEditorBridge = null
      this.contentState.clear()
      this.quickInsert?.destroy()
      this.codePicker?.destroy()
      this.tablePicker?.destroy()
      this.emojiPicker?.destroy()
      this.imagePathPicker?.destroy()
      this._mutationObserver?.disconnect()
      this._mutationObserver = null
      this.eventCenter.detachAllDomEvents()
      this.eventCenter.unsubscribeAll()
    }
  })
}
