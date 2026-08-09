import { describe, expect, it } from 'vitest'
import { countTestContracts, findForbiddenTestAdditions } from '../../../../../../build/scripts/test-integrity-core.mjs'

describe('test integrity aliases', () => {
  it.each([
    ['+fit("focused", () => expect(value).toBe(1))', 'focused test'],
    ['+fdescribe("focused", () => {})', 'focused test'],
    ['+xit("disabled", () => expect(value).toBe(1))', 'disabled test'],
    ['+xdescribe("disabled", () => {})', 'disabled test']
  ])('rejects alias invocation %s', (line, rule) => {
    expect(findForbiddenTestAdditions(line)).toEqual([expect.objectContaining({ rule })])
  })

  it.each([
    '+const fit = calculateFit(points)',
    '+const xitValue = "ordinary identifier"',
    '+const fdescribeResult = describeFeature()',
    '+const xdescribeResult = describeFeature()'
  ])('does not reject ordinary identifiers: %s', (line) => {
    expect(findForbiddenTestAdditions(line)).toEqual([])
  })
})

describe('complex table contract counting', () => {
  it('counts an each table containing arrow functions and parentheses', () => {
    const source = `
      test.each([
        [() => buildValue(), "first"],
        [() => otherValue(), "second"]
      ])("handles %s", (factory, expected) => {
        expect(factory()).toBe(expected)
      })
    `
    expect(countTestContracts(source)).toEqual({ tests: 1, assertions: 1 })
  })

  it('counts direct and table tests without double-counting', () => {
    const source = `
      it("direct", () => expect(one).toBe(1))
      it.each([[2], [3]])("table", value => expect(value).toBeGreaterThan(1))
    `
    expect(countTestContracts(source)).toEqual({ tests: 2, assertions: 2 })
  })
})
