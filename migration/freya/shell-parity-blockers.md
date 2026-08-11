# Audit des bloqueurs de parité du shell

Date de l’audit : 2026-08-11  
Périmètre : navigation, rail, sidebar et library Vue actuels comparés au rendu Freya actuel.  
Méthode : inspection des sources et des tests acceptance présents dans le dépôt. Aucun sélecteur, état ou comportement absent des sources n’est supposé.

## Verdict

État : **NOT PROVEN — bloqueurs critiques**.

La parité visuelle et fonctionnelle n’est pas démontrée. Le socle d’acceptance nommé Playwright n’exécute pas Tauri dans l’état actuel : `tests/app/e2e/helpers.js:4,169-176` importe `playwright` via `_electron` et lance `electron-main.js`; les tests de référence se nomment eux-mêmes « Electron baseline » (`tests/app/e2e/parity/ui-parity-contract.spec.js:17-91`). Ces résultats ne constituent donc pas une preuve Tauri.

Freya est testée avec `TestingRunner`, mais les tests acceptance actuels vérifient principalement des labels et quelques clics (`Elephant/freya/tests/freya_shell_acceptance.rs:39-61`; `Elephant/freya/tests/shell_freya_testing.rs:96-115`). Ils ne vérifient ni géométrie comparable, ni séquence de frames, ni hover, ni déplacement de pointeur, ni drag/drop, ni état persistant après redémarrage.

Les différences ci-dessous sont donc des écarts observables ou des preuves manquantes, pas des comportements déduits.

## 1. Sélecteurs, labels et contrats d’accessibilité

### Vue/Tauri-renderer de référence

Les scénarios actuels s’appuient sur des sélecteurs stables et des attributs d’état :

- shell et zones : `.en-shell`, `.en-body`, `.en-body-main`, `.en-sidebar-resizer`, `data-sidebar-resizer` (`Elephant/frontend/app/components/shell/AppShell.vue:9,84-110`);
- rail et navigation : `.en-rail`, `.en-rail-nav`, `.en-rail-icon`, `.en-rail-sidebar-toggle`, `.en-rail-vault`, labels `Search`, `Settings`, `Hide sidebar`/`Show sidebar` (`Elephant/frontend/app/components/navigation/IconRail.vue:9-31,287-341`);
- sidebar : `.en-sidebar`, `.en-all-notes`, `.en-sidebar-tree-row`, `.en-sidebar-tree-note`, `.en-sidebar-tree-toggle`, `.en-sidebar-tree-remove`, `Search notes` (`Elephant/frontend/app/components/navigation/SidebarNav.vue:2-36`; `Elephant/frontend/app/components/navigation/SidebarTreeEntry.vue:8-94`);
- library : `.en-library-toolbar`, `.en-create-button`, `.en-sort-cycle`, `.en-view-cycle`, `.en-library-grid`, `data-sort`, `data-view-mode`, `data-layout` (`Elephant/frontend/app/components/library/LibraryToolbar.vue:2-61`; `Elephant/frontend/app/components/library/LibraryGrid.vue:3-35`);
- menus et actions : `role="menu"`, `role="menuitem"`, `data-testid="excalidraw-logo"`, `[data-entry-action]`, `[data-entry-rename-input]` (`Elephant/frontend/app/components/library/CreateEntryMenu.vue:15-38`; `Elephant/frontend/app/components/library/NoteCard.vue:47-104`).

### Freya actuelle

Freya expose des `a11y_alt` pour `TopVaultBar`, `Retour`, `Avancer`, `Hide sidebar`/`Show sidebar`, `Search`, `Settings`, `All notes`, `Search notes`, les entrées et les contrôles de library (`Elephant/freya/src/app/navigation.rs:9-48,52-126,194-252`; `Elephant/freya/src/app/library.rs:50-106,202-237`).

