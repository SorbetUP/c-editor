# Desktop full functional audit

- Request: Corriger les régressions desktop (Sync, addons, citations, imports, IA, Dashboard, Sites, Excalidraw, navigation et éditeur) et fournir des commandes d’acceptance Tauri entièrement loggées.
- Level: high
- Date: 2026-07-19
- Slug: desktop-full-functional-audit

## Summary

- [x] Le transport d’acceptance Tauri loopback existe et journalise chaque commande, résultat et erreur.
- [x] Le premier lancement sans coffre et la persistance après redémarrage sont testés sur le vrai shell Tauri.
- [x] Le problème de retry des addons après sélection du premier coffre est corrigé.
- [x] Le harnais expose l’état des addons, leurs ressources, leurs actions et les appels enable/disable/run avec logs.
- [x] Le tampon de citations desktop (sélection, collage, informations et suppression) est implémenté et testé.
- [x] La matrice runtime Tauri installe et active les 16 addons officiels, journalise leur état et échoue si l’un reste en erreur.
- [x] Les ressources AI/OCR/Search/Calendar/Knowledge/Models/Wiki/Google Keep et les actions Graph/Knowledge/Wiki/Chat/Calendar sont exercées dans Tauri.
- [x] Le flux Code Execution Python et le flux citation desktop complet sont exercés dans Tauri.
- [x] Les builds Tauri desktop embarquent le catalogue officiel et les sidecars natifs de la plateforme dans les ressources du bundle.
- [ ] Les sidecars natifs de toutes les plateformes doivent être publiés dans le pipeline Elephant-Addons ; le core ne peut pas fournir les binaires absents.

## Repro Steps

- [x] Lancer `pnpm test:desktop:acceptance` dans le core.
- [x] Observer un profil Tauri vide, sélectionner le coffre fixture, puis vérifier que le registre d’addons retente son chargement.
- [x] Arrêter puis relancer le processus Tauri et relire le coffre et la note modifiée.
- [x] Lancer `cargo test --manifest-path official/sync/native/Cargo.toml` dans Elephant-Addons.
- [x] Avant correction, le test pair-à-pair Sync pouvait sélectionner une adresse LAN non joignable et expirer ; le test est désormais borné à loopback.
- [x] Installer et activer `elephant.sync` depuis le binaire Tauri release avec un profil propre ; le package officiel et son service natif ont été vérifiés sur macOS arm64.

## Environment

- [x] macOS arm64, desktop Tauri, renderer Rust/Muya.
- [x] Node.js 22, Rust/Cargo disponibles.
- [x] Tests exécutés avec un profil HOME temporaire pour éviter l’état utilisateur réel.
- [x] Le cache local `.cache/elephant-addons` est un checkout séparé, actuellement détaché sur le commit catalogue utilisé par `sync-elephant-addons.mjs`.

## Observed vs Expected

- Observed: les addons pouvaient être chargés sans coffre, échouer sur `No active ElephantNote vault`, puis rester non chargés après onboarding.
- Expected: le chargement doit être différé, journalisé, puis retenté dès qu’un coffre actif existe.
- Observed: l’acceptance bridge ne permettait pas d’inspecter ou d’exécuter les actions addons sans instrumentation ad hoc.
- Expected: les opérations addons doivent être pilotables et traçables par commandes.
- Observed: les citations ne proposaient qu’une copie dans la barre supérieure.
- Expected: un tampon desktop permet la sélection, le collage par clic gauche, les informations/suppression par clic droit.
- Observed: le test Sync réseau dépendait d’une adresse locale choisie dans un ordre non déterministe.
- Expected: le test local doit être déterministe et laisser les scénarios réseau réels à un test dédié.
- Observed: le catalogue distant `main` déclarait des sidecars mais certaines URLs natives étaient absentes (404), ce qui rendait une installation propre incapable de démarrer un service.
- Expected: un bundle desktop doit embarquer le catalogue et le sidecar de sa plateforme, avec une trace explicite de la source utilisée.

## Hypotheses

- [x] Le premier échec du registre venait de l’absence normale de coffre au moment du bootstrap.
- [x] Le retry devait être déclenché par un événement de changement de coffre, avec un chemin nul ignoré.
- [x] Le timeout Sync de test venait de la sélection d’une adresse LAN/IPv6 non routable dans l’environnement de test.
- [ ] Une installation propre de sidecars doit être vérifiée sur chaque plateforme supportée par un artefact package réel.

## Investigation Plan

- [x] Auditer les entrées Tauri, le runner, les logs renderer et stderr.
- [x] Auditer le catalogue et les manifests Elephant-Addons.
- [x] Exécuter les tests JS addon, Sites, AI, Google Keep et le test Rust Sync.
- [x] Ajouter des scénarios Tauri dédiés pour Dashboard, Google Keep, Sites, Sync, citations, Excalidraw et actions addons.
- [x] Ajouter des fixtures métier Tauri pour AI/chat, Calendar, Graph, Knowledge, Wiki, OCR, Code Execution et Open Models.
- [x] Exécuter les scénarios sur le binaire desktop Tauri release, pas uniquement sur le checkout de développement.

## Fix Plan

