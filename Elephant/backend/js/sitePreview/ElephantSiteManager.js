import fs from 'fs-extra'
import path from 'path'
import { createId, WORKSPACE_DIR } from '../core'
import { SITE_BUILD_MODE, SITE_PREVIEW_STATUS } from './siteTypes'
import { assertPathInsideVault, isAllowedSiteFile, isIgnoredForSite, isValidSiteSourceFolder } from './pathSafety'

const PREVIEWS_DIR = 'site-previews'
const BUILDS_DIR = 'site-builds'
const SITE_CSS_PATH = 'assets/elephantnote-site.css'

const createSiteId = (vaultRoot, folderPath) => {
  const relativePath = path.relative(path.resolve(vaultRoot), path.resolve(folderPath))
  return createId(relativePath || path.basename(folderPath) || 'site')
}

const writeLog = async(siteDir, message) => {
  const logsDir = path.join(siteDir, 'logs')
  await fs.ensureDir(logsDir)
  await fs.appendFile(path.join(logsDir, 'site-preview.log'), `${new Date().toISOString()} ${message}\n`, 'utf8')
}

const escapeHtml = (value) => String(value || '')
  .replace(/&/g, '&amp;')
  .replace(/</g, '&lt;')
  .replace(/>/g, '&gt;')
  .replace(/"/g, '&quot;')
  .replace(/'/g, '&#39;')

const toPosix = (value) => value.split(path.sep).join('/')

const stripMarkdownExtension = (relativePath) => relativePath.replace(/\.(md|markdown)$/i, '')

const routeForMarkdown = (relativePath) => {
  const withoutExt = stripMarkdownExtension(toPosix(relativePath))
  if (withoutExt.toLowerCase() === 'index') return 'index.html'
  if (withoutExt.toLowerCase().endsWith('/index')) return `${withoutExt.slice(0, -'/index'.length)}/index.html`
  return `${withoutExt}/index.html`
}

const outputRelativeDirForMarkdown = (relativePath) => path.posix.dirname(routeForMarkdown(relativePath))

const normalizeLocalHref = (href, fromMarkdownPath) => {
  if (!href || /^(?:[a-z]+:|#|\/\/)/i.test(href)) return href
  const [target, suffix = ''] = href.split(/(?=[?#])/)

  const fromDir = path.posix.dirname(toPosix(fromMarkdownPath))
  const normalizedTarget = path.posix.normalize(path.posix.join(fromDir, target))
  const targetRoute = /\.(md|markdown)$/i.test(target) ? routeForMarkdown(normalizedTarget) : normalizedTarget
  const fromOutputDir = outputRelativeDirForMarkdown(fromMarkdownPath)
  const relative = path.posix.relative(fromOutputDir === '.' ? '' : fromOutputDir, targetRoute) || 'index.html'
  const clean = /\.(md|markdown)$/i.test(target) ? relative.replace(/index\.html$/i, '') : relative
  return `${clean || './'}${suffix}`
}

const renderInlineMarkdown = (line, fromMarkdownPath) => {
  let html = escapeHtml(line)
  html = html.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, (_match, alt, href) => {
    const normalizedHref = normalizeLocalHref(href.trim(), fromMarkdownPath)
    return `<img src="${escapeHtml(normalizedHref)}" alt="${escapeHtml(alt)}">`
  })
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_match, label, href) => {
    const normalizedHref = normalizeLocalHref(href.trim(), fromMarkdownPath)
    return `<a href="${escapeHtml(normalizedHref)}">${escapeHtml(label)}</a>`
  })
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
  html = html.replace(/\*([^*]+)\*/g, '<em>$1</em>')
  html = html.replace(/`([^`]+)`/g, '<code>$1</code>')
  return html
}

const readTitle = (markdown, fallback) => {
  const heading = markdown.split(/\r?\n/).find((line) => /^#\s+/.test(line))
  return heading ? heading.replace(/^#\s+/, '').trim() : fallback
}

const renderMarkdown = (markdown, relativePath) => {
  const lines = markdown.replace(/\r\n/g, '\n').split('\n')
  const blocks = []
  let paragraph = []
  let list = []
  let code = []
  let inCode = false

  const flushParagraph = () => {
    if (!paragraph.length) return
    blocks.push(`<p>${renderInlineMarkdown(paragraph.join(' '), relativePath)}</p>`)
    paragraph = []
  }

  const flushList = () => {
    if (!list.length) return
    blocks.push(`<ul>${list.map((item) => `<li>${renderInlineMarkdown(item, relativePath)}</li>`).join('')}</ul>`)
    list = []
  }

  for (const line of lines) {
    if (/^```/.test(line)) {
      if (inCode) {
        blocks.push(`<pre><code>${escapeHtml(code.join('\n'))}</code></pre>`)
        code = []
        inCode = false
      } else {
        flushParagraph()
        flushList()
        inCode = true
      }
      continue
    }

    if (inCode) {
      code.push(line)
      continue
    }

    if (!line.trim()) {
      flushParagraph()
      flushList()
      continue
    }

    if (/^---+$/.test(line.trim())) {
      flushParagraph()
      flushList()
      blocks.push('<hr>')
      continue
    }

    const heading = /^(#{1,6})\s+(.+)$/.exec(line)
    if (heading) {
      flushParagraph()
      flushList()
      const level = heading[1].length
      blocks.push(`<h${level}>${renderInlineMarkdown(heading[2], relativePath)}</h${level}>`)
      continue
    }

    const listItem = /^\s*[-*]\s+(.+)$/.exec(line)
    if (listItem) {
      flushParagraph()
      list.push(listItem[1])
      continue
    }

    paragraph.push(line.trim())
  }

  if (inCode) blocks.push(`<pre><code>${escapeHtml(code.join('\n'))}</code></pre>`)
  flushParagraph()
  flushList()
  return blocks.join('\n')
}

const renderPage = ({ title, body, cssHref, navLinks }) => `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${escapeHtml(title)}</title>
  <link rel="stylesheet" href="${escapeHtml(cssHref)}">
</head>
<body>
  <aside class="site-nav">
    <a class="site-brand" href="${escapeHtml(navLinks.homeHref)}">ElephantNote</a>
    <nav>${navLinks.items.map((item) => `<a href="${escapeHtml(item.href)}">${escapeHtml(item.title)}</a>`).join('')}</nav>
  </aside>
  <main class="site-main">
${body}
  </main>
</body>
</html>
`

const renderCss = () => `:root {
  color-scheme: light dark;
  --bg: #f8fafc;
  --paper: #ffffff;
  --text: #111827;
  --muted: #64748b;
  --line: #d8e0ec;
  --accent: #2563eb;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: #0b111b;
    --paper: #111827;
    --text: #eef2ff;
    --muted: #9ca3af;
    --line: #253248;
    --accent: #60a5fa;
  }
}
* { box-sizing: border-box; }
body {
  margin: 0;
  background: var(--bg);
  color: var(--text);
  display: grid;
  grid-template-columns: minmax(180px, 260px) minmax(0, 1fr);
  min-height: 100vh;
  font: 16px/1.6 system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}
.site-nav {
  border-right: 1px solid var(--line);
  padding: 28px 22px;
  background: color-mix(in srgb, var(--paper) 86%, transparent);
}
.site-brand {
  display: block;
  margin-bottom: 22px;
  color: var(--text);
  font-weight: 800;
  text-decoration: none;
}
.site-nav nav {
  display: grid;
  gap: 8px;
}
.site-nav nav a {
  color: var(--muted);
  text-decoration: none;
}
.site-nav nav a:hover {
  color: var(--accent);
}
.site-main {
  width: min(920px, 100%);
  padding: 48px 32px 72px;
}
h1, h2, h3 {
  line-height: 1.15;
}
a {
  color: var(--accent);
}
img {
  max-width: 100%;
  height: auto;
  border-radius: 8px;
}
pre {
  overflow: auto;
  padding: 16px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: color-mix(in srgb, var(--paper) 74%, var(--bg));
}
hr {
  border: 0;
  border-top: 1px solid var(--line);
  margin: 28px 0;
}
@media (max-width: 760px) {
  body {
    display: block;
  }
  .site-nav {
    border-right: 0;
    border-bottom: 1px solid var(--line);
  }
  .site-main {
    padding: 32px 20px 56px;
  }
}
`

export class ElephantSiteManager {
  getSitePaths({ vaultRoot, folderPath, mode }) {
    const id = createSiteId(vaultRoot, folderPath)
    const baseDirName = mode === SITE_BUILD_MODE.STATIC_EXPORT ? BUILDS_DIR : PREVIEWS_DIR
    const siteRoot = path.join(vaultRoot, WORKSPACE_DIR, baseDirName, id)
    return {
      id,
      siteRoot,
      outputDir: path.join(siteRoot, 'site'),
      tmpDir: path.join(siteRoot, 'tmp')
    }
  }

  async preparePreview(request) {
    return this.buildWithMode({ ...request, mode: SITE_BUILD_MODE.PREVIEW })
  }

  async buildStaticSite(request) {
    return this.buildWithMode({ ...request, mode: SITE_BUILD_MODE.STATIC_EXPORT })
  }

  async buildWithMode({ vaultRoot, folderPath, mode }) {
    const sourceFolder = path.resolve(folderPath)
    assertPathInsideVault(vaultRoot, sourceFolder)
    if (!(await isValidSiteSourceFolder(vaultRoot, sourceFolder))) {
      throw new Error('This folder does not contain Markdown notes.')
    }

    const paths = this.getSitePaths({ vaultRoot, folderPath: sourceFolder, mode })
    await fs.remove(paths.tmpDir)
    await fs.remove(paths.outputDir)
    await fs.ensureDir(paths.outputDir)
    await fs.ensureDir(paths.tmpDir)

    const manifestPath = path.join(paths.siteRoot, 'elephant-site.json')
    const info = {
      id: paths.id,
      vaultRoot: path.resolve(vaultRoot),
      sourceFolder,
      configPath: manifestPath,
      outputDir: paths.outputDir,
      tmpDir: paths.tmpDir,
      status: SITE_PREVIEW_STATUS.BUILDING
    }

    try {
      await writeLog(paths.siteRoot, `Building ${mode} from ${sourceFolder}`)
      const pages = await this.collectSourceFiles({ sourceFolder, vaultRoot })
      const markdownPages = await this.writePages({ sourceFolder, outputDir: paths.outputDir, pages })
      if (!markdownPages.length) throw new Error('No Markdown pages were generated.')

      await fs.writeJson(manifestPath, {
        title: path.basename(sourceFolder) || 'ElephantNote',
        sourceFolder,
        outputDir: paths.outputDir,
        pages: markdownPages.map(({ title, route }) => ({ title, route })),
        generatedAt: new Date().toISOString()
      }, { spaces: 2 })
      await writeLog(paths.siteRoot, `Build complete: ${paths.outputDir}`)
      return {
        ...info,
        status: mode === SITE_BUILD_MODE.PREVIEW ? SITE_PREVIEW_STATUS.SERVING : SITE_PREVIEW_STATUS.READY
      }
    } catch (err) {
      await writeLog(paths.siteRoot, `Build failed: ${err.stack || err.message || err}`)
      return {
        ...info,
        status: SITE_PREVIEW_STATUS.ERROR,
        error: 'Website preview failed. See logs for details.'
      }
    }
  }

  async collectSourceFiles({ sourceFolder, vaultRoot }) {
    const files = []
    const walk = async(directory) => {
      const entries = await fs.readdir(directory, { withFileTypes: true }).catch(() => [])
      for (const entry of entries) {
        const fullPath = path.join(directory, entry.name)
        const relativeToVault = path.relative(vaultRoot, fullPath)
        const relativeToSource = path.relative(sourceFolder, fullPath)
        if (isIgnoredForSite(relativeToVault)) continue
        if (entry.isDirectory()) {
          await walk(fullPath)
        } else if (entry.isFile() && isAllowedSiteFile(fullPath)) {
          files.push({ fullPath, relativePath: relativeToSource })
        }
      }
    }
    await walk(sourceFolder)
    return files.sort((a, b) => a.relativePath.localeCompare(b.relativePath))
  }

  async writePages({ sourceFolder, outputDir, pages }) {
    const markdownFiles = pages.filter(({ relativePath }) => /\.(md|markdown)$/i.test(relativePath))
    const markdownPages = await Promise.all(markdownFiles.map(async(file) => {
      const markdown = await fs.readFile(file.fullPath, 'utf8')
      const title = readTitle(markdown, stripMarkdownExtension(path.basename(file.relativePath)))
      const route = routeForMarkdown(file.relativePath)
      return { ...file, markdown, title, route }
    }))

    const navItems = markdownPages.map((page) => ({
      title: page.title,
      route: page.route
    }))

    await fs.outputFile(path.join(outputDir, SITE_CSS_PATH), renderCss(), 'utf8')

    for (const page of markdownPages) {
      const outputPath = path.join(outputDir, page.route)
      const outputDirRelative = path.posix.dirname(page.route)
      const depthPrefix = outputDirRelative === '.' ? '.' : path.posix.relative(outputDirRelative, '.') || '.'
      const cssHref = `${depthPrefix}/${SITE_CSS_PATH}`.replace(/^\.\//, './')
      const homeHref = `${depthPrefix}/index.html`.replace(/^\.\//, './')
      const items = navItems.map((item) => ({
        title: item.title,
        href: path.posix.relative(outputDirRelative === '.' ? '' : outputDirRelative, item.route) || 'index.html'
      }))
      const body = renderMarkdown(page.markdown, page.relativePath)
      await fs.outputFile(outputPath, renderPage({
        title: page.title,
        body,
        cssHref,
        navLinks: { homeHref, items }
      }), 'utf8')
    }

    if (!await fs.pathExists(path.join(outputDir, 'index.html'))) {
      const links = markdownPages.map((page) => `<li><a href="./${escapeHtml(page.route)}">${escapeHtml(page.title)}</a></li>`).join('')
      await fs.outputFile(path.join(outputDir, 'index.html'), renderPage({
        title: path.basename(sourceFolder) || 'ElephantNote',
        body: `<h1>${escapeHtml(path.basename(sourceFolder) || 'ElephantNote')}</h1><ul>${links}</ul>`,
        cssHref: `./${SITE_CSS_PATH}`,
        navLinks: {
          homeHref: './index.html',
          items: navItems.map((item) => ({ title: item.title, href: `./${item.route}` }))
        }
      }), 'utf8')
    }

    for (const file of pages) {
      if (/\.(md|markdown)$/i.test(file.relativePath)) continue
      await fs.copy(file.fullPath, path.join(outputDir, file.relativePath))
    }

    return markdownPages
  }

  async cleanPreview(siteId, vaultRoot) {
    if (!siteId || !vaultRoot) return
    const previewRoot = path.join(vaultRoot, WORKSPACE_DIR, PREVIEWS_DIR)
    const target = path.join(previewRoot, siteId)
    assertPathInsideVault(previewRoot, target)
    await fs.remove(target)
  }

  async cleanAllPreviews(vaultRoot, olderThanMs = 7 * 24 * 60 * 60 * 1000) {
    const previewRoot = path.join(vaultRoot, WORKSPACE_DIR, PREVIEWS_DIR)
    const entries = await fs.readdir(previewRoot, { withFileTypes: true }).catch(() => [])
    const cutoff = Date.now() - olderThanMs

    for (const entry of entries) {
      if (!entry.isDirectory()) continue
      const fullPath = path.join(previewRoot, entry.name)
      const stats = await fs.stat(fullPath).catch(() => null)
      if (stats && stats.mtimeMs < cutoff) {
        await fs.remove(fullPath)
      }
    }
  }
}
