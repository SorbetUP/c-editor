# Baseline et preuves

Date de capture : 2026-08-11, macOS, branche `nsb/freya-native-migration`, arbre de travail déjà modifié avant cette migration.

## Oracle Tauri/Vue

Commandes exécutées avant le port :

```text
rtk pnpm tauri:check       PASS
rtk pnpm test:unit         PASS — 173 fichiers, 3 238 tests passés, 171 ignorés
rtk pnpm tauri:web:build   PASS
rtk pnpm tauri:dev         PASS — démarrage réel observé puis arrêt contrôlé
```

Artefacts :

- [capture Tauri avant migration](tauri-before.png)
- [log Tauri observable](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T12-59-46-807Z-tauri-dev-51175.log)
- [événements Tauri](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T12-59-46-807Z-tauri-dev-51175.ndjson)
- [rapport du bundle renderer](/Users/sorbet/Desktop/Dev/c-editor/build/out/renderer-bundle-report.json)
- [log Vitest baseline](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T12-58-24-345Z-vitest-unit-49210.log)

La capture montre le shell Tauri réel, mais aucune comparaison pixel à pixel n’est encore faite. Elle ne prouve pas la parité avec Freya.

## Tranche Freya

Preuve locale actuelle :

```text
rtk pnpm freya:test
PASS — 29 tests (25 Rust/service tests, 4 Freya-testing scenarios)
rtk pnpm freya:check
PASS — elephant-freya compiles on macOS after Skia dependencies were downloaded
rtk node tools/vue-to-freya/test.mjs
PASS — structured SFC analysis
```

Le smoke run de la vraie fenêtre Freya a produit :

- [capture Freya](freya-after-observable.png)
- [Freya headless fixture capture](freya-after-native.png)
- [log Freya observable](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T13-18-04-183Z-freya-dev-60456.log)
- [événements Freya](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T13-18-04-183Z-freya-dev-60456.ndjson)
- [latest Freya runtime log](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T14-30-07-950Z-freya-final-runtime-75921.log)
- [latest Freya runtime events](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T14-30-07-950Z-freya-final-runtime-75921.ndjson)
- [Freya release runtime log](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T14-44-00-665Z-freya-release-runtime-final-80500.log)
- [Freya release runtime events](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T14-44-00-665Z-freya-release-runtime-final-80500.ndjson)

Le processus a été arrêté par `SIGINT` après la capture ; son `success=false` dans l’enveloppe observable signifie l’arrêt contrôlé, pas un crash. Le log contient le démarrage, le chargement du fixture et `entries=2`.

Les scénarios Freya prouvent maintenant le menu Create (création réelle d’un Markdown et d’un dossier), l’ouverture d’une note depuis une carte, le rendu PNG headless, les chemins cachés/symlinkés et la lecture filtrée d’un vault. La capture headless n’est pas une preuve de fenêtre native pixel-perfect. Ils ne prouvent pas encore l’éditeur complet, le cycle add-on, le packaging, Android, la persistance applicative complète ou la parité visuelle.

## Preuves encore requises

Les suites desktop Tauri existantes (`test:desktop:acceptance` et `test:desktop:acceptance:packaged`) restent nécessaires pour l’oracle. Une suite d’acceptation Freya équivalente doit être ajoutée avant toute suppression du chemin Vue. Il faut conserver les captures, les logs renderer/host et les fixtures d’un profil/vault propre.

La commande `rtk pnpm test:desktop:acceptance` a été exécutée hors sandbox avec un profil et un vault temporaires. Elle a compilé le renderer, démarré Tauri et traversé les commandes de vault/add-ons/édition, puis a échoué sur le parcours `elephant.code-execution` avec `waitFor(.elephant-physical-code-run, 10000)`. Le log observable est [`desktop-acceptance-freya-migration`](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T13-22-32-785Z-desktop-acceptance-freya-migration-63238.log). Cet échec empêche de déclarer la baseline applicative entièrement verte.

La même acceptance a été relancée sur l’état final et reproduit exactement le même échec après 137 commandes réussies : [`desktop-acceptance-final`](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T14-31-12-629Z-desktop-acceptance-final-76065.log). Ce résultat reste `NOT PROVEN` pour l’oracle desktop complet ; il ne masque pas les preuves Freya ciblées ci-dessus.

Le binaire macOS packagé a aussi été trouvé et lancé, mais l’acceptance packagée a expiré avant le démarrage du serveur Tauri (`Timed out waiting for the Tauri acceptance server`). L’artefact observable est [`desktop-acceptance-packaged-final`](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T14-40-27-435Z-desktop-acceptance-packaged-final-78534.log), avec les événements [`desktop-acceptance-packaged-final.ndjson`](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T14-40-27-435Z-desktop-acceptance-packaged-final-78534.ndjson).

`rtk pnpm prod:check` reste bloqué dès `lint` sur l’arbre global (`42 errors`, `981 warnings` au dernier run). Les erreurs restantes sont dans des fichiers Tauri/Vue/tests déjà modifiés hors de cette tranche ; le run complet est conservé dans [`prod-check-final`](/Users/sorbet/Desktop/Dev/c-editor/test-results-observability/2026-08-11T14-44-27-424Z-prod-check-final-80545.log).
