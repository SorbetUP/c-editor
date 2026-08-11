# 01 — Addon runtime contract

## Problème

Plus le catalogue grandit, plus un runtime permissif ou implicite crée du couplage : chemins internes connus des addons, service lifecycle différent selon plateforme, resources ad hoc et erreurs impossibles à diagnostiquer.

## Proposition

Définir un ABI/API addon versionné autour de quatre primitives :

1. **Manifest** : identité/version/platforms/dependencies/permissions/contributions.
2. **Resources** : APIs typées fournies/consommées.
3. **Services** : processus/native modules package-owned avec lifecycle contrôlé.
4. **Contributions** : UI/routes/commands/file handlers/tools/widgets.

## Lifecycle

États : discovered -> validated -> installed -> enabled -> starting -> ready -> degraded/error -> stopping -> disabled/uninstalled.

Chaque transition doit être observable et idempotente. Changer de vault doit déclencher un hook explicite plutôt que laisser un addon deviner l'état global.

## Dependencies

- résolution semver/version de resource ;
- dépendance optionnelle vs obligatoire ;
- cycle détecté avant start ;
- message d'erreur utilisateur indiquant dépendance manquante ;
- pas d'import direct de fichiers d'un autre addon.

## Services natifs

- package data dir propre ;
- health check ;
- startup timeout ;
- shutdown/kill tree ;
- crash restart policy bornée ;
- stdout/stderr capturés avec addon/run id ;
- aucun binaire recherché dans un path global non déclaré.

## Sandbox

Le runtime JS/WASM/native reçoit des handles capabilities, pas des objets globaux donnant filesystem/network/process. Toute escalation nécessite permission manifest + grant utilisateur/policy.

## Versioning

Un addon déclare `hostApi` min/max et versions de resources requises. Une incompatibilité bloque proprement l'addon ; elle ne doit pas produire une exception aléatoire plusieurs écrans plus tard.

## Acceptance

- addon manquant/désactivé ne crash pas les consommateurs optionnels ;
- service crash isolé du renderer ;
- disable libère watchers/process/events ;
- dependency cycle test ;
- version mismatch test ;
- changement de vault test ;
- même semantics desktop/Android/iOS pour les resources supportées.

## Impact addons

Cette base permet de rendre Open Models, Sync, Knowledge, Code Execution et futurs connecteurs réellement indépendants du host interne.