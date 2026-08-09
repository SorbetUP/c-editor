---
name: truthful-delivery
description: >
  Keep delivery reports factual. Every claim must be backed by code, tests,
  CI, logs, runtime evidence, or an explicit not-proven status.
argument-hint: '<delivery-claim>'
---

# Truthful Delivery

Use this skill before final reports, PR summaries, status updates, and any claim that work is done.

## Claim states

Every claim must be one of:

- **proven**: exact evidence is available.
- **partially proven**: code changed, but a required runtime or CI check is missing or still running.
- **not proven**: no sufficient evidence was inspected.
- **blocked**: a specific blocker prevents completion.
- **out of scope**: intentionally not part of the task.

## Evidence map

- Code change: file path and behavior changed.
- Test proof: command, test file, and assertion purpose.
- CI proof: commit SHA, workflow, job, and final state.
- Runtime proof: app run, log, or output observed.
- Filesystem proof: exact disk path or fixture state.
- Artifact proof: exact generated path, size, or diagnostic.

## Required final report

- What changed.
- What evidence proves it.
- What is not proven.
- What failed or is still pending.
- What was deliberately left out.

## Rules

- Do not say a feature works without evidence.
- Do not call CI green while workflows are pending or running.
- Do not call UI/runtime verified from static reading alone.
- Do not present a placeholder as a real implementation.

## APEX integration

- Init: define what evidence will be needed.
- Validate: collect evidence or mark missing proof.
- Verify: classify each claim by proof state.
- Finish: report only proven facts and explicit uncertainty.
