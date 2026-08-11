# TODO — Branches, pinning Elephant-Addons et release engineering

## Problème observé

ElephantNote peut pinner un SHA d'Elephant-Addons différent du `main` courant, et les branches d'intégration peuvent diverger fortement. Il devient facile de croire qu'une correction addon est embarquée alors que le build consomme un autre commit.

## P0 — Source of truth

Définir clairement branche produit, branche intégration addons, SHA addon validé, version pack et release. Un fichier machine-readable unique référence repo+SHA+catalog version.

## P0 — Pin update workflow

Commande dédiée : fetch SHA, vérifie catalog/pack, construit services natifs nécessaires, exécute guards puis acceptance addon. Le commit de pin inclut ancien/nouveau SHA et résumé du diff.

## P0 — Drift guard

CI signale pin inconnu, SHA non reachable, catalog/package mismatch, snapshot non validé ou `develop_todo` accidentellement utilisé pour une release.

## P1 — Compatibility matrix

Chaque addon déclare min/max API, packages plateforme et migration version. Le host produit une matrice avant release et refuse un pack incohérent.

## P1 — Branch hygiene

Réduire les branches géantes divergentes : intégrer par séries cohérentes, conserver provenance, supprimer branches obsolètes seulement après validation.

## P1 — Reproducibility

Build release reconstruit exactement le set d'addons à partir de SHAs/hashes, sans dépendre d'un `main` mutable.

## Validation

- À partir d'un binaire, retrouver exactement le SHA Elephant-Addons embarqué.
- CI échoue si catalog/packages divergent.
- Update pin produit diff et test report clairs.
- Aucun merge aveugle d'une branche historique ne réintroduit un runtime obsolète.