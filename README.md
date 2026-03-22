# ElephantNote

ElephantNote est maintenant le projet unique du depot.

Le depot est organise autour d'un coeur natif en C, decoupe en sous-librairies dans `engines/`, puis reutilise par la couche web/WASM, les outils de test, Flutter et l'application macOS.

## Structure

- `engines/markdown` : parsing Markdown, JSON intermediaire, export Markdown
- `engines/editor` : coeur document, ABI stable, conversion Markdown/JSON/HTML, cible WASM
- `engines/cursor` : gestion intelligente du curseur et edition hybride
- `engines/render_engine` : rendu natif
- `engines/search_engine`, `engines/vault_manager`, `engines/file_manager`, `engines/crypto_engine` : sous-librairies C de support
- `web/site` : site GitHub Pages et demo principale
- `flutter` : application Flutter
- `MarkdownEditorApp` et fichiers Obj-C racine : shell macOS
- `scripts/build_github_pages.sh` : build unique pour GitHub Pages

## Demarrage rapide

```bash
cd engines/editor && make test
cd ../cursor && make test
cd ../markdown && make test
./scripts/build_github_pages.sh
cd web/site && python3 -m http.server 8001
```

Ouvrir ensuite :

- `http://127.0.0.1:8001/` pour la landing GitHub Pages
- `http://127.0.0.1:8001/docs/index.html` pour l'editeur web principal

## GitHub Pages

Le workflow [pages.yml](.github/workflows/pages.yml) publie directement `web/site/`.

Le script [scripts/build_github_pages.sh](scripts/build_github_pages.sh) :

- compile `editor` et `cursor` en WebAssembly
- publie les artefacts dans `web/site/docs`
- prepare un artefact Pages unique, deployable localement comme en CI

## Validation minimale

```bash
cd engines/editor && make clean && make test && ./editor_test
cd engines/markdown && make clean && make test && ./markdown_test
cd engines/cursor && make clean && make test
cd engines/crypto_engine && make clean && make test
```
