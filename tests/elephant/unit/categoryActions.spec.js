import { describe, expect, it, vi } from 'vitest'
import {
  buildCategoryDeleteConfirmation,
  deleteCategoryWithConfirmation,
  getCategoryItemCount,
  shouldConfirmCategoryDelete
} from '@/elephantnote/utils/categoryActions'

describe('ElephantNote category actions', () => {
  it('counts category items safely', () => {
    expect(getCategoryItemCount({ items: [{}, {}, {}] })).toBe(3)
    expect(getCategoryItemCount({ items: [] })).toBe(0)
    expect(getCategoryItemCount({})).toBe(0)
  })

  it('only asks for confirmation when a category contains items', () => {
    expect(shouldConfirmCategoryDelete({ items: [{ id: 'note-1' }] })).toBe(true)
    expect(shouldConfirmCategoryDelete({ items: [] })).toBe(false)
    expect(shouldConfirmCategoryDelete({})).toBe(false)
  })

  it('builds a delete confirmation message that reflects item count', () => {
    expect(buildCategoryDeleteConfirmation({ title: 'Getting started', items: [{ id: 1 }] }))
      .toContain('1 element')
    expect(buildCategoryDeleteConfirmation({
      title: 'Getting started',
      items: [{ id: 1 }, { id: 2 }]
    })).toContain('2 elements')
    expect(buildCategoryDeleteConfirmation({ title: 'Getting started', items: [] })).toBe('')
  })

  it('cancels deletion when the confirmation dialog is rejected', async() => {
    const confirm = vi.fn(() => false)
    const deleteCategory = vi.fn()

    const result = await deleteCategoryWithConfirmation({
      title: 'Getting started',
      items: [{ id: 'item-1' }]
    }, {
      confirm,
      deleteCategory
    })

    expect(confirm).toHaveBeenCalledTimes(1)
    expect(deleteCategory).not.toHaveBeenCalled()
    expect(result).toEqual({
      deleted: false,
      confirmed: false,
      message: 'Delete "Getting started"? It contains 1 element.'
    })
  })

  it('deletes a populated category when the confirmation dialog is accepted', async() => {
    const confirm = vi.fn(() => true)
    const deleteCategory = vi.fn()

    const result = await deleteCategoryWithConfirmation({
      title: 'Getting started',
      items: [{ id: 'item-1' }, { id: 'item-2' }]
    }, {
      confirm,
      deleteCategory
    })

    expect(confirm).toHaveBeenCalledTimes(1)
    expect(deleteCategory).toHaveBeenCalledTimes(1)
    expect(deleteCategory).toHaveBeenCalledWith({
      title: 'Getting started',
      items: [{ id: 'item-1' }, { id: 'item-2' }]
    })
    expect(result).toEqual({
      deleted: true,
      confirmed: true,
      message: 'Delete "Getting started"? It contains 2 elements.'
    })
  })

  it('deletes immediately when the category is empty', async() => {
    const confirm = vi.fn()
    const deleteCategory = vi.fn()

    const result = await deleteCategoryWithConfirmation({
      title: 'Empty category',
      items: []
    }, {
      confirm,
      deleteCategory
    })

    expect(confirm).not.toHaveBeenCalled()
    expect(deleteCategory).toHaveBeenCalledTimes(1)
    expect(deleteCategory).toHaveBeenCalledWith({
      title: 'Empty category',
      items: []
    })
    expect(result).toEqual({
      deleted: true,
      confirmed: true,
      message: ''
    })
  })
})
