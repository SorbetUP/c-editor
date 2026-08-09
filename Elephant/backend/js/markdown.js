export const updateMarkdownTitle = (markdown, title) => {
  const nextTitle = String(title || '').trim()
  if (!nextTitle) return markdown

  const frontmatterMatch = markdown.match(/^---\n([\s\S]*?)\n---\n?/)
  if (!frontmatterMatch) {
    const headingMatch = markdown.match(/^#\s+.+$/m)
    return headingMatch
      ? markdown.replace(/^#\s+.+$/m, `# ${nextTitle}`)
      : `# ${nextTitle}\n\n${markdown}`.trimEnd()
  }

  const frontmatterBody = frontmatterMatch[1]
  const nextFrontmatter = /^title:\s*.*$/m.test(frontmatterBody)
    ? frontmatterBody.replace(/^title:\s*.*$/m, `title: "${nextTitle}"`)
    : `title: "${nextTitle}"\n${frontmatterBody}`.trim()
  const body = markdown.slice(frontmatterMatch[0].length)
  const nextBody = body.replace(/^#\s+.+$/m, `# ${nextTitle}`)
  return `---\n${nextFrontmatter}\n---\n${nextBody}`.trimEnd()
}
