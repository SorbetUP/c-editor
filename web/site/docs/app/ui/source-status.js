export class SourceStatusElement extends HTMLElement {
  static get observedAttributes() {
    return ['mode', 'label'];
  }

  connectedCallback() {
    this.render();
  }

  attributeChangedCallback() {
    this.render();
  }

  render() {
    const mode = this.getAttribute('mode') || 'local-storage';
    const label = this.getAttribute('label') || 'Local';
    const isReadonly = mode === 'web-directory-readonly';
    const text = isReadonly
      ? 'Lecture seule'
      : mode === 'web-directory-readwrite'
        ? 'Lecture / ecriture'
        : mode === 'native-directory'
          ? 'Natif'
          : 'Local';

    this.className = `source-status${isReadonly ? ' is-readonly' : ''}`;
    this.innerHTML = `
      <span class="source-status-mode">${text}</span>
      <span class="source-status-label">${label}</span>
    `;
  }
}

if (!customElements.get('source-status')) {
  customElements.define('source-status', SourceStatusElement);
}

