# Remaining branch audit — 2026-07-16

Comparison target: `develop_next-integration-repair`.

## `develop_next-unified-candidate2`

- Tip: `1a511de6dc8c1f203c64e69c9b5d23ee35a0d155`
- Merge base: `1a511de6dc8c1f203c64e69c9b5d23ee35a0d155`
- Unique commits not in integration: **0**
- Integration commits not in branch: **199**

### Unique commits

_None._

### Unique changed paths

_None._

## `develop_next-mobile-recovery`

- Tip: `1a847708f7d541932b9b26be914ddfaca4bad6b9`
- Merge base: `1a847708f7d541932b9b26be914ddfaca4bad6b9`
- Unique commits not in integration: **0**
- Integration commits not in branch: **253**

### Unique commits

_None._

### Unique changed paths

_None._

## `develop_next`

- Tip: `e2bcf622e8649c0ceb85b6c7d5966ccdd21ebf90`
- Merge base: `e2bcf622e8649c0ceb85b6c7d5966ccdd21ebf90`
- Unique commits not in integration: **0**
- Integration commits not in branch: **310**

### Unique commits

_None._

### Unique changed paths

_None._

## `feature/physical-addon-packages`

- Tip: `638eeefbc383079fe4ef422bc6bb271e1e0f7540`
- Merge base: `638eeefbc383079fe4ef422bc6bb271e1e0f7540`
- Unique commits not in integration: **0**
- Integration commits not in branch: **2192**

### Unique commits

_None._

### Unique changed paths

_None._

## `refactor/muya-rust-file-by-file`

- Tip: `4429e21757260998891340e5d2933e0049e14b6d`
- Merge base: `4429e21757260998891340e5d2933e0049e14b6d`
- Unique commits not in integration: **0**
- Integration commits not in branch: **2479**

### Unique commits

_None._

### Unique changed paths

_None._

## `assistant/wiki-organization-continuation`

- Tip: `b69fb37b8b44b8fb9819547993b69a1b76a7162e`
- Merge base: `b69fb37b8b44b8fb9819547993b69a1b76a7162e`
- Unique commits not in integration: **0**
- Integration commits not in branch: **2912**

### Unique commits

_None._

### Unique changed paths

_None._

## `assistant/wiki-links-graph-integration`

- Tip: `b34b29efb13cdce74b89218206de629d15abe9c0`
- Merge base: `c01effa8eff8c450f9b2b8a22c3920f2b84b6e9e`
- Unique commits not in integration: **15**
- Integration commits not in branch: **3169**

### Unique commits

```text
ea7941c9 2026-07-14T02:08:00+02:00 SorbetUP :: ci: temporarily patch Wiki PR head
57b0a23e 2026-07-14T02:11:45+02:00 SorbetUP :: ci: trigger Wiki head fixer from PR reopen
6394f84e 2026-07-14T02:13:28+02:00 SorbetUP :: ci: temporarily apply final Wiki contract correction
049d8274 2026-07-14T02:15:56+02:00 SorbetUP :: ci: temporarily apply final Wiki test correction
992ee755 2026-07-14T02:18:57+02:00 SorbetUP :: ci: remove temporary Wiki head fixer
745db8cc 2026-07-14T02:19:23+02:00 SorbetUP :: ci: restore base Knowledge Core workflow
78de555e 2026-07-14T02:19:34+02:00 SorbetUP :: ci: restore base unit test workflow
0adcd8ee 2026-07-14T02:22:42+02:00 SorbetUP :: ci: temporarily apply Wiki metadata refresh fix
906fa3b5 2026-07-14T02:25:30+02:00 SorbetUP :: ci: apply Wiki metadata refresh correction from PR base
e160583d 2026-07-14T02:27:19+02:00 SorbetUP :: ci: restore base unit test workflow
fc6f70c3 2026-07-14T02:27:41+02:00 SorbetUP :: ci: restore base Knowledge Core workflow
0933d874 2026-07-14T00:37:48Z ElephantNote Bot :: [skip ci] Record Wiki links graph validation
7755667e 2026-07-14T03:03:16+02:00 SorbetUP :: ci: apply Wiki metadata refresh regression from PR base
671769c0 2026-07-14T03:05:56+02:00 SorbetUP :: ci: restore base E2E workflow
b34b29ef 2026-07-14T01:15:00Z ElephantNote Bot :: [skip ci] Record Wiki links graph validation
```

### Unique changed paths

```text
M	.github/wiki-links-graph-validation.txt
```

## `assistant/validate-knowledge-graph-wiki`

