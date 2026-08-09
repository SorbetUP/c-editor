# Muya Rust API non equivalente a Muya JS

- Request: La version Muya Rust est fortement buggee et son API publique devrait etre strictement equivalente a Muya JS. Les methodes et evenements de Muya JS ne sont pas tous disponibles dans le runtime Rust.
- Level: high
- Date: 2026-07-19
- Slug: muya-rust-api-non-equivalente-a-muya-js

## Summary
- [x] Le runtime Rust actuel n'expose pas la surface publique de `Elephant/frontend/src/muya/lib/index.js`.
- [x] Le binding public `editorRuntimeResource` expose seulement `snapshot`, `dispatch`, `queryBlocks` et `watch`.
- [ ] La parite comportementale JS/Rust reste a mesurer sur chaque commande et chaque evenement.

## Repro Steps
- [x] Ouvrir une note avec le runtime Rust actif.
- [x] Comparer l'instance Muya JS historique avec le binding Rust publie.
- [x] Constater l'absence des methodes `getMarkdown`, `setMarkdown`, `getCursor`, `setCursor`, `format`, `undo`, `redo`, `search`, `replace`, `find`, `on`, `off`, `once`, `setOptions`, `copyAsHtml`, `copyAsRich`, `pasteAsPlainText`, `insertImage`, `createTable` et `destroy` dans le binding Rust.
- [ ] Executer les traces differentielles JS/Rust sur un corpus commun et enregistrer le premier ecart de snapshot, selection, HTML ou evenement.

## Environment
- [x] macOS, application Tauri, branche de travail `develop`.
- [x] Runtime Rust actif via `RustMuyaRuntimeEditor.vue` et `Elephant/frontend/src/renderer/src/editor-rust`.
- [x] Reference JS : `Elephant/frontend/src/muya/lib`.

## Observed vs Expected
- Observed:
- Le runtime Rust est une facade de commandes bas niveau, pas un remplacement de l'objet Muya JS.
- Les commandes et snapshots Rust ne garantissent pas les memes retours, evenements, historique, curseur, options, recherche, clipboard, images et tables.
- Expected:
- Une facade Muya Rust doit proposer les memes methodes, signatures, retours et evenements que Muya JS, ou documenter explicitement toute exception testee.

## Hypotheses
- [x] La migration a remplace l'API riche de Muya par `bridge.dispatch()` sans construire de facade de compatibilite.
- [x] Le protocole Rust ne couvre pas encore plusieurs operations de l'API JS, notamment recherche/remplacement, options, export HTML, selection formats et certains outils UI.
- [ ] Des differences de canonicalisation Markdown, offsets UTF-16, selection et ordre des evenements provoquent les regressions visibles.

## Investigation Plan
- [x] Inventorier l'API publique de Muya JS et l'API exposee par Rust.
- [ ] Construire une matrice methodes/commandes/evenements/retours JS-Rust.
- [ ] Executer les tests differentiels existants et sauvegarder les sorties divergentes.
- [ ] Ajouter des logs structures avec revision, commande, markdown, selection et patches.

## Fix Plan
- [ ] Definir un contrat de compatibilite unique a partir de l'API Muya JS historique.
- [ ] Ajouter une facade `MuyaRustCompat` qui conserve les signatures publiques et traduit vers le protocole Rust.
- [ ] Implementer dans Rust les commandes manquantes avant de declarer la parite.
- [ ] Aligner les evenements (`change`, `selectionChange`, `selectionFormats`, `scroll`, `focus`, `blur`, `crashed`) et leurs payloads.
- [ ] Supprimer les doubles conversions Markdown et verifier les offsets UTF-16.

## Regression Tests
- [ ] Test de presence et de signature de toute l'API publique JS sur la facade Rust.
- [ ] Tests differentiels JS/Rust pour chargement, edition, undo/redo, selection, formatage, listes, tableaux, images, recherche et clipboard.
- [ ] Test d'ordre des evenements et de revision apres chaque commande.
- [ ] Test d'integration Tauri avec saisie reelle, rendu temps reel et sauvegarde disque.

## Release Notes
- [ ] Ne pas annoncer la parite Muya tant que la matrice et les traces differentielles ne sont pas vertes.

## Risks
- [ ] Une facade partielle masquerait les commandes non implementees.
- [ ] Une conversion Markdown supplementaire peut corrompre le frontmatter et le contenu.
- [ ] Les offsets UTF-16 et les selections multi-noeuds peuvent diverger entre JS et Rust.

## Rollout
- [ ] Garder le runtime JS comme reference oracle pendant la migration.
- [ ] Activer Rust seulement apres validation de la matrice complete sur les fixtures et le parcours Tauri.
