# Preuve d’automatisation Tauri par Playwright

Date de l’audit : 2026-08-11
Branche : `nsb/freya-native-migration`
HEAD observé : `84b3784272c083b72316f5591c524f032f8ff0cf`

## Conclusion factuelle

Dans l’état actuel du dépôt, Playwright pilote directement le harness Electron, pas le binaire Tauri réel.

Le binaire Tauri réel dispose d’un runner d’acceptance HTTP local. Ce runner permet d’envoyer des commandes au bridge Tauri de la page, mais ce n’est pas une `Page` Playwright : il n’expose ni locator Playwright, ni capture `page.screenshot`, ni trace, ni vidéo, ni événements de pointeur réels.

Le dépôt ne contient pas d’adaptateur Playwright vers le WebView macOS Tauri, de endpoint CDP Tauri, ni de configuration WebKit Inspector exploitable par Playwright. La comparaison actuelle ne peut donc pas prouver l’égalité visuelle, et encore moins l’égalité des parcours en mouvement, entre Tauri et Freya.

## 1. Harness Electron utilisé par Playwright

### Preuves dans le dépôt

- `tests/app/e2e/helpers.js:4` importe `playwright` et `_electron`.
- `tests/app/e2e/helpers.js:176` appelle `_electron.launch(...)`.
- `tests/app/e2e/helpers.js:193` récupère `app.firstWindow()` comme page Playwright.
- `tests/app/e2e/electron-main.js:40` sert `build/out/renderer` avec un serveur HTTP Node.
- `tests/app/e2e/electron-main.js:45-57` crée un `BrowserWindow` Electron et charge le renderer construit.
- `tests/app/e2e/tauri-preload-entry.js` charge `tauri-preload.js`, un shim de compatibilité Electron qui reproduit des appels Tauri côté test.
- `tests/app/e2e/playwright.config.js:21-27` active par défaut la capture Playwright des screenshots, traces et vidéos, avec un viewport de `1280x720`.

Le fait que le renderer chargé soit le renderer produit Tauri n’en fait pas un processus Tauri : le processus hôte reste Electron, et les appels natifs sont fournis par le preload Electron.

### CDP trouvé côté Electron

`tests/app/e2e/electron-main.js:90-92` active le port de debugging Chromium uniquement si `ELEPHANT_ACCEPTANCE_CDP_PORT` est défini :

```text
app.commandLine.appendSwitch('remote-debugging-port', ...)
```

Ce code est dans `electron-main.js`; aucune équivalence n’a été trouvée dans `Elephant/backend/tauri`, `build/scripts` ou `tests/app/e2e` pour le WebView Tauri.

## 2. Lancement du Tauri réel

### Développement

`build/scripts/run-desktop-acceptance.mjs:36-49` lance par défaut :

```text
./build/scripts/build_dev.sh
```

`build/scripts/build_dev.sh:24-32` se place dans `Elephant/backend/tauri` puis exécute :

```text
cargo tauri dev --no-watch
```

Sur macOS, ce chemin lance donc le binaire Tauri natif et son WebView macOS. Il ne passe pas par `_electron.launch` et ne crée aucune `Page` Playwright.

### Application packagée

`build/scripts/run-packaged-desktop-acceptance.mjs:9-31` sélectionne, sur macOS, le binaire :

```text
Elephant/backend/tauri/target/release/bundle/macos/Elephant.app/Contents/MacOS/Elephant
```

Il le passe ensuite à `run-desktop-acceptance.mjs` via `ELEPHANT_ACCEPTANCE_APP_PATH`, toujours sans Playwright.

## 3. Acceptance runner HTTP Tauri

### Transport

`build/scripts/run-desktop-acceptance.mjs:46-75` :

1. démarre l’application Tauri avec `spawn` ;
2. définit `ELEPHANT_ACCEPTANCE_TAURI_PORT=0` ;
3. attend une ligne stdout `ELEPHANT_ACCEPTANCE_TAURI_PORT=<port>` ;
4. appelle `GET /health` puis `POST /command` avec la fonction JavaScript native `fetch`.

`Elephant/backend/tauri/src/acceptance_server.rs:235-270` confirme que le serveur est activé par la variable d’environnement, écoute sur `127.0.0.1`, expose `/health` et `/command`, puis émet un événement Tauri `elephant:acceptance:command` vers le renderer.

Le résultat revient au serveur par la commande Tauri `tauri_acceptance_result`. Ce chemin exerce le binaire Tauri et le bridge Tauri, mais il ne fournit pas une session Playwright attachée au WebView.

### Surface du bridge renderer

`Elephant/frontend/src/renderer/src/platform/acceptanceTestBridge.js` expose notamment :

- `readDom(selector)` : `querySelector`, texte, HTML, attributs et visibilité approximative ;
- `click(selector)` : appelle `element.click()` ;
- `fill(selector, value)` : affecte `element.value` et envoie des événements `input`/`change` ;
- `insertText`, `press`, `selectText`, `contextClick` ;
- `waitFor`, `waitUntilGone`, `readState`, `readDisplayed` et des commandes métier.

Cette surface est une API applicative appelée par HTTP. Les actions de clic et de saisie sont synthétisées dans le DOM ; elles ne sont pas des actions Playwright envoyées à une page Tauri.

### Absence de capture et de mouvement

La recherche du bridge ne trouve aucune implémentation de :

