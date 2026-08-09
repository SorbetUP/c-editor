# Addon source ownership

The Elephant application repository no longer tracks physical addon implementations or protected addon packs.

Canonical source: `https://github.com/SorbetUP/Elephant-Addons`

Pinned integration revision: `8458c1cc9ed074148697e6edc5ffc0f05bbf05ab`

`build/scripts/sync-elephant-addons.mjs` materializes the pinned repository into the ignored `.cache/elephant-addons` directory and exposes ignored `addons` / `packs` compatibility links for existing build and validation tooling. Runtime package downloads and the official catalogue use the dedicated repository directly.

Elephant retains only the generic addon host, installer, permission broker, scoped APIs, service/sidecar host and UI extension points.
