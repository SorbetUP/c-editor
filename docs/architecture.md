# Architecture ElephantNote

Ce document donne une vue rapide de la structure du depot et des flux principaux.

## Vue d'ensemble du depot

```mermaid
flowchart TD
    root["ElephantNote"]

    root --> engines["engines/"]
    root --> web["web/site/"]
    root --> scripts["scripts/"]
    root --> workflows[".github/workflows/"]
    root --> dist["dist/github-pages/"]
    root --> docs["docs/"]
    root --> flutter["flutter/"]
    root --> macos["Shells macOS historiques"]

    engines --> markdown["markdown<br/>Parsing Markdown + JSON"]
    engines --> editor["editor<br/>Document + ABI + HTML/JSON + WASM"]
    engines --> cursor["cursor<br/>Gestion curseur / edition hybride"]
    engines --> render["render_engine<br/>Rendu natif"]
    engines --> support["file/search/vault/crypto<br/>Sous-librairies C de support"]
    engines --> platform["vault_core / link_engine / privacy_engine<br/>sync_engine / render_ext"]
    engines --> ui["ui_framework / search_interface / hybrid_editor<br/>Couches UI / integration"]

    web --> docsApp["docs/index.html<br/>Demo web principale"]
    web --> docsAssets["Assets publishes pour GitHub Pages"]

    scripts --> buildPages["build_github_pages.sh"]
    scripts --> smoke["smoke_test_github_pages.sh"]
    scripts --> validation["run-full-validation.sh / validate-release.sh"]

    workflows --> pages["pages.yml<br/>Build + publish Pages"]
    dist --> pagesArtifact["Artefact final deploye"]

    macos --> legacyApps["MarkdownEditorApp / app bundles / versions historiques"]
```

## Dependances logiques du coeur C

```mermaid
flowchart LR
    markdown["engines/markdown"]
    editor["engines/editor"]
    cursor["engines/cursor"]
    render["engines/render_engine"]
    support["search / vault / file / crypto"]
    platform["vault_core / link_engine / privacy_engine / sync_engine / render_ext"]
    web["web/site/docs/index.html"]
    wasmEditor["editor.js + editor.wasm"]
    wasmCursor["cursor_wasm.js + cursor_wasm.wasm"]

    markdown --> editor
    editor --> wasmEditor
    cursor --> wasmCursor

    wasmEditor --> web
    wasmCursor --> web

    editor --> render
    support --> render
    support --> ui["Interfaces et apps"]
    platform --> ui
    platform --> editor
    editor --> ui
    cursor --> ui
```

## Flux d'execution de l'editeur web

```mermaid
sequenceDiagram
    participant U as Utilisateur
    participant UI as "web/site/docs/index.html"
    participant CW as "cursor_wasm.js/.wasm"
    participant EW as "editor.js/.wasm"
    participant ABI as "editor_abi.c"
    participant Core as "editor.c + markdown.c + json.c"

    U->>UI: Ouvre la page
    UI->>CW: Charge le module curseur
    UI->>EW: Charge le module editeur
    EW->>ABI: Expose ccall/cwrap
    ABI->>Core: Parse Markdown / export HTML / JSON
    Core-->>ABI: Resultats structures
    ABI-->>UI: Strings HTML / JSON
    UI-->>U: Rendu hybride ligne par ligne

    U->>UI: Clique / tape / navigue
    UI->>CW: Ajuste positions de curseur / split / merge
    UI->>EW: Demande HTML pour la ligne ou le bloc
    EW->>ABI: Conversion Markdown -> HTML
    ABI->>Core: Reparse local
    Core-->>UI: Nouveau rendu
```

## Pipeline GitHub Pages

```mermaid
flowchart LR
    push["Push sur main"] --> workflow[".github/workflows/pages.yml"]
    workflow --> emsdk["Setup Emscripten"]
    emsdk --> build["scripts/build_github_pages.sh"]

    build --> editorWasm["Compile engines/editor -> editor.js + editor.wasm"]
    build --> cursorWasm["Compile engines/cursor -> cursor_wasm.js + cursor_wasm.wasm"]
    build --> copy["Copie web/site + Notes + assets"]
    copy --> artifact["dist/github-pages/"]

    artifact --> smoke["scripts/smoke_test_github_pages.sh"]
    smoke --> upload["upload-pages-artifact"]
    upload --> deploy["deploy-pages"]
    deploy --> live["GitHub Pages live"]
```

## Lecture rapide

- `engines/markdown` fait le parsing et la transformation structurelle du Markdown.
- `engines/editor` est le coeur le plus important pour la demo web: il expose l'ABI C, la conversion HTML et la cible WASM.
- `engines/cursor` sert la logique d'edition hybride et de positionnement.
- `engines/vault_core`, `engines/link_engine`, `engines/privacy_engine`, `engines/sync_engine` et `engines/render_ext` sont la fondation en cours pour le futur mode vault local, les liens de notes, la confidentialite, la sync reseau et les extensions Markdown enrichies.
- `web/site/docs/index.html` est l'interface de reference actuellement exposee sur GitHub Pages.
- `scripts/build_github_pages.sh` et `scripts/smoke_test_github_pages.sh` sont le chemin canonique pour valider la publication.
- Les shells macOS et certains repertoires d'apps sont encore presents, mais ne sont pas le chemin principal de la demo web actuelle.
