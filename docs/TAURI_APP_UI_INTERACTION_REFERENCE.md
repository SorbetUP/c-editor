# Elephant — référence complète de l’interface et des interactions Tauri

> Documentation déduite du code de l’application Tauri au 17 août 2026.
>
> Cette référence décrit le produit réellement assemblé par le dépôt : shell Vue/Tauri, éditeur Rust/Muya, ponts de compatibilité, extensions officielles et backend Tauri/Rust. La branche inspectée contient aussi une cible native Freya en migration ; elle est documentée séparément et ne doit pas être confondue avec le renderer WebView Tauri. Les contrats lisibles dans le code sont séparés des comportements qui restent à confirmer par une exécution graphique.

## 0. Statut, périmètre et méthode

### 0.1 Ce que cette documentation couvre

Elle couvre :

- la fenêtre native Tauri et son démarrage ;
- la composition exacte du shell desktop et mobile ;
- la bibliothèque de notes et dossiers ;
- la sidebar, la navigation, l’historique et les vaults ;
- l’éditeur Markdown, ses contrôles, ses raccourcis, son moteur Rust/Muya, les tableaux, listes, images, liens, composition IME, presse-papiers et glisser-déposer ;
- la recherche globale et la recherche dans l’éditeur ;
- les réglages, thèmes, préférences, addons et sync ;
- les vues Workspace : Canvas, Graph, Chat, Calendar et vues fournies par addons ;
- Excalidraw et les assets cachés ;
- les effets disque, la persistance locale, l’IPC, les erreurs et les événements de watcher ;
- l’inventaire des icônes, des transitions et des animations.

### 0.2 Hiérarchie des sources

La source de vérité est l’assemblage suivant :

1. `Elephant/frontend/src/renderer/src/main.js` : bootstrap du renderer Tauri.
2. `Elephant/frontend/app/**` : shell et composants Elephant modernes.
3. `Elephant/frontend/src/renderer/src/editor-rust/**` et `muya/**` : éditeur et interactions bas niveau encore utilisés par le shell.
4. `Elephant/backend/tauri/src/**` : commandes natives, stockage, sécurité des chemins, watcher et runtime des addons.
5. `addons/official/**` : addons officiels qui contribuent des vues, réglages, commandes ou services.
6. `Elephant/freya/**` : implémentation native Freya et contrats de migration présents sur la branche active ; elle ne remplace pas la hiérarchie Tauri/Vue ci-dessus, mais ses surfaces et ses écarts sont documentés en sections 31 et suivantes.

### 0.3 Niveaux de preuve

| Niveau | Signification |
|---|---|
| `CODE` | Le comportement est explicitement lisible dans le code source. |
| `CONTRAT` | Le comportement est protégé par un contrat, une commande ou un test ciblé. |
| `RUNTIME À EXÉCUTER` | Le code le prévoit, mais cette documentation n’exécute pas l’application graphique. |
| `DYNAMIQUE` | La surface dépend d’un addon installé, de son package ou d’un service natif. |

Les dimensions et délais ci-dessous sont des valeurs CSS/JS exactes, sauf lorsqu’ils sont exprimés comme `min()`, `clamp()`, `vw`, `vh`, `dvh` ou dépendants du viewport.

### 0.4 Snapshot du dépôt inspecté

- Branche : `nsb/freya-native-migration`.
- Commit inspecté : `03876f18f` (`test(freya): prove graph does not t...`).
- Runtime de référence : fenêtre Tauri WebView (`Elephant/backend/tauri` + `Elephant/frontend`) ; la branche contient en parallèle le binaire natif Freya (`Elephant/freya`).
- Modifications non committées observées avant rédaction : `Elephant/freya/src/app.rs`, `Elephant/freya/src/app/explorer.rs`, `Elephant/freya/src/app/library.rs`, `Elephant/freya/src/app/library_actions.rs`, `Elephant/freya/src/app/navigation.rs`, `Elephant/freya/src/main.rs`, `Elephant/freya/tests/freya_shell_acceptance.rs`, ainsi que le dossier non suivi `Elephant/feature-incoming/`.
- Le fichier documenté lui-même était non suivi au début de cette passe. Aucun fichier de code ou de test n’a été modifié par cette rédaction ; les changements utilisateur ci-dessus sont conservés.

---

## 1. Identité de l’application et architecture visible

Elephant est une application de notes Markdown locale. Un vault est un dossier réel sur disque. Les notes restent des fichiers Markdown ; les métadonnées, index, addons, modèles, sync et assets internes sont séparés dans des dossiers cachés.

### 1.1 Graphe de rendu

```text
Tauri window
  └─ Vite renderer (Elephant/frontend/src/renderer)
      └─ main.js
          ├─ bootstrap Tauri / globals / bridges
          ├─ Vue Main + router + Pinia + Element Plus
          └─ AppShell
              ├─ TopVaultBar + NavigationBar
              ├─ IconRail
              ├─ SidebarNav
              ├─ MainContent
              │   ├─ LibraryToolbar + LibraryGrid
              │   ├─ NoteEditorHost
              │   └─ AddonWorkspaceRouter
              ├─ SearchModal
              └─ SettingsPanel
```

### 1.2 Règle de visibilité des modes

`MainContent` choisit une seule surface principale :

| Condition | Surface |
|---|---|
| Aucun vault actif | `EmptyVaultPicker`. |
| Une note est réellement ouverte dans les tabs du store éditeur | `NoteEditorHost`. |
| Vue d’addon active sans note ouverte | `AddonWorkspaceRouter`. |
| Vue `notes` sans note ouverte | bibliothèque + zones workspace contribuées. |
| Addon absent ou view non supportée | état vide explicite avec bouton de retour aux notes. |

Une note est considérée ouverte uniquement si `openedNotePath` correspond à un fichier réel du vault et qu’un tab du store contient un `id` ainsi qu’un `markdown` string. Cela évite d’afficher un shell d’éditeur vide pendant une transition IPC.

### 1.3 Fenêtres et dimensions natives

Source : `Elephant/backend/tauri/tauri.conf.json`, `tauri.android.conf.json`, `tauri.linux.conf.json`.

| Cible | Taille initiale | Chrome | Autres règles |
|---|---:|---|---|
| Desktop Tauri | 1280 × 840 px | `titleBarStyle: Overlay`, `hiddenTitle: true` | `acceptFirstMouse: true`, fenêtre `main`. |
| Linux bundle | 1280 × 840 px | `decorations: false` | redimensionnable, targets `deb` et `appimage`. |
| Android | 390 × 844 px | fenêtre redimensionnable | asset protocol limité à l’espace applicatif Android. |

La fenêtre desktop n’est pas dessinée par les décorations système : la barre haute d’Elephant fournit une zone de drag et, hors macOS, ses propres contrôles minimiser/maximiser/fermer.

---

## 2. Démarrage Tauri, états initiaux et erreurs de bootstrap

### 2.1 Ordre de démarrage

`Elephant/frontend/src/renderer/src/main.js` réalise, dans cet ordre fonctionnel :

1. installe les globals de compatibilité (`path`, `fileUtils`, pont Muya) ;
2. installe les ponts Tauri runtime, fichiers, ElephantNote, recherche, providers, sauvegarde MarkText et IPC local ;
3. installe diagnostics renderer, store diagnostics, acceptance bridge et addons ;
4. charge `appDataDir()` et définit `window.__MARKTEXT_WINDOW_ID__ = 1`, `__MARKTEXT_WINDOW_TYPE__ = 'editor'` ;
5. démarre le bootstrap renderer historique ;
6. restaure l’état portable de fenêtre ;
7. monte Vue, Pinia, router, Element Plus et i18n ;
8. active les features core `addonPacksCoreFeature` et `excalidrawCoreFeature` ;
9. attend `router.isReady()` ;
10. monte l’application sur `#app` ;
11. marque `document.documentElement.dataset.elephantMounted = 'true'` et ajoute `aria-label="Elephant application ready"`.

### 2.2 Échec de démarrage

Si une étape lève une erreur :

- le diagnostic est enregistré avec `renderer startup failed` ;
- le root `#app` est remplacé par `<main id="elephant-startup" data-error="true" role="alert">` ;
- le logo `assets/static/icon.png` est rendu ;
- le texte visible est `Elephant n’a pas pu démarrer` ;
- le message et la stack d’erreur sont affichés dans le même écran.

Le code ne remplace pas silencieusement un échec Tauri par un écran vide. Niveau : `CODE`.

### 2.3 État sans vault

`EmptyVaultPicker` occupe `100dvh`, utilise une grille centrée et un padding de 32 px. La carte a une largeur de `min(440px, 100%)`.

Éléments :

- logo Elephant : 96 × 96 px, `object-fit: contain` ;
- titre : `Choose your first vault` ;
- description : `Keep notes in private app storage or choose an existing folder.` ;
- deux boutons empilés avec un gap de 12 px, hauteur minimale 64 px, padding 12 × 16 px et rayon 14 px ;
- bouton primaire `Private storage` / `Create a vault managed by Elephant` ;
- bouton secondaire `Choose folder` / `Open an existing notes folder`.

Interactions :

| Action | Effet |
|---|---|
| Clic `Private storage` | création d’un vault local géré par Elephant ; retour d’un payload appliqué au store. |
| Clic `Choose folder` | ouverture du sélecteur de dossier Tauri ; annulation laisse l’écran tel quel. |
| Erreur de création/sélection | `vaultStore.error`, visible via la surface concernée et loggé. |

---

## 3. Fondations visuelles

### 3.1 Tokens de base Elephant

Les tokens sont fournis par `Elephant/shared/appearance.js` et injectés sur `.en-shell` et `document.documentElement`.

#### Elephant Light

| Token | Valeur |
|---|---|
| fond | `#f7f9fc` |
| surface | `#ffffff` |
| sidebar | `#edf2f7` |
| soft | `#e9eff7` |
| soft strong | `#dfe7f1` |
| bordure | `#c5cfdd` |
| texte | `#101828` |
| texte secondaire | `#475467` |
| accent | `#2563eb` |
| danger | `#dc2626` |
| ombre | `0 30px 90px rgba(15, 23, 42, 0.16)` |

#### Elephant Dark

| Token | Valeur |
|---|---|
| fond | `#0f141d` |
| surface | `#141a24` |
| sidebar | `#101722` |
| soft | `#1b2432` |
| soft strong | `#202b3b` |
| bordure | `#283244` |
| texte | `#eef3fb` |
| texte secondaire | `#98a3b6` |
| accent | `#5ea1ff` |
| danger | `#ff6b7a` |
| ombre | `0 18px 44px rgba(0, 0, 0, 0.28)` |

Les tokens alimentent aussi l’éditeur : `--editorBgColor`, `--editorColor`, `--selectionColor`, `--floatBgColor`, `--floatHoverColor`, `--codeBlockBgColor`, `--linkColor`, `--headingColor` et `--maskColor`.

### 3.2 Familles de thèmes

Chaque famille possède une variante claire et sombre, sauvegardée dans `localStorage` sous `elephantnote:theme`.

| Famille | Variantes | Identité |
|---|---|---|
| Elephant | `light`, `dark` | workspace neutre, accents bleus. |
| Apple | `apple-light`, `apple-dark` | surfaces givrées, graphite, bleu iOS. |
| Graphite | `graphite-light`, `graphite-dark` | chrome monochrome dense. |
| Nord | `nord-light`, `nord-dark` | surfaces froides et accent cyan. |
| Solar | `solar-light`, `solar-dark` | palette chaude, amber et code blocks. |
| Forest | `forest-light`, `forest-dark` | vert éditorial naturel. |
| Beige | `beige-light`, `beige-dark` | ivoire papier, argile. |
| Pastel | `pastel-light`, `pastel-dark` | lavande, pêche, menthe. |
| Gamer Violet | `gamer-violet-light`, `gamer-violet-dark` | violet profond, focus néon. |

Le changement de thème met à jour la ref `theme`, écrit immédiatement le localStorage, met à jour les tokens CSS, informe le canvas et transmet la variante canonique `light`/`dark` à Excalidraw.

### 3.3 Règles générales de contrôle

- Les contrôles modernes utilisent des boutons sans bordure visible par défaut, puis un fond `--en-soft` au survol.
- Les focus clavier visibles utilisent le plus souvent un outline de 2 px avec `--en-primary` et un offset de 2 px.
- Les contrôles Settings sont normalisés : hauteur 34 px, variante compacte 29 px, rayon 9 px, switches 42 × 24 px, thumb 20 px.
- Les badges sont des pills de rayon `999px`.
- Les scrollbars globales héritées ont 12 px de largeur ; les listes de rail et certaines listes internes masquent ou réduisent leur scrollbar.
- Le shell réserve `min-width: 0` à chaque niveau critique pour éviter que les notes, graphes, réglages ou addons débordent horizontalement.

---

## 4. Shell desktop exact

### 4.1 Composition et géométrie

Le shell `.en-shell` est une flex-row de hauteur `100vh`/`100dvh`, cachant l’overflow.

```text
┌─ rail vertical 48 px ─┬──────────────────────────────┐
│                       │ top strip 28 px              │
│                       ├──────────────────────────────┤
│                       │ sidebar 232 px │ main        │
│                       │                 │ content     │
└───────────────────────┴─────────────────┴────────────┘
```

Valeurs :

- `.en-rail` : 48 px de large, `flex-basis: 48px`, hauteur complète de la zone sous la top strip, border-right 1 px ;
- `.en-topstrip` : hauteur finale 28 px, flex-shrink 0 ;
- `.en-body` : grille `var(--en-sidebar-width) 0 minmax(0, 1fr)` ;
- largeur initiale sidebar : 232 px via `sidebarWidth = ref(232)` ;
- borne resizer : 184–320 px ;
- persistance : `localStorage['elephantnote:sidebarWidth']` ;
- `.en-main` : colonne flex, `overflow: hidden`, background `--en-bg`.

La cascade contient un ancien override Tauri à 56 px/32 px, mais `app-shell-runtime-fixes.css` le remplace en dernier par 48 px et 28 px. La valeur active à la fin du bundle est donc 48/28.

### 4.2 Top strip et zone de drag

`TopVaultBar.vue` rend un `<header data-tauri-drag-region>`.

#### Zone gauche

- `NavigationBar` en position absolue ; desktop left 56 px, top 4 px, 76 px de large, 24 px de haut ;
- macOS : left 84 px ;
- `.en-topstrip-drag` occupe le reste de la barre et reçoit `-webkit-app-region: drag` ;
- le double-clic sur cette zone bascule maximisé/restauré ;
- l’étiquette passive `Tauri` est positionnée à top 5 px, hauteur 20 px, min-width 34 px, padding horizontal 8 px, rayon 7 px, police 10 px ;
- hors macOS, les contrôles natifs dessinés par Elephant sont à 46 × 100% px chacun.

#### Contrôles fenêtre hors macOS

Icônes Lucide :

- `Minus`, 15 × 15 px : minimiser ;
- `Square`, 15 × 15 px : maximiser ;
- `Copy`, 14 × 14 px : restaurer quand maximisé ;
- `X`, 15 × 15 px : fermer.

Chaque bouton fait 46 px de large. Au survol : `--en-soft`, texte `--en-text`. Le bouton close devient fond `#c42b1c` et texte blanc. L’état maximisé est relu avec `getCurrentWindow().isMaximized()` au montage et à chaque `onResized`.

#### Navigation retour/avance

`NavigationBar` affiche deux boutons 24 × 24 px, gap 2 px, rayon 5 px :

- `ChevronLeft` 18 × 18 px ;
- `ChevronRight` 18 × 18 px ;
- opacity 0.3 si la pile n’a pas de destination ;
- survol actif : fond soft et texte principal ;
- clic : `navigationStore.back()` ou `.forward()` puis `vaultStore.navigateTo(entry)`.

Raccourcis équivalents au niveau shell : `Alt/Option + ArrowLeft` et `Alt/Option + ArrowRight`.

### 4.3 Rail vertical

`IconRail` est un `<nav aria-label="Workspace navigation">` de 48 px. Il est techniquement dans la zone draggable du shell, mais le rail, ses boutons et ses menus sont `no-drag`.

- padding-top : 8 px ;
- sur macOS, `.en-rail-macos` ajoute 36 px de padding-top ;
- rail button standard : 34 × 34 px, rayon 8 px, icône Lucide 18 × 18 px ;
- gap entre boutons : 2 px ;
- séparateur : 24 × 1 px, marges verticales 5 px ;
- zone basse : padding 6 px 0 8 px.

#### Éléments core

| ID | Icône | Action |
|---|---|---|
| `vault` | icône du vault courant ou `Vault` | ouvre le sélecteur de vault au clic/survol. |
| `sidebar-toggle` | `PanelLeft` au repos, `PanelLeftClose` ou `PanelLeftOpen` selon état | masque/affiche la sidebar. Au survol, l’icône directionnelle remplace l’icône neutre. |
| `search` | `Search` | ouvre SearchModal. |
| `settings` | `Settings` | ouvre SettingsPanel. |

Les addons peuvent ajouter des vues au rail. Les icônes connues sont mappées vers `BookOpenText`, `CalendarDays`, `Database`, `FileText`, `GitFork`, `GraduationCap`, `Home`, `Landmark`, `LayoutDashboard`, `ListTodo`, `MessageCircle`, `Rocket`, `Search`, `Settings`, `Sparkles`, `Star`, `Terminal`, `Vault`, `Workflow`. Une icône inconnue tombe sur `Star`.

#### Ordre, visibilité et drag du rail

- l’ordre est lu dans la préférence `iconRailOrder` ;
- les éléments masqués viennent de `iconRailHidden` ;
- un séparateur peut être présent dans l’ordre ;
- les nouveaux addons sont ajoutés à l’ordre runtime sans supprimer les IDs existants ;
- un bouton, sauf `sidebar-toggle`, peut démarrer un drag HTML ou pointer ;
- le drop sur un autre item déplace l’ID dans l’ordre, persiste `iconRailOrder` et logge `drag:drop` ;
- un double clic involontaire sur le même item, à moins de 420 ms, est ignoré sauf pour `sidebar-toggle`.

#### Menu de vault

Le bouton vault possède un wrapper 34 × 34 px. Le menu apparaît au hover du wrapper et reste ouvert si le pointeur entre dans le menu.

- position : left 52 px, top -4 px ;
- largeur minimale 250 px ;
- padding 6 px, border 1 px, rayon 10 px ;
- items : font 13 px, padding 4 px, rayon 6 px ;
- bouton de sélection interne : min-height 32 px ;
- icône initiale : 24 × 24 px, rayon 6 px ;
- bouton d’édition d’icône : 26 × 26 px, opacity 0 par défaut, visible au hover/focus ;
- check actif : SVG 16 × 16 px en `--en-primary`.

Interactions :

- cliquer un vault appelle `setActiveVault`, recharge le payload, vide le chemin courant et ferme le menu ;
- `Pencil` affiche un picker d’icônes 5 colonnes, cellules 28 × 28 px, gap 5 px, padding 8 px ;
- les options sont `Home`, `Files`, `Database`, `Learning`, `Archive`, `Project`, `Favorite`, `Code`, `Workflow` ;
- `Add another vault` ouvre le sélecteur de dossier ;
- `Manage vaults` ouvre Settings sur `vaults` ;
- l’icône par défaut est `Vault`.

Animation du menu : transition Vue `en-vault-fade`, opacity + translation X de -4 px vers 0, 120 ms ease.

### 4.4 Sidebar

La sidebar `.en-sidebar` occupe la colonne de largeur runtime. Fond `--en-sidebar-bg`, border-right 1 px, overflow hidden.

- scroll interne hauteur 100%, padding top 8 px ;
- bouton `All notes` : min-height 38 px, margin 0 8 px 8 px, padding horizontal 12 px, gap 10 px, rayon 8 px, font 14 px/600 ; icône `Inbox` 18 × 18 px ;
- header `Notes` : padding 4 px 14 px 8 px, label 11 px uppercase avec letter-spacing .06em ; bouton recherche 24 × 24 px, icône `Search` 14 × 14 px ;
- arbre : padding horizontal 6 px, gap vertical 2 px.

`All notes` reçoit un fond active color-mix de l’accent à 20%. Un drag valide sur la racine ajoute outline 1 px accent ; un drag invalide ajoute outline 1 px danger.

#### Entrée d’arbre

Chaque ligne a une hauteur minimale de 36 px, padding gauche `10px + depth × 14px`, gap 6 px, rayon 8 px.

- dossier : chevron 22 × 22 px, `ChevronRight` ou `ChevronDown` 15 px ; label bouton flexible ; compteur optionnel ; bouton détacher 22 × 22 px, opacity 0 sauf hover/focus ;
- note : label directement dans la ligne, titre ellipsé ;
- ligne active ou hover : fond `--en-soft`, texte principal ;
- ligne en drag : opacity 0.45 ;
- dossier cible valide : fond accent 18%, outline accent 1 px ; cible invalide : outline danger 1 px.

Interactions :

| Geste | Effet |
|---|---|
| Clic chevron dossier | inverse `expanded`, puis charge les enfants une seule fois. |
| Clic label dossier | ouvre le répertoire, force l’expansion et recharge ses enfants. |
| Clic note | appelle `vaultStore.openNote`. |
| Drag d’un dossier/note | transporte un payload `application/x-elephantnote-entry`. |
| Drop sur dossier | déplace l’entrée sur disque si la cible est valide, puis recharge les enfants. |
| Bouton X | détache l’entrée de la sidebar et modifie le workspace canonique. |

Les chemins qui commencent par `.` dans un composant du chemin sont filtrés de la navigation normale. Seuls les dossiers et les fichiers Markdown visibles sont présentés.

### 4.5 Redimensionnement de sidebar

Le resizer est un separator ARIA vertical à largeur visuelle nulle mais zone pointer de 12 px via `::before`.

- valeur initiale : 232 px ;
- flèches gauche/droite : -16/+16 px ;
- drag : delta horizontal depuis le point de départ ;
- mise à jour pendant le drag via `requestAnimationFrame` ;
- largeur clampée 184–320 px ;
- ligne centrale accent : 3 × 64 px, centrée, opacity 0 par défaut, opacity 1 au hover/focus/drag ;
- pendant le drag : curseur `col-resize` et `user-select: none` sur tout le document ;
- à la fin : persistance locale immédiate.

---

## 5. Bibliothèque des notes et dossiers

### 5.1 Toolbar flottante

`LibraryToolbar` est positionnée absolute en haut de la bibliothèque, avec `padding: 10px 12px`, gap global 18 px, pointer-events désactivés sur le conteneur et réactivés sur les contrôles.

- bouton Create : 56 × 56 px, rayon 11 px, icône `Plus` 27 × 27 px ; fond primaire ;
- bouton sort : 52 × 52 px, rayon 12 px, icône 22 × 22 px, shadow 0 8px 22px ;
- bouton view : 52 × 52 px, rayon 12 px, icône `Grid3x3` ou `List` 22 px ;
- gap entre sort et view : 14 px ;
- desktop : la grille réserve 72 px de padding top pour ne pas passer sous les boutons.

#### Menu Create

Le menu est 280 px de large, top `100% + 8px` sur desktop, padding 8 px, border 1 px, rayon 14 px, shadow `0 16px 36px`.

Options, dans l’ordre :

1. `Note` — `FilePlus2`, `Create a new note` ;
2. `Drawing` — logo SVG Excalidraw (`shared-muya-icon`), `Open a new Excalidraw canvas` ;
3. `Folder` — `FolderPlus`, `Organize notes in a folder`.

Chaque option a padding 10 px, gap 12 px, icône 20 px, titre 14 px et description 12 px. Un clic ferme le menu avant d’exécuter l’action. Un pointerdown extérieur ou `Escape` le ferme sans action.

Actions disque :

- Note : création via `tauri_notes_create`, rafraîchissement de la liste, ouverture immédiate et enregistrement dans l’historique ;
- Folder : `tauri_folders_create`, rafraîchissement des entrées ;
- Drawing : ouverture de l’expérience Excalidraw.

Pendant une création : le bouton est disabled, `aria-busy` est positionné, l’erreur est affichée en texte rouge et loggée.

Sur mobile le menu est positionné fixed à droite/bas avec le même contenu, au-dessus du FAB.

### 5.2 Tri et modes d’affichage

Le bouton de tri avance cycliquement :

1. `Updated newest` — `ArrowDownNarrowWide` ;
2. `Updated oldest` — `ArrowUpNarrowWide` ;
3. `Title A-Z` — `ArrowDownAZ` ;
4. `Title Z-A` — `ArrowDownZA` ;
5. retour au premier.

Le store place toujours les entrées épinglées en premier, puis applique le tri choisi.

Le bouton de vue alterne :

- `grid` → `list`, icône affichée `Grid3x3`, label `Show notes as list` ;
- `list` → `grid`, icône `List`, label `Show notes as grid`.

### 5.3 Grille, pagination et rendu borné

`LibraryGrid` :

- `overflow: auto`, overscroll contain ;
- desktop padding top 72 px ;
- surface grid : padding 0 10 px 10 px ;
- colonnes : `repeat(auto-fill, minmax(min(240px, 100%), 1fr))` ;
- gap : 10 px ;
- surface list : flex-column, gap 6 px ;
- page backend : 120 entrées maximum ;
- fenêtre de rendu initiale : 72 entrées ;
- préfetch lorsque la distance au bas est ≤ 720 px ;
- le fetch suivant part avec offset égal au nombre déjà chargé ;
- les doublons de chemin sont éliminés avant append.

Un changement de vault, de chemin ou de workspace remet la fenêtre à 72. Le premier élément d’une grille de plus de trois entrées reçoit `featured`, sauf un dossier.

### 5.4 Carte note

La carte `.en-note-card` est aussi utilisée comme composant de dossier selon `entry.kind`.

#### Géométrie

- min-height normale : 176 px ;
- featured : 220 px ;
- dossier featured : reste 176 px ;
- padding : 10 px ;
- border : 1 px ;
- rayon : 10 px pour `NoteCard` ;
- fond : mix de surface 34% et fond ;
- icône document : 22 × 22 px ;
- titre : `clamp(18px, 1.6vw, 26px)`, line-height 1.12, max 4 lignes ;
- aperçu image dessin : 100% × 112 px, featured 156 px, border 1 px, rayon 8 px, image contain ;
- tags : texte `#tag`, tags fournis par l’entrée.

En mode liste :

- min-height : 58 px ;
- display grid 1fr/auto ;
- titre clamp 16–21 px, une ligne ;
- excerpt, tags, preview et folder preview masqués ;
- icône 20 × 20 px.

#### Actions

