---
name: elephantnote-mobile
description: >
  Rules for Android/mobile behavior, mobile parity, local vault assets, share flows,
  and sync with desktop.
argument-hint: '<mobile-task>'
---

# ElephantNote Mobile

Use this skill for Android/mobile code, mobile vault behavior, share/import flows, mobile sync, and desktop/mobile parity.

## Invariants

- Mobile must preserve local-first files and assets.
- Shared Markdown and asset rules still apply: images and generated files must use hidden internal storage where appropriate.
- Mobile sync must be compatible with desktop vault semantics.
- Mobile UI can adapt to screen size, but data behavior must match desktop.

## Read first

- Mobile app files under `build/build/android/**`.
- Shared contracts in `Elephant/shared/**`.
- Sync and vault skills when the task touches files or sync.
- Desktop implementation for the same feature.

## Verification

- Test a real mobile fixture or the smallest available mobile build/test path.
- Confirm note create/edit/import/share changes real storage.
- Confirm desktop can understand the files mobile creates.

## Anti-slop

No mobile-only shortcut that writes a different note or asset format from desktop.