- [x] Garder la génération des sidecars activée par défaut, y compris en CI ; ne permettre le saut qu’avec `ELEPHANT_SKIP_NATIVE_ADDON_BUILD=1` explicite.
- [x] Réessayer le registre externe après activation d’un coffre et éviter les retries sur les payloads sans chemin actif.
- [x] Journaliser l’état et le cycle de vie des actions addons dans le bridge.
- [x] Ajouter le tampon de citations et son intégration au bus d’édition.
- [ ] Publier ou rendre disponibles les packages natifs de chaque plateforme du catalogue Sync/OCR/Knowledge/etc.
- [x] Convertir le cycle installation/activation de chaque addon officiel en scénario d’acceptance Tauri avec état et logs.
- [x] Convertir les actions métier addon couvertes par le catalogue en scénario d’acceptance Tauri avec données, résultat attendu et log d’erreur.

## Regression Tests

- [x] `pnpm test:desktop:acceptance` — scénario Tauri complet, premier lancement, UI fonctionnelle, fichiers, Excalidraw, redémarrage.
- [x] `pnpm test:unit:raw tests/elephant/unit/noteCitationRuntime.spec.js` — citation et tampon.
- [x] `pnpm test:unit:raw` — suite core complète exécutée avec les changements.
- [x] `pnpm test:feature-matrix`.
- [x] `pnpm lint --quiet` après migration des dialogues Element Plus vers le slot `header`.
- [x] `pnpm test:e2e:raw` — 43 scénarios passants après reconstruction de `dist/`.
- [x] `pnpm build:mac` puis `pnpm test:desktop:acceptance:packaged` — binaire Tauri macOS arm64 empaqueté, 994 logs et 238 commandes.
- [x] `node --test tests/google-keep.test.mjs tests/sites-regression.mjs tests/sites-runtime.mjs tests/ai-inference.test.mjs tests/ai-chat.test.mjs tests/ai-agent.test.mjs`.
- [x] `cargo test --manifest-path official/sync/native/Cargo.toml --test two_endpoint_sync physical_package_pairs_and_synchronizes_two_real_iroh_endpoints`.
- [x] Acceptance runtime de chacun des addons officiels : activation, ressources, actions et vues principales sur macOS arm64.

## Release Notes

- Le test desktop est désormais un test fonctionnel Tauri observable, sans dépendance Electron/CDP.
- Les erreurs intentionnelles restent vérifiées séparément des erreurs applicatives.
- La génération native échoue tôt si un sidecar attendu ne peut pas être construit.
- La dernière exécution Tauri release a activé les 16 addons officiels sans erreur ; elle a produit 994 entrées structurées, vérifié l’exécution Python, les appels de chaque service natif, la citation, les sidecars/services natifs et trois erreurs attendues.
- Le bundle release est maintenant testé avec son propre `Contents/Resources` ; le journal confirme `source=bundled` pour le catalogue et les sidecars, sans dépendance au checkout local.
- Chaque service natif desktop est maintenant appelé par le scénario empaqueté (`interpreter.status`, `codex.status`, `knowledge.status`, `models.status`, `sync.status`) ; les indisponibilités optionnelles restent exposées dans le résultat structuré au lieu d’être silencieuses.
- Les scénarios d’erreur commande inconnue, chemin invalide et ressource addon inconnue sont rejoués et leurs arguments complets sont conservés dans l’archive d’acceptance.
- Le pont d’acceptance n’est plus installé dans une build desktop normale ; il est activé uniquement quand le transport loopback est explicitement demandé par `ELEPHANT_ACCEPTANCE_TAURI_PORT`.
- Les cinq dialogues utilisant encore le slot Element Plus `title` sont passés au slot `header`; l’avertissement de dépréciation a disparu après reconstruction du bundle web.
- Le preload Electron de compatibilité reconnaît explicitement `tauri_acceptance_enabled=false`, afin que le garde-fou desktop ne transforme pas les E2E Electron en erreurs `unhandled invoke`.
- La matrice CI Tauri empaquetée couvre Linux x86_64, Windows x86_64, macOS Intel et macOS arm64, avec conservation des logs en cas d’échec.
- Le runner accepte désormais `ELEPHANT_ACCEPTANCE_APP_PATH` pour rejouer la même suite contre un binaire empaqueté.
- Le contrôle global `prod:check` passe après correction du chargement Playwright (CLI racine unique) et du mock `tauri_acceptance_enabled` ; la suite E2E couvre désormais 43 scénarios passants.

## Risks

- [ ] La construction de sidecars en CI augmente la durée de pipeline et exige les toolchains natives disponibles.
- [ ] Les artefacts natifs Linux/Windows et macOS x86_64 restent à valider dans leur pipeline cible ; cet environnement prouve macOS arm64.
- [ ] L’exécution des addons trusted dans l’acceptance doit rester limitée au profil de test et ne pas devenir une API de production.
- [ ] Le tampon de citations est en mémoire de fenêtre ; sa persistance entre redémarrages n’est pas requise, mais doit rester explicitement testée comme volatile.

## Rollout

- [ ] Vérifier les artefacts natifs macOS, Linux et Windows dans Elephant-Addons.
- [ ] Fusionner les tests addon dans le dépôt qui possède l’implémentation concernée.
- [x] Rejouer la suite d’acceptance sur une installation Tauri macOS arm64 propre avant de déclarer la couverture macOS valide.