- clic sur la carte : ouverture de l’entrée après un délai logique de **220 ms** ;
- double-clic sur le titre : annule l’ouverture programmée et active le renommage inline ;
- pin : bouton 30 × 30 px, visible au hover ou si épinglé, icône `Pin` 20 px ; jaune `#facc15`, remplie quand active ;
- menu : `MoreHorizontal` 20 px ;
- clic droit sur la carte : ouvre le menu d’actions ;
- popover : top 42 px, right 8 px, padding 5 px, gap 4 px, rayon 10 px, boutons 32 × 32 px ;
- actions popover : `Pencil` rename, `Eye`/`EyeOff` sidebar pour dossier, `Trash2` delete ; danger en rouge.

Renommage : input prérempli, focus puis sélection complète. `Enter` commit, `Escape` cancel, clic extérieur cancel. Un titre vide ou inchangé ne déclenche aucun write.

Drag/drop :

- la carte est draggable ;
- son payload contient kind, path, title et éventuellement preview ;
- opacity pendant drag : 0.42 ;
- drop valide sur dossier : border accent, fond accent 10%, `vaultStore.moveEntry` ;
- drop invalide : border danger ; aucune mutation ;
- self-drop, descendant ou déplacement déjà dans le parent sont refusés par le store avant l’appel IPC.

### 5.5 Dossier

Le dossier présente :

- faux folder icon 42 × 32 px, gradient `#3d95ff` → `#1d63f0`, onglet pseudo-element 20 × 11 px en `#65adff`, top -7 px ;
- titre 17–24 px, jusqu’à 4 lignes ;
- nombre de notes en 15 px ;
- preview bas : min-height 42 px, padding 7 × 8 px, border, rayon 8 px ;
- jusqu’à trois enfants avec `Folder`, `PenLine` ou `FileText` 15 px ;
- état vide `No items yet`.

Un clic sur carte/titre ouvre le dossier. Le renommage garde le même protocole. Les dossiers peuvent être affichés/masqués dans la sidebar et épinglés.

---

## 6. Navigation, historique et vaults

### 6.1 État du vault

`vaultStore` conserve : vaults enregistrés, vault actif, workspace, entries, rootEntries, chemin courant, vue active, notes ouvertes, chemins épinglés, records Wiki et états de chargement.

Vues acceptées : `notes`, `wiki`, `chat`, `canvas`, `graph`, `calendar`, `models`. La vue inconnue retombe sur `notes`.

Changer de vault :

1. appelle `tauri_vaults_set_active` ;
2. remplace `vaults`, `activeVault`, `workspace`, `entries`, `rootEntries` ;
3. vide le chemin courant et la vue active ;
4. ferme les notes d’un autre vault ;
5. recharge les chemins épinglés dans une clé localStorage scopée au vault ;
6. émet `elephantnote:vault-active-changed`.

Le backend initialise les dossiers cachés à l’ouverture et renvoie immédiatement le payload du vault.

### 6.2 Création, ouverture, renommage, déplacement et suppression

| Action utilisateur | Commande/effet |
|---|---|
| créer note | `tauri_notes_create`; fichier Markdown créé puis ouvert dans le renderer. |
| créer dossier | `tauri_folders_create`; liste courante rafraîchie. |
| ouvrir dossier | `tauri_directory_list(relativePath)` ; navigation push `folder` ou `all_notes`. |
| ouvrir note | événement local `mt::open-file`, lecture `tauri_notes_read`, tab unique par chemin normalisé. |
| renommer | `tauri_entries_rename`; chemins épinglés, chemin courant et note ouverte sont remappés. |
| déplacer | `tauri_entries_move`; listes root/courante rechargées, chemins d’état remappés. |
| supprimer | `tauri_entries_delete`; chemin supprimé de l’état et des pins ; le backend applique le contrat trash. |
| restaurer trash | `tauri_vault_trash_restore`. |
| vider trash | `tauri_vault_trash_empty`. |

Le backend n’ouvre jamais un dossier comme note et refuse un chemin hors du vault. Les erreurs sont retournées comme `Result<_, String>` au bridge Tauri.

### 6.3 Épinglage

Les pins ne sont pas une propriété temporaire du composant : ils sont sauvegardés dans `localStorage` sous `elephantnote:pinnedNotes:<vault-id-ou-path>`. Un clic sur `Pin` ajoute le chemin en tête ; un second clic le retire. Les renommages et déplacements remappent les préfixes de chemin.

### 6.4 Résolution de navigation

L’historique ne pousse pas deux fois la même entrée. Les entrées peuvent représenter :

- `all_notes` ;
- `folder` avec `path` ;
- `note` avec `path`/`title` ;
- `workspace` avec vault id ;
- vue de workspace.

Retour/avance appelle `navigateTo` avec `record: false` pour éviter une boucle dans la pile.

---

## 7. Éditeur de note

### 7.1 Structure visible actuelle

`NoteEditorHost` rend :

```text
en-editor-layer
  └─ en-editor-panel
      ├─ NoteEditorTopBar
      │   ├─ titre
      │   ├─ date
      │   ├─ 2 tags visibles + overflow
      │   ├─ add tag
      │   ├─ pin
      │   └─ close
      ├─ en-note-editor-shell
      │   └─ en-editor-host
      │       └─ EditorWithTabs (tab bar masquée)
      └─ NoteEditorFooter (si showEditorFooter)
```

Le tab bar historique existe sous-jacent pour l’identité et la persistance du moteur, mais le host passe `show-tab-bar="false"`. Il ne faut donc pas documenter une rangée de tabs visible dans l’interface Tauri actuelle.

### 7.2 Barre de note

`NoteEditorTopBar` :

- hauteur normale minimale 52 px ;
- hauteur compacte après scroll vertical > 24 px : 36 px ;
- padding horizontal piloté par `noteEditorMargin`, par défaut 12 px et borné 8–48 px ;
- gap 8 px ;
- border bottom de 1 px à 42% de la couleur de bordure ;
- titre : input flexible, min-width 120 px, font-size normal 28 px / compact 19 px, font-weight 800 ;
- date/tag/action chip : hauteur 30 px, border 1 px, rayon 8 px, padding horizontal 8 px, font 14 px ;
- actions pin/close : 30 × 30 px, icône 18 px.

Animation de compaction : `min-height`, hauteur, font-size et padding transitent en **180 ms ease**.

Le titre est commit sur `change`, pas à chaque frappe. Le bouton pin inverse l’état localStorage du chemin courant. Close retire la note du shell et laisse l’autosave de fermeture finir via le host.

### 7.3 Tags dans la topbar

- les deux premiers tags sont rendus directement ;
- les suivants sont regroupés dans un bouton `+N` ;
- clic sur tag visible : mode édition ;
- clic droit sur tag visible : suppression demandée ;
- clic sur `+N` : ouvre un popover width `min(220px, calc(100vw - 24px))`, avec les tags cachés ;
- clic droit dans le popover : suppression du tag correspondant ;
- clic `+` : formulaire inline ;
- le formulaire affiche input et boutons de 30 px ; `Enter` soumet, `Escape` annule ;
- clic extérieur ferme le popover overflow.

La préférence `showTagHashInEditor` contrôle l’affichage `#tag` sans modifier le tag persistant.

### 7.4 Footer

`NoteEditorFooter` est conditionné par `preferencesStore.showEditorFooter`.

- hauteur minimale 50 px ;
- padding horizontal 24 px ;
- compteurs : `X words` et `Y characters`, gap 12 px ;
- actions : gap 8 px ; chaque bouton addon/typo/thème min-width 36 px, hauteur 36 px, rayon 8 px ;
- menu `Aa` : 36 × 36 px ; popover bottom 42 px, right 0, min-width 150 px, padding 8 px ;
- options : `Compact`, `Normal`, `Large` ;
- icône thème : `Moon` en clair, `SunMedium` en sombre.

Le choix de taille est conservé dans `localStorage['elephantnote:editorTextScale']` et appliqué au moteur d’édition.

### 7.5 Barre d’outils de formatage

`NoteEditorToolbar` existe comme surface réutilisable : hauteur 56 px, padding horizontal 24 px, gap 14 px, boutons 34 × 34 px, icônes 20 px. La toolbar complète est exposée aux contributions qui l’intègrent.

Actions et payloads :

| Label | Icône | Événement/payload |
|---|---|---|
| Heading 2 | `Heading2` | paragraph / `heading 2` |
| Bold | `Bold` | format / `strong` |
| Italic | `Italic` | format / `em` |
| Strikethrough | `Strikethrough` | format / `del` |
| Link | `Link2` | format / `link` |
| Bullet list | `List` | paragraph / `ul-bullet` |
| Numbered list | `ListOrdered` | paragraph / `ol-bullet` |
| Task list | `SquareCheckBig` | paragraph / `ul-task` |
| Inline code | `Code2` | format / `inline_code` |
| Quote | `Quote` | paragraph / `blockquote` |
| Table | `Table2` | paragraph / `table` |
| Image | `Image` | insert-image |
| Excalidraw | `PenLine` | insert-excalidraw |
| Horizontal rule | `Minus` | insert-horizontal-rule |
| Dictate | `Mic` | speech-to-text |
| Read aloud | `Volume2` | text-to-speech |
| Auto tag | `Tags` | auto-tag |
| Ask AI | `Sparkles` | ask-ai |
| Agents | `Bot` | open-agents |

`Ask AI` et `Agents` peuvent disparaître selon `tauri_features_get` / les feature flags. Si le chargement des flags échoue, ils restent visibles par défaut dans ce composant.

### 7.6 Éditeur Rust/Muya : événements DOM pris en charge

`ElephantRustInputController` attache au conteneur :

- `beforeinput` ;
- `click` ;
- `copy`, `cut`, `paste` ;
- `dragover`, `drop` ;
- `compositionstart`, `compositionend` ;
- `keydown`.

Chaque mutation est sérialisée par une queue promise. Une sélection DOM devenue obsolète après un refresh est remplacée par la sélection du snapshot Rust plutôt que d’être renvoyée avec un node id détaché.

#### `beforeinput`

| `inputType` | Commande Rust |
|---|---|
| `insertText`, `insertReplacementText` | insert text avec `event.data`. |
| `insertParagraph`, `insertLineBreak` | insert paragraph. |
| `deleteContentBackward`, `deleteContent`, `deleteByCut`, `deleteByDrag` | delete backward. |
| `deleteContentForward` | suppression du prochain grapheme ou fusion de paragraphe. |
| `deleteWordBackward/Forward` | suppression par mot selon `Intl.Segmenter` si disponible. |
| `deleteSoftLine*`, `deleteHardLine*`, `deleteEntireSoftLine` | suppression par ligne. |
| `historyUndo` | undo. |
| `historyRedo` | redo. |
| `formatBold` | toggle strong. |
| `formatItalic` | toggle emphasis. |
| `formatStrikeThrough` | toggle strike. |

Les événements sont preventDefault puis envoyés au bridge Rust avec la sélection courante.

#### Clavier

- flèches, Home, End, PageUp, PageDown invalident la sélection mise en cache ;
- `Enter` sans Ctrl/Meta/Alt et hors composition est repris par le contrôleur si WebKit ne fournit pas `beforeinput`, puis produit un nouveau paragraphe ;
- `Escape` annule une composition IME active ;
- `Tab` dans une table passe à la cellule suivante ; `Shift+Tab` à la précédente ;
- hors table, Tab n’est pas intercepté par ce contrôleur.

#### Listes de tâches

Le clic sur `[data-muya-rust-task-checkbox]` arrête la propagation et envoie `setTaskChecked(nodeId, checkbox.checked, autoCheck)`. Le CSS rend une case unique : 16 × 16 px, border 2 px, rayon 3 px, positionnée left -24 px ; une case cochée reçoit fond accent et coche blanche de 5 × 9 px tournée à 45°.

#### Copie/coupe/collage

La sélection peut produire simultanément :

- `application/x-elephant-markdown` ;
- `application/x-muya-markdown` ;
- `application/x-markdown` ;
- `text/markdown` ;
- `text/plain` ;
- `text/html`.

Le Markdown préserve les wrappers `**strong**`, `*em*`, `~~del~~` et le code inline avec backticks adaptés. Le HTML collé est converti en Markdown pour les paragraphes, listes, blockquotes, fences, tables, liens et images.

#### Composition IME

`compositionstart` fige une plage UTF-16 ; `insertCompositionText` remplace la plage sans commit ; `compositionend` commit le texte final ; Escape annule la composition. Les offsets sont des offsets UTF-16 et non des indices visuels.

#### Drag/drop dans l’éditeur

Les types acceptés :

- fichiers externes, si `onFileDrop` est présent ;
- `text/uri-list`, si `onUriDrop` est présent ;
- texte/HTML/Markdown, converti en Markdown.

Le caret est placé avec `caretRangeFromPoint` ou `caretPositionFromPoint`. L’action `dropEffect` est `copy`. Un fichier image local est copié dans `.assets`, le nom est nettoyé et rendu unique (`name`, `name-1`, etc.), puis une référence Markdown relative stable est insérée. Les compagnons Excalidraw déclarés par un addon sont copiés avec l’asset.

Un drop d’entrée interne sur `data-entry-drop-target="note-editor"` produit un lien ou une référence Markdown adaptée au kind, sans déplacer l’entrée source.

### 7.7 Autosave et sauvegarde Tauri

Constantes de `NoteEditorHost.vue` :

| Cas | Délai |
|---|---:|
| polling de cohérence | 500 ms |
| autosave par défaut | 160 ms après le dernier changement |
| delta ≥ 8 KiB | au plus 60 ms |
| delta ≥ 64 KiB | au plus 20 ms |

Le host protège contre deux writes simultanés, garde une écriture en attente si une autre est en vol, réécrit les références d’images locales vers `.assets`, puis rafraîchit les entrées visibles.

Le bridge `tauriMarkTextSaveBridge` refuse :

- une sauvegarde sans tab id ;
- une sauvegarde sans pathname ;
- une écriture de Markdown vers un fichier qui n’est pas `.md`.

Le write passe par `tauri_marktext_write_file`, le résultat est visible via `mt::tab-saved` ou `mt::tab-save-failure`. La fermeture de tabs ne force la fermeture que des tabs dont le write a réellement réussi.

---

## 8. Excalidraw et dessins

### 8.1 Ouverture

Le menu Create ou les cartes drawing déclenchent `open-excalidraw-from-image` / l’overlay `ExcalidrawDialog`. La surface est un Teleport body full-screen.

### 8.2 Géométrie

- overlay fixed inset 0, z-index 5000, no-drag ;
- shell 100% × 100%, padding-top 28 px ;
- header absolute top 0, hauteur 28 px, padding gauche 86 px, blur 16 px ;
- input nom height 20 px, font 12 px, largeur max 420 px ;
- actions absolute top 16 px, right 176 px, gap 6 px ;
- boutons fermer/sauver : 22 × 22 px, pills ; icônes 16 px ;
- canvas : hauteur `calc(100vh - 28px)`, fond blanc par défaut ;
- erreur : largeur max 520 px, margin top 96 px, padding 24 px, rayon 16 px.

### 8.3 Interactions

- saisie nom : l’édition active `nameWasEdited` ;
- `Ctrl/Cmd+S` : exporte la scène et le PNG, puis émet `save` ;
- `Escape` : ferme ou ouvre la demande de nom selon `askNameOnClose` ;
- bouton close : même logique ;
- si le nom a été modifié et `askNameOnClose`, la fermeture sauvegarde ; sinon une prompt demande un nom ;
- prompt : largeur max 420 px, padding 24 px, rayon 16 px, input min-height 40 px ; Save disabled si nom vide ; erreur `namePromptRequired` ;
- le nom final est nettoyé des extensions `.excalidraw.png`, `.excalidraw`, `.png` puis reçoit un nom PNG stable ;
- save exporte `sceneBlob`, bytes image, filename et callback d’erreur ;
- le thème de l’app met à jour la scène Excalidraw sans perdre la palette complète du shell.

Les scènes, PNG et compagnons vivent dans `.assets` selon le contrat partagé. Le dossier est caché de l’explorateur.

---

## 9. Recherche globale

### 9.1 Ouverture et overlay

Search s’ouvre via :

- bouton rail `Search` ;
- bouton recherche dans la sidebar ;
- `Ctrl/Cmd+K` ;
- `Ctrl/Cmd+F` si aucune note n’est ouverte.

Quand une note est ouverte, `Ctrl/Cmd+F` est délégué à la recherche dans le document via bus `find`, pas à la recherche globale.

`SearchModal` est un `el-dialog` de largeur 720 px, sans close button, sans close-on-escape Element Plus, avec fermeture au clic sur le modal. Le contenu commence à `margin-top: 12vh`.

La surface visuelle :

- shell full width 720 px, rayon 28 px ;
- blur 42 px, saturation 175% ;
- shadow light `0 30px 90px rgba(15,23,42,.24)` ; dark `0 24px 70px rgba(0,0,0,.5)` ;
- overlay blur 7 px ;
- barre min-height 72 px, padding gauche 24 px/droite 18 px, grid icon/input/loading/clear ;
- icône Search 22 px ; input font 22 px/720 ;
- mobile : barre 56 px, input 17 px, icône 18 px.

### 9.2 Saisie, délais et clavier

- chaque changement de query réinitialise l’index sélectionné à 0 ;
- la recherche est debounce de **220 ms** ;
- le statut est rafraîchi à l’ouverture puis toutes les 1500 ms ;
- clear vide la query et refocus/select l’input ;
- Escape avec query : clear seulement ; Escape sans query : ferme ;
- ArrowDown/ArrowUp boucle les résultats ;
- Enter ouvre le résultat sélectionné, sinon le premier concept ;
- clic résultat ou bouton `ArrowUpRight` ouvre la note.

### 9.3 États visibles

| État | Rendu |
|---|---|
| busy sans contenu | `ScanSearch`, `Searching locally…`, padding vertical 22 px. |
| erreur | bandeau rouge color-mix, texte de l’erreur. |
| query sans résultat | `Search` 28 px, `No matching notes found`, sous-texte. |
| résultats | sections `Wikis & concepts` puis `Notes & passages`. |
| index non prêt/indexing | `SearchStatusBadge` : dot, label, compteurs `indexed/total`, message. |

Les trois points de chargement font 6 × 6 px, animation `en-search-dot` de 1 s ease-in-out infinite, décalage 0/150/300 ms. L’animation alterne opacity .25→1 et scale .85→1.

### 9.4 Carte de résultat

- grille 38 px / 1fr / 34 px, gap 12 px, padding 12 × 14 px, rayon 16 px ;
- icône FileText dans un carré 38 × 38 px, rayon 14 px ;
- titre 15 px, une ligne ;
- badge 22 px de haut, pill ;
- chemin accent 12.5 px ;
- snippet 12.5 px, line-height 1.4, max 2 lignes ;
- bouton ouvrir 30 × 30 px, opacity 0 sauf hover/selected ;
- sur mobile : icon 32 px, bouton 28 px, grille 32/1fr/28.

Le résultat affiche `Semantic + keyword`, `Semantic`, `Keyword` ou `Local match` selon `matchType`. Les tokens du query sont surlignés avec le fond de l’accent.

### 9.5 Modes et réglages

`SearchSettingsPanel` expose :

- Exact : match de nom/fichier/texte, icône Search ;
- Smart : classement local équilibré, icône GitBranch ;
- Semantic : découverte par sens, icône Zap ;
- limite : range 1–50 résultats ;
- On/Off de l’index local ;
- Refresh status, Rebuild index, Clear index selon l’addon/contrat actif.

Le backend Tauri accepte une limite 1–200, lit par défaut 256 KiB max par fichier, clampé 4 KiB–1 MiB, et ignore les dossiers cachés.

---

## 10. Settings

### 10.1 Fenêtre et navigation

`SettingsPanel` est un dialog plein écran logique avec backdrop centré.

Final desktop via `runtime-layout-fixes.css` :

- padding backdrop : `clamp(6px, 2vw, 24px)` ;
- largeur panel : `min(1020px, calc(100vw - 2 * padding))` ;
- hauteur : `min(780px, calc(100vh - 2 * padding))` ;
- border-radius et border viennent du thème ;
- mobile ≤820 px : panel viewport moins 12 px, border 1 px et rayon 14 px ;
- mobile ≤520 px : nav 54 px, contenu padding horizontal 12 px.

Header :

- bouton close 32 × 32 px, icône 17 px ;
- titre `Settings` ;
- recherche avec icône Search et input `Search all settings` ;
- hint `<kbd>` `⌘ F` sur macOS ou `Ctrl F` ailleurs.

Nav :

- sections core : Appearance, Editor, Vaults, Addons ;
- chaque bouton min-height 38 px, grille 18px/1fr/14px, gap 9 px, padding 0 10 px, rayon 9 px ;
- chevron invisible sauf hover/état actif ;
- footer `Local-first` et `v0.1.0` ;
- mobile : icons seules, label/chevrons/footer cachés.

Transitions génériques Settings : 140 ms pour bordure/background/couleur/opacity ; chevrons 140–160 ms ; thumb de switch 170 ms.

### 10.2 Recherche dans Settings

La query cherche dans label, description et section. Tous les termes doivent apparaître.

- page devient `Search` ;
- chaque résultat est un bouton min-height 66 px, grille icône 32 / copie / section / chevron ;
- clic sélectionne la section exacte, vide la recherche et remonte le contenu ;
- zéro résultat : icône Search, `No setting found`, aide ;
- résultats animés par simple état, sans rechargement du shell.

### 10.3 Appearance

Contrôles :

- Light/Dark avec `SunMedium`/`Moon` ;
- `LanguageSettingsRow` pour système, langues intégrées et packs ISO ;
- Theme : accordion ouvert par défaut, chevron `ChevronDown`, neuf cartes thèmes ;
- carte thème : min-height 86 px, grille preview 84 px + copie ; preview 58 px de haut ; check actif 19 × 19 px ;
- `IconRailLayoutSettings` : reorder/hide/separators du rail ;
- Floating surfaces : switch 42 × 24 px.

### 10.4 Editor

Préférences exactes :

- Editor footer ;
- Tag prefix ;
- Quick insert menu ;
- Quick insert trigger, un seul caractère, max length 1 ;
- Pair brackets ;
- Pair Markdown syntax ;
- Pair quotes ;
- Spellchecker ;
- Code block line numbers ;
- Note margins, range 8–48, step 4, affichage en px ;
- Autosave ;
- Autosave delay : 250 ms, 500 ms, 1 s, 2 s, 5 s ; disabled si autosave off.

Les switches ont 42 × 24 px ; le thumb passe de translateX(0) à 18 px en 170 ms. Les contrôles disabled opacity .48–.52 selon le type.

### 10.5 Vaults

Chaque vault est une ligne détaillée :

- grille 32 px / contenu / actions ;
- gap 11 px ;
- padding vertical 14 px ;
- icône 32 px ;
- nom 12.5 px ;
- chemin ellipsé 10.5 px ;
- metadata `Last opened` en 10 px ;
- actions icon-only 26 × 26 px ;
- picker icônes cellules 28 × 28 px, gap 5 px.

Actions : changer icône, éditer nom, copier chemin, activer, enable/disable, retirer de la liste. Retirer ne supprime pas le dossier disque. Les vaults absents sont marqués missing ; l’erreur de chargement propose Retry.

Trash : accordion `Vault trash`, refresh, Restore par item, `Empty trash` danger. L’icône de chargement `LoaderCircle` tourne en 0.9 s linear infinite.

### 10.6 Addons

`AddonsSettingsPanel` affiche deux pages : `Addons` et `Addon packs`.

- nav tabs dans une toolbar sticky : min-height 32 px, padding 0 10 px ;
- search Addons/Addon packs ;
- switch `Installed only` ;
- refresh `RefreshCw` ;
- import file `Plus` ;
- catalogue en tuiles : min-height 150 px, grille icône 46 / copie / chevron, padding 16 px, rayon 14 px ;
- clic tuile ouvre un browser split avec liste gauche et détail droit ;
- détail montre nom, version, auteur, description, permissions, commandes, modules AI ;
- addon installé : switch enable et bouton Uninstall ;
- addon disponible : bouton Install ;
- action addon disabled si addon désactivé ;
- les addons communautaires verrouillés ne peuvent pas être activés quand la politique les interdit.

L’installation passe par le package réel et le registre Rust. La présence d’une tuile n’est pas une preuve que son runtime ou sidecar est actif.

### 10.7 Contributions Settings dynamiques

Les sections standalone viennent des contributions `settings.sections`. L’icône inconnue tombe sur `Package`. Les addons officiels ajoutent notamment AI, Chat, Search, Knowledge, OCR, Codex, Open Models, Sync, Sites, Import et Code execution.

---

## 11. Mobile Tauri

### 11.1 Détection

Le shell devient mobile si `matchMedia('(pointer: coarse)').matches` ou `matchMedia('(max-width: 760px)').matches`. Le changement est recalculé sur resize et changement d’orientation.

### 11.2 Topbar mobile

Hauteur logique 72 px + safe-area top. Grid `48px minmax(0,1fr) 48px`, gap 10 px, padding top safe-area + 10 px, horizontal 14 px, bottom 10 px.

- bouton menu : 48 × 48 px, rayon 16 px, icône Menu 23 px ;
- bouton Search : min-height 48 px, pill 999 px, padding 0 16 px, Search 23 px, texte `Search notes` ;
- bouton Settings : 48 × 48 px, rayon 16 px.

### 11.3 Drawer

La sidebar devient fixed :

- width `min(82vw, 340px)` ;
- max-width `calc(100vw - 28px)` ;
- top 0, bottom 0, z-index 50 ;
- transform `translate3d((progress - 1) * 100%, 0, 0)` ;
- box-shadow `18px 0 44px rgba(0,0,0,.38)` ;
- transition de settling : 260–280 ms, cubic-bezier `(0.22, 1, 0.36, 1)` ;
- pendant drag : transition none ;
- scrim fixed inset 0, z-index 45, opacity progress × .46, transition opacity 220–240 ms ;
- un geste vertical est abandonné si `abs(dy) > abs(dx) × 1.15` après le seuil de mouvement 7 px ;
- la décision finale projette la vitesse sur 180 ms et ouvre si progress projeté ≥ .5 ;
- settling timer : 280 ms.

Le drawer se ferme après le clic sur un bouton/lien interne, sauf toggle d’arbre, titres récents, bouton plus et éléments `aria-expanded`.

### 11.4 FAB mobile

Visible tant qu’aucune note n’est ouverte :

