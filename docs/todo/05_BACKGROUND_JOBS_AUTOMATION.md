# TODO — Background jobs, scheduling et automation

## Problème

Indexation Knowledge, embeddings, téléchargement de modèles, sync, imports, builds Sites et connecteurs dépassent le lifecycle d'une vue. Les laisser comme promises/timers du renderer fragilise cancellation, reprise et crash recovery.

## P0 — Job service durable

Créer jobs avec `id`, owner addon, type/version, payload validé, status, progress, timestamps, retry policy, cancellation, checkpoint et result/error. Les jobs longs survivent au changement de vue ; certains reprennent après restart.

## P0 — Classes

Distinguer `foreground`, `background`, `scheduled`, `condition/event triggered`. Une opération interactive peut afficher une progress UI tout en utilisant le même service.

## P0 — Idempotence

Chaque external write/mutation a une idempotency key ou stratégie documentée. Retry/restart ne doit pas envoyer deux mails, créer deux issues ou dupliquer un import.

## P0 — Cancellation réelle

Le cancel descend jusqu'au provider/transport/subprocess et libère les ressources. `cancel requested` n'est pas `cancelled` tant que le backend ne le confirme pas.

## P1 — Scheduler

Recurrence avec timezone, missed-run policy, overlap, max concurrency et pause. Les schedules appartiennent à un addon/workspace et sont désactivés si l'addon disparaît.

## P1 — Event triggers

Souscriptions document/object avec debounce, dedup et cursor durable. Remplacer les pollings locaux quand le host peut émettre un événement fiable.

## P1 — UI et ressources

Vue compacte running/queued/failed/recent, progress/logs, retry/cancel. Budgets CPU/RAM/network et backpressure : un rebuild embeddings ne doit pas rendre l'éditeur inutilisable.

## Validation

- Download/indexation continue si vue fermée.
- Restart reprend ou échoue explicitement selon capability.
- Retry external write idempotent.
- Unload addon sans timer/job non autorisé.
- Cancel prouvé sur vrai subprocess et transport réseau.