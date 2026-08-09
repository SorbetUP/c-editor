import { markdownToSearchText } from 'main_renderer/elephantnote/search/markdownToSearchText'

describe('markdownToSearchText', () => {
  it('keeps useful frontmatter text, headings, links and image alt text', () => {
    const input = `---
title: "World Model"
tags: ["ai", "memory"]
---

# World Model

![schema mémoire](./schema.png)

This note explains latent memory and semantic retrieval.

[Link to paper](https://example.com)
`

    const output = markdownToSearchText(input)

    expect(output).to.contain('World Model')
    expect(output).to.contain('Tags: ai, memory')
    expect(output).to.contain('schema mémoire')
    expect(output).to.contain('This note explains latent memory and semantic retrieval.')
    expect(output).to.contain('Link to paper')
  })

  it('reduces markdown noise', () => {
    const output = markdownToSearchText('* item 1\n1. item 2\n> quoted\n`inline code`')
    expect(output).to.equal('item 1\nitem 2\nquoted\ninline code')
  })

  it('returns an empty string for empty markdown', () => {
    expect(markdownToSearchText('')).to.equal('')
    expect(markdownToSearchText('   ')).to.equal('')
  })
})