- fixed right max(18 px, safe-area + 18 px), bottom max(22 px, safe-area + 22 px) ;
- 64 × 64 px ;
- rayon 22 px ;
- z-index 35 ;
- icône Plus 32 px ;
- accent avec shadow 0 14 px 32 px ;
- menu Create s’ouvre au-dessus, bottom `100% + 10px`.

### 11.5 Éditeur mobile

Quand `.en-main.has-editor-open` :

- topbar shell mobile cachée ;
- topbar note min-height 132 px ;
- grille title/actions puis chips ;
- titre font clamp 28–38 px, min-height 50 px ;
- actions note 46 × 46 px, rayon 15 px ;
- chip rail horizontal scroll ;
- toolbar mobile fixed left/right 12 px, bottom safe-area + 8 px, min-height 62 px, grid 5 colonnes, padding 7 px, rayon 22 px, blur 18 px ;
- boutons 48 px de haut, rayon 15 px, icônes 24 px ;
- espace bas éditeur : 112 px + safe-area.

Les sheets mobiles apparaissent en bas : backdrop z-index 4000 et fade 150 ms ; sheet max-height `min(76dvh, 680px)`, rayon 26 px en haut, animation `translateY(100%) → 0` en 220 ms cubic-bezier ; actions en grille deux colonnes, cellules min-height 58 px.

---

## 12. Workspace et addons visibles

Le système d’addons est dynamique : la présence d’un bouton, d’une vue ou d’un réglage dépend du manifest, de l’installation, de l’activation, des permissions et parfois d’un service natif.

### 12.1 Calendar

`CalendarAddonWorkspace` : padding 28 px ; titre 28 px, line-height 1.15 ; bouton import min-height 36 px, padding 0 12 px, rayon 8 px ; listes gap 14 px, margin-top 24 px ; buckets en grille 150 px + contenu, gap 14 px, padding 14 px, border et rayon 8 px ; boutons note/event min-height 42 px. Sous 720 px, header et buckets passent en colonne.

Import Google Calendar appelle `dispatch('importGoogle')`, désactive le bouton pendant l’opération et affiche les erreurs. Les événements offline sont groupés par date ; les notes sont groupées par date de dernière édition et un clic ouvre la note.

### 12.2 Canvas sémantique

`CanvasView` utilise un stage à marge 28 px, border et rayon 14 px, fond quadrillé 32 × 32 px. Le plan est de 1800 × 1200 px. Les nodes font 200 px de large, min-height 88 px, padding 14 × 15 px, rayon 14 px et blur 14 px. Les liens folder ont 1.5 px et dash `4 7`; les liens semantic 2.5 px à opacity .85.

`-` diminue l’échelle de .1 jusqu’à .5, `+` l’augmente jusqu’à 1.8. Pointerdown capture le node, pointermove déplace selon l’échelle et pointerup persiste dans `localStorage['elephantnote:canvas:<vault>']`. Un double-clic sur un node note ouvre la note ; un dossier n’ouvre rien.

### 12.3 Graph

`AtomicGraphView` utilise Graphology + Sigma + EdgeCurve.

Surface :

- boutons flottants top 14 px/right 22 px, 38 × 38 px, rayon 10 px : `Wand2` timelapse et `Settings` options ;
- stats bottom 20 px/left 22 px, pill, 12 px ;
- reset 30 × 30 px rond, slider 100 × 4 px, thumb 13 px, label pourcentage ;
- panneau options top 62/right 70, width 340 px, max-height calc(100%-90), rayon 16 px ;
- timeline bottom 20 px centrée, height 44 px, rayon 12 px ;
- preview card width 360 px, max-height 440 px, rayon 12 px, collapsed max-height 52 px.

Données : nodes depuis l’index sémantique si disponible, sinon depuis les entrées du vault ; edges `semantic`, `explicit-link`, `folder`, `tag`, `lexical`. Les edges affichés sont limités aux 3000 plus lourds. Caméra Sigma : ratio 0.05–8, stagePadding 40.

| Action | Effet et temps |
|---|---|
| hover node | augmente le node jusqu’à +35%, accentue voisins et diminue le reste ; convergence RAF facteur .22. |
| clic node | sélectionne, affiche la carte, recentre caméra ratio .4 en **520 ms** ease-out cubic. |
| clic stage | désélectionne. |
| reset | `animatedReset` Sigma en **500 ms**. |
| slider zoom | caméra animate en **120 ms**, range .1–4, affichage %. |
| bouton options | ouvre/ferme le panneau, transition panel **220 ms**. |
| section options | accordion opacity **180 ms**, chevron rotation **220 ms**. |
| filtres | titre/tags, mots-clés, fichiers existants, orphelins. |
| groups | ajoute `Groupe N` avec couleur de palette et count 0. |
| display | labels, stats, seuil 2–20, taille .5–2.5, épaisseur .3–2.5. |
| forces | centre .05–.6, répulsion 100–900, lien .1–1.2, distance 40–220. |
| Animer / Relancer | simulation RAF **1500 ms**, puis positions persistées par vault. |
| timelapse | visibilité chronologique, transition timeline **240 ms**, vitesses 1/2/4 ; durée restante `(100-progress) × 120 / speed`. |
| carte drag | header mousedown, déplacement borné, cursor grab/grabbing. |
| carte collapse | max-height vers 52 px, transition **220 ms**. |
| ouvrir note | ouvre `relativePath/path/id` avec le vault store. |

Le dot de construction pulse en 1 s ease-in-out, opacity 1→.4. Le bouton d’animation affiche `Loader` tournant en .8 s linear infinite.

### 12.4 Chat RAG

`ChatView` est une grille pleine hauteur : topbar, scroll, composer.

- historique fixed/absolute à gauche, width `min(260px, 80%)`, transform initial -100%, transition d’ouverture **220 ms ease** ;
- backdrop couvre le chat lorsque l’historique est ouvert ;
- topbar padding 14 × 18 px 10 px ; boutons icon 34 × 34 px ;
- messages scroll padding 10 px 0 20 px ; scroll smooth ;
- composer border top, capsule et textarea auto-grow max 168 px ;
- envoi par bouton `ArrowUp` ou Enter sans Shift ; Shift+Enter garde le saut de ligne ;
- bouton mode alterne Advanced → Graph-aware → Simple ;
- bouton send disabled si query vide ou génération en cours.

Historique : bouton Menu ouvre, X ferme, clic backdrop ferme, New chat crée une conversation, search conversations filtre, clic conversation sélectionne, Trash2 supprime au hover/actif.

Accueil : titre `Ask`, sous-texte `Grounded answers from the active vault and semantic graph`, quick prompts pour synthèse du vault, liens entre idées, travail à poursuivre et organisation Wiki. Les icônes sont `Sparkles`, `Link`, `FileText` et `BookOpen`.

Génération :

1. ajoute le message user ;
2. affiche `Searching notes and generating with local AI...` ;
3. ajoute un tool call `rag.search` en running ;
4. appelle `elephantnoteClient.rag.chat` avec limite 4 en Simple, 8 sinon ;
5. ajoute réponse assistant, citations, contexte Wiki et tools ;
6. revient à `Answered with local RAG.` ou affiche l’erreur comme message assistant ;
7. scroll en bas si l’utilisateur était proche du bas.

Le point de statut `Assistant is working` pulse en **1.1 s ease-in-out infinite** : opacity .35→1, scale .85→1.1. Les tools sont des accordéons avec status dot, label, summary, `ChevronDown`, nom d’outil et citations ouvrables.

### 12.5 Wiki

Le Wiki officiel est un addon dynamique. Il affiche des propositions provenant de notes citées, avec actions de génération, acceptation/création et rejet.

Contrat de write :

- les pages acceptées sont sous `Wiki/<slug>.md` ;
- le contenu contient titre, résumé, liens de notes reliées et sources ;
- accepter une proposition rafraîchit les records, positionne le chemin `Wiki`, puis ouvre physiquement la note ;
- rejeter retire la proposition sans écrire de note ;
- les actions AI vérifient le hash attendu avant append/replace d’une note.

### 12.6 Dashboard et Recently edited

`elephant.dashboard` contribue un item rail `Dashboard` avec `LayoutDashboard`. Il garantit une note cachée par vault, puis l’ouvre dans l’éditeur. La note est réelle et éditable, mais son emplacement interne ne doit pas apparaître comme entrée normale.

`elephant.recently-edited` contribue une zone `sidebar.after-tree`. Il rend une section de notes récentes, icône textuelle `◷`, boutons avec le chemin dans `title`; cliquer ouvre la note réelle. Sa désinstallation ne doit pas modifier le cœur.

### 12.7 Open Models, Knowledge, OCR, Codex et Code Execution

Ces surfaces sont dynamiques et leur UI dépend du package installé.

- `Open Models` : vue Models, input d’identifiant GGUF, download/reload, activation de provider ; service natif `elephant-addon-service-v1` sur desktop.
- `Knowledge` : réglage et commande de rebuild index, service natif desktop ou host Rust embarqué Android/iOS ; index, embeddings, communautés, graph et Wiki sont package-owned.
- `OCR` : settings de langue/output/image, refresh status et Run OCR ; sidecar natif desktop ; Android/iOS non supportés par le manifest actuel.
- `Codex Connection` : carte de connexion ChatGPT subscription, reload, provider `Codex`, usage/limits ; service natif desktop ; adaptateur mobile non présent dans le manifest.
- `Code execution` : toolbar injectée dans les fenced code blocks, nom de langage, boutons Copy/Run ; output borné, execution id, polling, timeout et interruption ; toolbar desktop min-height 32 px, boutons min 48 × 26 px ; variante mobile boutons min 58 × 40 px.

Une carte d’addon visible prouve uniquement l’enregistrement de l’addon. Le protocole réel, le sidecar, le modèle ou le service doivent être exécutés pour prouver la fonction.

### 12.8 Sites

`Sites` propose un input de répertoire relatif au vault et `Preview directory`/`Open another directory`.

- input min-height 34 px, min-width 220 px ;
- buttons min-height 34 px ;
- preview dans un iframe scoped ;
- `Close preview` arrête le site.

Les fichiers sont construits dans `.elephantnote/site-previews/**` ou `.elephantnote/site-builds/**`, jamais hors permission.

---

## 13. Sync

La surface moderne `SyncSettingsPanel` est une interface de trois pages : `Overview`, `Devices`, `Conflicts`.

### 13.1 Géométrie

- panel grid gap 14 px ;
- toolbar sticky top -28 px, padding 5 px, border, rayon 12 px, blur 14 px ;
- tabs min-height 32 px, padding 0 10 px ;
- cards rayon 14 px ; header min-height 52 px ;
- summary desktop en trois colonnes ; chaque cellule padding 15 × 16 px ;
- device/conflict row min-height 62 px ;
- résumé icon/avatar 30 × 30 px ;
- bouton standard min-height 34 px, compact 29 px ;
- refresh `RefreshCw`/boutons en état busy : spin **0.9 s linear infinite**.

### 13.2 Interactions

- Refresh recharge status et conflict settings ;
- Create invitation crée une invitation one-time et affiche le code ;
- Copy copie le code ou affiche un message manuel si Clipboard échoue ;
- Accept invite prend un code entrant et paire le second device ;
- Sync now compare manifests et transfère les fichiers modifiés ;
- retention days est borné minimum/maximum et se sauvegarde ;
- Restore restaure un conflit sous un chemin séparé ;
- Delete demande confirmation native `window.confirm`, puis supprime la copie temporaire ;
- polling silencieux status/conflicts toutes les 5000 ms si un vault est actif et aucune action ne tourne.

Les états visibles sont `Not paired`, `N paired device(s)`, `Syncing`, `error`, `Preserved`, `Verified`, et les messages d’erreur retournés par le service Iroh.

---

## 14. Contrats backend Tauri et effets disque

### 14.1 Commandes de vault

Source principale : `Elephant/backend/tauri/src/vault/commands.rs`, `core_commands.rs`.

| Commande | Fonction |
|---|---|
| `tauri_vaults_get` | liste vaults + payload active vault/workspace/entries. |
| `tauri_vaults_select_path` | enregistre et active un dossier choisi. |
| `tauri_vaults_set_active` | change le vault actif. |
| `tauri_vaults_set_icon` / `set_name` | persiste identité visuelle et nom. |
| `tauri_vaults_set_enabled` | active/désactive l’entrée du registre. |
| `tauri_vaults_remove` | retire le vault de la liste sans supprimer son dossier. |
| `tauri_directory_list` | page de directory, offset/limit, preview. |
| `tauri_notes_create` | crée une note Markdown réelle. |
| `tauri_notes_read` | lit une note réelle et refuse directory/non-file. |
| `tauri_notes_write` | écrit content/markdown dans le vault. |
| `tauri_folders_create` | crée un dossier réel. |
| `tauri_entries_rename` | renomme note/dossier. |
| `tauri_entries_move` | déplace note/dossier. |
| `tauri_entries_delete` | supprime selon le contrat trash. |
| `tauri_vault_trash_list/restore/empty` | liste, restaure ou vide le trash. |
| `tauri_sidebar_attach/detach` | modifie le workspace sidebar. |
| `tauri_search_query/status` | recherche locale et état de l’index. |
| `tauri_attachments_list/write_text` | assets internes. |
| `tauri_drawings_list/create/read/write` | scènes Excalidraw. |
| `tauri_features_get/set` | feature flags. |
| `tauri_marktext_write_file` | sauvegarde depuis le moteur de tabs. |

### 14.2 Fichiers cachés du vault

`vault_layout.rs` définit :

```text
<vault>/
├─ .assets/                    images, PNG, scènes/dessins et compagnons
├─ .elephantnote/
│  ├─ config/workspace.json    sidebar et configuration workspace
│  ├─ config/calendar.json
│  ├─ config/sources.json
│  ├─ models/models.json
│  ├─ sync/sync.json
│  ├─ index/index.json
│  ├─ cache/
│  ├─ state/
│  ├─ trash/
│  └─ addons/
└─ .config/                    configurations portables par catégorie
```

Les chemins dont le premier composant commence par `.` sont cachés de l’UI normale. Les documents Wiki acceptés sont visibles lorsqu’ils sont écrits sous `Wiki/**`; leurs indexes internes restent cachés.

### 14.3 Sécurité des chemins

Chaque lecture/écriture/déplacement/suppression passe par une vérification de containment canonical root. Sont refusés :

- `..` qui sort du vault ;
- chemins absolus injectés dans un champ relatif ;
- lecture d’un dossier comme note ;
- write Markdown vers une extension non Markdown ;
- écriture d’un asset hors `.assets` lorsque le chemin est interne ;
- déplacement dans soi-même, dans un descendant ou vers le parent déjà actuel.

### 14.4 Watcher fichier/dossier

Le watcher Rust utilise `notify` et expose create/modify/remove/access/other. Il ignore les composants cachés, `.git`, `node_modules` et `.tmp`. `ignore_next` garde les écritures initiées par l’app pendant 30 s maximum, avec un maximum de 1024 entrées.

Une modification externe d’une note propre peut être rechargée ; une modification externe concurrente d’une note dirty ne doit pas écraser silencieusement l’édition locale.

### 14.5 Bridge renderer → Tauri

Le bridge principal appelle `window.__TAURI__.core.invoke(command, payload)`. Les appels compatibilité passent par `window.tauri.ipcRenderer` et redispatchent des événements locaux :

- `mt::open-file` / `mt::open-file-by-window-id` ;
- `mt::response-file-save` ;
- `mt::save-tabs` ;
- `mt::save-and-close-tabs` ;
- `mt::tab-saved` / `mt::tab-save-failure` ;
- `mt::force-close-tabs-by-id`.

Un chemin Markdown externe au vault est refusé pour l’ouverture interne. Un asset non Markdown peut être remis à l’opener Tauri externe.

---

## 15. Inventaire des animations et transitions

| Surface | Animation/transition | Durée |
|---|---|---:|
| navigation boutons | background/color/opacity | 120 ms ease |
| menu vault | opacity + translateX -4 px | 120 ms ease |
| rail icon | color/background | 120 ms |
| vault button | opacity | 150 ms |
| sidebar resizer | opacity de la poignée | 120 ms |
| topbar note compact | height/font/padding | 180 ms ease |
| search shell | border-radius | 160 ms ease |
| search panel | opacity | 160 ms ease |
| search loading dots | pulse opacity/scale | 1 s infinite, delays 150/300 ms |
| search result | background | 120 ms |
| search open action | opacity/background/color | 140 ms |
| mobile drawer | transform | 260–280 ms cubic-bezier |
| mobile scrim | opacity | 220–240 ms ease |
| mobile sheet | translateY | 220 ms cubic-bezier |
| mobile fade | opacity | 150 ms ease-out |
| settings controls | border/background/color/opacity | 140 ms |
| settings switch thumb | translateX 18 px | 170 ms |
| settings vault loading | rotation | .9 s linear infinite |
| graph panel | opacity + translateX 20 px | 220 ms |
| graph accordion | opacity | 180 ms |
| graph timeline | opacity + translateY 20 px | 240 ms |
| graph preview card | opacity + translateY 10 px + scale .96 | 200 ms |
| graph reset camera | camera | 500 ms |
| graph select camera | camera | 520 ms ease-out cubic |
| graph zoom camera | camera | 120 ms |
| graph force simulation | RAF de position | 1500 ms |
| graph stats building | opacity | 1 s ease-in-out infinite |
| graph primary button | active translateY | 120 ms |
| chat history | translateX | 220 ms ease |
| chat status | opacity + scale | 1.1 s infinite |
| sync/settings refresh | rotation | .9 s linear infinite |
| tabs historiques | all | 150 ms ease-in-out |
| command palette | opacity | 200 ms |
| rename legacy | all | 300 ms ease-in-out |
| code execution progress | translateX | 1 s ease-in-out alternate |
| code execution copy feedback | retour label | 900 ms |
| citation feedback | disparition du feedback | 2200 ms |

Les délais `setTimeout` ne sont pas tous des animations : 220 ms de carte est un délai logique pour distinguer simple clic et double-clic ; 160/60/20 ms sont des politiques d’autosave ; 1500 ms du graph force est une durée de simulation ; 5000 ms du sync est une fréquence de polling.

---

## 16. Inventaire des icônes et assets graphiques

### 16.1 Bibliothèque Lucide

Icônes shell/navigation : `Menu`, `Plus`, `Search`, `Settings`, `ChevronLeft`, `ChevronRight`, `PanelLeft`, `PanelLeftClose`, `PanelLeftOpen`, `Vault`, `Pencil`, `Check`, `Minus`, `Square`, `Copy`, `X`.

Icônes bibliothèque : `FilePlus2`, `FolderPlus`, `PenLine`, `ArrowDownAZ`, `ArrowDownNarrowWide`, `ArrowDownZA`, `ArrowUpNarrowWide`, `Grid3x3`, `List`, `Pin`, `MoreHorizontal`, `Eye`, `EyeOff`, `FileText`, `Folder`, `Trash2`, `PencilLine`.

Icônes éditeur : `Heading2`, `Bold`, `Italic`, `Strikethrough`, `Link2`, `List`, `ListOrdered`, `SquareCheckBig`, `Code2`, `Quote`, `Table2`, `Image`, `Mic`, `Volume2`, `Tags`, `Sparkles`, `Bot`, `Moon`, `SunMedium`.

Icônes recherche : `ScanSearch`, `FileText`, `ArrowUpRight`, `RefreshCw`, `Power`, `GitBranch`, `SlidersHorizontal`, `Zap`.

Icônes workspace/graph/sync : `Wand2`, `Crosshair`, `RotateCcw`, `ChevronDown`, `ChevronUp`, `Loader`, `Play`, `Pause`, `ArrowRight`, `AlertTriangle`, `Archive`, `Clock3`, `Fingerprint`, `FolderSync`, `Gauge`, `Laptop`, `Link2`, `ShieldCheck`, `Undo2`.

### 16.2 Assets raster/SVG

- logo application : `Elephant/frontend/app/assets/ElephantLogo.png`, `AppLogo.png`, et `Elephant/assets/static/icon.png` au bootstrap ;
- Excalidraw : `Elephant/frontend/src/muya/lib/assets/icons/excalidraw.svg` ;
- icônes Muya legacy : `Elephant/frontend/src/muya/lib/assets/icons/*.svg` ;
- previews drawing : object URLs `Blob`, rendus lazy dans la carte ;
- graph labels : canvas 2D overlay, pas des éléments DOM ;
- fenêtre Tauri : icon.png/icns/ico selon bundle.

### 16.3 Iconographie de vault

Options persistables : Home, Files, Database, Learning, Archive, Project, Favorite, Code, Workflow. Un alias historique `book` est normalisé vers `file-text`.

---

## 17. Matrice d’interactions utilisateur

| Domaine | Entrée | Effet visible | Effet applicatif | Échec attendu |
|---|---|---|---|---|
| Fenêtre | double-clic drag region | maximise/restaure | Tauri window API | warning log, état inchangé |
| Fenêtre | bouton close | fenêtre disparaît | `getCurrentWindow().close()` | warning log |
| Vault | open folder | shell library | registre + init hidden dirs | erreur vault visible |
| Vault | switch | contenu entièrement remplacé | config active vault | ancien contenu non actionnable |
| Sidebar | clic note | editor | read note + tab identity | pas de note fantôme |
| Library | clic card | editor/folder | IPC note/directory | carte reste visible si erreur |
| Library | dblclick title | input inline | rename après Enter | Escape restaure titre |
| Library | context menu | popover actions | rename/sidebar/delete | clic extérieur ferme |
| Library | drag/drop | highlight cible | move réel | cible self/descendant refusée |
| Editor | frappe | rendu Markdown live | command Rust + autosave | erreur save visible/loggée |
| Editor | Ctrl/Cmd+S | état sauvegardé | Tauri write | `tab-save-failure` |
| Editor | Ctrl/Cmd+Z/Y | document revient/avance | Rust history | état de sélection récupéré |
| Editor | checkbox task | coche unique | Rust setTaskChecked | propagation arrêtée |
| Editor | paste | Markdown structuré | clipboard conversion | data type inconnu ignoré |
| Editor | drop image | image + lien | copy `.assets` | collision évitée |
| Editor | tag context menu | confirmation Delete | update frontmatter | annulation conserve tag |
| Editor | click pin | pin jaune | localStorage scoped | path vide ignoré |
| Search | Ctrl/Cmd+K | modal overlay | query debounce 220 ms | erreur visible |
| Search | arrows/Enter | sélection/ouvre note | store.openResult | query vide ferme sur Esc |
| Settings | search | résultats réglages | section exact | no result explicite |
| Settings | switch | thumb + comportement live | préférence persistée | disabled si non applicable |
| Graph | clic node | preview + focus | caméra 520 ms | stage vide désélectionne |
| Canvas | drag node | position change | localStorage | pointerup finalise |
| Chat | Enter | message + réponse | RAG réel | erreur devient message |
| Sync | Sync now | statut/progression | service Iroh | `lastError` visible |
| Addon | Install | addon detail enabled | package registry | capability/runtime peut rester indisponible |
| Addon | Disable | contribution disparaît | disposer/lifecycle | pas de fallback caché |

---

## 18. Contrats d’accessibilité et de clavier

- Les boutons interactifs ont `type="button"` sauf forms submit.
- Les rails ont `aria-label` et les contrôles fenêtre ont title + aria-label.
- Les switches utilisent `role="switch"` et `aria-checked`.
- Le resizer utilise `role="separator"`, orientation verticale et min/max/current.
- Les modals Settings/Excalidraw utilisent `role="dialog"`, `aria-modal="true"`.
- Les erreurs principales utilisent `role="alert"`; chargements vault utilisent `role="status"` et `aria-live="polite"`.
- Les champs titre, recherche et tag ont des labels accessibles.
- Les icônes décoratives ont `aria-hidden="true"`.

Raccourcis globaux :

| Raccourci | Contexte | Effet |
|---|---|---|
| `Ctrl/Cmd+K` | vault actif | Search globale. |
| `Ctrl/Cmd+F` | note fermée | Search globale. |
| `Ctrl/Cmd+F` | note ouverte | Find interne de l’éditeur. |
| `Ctrl/Cmd+R` | shell | synchronise/recharge le workspace. |
| `Alt/Option+←` | shell | navigation arrière. |
| `Alt/Option+→` | shell | navigation avant. |
| `Ctrl/Cmd+S` | Excalidraw/éditeur | sauvegarde réelle. |
| `Escape` | Search | clear puis close. |
| `Escape` | tag/rename | annule l’édition. |
| `Escape` | Excalidraw | close ou prompt name. |
| `Enter` | éditeur | nouveau paragraphe. |
| `Tab` / `Shift+Tab` | cellule de table | cellule suivante/précédente. |
| `Enter` | Chat | envoyer sans Shift. |
| `Shift+Enter` | Chat | nouvelle ligne. |

---

## 19. Ce qui est prouvé par le code et ce qui reste à exécuter

### CODE / CONTRAT

- la géométrie CSS et les temps d’animation listés ;
- l’ordre de bootstrap et le rejet des runtimes non-Tauri ;
- les commandes et payloads Tauri exposés ;
- les garde-fous de chemin côté Rust ;
- la distinction visible entre no-vault, library, editor, search, settings et addon view ;
- la logique de pagination 120/72/720 ;
- le délai card 220 ms ;
- les politiques autosave 500/160/60/20 ms ;
- l’éditeur Rust et ses types d’événements ;
- les actions desktop/mobile et les états disabled/error.

### RUNTIME À EXÉCUTER

Pour transformer cette référence en preuve fonctionnelle complète, il faut encore exécuter sur le commit final :

1. `pnpm tauri:web:build` puis démarrage Tauri réel ;
2. parcours clean profile/vault : no-vault → choose folder → create note → edit → autosave → restart ;
3. deux vaults et isolation après switch/restart ;
4. carte simple clic vs double-clic et drag/drop physique ;
5. image/fichier externe avec vérification `.assets` et reload ;
6. IME, clipboard Markdown, tables, tasks, undo/redo et curseur après re-render ;
7. Search overlay au-dessus de la library et de l’éditeur ;
8. Settings et persistance de chaque préférence ;
9. Graph avec au moins 205 notes, sélection, zoom, simulation, timelapse ;
10. vrai service Knowledge/Open Models/Codex/OCR/Code Execution lorsque l’addon est installé ;
11. vraie paire Sync sur deux vaults ;
12. paquet desktop et, pour Android, APK/emulator avec safe areas, provider de documents, permissions et logcat.

Cette documentation ne revendique pas qu’un sidecar, un provider IA, une sync ou une vue d’addon fonctionne uniquement parce que son bouton est rendu.

---

## 20. Index des sources principales

### Shell et interface