En revanche, les éléments Freya audités n’exposent pas les équivalents des classes, `data-*`, rôles de menu ou `data-testid` ci-dessus. Les tests Freya recherchent un label dans l’arbre d’accessibilité (`Elephant/freya/tests/freya_shell_acceptance.rs:23-36`; `Elephant/freya/tests/shell_freya_testing.rs:45-67`). Ce mapping par texte ne prouve pas que la même cible, la même zone de hit-test ou le même état DOM/visuel existe.

### Écarts de contenu de sidebar

Vue filtre explicitement les entrées masquées, les chemins cachés et tout ce qui n’est ni dossier ni Markdown (`Elephant/frontend/app/components/navigation/SidebarNav.vue:79-102`). Freya rend directement `snapshot.page.entries` (`Elephant/freya/src/app/navigation.rs:212-218`) et affiche le titre avec un préfixe `▸` pour les répertoires (`:257-283`). Il n’existe pas dans ce rendu de bouton d’expansion équivalent au `.en-sidebar-tree-toggle`, ni de bouton de suppression `Remove … from sidebar`.

Autre différence de source de données : Vue rend `store.rootSidebarEntries` (`SidebarNav.vue:98`) et charge récursivement les enfants ; Freya rend la page de vault actuellement chargée (`app.rs:82-99`, `navigation.rs:212-252`). Après ouverture d’un dossier, ces deux structures peuvent donc afficher des arbres différents.

## 2. Géométrie et composition

Les valeurs communes ne suffisent pas à établir une égalité de rendu.

| Zone | Vue | Freya | Différence observable / preuve manquante |
| --- | --- | --- | --- |
| Barre haute | Les boutons de navigation font `24×24` et la barre fait `24px` (`NavigationBar.vue:65-77`). | `TOPBAR_HEIGHT = 32` et les boutons sont placés dans un conteneur absolu (`app/navigation.rs:9-48`; `theme.rs:16-20`). | Hauteur de la barre différente dans les sources ; le décalage vertical global est donc différent avant même le contenu. |
| Rail | Largeur fixe `48px`, padding haut `8px` ou `36px` sur macOS, boutons `34×34` (`IconRail.vue:505-540`). | Largeur `48px`, padding uniforme `7px`, actions de `34px` (`app/navigation.rs:52-74`; `theme.rs:17`). | Même largeur nominale mais pas même padding, pas même traitement macOS, pas mêmes éléments ni icônes. |
| Sidebar | Largeur runtime par défaut `232px`, bornes `184..320`, grille `var(--en-sidebar-width) 0 minmax(0,1fr)` (`AppShell.vue:480-526,672-680`). | Le contrat déclare les mêmes bornes et la même clé de stockage (`navigation_contract.rs:23-26,58-68`), mais le rendu fixe la largeur à `snapshot.sidebar_width` (`app/navigation.rs:223`). | La borne existe dans le contrat, mais aucune interaction de redimensionnement équivalente n’est branchée dans le rendu Freya audité. |
| Toolbar | Couche absolue, `z-index:10`, zones pointer-transparentes, boutons `56px` et `52px` (`LibraryToolbar.vue:131-154,215-248`). | Toolbar absolue `72px` ; bouton Create `56px`, tri/vue `52px` (`app/library.rs:50-106`). | Les tailles principales sont proches, mais styles, hit-test, placement et comportement de couche ne sont pas équivalents. |
| Grille | Surface avec padding horizontal `10px`, `gap:10px`, colonnes `repeat(auto-fill,minmax(min(240px,100%),1fr))` (`LibraryGrid.vue:325-339`). | Surface horizontale avec `Content::wrap_spacing(10.)`, cartes grid à largeur fixe `240px` (`app/library.rs:185-196,214-220`). | À largeur disponible variable, le nombre de colonnes et les largeurs résultantes peuvent diverger ; aucune comparaison de bounding boxes n’est présente dans les tests Freya. |
| Cartes | Hauteur minimale `176px`, premier élément note potentiellement `220px`, dossier featured ramené à `176px` (`NoteCard.vue:433-459`). | Hauteur grid constante `176px`, liste constante `58px` (`app/library.rs:202-220`; `theme.rs:19-20`). | Freya ne reproduit pas l’état `featured` et ne rend pas la même composition de carte. |

