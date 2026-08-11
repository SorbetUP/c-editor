# Freya migration repository baseline

Audit snapshot: 2026-08-11, repository `/Users/sorbet/Desktop/Dev/c-editor`, branch `nsb/freya-native-migration`.

## Baselines and worktree truth

| Reference | Commit | Meaning | Trust level |
| --- | --- | --- | --- |
| `develop` | `62a606c145fe930682efbe491980c20ffeade14c` | Product baseline and Vue/Tauri behavior oracle | clean commit object; no claim about this checkout's dirty worktree |
| `nsb/freya-native-migration` | `387e0b151abc52c31ebd82ca2b24780b6df46b61` | Current migration HEAD when this audit started | migration commits only; checkout is dirty |
| merge-base | `62a606c145fe930682efbe491980c20ffeade14c` | Migration branch is based directly on `develop` | verified with `git merge-base` |

The checkout contains extensive pre-existing and concurrent user/agent modifications: backend Tauri, frontend Vue/renderer, Avalonia deletions, Freya untracked test/support files, WDIO/Playwright tooling and package/config changes. They are intentionally not treated as source implementations and are not included in this ledger unless they are committed at a cited SHA. A dirty file is not an integrable historical source.

The migration commits visible on the current branch are, in order, the shell/navigation/editor/library/settings/search/graph/drawing work followed by reproducibility and live-search corrections: `0f02a03b3`, `84b378427`, `2c27f01ba`, `298f3a026`, `42ee915f3`, `f0f0e4ab3`, `a20d92edc`, `d513f5113`, `f0e604472`, `620323cf4`, `6c55789c0`, `18efe047f`, `21cc7cb71`, `87fcff66d`, `39432fe41`, `d220cc1b2`, and `387e0b151`. These prove code history and focused boundaries, not complete product parity.

## Inventory snapshot

The checked-in inventories were parsed in full:

| Inventory | Count | Interpretation |
| --- | ---: | --- |
| `migration/freya/components.json` | 47 components | Vue component source coverage |
| `migration/freya/manual-port-required.json` | 2,000 findings | unsupported template/Web/CSS constructs; not implementation proof |
| `migration/freya/web-api-usage.json` | 193 findings | browser API usage requiring an explicit native adapter or Web island |

The previous migration map had 11 entries (10 `PARITY_PENDING`, 1 `WEB_ISLAND`). This audit expands it to all 47 component sources and the mission's cross-cutting functional surfaces. No component is silently dropped; several map many-to-one into the existing Freya modules.

## Branch and historical-source scan

Relevant local branches were inspected with `git log --all`, `git show --stat` and `git cat-file`. The source/provenance ledger records the exact evidence. Important reusable sources include:

| Functional area | Historical branch / SHA | What the commit actually contains |
| --- | --- | --- |
| vault picker, Android vault and drawer | `fix/android-foundations-rework` / `f88fd2b59e924efb44cebc50187bebaf54c3521a`; `integration/mobile-develop-refresh` / `7b52a6e984481a74b087b9cd838ad71f9dd3a262` | native Android vault/plugin foundations and mobile interaction runtime |
| folder and mobile sidebar actions | `fix/android-foundations-rework` / `483261a0b654f782236a75256a1019572c8d3583` | real SidebarNav note/folder actions |
| images and dropped attachments | `develop` / `4251a03dfd6f6cd100bd64ef7285f09a60bd2f72`; `origin/agent/fix-remaining-packaged-matrix` / `2e506f26f8bbb81e73e697dcde56c7b99bec12cd` | standard image contract and Excalidraw/drop paths |
| Muya/editor | `feature/rust-muya-editor-core` / `6e8643311ced2697cbb467d7dbee75faf06a2d11`; `feature/rust-muya-complete` / `879a6d163a7f1636b80753f372c2136da1ca2a87` | Rust-owned sessions and canonical rendering/serialization |
| settings/themes/locale | `feature/settings-ui-redesign` / `5c67c36520ce59a204fefe1f438c57758b0d25ca`; `feature/editor-excalidraw-i18n-themes` / `e05644a22210579e973457c287c5a0c33500a55a` | settings sections, tokens, tests and Excalidraw theme sync |
| Excalidraw durability | `origin/agent/excalidraw-durable-save-fix` / `a66ce839a46280e86992de9ff74d1ec3565e50d7` | persist/reopen scene and PNG path |
| search/wiki/graph | `assistant/wiki-links-graph-integration` / `0c2ebcc4e2a1a9194a91c96b14b20179bfc7a132`; `feature/wiki-knowledge-continuation` / `f96243e091f4ecd10fbc2c60342603e9ec09f6f6`; `feature/rust-knowledge-core` / `618678631f41dc03068799ed396cd868f31ac84d` | internal links, edge-aware graph, knowledge/wiki/chat routing and relation commands |
| addons | `develop` / `ca9841c6f526e8db36edd05bb4468dbbb5a9c641`; `develop` / `cad6f85c917867009e76044bb034980380a10dbe` | physical UI ownership boundary and lifecycle matrix |
| sync | `feature/iroh-vault-sync` / `736f5a09a1165aeae2e350c143a124a1ec85458d`; `feature/iroh-vault-sync` / `a5fde1d9519b0ca58063437ec024e450a8f1491a` | real Iroh backend/UI and smoke path |
| chat/AI | `develop` / `bd36da82677cbd83a965cb8cbbdf6aa210603ccc`; `feature/wiki-knowledge-continuation` / `f96243e091f4ecd10fbc2c60342603e9ec09f6f6` | Codex app-server runtime and knowledge/wiki routing |
| accessibility | `develop` / `7fec19c1c484415c5691e813492b11f75a667012`; `develop` / `b6e75837703d61bbbc36c1e462a2894a5cb7fece` | Android editor accessibility XML and chrome assertions |
| packaging/release | `develop` / `1e3e6ec5840d3439aa7de6a5b86a59853518f4ff` | release preparation and package contracts |
| performance | `develop` / `b7c85d786f7101c5e709b565aaf98e0212582b27` | large-edit autosave performance correction |

