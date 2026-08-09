import { describe, expect, it } from 'vitest'
import { isOutsideElement } from '@/elephantnote/utils/dom'

describe('ElephantNote DOM helpers', () => {
  it('detects whether a click happened outside of a form element', () => {
    const form = {
      contains(target) {
        return target === 'inside'
      }
    }

    expect(isOutsideElement('inside', form)).toBe(false)
    expect(isOutsideElement('outside', form)).toBe(true)
    expect(isOutsideElement(null, form)).toBe(true)
    expect(isOutsideElement('outside', null)).toBe(true)
  })
})
