# ElephantNote

ElephantNote est le projet unique du depot.

Le depot est organise autour d'un coeur natif en C, decoupe en sous-librairies dans `engines/`, puis reutilise par la couche web/WASM, les outils de test, Flutter et l'application macOS.

## Structure

- `README.md`, `Makefile`, `.github/workflows/pages.yml` : entrees racine et automatisation principale
- `engines/markdown` : parsing Markdown, JSON intermediaire, export Markdown
- `engines/editor` : coeur document, ABI stable, conversion Markdown/JSON/HTML, cible WASM
- `engines/cursor` : gestion intelligente du curseur et edition hybride
- `engines/render_engine` : rendu natif
- `engines/search_engine`, `engines/vault_manager`, `engines/file_manager`, `engines/crypto_engine` : sous-librairies C de support
- `web/site` : site GitHub Pages et demo principale
- `dist/github-pages` : artefact propre genere pour la publication Pages
- `flutter` : application Flutter
- `MarkdownEditorApp` et quelques fichiers Obj-C racine : shell macOS historique encore maintenu
- `versions` et `old_builds` : archives et historiques, hors chemin canonique pour le developpement courant
- `scripts/build_github_pages.sh` : build unique pour GitHub Pages
- `scripts/smoke_test_github_pages.sh` : validation HTTP locale de l'artefact Pages

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

Le workflow [pages.yml](.github/workflows/pages.yml) publie `dist/github-pages/` et ne pousse plus tout `web/site/`.

Le script [scripts/build_github_pages.sh](scripts/build_github_pages.sh) :

- compile `editor` et `cursor` en WebAssembly
- rafraichit les artefacts dans `web/site/docs`
- prepare un artefact Pages propre dans `dist/github-pages`

Le script [scripts/smoke_test_github_pages.sh](scripts/smoke_test_github_pages.sh) :

- sert `dist/github-pages` localement
- verifie la landing, l'editeur et les assets `.js` / `.wasm`
- detecte rapidement une publication Pages incomplete

## Validation minimale

```bash
cd engines/editor && make clean && make test && ./editor_test
cd engines/markdown && make clean && make test && ./markdown_test
cd engines/cursor && make clean && make test
cd engines/crypto_engine && make clean && make test
./scripts/build_github_pages.sh
./scripts/smoke_test_github_pages.sh
```
