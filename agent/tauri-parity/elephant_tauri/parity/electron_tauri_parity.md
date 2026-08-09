# Electron Tauri parity checklist

## Startup

- app mounts
- no blank page
- renderer logs appear in devtools
- renderer logs appear in terminal
- preferences restore
- buffered state restore

## Vaults

- choose vault
- restore vault after restart
- list folders
- list notes
- hide app folders
- open note
- close note
- recently edited list

## Library cards

- title uses note title or filename
- preview uses visible body text
- metadata header is hidden
- missing date is safe
- invalid date is safe
- grid and list show the same notes
- sort modes are stable

## Editor

- note content loads
- cursor value is always an object
- typing updates markdown
- title edit keeps body
- tag edit keeps body
- save updates saved state
- source mode and editor mode match

## Muya runtime

- disabled mode keeps legacy editor
- shadow mode keeps legacy editor visible
- active mode can be enabled manually
- empty notes are writable
- typed text is preserved
- paste is sanitized
- input rules do not erase text

## Models

- local list loads
- remote search loads or shows an error
- download starts with a resolved file
- progress appears
- cancel works
- remove works
- chat role persists
- embedding role persists
- OCR role persists

## Views

- search opens
- graph opens
- wiki opens
- calendar opens
- dashboard opens

## Error handling

- Vue errors show overlay
- promise errors show overlay
- terminal receives renderer diagnostics
