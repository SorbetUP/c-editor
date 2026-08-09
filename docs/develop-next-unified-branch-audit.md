# `develop_next` unified branch recovery audit

This document records how the historical Elephant branches were evaluated and
recovered into `develop_next-unified-candidate2`. It is intentionally more
strict than a Git merge log: a commit is retained only when its behavior still
belongs in the current addon-first architecture.

## Decision rules

1. **Production editing belongs to the Rust/WASM editor.** The former Muya
   JavaScript implementation may remain only as a characterization oracle. No
   historical branch may restore it as a production dependency.
2. **Optional product features belong to physical addons.** AI, Chat, Search,
   Knowledge, Wiki, Graph, Sync, OCR, local models, Codex, Calendar, Sites,
   imports and code execution must not be copied back into Tauri core.
3. **A newer implementation wins over a textual merge.** Historical behavior
   is reapplied against the current APIs instead of restoring obsolete files,
   commands or paths.
4. **Mobile is a first-class runtime.** Every recovered UI must preserve touch
   navigation, safe-area insets, addon zones and the Android/iOS package
   boundary.
5. **No smoke-only recovery.** Recovered behavior needs an executable contract,
   package build, emulator interaction or production bundle guard.

## Branch and PR decisions

| Historical work | Original value | Unified recovery decision |
| --- | --- | --- |
| Rust editor / Muya parity branches, culminating in PR #80 | Rust document ownership, UTF-16 selection/history parity, WASM adapter and differential traces | **Kept as the production editor foundation.** Legacy Muya remains absent from production imports and is retained only for differential tests. |
| Addon runtime and extraction work, PRs #64 and #77 | Physical addon installation, permissions, resources, native process/service runners and removal of optional implementations from core | **Kept as the architectural boundary.** Later feature recovery is implemented inside `addons/official/**`, never by restoring global renderer or Tauri modules. |
| Physical catalogue and pack branches, PRs #78 and #79 | Versioned first-party catalogue, platform packages, hashes and protected Base/Develop packs | **Reimplemented for the canonical `addons/official/**` layout.** The old `addons/<module>` paths were rejected. Protected packs, manifest/catalogue consistency, reproducible native archives, BLAKE3 metadata and release artifacts are validated by new workflows. |
| Wiki organisation and semantic community work, PR #75 and related branches | Whole-vault embedding, deterministic communities, evidence-backed Wiki proposals, Graph/Search/Chat integration | **Kept through the physical Knowledge/Wiki/Graph addons.** Accepted drafts are now materialized as visible `Wiki/<slug>.md` notes; hidden `.elephantnote` files remain internal metadata only. |
| Chat→Knowledge action branches | Approval/execution actions and note/wiki modifications | **Already represented by the Knowledge provider and physical Chat addon.** Obsolete global Chat and RAG modules were not restored. |
| Sync pairing redesign, PR #24, and mobile Sync polish, PR #40 | QR/manual/file pairing, encrypted invitations, progress UX and mobile-safe interactions | **Rewritten inside `elephant.sync`.** The addon now owns validation, links, clipboard, invitation files, pairing and responsive settings. The former global `SyncSettingsPanel` was rejected. |
| Executable code blocks, PRs #27 and #54 | Real interpreters, integrated output, bounded logs, timeout/Stop and interpreter configuration | **Recovered through `elephant.code-execution` 2.2.0.** Interpreter management, retained output, execution IDs, status polling, timeout, cancellation and bounded output are owned by a persistent addon-native service. The Rust editor exposes only the neutral block runtime. The old Tauri-global process registry and Muya-specific renderer patches remain rejected. |
| Subscription/model provider branch, PR #32 | Codex subscription authentication/usage and local model management | **Superseded by `elephant.codex-connection`, `elephant.open-models` and the AI provider registry.** The historical `lib_min.rs` additions were intentionally not merged. |
| Excalidraw/i18n/themes branch, PR #23 | Live drawing theme updates, keyboard save/cancel, centralized translations, ISO languages, RTL, Beige/Pastel/Gamer Violet | **Selectively reapplied.** The current compact mobile-safe Excalidraw chrome and addon-capable Settings panel were retained while the missing theme and localization behavior was restored. |
| Performance hardening, PR #41 | Linear frontmatter parsing, non-quadratic path resolution, focused coverage and anti-fake test guards | **Reapplied and repaired.** The original workflow used the wrong Vitest binary and could not produce coverage; the unified workflow uses the monorepo binary, provisions Rust WASM and keeps diagnostic artifacts. |
| Android/mobile foundation and recovery branches, PR #81 | Native vault selection, Android startup recovery, search/settings/drawer interactions and signed ARM64 APK | **Kept as the mobile base.** Additional density, scalable addon settings navigation and Sync/Wiki fixes are layered on top rather than restoring older shells. |
| Old physical catalogue merge attempt, PR #82 | Direct Git merge of legacy catalogue history | **Closed unmerged.** It conflicted structurally with `addons/official/**`; its guarantees are covered by the unified implementation instead. |
| Unified recovery, PR #83 | Selective reconstruction across branch families | **Active integration branch.** It remains draft until all permanent workflows, native addon package builds and Android interaction tests are green. |

## Recovered behavior currently present

- protected Base and Develop-parity addon packs aligned to the official
  catalogue;
- reproducible Linux native packages for OCR, Code Execution, Knowledge, Open
  Models, Codex Connection and Sync;
- Rust/WASM production editor ownership and differential parity gates;
- package-owned Knowledge, Wiki, Graph, Search, Chat, Sync and provider APIs;
- visible accepted Wiki notes with normalized safe paths;
- encrypted Sync invitation validation, link/file import, export and clipboard
  flows;
- interruptible Code Execution service with package-owned interpreter checks,
  execution IDs, status polling, cancellation, timeout and bounded output;
- Android vault binary commands, runtime-only Search compatibility and official
  addon catalogue loading;
- full-screen phone Settings, overlay navigation drawer, compact library cards,
  safe areas and a horizontally scalable addon section rail;
- Beige, Pastel and Gamer Violet themes;
- centralized multilingual messages, ISO language selection and RTL document
  direction;
- live Excalidraw theme updates plus Ctrl/Cmd-S and Escape handling;
- linear large-note preview parsing and linear deep-path resolution;
- anti-disabled/focused/fake-test checks and focused coverage thresholds.

## Merge gate

PR #83 must stay draft until all of the following pass on the same head commit:

- general CI quality gate, Rust checks/tests and packaged macOS smoke;
- full unit, lint, PR build and E2E workflows;
- Rust Editor Ownership and Muya/WASM parity workflows;
- Addon Platform Validation and protected pack validation;
- reproducible physical addon release packaging;
- Iroh Sync validation;
- signed ARM64 Android APK size guard;
- Android emulator startup plus real Search, Settings and drawer interactions.
