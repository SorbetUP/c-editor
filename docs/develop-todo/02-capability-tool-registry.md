# 02 — Capability / Tool Registry

## Pourquoi

Le futur agent, Automation, Dashboard et autres addons doivent découvrir ce qui est installé sans connaître des IDs hardcodés. Le même mécanisme doit aussi servir aux humains via command palette et UI contributions.

## Capability descriptor

Champs minimaux :

- stable id + version ;
- provider addon ;
- methods ;
- JSON Schema input/output ;
- side effects/risk class ;
- permissions requises ;
- supported platforms ;
- cancellable/streaming ;
- idempotent oui/non ;
- timeout/default limits ;
- availability/health.

## Tool projection

Une capability peut fournir une projection `agentTool`, mais toute resource n'a pas besoin d'être exposée à l'IA. Le host/policy construit l'allowlist effective ; l'agent ne voit jamais les tools auxquels il n'a pas droit.

## Risk classes

`read`, `compute`, `draft`, `write`, `external_write`, `execute`, `destructive`.

Le provider déclare le risque minimal ; le consommateur ne peut pas le réduire.

## Discovery

- query par capability/type/version ;
- événements when-added/removed/health-changed ;
- cache invalidé lors enable/disable/update ;
- conflit de providers résolu par routing/preference, pas ordre de chargement aléatoire.

## Human UI

Le même registry peut alimenter : command palette, context menus, Open With, Dashboard widget catalog, agent tool inspector. Les labels UI sont séparés des IDs stables.

## Acceptance

- un addon installé après ouverture devient découvrable sans restart complet lorsque supporté ;
- disable retire immédiatement ses capabilities ;
- schema validation avant appel ;
- risk class non falsifiable par consommateur ;
- version mismatch explicite ;
- aucun tool non accordé envoyé au modèle.