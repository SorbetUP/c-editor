export class MarkdownRenderer {
  constructor(editorEngine, options = {}) {
    this.editorEngine = editorEngine;
    this.cache = new Map();
    this.maxEntries = options.maxEntries || 80;
  }

  async render(markdown) {
    const cacheKey = String(markdown || '');
    if (this.cache.has(cacheKey)) {
      const value = this.cache.get(cacheKey);
      this.cache.delete(cacheKey);
      this.cache.set(cacheKey, value);
      return value;
    }

    const html = await this.editorEngine.renderMarkdown(cacheKey);
    this.cache.set(cacheKey, html);
    if (this.cache.size > this.maxEntries) {
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
    return html;
  }

  async renderPreview(markdown) {
    return this.render(markdown);
  }

  warmUp() {
    return this.editorEngine.warmUp();
  }
}
