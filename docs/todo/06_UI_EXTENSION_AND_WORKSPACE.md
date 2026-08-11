# TODO — UI extension points et workspace

## Problème

Les addons doivent enrichir sidebar, settings, dashboard, éditeur et views sans injection DOM, dépendance Vue privée ni copie de composants core.

## P0 — Extension points sémantiques

Stabiliser slots : sidebar header/tree/after-tree/footer, topbar, statusbar, command palette, settings sections/pages, workspace views/panels, dashboard widgets, context menus et document toolbar. Chaque slot définit layout, lifecycle, mobile behavior, accessibility et ordre. Aucun sélecteur CSS ne doit servir d'API.

## P0 — Design primitives

Exposer tokens/primitives host : buttons, fields, cards, list rows, empty/error/progress, icons, dialogs, menus. Un addon n'importe pas un composant interne non versionné et ne copie pas le CSS de l'app.

## P0 — Navigation

`workspace.openDocument(handle)`, `openView(id, state)`, back/forward/deep-link et restore session. Aucun addon ne manipule directement le store des tabs.

## P1 — Responsive contracts

Une contribution déclare desktop/mobile : full view, drawer, sheet, hidden ou alternative compacte. Le host dégrade proprement une contribution non supportée.

## P1 — State restoration

Persister état de view via schema/version addon. Ne jamais restaurer un route/view d'addon absent comme si elle existait.

## P1 — Error isolation

Exception d'un widget/panel = placeholder d'erreur, log corrélé, retry. Ne pas faire tomber toute sidebar/settings.

## P1 — Performance

Aucun render polling. Updates via événements/resources. Mesurer mount/unmount et subscriptions actives.

## Validation

- Dashboard/Recently Edited sans `pinia._s`.
- Aucun MutationObserver pour injecter une feature settings/sidebar.
- Disable retire immédiatement la UI et restaure layout.
- Keyboard/screen reader corrects.
- Fallback mobile défini pour chaque view officielle.