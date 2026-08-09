# ElephantNote addons

Initial skeleton for Obsidian-like addons.

## Current scope

This implementation intentionally starts with a safe renderer-side runtime:

- addon manifests are normalized and version-gated;
- builtin addons can be registered at startup;
- addons expose `activate(ctx)` and optional `deactivate(ctx)` lifecycle hooks;
- addon cleanup uses disposables;
- contribution areas are tracked centrally;
- a Pinia mirror makes addon state observable from Vue components;
- `/preference/addons` shows registered addons and lets the user enable or disable them.

External package loading is deliberately not implemented yet. It should only be added after a sandbox, permission prompts, signature/hash checks and filesystem boundaries exist.

## Manifest shape

```js
export default {
  manifest: {
    id: 'author.addon-name',
    name: 'Addon Name',
    version: '0.1.0',
    description: 'What this addon does.',
    apiVersion: 1,
    permissions: [],
    defaultEnabled: false,
    contributes: {}
  },

  activate(ctx) {
    ctx.addAction({ id: 'author.addon-name.action', title: 'Run action' })
  },

  deactivate(ctx) {
    // Optional explicit teardown. Registered disposables are still cleaned automatically.
  }
}
```

## Contribution areas

The first supported extension points are:

- `actions`
- `sidebar.items`
- `settings.sections`
- `views`
- `editor.extensions`
- `statusbar.items`

## Runtime access

The manager is available through:

- Vue injection: `useAddonManager()`
- Vue global property: `this.$addons`
- development debug global: `window.__ELEPHANT_ADDONS__`
- Pinia store mirror: `useAddonsStore()`