Le test Vue vérifie ces relations géométriques et les hit-targets (`tests/app/e2e/library-shell-regressions.spec.js:79-142`). Aucun test Freya actuel ne mesure les rectangles, les offsets, les colonnes ou les cibles de pointeur.

## 3. États rendus et actions de navigation

### États présents côté Vue

Le shell Vue possède notamment : sidebar visible/cachée, drawer mobile ouvert/fermé/en déplacement/en settling, redimensionnement, thème et persistance de largeur (`AppShell.vue:15-25,297-408,480-526,687-751`). Les entrées d’arbre ont les états actif, hover, dragging, drop-target et drop-disabled (`SidebarTreeEntry.vue:8-19,61-94,261-302`). Les cartes ont pinned, hover, menu ouvert, renaming, dragging, drop-target et drop-disabled (`NoteCard.vue:3-20,24-104,191-200,431-554`).

### États Freya effectivement reliés au rendu

`ShellState` contient `sidebar_visible`, `menu_open`, `vault_menu_open`, `search_open`, `settings_open`, `editor` et `error` (`Elephant/freya/src/app.rs:21-38,41-55`). `LibraryState` contient bien des champs de tri, vue, pagination, phase et `drop_target` (`Elephant/freya/src/library_contract.rs:391-423`), mais `app/library.rs` ne relie au rendu que l’ouverture de menu, le cycle de tri, le cycle de vue et l’ouverture d’une carte (`:50-106,108-172,174-264`).

Écarts observables dans le shell Freya actuel :

- les boutons `Retour` et `Avancer` sont rendus avec un label et un glyphe, mais aucun `on_mouse_up`/handler de navigation n’est attaché dans `app/navigation.rs:9-48`; Vue relie les deux boutons à `goBack` et `goForward` (`NavigationBar.vue:5-28,51-58`);
- `Search notes` est un label dans Freya (`app/navigation.rs:229-244`) sans handler visible, alors que Vue émet `search` au clic (`SidebarNav.vue:24-30`);
- l’arbre Freya n’a pas les états hover, dragging, drop-target, drop-disabled, expanded/collapsed ou detachable rendus par Vue (`SidebarTreeEntry.vue:8-94,206-249,261-349`);
- les cartes Freya n’ont pas les boutons pin/menu, rename, sidebar, delete, ni l’état de renommage inline présents dans Vue (`NoteCard.vue:24-104,299-388`; `FolderCard.vue:3-95,180-244`);
- les tags sont transportés dans `LibraryEntry` (`library.rs:290-305`) mais ne sont pas rendus par `library_card` (`library.rs:240-264`), alors que Vue rend `.en-tags` (`NoteCard.vue:116-119`);
- les aperçus de drawing et les images Excalidraw ne sont pas rendus par `library_card`; Vue prévoit une branche image et le chargement du preview (`NoteCard.vue:119-151,602-618`).

## 4. Hover, mouvement et transitions

La demande de parité concerne les parcours en mouvement, pas seulement l’image au repos.

### Vue

- rail et vault menu : ouverture/fermeture sur `mouseenter`/`mouseleave`, transitions d’opacité et hover (`IconRail.vue:57-78,508-515,537-540`);
- boutons, cartes, menus et resizer : styles `:hover`, transitions, apparition des actions et indicateur du resizer (`NavigationBar.vue:91-98`; `LibraryToolbar.vue:171-199,247-248`; `NoteCard.vue:14-20,277-286,463-478`; `AppShell.vue:709-726`);
- drawer mobile : `pointerdown`, `pointermove`, `pointerup`, axe de geste, vélocité, projection et settling à `280ms` (`AppShell.vue:22-25,328-391,750-752`);
- redimensionnement : capture du pointeur, `requestAnimationFrame`, mise à jour continue puis persistance au `pointerup` (`AppShell.vue:498-526`).