- `Elephant/frontend/app/components/shell/AppShell.vue`
- `Elephant/frontend/app/components/shell/TopVaultBar.vue`
- `Elephant/frontend/app/components/shell/MainContent.vue`
- `Elephant/frontend/app/components/shell/EmptyVaultPicker.vue`
- `Elephant/frontend/app/components/navigation/NavigationBar.vue`
- `Elephant/frontend/app/components/navigation/IconRail.vue`
- `Elephant/frontend/app/components/navigation/SidebarNav.vue`
- `Elephant/frontend/app/components/navigation/SidebarTreeEntry.vue`
- `Elephant/frontend/app/styles/app-shell.css`
- `Elephant/frontend/app/styles/app-shell-runtime-fixes.css`
- `Elephant/frontend/src/renderer/src/mobile-native-ux.css`

### Library et état

- `Elephant/frontend/app/components/library/LibraryToolbar.vue`
- `Elephant/frontend/app/components/library/CreateEntryMenu.vue`
- `Elephant/frontend/app/components/library/LibraryGrid.vue`
- `Elephant/frontend/app/components/library/NoteCard.vue`
- `Elephant/frontend/app/components/library/FolderCard.vue`
- `Elephant/frontend/app/stores/vaultStore.js`
- `Elephant/frontend/app/stores/navigationStore.js`

### Éditeur

- `Elephant/frontend/app/components/editor/NoteEditorHost.vue`
- `Elephant/frontend/app/components/editor/NoteEditorTopBar.vue`
- `Elephant/frontend/app/components/editor/NoteEditorToolbar.vue`
- `Elephant/frontend/app/components/editor/NoteEditorFooter.vue`
- `Elephant/frontend/app/components/editor/NoteTypographyMenu.vue`
- `Elephant/frontend/app/components/editor/NoteTagChip.vue`
- `Elephant/frontend/app/components/editor/NoteTagForm.vue`
- `Elephant/frontend/app/components/editor/ExcalidrawDialog.vue`
- `Elephant/frontend/src/renderer/src/editor-rust/inputController/**`
- `Elephant/frontend/src/renderer/src/platform/tauriMarkTextSaveBridge.js`
- `Elephant/frontend/src/renderer/src/platform/tauriLocalIpcBridge.js`

### Search, workspace et addons

- `Elephant/frontend/app/search/SearchModal.vue`
- `Elephant/frontend/app/search/SearchResultItem.vue`
- `Elephant/frontend/app/search/SearchSettingsPanel.vue`
- `Elephant/frontend/app/search/SearchStatusBadge.vue`
- `Elephant/frontend/app/components/views/AtomicGraphView.vue`
- `Elephant/frontend/app/components/views/CanvasView.vue`
- `Elephant/frontend/app/components/views/ChatView.vue`
- `Elephant/frontend/app/components/views/CalendarAddonWorkspace.vue`
- `Elephant/frontend/app/components/views/AddonWorkspaceRouter.vue`
- `Elephant/frontend/app/components/views/AddonWorkspaceHost.vue`
- `Elephant/frontend/app/components/settings/SettingsPanel.vue`
- `Elephant/frontend/app/components/settings/AddonsSettingsPanel.vue`
- `Elephant/frontend/app/components/settings/SyncSettingsPanel.vue`
- `addons/official/*/manifest.json`
- `addons/official/*/main*.js`

### Runtime Tauri/Rust

- `Elephant/frontend/src/renderer/src/main.js`
- `vite.tauri.config.mjs`
- `Elephant/backend/tauri/tauri.conf.json`
- `Elephant/backend/tauri/tauri.android.conf.json`
- `Elephant/backend/tauri/tauri.linux.conf.json`
- `Elephant/backend/tauri/capabilities/default.json`
- `Elephant/backend/tauri/src/lib_min.rs`
- `Elephant/backend/tauri/src/vault/commands.rs`
- `Elephant/backend/tauri/src/vault_layout.rs`
- `Elephant/backend/tauri/src/core_commands.rs`
- `Elephant/backend/tauri/src/markdown/commands.rs`
- `Elephant/backend/tauri/src/watcher.rs`
- `Elephant/backend/tauri/src/addons.rs`

---

## 21. Conclusion produit

Dans sa version Tauri actuelle, Elephant n’est pas un simple éditeur Markdown avec une sidebar. C’est un shell local-first qui combine :

- un vault réel sur disque ;
- une bibliothèque paginée de notes et dossiers ;
- une navigation persistante et un rail d’addons reconfigurable ;
- un éditeur Rust/Muya événementiel, autosauvegardé, sensible aux chemins, aux assets, aux tables et aux IME ;
- une recherche overlay avec états d’index explicites ;
- une surface de réglages dense, thèmeable et extensible ;
- des workspaces de graph, canvas, chat, calendrier, Wiki et modèles ;
- des addons physiques avec permissions, workers, services et sidecars ;
- un backend Rust qui valide les chemins et maintient la cohérence disque/UI.

Le caractère essentiel de l’app est la chaîne complète :

```text
interaction utilisateur
  → composant visible
  → store / bridge
  → commande Tauri ou addon
  → fichier / index / service réel
  → état retourné
  → UI de succès ou d’erreur
  → logs et nettoyage
```

Une interface n’est donc complète que lorsque son effet réel, sa persistance, son état d’erreur et son cycle de vie sont cohérents avec ce qui est dessiné à l’écran.

---

## 22. Surfaces legacy encore présentes dans le runtime Tauri

Le shell moderne n’a pas supprimé toutes les primitives historiques. `app.vue` monte encore `CommandPalette`, `AboutDialog`, `ExportSettingDialog`, `Rename`, `Tweet` et `ImportModal`. `EditorWithTabs` utilise toujours `runtimeEditor.vue`, même quand sa tab bar est masquée par `NoteEditorHost`.

Cette section décrit donc des surfaces atteignables par bus, commandes, compatibilité ou plugins. Elles ne doivent pas être confondues avec les contrôles modernes documentés plus haut.

### 22.1 Command palette legacy

Source : `Elephant/frontend/src/renderer/src/components/commandPalette/index.vue`.

Géométrie :

- `el-dialog` modal, largeur 500 px ;
- wrapper positionné absolute, width 500 px, padding 8 px, margin-top 8 px, top 0, left 50%, transform `translateX(-50%)` ;
- border 1 px, rayon 4 px, shadow `0 3px 8px 3px var(--floatShadow)` ;
- input wrapper width 100%, border 1 px, rayon 3 px ;
- input 100% × 30 px, font 14 px, margin horizontal 10 px ;
- résultat list max-height 300 px, scroll vertical ;
- ligne résultat height 35 px, padding horizontal 8 px, font 14 px, line-height 35 px ;
- raccourci font 12 px ;
- loader zone height 50 px, margin-top 8 px.

Comportement :

- bus `show-command-palette` initialise une commande racine, récupère ses sous-commandes puis ouvre le dialog ;
- le focus de l’input est programmé 50 ms après l’ouverture ;
- ArrowUp/ArrowDown bouclent dans la liste et scrol lent l’élément actif dans la vue ;
- Enter exécute l’élément sélectionné ;
- la recherche filtre la description case-insensitive, ou délègue à `currentCommand.search()` ;
- une commande qui ne possède que des sous-commandes descend d’un niveau sans fermer la palette ;
- un changement de langue recharge la liste si la palette est ouverte ;
- la fermeture remet query, index et commande à zéro et appelle `unload` si présent ;
- `editor-blur` est émis à l’ouverture.

L’entrée est animée par fade opacity en **200 ms**. Un clic sur une ligne a le même effet que Enter.

### 22.2 Tab bar historique

Source : `editorWithTabs/tabs.vue`. Cette bar est masquée par `NoteEditorHost :show-tab-bar="false"`, mais le store et les événements restent actifs.

- hauteur 35 px ;
- tabs scrollables horizontalement ;
- chaque tab : hauteur 35 px, padding 0 8 px, font 12 px, largeur max 280 px, ellipsis ;
- rayon haut gauche/droit 15 px ;
- tab active : fond `--itemBgColor`, underline accent 2 px ;
- tab non sauvegardée : dot accent 7 × 7 px ; au hover le dot devient l’icône close ;
- close icon : SVG 12 × 12 px, visible au hover ou si unsaved ;
- bouton new file : 35 × 35 px, opacity 0 sauf hover de la bar.

Interactions :

- clic : sélectionne le fichier ;
- clic middle : ferme ;
- clic close : ferme, avec fermeture forcée si saved ou flux de confirmation si dirty ;
- clic droit : menu contextuel de tab ;
- wheel : priorité à `deltaX`, sinon `deltaY`, pour faire défiler horizontalement ;
- drag horizontal via Dragula : réordonne les tabs, revert-on-spill, auto-scroll lorsque le pointeur est à 20 px d’une extrémité, vitesse max 6 ;
- bus : close this, close others, close saved, close all, rename, copy path, show in folder et change max width.

Transition tab : `all 150 ms ease-in-out`. Le miroir de drag reçoit opacity .8, le transit opacity .2 et le curseur devient `grabbing`.

### 22.3 Menu contextuel des tabs

Source : `contextMenu/tabs/menuItems.js`.

Actions :

- Close this ;
- Close others ;
- Close saved tabs ;
- Close all tabs ;
- Rename ;
- Copy path ;
- Show in folder.

Chaque item transporte l’ID du tab via `_tabId` et émet l’événement bus correspondant. Le clipboard utilise le bridge Tauri ; Show in folder utilise l’opener/shell de la plateforme.

### 22.4 Dialog de renommage legacy

Source : `components/rename/index.vue`.

- `el-dialog` modal, width 410 px, sans close button ;
- wrapper width 410 px, padding horizontal 8 px ;
- input height 30 px, font 14 px, padding horizontal 8 px ;
- icône Markdown 30 × 30 px, marge 0 5 px ;
- le bus `rename` ouvre le dialog et préremplit le nom du fichier courant ;
- focus automatique ; Enter appelle `editorStore.rename(tempName)` ;
- clic sur l’icône fait aussi commit ;
- fermeture remet le dialog à false.

L’icône change de couleur en 300 ms ease-in-out au hover.

### 22.5 Import legacy par drag/drop

Source : `components/import/index.vue`.

- dialog modal width 450 px ;
- zone drop border dashed 1 px, rayon 5 px, no-drag ;
- image import : wrapper 50 × 70 px, margin-top 40 px ;
- formats annoncés sous forme de cinq cellules 70 × 70 px : `.md`, `.html`, `.docx`, `.tex`, `.wiki` ;
- zone active : border accent dashed et fond `--itemBgColor` ;
- bus `importDialog` contrôle l’ouverture ;
- dragenter/dragover activent l’état ; dragleave le retire ; drop récupère les chemins via `webUtils.getPathForFile`, `file.path`, `webkitRelativePath` ou `name` ;
- le renderer envoie `mt::window::drop` avec la liste des fichiers.

Le shell global utilise un timer de **300 ms** pour ne pas fermer immédiatement la zone import pendant les mouvements de drag.

### 22.6 About dialog

Source : `components/about/index.vue`.

- dialog modal width 400 px ;
- logo 80 × 80 px centré ;
- titre et textes min-height 32 px, text-align center ;
- le bus `aboutDialog` ouvre la fenêtre et émet `editor-blur` ;
- affiche nom `Elephant`, version applicative, copyright année courante et contributeurs.

### 22.7 Export/print settings

Source : `components/exportSettings/index.vue`.

- dialog modal width 500 px ;
- body padding 0 20 px 20 px ;
- contenu tabs max-height 350 px avec scrollbar verticale 5 px ;
- onglets : Info, Page, Style, Theme, Header/Footer si PDF/print, TOC ;
- bouton export aligné à droite, top margin 8 px.

#### Valeurs initiales et bornes

| Contrôle | Valeur/bornes |
|---|---|
| page size | `A4`, ou Custom. |
| largeur/hauteur custom | 100 minimum. |
| orientation | portrait par défaut, booléen landscape. |
| marges top/bottom/left/right | 0–100, initiales 20/20/15/15. |
| police | `Default` par défaut. |
| taille | 8–32 px, initiale 14. |
| line height | 1.0–2.0 par .1, initiale 1.5. |
| auto numbering | false. |
| front matter | false. |
| header/footer type | 0 par défaut. |
| header/footer font | 8–20 px, initiale 12. |
| TOC top heading | true. |

Le mode `styledHtml` masque Page et Header/Footer, et ajoute le titre HTML. Le mode PDF produit du HTML print-optimized puis appelle le print service ; le mode print ignore taille/orientation. Une erreur de PDF/print déclenche notification et cleanup du print service.

### 22.8 Ancienne sidebar MarkText

Source : `components/sideBar/**`.

La sidebar legacy est distincte de `app/components/navigation/SidebarNav.vue` :

- largeur compacte 45 px sans panneau ;
- largeur de vue par défaut 280 px, minimum 220 px ;
- colonne gauche 45 px, padding-top 40 px ;
- boutons icône 45 × 45 px ;
- colonne droite flex, overflow hidden ;
- drag bar 3 px, cursor col-resize ;
- vue animée avec transform **250 ms ease-in-out**.

Icônes : Files, Search, TOC, Settings. Clic répété sur la vue active la ferme ; clic sur Settings ouvre la fenêtre de réglages legacy.

#### Tree legacy

- section Opened Files pliable, hauteur max 200 px, scroll 8 px ;
- section project tree pliable ;
- ligne dossier/fichier 30 px ;
- input création height 22 px, margin vertical 5 px, width 70% ;
- bouton `Save all` et `Close all` visibles au hover du titre de section ;
- dossier : icône open/close, clic inverse `isCollapsed` ;
- fichier non Markdown : opacity .75, clic ignoré ;
- fichier courant : barre accent 2 px à gauche dont la hauteur passe à 100% en transition 200 ms ;
- fichier dirty dans les opened files : dot 7 × 7 px ;
- entrée créée avec Enter ; click extérieur, contextmenu ou Escape annule create/rename cache.

#### Context menu legacy sidebar

Source : `contextMenu/sideBar/menuItems.js`.

Actions : New file, New directory, Copy, Cut, Paste, Rename, Move to trash, Show in folder. Elles passent toutes par le bus `SIDEBAR::*`; elles ne sont donc pas un simple changement visuel.

#### Recherche dans le dossier

La vue Search legacy dispose d’un champ 28 px de haut dans une capsule de 28 px, margin 37px 15px 10px, et de trois toggles 20 × 20 px : case-sensitive, whole-word, regex.

- recherche uniquement les extensions Markdown ;
- utilise ripgrep et les préférences exclusions, max file size, hidden, no-ignore, symlinks ;
- limite à 100 fichiers avec matches ;
- affiche un bouton cancel après **500 ms** de recherche ;
- résultats pliables ; auto-ouverts si ≤20 matches ;
- affiche 10 matches puis +15 ; Ctrl/Cmd affiche tout ;
- clic sur une ligne positionne le curseur/selection dans le tab ou ouvre le fichier avec cursor ;
- match highlight line-height/height 16 px ;
- compteur pill min-width/height 18 px, rayon 9 px.

### 22.9 Recherche Find/Replace dans le document

Source : `components/search/index.vue`.

- barre absolute width 400 px, top 0, right 20 px ;
- flèche gauche 20 px, icône 12 px ; elle pivote -90° lorsque le mode est Search ;
- chaque ligne Search/Replace height 28 px ;
- boutons prev/next 28 × 28 px, icônes 16 px ;
- champ principal height 26 px, font 14 px ;
- compteur `current / total` ;
- toggles case-sensitive, whole-word et regex, cellules 20 × 20 px ;
- mode replace : champ replacement + Replace All (`∞`) + Replace Single ;
- tooltip replace ouvre après 1000 ms ;
- query debounced de **150 ms** ;
- Enter passe au match suivant ;
- Escape, clic document ou `search-blur` ferme et vide ;
- regex invalide affiche une erreur ; regex qui matche la chaîne vide est refusée ;
- Find Previous/Next émet `find-action` ; replace émet `replaceValue` avec `isSingle` et les options.

---

## 23. Éditeur Muya : menus flottants et transformations

### 23.1 Conteneur commun

Tous les menus Muya passent par `.ag-float-wrapper` : absolute, font 12 px, opacity initiale 0, top/right hors écran avant placement, shadow `--floatShadow`, rayon 2 px, z-index 10000, overflow hidden et transition d’opacité **250 ms ease-in-out**. Popper passe l’élément à opacity 1 et le triangle fait 16 × 16 px, rotation 45°.

### 23.2 Quick insert `/`

Le menu est visible lorsque le trigger `/` est tapé et que `hideQuickInsertHint` est faux.

- width 360 px ;
- max-height 307 px ;
- padding bottom 20 px ;
- titre sticky, padding 10 × 14 px, font 14 px uppercase, letter-spacing 1 px ;
- item padding vertical 6 px ;
- icon box 40 × 40 px, margin-left 14 px, border 1 px, rayon 6 px ;
- icône interne 20 × 20 px ;
- texte principal 14 px, sous-texte 12 px ;
- raccourci width 70 px, margin-right 20 px, font 14 px ;
- hover/actif : fond `--floatHoverColor`.

Groupes et commandes natives :

| Groupe | Entrées |
|---|---|
| Writing tools | Bold, Italic, Strikethrough, Link, Inline Code, puis quick insert items d’addons. |
| Basic block | Paragraph, Horizontal line, Front matter. |
| Header | Heading 1 à Heading 6. |
| Advanced block | Table, Math block, HTML, fenced code, Blockquote. |
| List block | Ordered list, Bullet list, Todo list. |
| Diagram | Vega-Lite, Flowchart, Sequence, PlantUML, Mermaid. |

Raccourcis générés : `Ctrl/Cmd+0` paragraph ; `Ctrl/Cmd+1..6` headings ; `Shift+Ctrl/Cmd+T` table ; `Alt+Ctrl/Cmd+-` HR ; `Alt+Ctrl/Cmd+Y` front matter ; `Alt+Ctrl/Cmd+M` math ; `Alt+Ctrl/Cmd+J` HTML ; `Alt+Ctrl/Cmd+C` code ; `Alt+Ctrl/Cmd+Q` quote ; `Alt+Ctrl/Cmd+O/U/X` ordered/bullet/task.

Le bridge `writingCommandBridge` supprime la query `/...` avant d’exécuter les commandes `bold`, `italic`, `strike`, `link`, `bullets`, `numbers`, `tasks`, `code`, `quote`, `table`, `horizontal-rule`. Deux exécutions identiques dans une fenêtre de **350 ms** sont dédupliquées.

### 23.3 Format picker

- width 265 px, height 28 px ;
- dix items de 35 px : Bold, Italic, Underline, Strike, Highlight, Inline Code, Inline Math, Link, Image, Clear Formatting ;
- icônes 16 × 16 px ;
- dernier groupe séparé par une ligne de 1 × 18 px ;
- active : couleur `--themeColor` ;
- hover : `--floatHoverColor` ;
- chaque icône possède un tooltip et, lorsque défini, le raccourci : `Ctrl/Cmd+B`, `I`, `U`, `D`, `Shift+Ctrl/Cmd+H`, backtick, `Shift+Ctrl/Cmd+M`, `Ctrl/Cmd+L`, `Shift+Ctrl/Cmd+I`, `Shift+Ctrl/Cmd+R`.

### 23.4 Front menu / menu de bloc

- width 190 px, font 14 px ;
- liste padding vertical 10 px ;
- item hauteur 28 px, icône 16 px, hover float ;
- entrées racine : Duplicate, Turn into, New paragraph, Delete ;
- raccourcis : `Shift+Ctrl/Cmd+P`, `Shift+Ctrl/Cmd+N`, `Shift+Ctrl/Cmd+D` ;
- Delete utilise `--deleteColor` ;
- Turn into ouvre un submenu width 190 px, max-height 300 px, left `100% + 3px`, top -38 px, transform scale(0) → scale(1), opacity 0 → 1 en **250 ms** ;
- le submenu adapte son contenu au bloc : paragraph, heading, list, quote, code, table, HTML, math, HR, front matter.

### 23.5 Image toolbar Muya

- width 240 px, height 30 px ;
- items 28 × 30 px ;
- icônes 16 × 16 px ;
- actions : edit, inline, align left, center, right, delete ;
- tooltip au-dessus min-width 100 px, height 30 px, hidden par défaut, visible au hover ;
- des séparateurs 1 × 16 px organisent edit/open, inline et delete ;
- actions désactivées sont opacity .4 ;
- transition des icônes : **250 ms**.

### 23.6 Link tools

- width 100 px, height 34 px ;
- deux boutons 28 × 28 px, rayon 8 px, marges 11 px ;
- `unlink` et `jump` ;
- séparateur 1 × 18 px ;
- clic link avec Ctrl/Cmd ouvre le lien ; clic image avec Ctrl/Cmd ouvre Excalidraw ;
- hover : fond floatHover.

### 23.7 Table picker

- width 153 px, padding 10 px, font 12 px ;
- grille initiale 6 lignes × 8 colonnes ;
- cellule 16 × 16 px, border 1 px, gap 1 px ;
- hover montre la sélection rectangulaire rows × columns ;
- footer avec inputs row/column height 20 px, input width 30 px et bouton OK ;
- ArrowUp augmente le nombre, ArrowDown le diminue, Enter confirme ;
- clic sur une cellule confirme la table ;
- les nombres exposés sont base 1 alors que l’état interne est base 0.

### 23.8 Table bar tools

- width 150 px, liste padding 5 px ;
- items height/line-height 25 px, padding 0 8 px, font 13 px ;
- actions left : insert row above, insert row below, remove row ;
- actions bottom : insert column left, right, remove column ;
- remove est séparé par une ligne et margin-top 10 px ;
- hover : fond floatHover.

### 23.9 Emoji, code language, footnote et tooltip

- emoji picker width 348 px, max-height 350 px ; titre sticky font 12 px uppercase ; cellule 36 × 36 px ronde, emoji 24 × 24 px, scale transition .2 s ;
- code picker width 184 px, max-height 196 px ; ligne langue 30 px, icône 13 px, alias monospace 10 px ; no-result padding 8 × 10 px ;
- footnote tool width 300 px, chaque ligne 35 px, bouton width 40 px, icône 14 px ;
- tooltip : padding 5 × 7 px, rayon 5 px, font 12 px, transition opacity .2 s ease-in ; actif opacity 1 et translateY(-5 px) ; triangle 8 × 8 px ;
- image selector : width `min(560px, calc(100vw - 48px))`, min-height 190 px, max-height 400 px, rayon 16 px ; tabs header min-height 44 px ; search pill height 34 px ; thumbs 110 × 60 px dans des cartes 112 × 80 px ; spinner 22 px animé **1.1 s linear infinite**.

### 23.10 Transformer / redimensionnement image

Les poignées de transformation sont des cercles 14 × 14 px, border 2 px couleur du fond éditeur, shadow 0 2px 2px, curseurs `nwse-resize` et `nesw-resize`. Le résultat est réécrit dans le Markdown, pas seulement dans le style DOM temporaire.

---

## 24. Catalogue détaillé des addons officiels

Les manifests sont sous `addons/official/*/manifest.json`. Les versions et capacités ci-dessous sont celles lues dans le dépôt inspecté.

| ID | Nom/version | Surface visible | Effet runtime |
|---|---|---|---|
| `elephant.ai` | AI 2.2.0 | section AI dans Settings | provider API, secrets et inference versionnée. |
| `elephant.ai-chat` | Chat 2.0.0 | zone `shell.right`, item rail/sidebar Chat, section AI | chat RAG, citations, actions revues, stream. |
| `elephant.ai-search` | Semantic Search 1.3.0 | slot AI Search dans Settings | index lexical, vectors, fallback et query. |
| `elephant.ai-ocr` | OCR 1.0.0 | section OCR dans Settings | sidecar OCR desktop. |
| `elephant.calendar` | Calendar 1.3.0 | vue Calendar + import ICS | events package-owned par vault. |
| `elephant.code-execution` | Code execution 2.2.0 | toolbar dans code fences + section Editor | interpréteurs, Run/Stop, sortie bornée. |
| `elephant.codex-connection` | Codex Connection 2.0.0 | section ChatGPT subscription + provider | auth, modèles, usage, chat package service. |
| `elephant.dashboard` | Dashboard 1.0.1 | item rail Dashboard | note Dashboard cachée par vault. |
| `elephant.google-keep-import` | Google Keep Import 1.2.0 | section Import | JSON Takeout → Markdown sous `Imported/Google Keep/**`. |
| `elephant.graph` | Graph 1.3.0 | vue Graph | SVG relationnel package-owned. |
| `elephant.knowledge` | Knowledge 1.2.0 | section Knowledge index | index, chunks, embeddings, graph, Wiki. |
| `elephant.open-models` | Open Models 2.0.0 | vue Models + provider | GGUF, llama.cpp service. |
| `elephant.recently-edited` | Recently edited 1.1.0 | section après l’arbre sidebar | notes récentes. |
| `elephant.sites` | Sites 1.4.0 | section Sites | preview/build statique scoped. |
| `elephant.sync` | Sync 1.2.0 | topbar Sync + section Sync | pairing Iroh, sync bidirectionnelle, conflits. |
| `elephant.wiki` | Wiki 1.5.0 | vue Wiki | propositions, pages et citations. |

### 24.1 AI : providers externes

La section `AI` possède une nav tab dynamique : Providers puis les slots de Chat, Knowledge, Search, OCR, Codex et Open Models selon les contributions présentes.

La page Providers rend une carte `External API providers` :

- bouton `Add provider`, min-height 34 px, padding horizontal 11 px ;
- état vide `No external API provider configured.` ;
- grille provider en deux colonnes, une colonne sur mobile ≤760 px ;
- champs : Type select, Name text, Base URL url en largeur wide, API key password, Enabled checkbox, Chat model text, Embedding model text ;
- placeholders : `Chat model id`, `Embedding model id` ;
- bouton Remove par provider ;
- chaque `change` programme une sauvegarde après **700 ms**.

La configuration persistée contient le provider, endpoint, clé secrète, modèle chat, modèle embedding et enabled. Les clés ne doivent pas être loggées.

### 24.2 Chat physique dans shell.right

Le Chat officiel contribue une zone `shell.right`, visible seulement lorsque `chatSidebarOpen === true`. Il existe donc deux surfaces de Chat dans le code :

1. `frontend/app/components/views/ChatView.vue`, vue historique de workspace ;
2. `addons/official/ai-chat/main.js`, Chat physique actuellement branché comme panneau droit.

Le panneau physique :

