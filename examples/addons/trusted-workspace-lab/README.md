# Trusted Workspace Lab

Reference addon for ElephantNote's **Full app access** runtime.

It deliberately demonstrates capabilities that are impossible in the isolated Worker runtime:

- inject global CSS into ElephantNote;
- modify the root DOM directly;
- register workspace, settings, editor and layout contributions;
- access the Vue application, router, Pinia and application services through the trusted API;
- receive automatic cleanup when the addon is disabled.

## Install and enable

Trusted Workspace Lab is embedded in ElephantNote's official catalogue. In **Settings → Addons**:

1. turn on Community Addons once;
2. install Trusted Workspace Lab from the catalogue;
3. use its ordinary addon switch.

There is no separate full-access review panel or second confirmation flow. The access badge and description make clear that this package runs inside ElephantNote and is not sandboxed.

For local development, build the same installable package with:

```bash
node build/scripts/package-addon.mjs \
  examples/addons/trusted-workspace-lab \
  build/addons/trusted-workspace-lab.enaddon
```

## Contract

The entry file is a single ESM module exporting either:

```js
export default {
  async onload(api) {},
  async onunload(api) {}
}
```

or a class with the same lifecycle methods.

Full app access addons are expected to use the registration helpers whenever possible so ElephantNote can automatically remove commands, views, listeners and styles during deactivation. The `experimental` namespace exposes unstable internals for deeper integrations and can break between ElephantNote releases.
