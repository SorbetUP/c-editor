# TODO — Accessibilité, clavier, touch et gestes

## Objectif

Les extensions et refontes ne doivent pas sacrifier les interactions fondamentales. Définir des contrats host plutôt que corriger écran par écran.

## P0 — Semantics

Toute action UI a nom accessible, rôle, focus visible et état. Les contributions addon utilisent des primitives host qui héritent de ces garanties.

## P0 — Keyboard

Ordre de tab cohérent, shortcuts découvrables, escape/back semantics et command palette. L'éditeur ne vole pas des raccourcis globaux hors contexte.

## P0 — Touch/mobile

Targets suffisants, edge swipe sidebar sans conflit avec scroll/editor, long-press/context menu, keyboard avoidance et safe areas. Tester avec clavier logiciel réel.

## P1 — Reduced motion/themes

Respect prefers-reduced-motion, contrast, zoom/font scale et thèmes. Addons utilisent tokens, pas couleurs hardcodées.

## P1 — Screen readers

Sidebar, tabs, dialogs, settings, proposal cards et graph ont structure accessible. Une alternative liste/voisinage est nécessaire pour les canvas graph.

## P1 — Drag/drop alternatives

Toute action uniquement drag/drop possède commande/menu équivalent pour clavier/mobile.

## Validation

- Parcours create/open/edit/settings/proposal possible au clavier.
- Sidebar mobile ouvre depuis bord de façon fiable.
- Proposal annonce cible/risque/action aux technologies assistives.
- Themes addon respectent contraste/font scale.