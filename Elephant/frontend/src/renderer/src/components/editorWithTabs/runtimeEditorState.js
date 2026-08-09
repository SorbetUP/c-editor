export const applyRustEditorMarkdown = ({
  editorStore,
  file,
  editorMarkdown,
  fromEditorMarkdown = (markdown) => markdown,
  persist = () => {}
}) => {
  if (!file?.id) return false
  const nextMarkdown = fromEditorMarkdown(String(editorMarkdown || ''))
  if (file.markdown === nextMarkdown) return false

  file.markdown = nextMarkdown
  file.isSaved = false
  const index = editorStore.tabIdToIndex[file.id]
  if (index !== undefined && editorStore.tabs[index]) {
    editorStore.tabs[index].markdown = nextMarkdown
    editorStore.tabs[index].isSaved = false
  }
  persist()
  return true
}
