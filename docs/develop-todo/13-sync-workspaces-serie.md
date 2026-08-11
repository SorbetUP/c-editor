# 13 — Sync, workspaces et préparation Serie

## Direction

Elephant de base reste local-first et simple, mais certaines primitives doivent permettre à Serie d'ajouter des espaces organisationnels sans fork profond.

## Workspace primitive

Introduire un contexte `WorkspaceId` distinct de `VaultId` si nécessaire : un workspace représente frontière d'identité/permissions ; un vault représente stockage/document set. Une installation personnelle peut n'avoir qu'un workspace implicite.

## Isolation

Personal et organisation doivent être séparés par policy de données. Search, Knowledge, agent et connectors reçoivent toujours un workspace scope calculé par le host. Aucun « global search » ne traverse les espaces sans permission explicite.

## Principals

Préparer abstractions `Principal`, `Membership`, `Role/Grant` sans imposer une UI enterprise au core personnel. Le runtime addon peut demander `currentPrincipal/currentWorkspace` et le host applique authorization.

## Sync abstraction

Le core expose change journal/resource versions ; `elephant.sync` choisit transport P2P. Une variante Serie peut utiliser cloud/server sans changer les APIs des addons métier.

## Packs

Supporter installation transactionnelle d'un pack : liste addons + versions + config initiale + schemas/templates. Le pack ne devient pas propriétaire opaque des données.

## Audit

Mutations de workspace partagé peuvent alimenter un audit log selon policy. Personal vault ne nécessite pas forcément le même niveau de conservation.

## Acceptance

- aucun leak personal->org via query/agent ;
- changement de rôle prend effet sur tools et UI ;
- offline local edits puis reconnect ;
- pack upgrade ;
- export workspace ;
- addon commun fonctionne en personal et org sans branche métier spécifique.