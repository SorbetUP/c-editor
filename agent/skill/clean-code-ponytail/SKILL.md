---
name: clean-code-ponytail
description: >
  Adapter skill for clean-code requests. Delegates clean code, maintainability,
  over-engineering, and minimal-diff decisions to Ponytail.
argument-hint: '<code-or-diff>'
---

# Clean Code = Ponytail

Use this skill whenever the request says clean code, maintainability, refactor, simplify, reduce boilerplate, reduce complexity, or remove over-engineering.

## Rule

Load and apply `agent/skill/ponytail/SKILL.md`. Clean code in this repo means:

- less code when behavior stays correct;
- fewer abstractions;
- fewer dependencies;
- existing helpers before new helpers;
- deletion before addition;
- no removal of validation, security, accessibility, data safety, or explicitly requested behavior.

## Review output

```md
Ponytail clean-code review:

- delete:
- reuse:
- simplify:
- keep because safety/behavior:
```

## Anti-slop

Do not rewrite code into a new architecture just because it looks cleaner. The cleanest code is often the small fix in the existing flow.
