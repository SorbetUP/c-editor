# Elephant coding-agent instructions

Before changing any code, read and obey `/AGENTS.md` and `agent/skill/elephant-change-safety/SKILL.md`.

The following rules are non-negotiable:

- preserve the working `develop` UI and behavior unless the user explicitly asks for a change;
- integrate code from known-good branches/commits instead of manually recreating it;
- moving core functionality into an add-on is an extraction, not a redesign;
- never claim an add-on, Marvin/AI engine or runtime works from compilation, registration, mocks or status probes alone;
- reproduce regressions and demonstrate red-before-green tests;
- do not weaken tests, replace the system under test with mocks, swallow errors or return fake success;
- run the real Tauri application and the packaged acceptance scenario for user-visible desktop claims;
- validate Android changes on an installed APK with real permissions and logs;
- retain structured renderer, Tauri and add-on/runtime logs;
- use `PROVEN`, `PARTIALLY PROVEN`, `NOT PROVEN` or `BLOCKED` accurately;
- stop and report missing proof rather than inventing an implementation.

Run this guard before delivery:

```bash
node build/scripts/verify-agent-governance.mjs
```
