# Native and Service Connector Roadmap

This document distinguishes implemented addons from connectors that require new privileged APIs. It is intentionally explicit so a UI prototype is never presented as a working integration.

## Current status

| Capability | Status | Current implementation |
| --- | --- | --- |
| Addon profile configuration | Implemented | Built-in `elephant.addon-profiles` reads `Addon Profiles/default.json` and manages official catalogue addons |
| Notion Markdown export import | Implemented | `com.elephantnote.notion-markdown-importer` imports local Markdown exports from `Imports/Notion/` |
| Direct Notion API synchronization | Not implemented | Requires OAuth, secrets and incremental block synchronization |
| Apple Calendar synchronization | Not implemented | Requires a macOS EventKit bridge and calendar permission handling |
| Apple Notes synchronization | Not implemented | Requires a macOS-only bridge; Apple Notes has no cross-platform public sync API |
| Email ingestion | Not implemented | Requires OAuth/IMAP credentials, keychain storage and background jobs |
| Muya 3D blocks | Not implemented | Requires a public editor extension API, asset lifecycle and a sandboxed renderer |

## Apple Calendar

A production connector should use EventKit through a dedicated macOS Tauri plugin rather than generic shell execution.

Required design:

- explicit `calendar.read` and `calendar.write` permissions;
- macOS permission prompt and denied/restricted states;
- calendar selection by stable identifier;
- deterministic mapping between event identifiers and generated notes;
- configurable time range and destination folder;
- idempotent updates and deletion policy;
- conflict reporting when both the note and event changed;
- no availability claim on Windows, Linux, Android or web.

The first safe increment should be read-only calendar-to-note synchronization. Bidirectional writes should follow only after stable identity and conflict tests exist.

## Apple Notes

Apple Notes does not expose a general cross-platform public synchronization API. A macOS connector can use supported automation surfaces where available, but fidelity and attachment access vary by OS version.

Required design:

- explicit macOS-only capability and warning;
- read-only import as the first increment;
- stable source identifiers stored in frontmatter;
- folder and account selection;
- HTML-to-Markdown conversion with fixtures;
- attachment export into ElephantNote `.assets`;
- duplicate prevention and incremental import;
- clear reporting for locked, shared or unsupported notes.

A connector must not claim full bidirectional Apple Notes synchronization until edits, attachments, tables, checklists and account boundaries are tested on supported macOS versions.

## Notion API synchronization

The local Markdown export importer is implemented. Direct service synchronization requires a separate OAuth connector.

Required design:

- OAuth or internal-integration token stored in the OS keychain;
- workspace and page selection;
- cursor-based pagination;
- block-tree to Markdown conversion;
- database property/frontmatter mapping;
- attachment download through the hardened network broker;
- incremental cursor and source page ID persistence;
- API rate-limit and retry handling;
- deletion and conflict policy.

## Email ingestion

Email rules should support sender/address filters and destination folders, but credentials must never be stored in addon JSON or ordinary vault notes.

Required design:

- Gmail/Microsoft OAuth and generic IMAP connector implementations;
- OS keychain-backed secrets API;
- `mail.read` permission with account and folder scopes;
- background scheduler with minimum intervals and cancellation;
- sender, recipient, subject and attachment filters;
- idempotency based on provider message ID / RFC Message-ID;
- HTML-to-Markdown conversion and quoted-thread policy;
- attachment storage under `.assets`;
- generated note templates and source metadata;
- a dry-run preview before enabling a rule.

## Muya 3D blocks

A 3D addon should use an editor contribution rather than injecting arbitrary HTML or remote scripts.

Proposed Markdown contract:

```markdown
```model3d
source: .assets/models/engine.glb
camera: orbit
height: 420
```
```

Required design:

- public `editor.blockRenderer` contribution point;
- local `.glb` and `.gltf` assets managed through ElephantNote `.assets`;
- sandboxed renderer with no remote JavaScript execution;
- lazy loading and explicit GPU/resource cleanup;
- poster image fallback for export and unsupported devices;
- file-size, texture-size and triangle-count limits;
- mobile capability detection;
- deterministic Markdown serialization and copy/paste behavior;
- tests for editor undo/redo, note switching and asset deletion.

The renderer should be implemented only after the Muya extension contract is stable enough that addons do not import private editor internals.
