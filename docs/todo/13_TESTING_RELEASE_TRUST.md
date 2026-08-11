# TODO — Tests fiables et preuve de release

## Contrainte existante

Le repo interdit le retour du vieux système de tests JS/TS `.spec/.test` utilisé comme preuve artificielle. Conserver cette règle. Les comportements se prouvent dans le vrai produit ; les invariants statiques restent des guards séparés.

## P0 — Taxonomie de preuve

Étiqueter chaque check : compile/type/lint, static architecture guard, Rust contract, protocol adapter, real app acceptance, packaged app acceptance, platform physical/emulator. Une couche ne prétend jamais prouver la couche supérieure.

## P0 — Scénarios critiques

Startup, vault select/create/reopen, file/folder create, edit/autosave/restart, move/rename/delete, drag/drop file/image, file handler, settings, addons install/enable/use/disable/uninstall/reinstall, sync, editor/Excalidraw et erreurs visibles/loggées.

## P0 — Agent

Un provider contrôlé peut valider la state machine, mais l'acceptance app doit prouver UI -> tool call -> tool result -> second model turn -> final/proposal. Tester policy, stale proposal, cancel, max steps et prompt injection.

## P0 — Mutation sensitivity

Chaque nouveau scénario doit devenir rouge si son comportement protégé est volontairement cassé, puis repasser après restauration.

## P1 — Fixtures

Vaults synthétiques versionnés : Unicode, gros fichiers, malformed markdown, links, conflicts, assets. Aucune donnée utilisateur réelle en CI.

## P1 — Flakiness

Attendre conditions/événements plutôt que sleeps arbitraires. Seeds contrôlés, timeouts motivés. Une flakiness reste visible, pas masquée par retries illimités.

## P1 — Packaged proof

Les fonctions dépendant bundling/addons/sidecars se prouvent dans AppImage/DMG/MSI/APK réel selon le claim.

## Validation

- Aucun quality gate « pass » avec scénario critique skipped sans justification.
- Logs/artifacts localisent l'étape exacte.
- Smoke import addon jamais présenté comme preuve fonctionnelle.
- Chaque release expose sa matrice de plateformes/scénarios.