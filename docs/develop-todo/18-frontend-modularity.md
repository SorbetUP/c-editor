# 18 — Modularité frontend / app shell

## Direction

Le renderer doit séparer app shell, domain state, editor et addon contributions. Les composants ne doivent pas devenir des service locators accédant directement à Tauri/IPC partout.

## Layers

- `platform clients` : wrappers typed des host APIs ;
- `domain services/stores` : vault, navigation, document session, settings, addons ;
- `app shell` : layout/routes/sidebar/topbar ;
- `feature modules` ;
- `addon contribution host` ;
- `presentational components`.

## State ownership

Document content/session state différent de global UI state. Éviter un store global contenant contenu complet de toutes les notes ouvertes si inutile. Les addons reçoivent selectors/events bornés, pas accès au store privé.

## Async

Toute action async possède lifecycle pending/success/error/cancelled. Éviter fire-and-forget pour saves, vault operations et permission requests.

## Error boundaries

Niveau route, addon contribution et editor embed. Une erreur widget ne démonte pas l'éditeur. Les erreurs host restent typées jusqu'à l'UI.

## Performance

- imports lazy pour grosses surfaces ;
- éviter re-render global sur chaque frappe ;
- profile sidebar/file tree ;
- virtualisation uniquement si nécessaire ;
- nettoyage subscriptions lors unmount/vault switch.

## CSS/theme

Design tokens centralisés ; composants addon utilisent primitives publiques. Pas de sélecteurs dépendant de structure DOM privée.

## Acceptance

Tests de dependency boundaries, no direct IPC hors layer autorisée, render-count benchmarks editor, memory after route churn et addon crash isolation.