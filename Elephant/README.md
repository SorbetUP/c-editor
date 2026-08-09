# Elephant app layout

`Elephant` contains the actual application source. The active desktop runtime is
Tauri, with frontend and backend code split by responsibility.

## Folders

- `frontend/src`: current Vue/Muya renderer source and runtime bridges.
- `frontend/app`: ElephantNote UI modules kept as compatibility layers.
- `backend/tauri`: Rust/Tauri backend, capabilities, Cargo manifest, and app
  configuration.
- `backend/js`: compatibility JavaScript backend modules retained for parity
  and migration.
- `shared`: domain helpers shared by frontend, backend, and tests.
- `assets/static`: app icons and static assets.

## Aliases

- `elephant-back/*` -> `Elephant/backend/js/*`
- `elephant-front/*` -> `Elephant/frontend/app/*`
- `elephant-shared/*` -> `Elephant/shared/*`

Compatibility aliases are still kept for existing imports:

- `@/elephantnote/*`
- `common/elephantnote/*`
- `main_renderer/elephantnote/*`
