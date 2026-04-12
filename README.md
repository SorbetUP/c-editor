# ElephantNote

ElephantNote repose sur un coeur natif en C dans `engines/`, reutilise par la couche web/WASM, les outils CLI, Flutter et un shell macOS minimal.

## Structure canonique

- `engines/` : bibliotheques C et tests module par module
- `MarkdownEditorApp/` : shell macOS actif construit par le `Makefile` racine
- `tools/` : outils de debug, TUI et essais locaux
- `web/site/` : source GitHub Pages
- `dist/github-pages/` : artefact de publication genere
- `flutter/` : client Flutter
- `scripts/` : build, validation et maintenance
- `docs/` : documentation projet, rapports et notes de maintenance
- `legacy/` : code historique conserve hors du chemin de build canonique

## Ce qui n'est plus canonique

- `legacy/macos-root/` contient les anciens fichiers Objective-C qui encombraient la racine
- `versions/`, `old_builds/` et `backups/` restent des archives
- la racine doit rester reservee aux points d'entree, metadonnees et reperes projet

## Demarrage rapide

```bash
cd engines/editor && make test
cd ../cursor && make test
cd ../markdown && make test
./scripts/build_github_pages.sh
cd web/site && python3 -m http.server 8001
```

Puis ouvrir :

- `http://127.0.0.1:8001/` pour la landing Pages
- `http://127.0.0.1:8001/docs/index.html` pour l'editeur web

## Validation

Validation minimale :

```bash
cd engines/editor && make clean && make test && ./editor_test
cd engines/markdown && make clean && make test && ./markdown_test
cd engines/cursor && make clean && make test
cd engines/crypto_engine && make clean && make test
./scripts/build_github_pages.sh
./scripts/smoke_test_github_pages.sh
```

Nettoyage local :

```bash
make clean
make clean-repo
```

Les rapports de validation sont maintenant ranges dans `docs/reports/`.
