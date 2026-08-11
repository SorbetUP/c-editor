# 14 — Données, relations et query primitives

## Pourquoi

Projects, Serie, Analytics, Graph, GitHub et Calendar ont tous besoin de lier des objets. Si chaque addon invente ses IDs/backlinks, les requêtes transverses deviennent impossibles ou nécessitent des scans complets.

## Core minimal

Le host devrait fournir :

- `ResourceId` pour ressources internes ;
- `ExternalEntityRef { provider, type, id }` ;
- typed relations namespacées ;
- provenance ;
- query/filter primitive tenant compte workspace/permissions ;
- change stream.

Le host n'a pas besoin de connaître `Task.priority` ou `PullRequest.reviewState` ; ces schemas restent addons.

## Relation ownership

Chaque relation indique owner namespace. Supprimer un addon ne doit pas supprimer automatiquement les documents sources ; ses relations dérivées peuvent être reconstruisibles ou archivées selon type.

## Query

Filters structurés + pagination + sorting + projection. Éviter d'exposer directement SQL sur tables internes du host comme contrat public. Un addon Analytics avancé peut compiler un DSL stable vers son moteur.

## Provenance

Distinguer user-authored, imported, inferred, calculated. Une relation IA n'est pas équivalente à un lien explicite.

## Authorization

Filtrer avant retour des résultats. Ne jamais retourner des IDs interdits en comptant sur l'UI pour les cacher.

## Acceptance

Rename stable, external id collision namespace, addon uninstall/reinstall, workspace filtering, pagination stable, provenance et query performance benchmark.