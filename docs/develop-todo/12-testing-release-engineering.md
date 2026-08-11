# 12 — Tests et release engineering

## Principe

Un test qui importe une classe n'est pas une preuve qu'une fonctionnalité marche. Chaque promesse utilisateur importante doit avoir un test indépendant traversant les vraies frontières pertinentes.

## Test pyramid

- unit : parsers, policies, schemas, algorithms ;
- contract : host APIs et addon resource conformance ;
- integration : renderer↔host↔addon/native service ;
- E2E : workflow utilisateur ;
- fault tests : crash, cancel, network loss, disk full, permission denied ;
- packaging tests : artefact réellement distribué sur plateforme cible.

## Workflows E2E minimaux

Vault choose/create, note/folder create, external filesystem change, editor save/reopen, drag image/file, file handler fallback, settings persistence, addon install/enable/disable, graph/wiki/search, sync conflict, code execution stop, AI tool loop lorsqu'activé.

## Independent packaging proof

Tester AppImage/APK/etc. depuis artefact propre, pas depuis dev server. Vérifier bundled resources/addons/sidecars et absence de dépendance au checkout local.

## No fake success

Interdire tests/CI qui remplacent un backend manquant par un no-op tout en affirmant la feature valide. Les mocks servent aux tests unit/contract ; l'E2E doit signaler clairement mock vs real.

## Regression suites addon

Les tests spécialisés (retry, path traversal, tool actions, sync, knowledge, etc.) restent même après ajout d'un smoke global. Un test plus générique ne supprime pas une preuve plus forte.

## Flakiness

Quarantine documentée uniquement temporaire. Capturer seed/logs/artifacts. Ne pas relancer automatiquement jusqu'au vert sans exposer le premier échec.

## Release gate

Version release seulement si : source clean, migrations testées, licenses validées, platform artefact built, workflow matrix passée et release notes reliées aux commits. Une feature non prouvée doit être marquée experimental/unsupported.

## Acceptance

CI doit permettre de répondre précisément : quel workflow, quelle plateforme, quel artefact, quelle version addon et quelle preuve.