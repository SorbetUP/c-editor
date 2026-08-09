# ElephantNote Iroh vault synchronization

## Scope

The Iroh backend synchronizes vault content between explicitly paired ElephantNote devices. It includes normal notes, folders, `.assets`, and other non-device-specific content.

Configuration is intentionally local to each device. A phone and a desktop can therefore use different providers, models, layouts, runtime settings, and local UI state without overwriting each other.

The following local/runtime paths are intentionally excluded:

- `.config/` — provider and runtime configuration
- `.conflit/` — temporary local conflict archive
- `.elephantnote/config/` — workspace, vault, calendar, sources, and other application configuration
- `.elephantnote/models/` — local model/provider selection and locally available runtime information
- `.elephantnote/state/` — device-specific application and UI state
- `.elephantnote/sync/` — peer identities, baselines, queues, and transfer state
- `.elephantnote/cache/` — rebuildable cache
- `.elephantnote/index/` — rebuildable search index
- `.git/`, `node_modules/`, OS metadata, editor temporary files, and symlinks

These exclusions happen when the manifest is built. Consequently, excluded files cannot be uploaded, downloaded, deleted remotely, or turned into ordinary synchronization changes. An older baseline that still mentions a configuration file also cannot cause that local configuration file to be deleted.

## Transport and identity

- Iroh endpoint identity is a persistent Ed25519 secret stored in the Tauri application config directory, outside every vault.
- The endpoint uses QUIC with the ALPN `elephantnote/vault-sync/1`.
- LAN address discovery uses `iroh-mdns-address-lookup` with the service name `elephantnote-v1`.
- The `Minimal` Iroh preset is used: no public relay or cloud service is required.
- Every accepted connection is authenticated by its Iroh `EndpointId`.

## Pairing and application integration

The Sync settings panel calls the real Tauri commands directly. It does not expose the previous rclone/shared-folder controls.

1. Device A creates a ten-minute invitation from Settings → Sync.
2. The invitation contains Device A's `EndpointAddr`, vault identifier, invitation identifier, and a random one-time token.
3. Only the BLAKE3 hash of the one-time token is persisted by Device A.
4. Device B pastes the invitation from its own Sync settings panel.
5. Device B connects over Iroh and sends the token on the encrypted connection.
6. Both devices persist the authenticated remote `EndpointId` as a trusted peer for that vault.
7. Either device can run **Sync now**.

The invitation is a temporary bearer credential and must not be logged or committed.

## Synchronization algorithm

Each peer pair keeps its own last common manifest under:

```text
.elephantnote/sync/baselines/<endpoint-id>.json
```

A synchronization compares:

- current local content manifest;
- current remote content manifest;
- last common content manifest.

This three-way comparison handles:

- new files and folders;
- modifications;
- deletions;
- renames as delete plus create;
- concurrent modifications;
- edit-versus-delete conflicts.

## Conflict policy

When two different file versions were both changed since the common baseline:

1. both versions are preserved;
2. the version with the largest filesystem modification timestamp keeps the original vault path;
3. if both timestamps are equal, EndpointId ordering provides a deterministic tie-breaker;
4. the older/losing version is copied to `.conflit/`, preserving the original directory hierarchy;
5. the conflict copy is transferred to the other peer so both devices initially receive it.

Example:

```text
Notes/Project.md
.conflit/Notes/Project.<device>-conflict-<hash>-<mtime>.md
```

The `.conflit/` directory is not part of normal content manifests. This is deliberate: each device may use a different retention duration, and deleting an expired local archive must never propagate as a remote note deletion.

The default retention is **3 days**. It can be configured from Settings → Sync between 1 and 365 days. The setting is stored locally under `.elephantnote/sync/conflict-settings.json`, which is itself excluded from synchronization. Cleanup runs before local sync operations, status refreshes, settings reads/updates, and incoming Iroh sessions.

Modification timestamps are used only to choose which preserved version keeps the original path. They are never used as permission to discard the other version, so clock skew cannot cause data loss.

For edit-versus-delete conflicts, the edited content is preserved because the current protocol does not yet carry a timestamped deletion tombstone.

## Transfer protocol

- Control messages are length-prefixed JSON on one bidirectional QUIC stream.
- File data is sent on independent unidirectional QUIC streams.
- Files are read and written in 256 KiB chunks.
- Every transfer includes expected size and BLAKE3 hash.
- Incoming files are written to a `.syncpart` file, verified, and atomically renamed.
- Paths are validated as vault-relative paths and traversal is rejected.
- The receiving peer independently recomputes the synchronization plan.
- The baseline advances only after both final content manifests contain identical paths and content hashes.

## Important implementation rule

Do not replace this protocol with a shared-folder copy, fake success response, or smoke-only implementation. A successful sync result requires an authenticated Iroh peer connection and matching final content manifests.