No `refs/pull/*` or `refs/merge-requests/*` refs are present in the local ref database. Remote tracking refs are available and were searched; a remote branch is recorded only when its SHA is directly available and relevant.

## Evidence boundary at migration HEAD

| Surface | Current committed target | Evidence observed | Classification |
| --- | --- | --- | --- |
| shell/navigation/library/settings | `Elephant/freya/src/app/**` | focused Freya Testing tests and source-grounded contracts; visual capture differs materially from source | `PARITY_PENDING` |
| editor | `Elephant/freya/src/editor.rs`, `app/editor_*.rs` | lifecycle, keyboard, IME and table/task tests; clipboard test remains blocked by headless clipboard/provider compilation and no GUI proof | `PARITY_PENDING` |
| search | `Elephant/freya/src/app/explorer*.rs` | production FTS path and live query commit; strict differential run reached search and then failed | `PARITY_PENDING` |
| graph/wiki | `Elephant/freya/src/app/graph_runtime.rs` plus knowledge-core | real service boundary test; current Freya rendering is not yet an equivalent interactive graph canvas | `PARITY_PENDING` |
| drawing | `Elephant/freya/src/app/drawing*.rs` | Freya Testing primitive scene/drag/zoom/serialization artifact; renderer is not wired into the production entry path and is not Excalidraw parity | `PARITY_PENDING` |
| Excalidraw | no isolated `Elephant/web_islands/excalidraw` target | historical real Web integration exists; current native primitives are only a prototype | `WEB_ISLAND` |
| Tauri source oracle | `/private/tmp/elephant-source-playwright.GtMMxG/output/manifest.json` | 14 actions/120 frames, but scroll and rail-drag postconditions were false and must be rerun | `PARITY_PENDING` |
| Tauri native capture | `/private/tmp/tauri-profile-fixed-runtime-10/manifest.json` | real window/native input launch passed; WKWebView accessibility tree exposed no DOM controls, remaining actions blocked | `BLOCKED` |
| packaged Freya | none | no packaged Freya target | `NOT_STARTED` |
| Android Freya | none | historical Android/Tauri proof is not Freya proof | `NOT_STARTED` |

The source and target artifacts are evidence for their respective layers only. No `PROVEN` claim is made in this audit.

## Rules for the next ports

1. Treat `develop@62a606c145fe930682efbe491980c20ffeade14c` and its real Tauri/Playwright behavior as the source oracle.
2. Reuse the complete historical modules cited in `source-provenance.json`; do not recreate UI from screenshots or from this ledger.
3. Treat all dirty checkout files as non-integrable until they acquire a reviewed commit SHA.
4. A green Freya Testing test proves only its headless native boundary. It does not prove packaged runtime, persistence/restart, visual parity, motion parity, Android, or addon lifecycle.
5. Do not change a status to `PROVEN` until the functional matrix's required artifacts exist for the final commit and the real user path passes.
