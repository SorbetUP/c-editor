const DEFAULT_TITLE = 'Nouvelle page';

export function normalizeTitleText(value, fallbackTitle = DEFAULT_TITLE) {
  const normalized = String(value || '')
    .replace(/^\uFEFF/, '')
    .replace(/^#+\s*/, '')
    .replace(/\s+/g, ' ')
    .trim();
  return normalized || fallbackTitle;
}

export function splitMarkdownDocument(markdownText, fallbackTitle = DEFAULT_TITLE) {
  const normalized = String(markdownText || '').replace(/^\uFEFF/, '').replace(/\r\n/g, '\n');
  const lines = normalized.split('\n');
  const firstContentIndex = lines.findIndex((line) => line.trim().length > 0);

  if (firstContentIndex === -1) {
    return {
      title: fallbackTitle,
      body: '',
      content: `# ${fallbackTitle}\n`
    };
  }

  const firstContentLine = lines[firstContentIndex];
  const headingMatch = firstContentLine.match(/^\s*#\s+(.+?)\s*$/);
  const title = normalizeTitleText(headingMatch ? headingMatch[1] : firstContentLine, fallbackTitle);
  const bodyLines = headingMatch ? lines.slice(firstContentIndex + 1) : lines.slice(firstContentIndex);

  while (bodyLines.length > 0 && bodyLines[0].trim() === '') {
    bodyLines.shift();
  }

  return {
    title,
    body: bodyLines.join('\n'),
    content: buildMarkdownDocument({ title, body: bodyLines.join('\n') })
  };
}

export function buildMarkdownDocument({ title, body }) {
  const normalizedTitle = normalizeTitleText(title, DEFAULT_TITLE);
  const normalizedBody = String(body || '').replace(/\r\n/g, '\n').replace(/^\n+/, '').replace(/\s+$/, '');
  if (!normalizedBody) {
    return `# ${normalizedTitle}\n`;
  }
  return `# ${normalizedTitle}\n\n${normalizedBody}`;
}

export function documentToNoteContent(document) {
  return buildMarkdownDocument(document);
}

export function getMarkdownDocumentTitle(markdownText, fallbackTitle = DEFAULT_TITLE) {
  return splitMarkdownDocument(markdownText, fallbackTitle).title;
}

export function buildMarkdownFilename(title) {
  const safeTitle = normalizeTitleText(title, DEFAULT_TITLE)
    .replace(/[<>:"/\\|?*\u0000-\u001F]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
  return `${safeTitle || DEFAULT_TITLE}.md`;
}

export function createPlainPreview(markdownText, maxLength = 220) {
  const { body } = splitMarkdownDocument(markdownText, DEFAULT_TITLE);
  const plain = body
    .replace(/!\[[^\]]*\]\([^)]+\)/g, '[image]')
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
    .replace(/[*_~`>#|-]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
  return plain.slice(0, maxLength) || 'Note vide';
}

export function createDocumentFromNote(note) {
  const { title, body } = splitMarkdownDocument(note.content, DEFAULT_TITLE);
  return {
    id: note.id,
    title,
    body,
    writable: note.writable !== false,
    sourceKind: note.sourceKind || 'local-storage'
  };
}

