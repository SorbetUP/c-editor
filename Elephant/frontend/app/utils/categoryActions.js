export const getCategoryItemCount = (category) => Array.isArray(category?.items) ? category.items.length : 0

export const shouldConfirmCategoryDelete = (category) => getCategoryItemCount(category) > 0

export const buildCategoryDeleteConfirmation = (category) => {
  const itemCount = getCategoryItemCount(category)
  if (!itemCount) return ''
  const noun = itemCount === 1 ? 'element' : 'elements'
  return `Delete "${category.title}"? It contains ${itemCount} ${noun}.`
}

export const deleteCategoryWithConfirmation = async(category, { confirm, deleteCategory } = {}) => {
  if (typeof deleteCategory !== 'function') {
    throw new Error('deleteCategory is required.')
  }

  const message = shouldConfirmCategoryDelete(category)
    ? buildCategoryDeleteConfirmation(category)
    : ''

  if (message && typeof confirm === 'function' && !confirm(message)) {
    return { deleted: false, confirmed: false, message }
  }

  await deleteCategory(category)
  return { deleted: true, confirmed: true, message }
}
