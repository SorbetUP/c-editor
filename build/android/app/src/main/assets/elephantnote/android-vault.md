# Android local vault

folder: Guides
#android #vault #sync

ElephantNote Android now stores notes as Markdown files under the app-specific `ElephantVault` directory instead of keeping user notes inside a single preferences JSON value.

## Layout

```text
ElephantVault/
  Notes/
    Mobile Inbox/
      Example.md
  Attachments/
    attachment-....jpg
  Canvas/
  .elephantnote/
    mobile-vault.json
```

## Why this matters

- Notes are real `.md` files.
- Folders are represented by real directories under `Notes`.
- Shared images are copied into `Attachments`.
- The desktop app can later sync or import the same Markdown content.
- AI stays desktop-side for now; generated summaries, tags or OCR text can come back as Markdown.

## Practical workflow

1. Capture notes on Android.
2. Add `folder: Research` or another folder line when useful.
3. Add `#tags` and `[[links]]` for graph/wiki navigation.
4. Export or sync the vault content to desktop.
5. Run heavy AI/OCR locally on desktop.
6. Sync the resulting Markdown back to the phone.
