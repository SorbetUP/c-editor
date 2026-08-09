# Full app access addons

ElephantNote supports three addon access models.

## Limited access

Limited access addons run in a dedicated Worker. They receive a small capability API and cannot directly access Vue, Pinia, Muya, the DOM, Tauri or unrestricted browser networking.

Use this mode for reports, task engines, templates, importers and API clients that fit the brokered API.

## Full app access

Full app access addons run as JavaScript modules inside ElephantNote's renderer. They can modify the application, editor, workspace, layout, settings, services and DOM.

A package requests this mode with:

```json
{
  "runtime": {
    "type": "javascript-worker",
    "entry": "main.js"
  },
  "contributes": {
    "runtimeMode": "trusted"
  }
}
```

The Worker-compatible runtime declaration keeps one `.enaddon` package format and one Rust installation validator. `runtimeMode: "trusted"` changes how the installed entry is executed.

Full app access is intentionally not a sandbox. The permissions shown for such an addon are descriptive rather than a strict security boundary.

## One consent flow

The user accepts the general third-party-code risk once when enabling Community Addons.

After that, the ordinary switch is the only activation control for every addon. Enabling a full app access addon starts that package directly; there is no second review dialog, extra checkbox, grant button or visible safe-mode setting.

Internally, activation is associated with the exact installed package hash and interrupted startup is recovered automatically. These mechanisms do not add another user-facing consent step. Updating or rebuilding the package changes its hash, and explicitly enabling the updated package is the approval action.

## Built in by ElephantNote

System addons ship with the application, are tested with it and remain independently enabled or disabled.

The first extracted system features are listed first in Settings:

1. Addon Packs;
2. Google Keep Import and web/RSS import;
3. Codex Connection;
4. Calendar;
5. Sites.

Google Keep Import, Codex Connection, Calendar and Sites are disabled by default. Their navigation and Settings surfaces exist only while the corresponding addon is enabled. Their existing backend services remain host capabilities so enabling the addon restores the same functionality instead of a reduced reimplementation.

## Application host registry

The host registry is the escape hatch for changing real application behavior without guessing globals:

```js
const services = api.resources.get('services')
const editor = api.resources.get('editor')

const disposeResource = api.resources.provide('myFeature', feature)
const disposeWatch = api.resources.watch('editor', ({ value }) => {
  console.log('active editor changed', value)
})
```

ElephantNote publishes resources including `window`, `document`, `tauri`, `marktext`, `elephantnote`, `fileUtils`, `router`, `pinia`, `services`, `vueApp`, `runtime`, `addons` and `addonManager`. Runtime components can publish further resources later.

Resources and watchers registered through the API are removed automatically when the addon is disabled.

## Reversible patches

Trusted addons can replace or wrap real application behavior:

```js
const service = api.resources.get('services').example

api.patch.method(service, 'save', async (original, payload) => {
  const transformed = await api.patch.runHook('before-save', payload)
  return await original(transformed)
})

api.patch.property(service, 'mode', 'custom')
api.patch.hook('before-save', (payload) => ({ ...payload, taggedByAddon: true }))
```

Every helper records the previous method or property descriptor and restores it on unload.

## Vue and router mutation

```js
api.vue.component('MyAddonPanel', MyAddonPanel)
api.vue.directive('my-addon', directive)
api.vue.provide('my-addon-service', service)

api.router.addRoute({ path: '/my-addon', component: MyAddonPage })
api.router.beforeEach((to) => {
  // inspect or redirect navigation
})
```

Components, directives, providers, routes and guards are cleaned up automatically where Vue and Vue Router expose removal APIs.

## Settings extensions

Settings contributions are rendered by the application rather than stored as unused metadata.

```js
api.settings.registerSection({
  id: 'com.example.deep-plugin.settings',
  title: 'Deep Plugin',
  description: 'Plugin-owned controls.',
  section: 'editor',
  fields: [
    { id: 'enabled', label: 'Enabled', type: 'boolean', value: true }
  ]
})
```

A contribution may alternatively define `render(container, context)` for arbitrary controls. It is mounted only in the requested category and removed on category change or addon disable.

Metadata-only contributions are ignored. An addon therefore cannot create an empty title-and-description block without providing controls or a renderer.

System addons may create standalone Settings categories with `standalone`, `navigationLabel`, `navigationIcon` and `order`.

## Stable API

The trusted API exposes registration helpers for:

- commands;
- workspace views and sidebar entries;
- targeted and standalone Settings sections;
- editor extensions, block types, inline types, input rules, toolbar items and paste handlers;
- Markdown post-processors, code block processors and embed renderers;
- layout items and zones;
- status bar items;
- styles, events, mutation observers and DOM mounts;
- Vue components, directives and providers;
- routes and navigation guards;
- resources, hooks and reversible patches.

## Experimental API

`api.experimental` exposes raw renderer internals:

```js
api.experimental.window
api.experimental.document
api.experimental.tauri
api.experimental.router
api.experimental.pinia
api.experimental.services
api.experimental.vueApp
api.experimental.host
api.experimental.rawContext
```

This namespace allows addons to change behavior beyond the stable API. It may change between ElephantNote releases. Direct mutations made through it remain the addon author's cleanup responsibility.

## Trusted Workspace Lab

Trusted Workspace Lab is embedded in the application catalogue. Open **Settings → Addons**, refresh the catalogue, install it, then enable it with its ordinary switch. No manually downloaded `.enaddon` is required.

It exercises:

- installation through the real Rust validator;
- full application access;
- global style injection;
- a command and sidebar entry;
- a visible addon-owned Settings control;
- editor and layout contributions;
- host resource publication;
- complete cleanup on disable.

## Security expectations

Only enable a full app access addon when you trust its publisher, source, dependencies and installed package. Publisher signatures remain a future hardening increment.
