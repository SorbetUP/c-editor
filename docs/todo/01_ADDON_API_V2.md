# TODO — Addon API v2 / frontière host stable

## Problème

L'API v1 additive offre déjà contributions, resources, lifecycle et storage. Pourtant plusieurs addons officiels accèdent encore à `experimental.window`, Pinia, Tauri ou au DOM. Le code est donc déplacé hors core sans être réellement découplé.

## Objectif

Créer une évolution additive `Addon API v2`, sans casser v1, centrée sur des domaines publics stables.

## P0 — Identité et contexte

Chaque appel reçoit implicitement `addonId`, version package, runtime mode, permissions accordées, vault/workspace actif et lifecycle signal. L'addon ne peut pas falsifier son identité dans un payload.

## P0 — Domaines publics

Namespaces versionnés : `app`, `vaults`, `documents`, `assets`, `workspace`, `commands`, `contributions`, `settings`, `editor`, `events`, `jobs`, `resources`, `secrets`, `permissions`, `proposals`, `objects`, `notifications`. Les retours sont des contrats, pas des formes du store Vue.

## P0 — Capability negotiation

`api.describe()` expose versions et features exactes. Un addon déclare `requires` / `optionalRequires`. Installation ou activation refuse proprement une incompatibilité avant le premier appel.

## P0 — Lifecycle

Toute subscription, timer, job, service natif, watcher, contribution et resource créée via l'API est rattachée au lifecycle. Après unload, aucun callback d'ancienne génération ne peut agir.

## P1 — Registration transactionnelle

Valider un batch complet de contributions puis publier atomiquement ; rollback si une entrée échoue. Supporter mise à jour versionnée sans clignotement UI.

## P1 — Dépréciations

Diagnostics de dépréciation : API, addon/version appelante, remplacement et échéance. Aucune suppression avant migration des addons officiels et preuve de parité.

## P1 — Isolation

Le worker isolé reste sans DOM/Tauri/Pinia. Les addons trusted doivent eux aussi préférer l'API publique : trusted ne doit pas signifier couplage aux internals.

## Validation

- Dashboard et Recently Edited migrés sans `pinia._s`.
- Graph/Wiki/Search sans commandes Tauri privées.
- Disable/uninstall sans listener, timer, service ou contribution résiduelle.
- Incompatibilité détectée avant activation.
- Même contrat desktop/mobile avec `unsupported` explicite.