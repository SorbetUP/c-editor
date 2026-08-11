# 04 — Vault, stockage, identité et événements

## Problème

Les paths sont pratiques mais insuffisants comme identité : rename/move casse graph, citations, tasks, sync baselines et proposals. Les addons ont aussi besoin d'un flux fiable des modifications, y compris celles venant de l'explorateur système.

## Stable Resource Handle

Proposer un `ResourceId` interne associé à path + type + version/content hash. Le path reste visible/exportable mais les relations internes utilisent l'ID.

Le mapping doit survivre rename/move lorsque l'identité peut être établie. En cas de remplacement ambigu, mieux vaut créer une nouvelle identité que fusionner silencieusement deux fichiers.

## Version / preconditions

Toute lecture peut retourner `version`. Une écriture sensible peut exiger `ifVersion`. Un mismatch produit `conflict`, pas overwrite.

## File change journal

Events ordonnés : create, modify, move, delete, metadata_changed, vault_changed. Inclure resource id, old/new path si move, version, origine lorsque connue (`editor`, `addon`, `sync`, `external_fs`).

## Watchers

- un seul service host mutualisé plutôt qu'un watcher par addon ;
- debounce/coalescing documenté ;
- overflow détecté => rescan/reconciliation explicite ;
- ignore rules communes ;
- symlink policy claire.

## Writes

- atomic temp+rename lorsque possible ;
- fsync selon niveau de durabilité ;
- save errors remontées jusqu'à UI ;
- aucune promesse « saved » avant confirmation host.

## Hidden/internal directories

`.assets`, `.dashboard`, `.elephantnote` et autres espaces réservés doivent être connus via metadata/policy, pas des comparaisons string recopiées partout.

## Acceptance

- création manuelle dans explorateur visible dans l'app ;
- rename externe conserve relations lorsque possible ;
- overwrite concurrent détecté ;
- crash pendant save ;
- watcher overflow ;
- symlink/path escape ;
- changement de vault invalide proprement handles précédents.