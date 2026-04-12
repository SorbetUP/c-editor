import { buildMarkdownDocument } from '../app/models/markdown-document.js';
import { dispatchIntent, UI_INTENT } from '../app/ui/ui-intents.js';

const FRAME_SOURCE = './components/hybrid-note-editor-frame.html?v=20260412a';
const HOST_SOURCE = 'hybrid-note-editor-host';
const FRAME_SOURCE_NAME = 'hybrid-note-editor-frame';

function normalizeDocument(documentValue = {}) {
  const title = String(documentValue.title || 'Nouvelle page').trim() || 'Nouvelle page';
  const body = String(documentValue.body || '');
  const content = documentValue.content || buildMarkdownDocument({ title, body });
  return {
    id: documentValue.id || null,
    title,
    body,
    content,
    writable: documentValue.writable !== false,
    sourceKind: documentValue.sourceKind || 'local-storage'
  };
}

export class HybridNoteEditorElement extends HTMLElement {
  constructor() {
    super();
    this.documentValue = normalizeDocument();
    this.dirty = false;
    this.frameReady = false;
    this.pendingRequestId = 0;
    this.pendingResolvers = new Map();
    this.boundMessageHandler = this.handleFrameMessage.bind(this);
  }

  connectedCallback() {
    if (this.dataset.initialized === 'true') {
      return;
    }

    this.dataset.initialized = 'true';
    this.renderShell();
    this.frame = this.querySelector('[data-editor-frame]');
    window.addEventListener('message', this.boundMessageHandler);
  }

  disconnectedCallback() {
    this.dispose();
  }

  renderShell() {
    this.innerHTML = `
      <div class="editor-host-shell">
        <iframe
          class="editor-host-frame"
          data-editor-frame
          src="${FRAME_SOURCE}"
          title="ElephantNote Editor"
          loading="eager"
          referrerpolicy="no-referrer"
        ></iframe>
      </div>
    `;
  }

  loadDocument(documentValue) {
    this.documentValue = normalizeDocument(documentValue);
    this.dirty = false;
    this.dispatchDirtyChange();
    this.postToFrame('load-document', { document: this.documentValue });
  }

  getDocument() {
    return this.documentValue;
  }

  focus() {
    this.postToFrame('focus-editor');
  }

  hasPendingChanges() {
    return this.dirty;
  }

  saveRequest() {
    this.postToFrame('save-request');
  }

  dispose() {
    window.removeEventListener('message', this.boundMessageHandler);
    this.pendingResolvers.clear();
  }

  postToFrame(type, payload = {}) {
    if (!this.frameReady || !this.frame?.contentWindow) {
      return;
    }

    this.frame.contentWindow.postMessage({
      source: HOST_SOURCE,
      type,
      ...payload
    }, '*');
  }

  dispatchDirtyChange() {
    this.dispatchEvent(new CustomEvent('dirty-change', {
      bubbles: true,
      composed: true,
      detail: { dirty: this.dirty }
    }));
  }

  dispatchChange() {
    this.dispatchEvent(new CustomEvent('change', {
      bubbles: true,
      composed: true,
      detail: { document: this.documentValue }
    }));
  }

  handleFrameMessage(event) {
    const data = event.data || {};
    if (data.source !== FRAME_SOURCE_NAME) {
      return;
    }

    if (this.frame?.contentWindow && event.source !== this.frame.contentWindow) {
      return;
    }

    if (data.type === 'ready') {
      this.frameReady = true;
      this.postToFrame('load-document', { document: this.documentValue });
      this.dispatchEvent(new CustomEvent('ready', { bubbles: true, composed: true }));
      return;
    }

    if (data.type === 'document-loaded' && data.document) {
      this.documentValue = normalizeDocument({ ...this.documentValue, ...data.document });
      this.dirty = false;
      this.dispatchDirtyChange();
      return;
    }

    if (data.type === 'change' && data.document) {
      this.documentValue = normalizeDocument({ ...this.documentValue, ...data.document });
      this.dirty = true;
      this.dispatchChange();
      this.dispatchDirtyChange();
      return;
    }

    if (data.type === 'dirty-change') {
      this.dirty = Boolean(data.dirty);
      this.dispatchDirtyChange();
      return;
    }

    if (data.type === 'save-request') {
      if (data.document) {
        this.documentValue = normalizeDocument({ ...this.documentValue, ...data.document });
      }
      dispatchIntent(this, UI_INTENT.SAVE_NOTE, { document: this.documentValue });
      return;
    }

    if (data.type === 'outside-click') {
      dispatchIntent(this, UI_INTENT.CLOSE_EDITOR, { reason: 'outside-click' });
      return;
    }

    if (data.type === 'document-response') {
      const resolver = this.pendingResolvers.get(data.requestId);
      if (resolver) {
        this.pendingResolvers.delete(data.requestId);
        resolver(this.documentValue = normalizeDocument({ ...this.documentValue, ...data.document }));
      }
    }
  }
}

if (!customElements.get('hybrid-note-editor')) {
  customElements.define('hybrid-note-editor', HybridNoteEditorElement);
}
