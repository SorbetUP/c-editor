# TODO — Capabilities, tools, policies et proposals

## Pourquoi dans le host

Le futur agent ne doit pas connaître une liste codée en dur d'addons. À l'inverse, laisser chaque addon implémenter sa confirmation de sécurité produit des policies incohérentes. ElephantNote doit fournir registre, validation et gouvernance ; les addons fournissent les handlers métier.

## P0 — Tool registry

Un addon publie un tool versionné avec `name`, description, input/output JSON Schema, owner, permissions, risk class, timeout, cancellable, idempotency et availability. Noms stables : `knowledge.search`, `documents.read`, `calendar.events.create`, `code.execute`.

## P0 — Risk classes

- `read`: aucune mutation ;
- `compute`: calcul local sans effet durable ;
- `draft`: sortie non envoyée/non appliquée ;
- `write`: mutation locale ;
- `external_write`: mutation/envoi externe ;
- `execute`: code/processus ;
- `destructive`: suppression/overwrite irréversible.

Le modèle ne peut jamais réduire la classe déclarée par le tool.

## P0 — Proposal service

Pour mutation gouvernée : `intent -> proposal -> validate -> preview -> approve/reject -> apply`. La proposal contient targets, expected revisions, diff/payload, sources, risk, owner, expiration et status.

## P0 — Apply transactionnel

Au clic approve, revérifier permissions et preconditions. Une proposal stale devient `stale` et exige régénération/rebase ; aucun apply silencieux.

## P0 — Policy host

Décision = risk tool + permissions addon + préférences + règles workspace + contexte. Autoriser éventuellement read/compute pour session, jamais une règle de privilège créée par le LLM.

## P1 — Audit et UI commune

Journaliser metadata d'exécution sans contenu sensible complet. Cards communes affichent cible, raison, diff, sources et risque. Une preview addon spécialisée ne peut masquer ces données host.

## P1 — MCP/agents externes

Tous les bridges utilisent le même registre et la même policy. Aucun protocole externe ne contourne approvals/permissions.

## Validation

- Tool inconnu/arguments invalides : aucune exécution.
- Note modifiée depuis génération : proposal stale.
- `code.execute`/`mail.send` ne peuvent pas être auto-approuvés par prompt.
- Un addon désinstallé rend ses proposals inapplicables mais garde l'historique.
- Le Chat découvre un nouveau tool sans modification de son code.