- `screenshot`, `capture` ou rendu d’image ;
- `PointerEvent`, `WheelEvent`, mouvement de souris ou glissement ;
- `requestAnimationFrame` ou échantillonnage de frames ;
- lecture de rectangles/coordonnées permettant de comparer la géométrie visuelle.

Les occurrences de mouvement observées sont limitées à des `MouseEvent` synthétiques pour `mouseup` et `contextmenu`. Les attentes temporelles utilisent `setTimeout`; elles ne constituent pas une capture de l’animation en cours.

Le runner actuel peut donc vérifier du DOM, des commandes et de la persistance. Il ne peut pas produire la séquence d’images nécessaire à une comparaison visuelle fixe et dynamique avec Freya.

## 4. CDP et WebKit Inspector

### Probe Playwright locale

Commande exécutée :

```text
rtk proxy node -e "(async()=>{for(const [name,browserType] of [['chromium',require('playwright').chromium],['webkit',require('playwright').webkit],['firefox',require('playwright').firefox]]){try{await browserType.connectOverCDP('http://127.0.0.1:1'); console.log(name+':unexpected-success')}catch(error){console.log(name+':'+String(error.message||error).split('\\n')[0])}}})().catch(error=>{console.error(error);process.exitCode=1})"
```

Résultat observé avec Playwright `1.60.0` :

```text
chromium:browserType.connectOverCDP: connect EPERM 127.0.0.1:1 - Local (0.0.0.0:0)
webkit:Connecting over CDP is only supported in Chromium.
firefox:Connecting over CDP is only supported in Chromium.
```

Cette probe montre que Playwright ne fournit pas de connexion CDP WebKit. Le fait que `webkit.connectOverCDP` existe dans l’objet JavaScript ne signifie pas que le protocole est supporté : l’appel le rejette explicitement.

### Recherche d’une activation WebKit/Tauri dans le dépôt

Commande exécutée :

```text
rtk rg -n "open_devtools|close_devtools|is_devtools_open|devtools|WebInspector|WKWebView|remote.?inspect" Elephant/backend/tauri/src Elephant/backend/tauri/tauri.conf.json Elephant/backend/tauri/capabilities build/scripts tests/app/e2e --glob '*.rs' --glob '*.json' --glob '*.js' --glob '*.mjs' --glob '*.sh'
```

Résultat : aucune occurrence d’un mécanisme d’ouverture/configuration de DevTools, WebKit Inspector ou remote inspection dans ces chemins.

`Elephant/backend/tauri/tauri.conf.json` configure `devUrl`, `frontendDist`, `withGlobalTauri` et la fenêtre, mais ne configure pas de port d’inspection WebKit.

Le document `agent/docs/project/dev/DEBUGGING.md` décrit `--inspect` et `--remote-debugging-port` pour MarkText/Electron. Il ne constitue pas une procédure d’attachement au WebView Tauri et aucune procédure Tauri équivalente n’a été trouvée dans le dépôt.

## 5. Exécution Tauri observée

Un essai du runner réel a été exécuté avec :

```text
ELEPHANT_ACCEPTANCE_UI_ONLY=1 ELEPHANT_ACCEPTANCE_SHOW_WINDOW=1 ELEPHANT_ACCEPTANCE_SKIP_BUILD=1 pnpm test:desktop:acceptance
```

Le runner a bien atteint le transport Tauri et l’application a répondu aux commandes HTTP. L’échec enregistré dans `test-results/acceptance/latest.json` est :

```text
runtime: tauri
ok: false
Error: Create menu is incomplete
...
excalidrawLogo.exists: true
excalidrawLogo.visible: false
```

La preuve établit que le chemin d’acceptance Tauri réel a été lancé et qu’il a inspecté le DOM via le bridge. Elle ne fournit ni preuve Playwright Tauri, ni preuve de parité visuelle. Le scénario est en échec fonctionnel sur cette révision.

## 6. Matrice des chemins réellement disponibles

| Chemin | Processus hôte | Contrôle | DOM | Screenshot/trace/video Playwright | Mouvement réel |
|---|---|---|---|---|---|
| Tests `tests/app/e2e` | Electron | `_electron.launch` | Oui, via `Page` | Oui, selon `playwright.config.js` | Oui côté Electron/Chromium |
| Acceptance `run-desktop-acceptance.mjs` | Tauri natif | `fetch` vers `127.0.0.1` | Oui, via commandes bridge synthétiques | Non | Non : pas de page Playwright ni d’événements pointer/wheel |
| CDP | Chromium/Electron seulement | `chromium.connectOverCDP` possible si un endpoint existe | Oui | Oui | Oui côté Chromium |
| WebKit Inspector Tauri macOS | Aucun adaptateur dans le dépôt | Non démontré | Non | Non | Non |

## Limite de preuve

La sortie « Tauri » et la sortie « Playwright » ne sont pas comparables à armes égales dans ce dépôt : la première est une API HTTP de commandes synthétiques, la seconde une page Playwright attachée à Electron. En particulier, aucune sortie image ou timeline de frames Tauri n’est disponible pour être comparée à Freya Testing.

Pour rendre la comparaison demandée possible, il faudrait d’abord fournir un adaptateur de capture/interaction pour le Tauri macOS réel (inspection WebKit réellement activée et pilotable, ou protocole d’acceptance exposant captures, géométrie et frames), puis faire exécuter aux deux runtimes le même scénario et les mêmes checkpoints temporels. Cette évolution n’a pas été réalisée dans cet audit.

## Modifications effectuées

Seul ce rapport a été créé. Aucun fichier produit n’a été modifié et aucun commit n’a été créé.