- Tip: `f2aee77dd4e86ef830bf6c7ab091bc9038206a71`
- Merge base: `e1123ba3377bff11a23eb21eca76f105b590aeef`
- Unique commits not in integration: **63**
- Integration commits not in branch: **3466**

### Unique commits

```text
eec7f590 2026-07-10T02:46:57+02:00 SorbetUP :: chore(ci): allow applying wiki continuation patch
0b6f7da9 2026-07-10T02:49:01+02:00 SorbetUP :: chore(ci): expose wiki patch failures
5ed3770c 2026-07-10T02:51:56+02:00 SorbetUP :: chore(ci): simplify wiki patch diagnostics
bf1ec97d 2026-07-10T02:52:11+02:00 SorbetUP :: chore(ci): restore wiki patch validation after diagnostics
78131b9d 2026-07-10T02:52:16+02:00 SorbetUP :: placeholder
65d2c679 2026-07-10T02:52:25+02:00 SorbetUP :: chore(ci): capture automatic wiki patch error
86b15ab5 2026-07-10T02:55:22+02:00 SorbetUP :: chore(ci): post automatic wiki patch error
5ba01816 2026-07-10T02:56:35+02:00 SorbetUP :: chore(ci): keep compact wiki patch diagnostic workflow
d7478cb9 2026-07-10T02:57:27+02:00 SorbetUP :: fix(ci): parse branch patch as YAML
3db72265 2026-07-10T03:03:13+02:00 SorbetUP :: chore(ci): diagnose latest automatic wiki patch
5ecf96d8 2026-07-10T03:09:00+02:00 SorbetUP :: fix(ci): run validated automatic wiki patch payload
d5848433 2026-07-10T03:12:53+02:00 SorbetUP :: chore(ci): diagnose validated wiki patch
d884750c 2026-07-10T03:15:37+02:00 SorbetUP :: fix(ci): make wikilink patch resilient
5861dedc 2026-07-10T03:17:46+02:00 SorbetUP :: chore(ci): diagnose wiki format failure
4c7410af 2026-07-10T03:19:18+02:00 SorbetUP :: fix(ci): preserve Rust escaped characters in wiki patch
7bb8bf98 2026-07-10T03:21:00+02:00 SorbetUP :: chore(ci): diagnose escaped wiki format failure
dba8fc92 2026-07-10T03:22:43+02:00 SorbetUP :: fix(ci): sanitize generated Rust character escapes
95758a9b 2026-07-10T03:31:56+02:00 SorbetUP :: chore(ci): remove temporary wiki patch workflow
a412b737 2026-07-10T03:32:02+02:00 SorbetUP :: chore(ci): remove temporary wiki diagnostic workflow
7019335d 2026-07-10T03:33:00+02:00 SorbetUP :: chore(ci): validate wiki frontend continuation
1f30f7b9 2026-07-10T03:34:15+02:00 SorbetUP :: fix(ci): enable pnpm before frontend validation
83d670ed 2026-07-10T03:36:28+02:00 SorbetUP :: chore(ci): remove temporary frontend validation workflow
51c34a30 2026-07-13T00:54:30+02:00 SorbetUP :: ci(knowledge): format Rust fixes and remove native graph cap
2faff3e0 2026-07-12T22:54:51Z github-actions[bot] :: fix(knowledge): format fixes and remove native graph cap
1e77f085 2026-07-13T00:56:27+02:00 SorbetUP :: docs(knowledge): record graph and wiki runtime guarantees
cebd1e98 2026-07-13T01:04:21+02:00 SorbetUP :: test(graph): guard both renderer bridges against hidden caps
c8363e10 2026-07-13T01:13:09+02:00 SorbetUP :: test(chat): match explicit knowledge index lifecycle
33bb6729 2026-07-13T01:13:21+02:00 SorbetUP :: test(chat): require direct RAG calls without hidden indexing
d6ed1a7c 2026-07-13T01:13:31+02:00 SorbetUP :: ci(test): run maintained app unit suite
afebcb56 2026-07-13T01:14:30+02:00 SorbetUP :: ci(e2e): retain Playwright diagnostics
b81a2970 2026-07-13T01:18:41+02:00 SorbetUP :: test(e2e): run against the Tauri web renderer
060a1cd9 2026-07-13T01:18:55+02:00 SorbetUP :: test(e2e): smoke-test the Tauri renderer shell
bb06b54c 2026-07-13T01:19:07+02:00 SorbetUP :: test(e2e): validate the uncapped graph bridge in browser
169a82fe 2026-07-13T01:19:19+02:00 SorbetUP :: test(e2e): validate wiki command wiring in browser
8e286384 2026-07-13T01:19:30+02:00 SorbetUP :: test(e2e): verify renderer security policy in browser
fc20fbcf 2026-07-13T01:19:49+02:00 SorbetUP :: ci(e2e): install Chromium for Tauri renderer tests
939a70b3 2026-07-13T01:25:05+02:00 SorbetUP :: test(e2e): replace Electron launcher with Tauri web mock
19646f68 2026-07-13T01:25:25+02:00 SorbetUP :: test(e2e): boot renderer through the Tauri mock
5023f6d6 2026-07-13T01:25:38+02:00 SorbetUP :: test(e2e): migrate parity baseline from Electron to Tauri
ed32e7d5 2026-07-13T01:28:16+02:00 SorbetUP :: test(e2e): target the mounted Vue root
e69b4ed4 2026-07-13T01:28:36+02:00 SorbetUP :: test(e2e): define non-blank by mounted layout
a607f5af 2026-07-13T01:32:07+02:00 SorbetUP :: test(e2e): detect visible fixed-position renderer content
8442a5b9 2026-07-13T01:37:41+02:00 SorbetUP :: ci(parity): validate the directory actually generated
550f1a98 2026-07-13T01:43:01+02:00 SorbetUP :: ci(guard): prepare explicit chat lifecycle update
3472747f 2026-07-13T01:43:13+02:00 SorbetUP :: ci(guard): trigger one-shot guard update
68d19c12 2026-07-12T23:45:21Z github-actions[bot] :: fix(ci): guard explicit chat knowledge lifecycle
153c77d9 2026-07-13T01:46:29+02:00 SorbetUP :: docs(knowledge): document uncapped graph validation
facd8685 2026-07-13T01:53:03+02:00 SorbetUP :: ci(guard): retain critical-flow diagnostics
24ca9360 2026-07-13T01:56:38+02:00 SorbetUP :: ci(guard): prepare Iroh critical-flow invariants
5df12738 2026-07-13T01:56:49+02:00 SorbetUP :: ci(guard): trigger Iroh guard migration
480444aa 2026-07-12T23:56:54Z github-actions[bot] :: fix(ci): validate the real Iroh synchronization runtime
4aacd6a5 2026-07-13T01:58:05+02:00 SorbetUP :: docs(validation): record Iroh runtime guard coverage
eeba571f 2026-07-13T02:01:09+02:00 SorbetUP :: ci(platform): retain compatibility-guard diagnostics
f1196830 2026-07-13T02:06:23+02:00 SorbetUP :: fix(chat): isolate desktop knowledge grounding from mobile
58bbbed0 2026-07-13T02:06:43+02:00 SorbetUP :: fix(platform): guard the current desktop grounding functions
8a40178a 2026-07-13T02:10:44+02:00 SorbetUP :: ci(test): run contracts with the installed Vitest binary
c0a60453 2026-07-13T02:10:58+02:00 SorbetUP :: ci(guard): prepare installed Vitest command invariant
3ad8ab4f 2026-07-13T02:11:07+02:00 SorbetUP :: ci(guard): trigger contract command update
ecaf15b1 2026-07-13T00:11:13Z github-actions[bot] :: fix(ci): guard the installed Vitest contract command
52950a10 2026-07-13T02:11:59+02:00 SorbetUP :: docs(validation): record focused contract test runtime
bcb0d01a 2026-07-13T02:17:17+02:00 SorbetUP :: ci(diagnostics): retain Cargo test and license failures
b45458c5 2026-07-13T02:24:54+02:00 SorbetUP :: test(platform): match current desktop chat grounding
f2aee77d 2026-07-13T02:25:10+02:00 SorbetUP :: fix(licenses): support the configured pnpm modules directory
```

