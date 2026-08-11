# 05 — UI contribution system

## Objectif

Permettre aux addons d'intégrer une UI native/cohérente sans leur donner la responsabilité de patcher directement l'arbre global de composants.

## Contribution points

- icon rail ;
- sidebar sections ;
- routes/pages ;
- editor toolbar/context actions ;
- file context menu ;
- settings sections ;
- command palette ;
- dashboard widgets ;
- status/topbar ;
- viewer/file handlers ;
- inspector/right panel ;
- notifications/inbox.

## Contrat

Chaque contribution déclare id stable, emplacement, ordre/priorité bornée, icon/label, condition de visibilité, command/action associée, permissions et cleanup lifecycle.

## Design tokens

Exposer tokens theme, typography, spacing, radius, elevation et semantic colors. Éviter les CSS copiés ou les dépendances au DOM privé d'Elephant.

## Isolation

Un composant addon qui throw ne doit pas casser l'app shell. Error boundary + disable/reload action. Les addons ne doivent pas pouvoir masquer des contrôles sécurité/permission host-owned.

## Responsive

Les contributions doivent déclarer comportement compact/mobile. Pas de hover-only. Touch targets et safe areas gérés par primitives communes.

## Navigation

Routes addon avec back/forward/deep-link. L'addon ne manipule pas directement l'history global sans passer par router API.

## Acceptance

- addon enable/disable ajoute/retire UI sans restart lorsque possible ;
- collision d'IDs détectée ;
- thème light/dark ;
- mobile/desktop ;
- crash contribution isolé ;
- keyboard navigation ;
- aucune UI de permission sensible remplaçable par addon.