### Freya

Les surfaces auditées n’utilisent que des handlers `on_mouse_up` pour les actions (`app/navigation.rs:91,108,174,202,268`; `app/library.rs:58,72,81,118-121,224-234`). Aucun handler de hover, mouvement de pointeur, capture de pointeur, transition de drawer, animation de resizer ou état intermédiaire de drag n’est présent dans ces rendus.

La capacité de `freya-testing` à déplacer le curseur et à échantillonner des frames ne prouve donc pas la parité : le parcours Vue expose des états temporels que le rendu Freya actuel ne produit pas. Les tests Freya existants ne font aucun déplacement ni rendu de séquence (`freya_shell_acceptance.rs:39-61`; `shell_freya_testing.rs:60-67,96-115`).

## 5. Drag/drop

### Référence Vue

Le drag/drop existe dans plusieurs parcours distincts :

- rail : réordonnancement de `Search` et du toggle sidebar (`IconRail.vue:24-29,436-463`), avec persistance via `preferences.SET_SINGLE_PREFERENCE` (`:451-459`);
- sidebar root : dragover/dragleave/drop vers `All notes`, validation `canDropEntryOnDirectory`, puis `store.moveEntry` (`SidebarNav.vue:5-15,111-139`);
- arbre : drag de note/dossier, validation de cible, feedback `is-drop-target`/`is-drop-disabled`, puis déplacement (`SidebarTreeEntry.vue:8-19,206-249`);
- library/cards : mêmes états sur les cartes et drop vers un dossier ou la racine (`LibraryGrid.vue:3-13,280-296`; `NoteCard.vue:13-20,328-362`).

Les tests acceptance exécutent réellement ces parcours et inspectent la sortie persistée : réordonnancement du rail (`ui-feature-regressions.spec.js:469-500`) et déplacement de `Projects/Beta.md` vers la racine (`:592-630`).

Dans Freya, `DropTarget::Root` et `set_root_drop_target` existent dans le contrat (`library_contract.rs:316-320,610-612`), mais aucun événement de drag/drop n’est attaché à `icon_rail`, `sidebar_entry`, `library_grid` ou `library_card` (`app/navigation.rs:52-283`; `app/library.rs:174-264`). Aucun chemin Freya actuel ne peut donc produire le feedback ou l’effet de déplacement prouvé par les tests Vue.

## 6. Menus

### Menu Create

Vue rend un vrai `role="menu"`, trois `role="menuitem"`, l’asset Excalidraw partagé et ferme le menu sur pointer extérieur ou `Escape` (`CreateEntryMenu.vue:15-38,68-102`). Les tests vérifient les trois entrées et l’asset SVG (`ui-feature-regressions.spec.js:32-59`).

Freya rend un rectangle absolu de `280×248`, trois rectangles de `64px` et des labels d’accessibilité sur les entrées (`app/library.rs:108-172`), mais pas de rôle de menu, pas d’asset Excalidraw et pas de fermeture extérieure/`Escape` dans ce module. Surtout, `CreateAction::Drawing` échoue explicitement avec le message « Drawing requires the Excalidraw web island » (`app.rs:164-168`). L’action n’est donc pas équivalente.

### Menu vault

Vue ouvre le menu au hover, liste les vaults, permet la sélection, le changement d’icône, l’ajout d’un vault et l’accès à la gestion (`IconRail.vue:57-157,386-425`). Freya ne rend dans `vault_switcher` que le vault courant et `Manage vaults` (`app/navigation.rs:134-179`). L’absence du sélecteur multi-vault, de l’icon picker et de `Add another vault` est observable.

## 7. Persistence et effets externes

### Vue

