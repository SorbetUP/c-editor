export const normalizePastedHtml = (html = '') => String(html)
  .replace(/<!--[^]*?-->/g, '')
  .replace(/\sclass="Mso[^"]*"/g, '')
  .replace(/\sstyle="[^"]*mso-[^"]*"/gi, '')
  .replace(/\sstyle="[^"]*font-family:[^"]*"/gi, '')
  .replace(/\sdata-block-id="[^"]*"/g, '')
  .replace(/\sdata-notion-[^=]+="[^"]*"/g, '')
  .replace(/\scontenteditable="[^"]*"/g, '')
  .replace(/<meta[^>]*>/gi, '')
  .replace(/<o:p>[\s\S]*?<\/o:p>/gi, '')
  .replace(/<span[^>]*>/gi, '')
  .replace(/<\/span>/gi, '')

export const sanitizePastedHtml = (html = '') => normalizePastedHtml(html)
  .replace(/<script[\s\S]*?<\/script>/gi, '')
  .replace(/<style[\s\S]*?<\/style>/gi, '')
  .replace(/\son[a-z]+="[^"]*"/gi, '')
  .replace(/javascript:/gi, 'blocked-javascript:')

export const pastedHtmlToMarkdown = (html = '') => {
  const safe = sanitizePastedHtml(html)
  const tables = []
  const withTables = safe.replace(/<table[^>]*>([\s\S]*?)<\/table>/gi, (_, table) => {
    const token = `\n\n@@MUYA_TABLE_${tables.length}@@\n\n`
    tables.push(tableToMarkdown(table))
    return token
  })
  const markdown = withTables
    .replace(/<h([1-6])[^>]*>([\s\S]*?)<\/h\1>/gi, (_, level, text) => `${'#'.repeat(Number(level))} ${stripTags(text)}\n\n`)
    .replace(/<li[^>]*>\s*<input[^>]*checked[^>]*>\s*([\s\S]*?)<\/li>/gi, (_, text) => `- [x] ${stripTags(text)}\n`)
    .replace(/<li[^>]*>\s*<input[^>]*>\s*([\s\S]*?)<\/li>/gi, (_, text) => `- [ ] ${stripTags(text)}\n`)
    .replace(/<li[^>]*>([\s\S]*?)<\/li>/gi, (_, text) => `- ${stripTags(text)}\n`)
    .replace(/<img[^>]*src="([^"]*)"[^>]*alt="([^"]*)"[^>]*>/gi, (_, src, alt) => `![${alt}](${src})`)
    .replace(/<img[^>]*alt="([^"]*)"[^>]*src="([^"]*)"[^>]*>/gi, (_, alt, src) => `![${alt}](${src})`)
    .replace(/<img[^>]*src="([^"]*)"[^>]*>/gi, (_, src) => `![](${src})`)
    .replace(/<a[^>]*href="([^"]*)"[^>]*>([\s\S]*?)<\/a>/gi, (_, href, text) => `[${stripTags(text)}](${href})`)
    .replace(/<strong[^>]*>([\s\S]*?)<\/strong>/gi, (_, text) => `**${stripTags(text)}**`)
    .replace(/<b[^>]*>([\s\S]*?)<\/b>/gi, (_, text) => `**${stripTags(text)}**`)
    .replace(/<em[^>]*>([\s\S]*?)<\/em>/gi, (_, text) => `*${stripTags(text)}*`)
    .replace(/<i[^>]*>([\s\S]*?)<\/i>/gi, (_, text) => `*${stripTags(text)}*`)
    .replace(/<code[^>]*>([\s\S]*?)<\/code>/gi, (_, text) => `\`${stripTags(text)}\``)
    .replace(/<p[^>]*>([\s\S]*?)<\/p>/gi, (_, text) => `${stripTags(text)}\n\n`)
    .replace(/<div[^>]*>([\s\S]*?)<\/div>/gi, (_, text) => `${stripTags(text)}\n\n`)
    .replace(/<br\s*\/?>/gi, '\n')
    .replace(/<[^>]+>/g, '')
    .replace(/@@MUYA_TABLE_(\d+)@@/g, (_, index) => tables[Number(index)] || '')
    .replace(/\n{3,}/g, '\n\n')
    .trim()
  return decodeEntities(markdown)
}

export const tableToMarkdown = (tableHtml = '') => {
  const rows = [...tableHtml.matchAll(/<tr[^>]*>([\s\S]*?)<\/tr>/gi)].map((row) => [...row[1].matchAll(/<t[hd][^>]*>([\s\S]*?)<\/t[hd]>/gi)].map((cell) => stripTags(cell[1])))
  if (!rows.length) return ''
  const header = rows[0]
  const body = rows.slice(1)
  return [`| ${header.join(' | ')} |`, `| ${header.map(() => '-').join(' | ')} |`, ...body.map((row) => `| ${row.join(' | ')} |`)].join('\n')
}

export const clipboardPayloadToMarkdown = ({ html = '', text = '' } = {}) => html ? pastedHtmlToMarkdown(html) : String(text || '')

export const copyMarkdownAndHtml = (markdown = '', renderHtml = (value) => value) => ({ markdown, html: renderHtml(markdown) })

const stripTags = (html = '') => html.replace(/<[^>]+>/g, '').replace(/&nbsp;/g, ' ').replace(/&amp;/g, '&').trim()
const decodeEntities = (text = '') => text.replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&amp;/g, '&')
