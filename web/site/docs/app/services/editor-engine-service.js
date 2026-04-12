function loadScript(src) {
  return new Promise((resolve, reject) => {
    const existing = document.querySelector(`script[data-elephantnote-src="${src}"]`);
    if (existing) {
      existing.addEventListener('load', resolve, { once: true });
      existing.addEventListener('error', reject, { once: true });
      if (existing.dataset.loaded === 'true') {
        resolve();
      }
      return;
    }

    const script = document.createElement('script');
    script.src = src;
    script.async = true;
    script.dataset.elephantnoteSrc = src;
    script.addEventListener('load', () => {
      script.dataset.loaded = 'true';
      resolve();
    }, { once: true });
    script.addEventListener('error', reject, { once: true });
    document.head.appendChild(script);
  });
}

export class EditorEngineService {
  constructor() {
    this.module = null;
    this.initPromise = null;
    this.stats = {
      initStartedAt: 0,
      initDurationMs: 0,
      renderCount: 0
    };
  }

  async ensureReady() {
    if (this.module) {
      return this.module;
    }

    if (!this.initPromise) {
      this.initPromise = this.#initialize();
    }

    return this.initPromise;
  }

  async #initialize() {
    this.stats.initStartedAt = performance.now();
    const editorScriptUrl = new URL('../../editor.js', import.meta.url).href;
    await loadScript(editorScriptUrl);

    if (typeof window.EditorModule === 'undefined') {
      throw new Error('EditorModule is unavailable.');
    }

    const module = await window.EditorModule();
    if (typeof module.ccall === 'function') {
      try {
        module.ccall('editor_library_init', 'number', [], []);
      } catch (error) {
        console.warn('Unable to initialize editor library:', error);
      }
    }

    this.module = module;
    this.stats.initDurationMs = performance.now() - this.stats.initStartedAt;
    return module;
  }

  async renderMarkdown(markdown) {
    const module = await this.ensureReady();
    this.stats.renderCount += 1;

    if (!markdown || !markdown.trim()) {
      return '<p>Note vide</p>';
    }

    try {
      const html = module.ccall('editor_markdown_to_html', 'string', ['string'], [markdown]);
      if (html && html.trim()) {
        return html;
      }
    } catch (error) {
      console.warn('Direct html rendering failed:', error);
    }

    try {
      const json = module.ccall('editor_parse_markdown_simple', 'string', ['string'], [markdown]);
      if (!json) {
        return this.#fallbackMarkdown(markdown);
      }
      return this.#jsonToHtml(json);
    } catch (error) {
      console.warn('JSON rendering failed:', error);
      return this.#fallbackMarkdown(markdown);
    }
  }

  async warmUp() {
    try {
      await this.ensureReady();
    } catch (error) {
      console.warn('Unable to warm up editor engine:', error);
    }
  }

  destroy() {
    this.module = null;
    this.initPromise = null;
  }

  #jsonToHtml(jsonString) {
    const escapeHtml = (text) => {
      const div = document.createElement('div');
      div.textContent = text;
      return div.innerHTML;
    };

    const renderSpan = (span) => {
      if (!span) {
        return '';
      }

      let html = escapeHtml(span.text || '');
      if (span.code) html = `<code>${html}</code>`;
      if (span.italic) html = `<em>${html}</em>`;
      if (span.bold) html = `<strong>${html}</strong>`;
      if (span.has_underline) html = `<u>${html}</u>`;
      if (span.has_highlight) html = `<mark>${html}</mark>`;
      if (span.strikethrough) html = `<del>${html}</del>`;
      if (span.link && span.href) {
        html = `<a href="${escapeHtml(span.href)}" target="_blank" rel="noreferrer">${html}</a>`;
      }
      return html;
    };

    try {
      const payload = JSON.parse(jsonString);
      const elements = Array.isArray(payload.elements) ? payload.elements : [];
      return elements
        .map((element) => {
          if (Array.isArray(element.spans) && element.spans.length > 0) {
            return `<p>${element.spans.map(renderSpan).join('')}</p>`;
          }
          return `<p>${escapeHtml(element.text || '')}</p>`;
        })
        .join('');
    } catch (error) {
      console.warn('Unable to parse engine json payload:', error);
      return this.#fallbackMarkdown(jsonString);
    }
  }

  #fallbackMarkdown(markdown) {
    const escapeHtml = (text) => {
      const div = document.createElement('div');
      div.textContent = text;
      return div.innerHTML;
    };

    return String(markdown || '')
      .split('\n')
      .map((line) => `<p>${escapeHtml(line)}</p>`)
      .join('');
  }
}

