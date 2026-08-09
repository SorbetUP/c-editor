# Built-in Addons

Built-in addons are compiled with ElephantNote, use the same manager and contribution contracts as the addon platform, and do not require Community Addons mode.

## Daily Notes

**ID:** `elephant.daily-notes`  
**Version:** `1.1.0`  
**Enabled by default:** yes

Creates or opens:

```text
Daily/YYYY-MM-DD.md
```

A new daily note contains:

- frontmatter with date and timestamps;
- links to the previous and next day;
- Focus;
- Tasks;
- Notes;
- End-of-day review.

Existing daily notes are never overwritten. All adjacent-day links are derived from one timestamp to avoid midnight boundary inconsistencies.

## Quick Capture

**ID:** `elephant.quick-capture`  
**Version:** `1.1.0`  
**Enabled by default:** yes

Creates a collision-resistant note under:

```text
Inbox/Quick capture YYYY-MM-DD HH-MM-SS-mmm.md
```

Each note is marked with:

```yaml
status: unprocessed
```

and includes a review task. Programmatic invocations may pass an optional `{ title, content }` payload.

## Vault Overview

**ID:** `elephant.vault-overview`  
**Version:** `1.1.0`  
**Enabled by default:** yes

Generates:

```text
Reports/Vault Overview.md
```

The report is derived from the real Markdown index and resolved wiki graph. It includes:

- indexed note count;
- resolved-link count;
- connected-note count;
- graph link coverage;
- most connected notes with incoming/outgoing counts;
- orphan notes;
- a deterministic path-sorted note index.

## Addon Profiles

**ID:** `elephant.addon-profiles`  
**Version:** `1.0.0`  
**Enabled by default:** no

Creates and applies a versioned configuration file:

```text
Addon Profiles/default.json
```

The profile can install, update, enable and disable addons from the official catalogue. It deliberately rejects unknown addon IDs and cannot install arbitrary URLs or local packages. Enabling community addons through a profile still requires Community Addons consent to be active first.

Each application writes an auditable result to:

```text
Reports/Addon Profile.md
```

## Addon Inspector

**ID:** `elephant.addon-inspector`  
**Version:** `0.3.0`  
**Enabled by default:** no

A developer reference showing action, settings and sidebar contributions. It is intentionally not part of the default user workflow.

## Per-vault external addon data

User-installed packages remain scoped to the active vault:

```text
.elephantnote/addons/
├── registry.json
├── packages/
│   └── <addon-id>/
└── data/
    └── <addon-id>/storage.json
```

The `.elephantnote` directory remains hidden from the normal vault explorer.
