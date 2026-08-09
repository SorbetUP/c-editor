# Addon API v1: additive expansion

Elephant keeps every existing Addon API v1 entry point. No legacy helper is removed, renamed or deprecated by this change.

The expansion covers both addon runtime models:

- built-in and full-app-access addons receive a structured host API through `context.api`;
- isolated Worker addons keep their existing capability API and receive additional helpers in the same namespaces.

Addons can migrate one call at a time. There is no forced manifest migration and `apiVersion` remains `1`.

## Compatibility contract

1. Existing flat host helpers such as `addAction`, `addView`, `addEditorExtension`, `registerContribution` and `addDisposable` keep their behavior.
2. Existing isolated Worker methods such as `api.app.info`, `api.notes.list/read/write`, `api.http.request`, `api.storage.get/set/remove/entries`, `api.commands.register` and `api.views.register` keep their names.
3. New APIs are additive and grouped by domain.
4. The host API validates known extension-point names while the historical low-level registration helper remains available.
5. Note reads and writes preserve the historical `content` and `ok` fields while also exposing the richer Rust metadata.
6. Disabling, uninstalling or failing an addon removes registered contributions, resources, subscriptions and timers.

# Host API

The activation context used by built-in addons and full-app-access addon adapters now includes `context.api`.

```js
export default {
  activate(context) {
    const { api } = context
    api.log.info('Starting', api.manifest.id)
  }
}
```

A full-app-access package can retrieve the same host API through its existing resource API:

```js
const hostApi = api.resources.get(`addon.api.${api.manifest.id}`)
```

Full-app-access packages should continue using their existing package `api.storage` for package persistence. The host resource mainly provides contributions, lifecycle, resources, hooks and host orchestration.

## Identity and capability discovery

```js
const commandId = api.ids.qualify('refresh')
// com.example.publisher.refresh

api.ids.owns(commandId) // true
api.capabilities.supports('editor.footer-items')
api.capabilities.extensionPoints
```

`qualify()` leaves an already-qualified identifier unchanged.

## Contribution registration

```js
const dispose = api.contributions.registerMany({
  actions: {
    id: api.ids.qualify('publish'),
    title: 'Publish',
    async run() {}
  },
  'sidebar.items': {
    id: api.ids.qualify('navigation'),
    title: 'Publisher',
    actionId: api.ids.qualify('publish')
  },
  'statusbar.items': [
    { id: api.ids.qualify('status'), title: 'Ready' }
  ]
})
```

Batch registration validates the complete batch before registration and rolls back a partially registered batch if registration fails.

Introspection is scoped to the current addon:

```js
api.contributions.list()
api.contributions.list('sidebar.items')
api.contributions.get('sidebar.items', api.ids.qualify('navigation'))
api.contributions.has('sidebar.items', api.ids.qualify('navigation'))
```

## Commands

```js
api.commands.register(command)
api.commands.registerMany([commandA, commandB])
api.commands.get(commandId)
await api.commands.execute(commandId, payload)
```

`get()` and `execute()` resolve only commands owned by the current addon.

## Workspace and Settings

```js
api.workspace.registerView(view)
api.workspace.registerPanel(panel)
api.workspace.registerSidebarItem(item)
api.workspace.registerStatusBarItem(item)
api.workspace.openView(view.id, { selectedId: '42' })

api.settings.registerSection(section)
api.settings.registerPage(page)
api.settings.open('addons')
```

## Editor, Markdown and layout

```js
api.editor.registerExtension(extension)
api.editor.registerBlockType(blockType)
api.editor.registerInlineType(inlineType)
api.editor.registerInputRule(inputRule)
api.editor.registerToolbarItem(toolbarItem)
api.editor.registerFooterItem(footerItem)
api.editor.registerPasteHandler(pasteHandler)

api.markdown.registerPostProcessor(processor)
api.markdown.registerCodeBlockProcessor(processor)
api.markdown.registerEmbedRenderer(renderer)

api.layout.registerItem(item)
api.layout.registerZone(zone)
```

Domain-specific registrations are grouped as well:

```js
api.ai.registerProvider(provider)
api.imports.register(importer)
api.sites.registerGenerator(generator)
```

## Scoped storage

The historical `context.storage` remains available. The structured namespace adds convenience helpers without changing the storage format.

```js
await api.storage.set('options', { compact: true })
const options = await api.storage.get('options', {})
const exists = await api.storage.has('options')
const keys = await api.storage.keys()
await api.storage.update('launchCount', count => (count || 0) + 1, 0)
await api.storage.remove('options')
await api.storage.clear()
```

`update()` is a read-modify-write convenience operation, not a cross-process transaction.

## Resources and hooks

```js
api.resources.get('editor.runtime')
api.resources.has('editor.runtime')
api.resources.list()
api.resources.provide('publisher.session', session)
api.resources.watch('editor.runtime', event => {})

api.hooks.register('publisher.beforePublish', payload => payload)
const result = await api.hooks.run('publisher.beforePublish', input)
```

