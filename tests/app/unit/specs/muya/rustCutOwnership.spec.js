import { describe, expect, it } from 'vitest'

import { commandForBeforeInput } from '../../../../../Elephant/frontend/src/muya/lib/rust/inputController/commands'

describe('Rust cut and drag ownership', () => {
  it.each(['deleteByCut', 'deleteByDrag', 'deleteContent'])(
    'routes %s through the Rust deletion transaction',
    (inputType) => {
      expect(commandForBeforeInput({ inputType })).toEqual({ type: 'delete_backward' })
    }
  )

  it('routes replacement text through the Rust insertion transaction', () => {
    expect(
      commandForBeforeInput({ inputType: 'insertReplacementText', data: 'replacement' })
    ).toEqual({ type: 'insert_text', text: 'replacement' })
  })
})
