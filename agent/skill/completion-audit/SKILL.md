---
name: completion-audit
description: >
  Audit delivery status before final response and classify each acceptance
  criterion as proven, partial, pending, blocked, or out of scope.
argument-hint: '<delivery-status>'
---

# Completion Audit

Use this skill before a final answer or PR summary.

## Required fields

For each acceptance criterion, report:

- criterion;
- status;
- supporting file, command, workflow, or log;
- remaining gap.

## Status values

Use only these statuses: proven, partial, pending, blocked, out of scope.

## Rules

- Green tests prove only the boundary they cover.
- A changed file alone does not prove runtime behavior.
- A local command alone does not replace CI when CI is available.
- A pending CI run is still pending.

## APEX integration

- Init: define criteria.
- Validate: attach command and CI evidence.
- Verify: attach runtime evidence when needed.
- Finish: report criteria by status.
