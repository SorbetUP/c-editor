# 11 — Performance et observabilité

## Objectif

Diagnostiquer une app lente ou un addon bloqué sans reproduire au hasard. Les logs doivent être corrélés, les performances mesurées par workflow et les métriques ne doivent pas exposer de contenu utilisateur.

## Correlation

Chaque opération longue reçoit `traceId`; addon call, IPC, native service et renderer events reprennent cet ID. Les erreurs UI peuvent afficher un diagnostic ID copiable.

## Structured logs

Champs : timestamp, level, subsystem/addon, traceId, operation, duration, result/error class. Redaction centralisée pour tokens, auth headers, contenu note et paths si nécessaire.

## Metrics locales

- startup phases ;
- vault open ;
- note open ;
- editor input latency ;
- save latency ;
- addon start ;
- IPC latency ;
- memory RSS/renderer ;
- graph/search/query latency ;
- sync throughput ;
- AI TTFT/tokens/s lorsque disponible.

## Performance budgets

Fixer seuils de non-régression par scénario sur hardware de référence, plutôt qu'un unique benchmark synthétique. Comparer p50/p95/p99 lorsque pertinent.

## Profiling mode

Mode développeur activable donnant timeline des opérations et ressources addons. Pas de profiler lourd permanent en production.

## UI

Settings > Diagnostics : version app/platform, enabled addons, health, jobs, recent typed errors, export logs redacted. Ne pas obliger l'utilisateur à lancer depuis terminal pour connaître un crash addon.

## Acceptance

- trace host↔addon↔service ;
- redaction tests ;
- startup benchmark ;
- memory leak repeated note open/close ;
- stalled job diagnostic ;
- aucune télémétrie externe sans consentement/policy explicite.