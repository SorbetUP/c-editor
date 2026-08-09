---
name: elephantnote-addons
description: >
  Rules for ElephantNote addons/plugins: real execution model, explicit capabilities,
  UI integration, tests, and no demo-only plugin behavior.
argument-hint: '<addons-task>'
---

# ElephantNote Addons

Use this skill for addon/plugin APIs, addon UI, capability prompts, install/remove flows, isolation, and tests.

## Invariants

- Addons must have a real execution path or be clearly labeled unavailable.
- Capabilities must be explicit and narrow.
- Installing, enabling, disabling, and removing must update real stored state.
- Addons must not break base note editing, vault safety, sync, search, or app startup.

## Read first

- Shared extension/addon contracts.
- Runtime bridge used to execute addon actions.
- Settings/UI where addons are managed.
- Tests for extension contracts and runtime isolation.

## Verification

- Install/enable a minimal real addon fixture.
- Execute one real action through the same path users trigger.
- Disable/remove it and confirm it stops running.
- Confirm unapproved capabilities are denied.

## Anti-slop

No fake marketplace, fake installed addon, or demo button that only updates frontend state.