Provided resources, watchers and hooks are removed automatically when the addon stops.

## Events, scheduler and lifecycle

```js
api.events.on('refreshed', payload => {})
api.events.once('ready', initializeOnce)
api.events.emit('refreshed', { count: 12 })

api.scheduler.timeout(refresh, 250)
api.scheduler.interval(poll, 30_000)

api.lifecycle.signal.addEventListener('abort', stopWork)
api.lifecycle.onAbort(stopWork)
api.lifecycle.addDisposable(() => closeSession())
api.lifecycle.addDisposable({ dispose: () => closeSession() })
api.lifecycle.addDisposable(subscription) // unsubscribe()
api.lifecycle.addDisposable(controller)   // abort()
```

Timers and structured disposables are cleaned automatically. Promise-returning cleanup functions are observed and rejected cleanup is logged instead of becoming an unhandled rejection.

## Logging

```js
api.log.debug('details')
api.log.info('started')
api.log.warn('degraded mode')
api.log.error('failed', error)
```

Messages are prefixed with the addon identifier through Elephant's logger.

# Isolated Worker API

The isolated runtime remains a dedicated Web Worker with direct browser networking, Tauri, DOM, Vue and Pinia access disabled. Every privileged operation still crosses the capability broker and is checked against the manifest.

## Capabilities and identifiers

```js
const capabilities = api.app.capabilities()
api.capabilities === capabilities

capabilities.runtime // isolated-worker
capabilities.apiVersion // 1
capabilities.permissions
capabilities.contributes

const id = api.ids.qualify('refresh')
api.ids.owns(id)
```

The capability object is frozen and derived from the installed manifest.

## Notes

The original methods remain available:

```js
const entries = await api.notes.list('Inbox')
const document = await api.notes.read(entries[0].path)
await api.notes.write('Generated/Report.md', '# Report')
```

The richer read result contains `path`, `size`, `modifiedAt`, `markdown` and the legacy `content` alias. Writes return `ok`, `path`, `size`, `modifiedAt` and `created`.

Writes overwrite by default, preserving previous Worker behavior. Creation-only behavior can be requested explicitly:

```js
await api.notes.write('Generated/New.md', '# New', { overwrite: false })
```

A read-modify-write helper is available:

```js
const result = await api.notes.update('Inbox/Status.md', (markdown, document) => {
  return `${markdown}\n\nUpdated from ${document.path}`
})
```

Note access is routed through the dedicated Rust note commands. Paths remain relative, permission-scoped, hidden-directory protected, symlink checked and size limited.

## HTTP

```js
await api.http.request({ url, method: 'PATCH', body })
await api.http.get(url, { headers })
await api.http.post(url, { status: 'ready' })
const response = await api.http.requestJson({ url })
console.log(response.json)
```

All methods use the same hardened HTTPS broker, host allowlist, public-address validation, redirect policy and response limits.

## Storage

The original storage methods remain available and the following helpers are additive:

```js
await api.storage.has('options')
await api.storage.keys()
await api.storage.update('counter', value => (value || 0) + 1, 0)
await api.storage.clear()
```

## Commands and views

```js
const disposeCommands = api.commands.registerMany([commandA, commandB])
const disposeViews = api.views.registerMany([viewA, viewB])
```

The original single-item `register()` methods remain available. Dynamic command and view disposers now notify the host immediately, and all remaining contributions are removed when the Worker stops.

## Worker-local events, scheduling and lifecycle

```js
api.events.on('refresh', refresh)
api.events.once('ready', initialize)
api.events.emit('refresh', payload)

api.scheduler.timeout(refresh, 250)
api.scheduler.interval(poll, 30_000)

api.lifecycle.onAbort(stopWork)
api.lifecycle.addDisposable(subscription)
```

These APIs are local to the Worker and do not create a cross-addon event bus. The lifecycle signal is aborted before addon deactivation and every registered disposable is cleaned in reverse order.

## Worker logging

```js
api.log.debug('details')
api.log.info('started')
api.log.warn('degraded')
api.log.error(error)
```

Values are converted to structured-clone-safe payloads and forwarded to the Elephant logger with the addon identifier.

# Legacy equivalents

| Existing host helper | Structured host equivalent |
| --- | --- |
| `addAction()` | `api.commands.register()` |
| `addView()` | `api.workspace.registerView()` |
| `addSidebarItem()` | `api.workspace.registerSidebarItem()` |
| `addSettingsSection()` | `api.settings.registerSection()` |
| `addEditorExtension()` | `api.editor.registerExtension()` |
| `addStatusBarItem()` | `api.workspace.registerStatusBarItem()` |
| `registerContribution()` | `api.contributions.register()` |
| `addDisposable()` | `api.lifecycle.addDisposable()` |

Additional flat aliases are available for extension points that already existed but lacked activation-context helpers: Settings pages, workspace panels, editor block/inline/input/toolbar/footer/paste extensions, Markdown processors, layout items/zones, AI providers, importers and site generators.
