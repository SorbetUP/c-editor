# Finance Notes example addon

This example demonstrates the first external ElephantNote addon contract:

1. A command is registered in an isolated JavaScript Worker.
2. The command requests current data through the Rust HTTP broker.
3. The broker allows only the host declared in `manifest.json`.
4. The addon transforms the JSON response inside its Worker.
5. The generated Markdown is written only under the declared `Finance/**` scope.
6. Execution metadata is stored in the addon's private storage namespace.

## Build the package

Create a ZIP archive whose root directly contains `manifest.json` and `main.js`, then rename it with the `.enaddon` extension.

```bash
cd examples/addons/finance-notes
zip -r finance-notes.enaddon manifest.json main.js
```

Install it from **Settings → Addons → Install from file**. External addons are installed disabled. Review the displayed capabilities, enable the addon, then run **Refresh finance note** from the Addons page.

The example defaults to `AAPL`. Programmatic command callers can provide `{ "symbol": "MSFT" }` as the payload.
