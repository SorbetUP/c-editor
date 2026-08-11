# Migration Elephant vers Freya

Cette branche contient la première tranche native vérifiable de la migration :

- `Elephant/freya/` est un binaire Freya séparé, sans modification du chemin Tauri/Vue par défaut ;
- le shell natif charge un vault réel via `ELEPHANT_FREYA_VAULT` et applique les règles de visibilité existantes ;
- `freya-testing` couvre le rendu headless, l’état de navigation et le chargement du vault ;
- `tools/vue-to-freya/` produit une analyse AST des SFC Vue et signale explicitement les constructions qui exigent un port manuel.

Le travail n’est pas une migration complète. L’éditeur Muya, la recherche sémantique, le graphe, les réglages persistants, les add-ons, Excalidraw, la synchronisation, Android et le packaging Freya restent à porter et à prouver. Le frontend Tauri/Vue reste l’oracle fonctionnel jusqu’à preuve de parité.

Statut de livraison de cette tranche : `PARTIALLY PROVEN`. Le shell Freya compile, s’exécute réellement sur macOS et passe ses tests headless ; la parité visuelle et fonctionnelle complète demandée est `NOT PROVEN`.

## Exécution

```bash
ELEPHANT_FREYA_VAULT=/chemin/vers/vault cargo run --manifest-path Elephant/freya/Cargo.toml
pnpm freya:test
pnpm test:vue-to-freya
```

Voir les preuves et limites dans [`baseline/README.md`](baseline/README.md), les cartes dans [`migration-map.json`](migration-map.json) et les inventaires dans [`inventory/`](inventory/).
