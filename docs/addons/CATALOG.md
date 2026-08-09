# Official Addon Catalogue

ElephantNote reads the official catalogue from the dedicated `addon-catalog` branch of this repository.

The branch snapshot contains only:

```text
catalog.json
ADDON_CATALOG.md
addons/
  <slug>/
    manifest.json
    main.js
    README.md
```

## Current official addons

- **Task Dashboard** — aggregates Markdown tasks across the visible vault;
- **Broken Links Auditor** — reports unresolved and ambiguous wikilinks;
- **GitHub Release Watcher** — tracks releases for configured public repositories;
- **Notion Markdown Importer** — imports local Notion Markdown exports and rewrites internal links;
- **Inbox Digest** — creates a review report from `Inbox/`;
- **Finance Notes** — creates a multi-asset market dashboard with cached fallback data;
- **Addon Platform Proof** — exercises the public Worker API and persistent storage.

## Runtime flow

1. Settings asks Rust for `tauri_addons_catalog_list`.
2. Rust downloads the fixed `catalog.json` URL from `raw.githubusercontent.com`.
3. Catalogue paths are rejected if they are absolute, contain traversal, or leave `addons/<slug>/`.
4. Installing an entry downloads its manifest and JavaScript entry from the same fixed branch.
5. Rust creates a temporary `.enaddon` ZIP locally.
6. The archive is passed through the normal `tauri_addons_install` validator and per-vault registry.
7. The package is registered disabled; the user still reviews permissions and enables it explicitly.

The application does not accept an arbitrary catalogue URL. Changing the official source requires a code change and review.

## Publishing an addon

Add the source under `addons/<slug>/`, then add one entry to `catalog.json` with matching `id`, `name` and `version` values. The catalogue and manifest values must match or installation fails.

Updating an addon requires incrementing the version in both files. ElephantNote displays **Update** when the listed version differs from the installed version.

The dedicated validation workflow checks the actual catalogue branch, executes every official addon pipeline against deterministic broker fixtures, packages every addon and validates each resulting ZIP archive.
