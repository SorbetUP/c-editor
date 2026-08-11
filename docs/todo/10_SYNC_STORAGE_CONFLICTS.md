# TODO — Sync, storage addon et conflits

## Problème

Les addons introduisent fichiers source, metadata utilisateur, config, secrets, caches, indexes, sessions et jobs. Sync doit distinguer ce qui a du sens sur un autre appareil.

## P0 — Storage classification

Chaque namespace déclare `source`, `user-data`, `syncable-config`, `derived`, `secret`, `device-local`, `ephemeral`. Defaults sûrs : secret/device-local/derived ne synchronisent pas.

## P0 — Revisions/tombstones

Sync utilise revisions et tombstones pour rename/delete/edit. Les conflits sont explicites ; pas de last-write-wins silencieux.

## P0 — Atomic apply

Download vers temp, hash/espace disque, replace atomique. Crash entre download/apply ne laisse jamais un fichier partiel canonique.

## P0 — Conflict surface

API host pour créer conflit, afficher base/local/remote, garder les deux, resolve/merge. Un addon peut apporter un merge spécialisé mais ne peut perdre une version.

## P1 — Addon absent

Recevoir user-data syncable d'un addon non installé doit les conserver sans exécuter de code. Installation ultérieure peut migrer le schema.

## P1 — Quotas

Gros assets/cache/historique ont quotas et cleanup. Sync expose bytes pending et ne monopolise pas réseau/IO.

## P1 — E2EE/keys

Si confidentialité E2E promise : modèle de clés, pairing, rotation, revoke et recovery documentés. Aucun secret provider dans sync.

## Validation

- edit/edit, rename/edit, delete/edit concurrent sans perte.
- cache Knowledge non transféré.
- config syncable transférée.
- secret provider jamais transféré.
- crash pendant apply récupérable.
- uninstall/reinstall conserve user-data selon policy.