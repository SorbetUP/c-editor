# 00 — Priorités et invariants du host

## Priorités P0

1. **Intégrité du vault** : aucune opération UI/addon/sync ne doit perdre ou écraser silencieusement une donnée utilisateur.
2. **Runtime addons réellement sandboxé et versionné** : capabilities, permissions, services et lifecycle prévisibles.
3. **Identité stable des ressources** : les addons ne peuvent pas bâtir relations/provenance fiables uniquement sur des paths mutables.
4. **Event bus + jobs durables** : indexation, sync, downloads et automations ne doivent pas dépendre de la durée de vie d'un composant renderer.
5. **Parité fonctionnelle desktop/mobile** pour les primitives core : choisir vault, lire/écrire/rename/delete, attachments, drag/share, settings, addon permissions.
6. **Recovery** : autosave, crash, process restart, update interruption et addon failure doivent être récupérables.

## Invariants

- Une API refusée doit échouer explicitement, jamais simuler un succès.
- Une capability non disponible sur une plateforme est `unsupported`, jamais no-op.
- Toute mutation possède cible + precondition/version lorsque le risque de stale write existe.
- Les secrets ne transitent pas dans les settings JSON ordinaires ni les logs.
- Le renderer ne reçoit pas un accès filesystem général.
- Les addons n'accèdent qu'aux resources/scopes accordés.
- Le core ne dépend pas d'un provider commercial spécifique.
- Toute fonctionnalité déclarée supportée possède un test indépendant correspondant.
- Les caches/index sont reconstructibles ; le vault reste la source de vérité pour les documents.

## P1

- UI contribution system générique.
- Query/relations primitives.
- Connector lifecycle commun.
- File-handler registry.
- Workspace/identity primitives pour Serie.
- Observabilité corrélée host/addon/service.

## Méthode de validation

Chaque grande évolution doit fournir : tests unitaires du contrat, integration test host↔addon, test E2E utilisateur, test de panne/annulation et matrice plateforme. Les smoke tests seuls ne suffisent pas.

## Ce qu'il faut éviter

- déplacer une fonctionnalité dans un addon sans lui fournir l'API host nécessaire ;
- multiplier des IPC commands spécifiques par addon ;
- dupliquer secrets/jobs/file watchers dans chaque package ;
- ajouter des fallbacks silencieux pour faire passer les tests ;
- considérer « ça build » comme preuve de fonctionnement.