# TODO — Storage, migrations, backup et recovery

## Problème

Le vault reste lisible sur disque, mais l'app et les addons accumulent metadata, settings, jobs et indexes. Sans schéma explicite, une mise à jour peut casser un état non reconstructible.

## P0 — Registry des stores

Chaque store déclare owner, schema version, classification, quota, sync policy, backup policy et `rebuildable`.

## P0 — Migrations transactionnelles

Migration N->N+1 idempotente, backup/checkpoint avant modification, rollback ou erreur bloquante explicite. Ne pas lancer une migration destructive depuis le renderer sans journal.

## P0 — Source vs derived

Données source/user : protéger et sauvegarder. Derived/cache : suppression/rebuild acceptable. Secrets : keystore, jamais backup ordinaire. Device-local : pas sync.

## P0 — Disk pressure

API espace libre/quota, cleanup cache LRU, downloads partiels. Refuser une opération si elle ne peut garantir un write atomique.

## P1 — Backup vault

Snapshot/export incluant fichiers source + metadata user nécessaire + manifest versions, excluant secrets/caches. Restore testé dans un profil neuf.

## P1 — Corruption detection

Checksums/SQLite integrity selon stores, quarantine et rebuild quand possible. Ne jamais réinitialiser silencieusement des données utilisateur corrompues.

## P1 — Uninstall policy

Lors d'un uninstall : choix conservation user-data vs suppression ; caches toujours supprimables. Réinstallation peut reprendre un store compatible.

## Validation

- Upgrade interrompu sans store demi-migré.
- Backup/restore reproduit notes et metadata user.
- Cache Knowledge corrompu reconstruit sans toucher notes.
- Low disk ne tronque pas document.
- Secrets absents du backup exporté.