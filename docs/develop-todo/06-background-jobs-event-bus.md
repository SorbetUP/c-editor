# 06 — Background jobs et event bus

## Pourquoi

Sync, Knowledge rebuild, model downloads, embeddings, imports, connectors et Automation ne doivent pas mourir parce qu'une route React/Vue est démontée ou qu'une WebView est recréée.

## Job model

`Job { id, ownerAddon, type, state, progress, createdAt, startedAt, finishedAt, cancellable, resumePolicy, error }`.

États : queued/running/paused/succeeded/failed/cancelled/interrupted.

## Durabilité

Les jobs longs persistables enregistrent un checkpoint minimal. Au restart, le runtime décide selon `resumePolicy`: resume, restart, mark interrupted. Aucun job ne doit se réexécuter aveuglément et dupliquer une mutation.

## Progress/event stream

UI s'abonne par job id. Éviter polling. Les events ont séquence monotone ; un subscriber tardif peut lire snapshot courant puis deltas.

## Event bus durable vs ephemeral

- UI events éphémères : sélection, hover, route ;
- domain events durables : file changed, sync applied, task changed, connector item received.

Les automations reposent uniquement sur des events ayant des semantics/idempotency documentées.

## Scheduling

Support one-shot, delayed, recurring et retry-backoff. Mobile doit respecter contraintes OS/batterie ; ne pas promettre une exécution exacte impossible en background.

## Ownership

Disable/uninstall addon : jobs correspondants stoppés/archivés selon policy. Un addon ne peut pas annuler les jobs d'un autre sans capability dédiée.

## Acceptance

- restart pendant download/rebuild ;
- cancel race ;
- duplicate event/idempotency ;
- subscriber reconnect ;
- disabled addon ;
- mobile suspension ;
- job failure visible dans UI sans terminal.