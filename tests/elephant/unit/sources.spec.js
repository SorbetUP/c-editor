import { describe, expect, it } from 'vitest'
import {
  createSourceId,
  extractHtmlTitle,
  htmlToReadableText,
  normalizeSourceRecord,
  normalizeSourceUrl,
  parseRssFeed
} from 'common/elephantnote/sources'

describe('ElephantNote sources', () => {
  it('normalizes source URLs and ids', () => {
    expect(normalizeSourceUrl('example.com/post')).toBe('https://example.com/post')
    expect(createSourceId('https://Example.com/Post?id=1')).toBe('example-com-post-id-1')
  })

  it('extracts readable title and text from HTML', () => {
    const html = '<html><title>Article &amp; Notes</title><body><h1>Title</h1><p>Hello <strong>world</strong>.</p></body></html>'

    expect(extractHtmlTitle(html)).toBe('Article & Notes')
    expect(htmlToReadableText(html)).toContain('Hello world.')
  })

  it('parses RSS feed items', () => {
    const items = parseRssFeed(`
      <rss><channel>
        <item>
          <title>First</title>
          <link>https://example.com/first</link>
          <description><![CDATA[<p>Body</p>]]></description>
        </item>
      </channel></rss>
    `)

    expect(items).toEqual([
      {
        title: 'First',
        url: 'https://example.com/first',
        publishedAt: '',
        description: 'Body'
      }
    ])
  })

  it('normalizes source records', () => {
    expect(normalizeSourceRecord({
      url: 'example.com/a',
      title: 'A',
      notePath: 'Sources/A.md'
    })).toMatchObject({
      id: 'example-com-a',
      url: 'https://example.com/a',
      title: 'A',
      type: 'url',
      notePath: 'Sources/A.md'
    })
  })
})
