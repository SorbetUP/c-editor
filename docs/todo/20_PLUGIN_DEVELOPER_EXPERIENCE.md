# TODO — Developer experience des addons

## Objectif

Une API d'addons n'est durable que si un développeur peut comprendre permissions, lifecycle et surfaces sans lire les internals d'ElephantNote.

## P0 — SDK et types

Package SDK versionné avec types/schemas, manifest builder, IDs, errors, resource handles et examples. Aucun besoin d'importer des fichiers du repo principal.

## P0 — Dev mode

Install/link local, hot reload sûr, logs filtrés par addon, capability inspector, permissions/grants et restart service. Le dev mode reste opt-in et absent des releases normales.

## P0 — Validation manifest

CLI vérifie API version, permissions inutiles/manquantes, entry graph, platform packages, native hashes, extension points inconnus et package size.

## P0 — Test harness

Créer package, installer dans vraie app temp, activer, ouvrir contribution, exécuter action, disable/uninstall/reinstall. Automation API peut piloter la vraie app mais ne remplace pas le host par un mock.

## P1 — Documentation par contrat

Pour chaque namespace : invariants, exemples, error codes, permissions, lifecycle, platform notes. Expliquer explicitement que Pinia/Tauri/private DOM sont hors contrat et fournir l'alternative publique.

## P1 — Compatibility report

Commande comparant manifest/API avec le host local et expliquant incompatibilités avant runtime.

## P1 — Templates

Minimal worker addon, trusted UI addon, native service addon, provider connector et file handler. Templates courts, sans boilerplate non utilisé.

## P1 — Security review aid

Rapport permissions + network hosts + native binaries + external writes/tools exposés pour revue catalogue officiel/tiers.

## Validation

- Nouvel addon simple sans copier un addon officiel pour découvrir l'API.
- CI addon tiers valide package/manifest sans cloner tout ElephantNote.
- API dépréciée donne diagnostic clair.
- Harness prouve unload sans fuite.