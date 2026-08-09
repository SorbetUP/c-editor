# Sync, Web, and Mobile

ElephantNote sync uses the same compact contract on every target:

- a stable `deviceId`
- a `folderId` for the vault
- Syncthing as the LAN/device replication layer
- a Git repository as local ordered history, not as the default network transport
- a small default operation queue: `init`, `snapshot`

This follows the Syncthing shape of devices plus folders while keeping the
storage backend inspectable with ordinary Git commands. Git remotes can still be
added later as an advanced transport, but the product flow does not push to
GitHub by default.

## Desktop

The Electron app calls the `sync` API domain. A sync run initializes the active
vault as a Git repository when needed, writes `.elephantnote/sync-config.json`,
configures the Syncthing folder and optional peer device/address, then commits
local changes as an ordered snapshot.

## Web Docker

Build and run:

```bash
pnpm web:docker:build
pnpm web:docker:run
```

The container listens on `http://localhost:8787` and stores the vault in the
`/data/vault` volume. The web server exposes:

- `GET /api/notes`
- `POST /api/notes`
- `GET /api/sync/status`
- `POST /api/sync/run`

To use a host folder instead of the named Docker volume:

```bash
docker run --rm -p 8787:8787 -v "$PWD/vault:/data/vault" elephantnote-web
```

## Android

The Android companion remains offline-first and stores quick captures locally.
It now creates a stable sync identity compatible with the shared sync status.

```bash
./gradlew -p android assembleDebug
```

## iOS

The iOS scaffold is a Swift Package with a SwiftUI note capture view, local
`UserDefaults` storage, and the same compact sync identity model.

```bash
swift test --package-path ios
```

Creating a signed iOS app archive still requires an Xcode app target and Apple
provisioning, which are intentionally not stored in this repository.