- largeur minimale 340 px ;
- grille de haut en bas : topbar / history / composer ;
- border-left 1 px ;
- topbar padding 14 × 18 px 10 px ;
- status `source · model` en 11 px ;
- history width `min(260px, 80%)`, padding 18 × 14 px, translation X -100%, transition **200 ms** dans le style package et **220 ms** dans le style shared v2 ;
- empty state max-width 520 px, margin-top 10vh ; quick prompts en 2 colonnes, une colonne ≤700 px ;
- user message : margin-left 36 px, assistant : margin-right 20 px ; cards padding 12 px, border 1 px, rayon 14 px ;
- tool/citation pills min-height 32 px, rayon 9 px ;
- form padding 12 px, border top 1 px ;
- bouton `Arrêter` visible pendant le stream ;
- l’état du provider et du modèle est visible, ainsi que `Recherche et génération en cours…`.

Actions : historique, nouvelle conversation, recherche conversation, sélection conversation, close panel, envoi, stop generation, citations ouvrables, approbation/refus de propositions d’écriture. Les actions AI en écriture peuvent être auto-approuvées selon la configuration, mais le défaut reste la proposition revue.

### 24.3 Semantic Search addon

La section `Search` est une carte padding 16 px, border 1 px, rayon 14 px.

Champs :

- Enabled checkbox ;
- Result limit number borné 1–100 ;
- Rebuild automatically checkbox ;
- Generate semantic vectors checkbox.

Actions :

- `Rebuild index and vectors` : désactive le bouton, affiche `Reading notes, rebuilding Knowledge and generating pending vectors…`, puis réactive ;
- `Clear lexical fallback` : vide le fallback lexical ;
- statut : nombre de notes lexicales, vectors disponibles, raison de semantic fallback et état build.

La grille est 2 colonnes desktop, une colonne sous 760 px. Le addon dépend du Knowledge pour l’index et de l’AI pour les embeddings ; un statut absent doit rester visible et ne doit pas être présenté comme du semantic search exécuté.

### 24.4 OCR

La section OCR contient :

- header `OCR sidecar` ;
- badge `Checking…`, puis `Ready · <platform>` ou `Unavailable` ;
- champ Languages, placeholder `eng,fra` ;
- select Output : Plain text ou Markdown ;
- champ Test image path, placeholder `/path/to/image.png` ;
- boutons `Refresh status` et `Run OCR` ;
- pre feedback min-height 80 px, max-height 280 px, scroll, font 11 px monospace.

Le changement de langue/output persiste les préférences. Run OCR désactive le bouton, affiche `Running OCR…`, appelle le sidecar, rend le texte ou l’erreur, puis réactive. Le manifest précise que le sidecar process n’est pas supporté Android/iOS.

### 24.5 Calendar physique et Calendar workspace

L’addon Calendar contient aussi deux niveaux : vue package `calendar-v3` et wrapper moderne `CalendarAddonWorkspace`.

La vue physique package :

- padding 18 px, gap 14 px, overflow auto ;
- header `Calendar` + nombre d’events ;
- file input multiple `.ics,.ical,text/calendar`, min-width 260 px, padding 10 px, border dashed, rayon 10 px ;
- `Import ICS` disabled tant qu’aucun fichier n’est sélectionné ;
- `Clear events` ;
- feedback `<pre>` min-height 18 px ;
- event card padding 14 px, border, rayon 12 px, gap interne 5 px ;
- import accepte plusieurs fichiers, parse VEVENT, affiche imported/failures.

La vue moderne importe Google Calendar via contribution, tandis que la vue physique importe ICS local. Ce sont deux sources et deux parcours distincts.

### 24.6 Graph physique package

La vue `graph-v3` est différente de `AtomicGraphView` :

- min-height 480 px, padding 16 px, gap 12 px ;
- header avec `Graph`, counts nodes/edges et bouton Rebuild min-height 34 px ;
- SVG width/height 100%, border 1 px, rayon 14 px ;
- node circle radius `min(13, 6 + sqrt(degree))`, stroke 2 px ;
- labels SVG font 11 px ;
- liens tag en dash `3 4`, opacity .45 ;
- clic sur un groupe node dispatch `elephantnote:open-note` avec path ;
- état pendant rebuild : `Building graph…` ;
- état vide : `No note relationship yet.` ;
- erreur : texte danger.

La construction produit un edge `link` pour les liens explicites, et relie linéairement les notes partageant un tag, sur un bucket limité aux 40 premiers IDs par tag.

### 24.7 Wiki physique package

La vue `ai-wiki-v3` :

- padding 18 px, gap 14 px, scroll ; mobile padding 12 px ;
- header `Wiki`, nombre de pages/propositions ;
- boutons `Propose with AI` et `Refresh`, min-height 34 px, gap 8 px ;
- liste grid auto-fill min 280 px, gap 12 px, une colonne ≤760 px ;
- record card padding 14 px, border, rayon 13 px, gap 8 px ;
- affiche titre, résumé, qualité/origine/nombre de sources ;
- proposition : `Approve` et `Refuse` ; accepté : bordure success color-mix.

### 24.8 Open Models

La vue `open-models-v3` :

- padding 18 px, gap 14 px, scroll ;
- header `Open Models` + nombre de modèles locaux ;
- input min-width 260 px, min-height 34 px, placeholder `Hugging Face repository or direct GGUF URL` ;
- Download et Refresh min-height 34 px ;
- Download devient `Downloading…`, disabled, timeout de service 120 secondes ;
- liste en grid auto-fill min 260 px, gap 12 px ;
- model card padding 14 px, rayon 13 px, gap 9 px ;
- affiche nom, filename, taille MB et status ;
- bouton Activate/Deactivate et Remove ;
- activation/désactivation passe par le service package-owned.

### 24.9 Codex Connection

La section `ChatGPT subscription` affiche :

- état `Connected` ou `Disconnected` ;
- bouton `Connect` ou `Disconnect` ;
- bouton `Refresh` ;
- compte si connu ;
- nombre de modèles ;
- limites hourly/weekly si disponibles ;
- runtime path si fourni ;
- erreur en danger.

Connect appelle login/logout. Si login renvoie une URL, elle est ouverte avec l’opener Tauri ou une fenêtre externe. Le service package doit prouver l’authentification et le chat ; le simple état Connected ne suffit pas.

### 24.10 Knowledge

La section `Knowledge index` :

- texte de statut initial `Loading knowledge index status…` ;
- bouton `Rebuild knowledge index` ;
- résultat : `N notes · N chunks · N links · N semantic vectors (model)` ;
- pendant rebuild : `Rebuilding the package-owned knowledge index…` ;
- après : `Indexed X; unchanged Y; removed Z.` ;
- timeout rebuild : 30 minutes ;
- erreur rendue dans le même statut.

Le package démarre le service natif au `onload`, fournit la resource `knowledge`, et le stoppe au `onunload`.

### 24.11 Google Keep Import

La section Import :

- file input multiple, accept `application/json,.json` ;
- checkbox Include trashed ;
- bouton `Import selected files` disabled sans sélection ;
- status pre min-height 80 px, max-height 280 px, padding 12 px, border/rayon 12 px, font 11 px ;
- import écrit sous `Imported/Google Keep/**` ;
- le bouton devient disabled pendant l’import, puis affiche le rapport.

### 24.12 Code execution

Dans un bloc fenced :

- toolbar en haut du bloc, min-height 32 px, padding horizontal 8 px, font 11 px ;
- langage à gauche ; actions à droite, gap 4 px ;
- Copy et Run min-width 48 px/min-height 26 px ; mobile 58 × 40 px ;
- Copy devient `Copied` pendant **900 ms** ;
- Run devient `Starting…`, puis `Stop` ;
- Stop devient `Stopping…`, disabled jusqu’au retour ;
- résultat `<pre>` padding 10 px, max-height 260 px, scroll, bordure supérieure, white-space pre-wrap ;
- un échec expose le message et l’execution id/log dans l’état prévu.

Settings :

- checkbox Retain output ;
- select Default interpreter ;
- lignes d’interpréteur avec id, label, executable, arguments ;
- bouton Test ;
- bouton Remove danger ;
- bouton Add interpreter ;
- champs min-height 34 px, padding horizontal 9 px, border, rayon 8 px ;
- liste responsive : une colonne sous 720 px.

---

## 25. Protocole éditeur Rust, commandes natives et persistance renderer

### 25.1 Protocole Rust/Muya

Source : `frontend/src/renderer/src/editor-rust/protocol.js`, `bridge.js`, `RustMuyaRuntimeEditor.vue`.

Version de protocole : `1`.

Une requête contient toujours :

```json
{
  "protocol_version": 1,
  "expected_revision": 12,
  "command": { "type": "insert_text", "text": "A" }
}
```

Réponses possibles : `snapshot`, `update`, `error`. Toute réponse doit avoir un payload objet. Une révision invalide, un type inconnu ou un payload absent est une erreur de protocole.

`update` contient une révision, une sélection et une liste de patches. `snapshot` contient l’état complet. Si l’application des patches échoue, le bridge passe `desynchronized = true`, expose `ElephantRustPatchError` et exige `recover()` avant une nouvelle mutation.

La queue `_tail` sérialise `dispatch`, `setSelection` et `snapshot`. Les commandes ne peuvent pas être exécutées dans le désordre.

### 25.2 Commandes Rust exposées par le protocole

| Famille | Commandes |
|---|---|
| état | `snapshot`, `set_selection` |
| texte | `insert_text`, `paste_markdown` |
| paragraphes | `insert_paragraph`, `set_paragraph`, `insert_paragraph_after_block`, `duplicate_block`, `delete_block` |
| titres | `set_heading(level)` |
| inline | `toggle_strong`, `toggle_emphasis`, `toggle_strike` |
| blocs | `toggle_block_quote`, `toggle_code_block`, `insert_horizontal_rule` |
| listes | `set_list_kind`, `indent_list_item`, `outdent_list_item`, `set_task_checked` |
| tableaux | `create_table`, `insert_table_row_after`, `delete_table_row`, `insert_table_column_after`, `delete_table_column`, `next_table_cell`, `previous_table_cell` |
| images | `insert_image`, `replace_image`, `delete_image` |
| historique | `undo`, `redo` |
| IME | `begin_composition`, `update_composition`, `commit_composition`, `cancel_composition` |

### 25.3 Synchronisation prop → runtime

`RustMuyaRuntimeEditor` protège l’éditeur contre une boucle de re-render :

- il marque la première vraie mutation utilisateur via `beforeinput`, paste ou drop ;
- une modification programmatique reçue avant cette frontière est ignorée pour éviter qu’un état initial ne soit publié comme une frappe utilisateur ;
- les patches reçus sont synchronisés avec un timer 0 ms ;
- une réponse qui modifie le Markdown émet `update:modelValue` et `change` seulement après une mutation utilisateur ;
- une prop différente du Markdown runtime remonte l’éditeur avec une nouvelle génération ;
- la génération précédente est détruite si elle finit après un remount ;
- onBeforeUnmount détruit le runtime, annule les timers et détache les listeners.

En cas d’erreur, le shell Rust affiche une alerte absolute inset horizontal 16 px, top 16 px, padding 12 × 14 px, border danger 1 px, rayon 6 px, font 13 px, et ajoute un événement dans `window.__ELEPHANT_DEBUG_LOGS__` avec revision, selection et longueur Markdown.

### 25.4 Commandes Tauri Markdown/Muya

Source : `backend/tauri/src/markdown/commands.rs` et `tauriElephantNoteBridge.js`.

Le bridge expose :

- Markdown : `tauri_markdown_parse`, `tauri_markdown_render_html`, `tauri_markdown_to_text`, `tauri_markdown_extract_frontmatter`, `tauri_markdown_extract_links` ;
- compatibilité Muya : `tauri_muya_parse`, `tauri_muya_render_html`, `tauri_muya_tokens`, `tauri_muya_extras`, `tauri_muya_contract` ;
- clipboard : `tauri_muya_clipboard`, `tauri_muya_copy_markdown`, `tauri_muya_copy_html`, `tauri_muya_paste` ;
- mutations : `tauri_muya_backspace`, `tauri_muya_remove_next`, `tauri_muya_undo`, `tauri_muya_redo`, `tauri_muya_move_cursor`, `tauri_muya_input_rule` ;
- table/image : `tauri_muya_table_insert_row`, `tauri_muya_table_insert_column`, `tauri_muya_table_contract`, `tauri_muya_image_selection` ;
- composition : `tauri_muya_start_composition`, `tauri_muya_update_composition`, `tauri_muya_commit_composition`, `tauri_muya_cancel_composition`, `tauri_muya_editor_snapshot`.

### 25.5 Familles de commandes natives supplémentaires

Le registre `lib_min.rs` installe aussi :

- plateforme : `healthcheck`, `tauri_platform_info` ;
- fichiers binaires : `tauri_vault_read_binary`, `tauri_vault_write_binary`, `tauri_vault_ensure_dir`, `tauri_vault_remove_path`, `tauri_vault_rename_path` ;
- préférences : `tauri_prefs_get`, `tauri_prefs_all`, `tauri_prefs_set`, `tauri_prefs_set_many` ;
- user data/secrets : `tauri_user_data_get/all/set/set_many`, `tauri_secret_set/get/delete` ;
- buffers : `tauri_buffer_save/load/clear` ;
- filesystem : `tauri_fs_read_markdown`, `tauri_fs_write_markdown`, `tauri_fs_resolve_path`, `tauri_fs_detect_encoding`, `tauri_fs_trash_item` ;
- watcher : `tauri_watcher_watch_file`, `tauri_watcher_watch_directory`, `tauri_watcher_unwatch_file`, `tauri_watcher_unwatch_directory`, `tauri_watcher_unwatch_all`, `tauri_watcher_ignore_next` ;
- récents : `tauri_recents_list/add/clear` ;
- raccourcis : `tauri_keybindings_get`, `tauri_keybindings_save` ;
- features atomiques : `tauri_atomic_features_list/get/toggle/set` ;
- relations : `tauri_knowledge_graph`, `tauri_knowledge_relation_save`, `tauri_knowledge_relation_status_set`, `tauri_knowledge_relation_get`, `tauri_knowledge_relations_for_node`, `tauri_knowledge_relations_list` ;
- addons : list/install/uninstall/set_enabled, catalogue officiel, lecture module/entry, notes addon, HTTP scoped, sidecar status/call, service status/start/call/stop ;
- Android : `tauri_android_vault_pick`, `restore`, `sync`, `clear`, `tauri_android_share_text` ;
- sync Iroh : `tauri_sync_status`, `tauri_sync_create_invite`, `tauri_sync_accept_invite`, `tauri_sync_run`, `tauri_sync_shutdown`, `tauri_sync_conflict_settings_get/set`, `tauri_sync_conflict_restore/delete` ;
- acceptance/debug : `tauri_debug_log`, `tauri_acceptance_enabled`, `tauri_acceptance_result`, `tauri_acceptance_ready`.

Le commande `shell_exec` n’accepte que `pandoc`, vérifie le cwd dans le vault et ne transmet que des variables `PANDOC_*`.

### 25.6 Clés de persistance renderer

| Clé | Contenu |
|---|---|
| `elephantnote:theme` | thème courant. |
| `elephantnote:sidebarWidth` | largeur sidebar moderne. |
| `elephantnote:pinnedNotes:<vault>` | chemins épinglés du vault. |
| `elephantnote:editorTextScale` | compact/normal/large. |
| `elephantnote:search:queryLimit` | limite résultats. |
| `elephantnote:search:defaultMode` | exact/smart/semantic. |
| `elephantnote:search:visualizationMode` | space/graph/list legacy store. |
| `elephantnote:search:graphDensity` | densité graph 1–8. |
| `elephantnote:chat:conversations` | conversations JSON du ChatView. |
| `elephantnote:chat:active` | conversation active. |
| `elephantnote:canvas:<vault>` | positions du Canvas. |
| `elephantnote:lastSettingsSection` | dernière section Settings. |
| `elephantnote:ai-settings-draft` | draft local AI settings. |
| `elephantnote:debugAutosave` | active la verbosité autosave. |
| `elephantnote:addons:<addonId>:*` | storage isolé d’un addon. |
| `elephantnote:pref:*` / `elephantnote:data:*` | compatibilité préférences/données Tauri. |
| `elephantnote:diagnostics:verbose` | logs renderer verbose. |
| `elephantnote:addons:trusted-safe-mode` | safe mode du runtime trusted. |
| `elephantnote:mobile-vault-choice-v4` | choix vault Android. |
| `elephantnote:mobile-advanced-vault-v1` | état vault avancé mobile. |
| `elephantnote:mobile-advanced-vault-dirty-v1` | dirty flag mobile. |

Les préférences structurées sont aussi persistées côté Rust via `tauri_prefs_*`. Une clé localStorage ne doit pas être interprétée comme preuve que la donnée disque correspond déjà : les surfaces qui écrivent un vault doivent attendre le retour Tauri.

---

## 26. Route de préférences legacy

### 26.1 Activation de la route

Le router expose :

- `/editor` → `pages/app.vue` ;
- `/preference` → `pages/preference.vue` ;
- `/preference/general` ;
- `/preference/editor` ;
- `/preference/markdown` ;
- `/preference/spelling` ;
- `/preference/theme` ;
- `/preference/image` ;
- `/preference/keybindings` ;
- `/preference/rclone` ;
- `/muya-runtime-test` pour la surface de test runtime.

Le redirect de `/` dépend de `windowType`. Dans le démarrage Tauri actuel, `__MARKTEXT_WINDOW_TYPE__` vaut `editor`, donc le chemin nominal est `/editor`. La route Preferences reste toutefois utilisée par les fenêtres/commandes compatibles.

### 26.2 Shell de préférences

`pages/preference.vue` rend :

- une sidebar legacy ;
- une `title-bar` si custom title bar ou macOS ;
- une zone de contenu `pref-content` ;
- un `router-view` scrollable.

Géométrie :

- viewport fixed 100vw × 100vh ;
- variable `--prefSideBarWidth: 280px` ;
- contenu max-width calc(100vw - 280px) ;
- title bar 32 px ;
- setting padding 50 px 20 px, padding-top title bar ;
- si frameless : margin-top 32 px et padding-top 0 ;
- contenu height calc(100vh - 32px), overflow auto.

### 26.3 Primitives des préférences

Les vues utilisent les composants communs `Bool`, `Compound`, `FontTextBox`, `Range`, `Select`, `TextBox`, `Separator`.

- Bool : description à gauche et `el-switch`, section margin 20 px ; les liens `more` ouvrent une URL externe ; état disabled applique `.ag-underdevelop` ;
- Compound : header cliquable avec chevron, body padding 8 px 16 px ;
- Select : `el-select` height 30 px, width 100%, options `el-option` ;
- TextBox : input height 30 px, validation regex possible, erreur visuelle ;
- Range : slider width 100%, track 4 px, thumb 12 px/20 px selon Element Plus ; affiche la valeur et l’unité ;
- FontTextBox : autocomplete full width, champ 30 px, liste famille font ; `only-monospace` limite aux polices mono ;
- Separator : 100% × 2 px, margin 20 px 0 ;
- key input : overlay/dial width 500 px, input 30 px, animation fade 200 ms.

### 26.4 Preferences General

Sections :

#### Auto Save

- `autoSave` bool ;
- `autoSaveDelay` range 1000–10000 ms, step 100, unité ms.

#### Window

- title bar style, masqué sur macOS, note requires restart ;
- hide scrollbars ;
- open files in new window ;
- open folders in new window ;
- zoom via options prédéfinies.

#### Local AI runtime

- radio `bundled` : `Install and use llama-server inside the app` ;
- radio `path` : `Use an existing llama-server path` ;
- input disabled hors path, placeholder `/opt/homebrew/bin/llama-server` ;
- bouton Save disabled pendant `llamaRuntimeSaving` ;
- configuration stockée dans `window.elephantnote.ai` ou fallback `elephantnote:tauri:ai-config`.

#### Sidebar

- wrap text in TOC ;
- exclude patterns, chaîne séparée par virgules ;
- file sort by, actuellement disabled.

#### Startup

- restore previous layout state / open blank state ;
- restore all ;
- open last folder ;
- open default directory + bouton select folder ;
- open blank page.

#### Misc

- language select via `getLanguageOptions()`.

### 26.5 Preferences Editor

#### Text editor

- font-size 12–32 px ;
- line-height 1.2–2.0 ;
- font family ;
- max width validé par `^(?:$|[0-9]+(?:ch|px|%)$)`.

#### Code block

- code font-size 12–28 px ;
- code font family monospace uniquement ;
- line numbers conservé dans le store mais UI hidden pour l’instant ;
- remove empty lines.

#### Writing behavior

- auto close brackets ;
- auto-complete Markdown ;
- auto close quotes.

#### File representation

- tab size ;
- line separator ;
- default encoding ;
- auto detect encoding ;
- auto normalize line endings ;
- trailing newline policy.

#### Misc

- text direction ;
- hide quick insert hint ;
- hide link popup ;
- auto-check tasks ;
- wrap code blocks.

### 26.6 Preferences Markdown

#### Lists

- prefer loose list item ;
- bullet marker ;
- ordered list delimiter ;
- list indentation.

#### Extensions

- frontmatter type ;
- superscript/subscript ;
- footnote.

#### Compatibility

- HTML enabled ;
- GitLab compatibility enabled.

#### Diagrams

- sequence diagram theme.

Prefer heading style est affiché mais disabled.

### 26.7 Preferences Theme

Les cartes legacy font 248 × 100 px, margin 0 20 px 10 px, padding left 30 px/top 20 px, rayon 5 px et shadow 0 9px 28px -9px. Une carte active reçoit outline 2 px accent sans déplacer le layout ; une carte disabled est opacity .4 et cursor not-allowed.

Le toggle `followSystemTheme` désactive le clic direct et affiche deux selects : light mode theme et dark mode theme. Le champ custom CSS est un textarea width 100%, rows 10, border 1 px et mise à jour sur `change`.

Les thèmes historiques incluent notamment Light, Dark, Graphite, Material Dark, One Dark, Ulysses, Dracula, Nord, Catppuccin, Gruvbox, Tokyo Night, Solarized, Ayu, Everforest, Rose Pine, Monokai, Synthwave, Horizon, Palenight, Oxocarbon, Kanagawa, Nightfox et Cyberdream.

### 26.8 Preferences Image

La préférence `imageInsertAction` détermine :

- clipboard/path ;
- dossier ;
- upload.

Le choix est accompagné d’un tooltip InfoFilled 16 × 16 px. En mode folder/path, `FolderSetting` permet modifier/ouvir le dossier. En mode upload, `Uploader` expose :

- détecteur PicGo avec refresh ;
- statut success/warning et temps de détection ;
- commandes npm/yarn/pnpm d’installation ;
- lien vers PicGo Core ;
- guide `picgo set uploader`, `picgo upload /path/to/image.png`, `picgo config` ;
- configuration GitHub : token, owner, repo, branch, legal notices ;
- configuration CLI script ;
- boutons Save et états d’erreur.

### 26.9 Preferences Spellchecker

- enable spell checking ;
- hide marks for errors, disabled si spellchecker off ;
- macOS : auto-detect language visible mais disabled ;
- non-macOS : default language select, disabled si off ;
- custom dictionary : table des mots, colonne Options width 90 px, Delete icon 16 × 16 px ;
- suppression appelle `mt::spellchecker-remove-word` et retire la ligne si succès ; erreur : notification.

### 26.10 Preferences Keybindings

La table affiche Description, Key combination width 220 px et Options width 90 px.

Actions par ligne :

- Edit `Edit` 14 px : ouvre `key-input-dialog` ;
- Reset `RefreshRight` 14 px ;
- Unbind `Delete` 14 px.

Footer : Save et Restore defaults. Le debug optionnel affiche `Dump keyboard information` et envoie `mt::keybinding-debug-dump-keyboard-info`.

Le chargement récupère layout/keymap via `mt::keybinding-get-keyboard-info`, puis les bindings via `mt::keybinding-get-pref-keybindings`. Une collision affiche une notification warning `shortcutInUse`.

### 26.11 Preferences Sync/rclone

La vue Rclone est une interface hybride Vue render function :

1. Local network sync : password minimum 8 caractères, vault scope `Only current vault`/`All vaults`, bouton Create pairing invite disabled si password trop court ;
2. Connect another device : JSON invitation dans `<pre>`, Copy invite devient Copied ;
3. Fallback shared location : input `remote:ElephantNote or /Volumes/NAS/ElephantNote`, Refresh status, Save location, Sync now ;
4. Status : statut et détails.

La vue appelle `sync.status` et `sync.run` via l’API unifiée et construit une invitation `elephant-lan-pairing-invite`, version 1, encrypted true, transport `local-network-rclone`.

---

## 27. Spécification de recodage indépendante du code

Cette section donne les contrats minimaux à reproduire dans une nouvelle implémentation. Elle transforme les éléments dispersés du dépôt en modèles et en flux de données explicites.

### 27.1 Modèle `VaultDescriptor`

```ts
type VaultDescriptor = {
  id: string              // slug stable, ex. "work" ou "work-2"
  name: string            // nom affiché
  path: string            // chemin absolu normalisé
  icon: string            // nom d’icône ou chaîne vide
  lastOpenedAt: string    // timestamp ISO ou chaîne backend
  enabled: boolean        // true par défaut
}

type VaultConfig = {
  schemaVersion: 1
  vaults: VaultDescriptor[]
  activeVaultId: string | null
}
```

Règles :

- slugifier le nom en minuscules ASCII et tirets ;
- si l’ID existe, suffixer `-2`, `-3`, etc. ;
- canonicaliser le chemin au moment de l’upsert ;
- conserver les vaults manquants dans le registre pour permettre une récupération ;
- un vault disabled ne peut pas être le vault actif ;
- désactiver le vault actif sélectionne le prochain vault enabled ;
- retirer un vault retire le registre mais ne supprime pas le dossier utilisateur.

### 27.2 Modèle d’entrée de bibliothèque

```ts
type VaultEntry = {
  name: string
  title: string
  path: string          // relatif au vault, slash normalisés
  fullPath: string      // absolu, retourné seulement au runtime de confiance
  type: 'folder' | 'note' | 'drawing' | 'file'
  isDirectory: boolean
  noteCount: number
  childrenPreview: Array<{ title: string; type: string }>
  preview: string
  excerpt: string
  updatedAt: string
  tags?: string[]
}
```

Le backend trie d’abord les dossiers avant les autres entrées, puis alphabétiquement en minuscules. Il ignore `.elephantnote`, `.git`, `node_modules`, tout nom commençant par `.`, `~` ou `.tmp`, et ignore les symlinks.

