# TODO — Performance, observabilité et budgets

## Problème

Avec de nombreux addons, polling, full-vault scans, MutationObservers, rebuilds et services natifs s'additionnent. Il faut pouvoir attribuer le coût au bon sous-système.

## P0 — Correlation IDs

Toute action importante reçoit un ID propagé renderer -> host -> addon -> native service/provider. Les logs permettent de reconstruire le chemin sans exposer le contenu privé.

## P0 — Structured logs

Champs : timestamp, level, subsystem/addon, operation, request/job ID, duration, state transition, error code/chain. Redaction centralisée.

## P0 — Resource accounting

Mesurer timers/subscriptions/jobs, service processes, CPU approximatif, mémoire si disponible, network bytes et slow callbacks par addon. Diagnostics locaux ; pas de télémétrie externe obligatoire.

## P0 — Performance budgets

Budgets explicites pour startup, open note, save latency, editor input, sidebar, graph 1k/10k, search et addon activation. Différenciation possible par device class.

## P1 — Trace view

Mode diagnostics : timeline appels/jobs/IPC/addons/logs/errors et export d'un bundle sanitised.

## P1 — Polling detector

Guard/dev diagnostics pour intervals fréquents et full scans répétés. Favoriser events/reactive resources.

## P1 — Degraded mode

Si un addon crash-loop ou consomme excessivement, le host peut le suspendre avec raison et bouton réactiver sans tuer l'éditeur.

## Validation

- Identifier quel addon provoque hausse CPU.
- Une action Chat/tool est traçable end-to-end.
- Aucun secret dans bundle diagnostics.
- Recently Edited sans timer périodique.
- Graph/index lourd n'empêche pas saisie/sauvegarde.