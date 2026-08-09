---
name: test-integrity
description: >
  Keep tests meaningful. A test must fail for the broken behavior it claims to
  protect and must not bypass the implementation under test.
argument-hint: '<test-change>'
---

# Test Integrity

Use this skill whenever tests are added, changed, reviewed, or used as proof.

## Test proof rule

A test is meaningful only if it would fail when the claimed behavior is absent or broken.

## Required review

For each new or changed test, answer:

- What product behavior does it protect?
- What exact bug would make it fail?
- Does it call the real helper, bridge, command, store, or runtime path?
- What is mocked, and why is that mock not replacing the behavior under test?
- Does it assert a specific output, effect, error, or persisted state?

## Strong assertions

Prefer assertions about:

- disk state;
- command name and payload;
- returned data shape and value;
- visible UI state;
- error message or recovery path;
- generated artifact;
- CI job or workflow state.

## Weak patterns

- only checks that a component renders;
- only checks that a function was called with no meaningful payload;
- mocks the full implementation path;
- snapshots unstable or empty output;
- accepts any truthy value;
- catches and ignores the error being tested.

## APEX integration

- Plan: define the failing old behavior.
- Execute: write test and implementation together.
- Validate: run the narrow test and suite.
- eXamine: ask whether the test would pass with an empty or hardcoded implementation.
