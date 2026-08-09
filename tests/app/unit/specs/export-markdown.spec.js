import ExportMarkdown from '../../../../Elephant/frontend/src/muya/lib/utils/exportMarkdown'
import { CURSOR_ANCHOR_DNA, CURSOR_FOCUS_DNA } from '../../../../Elephant/frontend/src/muya/lib/config'

// Build a minimal table block structure that ExportMarkdown.normalizeTable expects.
const makeTableBlock = (headerCells, bodyRows) => {
  const makeCell = (text, type = 'td', align = '') => ({
    type,
    align,
    children: [{ text }],
  })

  return {
    type: 'table',
    children: [
      {
        type: 'thead',
        children: [
          {
            type: 'tr',
            children: headerCells.map((text) => makeCell(text, 'th')),
          },
        ],
      },
      {
        type: 'tbody',
        children: bodyRows.map((cells) => ({
          type: 'tr',
          children: cells.map((text) => makeCell(text, 'td')),
        })),
      },
    ],
  }
}

describe('ExportMarkdown.normalizeTable', () => {
  let exporter

  beforeEach(() => {
    exporter = new ExportMarkdown([])
  })

  it('exports a well-formed table without error', () => {
    const table = makeTableBlock(['col1', 'col2'], [['a', 'b'], ['c', 'd']])
    expect(() => exporter.normalizeTable(table, '')).not.to.throw()
    const output = exporter.normalizeTable(table, '')
    expect(output).to.include('col1')
    expect(output).to.include('col2')
  })

  it('does not throw when a body row has more cells than the header (issue #4190)', () => {
    // Header has 2 columns, but the body row has 3 — this crashed before the fix.
    const table = makeTableBlock(['col1', 'col2'], [['a', 'b', 'EXTRA']])
    expect(() => exporter.normalizeTable(table, '')).not.to.throw()
    const output = exporter.normalizeTable(table, '')
    // Extra cell should be silently dropped — only 2 columns in output.
    expect(output).to.include('col1')
    expect(output).to.include('col2')
    expect(output).not.to.include('EXTRA')
  })

  it('handles a body row with fewer cells than the header', () => {
    const table = makeTableBlock(['col1', 'col2', 'col3'], [['a']])
    expect(() => exporter.normalizeTable(table, '')).not.to.throw()
  })
})

describe('ExportMarkdown cursor markers', () => {
  it('never leaks Muya cursor DNA into exported Markdown', () => {
    const heading = {
      type: 'h1',
      headingStyle: 'atx',
      children: [{ text: `${CURSOR_ANCHOR_DNA}# Title${CURSOR_FOCUS_DNA}` }]
    }
    const output = new ExportMarkdown([heading]).generate()

    expect(output).to.equal('# Title\n')
    expect(output).not.to.include(CURSOR_ANCHOR_DNA)
    expect(output).not.to.include(CURSOR_FOCUS_DNA)
  })
})
