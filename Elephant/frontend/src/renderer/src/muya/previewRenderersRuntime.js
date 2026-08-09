export const renderKatexPreview = async(latex = '') => {
  try {
    const katex = await import('katex')
    return {
      type: 'katex',
      ok: true,
      html: katex.default?.renderToString ? katex.default.renderToString(latex, { throwOnError: false, displayMode: true }) : katex.renderToString(latex, { throwOnError: false, displayMode: true })
    }
  } catch (error) {
    return { type: 'katex', ok: false, error: error.message, html: `<pre class="katex-fallback">${escapeHtml(latex)}</pre>` }
  }
}

export const renderMermaidPreview = async(source = '', id = 'muya-mermaid-preview') => {
  try {
    const mermaid = await import('mermaid')
    const api = mermaid.default || mermaid
    api.initialize?.({ startOnLoad: false })
    const rendered = await api.render(id, source)
    return { type: 'mermaid', ok: true, html: rendered.svg || rendered }
  } catch (error) {
    return { type: 'mermaid', ok: false, error: error.message, html: `<pre class="diagram-fallback">${escapeHtml(source)}</pre>` }
  }
}

export const renderPreviewBlock = async(block) => {
  if (block?.type === 'math_block') return renderKatexPreview(block.text || '')
  if (block?.type === 'code_fence' && block.language === 'mermaid') return renderMermaidPreview(block.text || '')
  return { type: 'none', ok: true, html: '' }
}

const escapeHtml = (text) => String(text).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;')