Pour une note Markdown, le preview lit au plus 16 KiB et extrait au plus trois lignes d’excerpt. Pour un dossier, `noteCount` compte seulement les fichiers `.md` directs et `childrenPreview` est limité à trois éléments.

### 27.3 Document Markdown canonique

Le document recommandé est :

```md
---
title: "Project note"
type: "note"
tags: ["work", "planning"]
createdAt: "2026-08-17T10:00:00.000Z"
updatedAt: "2026-08-17T10:05:00.000Z"
---

# Project note

Body Markdown visible dans l’éditeur.
```

Règles de transformation :

1. frontmatter délimité par une ligne `---` au début et à la fin ;
2. `title` prioritaire ; sinon premier H1 ; sinon nom du fichier ;
3. le titre et le frontmatter sont masqués dans la zone d’édition ;
4. le titre visible est reconstitué au save ;
5. tags inline et tags YAML en bloc sont acceptés, puis normalisés en tableau inline ;
6. les tags sont trim, débarrassés du préfixe `#`, espaces multiples réduits et dédoublonnés ;
7. `createdAt` est conservé lors d’une édition ; `updatedAt` est mis à jour à l’écriture ;
8. un document sans frontmatter reçoit `type: note`, `tags: []`, createdAt et updatedAt au premier enrichissement ;
9. un placeholder `# Untitled` vide peut être ignoré au save pour ne pas créer un fichier artificiel ;
10. les images locales doivent être relocalisées sous `.assets` avant le write.

### 27.4 Modèle workspace/sidebar

Format canonique simplifié :

```ts
type Workspace = {
  version: 1
  vaultName: string
  sidebar: Array<{
    id?: string
    title: string
    type: 'note' | 'folder'
    path: string
    collapsed?: boolean
  }>
}
```

Le premier workspace peut contenir `Getting Started` et `Welcome`. La normalisation aplatit aussi les anciens items imbriqués en entrées de sidebar simples. L’UI garde la récursivité visuelle par chargement lazy des enfants.

### 27.5 Enveloppe API unifiée

Version courante lue dans `shared/apiContracts.js` : `2026-07-14-core`.

Une action doit être exposée par :

```ts
type ApiActionResult<T> = {
  ok: boolean
  version: string
  action: string
  data?: T
  error?: { code?: string; message: string }
}
```

Avant appel, valider le payload. Les règles communes :

- payload objet, jamais tableau ;
- required string non vide ;
- required boolean strict ;
- numbers finis ;
- enums exactes ;
- filenames sans slash, NUL, `.`/`..`, parent path ou traversal.

Actions core actuelles :

```text
api.describe
vaults.get/select/setActive/setIcon/setName/remove/setEnabled
vaults.trash.list/restore/empty
directory.list
notes.create/read/write
folders.create
sidebar.attach/detach
entries.rename/move/delete
search.query/status
features.get/set
atomic.catalog.get
```

Pour recoder le frontend, chaque namespace doit présenter une fonction pure qui normalise le payload, appelle le bridge, transforme l’enveloppe et laisse remonter l’erreur au store. Le composant ne doit pas appeler directement le disque.

### 27.6 Flux recodable : ouverture d’une note

```text
clic carte/sidebar/search result
  → normalize relative path
  → si dossier : directory.list
  → si note déjà ouverte : réutiliser l’identité existante
  → sinon invoke notes.read / événement open-file
  → créer ou sélectionner un tab par pathname normalisé
  → monter le moteur Rust/Muya avec markdown + cursor
  → publier l’état editor ready
  → afficher NoteEditorHost
```

Le chemin est toujours relatif pour l’API vault. L’absolu n’est utilisé qu’au bord Tauri ou pour l’opener externe.

### 27.7 Flux recodable : édition et save

```text
DOM beforeinput / click / paste / drop
  → lire sélection UTF-16
  → créer commande Rust protocol v1
  → vérifier expected_revision
  → appliquer snapshot/update/patch
  → rendre DOM et mettre à jour selection
  → convertir editor Markdown → document Markdown
  → déplacer les assets locaux vers .assets
  → programmer autosave 160 ms
  → write notes.write ou marktext_write_file
  → rafraîchir entry + emit tab-saved
```

Si un write est en vol, conserver exactement une sauvegarde pending avec le dernier Markdown. Ne jamais envoyer un `saved` avant le retour réel du write.

### 27.8 Flux recodable : drag/drop

```text
dragstart
  → payload kind/path/title/preview
dragover
  → vérifier cible folder ou note
  → dropEffect move pour explorer, copy pour éditeur
drop explorer
  → refuser self, descendant, parent actuel
  → entries.move
  → remapper pins/openedNote/currentPath
  → reload current/root directory
drop editor
  → placer caret à x/y
  → image externe : copie unique sous .assets
  → note/folder : produire lien Markdown
  → commande paste_markdown
```

### 27.9 Flux recodable : addon

```text
catalogue/manifest
  → installation package
  → registre Rust
  → validation dépendances/permissions
  → worker trusted ou isolated
  → contributions views/settings/sidebar/topbar/zones
  → activation + disposer
  → UI visible
  → calls scoped notes/storage/network/native
disable/uninstall
  → stop stream/service/sidecar
  → disposer contributions
  → retirer routes/zones/commands
  → conserver les notes utilisateur sauf action explicite
```

Un addon qui n’a pas de permission `notes.write` ne doit jamais pouvoir écrire une note. Un sidecar doit exposer un status explicite, un démarrage réel, une erreur visible et une fermeture réelle.

### 27.10 Flux recodable : recherche

```text
open SearchModal
  → focus/select input
  → query change
  → debounce 220 ms
  → search.query(mode, limit)
  → status poll 1500 ms
  → render concepts + notes + snippets
  → ArrowUp/Down sélection
  → Enter/open result
```

Le mode Exact peut être prouvé par scan réel des Markdown. Le mode Semantic ne peut être annoncé que si le Knowledge index et le provider embedding ont réellement répondu.

### 27.11 Flux recodable : cycle de vie Tauri

```text
start without vault
  → choose/create vault
  → register + initialize hidden layout
  → load root entries
  → install/enable addon if requested
  → open UI
  → perform primary action and observe disk/service effect
  → visible/logged error path
  → restart and reload persisted state
  → disable addon and verify contributions disappear
  → uninstall and verify no active runtime residue
```

Ce cycle est la définition fonctionnelle du produit, au-delà de la simple présence de composants.

---

## 28. Surfaces contribuées et panels secondaires encore à recoder

### 28.1 Site preview moderne

Sources : `app/sitePreview/SitePreviewPanel.vue`, `SitePreviewToolbar.vue`, `sitePreviewStore.js`.

Le panel affiche trois états : aucun preview, preview en cours et preview arrêté/erreur.

Toolbar :

- ligne flex, min-width 0 ;
- titre/chemin ellipsés ;
- bouton `Open externally` lorsque l’URL existe ;
- bouton close avec aria-label `Close website preview` ;
- boutons hauteur 34 px, padding horizontal 12 px ;
- bouton icon-only width 34 px ;
- icône 18 × 18 px ;
- mobile ≤760 px : les éléments passent en largeur disponible.

Cycle :

1. l’utilisateur fournit un dossier relatif ;
2. `previewFolder` ou `buildFolder` appelle le service Sites ;
3. la preview rend un iframe/URL scoped ;
4. Open externally appelle l’opener Tauri ;
5. Stop arrête le serveur statique ;
6. le store conserve `siteId`, `previewUrl`, `status`, `error` et la liste des fichiers.

Les artefacts sont sous `.elephantnote/site-previews/**` ou `.elephantnote/site-builds/**`. La suppression du panel doit arrêter le serveur et nettoyer l’écoute associée.

### 28.2 Contrôle Sync dans la navigation

`SyncNavigationControl.vue` contribue un bouton de rail/topbar de 24 × 24 px. Il utilise :

- titre dynamique basé sur le statut ;
- `Sync`/`Syncing…`/`Sync error` ;
- spinner `RefreshCw` de 18 × 18 px, animation **0.8 s linear infinite** ;
- dot d’état 6 × 6 px ;
- transition background/color/opacity **120 ms**.

Un clic stoppe la propagation, lance la synchronisation du workspace courant et publie les états `running`, `done` ou `error`. Les erreurs restent dans le titre/aria-label et dans les logs.

### 28.3 Addon packs

`AddonPacksSettings.vue` rend un catalogue de packs dans le slot `addons.packs`.

- header avec recherche/refresh/import pilotés par événements `elephantnote:addon-packs-*` ;
- ligne pack min-height 78 px, grille 36 px / contenu / actions, gap 12 px, padding 13 × 15 px ;
- icône 36 × 36 px, rayon 9 px, SVG 17 px ;
- description en line-height 1.4 ;
- Delete n’est pas immédiat : premier clic place le chemin dans `confirmDeletePath`, puis affiche `Confirm` et `Cancel` ;
- packs protégés ne peuvent pas être supprimés ;
- état vide padding 28 × 16 px, font 12 px ;
- mobile ≤760 px : layout en colonne.

### 28.4 AI Provider Settings moderne

`AiProviderSettingsPanel.vue` normalise les routes chat, embedding et OCR dans des cards Settings.

#### Route Chat

- source provider select ;
- modèle provider select si la liste existe, sinon input `Provider model id` ;
- system prompt textarea rows 5 ;
- switch Enable RAG ;
- Temperature number 0–2, step .05 ;
- Max tokens min 1, step 128 ;
- Context window min 512, step 512 ;
- RAG notes limit 1–50.

#### Route Embedding

- source + model ;
- bouton `Rebuild index` disabled/pending `Rebuilding…` ;
- Search result limit 1–100 ;
- Semantic threshold 0–1 ;
- Chunk strategy : Markdown headings, Paragraphs, Fixed size, Hybrid ;
- Distance : Cosine, Dot product, Euclidean ;
- Chunk size min 64 step 64 ;
- Chunk overlap min 0 step 16 ;
- Dimensions min 0 ;
- Debounce min 0 step 250.

#### Route OCR

- source + model ;
- Languages placeholder `eng,fra` ;
- output Markdown/Plain text/Layout Markdown ;
- confidence threshold 0–1 ;
- PDF mode Only pages without text / All pages / Skip text PDFs.

Cards :

- header padding 15 × 16 px, border bottom ;
- body padding 16 px ;
- rows min-height 66 px, padding 13 × 16 px ;
- badges padding 4 × 8 px, pills, font 11 px ;
- inputs/selects/textarea padding 8 × 9 px, border 1 px, rayon 8 px ;
- boutons min-height 34 px, padding 0 11 px, rayon 9 px ;
- mobile ≤760 px : grille une colonne.

Les modifications sont auto-sauvegardées et émettent `elephantnote:ai-config-changed`. Le provider selectionné doit venir de la liste runtime addon, pas d’une liste fictive.

### 28.5 ChatGPT usage card moderne

`ChatgptSubscriptionCard.vue` complète Codex Connection lorsque la contribution expose l’usage.

- header min-height 72 px, padding 12 × 16 px ;
- logo OpenAI 30 × 30 px ;
- compte ellipsé max 320 px, mobile 220 px ;
- expand button 34 × 34 px ; chevron transition rotation **160 ms** ;
- actions Connect/Disconnect, Open auth, Toggle usage ;
- login box margin 0 16px 14px, padding 9 × 10 px, rayon 10 px ;
- usage panel margin 0 16px 16px, padding 14 px, rayon 12 px ;
- limite row min-height 62 px, grille 150/.8fr + 180/1.4fr + auto ;
- progress width 100%, height 7 px ; remaining min-width 102 px ;
- reset option min-height 48 px ;
- boutons standard min-height 34 px, compact 30 px.

Interactions : connecter/déconnecter, ouvrir une URL d’authentification, développer l’usage, choisir un reset crédit. L’état Connected doit provenir du service Codex.

### 28.6 Task manager declarative view

`AddonWorkspaceHost.vue` prend en charge `kind === 'task-manager-v1'`.

#### Navigation gauche

- brand mark 34 × 34 px, rayon 11 px, accent ;
- nav padding 4 × 8 × 16 ;
- item min-height 34 px, grid icône 20 / contenu / count, gap 8 px, padding 0 9 px ;
- Areas/Projects heading min-height 28 px, font 10 px uppercase ;
- bouton add 22 × 22 px ;
- area dot 8 × 8 px ;
- project progress 12 × 12 px, border rond et conic-gradient ;
- composer input/select height 30 px, bouton 28 px.

#### Main

- header padding 25 × 30 × 14 ; actions 30 × 30 ;
- quick task min-height 45 px, margin horizontal 30 px, grille plus/input/schedule, rayon 12 px ;
- add task rond 25 × 25 px ;
- toolbar padding 0 30 px 12 px ; search height 31 px ; select height 33 px ;
- scroll padding 0 30 px 40 px ;
- section title height 31 px ;
- task row min-height 54 px, grille checkbox 24 / content / today 28, padding 7 × 5 ;
- checkbox 21 × 21 px rond ;
- message empty/error min-height 180 px ;

#### Detail

- desktop colonne droite avec border-left ;
- ≤1050 px : panneau absolute right, width `min(360px, 90vw)`, shadow -18 × 0 × 40 ;
- header height 50 px ;
- scroll height calc(100% - 50px), padding 17 px, gap 14 px ;
- textareas/inputs/selects padding 8 px, font 12 px, rayon 8 px ;
- repeat editor border/rayon 9 px ;
- Save changes, Cancel task, Delete ; Delete est armé en deux temps : `Delete` puis `Confirm delete`.

Interactions : sélection de liste, création area/project, quick task, toggle Today/Evening, recherche debounce 180 ms, filtre tags, refresh/retry, sélection task, complete/reopen, édition détail, repeat daily/weekly/monthly/yearly fixed/after-completion, save/cancel/delete.

### 28.7 Excalidraw core overlay builtin

`addons/builtin/ui/ExcalidrawEditorOverlay.vue` ajoute les contributions qui permettent à l’éditeur et aux cards d’ouvrir le dessin. La configuration du thème est injectée depuis `elephantnoteTheme`. Le lifecycle dispatch :

- `elephantnote:excalidraw-core-enabled` au montage ;
- `elephantnote:excalidraw-core-disabled` au cleanup.

Le composant doit être démonté avant de détruire la racine React Excalidraw, sinon l’overlay et ses listeners clavier peuvent rester actifs.

### 28.8 LanguageSettingsRow

Le sélecteur moderne de langue est une ligne Settings :

- copie label/description à gauche ;
- select compact à droite ;
- option système affichée sous forme `System language · <displayName>` ;
- autres options affichées `nativeName · displayName` ;
- `setLanguage()` écrit la préférence et émet `elephantnote:language-changed` ;
- les composants montés réécoutent l’événement et reconstruisent leurs options.

### 28.9 IconRailLayoutSettings

Le panneau de configuration du rail :

- header min-height 34 px ;
- boutons Add divider, Reset icon bar et Collapse/Expand, chacun 28 × 28 px, icône 14 px ;
- liste border/rayon 10 px, overflow hidden ;
- item normal min-height 48 px, grille grip 22 / icon 24 / copy / actions, gap 10 px, padding 6 × 9 px ;
- item divider min-height 42 px ;
- grip 16 × 16 px, cursor grab ;
- preview icon 24 × 24 px, rayon 6 px ; divider preview 20 × 1 px ;
- actions 28 × 28 px ; move up/down disabled aux limites ;
- item dragging opacity .48 ; item hidden copie et icône opacity .55 ;
- chevron collapse rotate -90°.

Interactions :

- Add divider crée un ID separator ;
- Reset restaure `DEFAULT_ICON_RAIL_ORDER` et enlève tous les hidden IDs ;
- collapse ne supprime pas le layout, il masque seulement la liste ;
- boutons up/down déplacent un item ;
- drag/drop déplace un item ;
- Eye/EyeOff persiste la visibilité ;
- Trash2 supprime uniquement un divider.

### 28.10 AddonSettingsRow

Une ligne addon moderne possède :

- summary en grille 34 px / contenu / chevron 16 px, gap 11 px, padding 13 × 14 px ;
- logo 34 × 34 px, rayon 9 px, SVG 17 px ;
- nom 12.5 px, version 9.5 px, description 10.5 px ;
- badge access `Limited access` isolated vert ou `Full app access` trusted orange ;
- chevron 15 × 15 px, rotation 180° en 140 ms ;
- controls switch + uninstall ;
- détails padding left 59 px, permissions en pills font 9.5 px ;
- explication access border/rayon 10 px, icône 17 px ;
- commandes action avec `Play` 13 px ;
- désinstallation en deux étapes Confirm uninstall/Cancel ;
- mobile ≤720 px : détails sans indentation, controls plus compacts.

Le statut `activating`, l’accès locked, les permissions et les erreurs du manifest doivent être visibles. Une désinstallation d’addon externe conserve les données privées selon le message spécifique ; un addon officiel retiré disparaît jusqu’à réinstallation.

---

## 29. Audit de couverture documentaire

| Zone du dépôt | Couverture dans ce document | Niveau |
|---|---|---|
| `frontend/app/components/shell/**` | shell, topbar, mobile, drawers, routes visuelles | détaillé, `CODE` |
| `frontend/app/components/navigation/**` | rail, vault, sidebar, sync control, drag/order | détaillé, `CODE` |
| `frontend/app/components/library/**` | cartes, tri, pagination, DnD, create | détaillé, `CODE` |
| `frontend/app/components/editor/**` | topbar, tags, footer, toolbar, Excalidraw | détaillé, `CODE` |
| `frontend/src/renderer/src/editor-rust/**` | protocole, sélection, input, clipboard, composition, drop | détaillé, `CODE` |
| `frontend/src/renderer/src/components/editorWithTabs/**` | tabs, runtime, dialogs image/table, export | détaillé, legacy `CODE` |
| `frontend/src/renderer/src/muya/lib/ui/**` | dimensions, menus, actions, transitions | détaillé, legacy `CODE` |
| `frontend/src/renderer/src/components/sideBar/**` | arbre, recherche, résultats, context menus | détaillé, legacy `CODE` |
| `frontend/src/renderer/src/prefComponents/**` | route, primitives, sections et contrôles | détaillé, legacy `CODE` |
| `frontend/app/search/**` | modal, états, résultats, réglages | détaillé, `CODE` |
| `frontend/app/components/views/**` | Chat, Canvas, Graph, Calendar, addon host/router | détaillé, `CODE`/`DYNAMIQUE` |
| `frontend/app/components/settings/**` | Settings, AI, addons, sync, language, rail | détaillé, `CODE` |
| `addons/official/**` | manifests, contributions, UI physique et services | détaillé, `DYNAMIQUE` |
| `backend/tauri/src/vault/**` | config, entries, sécurité, trash, workspace | détaillé, `CODE` |
| `backend/tauri/src/markdown/**` | commandes Markdown/Muya et protocole frontend | détaillé, `CODE` |
| `backend/tauri/src/state.rs/preferences.rs` | prefs, data, secrets, buffers, recents, keybindings, atomics | détaillé, `CODE` |
| `backend/tauri/src/addon_*.rs` | catalogue, permissions, notes, HTTP, sidecars, services | résumé contractuel, `CODE` |
| `backend/tauri/src/watcher.rs` | watcher et ignore-next | détaillé, `CODE` |
| tests/acceptance/runtime | parcours et preuves à exécuter | explicitement `RUNTIME À EXÉCUTER` |

Les seuls éléments volontairement non transformés en inventaire ligne-par-ligne sont les bibliothèques tierces, les fichiers générés, les snapshots, les implémentations purement mathématiques de rendu et les tests qui répètent un contrat déjà décrit. Ils sont cités lorsque leur comportement est visible dans le produit.

---

## 30. Limites de preuve finale

La présente documentation est la spécification statique la plus complète produite depuis le code inspecté. Elle permet de reconstruire l’architecture et les contrats, mais les affirmations suivantes restent à exécuter pour devenir des preuves runtime :

- chaque mesure en pixels sur les différents moteurs WebView et DPI ;
- chaque animation au rythme réel ;
- la visibilité finale des addons selon catalogue/permissions ;
- le rendu des scènes Mermaid/Vega/PlantUML/Sequence ;
- le vrai parcours Tauri desktop empaqueté ;
- le vrai parcours Android avec DocumentProvider/safe areas ;
- les sidecars AI/OCR/Codex/Open Models/Knowledge/Code Execution ;
- une paire Sync réelle et les conflits ;
- les sauvegardes/reloads après redémarrage.

Ces limites ne sont pas des omissions silencieuses : elles sont des critères d’acceptation explicites pour une prochaine passe de validation.

## 31. Cible native Freya présente sur la branche active

Cette section est indispensable pour recoder l’état actuel du dépôt. Elle ne
décrit pas une hypothétique future version : elle décrit `Elephant/freya/**`,
une application Rust/Freya native qui coexiste avec le produit Tauri/Vue.
Freya n’est pas une fenêtre Tauri WebView. Les deux cibles partagent les
contrats de vault, Markdown, Muya, addons et thèmes, mais leurs arbres de
rendu ne sont pas interchangeables.

### 31.1 Binaire, dépendances et démarrage

Sources : `Elephant/freya/Cargo.toml`, `Elephant/freya/src/main.rs`,
`Elephant/freya/src/lib.rs`.

Le crate est `elephant-freya`, Rust 2021, minimum Rust `1.91`, Freya `0.4.1`
avec la feature `sdk`, `muya-core` local et `elephantnote-knowledge-core`
local. Les dépendances de frontière sont `rusqlite` bundled, `reqwest` avec
Rustls, `serde`, `serde_json`, `blake3` et `zip`. Les tests utilisent
`freya-testing` et `freya-clipboard`.

Commandes de développement déclarées dans le workspace :

| Commande | Effet prévu par le dépôt |
|---|---|
| `pnpm freya:dev` | lance `cargo run --manifest-path Elephant/freya/Cargo.toml` ; |
| `pnpm freya:check` | lance `cargo check` sur le crate Freya ; |
| `pnpm freya:test` | lance les tests Rust du crate et les tests d’intégration sous `Elephant/freya/tests` ; |
| `pnpm start` / `pnpm dev` | alias vers `freya:dev` dans le `package.json` racine. |

Le launcher effectue les opérations suivantes :

1. installe un panic hook qui écrit `[freya][fatal]`, capture la backtrace et
   termine le processus avec le code `1` au lieu de laisser un dialogue natif
   bloquant survivre au terminal ;
2. écrit `[freya][runtime] action=launch runtime=freya` ;
3. crée une fenêtre `ElephantNote · Freya` de `1280 × 840 px` ;
4. impose une taille minimale de `720 × 480 px` ;
5. charge `Elephant/assets/static/icon.png` comme icône ;
6. active la transparence de fenêtre ;
7. sous macOS, rend la titlebar transparente, masque le titre natif et active
   le full-size content view ;
8. monte `app::app` comme composant de contenu.

Les dimensions et l’icône du launcher sont du code de la branche active. Elles
ne sont pas une preuve qu’une fenêtre Freya a été ouverte dans cette passe.

Les trois points d’entrée du composant sont :

```rust
app() -> impl IntoElement
app_with_vault(root) -> impl IntoElement
app_with_vault_view(root, view) -> impl IntoElement
```

`app()` charge l’environnement normal et le registre persistant. Les deux
fonctions `app_with_vault*` servent au host/test : elles montent un vault
fourni par l’appelant et utilisent `load_root_without_persisted_registry`,
donc elles ne doivent pas écrire le registre utilisateur global. La nouvelle
fonction `app_with_vault_view` sélectionne une vue de workspace avant le
premier rendu.

### 31.2 Arbre de rendu Freya

```text
Freya Window 1280×840 (min 720×480)
└─ app::app
   ├─ si aucun vault : EmptyVaultPicker
   └─ shell
      ├─ desktop : TopVaultBar 28px
      │  └─ ligne horizontale : IconRail 56px + Sidebar 184..320px + Main
      ├─ mobile (<760px) : MobileTopBar 52px + Main plein écran
      ├─ Search overlay éventuellement monté au-dessus du workspace
      ├─ drawer navigation mobile éventuellement monté à top 52px
      ├─ CreateEntryMenu si menu_open
      └─ VaultWatcherHost
```

Le `Main` choisit la surface à partir de l’état `ShellState` :

| État | Surface montée |
|---|---|
| `vault == None` | `EmptyVaultPicker` ; le shell normal n’est pas rendu ; |
| `drawing != None` | éditeur de dessin natif Freya ; |
| `editor != None` | `NoteEditorHost` avec document Muya réel ; |
| `view == Notes` et aucun éditeur | toolbar + bibliothèque paginée + FAB Create ; |
| `view == Wiki` | liste de brouillons Wiki stockés ; |
| `view == Calendar` | événements ICS + notes groupés par date ; |
| `view == Chat` | conversation provider-backed ; |
| `view == Models` | fichiers GGUF locaux ; |
| `view == Sync` | service Sync package-owned ; |
| `view == Canvas` | projection graphique + positions persistées ; |
| `view == Graph` | explorer Graph ; |
| autre `WorkspaceView::Addon(id)` | avis de route avec l’identifiant, sauf contribution native explicitement mappée. |

`Search` et `Settings` sont des états de shell, pas des remplacements du
workspace : Search est un overlay et Settings devient le contenu principal.
La fermeture de Search conserve donc la bibliothèque, le Graph ou le Canvas
qui se trouvait dessous.

### 31.3 État de shell à reproduire

`ShellState` contient les champs suivants. Un recode doit conserver cette
séparation : les flags d’affichage ne doivent pas être confondus avec les
documents ou les services.

| Champ | Rôle |
|---|---|
| `view` | `WorkspaceView` actif ; |
| `sidebar_visible` | colonne sidebar visible ou largeur zéro ; |
| `mobile_navigation_open` | drawer mobile ouvert/fermé ; |
| `sidebar_width` | largeur validée `184..320`, défaut `232` ; |
| `vault` | `VaultAdapter` du vault actif, ou `None` ; |
| `vault_registry` | descripteurs, actif, enabled et persistance ; |
| `page` | dernière `VaultPage` chargée pour la bibliothèque ; |
| `library` | entries, tri, mode grille/liste, pagination, pins, chemin ; |
| `menu_open` | overlay CreateEntryMenu ; |
| `hovered_target` | identifiant de hover de shell ; |
| `card_action_target` | carte dont le menu d’actions est ouvert ; |
| `vault_menu_open` | popup Vaults ; |
| `search_open` | état fonctionnel Search ; |
| `settings_open` | état fonctionnel Settings ; |
| `settings_target_section` | section à sélectionner au prochain rendu ; |
| `editor` | `EditorDocument` ouvert ; |
| `editor_tag_draft` | draft temporaire de formulaire de tags ; |
| `error` | dernière erreur visible du shell ; |
| `navigation_history` / `navigation_index` | historique borné de dossiers et notes ; |
| `rail_order` | ordre persisté du rail ; |
| `settings_revision` | invalidation de Settings après une mutation ; |
| `rail_drag` / `rail_drop_target` | état du DnD du rail ; |
| `sidebar_resize` | capture du début d’un resize ; |
| `library_drag` | état DnD d’une carte ; |
| `drawing` / `drawing_path` | scène de dessin en cours ; |
| `canvas` | `CanvasRuntime` du Canvas actif ; |
| `calendar`, `chat`, `models`, `sync` | sous-états des services secondaires ; |
| `code_execution` | service et résultat du dernier bloc exécuté. |

