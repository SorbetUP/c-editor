# ElephantNote — develop_todo

Ce dossier décrit les améliorations proposées pour l'application hôte. Il complète les TODO de `Elephant-Addons/develop_todo`.

## Règle de conception

Le core Elephant doit fournir des **primitives stables**, pas absorber les domaines des addons. Une demande répétée par plusieurs addons (jobs, secrets, file identity, proposals, UI slots, connectors, etc.) est un bon candidat au host. Une logique spécifique à Calendar, GitHub, AI ou Projects reste dans l'addon.

## Documents

1. `00-priorities-and-invariants.md`
2. `01-addon-runtime-contract.md`
3. `02-capability-tool-registry.md`
4. `03-permissions-proposals-security.md`
5. `04-vault-storage-identity-events.md`
6. `05-ui-contribution-system.md`
7. `06-background-jobs-event-bus.md`
8. `07-settings-connections-secrets.md`
9. `08-editor-muya-reliability.md`
10. `09-file-handlers-drag-drop.md`
11. `10-mobile-desktop-parity.md`
12. `11-performance-observability.md`
13. `12-testing-release-engineering.md`
14. `13-sync-workspaces-serie.md`
15. `14-data-query-relations.md`
16. `15-accessibility-navigation-input.md`
17. `16-crash-recovery-state.md`
18. `17-api-versioning-sdk.md`
19. `18-frontend-modularity.md`
20. `19-branch-release-discipline.md`

## Classification

Chaque document distingue autant que possible :

- **Invariant** : propriété qui doit toujours être vraie ;
- **Proposition** : choix d'architecture à implémenter/benchmarker ;
- **Pourquoi** : risque que l'on évite ;
- **Acceptance** : preuve minimale requise ;
- **Impact addons** : APIs débloquées pour le repo Elephant-Addons.

Cette branche est documentaire : la présence d'un TODO ne signifie pas que la fonctionnalité existe déjà.