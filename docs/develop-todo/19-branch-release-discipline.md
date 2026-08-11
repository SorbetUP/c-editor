# 19 — Discipline de branches et releases

## Constat

Le repository possède des branches de travail fortement divergentes. Une branche avec des centaines de commits d'écart ne peut pas être traitée comme un simple feature branch à merger aveuglément : cela réintroduit facilement workflows, APIs ou fixes obsolètes.

## Proposition

Définir clairement :

- `main` : état release/intégration stable selon politique choisie ;
- `develop` si réellement utilisé : future integration line ;
- feature branches courtes ;
- branches historiques archivées après extraction des commits utiles ;
- `develop_todo` : documentation/propositions, pas cible de release automatique.

## Integration ledger

Pour chaque grande branche divergente : objectif, commits uniques utiles, composants touchés, tests requis, décision `merge/cherry-pick/reimplement/drop`. Ne jamais merger 500+ commits uniquement parce qu'une branche est « en avance ».

## Addon snapshot

ElephantNote doit enregistrer explicitement le commit/catalog version Elephant-Addons utilisé par une release. CI vérifie que les packages distribués correspondent à ce snapshot et que le catalog n'est pas un mélange de versions.

## Release candidates

Créer un commit/tag RC immuable, construire les artefacts depuis lui, exécuter les tests de packaging puis promouvoir exactement les mêmes artefacts ou le même commit. Éviter de rebuild depuis une branche qui a bougé entre test et release.

## Dependency updates

Dependabot/major upgrades passent isolément avec tests ; ne pas les absorber dans un merge de consolidation sans revue spécifique.

## Acceptance

- aucune release depuis worktree dirty ;
- exact source SHA dans artefact/version info ;
- addon snapshot vérifié ;
- branch integration ledger à jour ;
- rollback vers release précédente documenté ;
- branches obsolètes ne bloquent pas la lecture de l'état courant.