### Unique changed paths

```text
M	.github/workflows/ci.yml
M	.github/workflows/e2e.yml
M	.github/workflows/test.yml
M	Elephant/backend/knowledge-core/README.md
M	Elephant/backend/knowledge-core/src/chunking.rs
M	Elephant/backend/tauri/src/chat_runtime.rs
M	Elephant/backend/tauri/src/knowledge_wikis.rs
M	Elephant/backend/tauri/src/platform_contract_tests.rs
M	Elephant/frontend/src/renderer/src/platform/tauriElephantNoteBridge.js
M	build/scripts/thirdPartyChecker.js
M	build/scripts/verify-critical-flows.mjs
M	build/scripts/verify-tauri-platforms.mjs
M	tests/app/e2e/atomic-views.spec.js
M	tests/app/e2e/helpers.js
M	tests/app/e2e/launch.spec.js
M	tests/app/e2e/parity/ui-parity-contract.spec.js
M	tests/app/e2e/playwright.config.js
M	tests/app/e2e/search-inspect.spec.js
M	tests/app/e2e/xss.spec.js
M	tests/app/unit/elephantnote/domainClients.spec.js
M	tests/app/unit/platform/installKnowledgeRuntimeBridge.spec.js
M	tests/app/unit/specs/main/elephantnote/chatClient.spec.js
```