Les transitions qui changent de surface nettoient les états incompatibles :

- ouvrir un workspace efface l’éditeur, le dessin, l’erreur de code et Search/
  Settings ;
- passer à une vue différente de Canvas détruit `canvas` ;
- ouvrir une note efface drawing et code execution, force `view = Notes` et
  ferme Search/Settings ;
- ouvrir un dossier efface l’éditeur et le draft de tags, recharge le dossier
  et peut enregistrer l’entrée dans l’historique ;
- fermer l’éditeur ne supprime pas le fichier : `EditorDocument::close()`
  sauvegarde d’abord si nécessaire ;
- changer de vault remplace tout l’état vault-scoped, y compris le document
  ouvert et la page de bibliothèque.

### 31.4 Palette et géométrie natives

Le module `theme.rs` expose une palette typée, des mélanges de couleurs et
des dimensions. Les couleurs ci-dessous sont les valeurs RGBA de la palette
de démarrage Elephant ; les autres familles réutilisent la même structure de
tokens.

| Token | Light | Dark |
|---|---|---|
| `bg` | `rgb(247,249,252)` | `rgb(15,20,29)` |
| `surface` | `rgb(255,255,255)` | `rgb(20,26,36)` |
| `sidebar` | `rgb(237,242,247)` | `rgb(16,23,34)` |
| `soft` | `rgb(233,239,247)` | `rgb(27,36,50)` |
| `border` | `rgb(197,207,221)` | `rgb(40,50,68)` |
| `borderStrong` | `rgb(174,186,205)` | `rgb(58,70,90)` |
| `text` | `rgb(16,24,40)` | `rgb(238,243,251)` |
| `muted` | `rgb(71,84,103)` | `rgb(152,163,182)` |
| `primary` | `rgb(37,99,235)` | `rgb(94,161,255)` |
| `danger` | `rgb(220,38,38)` | `rgb(255,107,122)` |

La police shell est `Helvetica Neue`. Les constantes de géométrie sont :

| Surface | Mesure exacte |
|---|---:|
| topbar | hauteur `28px` ; bouton de navigation `24px`, top `4px` ; |
| rail | largeur `56px`, actions `34 × 34px`, gap `2px` ; |
| rail desktop/macOS | padding-top `8px` / `36px` ; bottom `6px` puis `8px` ; |
| sidebar | largeur par défaut `232px`, min `184px`, max `320px` ; |
| resizer | zone `12px`, poignée centrale `3 × 64px` ; |
| All notes | hauteur `38px` ; |
| tags header | hauteur `36px` ; |
| arbre | ligne `36px`, indentation par profondeur `14px`, toggle `22px` ; |
| card grid | hauteur `176px`, largeur minimale `240px`, gap `10px` ; |
| card list | hauteur `58px` ; |
| library toolbar | hauteur `72px`, boutons sort/view `52 × 52px`, gap `14px` ; |
| Create FAB | `56 × 56px`, position right/bottom `20px`, rayon `11px` ; |
| Create popover | largeur `280px`, bottom `86px`, rayon `14px`, padding `8px` ; |
| Create option | hauteur `52px`, padding `10px`, gap `12px`, rayon `10px` ; |
| vault switcher | largeur minimale `250px`, item `34px`, position left `52px`, bottom `42px` ; |
| erreurs bibliothèque | left/right `12px`, top `80px`, padding `10px`, rayon `8px`, overlay level `20` ; |
| route notice | padding `18px`, gap `10px`, fond surface ; |
| mobile topbar | hauteur `52px`, padding vertical `8px`, horizontal `12px`, boutons `36px` ; |
| mobile drawer | largeur `300px`, top `52px`, hauteur restante, overlay level `40` ; |
| note card metadata | date `90px`, base tag `18px`, `7px` par caractère, right `38px`, chrome `30px` ; |
| carte/preview | card background = surface mélangée à `34%` sur bg ; preview = surface mélangée à `55%` sur card background. |

La version native n’invente pas de transition CSS pour chaque hover. Les
changements de couleur, d’opacité et de sélection sont immédiats dans le code
Freya sauf pour l’overlay Search, décrit en section 33.1. Une reproduction qui
ajoute des durées arbitraires ne serait pas fidèle.

### 31.5 Topbar desktop, topbar mobile et rail

#### Topbar desktop

`top_vault_bar` est une barre Freya de `28px`. Elle rend la navigation
retour/avance et le badge `Freya` ; elle ne rend pas les contrôles de fenêtre
décrits par `TopVaultBar.vue`.

- zone de navigation à `left 56px` (`84px` sous macOS), `top 4px`, largeur
  native `92px` et boutons `24 × 24px` ; la largeur `76px` appartient au
  contrat Tauri/Vue ;
- icônes de navigation `18px` et gap `2px` ;
- badge `Freya` à `top 5px`, hauteur `20px`, largeur minimale `34px` ;
- `main.rs` configure la titlebar macOS, mais cette topbar native ne fournit
  ni boutons de fenêtre, ni double-clic de maximisation, ni resynchronisation
  de l’état maximisé.

#### Topbar mobile

Introduite dans les changements non committés de la branche, elle est rendue
quand `root_size.width < 760` :

- bouton `Open navigation` ou `Close navigation`, icône `PanelLeft`, `20px` ;
- titre `Elephant`, bold, `16px` ;
- séparateur flexible au centre ;
- bouton `Search`, icône `Search`, `20px` ;
- bouton `Settings`, icône `Settings`, `20px` ;
- chaque hit box mesure `36 × 36px` ;
- cliquer le premier bouton inverse `mobile_navigation_open` ;
- Search et Settings utilisent les mêmes actions de shell que le rail ;
- le drawer est fermé par une pression globale dans son conteneur.

#### Rail

Le rail porte le label accessible `Workspace navigation`. Les actions centrales
et leurs icônes sont :

| ID | Label visible | Icône normale | Action |
|---|---|---|---|
| `vault` | `Vault` | `Vault` | ouvre le switcher de vaults ; |
| `sidebar-toggle` | `Sidebar` | `PanelLeft` au repos, `PanelLeftClose`/`PanelLeftOpen` au hover | masque ou montre la sidebar ; |
| `search` | `Search` | `Search` | ouvre/ferme Search ; |
| `settings` | `Settings` | `Settings` | ouvre/ferme Settings ; |
| vue native addon | titre de la vue | `BookOpen`, `Calendar`, `MessageCircle`, `LayoutDashboard`, `GitFork`, `Database` ou `RefreshCw` | ouvre le workspace correspondant. |

Le renderer Freya persiste actuellement uniquement l’ordre des actions core
`sidebar-toggle` et `search` dans `freyaShell.railOrder`. Les vues natives
d’addons activées sont ajoutées après cet ordre et ne sont pas incluses dans sa
persistance. Les IDs `separator:<id>` appartiennent au contrat portable, mais
le renderer Freya les filtre actuellement au lieu de les rendre.
Démarrer un drag mémorise la position ; le mouvement doit dépasser `4px` sur x
ou y ; déposer sur une autre entrée core déplace l’ID, nettoie la cible et
persiste `freyaShell.railOrder`.

Le bouton Vault affiche un tooltip de la forme `<nom> - open vault switcher`
ou `No vault - open vault switcher`. Son menu utilise la géométrie `250px`
et chaque ligne `34px` :

- titre `VAULTS` en `11px` bold muted ;
- vault actif avec fond `soft` ;
- icône Vault `15px` ;
- nom en `13px` ;
- `Add another vault` avec `Plus 16px` ;
- `Manage vaults` avec `Settings 16px` ;
- vault sans registre : une ligne de gestion qui ouvre Settings/Vaults.

Le clic d’une ligne de vault traverse la zone mesurée de la ligne pour que le
texte et l’icône déclenchent la même activation. L’activation recharge le
vault depuis le chemin persistant, puis remplace l’état courant.

### 31.6 Sidebar native et arbre

La sidebar a un fond `sidebar`, une bordure droite de `1px` et un ScrollView
avec scrollbar et flèches activées, mais avec le drag scrolling désactivé. La
zone supérieure est :

1. `All notes`, hauteur `38px`, icône `Inbox 18px`, padding horizontal `12px` ;
2. espace `8px` ;
3. header `NOTES`, hauteur `36px`, `11px` bold muted ;
4. bouton `Search notes`, `24 × 24px`, icône `Search 14px` ;
5. arbre lazy.

Une entrée est visible si elle est un dossier ou un chemin `.md` insensible à
la casse, et si aucun composant de son chemin ne commence par `.`. Les fichiers
image, binaires, `.assets`, `.dashboard`, `.elephantnote` et autres chemins
internes ne sont jamais rendus dans l’arbre.

Chaque ligne mesure `36px`, reçoit un padding gauche de
`6px + depth × 14px`, un gap de `6px` et un rayon de `8px` :

- dossier : chevron `ChevronRight 15px` ou `ChevronDown 15px`, bouton de toggle
  `22 × 22px`, titre, compteur de notes si supérieur à zéro ;
- note : titre direct sans chevron ;
- entrée active ou survolée : couleur texte et fond soft ;
- entrée en drag : opacité `0.45` ;
- cible de drop autorisée : bordure primaire `1px` ;
- cible refusée : bordure danger `1px` ;
- erreur de lecture d’un dossier : ligne rouge `Unable to read <title>: <error>`.

Cliquer le chevron ne navigue pas : il inverse seulement l’ensemble
`expanded_paths`. Cliquer le titre d’un dossier ouvre le dossier et l’ajoute à
l’historique ; cliquer le titre d’une note ouvre le fichier. Un dossier actif
est aussi considéré expanded, ce qui reconstruit la branche après reload.

Le redimensionnement :

- pointerdown primaire dans la zone mesurée du resizer capture `start_x` et la
  largeur courante ;
- pointermove met à jour `start_width + delta_x` ;
- la valeur est clampée `184..320` ;
- pointerup persiste la largeur ;
- `ArrowLeft` et `ArrowRight` déplacent par pas de `16px` et persistent ;
- la poignée passe de la couleur bg à primary au hover.

Le DnD sidebar exige un mouvement de `4px` minimum. Une cible est refusée si
elle est le parent actuel, la source elle-même, ou un descendant de la source
pour un dossier. Un move accepté :

1. appelle le déplacement réel du `VaultAdapter` ;
2. calcule le nouveau chemin relatif ;
3. rebasing `current_path` et les chemins descendants ;
4. recharge la directory ;
5. recharge l’éditeur si le fichier ouvert a été déplacé ;
6. rebasing `expanded_paths` ;
7. signale une erreur visible et loggée au moindre échec.

### 31.7 Bibliothèque native : toolbar, menu, cartes et pages

La bibliothèque native reprend les contrats `LibraryToolbar`, `CreateEntryMenu`,
`LibraryGrid` et `NoteCard` de la cible Vue.

#### Toolbar

La toolbar absolue occupe `72px` de haut. Les boutons sont à droite, chacun
`52 × 52px`, rayon `12px`, bordure `1px`, gap `14px` :

- Sort : aria `Sort: Updated newest|Updated oldest|Title A-Z|Title Z-A` ;
- View : aria `Show notes as list` en mode grid, `Show notes as grid` en mode
  list ;
- icônes Sort : `ArrowDownNarrowWide`, `ArrowUpNarrowWide`, `ArrowDownAz`,
  `ArrowDownZa` ;
- icônes View : `List` ou `Grid3x3`, tailles `22px`.

Le cycle de tri est strictement : `updated-newest → updated-oldest →
title-az → title-za → updated-newest`. Le mode d’affichage alterne
`grid ↔ list`. Le tri garde les entrées pinned en tête, puis applique le mode
à chaque groupe.

#### Create

Le FAB `Create` est `56 × 56px`, icône `Plus 27px`, fond primary, positionné
à `right 20px; bottom 20px`. Le clic ne crée pas immédiatement : il inverse
`menu_open` et monte un backdrop global avec le popover `Create`.

Le menu est une surface `280px` avec `CREATE` en `12px` bold muted et trois
options `52px` :

| Option | Icône | Description | Effet |
|---|---|---|---|
| `Note` | `FilePlus2 20px` | `Create a new note` | crée le Markdown puis ouvre le document ; |
| `Drawing` | `Excalidraw 20px` | `Open a new Excalidraw canvas` | délègue à `drawing::request_create` ; |
| `Folder` | `FolderPlus 20px` | `Organize notes in a folder` | crée le dossier puis recharge le répertoire. |

Le clic sur le backdrop ferme le menu. `Escape` le ferme. Les deux routes de
pointer (`on_mouse_up` et fallback global dans la zone mesurée) sont
idempotentes : une action ne doit pas créer deux notes.

Dans l’état Freya actuel, la création Drawing renvoie explicitement
`Drawing requires the Excalidraw web island; no fake native fallback is used.`
Cette erreur est visible et loggée ; il ne faut pas la remplacer par un succès
factice.

#### Pages et chargement

Une ouverture de dossier commence une génération de requête, demande `121`
entrées au backend pour stocker au plus `120`, révèle d’abord `72` items et
précharge la page suivante à moins de `720px` de la fin. Une réponse dont la
génération, le chemin, la vue ou l’id de vault ne correspondent plus est
ignorée. Une page suivante déduplique par chemin.

Les chemins sont relatifs, séparés par `/`. La racine `wiki` en mode Notes est
filtrée comme compatibilité et ne devient pas une carte normale.

#### Carte

Une carte porte le label accessible du titre et l’icône `MoreHorizontal`.
L’icône principale est `Folder` pour un dossier, `PenLine` pour un drawing et
`FileText` pour une note. La taille est `22px` en grid et `20px` en list.

- un clic sur le corps programme l’ouverture après `220ms` ;
- le timer est attaché à la carte cible et non aux coordonnées globales ;
- double-clic sur le titre annule le timer, entre en renommage, sélectionne
  l’input et ne navigue pas ;
- Enter commit le titre trimé ; Escape annule ; titre vide ou inchangé est
  ignoré ;
- `MoreHorizontal` ouvre le menu local de la carte et ne déclenche pas
  l’ouverture ;
- Pin, Rename, Delete et Show/Hide from sidebar opèrent sur le chemin capturé
  de cette carte ;
- une carte en drag exige `4px`, affiche opacity `0.45` pour la source ;
- seuls les dossiers acceptent un drop ; self, parent direct et descendant
  sont refusés ;
- le drop accepté appelle le move réel puis recharge l’affichage.

Une carte dossier montre au plus trois `children_preview`. Les titres retirent
`.md` et `.excalidraw` pour l’affichage. Un preview vide affiche `No items yet`
ou `Empty folder` selon le type de source. Une note affiche l’excerpt et les
tags.

### 31.8 Sélecteur initial, registre et changements de branche

Au démarrage normal, le shell essaie dans cet ordre :

1. `ELEPHANT_FREYA_VAULT` si l’environnement est défini ;
2. le vault actif du registre ;
3. le dernier vault mémorisé par `vault_picker` ;
4. l’état vide récupérable.

Le registre utilisateur utilise `elephantnote.json`, version `1`, dans le
profil Freya. `ELEPHANT_FREYA_PROFILE` remplace le profil pour les tests ; dans
ce cas le fichier est `<profile>/elephantnote.json`. Le registre conserve les
vaults disparus afin d’afficher une action de récupération et ne supprime pas
le dossier utilisateur lorsqu’un vault est retiré.

`EmptyVaultPicker` natif :

- occupe tout le viewport ;
- carte largeur `420px`, padding `28px`, fond surface, bordure `1px`, rayon
  `14px` ;
- titre `Choose a vault`, taille `22px` ;
- description `Select an existing Elephant vault folder to continue.` ;
- erreurs sous le label `Vault unavailable` ;
- section `Registered vaults` si le registre contient des entries enabled ;
- une action par vault `Open <name>` avec aria `Open registered vault <name>` ;
- bouton `Choose folder` qui ouvre le picker natif ;
- annulation du picker : aucun changement ;
- erreur : reste visible dans l’état `error`.

Une ouverture réussie canonicalise le chemin, initialise le layout caché et le
workspace, ajoute/active le descriptor, persiste le registre et mémorise le
vault. Une erreur d’enregistrement ou de mémorisation ne doit pas masquer le
vault ouvert : le shell conserve le vault mais expose l’erreur.

### 31.9 Vues de workspace et navigation historique

`WorkspaceView` accepte exactement : `Notes`, `Wiki`, `Chat`, `Dashboard`,
`Canvas`, `Graph`, `Calendar`, `Models`, `Sync` et `Addon(String)`. Les IDs
sources sont `notes`, `wiki`, `chat`, `dashboard`, `canvas`, `graph`,
`calendar`, `models`, `sync`, ou l’id addon.

`NavigationTarget` ne stocke que `Directory(String)` et `Note(String)`. Les
règles sont :

- ne pas ajouter deux fois la même cible à l’index courant ;
- une nouvelle navigation après Back tronque la branche Forward ;
- conserver au plus `100` entrées ; au dépassement retirer les `20` plus
  anciennes ;
- Back est disabled à l’index `0` ; Forward est disabled au dernier index ;
- rouvrir une note par historique recharge le répertoire parent puis lit le
  fichier ;
- si la cible physique a disparu, ne pas ouvrir un éditeur vide : afficher
  `Navigation target is no longer present: <path>`.

Ouvrir une vue secondaire efface l’éditeur et les ressources spécifiques de la
vue précédente. Le dashboard crée au besoin `.elephantnote/Dashboard.md` avec
`# Dashboard`, puis l’ouvre comme un vrai document ; ce chemin interne reste
hors de l’arbre visible.

## 32. Éditeur natif Freya et Muya

L’éditeur Freya n’est pas un champ texte qui simule Markdown. `EditorDocument`
charge le fichier réel, crée une session `muya-core`, garde la révision, la
sélection, l’historique et sérialise le document Muya au moment de l’écriture.

### 32.1 Dimensions et états de l’éditeur

Sources : `Elephant/freya/src/editor.rs`, `Elephant/freya/src/app/editor_view.rs`.

| Élément | Valeur |
|---|---:|
| largeur de contenu | `780px` maximum ; |
| corps | `16px` ; |
| line-height | `1.58` ; |
| topbar note normale | `52px` ; |
| topbar note compacte après scroll | `36px` ; |
| action topbar | `30 × 30px` ; |
| toolbar formatage | `56px` ; |
| bouton toolbar | `34 × 34px` ; |
| footer | `50px` ; |
| police UI | `sans-serif` dans la vue native ; |
| police code | `monospace` ; |
| pin actif | `rgb(250,204,21)` ; |
| seuil de compaction | scroll top `>24px` ; |
| position verticale approximative d’un fragment | `line_index × 24px`. |

`EditorDocument` expose : `path`, `session`, `saved_markdown`, `dirty`,
`last_edit_at`, `preferences`, `view_state`, `focus_target`,
`composition_selection` et `autosave_failure_revision`.

`EditorUpdate` renvoie toujours :

```ts
{
  markdown: string,
  revision: number,
  selection: { anchor, focus },
  patches: ViewPatch[],
  canUndo: boolean,
  canRedo: boolean,
  compositionActive: boolean
}
```

Une mutation réussie met à jour le document, la révision et la sélection,
marque dirty si le Markdown diffère du dernier write, incrémente la génération
d’autosave et peut déplacer le focus vers le nouveau node. Un write qui échoue
ne remet jamais `dirty` à false.

### 32.2 Topbar de note

La topbar a aria `NoteEditorHost`, contient un champ `Note title`, des chips de
tags et les actions :

| Label | Icône/texte | État disabled/actif | Effet |
|---|---|---|---|
| `Undo` | `↶` | disabled si `can_undo == false` | annule une transaction Muya ; |
| `Redo` | `↷` | disabled si `can_redo == false` | réapplique une transaction Muya ; |
| `Save` | `✓` | toujours disponible quand le document est ouvert, dirty visuellement | renomme si le titre a changé puis écrit le Markdown réel ; |
| `Pin note` / `Unpin note` | `⌖` | disabled sans chemin relatif | met à jour les pins de workspace ; |
| `Close note` | `×` | actif | flush le dirty document puis retire l’éditeur ; |
| `Add tag` | `+` | actif | ouvre le formulaire de tag ; |
| `Cancel tag` | `×` | formulaire ouvert | ferme le formulaire sans write ; |
| `Save` du formulaire | `✓` | formulaire ouvert | ajoute ou remplace un tag et persiste ; |
| `Tag` | chip `#tag` | actif | clic : ouvre l’édition du chip ; context-click : supprime le tag. |

Le champ titre trim la valeur. Un titre vide rétablit l’ancien titre. Un titre
valide est réécrit dans le H1/frontmatter par `rename_title`, puis sauvegardé
sur disque avant de mettre à jour l’input.

Au maximum deux tags sont affichés individuellement dans la topbar ; si la
note en a davantage, un chip `+N` les représente. Le nom accessible d’un chip
conserve le préfixe `#` même si la préférence d’affichage le masque.

### 32.3 Toolbar de formatage

La toolbar native expose les actions suivantes dans l’ordre du code :

| Interaction | Label accessible | Commande Muya |
|---|---|---|
| `B` | `Bold` | `ToggleStrong` ; |
| `I` | `Italic` | `ToggleEmphasis` ; |
| `S` | `Strikethrough` | `ToggleStrike` ; |
| `H2` | `Heading 2` | `SetHeading(2)` ; |
| `•` | `Bulleted list` | `SetListKind(Unordered)` ; |
| `1.` | `Numbered list` | `SetListKind(Ordered)` ; |
| `☑` | `Task list` | `SetListKind(Task)` ; |
| `❞` | `Quote` | `ToggleBlockQuote` ; |
| `{}` | `Code block` | `ToggleCodeBlock` ; |
| `</>` | `Inline code` | paste transaction qui enveloppe la sélection ; |
| `↗` | `Link` | formulaire puis insertion du lien sur la sélection. |

`Inline code` est disabled et son aria devient `Inline code requires a
selection` sans sélection. `Link` suit la même règle avec `Link requires a
selection`.

Le formulaire Link mesure `42px` de haut, contient un input destination et
`Cancel link`. Enter exige une destination non vide et produit
`[label](destination)` après échappement du label et normalisation de la
destination. Escape ferme et vide le formulaire.

Les boutons suivent le cycle visuel hover/pressed de `action_button` :
background soft au hover, translation/état de pression natif, accent quand
l’action est active, puis log `[freya][editor] action:complete` ou
`action:failure`.

### 32.4 Rendu des blocs et des inline

Le document est rendu par `render_document` puis `render_block`. Les niveaux
de titre ont les tailles exactes : H1 `28px`, H2 `24px`, H3 `20px`, H4 `18px`,
H5 `17px`, H6 `16px`.

| Type Muya | Rendu natif |
|---|---|
| paragraphe | zone editable, texte `16px × text_scale`, line-height `1.58` ; |
| heading | texte bold avec taille de niveau ci-dessus ; |
| block quote | barre verticale `3px`, rayon `2px`, gap `12px` ; |
| code block | surface bordée, rayon `7px`, padding `6px`, police monospace, actions copy/run ; |
| liste unordered | marqueur `•`, gap `7px` ; |
| liste ordered | compteur `N.` ; |
| liste task | `☐` ou `☑`, hitbox `24 × 24px` ; |
| table | lignes/cellules rendues et navigation Tab/Shift+Tab ; |
| thematic break | ligne `1px` ; |
| image inline | placeholder danger `[Image non rendue: alt="…" source="…"]` dans la cible Freya actuelle ; |
| HTML inline | placeholder danger `[HTML inline non rendu: …]` ; |
| math inline | placeholder danger `[Math inline non rendu: …]` ; |
| footnote reference | texte `[footnote: label]` ; |
| emoji | valeur emoji ; |
| superscript/subscript | taille `12px × text_scale` ; |
| lien/auto-link | couleur primary, bouton d’ouverture séparé ; |
| bloc inconnu | `unsupported_block` avec label d’erreur, jamais ignoré silencieusement. |

Les styles inline se combinent : strong bold, emphasis italic, strike
line-through, code monospace, link primary, status danger. Les soft/hard
breaks deviennent un saut de ligne.

### 32.5 Édition, sélection et révisions

Chaque editable inline possède un `UseEditable` et une liste de nodes texte
Muya. La sélection UI est convertie en offsets UTF-16, puis remappée vers les
nodes Muya. Les bornes au milieu d’un surrogate pair sont arrondies au floor ou
au ceil selon le sens de la sélection ; un point de code Unicode ne doit donc
pas être coupé par la conversion (cela ne constitue pas une gestion générale
des graphèmes).

Une frappe suit ce chemin :

```text
key/IME/paste
  → capture sélection UI
  → comparer before/after ou router une commande explicite
  → résoudre les offsets UTF-16 vers NodeId + offset
  → vérifier expected_revision
  → dispatch Muya
  → récupérer markdown/revision/selection/patches
  → resynchroniser le champ editable
  → incrémenter autosave_generation
```

Une commande portant une révision périmée renvoie une erreur et ne modifie pas
le document. Les changements de sélection seuls gardent la révision ; la
mutation suivante doit utiliser cette même révision actualisée.

Les raccourcis spécialisés comprennent :

- Enter : `InsertParagraph`, avec fallback `ParagraphBoundary` pour une
  structure non supportée ;
- Backspace : `DeleteBackward` grapheme ;
- Delete : suppression forward avec fusion de blocs ;
- Ctrl/Cmd+Z : Undo ; Ctrl/Cmd+Shift+Z ou Ctrl/Cmd+Y : Redo selon le routeur ;
- Ctrl/Cmd+End : fin du bloc courant sans laisser le ScrollView interpréter le
  raccourci comme un saut global ;
