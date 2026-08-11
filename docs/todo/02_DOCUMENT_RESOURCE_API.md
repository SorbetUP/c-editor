# TODO — Documents, ressources et identité stable

## Problème

L'API publique actuelle est largement path-based (`notes.read`, `notes.write`, `entries.move`, etc.). Les paths restent utiles à l'utilisateur mais sont une mauvaise identité : rename/move invalident citations, proposals, jobs et références d'addons.

## P0 — Resource handle

Introduire un `DocumentHandle` opaque/sérialisable avec au minimum `id`, `vaultId`, `path`, `kind`, `revision`, `contentHash`, `modifiedAt`. Le host résout le path actuel depuis l'ID.

## P0 — Compare-and-swap

`documents.write(handle, content, { expectedRevision })` refuse avec `STALE_REVISION` si la cible a changé. Même règle pour rename/move/delete. Cela protège proposals IA, sync et automatisations.

## P0 — Event journal

Événements ordonnés/idempotents : created, contentChanged, metadataChanged, moved, renamed, deleted/restored, avec revision avant/après. Knowledge/Search/Graph consomment ce journal au lieu de rescanner le vault.

## P0 — Batch

Planifier plusieurs opérations, valider toutes les preconditions puis appliquer atomiquement lorsque possible. Si le backend ne peut pas assurer l'atomicité globale, retourner un journal exact et stratégie de compensation.

## P1 — Block anchors

`BlockHandle` ou ancre stable/réconciliable pour citations, annotations, tasks, Anki et outputs : document ID, block ID, revision et fallback hash/contexte.

## P1 — Documents génériques

Ne pas réduire tout à `note`: markdown, text, image, PDF, binary, Excalidraw et types fournis par file handlers. Le core gère identité/stockage/ouverture ; la logique spécialisée reste addon.

## P1 — Watch API

`documents.watch(scope)` avec filtres, coalescing et cursor de reprise. Pas de watcher filesystem arbitraire exposé aux addons.

## P1 — Corbeille

Delete réversible selon policy : trash/tombstone puis purge explicite. Les consommateurs savent distinguer deleted/restored/purged.

## Validation

- Citation toujours résolvable après rename/move.
- Proposal revision N ne peut écraser N+1.
- Une édition produit une mise à jour Knowledge incrémentale, pas un rescan.
- Delete/restore reflété dans search/graph/sync.
- Compatibilité temporaire avec les appels path historiques.