## `feature/trusted-addon-runtime`

- Tip: `fff77240bd7ba3f02f917f2844b3fc1255fd20d5`
- Merge base: `fff77240bd7ba3f02f917f2844b3fc1255fd20d5`
- Unique commits not in integration: **0**
- Integration commits not in branch: **2981**

### Unique commits

_None._

### Unique changed paths

_None._

## `feature/rust-knowledge-core`

- Tip: `1e77f085713d26fba1c4517c2cd5a25df5e65bdc`
- Merge base: `e1123ba3377bff11a23eb21eca76f105b590aeef`
- Unique commits not in integration: **25**
- Integration commits not in branch: **3466**

### Unique commits

```text
eec7f590 2026-07-10T02:46:57+02:00 SorbetUP :: chore(ci): allow applying wiki continuation patch
0b6f7da9 2026-07-10T02:49:01+02:00 SorbetUP :: chore(ci): expose wiki patch failures
5ed3770c 2026-07-10T02:51:56+02:00 SorbetUP :: chore(ci): simplify wiki patch diagnostics
bf1ec97d 2026-07-10T02:52:11+02:00 SorbetUP :: chore(ci): restore wiki patch validation after diagnostics
78131b9d 2026-07-10T02:52:16+02:00 SorbetUP :: placeholder
65d2c679 2026-07-10T02:52:25+02:00 SorbetUP :: chore(ci): capture automatic wiki patch error
86b15ab5 2026-07-10T02:55:22+02:00 SorbetUP :: chore(ci): post automatic wiki patch error
5ba01816 2026-07-10T02:56:35+02:00 SorbetUP :: chore(ci): keep compact wiki patch diagnostic workflow
d7478cb9 2026-07-10T02:57:27+02:00 SorbetUP :: fix(ci): parse branch patch as YAML
3db72265 2026-07-10T03:03:13+02:00 SorbetUP :: chore(ci): diagnose latest automatic wiki patch
5ecf96d8 2026-07-10T03:09:00+02:00 SorbetUP :: fix(ci): run validated automatic wiki patch payload
d5848433 2026-07-10T03:12:53+02:00 SorbetUP :: chore(ci): diagnose validated wiki patch
d884750c 2026-07-10T03:15:37+02:00 SorbetUP :: fix(ci): make wikilink patch resilient
5861dedc 2026-07-10T03:17:46+02:00 SorbetUP :: chore(ci): diagnose wiki format failure
4c7410af 2026-07-10T03:19:18+02:00 SorbetUP :: fix(ci): preserve Rust escaped characters in wiki patch
7bb8bf98 2026-07-10T03:21:00+02:00 SorbetUP :: chore(ci): diagnose escaped wiki format failure
dba8fc92 2026-07-10T03:22:43+02:00 SorbetUP :: fix(ci): sanitize generated Rust character escapes
95758a9b 2026-07-10T03:31:56+02:00 SorbetUP :: chore(ci): remove temporary wiki patch workflow
a412b737 2026-07-10T03:32:02+02:00 SorbetUP :: chore(ci): remove temporary wiki diagnostic workflow
7019335d 2026-07-10T03:33:00+02:00 SorbetUP :: chore(ci): validate wiki frontend continuation
1f30f7b9 2026-07-10T03:34:15+02:00 SorbetUP :: fix(ci): enable pnpm before frontend validation
83d670ed 2026-07-10T03:36:28+02:00 SorbetUP :: chore(ci): remove temporary frontend validation workflow
51c34a30 2026-07-13T00:54:30+02:00 SorbetUP :: ci(knowledge): format Rust fixes and remove native graph cap
2faff3e0 2026-07-12T22:54:51Z github-actions[bot] :: fix(knowledge): format fixes and remove native graph cap
1e77f085 2026-07-13T00:56:27+02:00 SorbetUP :: docs(knowledge): record graph and wiki runtime guarantees
```

### Unique changed paths

```text
M	Elephant/backend/knowledge-core/README.md
M	Elephant/backend/knowledge-core/src/chunking.rs
M	Elephant/backend/tauri/src/knowledge_wikis.rs
M	Elephant/frontend/src/renderer/src/platform/tauriElephantNoteBridge.js
```

