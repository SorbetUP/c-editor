export const dispatchMuyaChange = ({
  changes,
  currentFileId,
  fromEditorMarkdown = (markdown) => markdown,
  onDocumentChange = () => {},
  onRuntimeChange = () => {}
}) => {
  const nextChanges = {
    ...changes,
    markdown: fromEditorMarkdown(changes.markdown)
  }

  if (currentFileId) {
    onDocumentChange({ ...nextChanges, id: currentFileId })
  }

  onRuntimeChange(nextChanges)
}