- Tab / Shift+Tab en table : cellule suivante/précédente ;
- click d’un task marker : inverse `checked` avec `auto_check=false` ;
- IME preedit : begin/update/commit/cancel avec sélection de composition
  conservée entre les updates.

### 32.6 Autosave, fermeture et changements externes

Les préférences canonique sont `autoSave` (défaut `false`) et
`autoSaveDelay` (défaut `5000ms`). Le host ne lance pas d’autosave si le
document n’est pas dirty ou si l’autosave est désactivé.

Quand une mutation rend le document dirty :

1. capturer la génération courante ;
2. attendre un `Delay` non bloquant de la durée configurée ;
3. abandonner si une nouvelle génération existe ;
4. vérifier que la révision n’est pas marquée en échec ;
5. appeler `save_open_editor` ;
6. ne clear dirty qu’après le succès de `fs::write` ;
7. en cas d’échec, mémoriser la révision en erreur et afficher l’erreur.

La fermeture manuelle appelle `EditorDocument::close` : si dirty, elle tente
le save immédiat ; si le write échoue, l’éditeur reste monté et l’erreur est
visible. La fermeture réussie retire seulement l’état runtime.

Le watcher appelle `refresh_current_directory_from_external`. Pour une note
clean, `reload_external` remplace le document et conserve l’éditeur. Pour une
note dirty, il renvoie `ExternalConflict` avec le message
`the note changed on disk while it had unsaved Freya edits` et ne remplace ni
le Markdown local ni les bytes externes.

### 32.7 Liens et sécurité d’ouverture

Les liens externes acceptés commencent par `http://`, `https://`, `mailto:` ou
`ftp://`, puis passent par `open` (macOS), `xdg-open` (Linux) ou `cmd /C start`
(Windows). Le code de sortie non nul devient une erreur visible.

Les liens internes :

- refusent les chemins absolus, `..`, préfixes Windows et sorties du vault ;
- ajoutent `.md` si la cible sans extension n’existe pas ;
- canonicalisent source, cible et root ;
- ouvrent le dossier ou la note correspondante ;
- après `#fragment`, recherchent un heading H1–H6 et positionnent le scroll à
  `line_index × 24px` ;
- signalent `Link target is unavailable`, `Link target escapes the active vault`
  ou `Link anchor not found`.

### 32.8 Exécution de code dans un bloc

Le bouton `Copy code block` copie le texte non vide via le clipboard Freya. Le
bouton `Run code block` est disabled si le code est vide ou si une exécution
est déjà en cours.

Le service `elephant.code-execution` utilise le protocole
`elephant-addon-service-v1`, un process package-owned et le dossier
`.elephantnote/addons/data/elephant.code-execution`. Le mapping d’interpréteur
est :

| Langage Markdown | Exécutable | Arguments |
|---|---|---|
| JavaScript/JS/JSX/Node/MJS/CJS | `node` | `-` |
| Shell/sh/bash/zsh | `bash` | `-s` |
| tout autre langage | `python3` | `-` |

Avant `execute`, le host demande `interpreter.status`. Une disponibilité false
est une erreur, pas un succès. Le service reçoit `outputLineLimit=200` et
`timeoutMs=15000`. Le host poll `execution.status` toutes les `50ms`, a une
deadline de sécurité `125s`, annule à l’expiration, puis appelle
`execution.forget`. Le résultat distingue code numérique, `interrupted` et
`timeout`; stdout et stderr sont concaténés avec un saut de ligne. Le Drop du
client tue et attend le service.

## 33. Complément d'audit statique borné

Cette passe complète les éléments qui n'étaient pas référencés par leur fichier
source ou dont le contrat opérationnel était trop résumé. Tout ce qui suit est
`CODE` : aucune application Tauri, aucun APK, aucun service d'addon et aucun
sidecar n'a été lancé pendant cette passe.

### 33.1 Composants modernes non indexés

#### `NoteEditorHeader.vue`

Le header de note est distinct de la topbar historique. Il mesure au minimum
`74px`, possède un padding horizontal de `24px`, une bordure basse et un champ
de titre en `28px`, poids `800`. Le champ utilise `@change`, pas `@input`, et
émet `update-title` avec la valeur saisie.

À droite, deux boutons `36 × 36px` :

- `Pin note`/`Unpin note`, qui émet `toggle-pin` et reçoit la classe `active` ;
- `Close note`, qui émet `close`.

Les deux boutons sont `no-drag`; le pin actif colore et remplit l'icône en
jaune. Le composant expose uniquement les props `title` et `isPinned` et les
événements `update-title`, `toggle-pin` et `close`.

#### `NoteEditorMeta.vue`

La ligne de métadonnées mesure au minimum `48px`, possède un padding horizontal
de `24px`, une bordure basse et reste explicitement `no-drag`. Elle rend la date,
les chips de tags et `+ Add tag`.

Le composant garde localement l'état de création/édition du tag afin de
maintenir le focus même quand le parent met à jour les props. La création ou
l'édition :

1. monte un input autofocus de largeur `112px` (min `72px`, max `180px`) ;
2. émet `start-tag-creation` ou `edit-tag` et `update-tag-draft` ;
3. valide sur Enter ou blur après trim ;
4. émet `submit-tag` seulement pour une valeur non vide ;
5. annule sur Escape ou valeur vide et émet `cancel-tag`.

Les clics et les événements pointer des contrôles arrêtent la propagation pour
ne pas ouvrir ou fermer la note par le parent. Les événements publics complets
sont `start-tag-creation`, `edit-tag`, `delete-tag`, `submit-tag`, `cancel-tag`
et `update-tag-draft`.

#### `MainContent.vue` et panels de workspace

`MainContent` ne considère une note comme ouverte que si le chemin absolu de
`vaultStore.openedNotePath` correspond à `editorStore.currentFile` ou à un tab,
et que ce fichier possède à la fois un `id` et un `markdown` string. La
comparaison passe par `window.fileUtils.isSamePathSync` quand elle existe, avec
un fallback de normalisation des slashs.

La priorité de rendu est :

1. `AddonWorkspaceRouter` si une vue d'addon est active et qu'aucune note n'est
   ouverte ;
2. bibliothèque si le workspace est `notes` et qu'aucune vue d'addon n'est
   active ;
3. les contributions de zone `workspace.notes`, triées par `order` numérique,
   seulement en workspace `notes` sans note ouverte ;
4. `NoteEditorHost` dès que le test d'ouverture devient vrai.

Une contribution de zone peut fournir `when()`. Si ce prédicat lève une erreur,
le panel est masqué et l'erreur est écrite dans le log renderer avec son id ; le
shell ne remplace pas l'erreur par un panel vide. Les changements de visibilité
de note sont aussi loggés par le watcher `main-content` avec vault, chemins,
nombre de tabs et workspace.

#### `SyncNavigationControl.vue`

Le contrôle de synchronisation est disabled si le vault actif est absent, si
une sync est déjà en cours ou si aucun appareil Iroh n'est appairé. Ses titres
accessibles distinguent précisément : vérification en cours, aucun appareil,
sync en cours, erreur avec le message du store, état synchronisé et action
disponible.

Au montage et au focus de fenêtre, il rafraîchit le statut. Il écoute
`IROH_SYNC_STATUS_EVENT` et applique l'état reçu en conservant éventuellement
`running`. Un changement de vault efface l'état précédent puis relance le
refresh. Au démontage, les listeners sont retirés et l'état de navigation est
nettoyé. Le clic appelle uniquement `nav.syncWorkspace(activeVaultPath)` et
laisse au store l'état d'erreur durable.

### 33.2 Runtime mobile réellement assemblé

Les modules `mobileEditorRuntime.js`, `mobileInteractionRuntime.js`,
`mobileLibraryChromeRuntime.js`, `mobileNodeBuiltinsShim.js` et
`mobileVaultBridge.js` s'installent à l'import de module. Chaque installation
est idempotente par flag global et fournit une fonction `__DISPOSE__` dédiée.
La détection commune est `max-width: 760px` ou `(hover: none) and (pointer:
coarse)` ; cela décrit le code, pas une preuve de comportement sur un appareil.

#### Éditeur mobile, médias et sheets

Quand `.en-main.has-editor-open .en-editor-layer` existe, le runtime injecte un
`nav.en-mobile-editor-toolbar` avec cinq actions : `Insert`, `Formatting`,
`Undo`, `Redo`, `More`. Les deux actions d'historique émettent directement
`undo`/`redo` sur le bus ; les trois autres montent une sheet dialog avec
backdrop, fermeture par clic extérieur ou bouton Close, et boutons accessibles.

Les actions Insert sont `Take a photo`, `Add an image`, `Drawing`, `Checklist`,
`Bullet list`, `Numbered list`, `Quote`, `Table` et `Horizontal rule`. Formatting
propose Body text, Heading 1/2, Bold, Italic, Strikethrough, Inline code et
Link. More propose Share note, Duplicate note et Add or remove tags.

La photo demande d'abord les permissions via `__TAURI__.barcodeScanner` si
l'API existe, puis ouvre `getUserMedia` avec caméra arrière préférée et
contraintes idéales `1920 × 1080`. Le bouton Switch camera alterne
`environment`/`user`. La capture passe par un canvas JPEG qualité `0.9`, arrête
toutes les tracks et retire le listener `elephantnote:android-back` à la
fermeture. Une permission refusée devient une erreur lisible demandant
d'autoriser Camera dans les réglages Android.

Une image galerie utilise un input temporaire `image/*`. Photo ou galerie :

1. récupère le vault via `tauri_vaults_get` ;
2. crée `<vault>/.assets` via `fileUtils.ensureDir` ;
3. assainit le nom en ne gardant que `[a-zA-Z0-9._-]` ;
4. écrit le Blob avec `fileUtils.writeFile` ;
5. émet `insert-image` sur le bus puis
   `elephantnote:vault-files-changed`.

Le partage lit le titre `.en-note-title-input` et le Markdown exposé par
`editor.runtime`, puis appelle `tauri_android_share_text` ou, à défaut,
`navigator.share`. Si aucun des deux n'existe, l'action échoue explicitement.
Duplicate et tags émettent respectivement `elephantnote:duplicate-note` et
`elephantnote:open-tags`.

Le runtime suit aussi `visualViewport` pour calculer
`--en-mobile-keyboard-offset` et la classe `en-mobile-keyboard-open` quand le
décalage dépasse `80px`. La toolbar, l'observer et les listeners resize/scroll
sont supprimés par le disposer.

#### Retour Android et chrome de bibliothèque

`mobileInteractionRuntime.js` arme un sentinel `history.pushState` nommé
`__elephantMobileBackState`. Sur `popstate`, l'ordre de consommation est :

1. événement cancelable `elephantnote:android-back` ;
2. fermeture de Settings ;
3. fermeture de Search ;
4. fermeture de la note ;
5. fermeture du drawer via `.en-mobile-scrim.visible` ;
6. retour dans `navigationStore` puis navigation Tauri.

Après une action consommée, le sentinel est réarmé au tick suivant. Les erreurs
de traitement sont loggées sous `[mobile-navigation]` et le code ne prétend pas
avoir géré le retour si aucune de ces surfaces ne l'a consommé.

Quand la bibliothèque est visible, `mobileLibraryChromeRuntime.js` insère deux
contrôles dans `.en-mobile-topbar` avant Settings : bascule grid/list et sheet
de tri. Les quatre valeurs de tri sont `updated-newest`, `updated-oldest`,
`title-az` et `title-za`; le runtime clique le contrôle moderne jusqu'à atteindre
la valeur demandée. Il retire les contrôles dès que la bibliothèque disparaît.

#### Shim Node et vault Android

`mobileNodeBuiltinsShim.js` ne simule pas Node : `spawn`, `exec`, `execFile`,
`fork`, les opérations fs/zlib et les streams lèvent une erreur explicitement
Android. `existsSync` renvoie toujours `false`, `tmpdir()` renvoie `/tmp`,
`homedir()` renvoie `/`, `platform()` renvoie `android`. Le `createHash` fourni
est un FNV-1a local répété pour produire 40 caractères hexadécimaux ; ce n'est
pas une preuve ni un remplacement d'un hash cryptographique.

`mobileVaultBridge.js` persiste trois flags WebView :
`elephantnote:mobile-vault-choice-v4`,
`elephantnote:mobile-advanced-vault-v1` et
`elephantnote:mobile-advanced-vault-dirty-v1`. Sans choix explicite, le vault
automatique `<appDataDir>/vaults/Personal` est masqué du payload pour forcer le
choix utilisateur.

`createLocalVault` efface le provider Android si disponible puis sélectionne ce
chemin privé. `selectVault` Android appelle le picker DocumentProvider,
sélectionne le `shadowPath`, tente de donner au vault le `displayName`, puis
marque le choix comme avancé. Les annulations sont retournées comme
`{ canceled: true }`; une erreur qui ne ressemble pas à une annulation remonte.

Pour un vault avancé, les mutations et `pagehide` marquent le dirty flag et
programment `tauri_android_vault_sync` après `350ms` (ou immédiatement pour la
sortie). Au retour de l'application, `tauri_android_vault_restore` est appelé
avant `elephantnote:vault-files-changed`. Les erreurs de sync/restore sont
loggées sous `[mobile-vault]` et ne sont pas converties en succès.

### 33.3 Bridges legacy et menus contextuels

#### `runtimeBridge.js`

`installRuntimeBridge` installe toujours `window.path`, `window.commandExists`,
`window.i18nUtils` et `window.elephantnote`. Avec `window.__TAURI__`, il installe
la façade Tauri ; sans Tauri, il installe une façade `tauri-compatible` qui
fournit seulement des événements locaux, des fallbacks Web et des erreurs
explicites des opérations fichiers non disponibles.

La façade path est POSIX-normalisée (`/`), accepte préfixes `/`, `//` et lecteurs
Windows, et expose `normalize`, `join`, `resolve`, `dirname`, `basename`,
`extname`, `isAbsolute` et `relative`. La façade event bus traduit les appels
Electron en `CustomEvent` avec les arguments dans `detail`; `invoke` utilise un
handler local enregistré, retourne `true` pour `update-buffer-state`, et lève
`No invoke handler registered` sinon.

Les stores de compatibilité utilisent les clés :

| Donnée | Clé Tauri renderer | Comportement |
|---|---|---|
| keybindings | `elephantnote:tauri:user-keybindings` | defaults Mac/Windows/Linux puis merge utilisateur ; |
| spellchecker | `elephantnote:tauri:spellchecker` | enabled, language, words ; dictionnaire exposé : `en-US` ; |
| préférences | `elephantnote:tauri:preferences` | merge JSON et événement `language-changed` ; |
| user data | `elephantnote:tauri:user-data` | merge JSON ; |
| buffers | `elephantnote:tauri:buffer-state` | dernière payload reçue. |

L'écriture tente localStorage et conserve un `Map` mémoire si le WebView refuse
le stockage. `fileUtils` Tauri utilise un cache de metadata alimenté par
`stat`; `isFile`/`isDirectory` restent faux tant que le chemin n'a pas été
sondé. `readDir` convertit une erreur en liste vide, tandis que les fallbacks
`readFile`, `writeFile`, `move`, `copy`, `stat` et `ensureDir` lèvent une erreur
quand l'API native manque.

Les commandes natives dispatchées localement couvrent fermeture/nouvelle
fenêtre, ouverture de fichiers ou dossiers, préférences, données utilisateur,
autosave, import Pandoc, dossier d'images, langue, keybindings, fenêtre
Settings, réponse de recherche d'image et drop de fichiers. `make-screenshot`
et `check-for-update` rendent une notification d'indisponibilité explicite.
Les autres channels sont transmis au `eventBus`; `shell.exec` appelle seulement
le command `shell_exec` Tauri, avec cwd/env contrôlés par le backend.

#### `contextMenu/tabs` et `contextMenu/sideBar`

Les menus sont recréés à chaque ouverture afin de traduire le label dans la
langue courante. Le menu de tabs est dans l'ordre : Close this, Close others,
Close saved tabs, Close all tabs, séparateur, Rename, Copy path, Show in folder.
Chaque `RemoteMenuItem` reçoit `_tabId`; Rename/Copy/Show sont disabled sans
`tab.pathname`. Les actions émettent `TABS::close-this`, `TABS::close-others`,
`TABS::close-saved`, `TABS::close-all`, `TABS::rename`, `TABS::copy-path` et
`TABS::show-in-folder`.

Le menu sidebar est : New file, New directory, séparateur, Copy, Cut, Paste,
séparateur, Rename, Move to trash, séparateur, Show in folder. Paste est
disabled si `hasPathCache` est faux. Les actions émettent `SIDEBAR::new` avec
`file`/`directory`, `SIDEBAR::copy-cut` avec `copy`/`cut`, puis
`SIDEBAR::paste`, `SIDEBAR::rename`, `SIDEBAR::remove` et
`SIDEBAR::show-in-folder`. Le popup reçoit la fenêtre courante et les
coordonnées client du pointer event.

#### Primitives `prefComponents`

La sidebar legacy écoute `settings::change-tab`, navigue vers les routes
`/preference/*` et retire son listener au démontage. Sa recherche est générée
depuis `common/preferences/schema.json`, avec catégorie, clé, description et
valeurs d'enum traduites ; une erreur de navigation est loggée au lieu d'être
ignorée.

Les primitives ont des contrats distincts : `Bool` synchronise sa valeur par
watch et appelle `onChange` sur changement ; `Select` fait de même avec les
options `el-option` ; `Range` expose valeur, unité, min/max/step et appelle
`onChange` à la fin du slider ; `TextBox` valide par RegExp et debounce par
défaut de `800ms` (`emitTime: 0` signifie immédiat) ; `FontTextBox` charge
facultativement `font-list` via `window.require` et ne valide une famille que
si son nom respecte le format sans espace initial/final ; `Compound` et
`Separator` structurent la section sans persistance autonome. Tous les liens
`more` passent par `window.tauri.shell.openExternal`.

### 33.4 Backend Rust : état, filesystem et addons

#### Enregistrement et état global

`lib_min.rs` installe les plugins fs, opener, clipboard, dialog, notification,
le plugin de vault Android et, hors mobile, le plugin de window state. Au setup,
il gère `AppState`, `WatcherState`, `AddonState`, `AddonSidecarState`,
`AddonServiceState`, les sessions Muya et l'état acceptance. Tous les appels
décrits dans cette référence doivent être ajoutés à
`tauri::generate_handler!`; un module exporté seul n'est pas une commande
atteignable.

`AppState` charge au démarrage :

| Store | Fichier de profil |
|---|---|
| `Preferences` | `app_config_dir/preferences.json` |
| `DataCenter` | `app_data_dir/userData.json` ; secrets hors JSON via keyring quand disponible |
| `BufferStore` | `app_data_dir/window_buffers/<windowId>.json` |
| `RecentsStore` | `app_config_dir/recently-used-documents.json`, au plus 20 chemins |
| `KeybindingsStore` | `app_config_dir/keybindings.json` |
| `AtomicFeatureStore` | `app_data_dir/atomic-features.json` |

Les commandes `tauri_prefs_*`, `tauri_user_data_*`, `tauri_secret_*`,
`tauri_buffer_*`, `tauri_recents_*`, `tauri_keybindings_*` et
`tauri_atomic_features_*` verrouillent le store correspondant et renvoient une
`Result`; une store absente renvoie une erreur pour les écritures ou une valeur
vide/default pour les lectures selon la commande. Les writes JSON passent par
les helpers atomiques du backend.

#### `filesystem.rs` : contrat bas niveau à ne pas confondre avec Vault API

`tauri_fs_read_markdown` et `tauri_fs_write_markdown` travaillent sur le
`path: String` fourni et transportent `markdown`, `filename`, `pathname`,
`encoding`, BOM, line ending, `adjust_line_ending_on_save`, mixed line endings
et `trim_trailing_newline`. La lecture détecte UTF-8/UTF-16 BOM puis chardetng;
une écriture peut encoder UTF-8, UTF-16LE/BE, GB18030, Big5, EUC-KR, EUC-JP,
Shift-JIS ou Windows-1252, avec conversion LF/CRLF et write atomique.

Point important pour un recodage : contrairement aux commandes
`tauri_vault_*` de `vault_file_commands.rs`, ces commandes bas niveau reçoivent
un chemin système directement et `tauri_fs_trash_item` peut supprimer un
fichier ou dossier par ce chemin. Le code de `filesystem.rs` ne fait pas de
containment avec le root du vault. Elles ne constituent donc pas à elles seules
une API de vault sûre ; le chemin doit être validé par l'appelant ou la route
doit être remplacée par les commandes vault guardées.

`tauri_vault_read_binary`, `write_binary`, `ensure_dir`, `remove_path` et
`rename_path`, eux, canonicalisent le root actif, refusent NUL, `..`, les
sorties du root et la suppression du root lui-même ; les parents d'une écriture
doivent aussi rester dans le root. Ce sont les commandes utilisées par les
addons qui manipulent des assets ou des builds.

#### Garde-fous addons

Le registre addon est version `1`. `read_enabled_addon` exige un vault actif,
un `registry.json` valide, un addon connu et `enabled: true`. Les chemins addon
sont relatifs, sans absolu, préfixe, `..` ou composant parent ; les scopes `*`,
`Inbox/**` et les chemins exacts sont boundary-aware.

L'accès notes impose : profondeur maximale `64`, au plus `1000` notes listées,
lecture/écriture d'une note de `5 MiB` maximum, exclusion de tout composant
commençant par `.` et écriture atomique avec `overwrite` explicite. L'accès HTTP
impose HTTPS, port `443`, aucune credential dans l'URL, hôtes accordés par le
manifest, adresses publiques seulement, corps de requête de `1 MiB`, réponse de
`5 MiB` et au plus cinq redirects, les redirects étant réservées à GET.

Les dépendances empêchent d'activer un addon si une dépendance versionnée n'est
pas installée et active ; elles empêchent aussi de désactiver ou désinstaller
un addon qui possède des dépendants installés/actifs.

Les services persistants utilisent `elephant-addon-service-v1`, timeout par
défaut `30s`, maximum `30min`, démarrage `20s`, arrêt `5s`, réponse maximum
`16 MiB`. Sur desktop l'exécutable est résolu dans le package, rendu
exécutable, lancé avec stdin/stdout JSON line et stderr relayé sous
`[addon-service:<id>]`; il est tué si le process tombe ou lors de l'arrêt. Sur
mobile, seul un addon officiel déclarant `native.mobile.<os>` avec
`runner: embedded-rust`, `supported: true` et un host supporté peut utiliser un
service embarqué.

Les sidecars process utilisent `elephant-addon-sidecar-v1`, timeout par défaut
`30s`, maximum `120s` et sortie cumulée maximale `16 MiB`. Ils exigent
`permissions.native`, un exécutable déclaré pour la plateforme, et sont
explicitement desktop-only ; Android/iOS doivent fournir un adaptateur package
host au lieu d'un process téléchargé.

Le catalogue distant est HTTPS sous la branche `addon-catalog`, limité à `1 MiB`
pour `catalog.json`, `256 KiB` par manifest, `5 MiB` par entry et `250 MiB` par
package. Les packages précompilés exigent un hash Blake3 hexadécimal de 64
caractères ; un addon natif source-only est refusé. Les packages officiels sont
marqués par `.elephant-official-package.json`, lié à leur `packageHash`.

### 33.5 Matrice exacte des manifests officiels

Les contributions et permissions ci-dessous sont celles des
`addons/official/*/manifest.json`; elles ne prouvent ni installation ni
disponibilité du runtime.

| ID/version | Entrée worker | Permissions notables | Dépendances/native/mobile |
|---|---|---|---|
| `elephant.ai` 2.2.0 | `main.js` | storage, secrets, réseau `api.openai.com`, `openrouter.ai`, `api.mistral.ai` | trusted, startup |
| `elephant.ai-chat` 2.0.0 | `main.v2.js` | storage, commands, views | parent `elephant.ai >=2.1.0`, trusted |
| `elephant.ai-ocr` 1.0.0 | `main.js` | storage, commands, native | sidecar process desktop ; Android/iOS false |
| `elephant.ai-search` 1.3.0 | `main.js` | storage, commands, notes read `*`, write `[]` | `ai >=2.2.0`, `knowledge >=1.1.0`, trusted |
| `elephant.calendar` 1.3.0 | `main.js` | storage, commands, views | trusted, startup |
| `elephant.code-execution` 2.2.0 | `main.js` | storage, commands, native | service desktop ; Android/iOS false |
| `elephant.codex-connection` 2.0.0 | `main.js` | storage, commands, views, native | `ai >=2.1.0`, service desktop ; mobile false |
| `elephant.dashboard` 1.0.1 | `main.js` | commands | trusted, startup |
| `elephant.google-keep-import` 1.2.0 | `main.js` | commands, settings, write `Imported/Google Keep/**` | trusted, startup |
| `elephant.graph` 1.3.0 | `main.v2.js` | notes read `*`, commands, views | `ai >=2.2.0`, trusted |
| `elephant.knowledge` 1.2.0 | `main.js` | storage, commands, views, native | service desktop ; Android/iOS embedded `elephant-knowledge-v1` |
| `elephant.open-models` 2.0.0 | `main.js` | storage, commands, views, native | `ai >=2.2.0`, service desktop ; mobile false |
| `elephant.recently-edited` 1.1.0 | `main.js` | views | trusted, startup |
| `elephant.sites` 1.4.0 | `main.js` | storage, commands, views, read `*`, write `.elephantnote/site-previews/**` et `.elephantnote/site-builds/**` | trusted ; pas de host réseau déclaré |
| `elephant.sync` 1.2.0 | `main.service.js` | storage, commands, views, native | service desktop ; Android/iOS false |
| `elephant.wiki` 1.5.0 | `main.v2.js` | storage, commands, views, read `*`, write `Wiki/**` | `ai >=2.2.0`, `knowledge >=1.1.0`, trusted |

Les manifests dont `native.mobile.android`/`ios` est `supported: false` ne
deviennent pas disponibles sur mobile par la seule présence de leur worker.
`elephant.knowledge` est l'exception de cette matrice : ses deux hosts mobiles
sont déclarés `embedded-rust`. Les permissions `notes.write` restent des
scopes exacts ; elles ne donnent pas un accès disque général.

### 33.6 Preuve de cette passe

Preuve réalisée : lecture statique des sources et modification de ce document.

Preuve non réalisée : démarrage Tauri, fenêtre graphique, parcours vault,
permissions Android, APK, installation/activation d'addon, sidecar/service,
sync, OCR, provider IA, test d'édition ou logs runtime. La seule validation
demandée pour cette passe est le contrôle Prettier du fichier présent.
