# ElephantNote Addon SDK v1

ElephantNote external addons are user-installable `.enaddon` packages. API v1 exposes a capability-based Worker surface for isolated addons and an explicit full-application runtime for addons that need to modify ElephantNote itself.

## Addon-first application model

ElephantNote keeps the vault, editor, navigation shell and addon host in core. Optional application features are independently installable, removable, enabled and disabled addons.

The first-party catalogue contains:

- **Addon Packs** manages portable configurations and is the only required built-in addon;
- **Google Keep Import** owns Google Keep, web-page and RSS import controls;
- **Codex Connection** owns ChatGPT subscription connection, account state, usage limits and the Codex provider contribution, and uses the monochrome OpenAI mark in the addon catalogue;
- **Calendar** owns the native ElephantNote calendar workspace;
- **Sites** owns static-site generation and preview controls;
- **AI** owns providers, chat, semantic search, OCR and the local model library;
- **Sync** owns encrypted Iroh pairing, synchronization and conflict recovery.

Daily Notes, Quick Capture, Vault Overview and Addon Inspector were removed from the shipped application. They were narrow examples or duplicate workflows rather than complete product features.

Installation and enabled state are persisted separately. Removing an addon removes its Settings pages, navigation entries, workspaces and runtime behavior. Reinstalling it does not silently enable it. Built-in addons do not require Community Addons mode; third-party addons require the single global Community Addons acknowledgement.

Optional runtimes are lazy. In particular, ElephantNote does not start an Iroh endpoint at application startup. Enabling Sync activates the guarded client; disabling or removing Sync closes the endpoint and blocks later Sync calls. Disabling AI closes its chat sidebar and removes its Settings and model-library views. Disabling Sites stops the preview and clears its feature state.

## Addon packs

Addon Packs uses the portable `elephantnote-addon-pack` format. Pack files are stored under:

```text
.elephantnote/addons/packs/
```

ElephantNote creates a protected first-party parity pack at:

```text
.elephantnote/addons/packs/develop-parity.enaddonpack
```

Applying that pack installs and enables every useful first-party addon so the modular build reproduces the complete `develop` application. User-created packs capture the currently installed addons, versions and enabled states.

Example:

```json
{
  "format": "elephantnote-addon-pack",
  "version": 1,
  "name": "Writing workspace",
  "description": "Import, AI and publishing tools.",
  "addons": [
    { "id": "elephant.google-keep-import", "source": "builtin", "enabled": true },
    { "id": "elephant.ai", "source": "builtin", "enabled": true },
    { "id": "elephant.sites", "source": "builtin", "enabled": true },
    { "id": "com.example.publisher", "source": "catalog", "version": "1.2.0", "enabled": true }
  ]
}
```

Supported sources:

- `builtin`: a first-party feature shipped in the built-in catalogue and installed on demand;
- `catalog`: an addon installed or updated from the fixed official catalogue;
- `installed`: a local package that must already exist in the vault.

Applying a pack may install missing built-in and official catalogue addons. It never bypasses the global Community Addons switch. Full-app-access packages use the same ordinary addon switch and pack application path as other community addons.

## Official catalogue

The application loads its official catalogue from the dedicated `addon-catalog` branch. The branch contains only catalogue metadata and addon sources.

Current catalogue examples include:

- Task Dashboard;
- Broken Links Auditor;
- GitHub Release Watcher;
- Notion Markdown Importer;
- Inbox Digest;
- Finance Notes;
- Addon Platform Proof.

The Rust backend:

1. fetches the fixed catalogue URL;
2. rejects unsafe or cross-addon paths;
3. verifies catalogue metadata against the addon manifest;
4. downloads the declared manifest and Worker entry;
5. creates a temporary `.enaddon` archive;
6. passes it through the same package validator used for local files;
7. installs the addon disabled in the active vault.

The application does not accept arbitrary catalogue URLs. See `CATALOG.md`.

## Per-vault storage

Every initialized vault contains:

```text
.elephantnote/addons/
├── registry.json
├── packages/
│   └── <addon-id>/
├── packs/
│   ├── develop-parity.enaddonpack
│   └── <user-pack>.enaddonpack
└── data/
    └── <addon-id>/storage.json
```

