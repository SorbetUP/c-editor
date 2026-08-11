# 15 — Accessibilité, navigation et input

## Direction

Keyboard, touch, mouse/stylus et assistive technologies doivent activer les mêmes commandes métier. Une feature ne doit pas exister uniquement via hover ou geste non découvrable.

## Focus/navigation

- focus order explicite ;
- visible focus ring ;
- Escape semantics cohérentes ;
- route change place le focus correctement ;
- dialogs trap focus et restaurent au close ;
- command palette accessible clavier.

## Sidebar/tree

Arrow keys pour naviguer/déplier, Enter pour ouvrir, context action accessible. Mobile : tap pour ouvrir, disclosure distinct pour déplier dossier si nécessaire afin d'éviter ambiguïté.

## Gestures

Gestes rapides restent des raccourcis ; fournir un bouton/action équivalent. Documenter conflicts avec OS back gesture et editor selection.

## Screen readers

Labels sur icon-only controls, états expanded/collapsed/selected, progress jobs, notifications et editor toolbar. Les graphes avancés nécessitent une vue liste/table alternative pour les informations essentielles.

## Motion

Respect reduced-motion. Les animations sidebar/topbar doivent rester fonctionnelles si désactivées.

## Internationalisation

Éviter layout dépendant de labels anglais courts. Direction/locale et formats date/heure passent par helpers communs.

## Acceptance

Audit clavier sans souris, TalkBack/VoiceOver scénarios P0, zoom/font scaling, reduced-motion et tests des touch targets.