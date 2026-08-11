# TODO — Spaces, ACL, provenance et fondations organisationnelles

## Motivation

Des packs enterprise comme Serie auront besoin d'équipes, rôles, objets externes, partage et audit. Mettre tout le métier dans le core serait excessif ; certaines primitives de sécurité/identité doivent néanmoins être host-level pour éviter un ACL différent par addon.

## P0 — Spaces

Concept générique de scope : personal/private, vault/workspace, shared/team/organization. Le mode local simple continue à fonctionner sans serveur ni organisation.

## P0 — Subject/role primitives

Identités utilisateur/device/service/addon et rôles/scopes. Permission d'un addon et ACL d'un objet sont distinctes ; les deux doivent être satisfaites.

## P0 — Provenance

Contrat source : `originType`, provider/addon, externalId/URL, capturedAt, sourceRevision/hash, parent evidence. Classification `fact`, `inference`, `proposal` pour les objets dérivés.

## P0 — Audit

Mutation partagée/externe enregistre actor, target, before/after refs, approval, timestamp et correlation ID. Audit append-only selon policy et exportable.

## P1 — Generic object identity

Le host fournit IDs/handles et ACL/provenance ; schemas métier comme client, produit, requirement ou employé restent dans addons/ontology layer.

## P1 — Private/shared separation

Une ressource privée ne doit pas être indexée/exportée/synchronisée vers un space partagé sans action explicite. Changer de scope est une mutation sensible avec preview des dépendances.

## P1 — Server optionality

Les primitives fonctionnent localement. Auth serveur, ACL distantes et webhooks peuvent être apportés par un backend enterprise sans contaminer le core offline.

## Validation

- Un pack enterprise représente besoin->requirement->issue->commit->test sans imposer ces types au core.
- Donnée personnelle absente de recherche partagée sans grant.
- Toute inference garde ses sources et peut devenir stale.
- ACL indépendant d'une convention de dossier.