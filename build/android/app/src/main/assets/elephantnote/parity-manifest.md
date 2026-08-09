# Android parity manifest

folder: Guides
#android #parity #release

This release is a native Android implementation of the mobile-compatible parts of ElephantNote. It is not a zipped Electron desktop app. Android gets dedicated storage, UI and lifecycle behavior while preserving vault concepts.

## Included in the APK

- Native launcher activity and Android share receiver.
- Offline Markdown notes stored as real `.md` files under `ElephantVault/Notes`.
- App-specific local vault structure for notes, attachments, canvas data and mobile metadata.
- Markdown notes with local clipboard and file import/export.
- Mobile Markdown toolbar for headings, bold, italic, tasks, wiki links, code, quotes, tags and folders.
- Native Markdown preview for note details.
- Folder inference through `folder:` metadata and matching vault directories.
- Tag extraction, tag counts and clickable wiki topics.
- Search across local vault content.
- Backlink parsing for `[[Wiki Links]]`.
- Graph summary for notes, tags and backlinks.
- Topic drill-down from Wiki and Graph into linked notes.
- Shared text, single image and multiple image capture.
- Shared images copied into `ElephantVault/Attachments`.
- Attachment browser with file URI copy, generated notes and deletion.
- Source URL ingestion and generated source notes.
- Calendar event storage in `.elephantnote/mobile-calendar.json`.
- Canvas drawing activity with metadata in `Canvas/drawings.json`.
- Model slot settings for embedding, chat and OCR roles.
- Sync device id, folder id and remote path configuration.
- Bundled starter guides and reusable templates.

## Still desktop-specific

- Electron IPC.
- Full MarkText editor engine.
- Node-based local model runtimes.
- Desktop OCR execution.
- Excalidraw desktop file compatibility.
- Background sync daemons running inside the APK.

Heavy AI/OCR should run on desktop for now. The phone keeps capture, browsing, editing, attachments, graph/wiki navigation and Markdown handoff.
