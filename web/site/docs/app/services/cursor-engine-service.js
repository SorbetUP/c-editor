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

export class CursorEngineService {
  constructor() {
    this.initPromise = null;
    this.manager = null;
  }

  async ensureReady() {
    if (this.manager) {
      return this.manager;
    }

    if (!this.initPromise) {
      this.initPromise = this.#initialize();
    }

    return this.initPromise;
  }

  async #initialize() {
    const cursorScriptUrl = new URL('../../cursor_wasm.js', import.meta.url).href;
    const cursorBridgeUrl = new URL('../../cursor_c_interface.js', import.meta.url).href;
    await loadScript(cursorScriptUrl);
    await loadScript(cursorBridgeUrl);

    if (typeof window.CursorModule === 'undefined' || typeof window.initializeCCursorManager === 'undefined') {
      return null;
    }

    const module = await window.CursorModule();
    this.manager = window.initializeCCursorManager(module);
    return this.manager;
  }
}

