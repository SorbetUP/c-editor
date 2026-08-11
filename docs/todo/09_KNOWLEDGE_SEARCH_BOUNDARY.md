# TODO — Frontière core / Knowledge / Search

## Décision

Le core ne doit pas devenir un vector database. Il fournit documents, events, identities, jobs et stockage scoped. `elephant.knowledge` possède la projection dérivée ; `ai-search` orchestre le retrieval.

## P0 — Event source fiable

Knowledge consomme le journal `documents` avec revisions. Pas de rescan complet requis pour une petite édition.

## P0 — Derived storage

Storage/cache pour indexes avec quota, schema/version, cleanup et classification `derived/rebuildable`. Sync ignore ces données par défaut.

## P0 — Search API claire

L'API core `search.query` doit avoir une frontière nette : recherche simple core ou routage vers provider enregistré. Ne pas maintenir deux systèmes sémantiques concurrents aux résultats incompatibles.

## P1 — Query providers

Contributions `search.providers` / `knowledge.provider` avec capabilities lexical/semantic/hybrid/filter. Les résultats exposent leur provenance/engine.

## P1 — Citation anchors

Hits avec document/block handles, excerpt, score components et revision afin d'ouvrir/surligner la source exacte.

## P1 — Staleness

Status index building/stale/degraded. Une recherche ne prétend pas être exhaustive si l'index est partiel.

## P2 — External objects

Les addons peuvent publier des objets indexables normalisés sans les convertir en faux Markdown. Knowledge garde provenance et ACL.

## Validation

- Edit note met Search à jour sans rebuild global.
- Vectors non synchronisés comme source.
- Résultat indique engine/provider/revision.
- Provider Knowledge retiré : dégradation propre.
- Aucun cap silencieux de documents imposé par le core.