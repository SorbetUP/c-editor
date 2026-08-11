# ElephantNote — backlog d'architecture et de fiabilité

Cette branche `develop_todo` contient des propositions argumentées. Elle ne signifie pas que les éléments sont déjà implémentés ni validés.

## Point de départ observé

Le core expose déjà une API applicative versionnée (`Elephant/shared/apiContracts.js`) et une API addon additive, mais plusieurs addons officiels utilisent encore des détails d'implémentation (`Pinia._s`, `experimental.window`, commandes Tauri directes, DOM privé). Cela crée une fausse modularité : le code est physiquement dans un addon, mais reste couplé au host.

L'objectif est de faire d'Elephant un host stable : fichiers/vault, identité des ressources, lifecycle, permissions, événements, contributions UI, jobs, secrets, transactions et observabilité. Les domaines métier restent dans les addons.

## Invariants proposés

1. Un addon officiel ne dépend pas d'un store interne, d'un nom de composant Vue, d'un sélecteur DOM privé ou d'une commande Tauri non publique.
2. Toute API privilégiée est permission-scopée et auditable.
3. Les lectures peuvent être autorisées automatiquement ; les écritures externes, exécutions et destructions suivent une policy explicite.
4. Une mutation issue de l'IA est une proposal validée avec preconditions, pas un texte interprété librement.
5. Les ressources ont une identité stable ; les paths restent une propriété mutable, pas l'identité unique.
6. Desktop et mobile partagent les mêmes contrats produit, même si l'adapter natif diffère.
7. Les opérations longues sont des jobs cancellables et observables, pas des promises suspendues au renderer.
8. Les données dérivées restent reconstructibles et ne sont pas synchronisées par défaut.
9. Les secrets ne transitent jamais par les settings JSON ordinaires.
10. Un test de compilation/smoke ne vaut pas preuve utilisateur ; les scénarios réels restent obligatoires.

## Ordre recommandé

### P0 — Fondation

- `01_ADDON_API_V2.md`
- `02_DOCUMENT_RESOURCE_API.md`
- `03_CAPABILITIES_TOOLS_PROPOSALS.md`
- `04_SECRETS_PERMISSIONS_SECURITY.md`
- `05_BACKGROUND_JOBS_AUTOMATION.md`
- `13_TESTING_RELEASE_TRUST.md`
- `14_BRANCHING_ADDON_PINNING.md`

### P1 — Surfaces et performance

- `06_UI_EXTENSION_AND_WORKSPACE.md`
- `07_EDITOR_BLOCK_RUNTIME.md`
- `08_FILE_HANDLERS_ASSETS.md`
- `09_KNOWLEDGE_SEARCH_BOUNDARY.md`
- `10_SYNC_STORAGE_CONFLICTS.md`
- `11_CROSS_PLATFORM_MOBILE.md`
- `12_PERFORMANCE_OBSERVABILITY.md`

### P2 — Produit élargi

- `15_CORE_UX_RELIABILITY.md`
- `16_SPACES_ACL_PROVENANCE.md`
- `17_EXTERNAL_INTEGRATIONS.md`
- `18_STORAGE_MIGRATIONS_BACKUP.md`
- `19_ACCESSIBILITY_INPUT.md`
- `20_PLUGIN_DEVELOPER_EXPERIENCE.md`

## Règle de mise en œuvre

Ne pas implémenter une API parce qu'un addon particulier la demande sous une forme ad hoc. Identifier la primitive générique minimale, définir son contrat et ses invariants, puis migrer l'addon. Conserver les anciennes surfaces tant que la parité réelle n'est pas prouvée.