Community package files, enable state, packs and private addon storage are scoped to the active vault. First-party installation and enabled state are stored by the desktop application. `.elephantnote` remains hidden from the normal explorer.

## Package format

An `.enaddon` package is a ZIP archive with `manifest.json` at its root.

```text
example.enaddon
├── manifest.json
├── main.js
└── assets/
    └── icon.svg
```

Limits:

- compressed package: 25 MiB;
- extracted package: 100 MiB;
- archive entries: 512;
- JavaScript entry: 5 MiB;
- symbolic links and archive traversal are rejected.

External addons are installed disabled.

## Community Addons mode

Community Addons mode is disabled by default and requires one explicit risk acknowledgement. Turning it off stops every external addon, persists every installed third-party package as disabled and prevents those packages from restarting, without uninstalling packages or deleting private data.

After Community Addons is enabled, every package uses its ordinary enable switch. There is no second visible review flow or permanent full-app-access safe-mode setting.

Worker isolation and permission checks reduce risk but do not prove that third-party code is safe. Full-app-access addons intentionally run inside ElephantNote and can change the DOM, editor and application behavior.

## Access models

### Limited access

The entry runs in a dedicated Web Worker and receives only the public capability API. It cannot directly access Vue, Pinia, the DOM, Muya, Tauri or browser networking.

### Full app access

A package opts in with:

```json
{
  "contributes": {
    "runtimeMode": "trusted"
  }
}
```

It is loaded in the ElephantNote renderer and can use the full addon host, including application resources, DOM integration, routes, editor and Markdown contributions, settings pages and reversible patches. This mode is deliberately comparable to installing a powerful desktop plugin: the user must trust the package.

## Manifest

```json
{
  "id": "com.example.hello",
  "name": "Hello Notes",
  "version": "1.0.0",
  "apiVersion": 1,
  "minAppVersion": "0.1.0",
  "runtime": {
    "type": "javascript-worker",
    "entry": "main.js"
  },
  "permissions": {
    "commands": true,
    "storage": false,
    "notes": {
      "read": ["Inbox/**"],
      "write": ["Generated/**"]
    },
    "network": {
      "hosts": []
    }
  },
  "contributes": {
    "commands": [
      {
        "id": "com.example.hello.generate",
        "title": "Generate hello note"
      }
    ]
  }
}
```

Addon identifiers use lowercase ASCII letters, numbers, dots, dashes or underscores. Command identifiers must start with the addon identifier followed by a dot.

## Isolated Worker entry contract

The entry assigns `self.elephantAddon`.

```js
self.elephantAddon = {
  activate(api) {
    return api.commands.register({
      id: 'com.example.hello.generate',
      title: 'Generate hello note',
      async run() {
        await api.notes.write('Generated/Hello.md', '# Hello\n')
        return { path: 'Generated/Hello.md' }
      }
    })
  }
}
```

`activate()` may return a cleanup function. Contributions are removed automatically when an addon is disabled.

## API v1

### Application metadata

```js
const info = await api.app.info()
```

Returns the ElephantNote version and addon API version.

### Commands

Requires `permissions.commands: true`.

```js
const unregister = api.commands.register({
  id: 'com.example.addon.refresh',
  title: 'Refresh generated report',
  async run(payload) {
    return { ok: true, payload }
  }
})
```

The Addons settings page runs commands without a payload. Other internal invocation surfaces may provide one.

### Notes

All paths are relative to the active vault and checked in Rust against the manifest scopes.

```js
const inboxEntries = await api.notes.list('Inbox')
const allVisibleNotes = await api.notes.list('.')
const note = await api.notes.read(inboxEntries[0].path)
await api.notes.write('Generated/Report.md', '# Report\n')
```

`notes.list(prefix)`:

- requires a matching read scope;
- accepts `.` as the vault-root sentinel only when the manifest declares `read: ["*"]`;
- returns only Markdown files;
- does not follow symbolic links;
- excludes hidden files and directories, including `.elephantnote`;
- is limited to 1,000 notes and 64 directory levels;
- returns `path`, `size` and `modifiedAt` metadata.

Scopes can target a file, a directory tree, or the whole visible vault:

```json
{
  "read": ["Reference/README.md", "Inbox/**"],
  "write": ["Generated/**"]
}
```
