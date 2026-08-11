# 16 — Crash recovery et état durable

## Objectif

Un crash renderer, service addon ou process complet ne doit pas transformer un incident temporaire en perte de travail ou état incohérent.

## Editor recovery

Conserver un journal/checkpoint minimal des edits non confirmés sur disque. Au restart, comparer version disque et recovery entry ; proposer récupération si divergence, ne jamais écraser automatiquement une version plus récente.

## Atomic state

Settings, addon registry, job checkpoints, sync baselines et autres petits états critiques utilisent atomic replace/checksum/version. Une écriture partielle doit être détectable.

## Service crash

Addon service : health state `error`, cleanup handles, restart policy bornée. Après N crashes rapprochés, désactiver auto-restart et proposer diagnostics plutôt qu'une crash loop.

## Renderer crash

Le host conserve jobs/services indépendants lorsque sûr. Une nouvelle UI se resubscribe à leurs snapshots.

## Update interruption

Addon/app update doit staging + verify + atomic activate. Garder version précédente tant que nouvelle n'est pas validée. Ne pas laisser un package half-extracted actif.

## Disk full / permission loss

Save échoue explicitement ; document reste dirty ; retry possible après résolution. Ne jamais afficher `Saved` si fs write a échoué.

## Recovery UI

Au démarrage après crash : message minimal avec Recover/Inspect/Discard selon artefact. Ne pas imposer une modale pour un service addon non critique si l'app peut fonctionner dégradée.

## Acceptance

Kill process pendant save/settings/update/sync apply, disk full, revoked filesystem permission, addon crash loop, renderer reload avec job actif et recovery note concurrente.