- largeur sidebar : lecture au montage et écriture dans `localStorage` sous `elephantnote:sidebarWidth` (`AppShell.vue:482-485,599-603`);
- pins : lecture/écriture par vault dans `localStorage`, et actions pin des cartes (`vaultStore.js:20-35,200-227`);
- visibilité sidebar : attach/detach via `elephantnoteClient.sidebar` et mise à jour du workspace (`vaultStore.js:236-293`);
- déplacement : appel de production `elephantnoteClient.entries.move`, réécriture des chemins ouverts/pinnés et rechargement des entrées (`vaultStore.js:730-760`).

### Freya

Le contrat déclare `SIDEBAR_WIDTH_STORAGE_KEY` (`navigation_contract.rs:23-26`), mais dans les fichiers de rendu et d’état audités, `SidebarWidth` est seulement lu depuis `ShellState` et modifié par aucune interaction de redimensionnement (`app.rs:27-49`; `app/navigation.rs:185-252`). Je ne trouve aucune lecture/écriture de cette clé dans le chemin Freya audité.

Freya prouve seulement les créations Note/Folder via `VaultAdapter` (`app.rs:154-163`) et le rechargement de la directory après succès (`:168-181`). Il n’y a pas de chemin correspondant pour pin, visibilité sidebar, renommage, suppression, réordonnancement du rail ou déplacement par drag/drop. Les tests Freya vérifient la création de `Untitled.md` et l’ouverture d’un dossier (`shell_freya_testing.rs:104-117`), mais pas le redémarrage ni la restauration de configuration.

## 8. Couverture acceptance actuelle

La couverture Vue/Electron vérifie déjà : ouverture du vault et de la note (`ui-parity-contract.spec.js:17-87`), géométrie et hit-targets (`library-shell-regressions.spec.js:79-142`), création et asset Excalidraw (`ui-feature-regressions.spec.js:32-83`), tri et grille/liste (`ui-feature-regressions.spec.js:85-188`), drag du rail, renommage, visibilité sidebar, drawer mobile et déplacement de fichier (`ui-feature-regressions.spec.js:469-630`).

La couverture Freya actuelle vérifie les labels, l’ouverture de Create, la création Note, l’ouverture de dossier et l’accessibilité de Settings/Search/Graph/editor (`freya_shell_acceptance.rs:39-61`; `shell_freya_testing.rs:96-155`). Elle ne couvre pas les sélecteurs structuraux Vue, les rectangles, les transitions, le hover, les menus extérieurs, le drag/drop ou la persistence après restart.

## Bloqueurs de validation différentielle

1. **Runner Tauri manquant dans l’acceptance Playwright actuelle** : le harness prouvé est Electron (`helpers.js:4,169-176`).
2. **Parcours commun non prouvé** : les tests Freya et Vue n’exécutent pas la même séquence d’actions avec les mêmes checkpoints.
3. **Géométrie non comparable** : différences explicites de topbar, composition de sidebar, toolbar et cartes ; aucun test Freya de rectangles.
4. **Mouvement non comparable** : Vue a hover, pointer gestures, drag et transitions ; Freya n’a que des actions `on_mouse_up` dans les surfaces auditées.
5. **Drag/drop non converti** : le contrat Freya possède des types, mais le rendu ne branche aucun événement ni effet de move.
6. **Menus non équivalents** : rôles/asset/fermeture et actions vault absents ou différents ; Drawing est explicitement en erreur côté Freya.
7. **Persistence non équivalente** : la largeur sidebar est persistée côté Vue mais pas reliée côté Freya ; plusieurs effets workspace/pin/move n’ont aucun chemin Freya.
8. **Niveau de preuve insuffisant** : les tests Freya actuels valident des labels/clics, pas une égalité visuelle fixe et temporelle.

Conclusion d’audit : toute déclaration « visuellement identique » ou « parcours équivalent » serait actuellement unsupported. Le statut reste **NOT PROVEN** jusqu’à ce que les mêmes parcours soient exécutés sur les deux runtimes avec états intermédiaires, captures géométriques et séquences de frames, puis que les effets persistés et les erreurs soient comparés.
