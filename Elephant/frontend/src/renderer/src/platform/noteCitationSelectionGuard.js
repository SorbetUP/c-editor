const RUNTIME_KEY = '__ELEPHANT_NOTE_CITATION_SELECTION_GUARD__'

const selectionBelongsToEditor = (selection, editorHost) => {
  if (!selection || selection.rangeCount === 0 || selection.isCollapsed || !editorHost) return false
  const anchorElement = selection.anchorNode?.nodeType === 1
    ? selection.anchorNode
    : selection.anchorNode?.parentElement
  const focusElement = selection.focusNode?.nodeType === 1
    ? selection.focusNode
    : selection.focusNode?.parentElement
  return !!anchorElement && !!focusElement && editorHost.contains(anchorElement) && editorHost.contains(focusElement)
}

export const installNoteCitationSelectionGuard = (target = globalThis.window) => {
  if (!target?.document) return { dispose() {} }
  target[RUNTIME_KEY]?.dispose?.()

  const preserveEditorSelection = (event) => {
    const citationButton = event.target?.closest?.('[data-elephant-note-citation]')
    if (!citationButton) return
    const editorHost = target.document.querySelector('.en-editor-host')
    if (!selectionBelongsToEditor(target.getSelection?.(), editorHost)) return
    event.preventDefault()
  }

  target.document.addEventListener('pointerdown', preserveEditorSelection, true)
  target.document.addEventListener('mousedown', preserveEditorSelection, true)

  const runtime = {
    dispose() {
      target.document.removeEventListener('pointerdown', preserveEditorSelection, true)
      target.document.removeEventListener('mousedown', preserveEditorSelection, true)
      if (target[RUNTIME_KEY] === runtime) delete target[RUNTIME_KEY]
    }
  }
  target[RUNTIME_KEY] = runtime
  return runtime
}
