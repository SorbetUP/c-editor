import { createPlainPreview, getMarkdownDocumentTitle } from '../models/markdown-document.js';
import { resolveNotesLibraryIntent, resolveNotesLibraryKeyIntent } from './notes-library-intents.js';
import { dispatchIntent, UI_INTENT } from './ui-intents.js';

function formatDate(value) {
  return new Intl.DateTimeFormat('fr-FR', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  }).format(new Date(value));
}

export class NotesLibraryElement extends HTMLElement {
  constructor() {
    super();
    this.state = null;
    this.renderer = null;
    this.richPreviewEnabled = false;
    this.previewRenderId = 0;
  }

  connectedCallback() {
    if (!this.dataset.bound) {
      this.dataset.bound = 'true';
      this.addEventListener('click', this.#handleClick.bind(this));
      this.addEventListener('input', this.#handleInput.bind(this));
      this.addEventListener('keydown', this.#handleKeydown.bind(this));
    }
    this.render();
  }

  setState(state) {
    this.state = state;
    this.render();
  }

  setRenderer(renderer) {
    this.renderer = renderer;
  }

  enableRichPreview() {
    this.richPreviewEnabled = true;
    this.render();
  }

  render() {
    if (!this.state) {
      return;
    }

    const { notes, source, ui } = this.state;
    const pinnedNotes = notes.filteredItems.filter((note) => note.pinned);
    const regularNotes = notes.filteredItems.filter((note) => !note.pinned);

    this.innerHTML = `
      <section class="notes-browser-shell">
        <div class="notes-browser-toolbar">
          <div class="notes-search">
            <svg class="notes-search-icon" viewBox="0 0 24 24" aria-hidden="true">
              <circle cx="11" cy="11" r="6.5"></circle>
              <path d="M16 16l4.5 4.5"></path>
            </svg>
            <input id="notesSearchInput" type="search" placeholder="Recherche" value="${this.#escape(ui.searchQuery)}" />
          </div>
          <div class="notes-browser-actions">
            <source-status mode="${this.#escape(source.type)}" label="${this.#escape(source.label)}"></source-status>
            <button class="browser-btn" type="button" data-intent-action="${UI_INTENT.OPEN_SOURCE}">Ouvrir dossier</button>
            <button class="browser-btn" type="button" data-intent-action="${UI_INTENT.IMPORT_FILE}">Importer fichier</button>
          </div>
        </div>

        <div class="notes-browser-content">
          <button class="notes-create-bar" type="button" data-intent-action="${UI_INTENT.CREATE_NOTE}" ${source.capabilities.canCreate ? '' : 'disabled'}>
            <div class="notes-create-text">${source.capabilities.canCreate ? 'Créer une note...' : 'Source en lecture seule'}</div>
          </button>

          ${this.#renderSection('Épinglées', pinnedNotes, { hidden: pinnedNotes.length === 0, showEmpty: false, canDelete: source.capabilities.canDelete })}
          ${this.#renderSection('Notes', regularNotes, { hidden: false, showEmpty: true, canDelete: source.capabilities.canDelete })}
        </div>
      </section>
    `;

    if (this.richPreviewEnabled && this.renderer) {
      this.#progressivelyEnhancePreviews();
    }
  }

  #renderSection(label, notes, options) {
    const hiddenClass = options.hidden ? ' is-hidden' : '';
    const body = notes.length === 0
      ? (options.showEmpty ? '<div class="note-tile-empty">Aucune note.</div>' : '')
      : notes.map((note) => this.#renderNoteTile(note, options.canDelete)).join('');

    return `
      <section class="notes-section${hiddenClass}">
        <div class="notes-section-label">${label}</div>
        <div class="notes-grid">
          ${body}
        </div>
      </section>
    `;
  }

  #renderNoteTile(note, canDelete) {
    const title = getMarkdownDocumentTitle(note.content, 'Nouvelle page');
    const relativePath = note.relativePath
      ? `<div class="note-tile-path">${this.#escape(note.relativePath)}</div>`
      : '';

    return `
      <article class="note-tile" role="article" data-note-id="${this.#escape(note.id)}">
        <div class="note-tile-controls">
          <button class="note-tile-pin${note.pinned ? ' is-active' : ''}" type="button" data-intent-action="${UI_INTENT.TOGGLE_PIN}" data-note-id="${this.#escape(note.id)}" aria-label="${note.pinned ? 'Retirer des épinglées' : 'Épingler la note'}">
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M9 3h6"></path>
              <path d="M10 3v5l-3 4v1h10v-1l-3-4V3"></path>
              <path d="M12 13v8"></path>
            </svg>
          </button>
          ${canDelete ? `
            <button class="note-tile-delete" type="button" data-intent-action="${UI_INTENT.DELETE_NOTE}" data-note-id="${this.#escape(note.id)}" aria-label="Supprimer">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <path d="M18 6L6 18"></path>
                <path d="M6 6l12 12"></path>
              </svg>
            </button>
          ` : ''}
        </div>
        <div class="note-tile-open" role="button" tabindex="0" data-intent-open-note="true" data-note-id="${this.#escape(note.id)}" aria-label="Ouvrir ${this.#escape(title)}">
          <div class="note-tile-meta">${this.#escape(formatDate(note.updatedAt))}</div>
          <div class="note-tile-title">${this.#escape(title)}</div>
          ${relativePath}
          <div class="note-tile-body" data-preview-id="${this.#escape(note.id)}">${this.#escape(createPlainPreview(note.content))}</div>
        </div>
      </article>
    `;
  }

  async #progressivelyEnhancePreviews() {
    const renderId = ++this.previewRenderId;
    const cards = Array.from(this.querySelectorAll('[data-preview-id]'));
    for (const card of cards) {
      if (renderId !== this.previewRenderId) {
        return;
      }
      const note = this.state.notes.filteredItems.find((entry) => entry.id === card.dataset.previewId);
      if (!note) {
        continue;
      }
      const html = await this.renderer.renderPreview(note.content);
      if (renderId !== this.previewRenderId) {
        return;
      }
      card.innerHTML = html;
    }
  }

  #handleClick(event) {
    const intent = resolveNotesLibraryIntent(event.target);
    if (!intent) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    dispatchIntent(this, intent.type, intent.detail);
  }

  #handleInput(event) {
    if (event.target.id === 'notesSearchInput') {
      dispatchIntent(this, UI_INTENT.SEARCH_CHANGE, { query: event.target.value });
    }
  }

  #handleKeydown(event) {
    const intent = resolveNotesLibraryKeyIntent(event.target, event.key);
    if (!intent) {
      return;
    }

    event.preventDefault();
    dispatchIntent(this, intent.type, intent.detail);
  }

  #escape(value) {
    return String(value ?? '')
      .replaceAll('&', '&amp;')
      .replaceAll('<', '&lt;')
      .replaceAll('>', '&gt;')
      .replaceAll('"', '&quot;');
  }
}

if (!customElements.get('notes-library')) {
  customElements.define('notes-library', NotesLibraryElement);
}
