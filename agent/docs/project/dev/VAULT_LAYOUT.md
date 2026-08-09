# Vault layout contract

A vault is split into two zones.

## Hidden application zone

All application-managed data lives under one hidden folder:

```text
.vault-root/
  .elephantnote/
    config/
    wiki/
    models/
    sync/
    index/
    cache/
    state/
    trash/
    assets/
```

This zone is owned by ElephantNote. It is not displayed as normal user content.

### Hidden folders

- `.elephantnote/config/`: vault settings, workspace metadata, calendar and sources metadata.
- `.elephantnote/wiki/`: generated wiki records and wiki metadata.
- `.elephantnote/models/`: model/provider selection and model metadata.
- `.elephantnote/sync/`: sync queue, sync state and conflict metadata.
- `.elephantnote/index/`: search indexes and derived indexes.
- `.elephantnote/cache/`: disposable cache.
- `.elephantnote/state/`: local runtime state.
- `.elephantnote/trash/`: future safe-delete area.
- `.elephantnote/assets/`: app-managed attachments and embedded assets.

## Visible user zone

Everything outside `.elephantnote/` is visible user content:

```text
.vault-root/
  Projects/
  Notes/
  Drawings/
  Getting Started/
  any-user-folder/
```

The UI must list these folders and files normally.

## Rule

Do not create visible root folders for internal features such as wiki, models, sync, config, index, cache, state or trash.

Feature storage must use `src-tauri/src/vault_layout.rs` instead of hard-coded paths.
