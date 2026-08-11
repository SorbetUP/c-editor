# Audit des bloqueurs de parité de l’éditeur

Statut de cet audit : `NOT PROVEN` — audit statique uniquement, sans
modification du code produit, sans test d’exécution et sans commit.

Branche inspectée : `nsb/freya-native-migration`.

## Verdict

Le chemin Freya actuel n’est pas une conversion de l’éditeur Vue/Tauri. Il
charge bien un fichier dans `muya-core` et sait sérialiser quelques mutations,
mais `editor_view.rs` ne monte pas de surface éditable et ne relie pas les
événements de saisie à `EditorDocument`.

Les faits bloquants sont précis :

- `EditorDocument` n’expose que `InsertText`, `Undo` et `Redo`
  (`Elephant/freya/src/editor.rs:19-25,133-163`).
- `editor_view.rs` ne branche que deux événements souris, pour Undo et Save;
  le document est transformé en `paragraph`, `label` et `rect` statiques
  (`Elephant/freya/src/app/editor_view.rs:46-123`). Aucun caret, hit-test,
  sélection, clavier, composition ou input text n’est relié au document.
- Le rendu Freya remplace explicitement certaines sémantiques par du texte
  d’erreur ou de substitution : images, HTML inline, math inline, front matter,
  footnotes, références et diagrammes (`Elephant/freya/src/app/editor_view.rs:255-279,371-396,458-498`).
- Le parcours Vue/Tauri réel est un éditeur `contenteditable` Muya/Rust avec
  plugins, contrôleur d’input, sélection DOM, événements de changement, store,
  autosave et restauration de session (`Elephant/frontend/src/muya/lib/index.js:27-82,124-149,580-620`,
  `Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditor.vue:1197-1490`).

Conclusion : aucune comparaison différentielle Freya/Tauri ne peut être
considérée comme valide avant de lever les bloqueurs ci-dessous. Capturer deux
écrans aujourd’hui mesurerait surtout une vue en lecture seule contre un
éditeur vivant.

## Provenance du parcours de référence

Le parcours Vue/Tauri de référence est composé de quatre couches qui doivent
rester présentes après conversion :

1. `NoteEditorHost.vue` possède l’identité de la note, le titre, les tags, le
   footer, le drag-and-drop, la synchronisation du document, la sauvegarde et le
   nettoyage (`Elephant/frontend/app/components/editor/NoteEditorHost.vue:1-50,201-249,619-831,897-948`).
2. `EditorWithTabs`/`runtimeEditor.vue` monte Muya, ses plugins et son moteur
   Rust, puis raccorde les événements d’édition, de sélection, de scroll, de
   clipboard, d’image, de recherche et de commandes (`Elephant/frontend/src/renderer/src/components/editorWithTabs/index.vue:1-19`,
   `Elephant/frontend/src/renderer/src/components/editorWithTabs/runtimeEditor.vue:89-127,1200-1260,1317-1425`).
3. Le contrôleur d’input intercepte `beforeinput`, clavier, copier/couper,
   coller, drop et composition et les traduit en commandes Rust
   (`Elephant/frontend/src/renderer/src/editor-rust/inputController/index.js:64-95,101-224`).
4. Le store transporte le Markdown, le curseur, `muyaIndexCursor`, l’historique,
   `scrollTop`, l’état sauvegardé et le TOC; il déclenche l’autosave et restaure
   ces champs après redémarrage (`Elephant/frontend/src/renderer/src/store/help.js:8-64`,
   `Elephant/frontend/src/renderer/src/store/editor.js:49-101,1153-1257`,
   `Elephant/frontend/src/renderer/src/store/bufferedState.js:17-50`).

Le runner d’acceptance confirme le comportement attendu sur le chemin réel :
ouverture, édition clavier, rendu des titres/code/tâches/images, sauvegarde
manuelle, exécution de code, sélection/citation, recherche et checkbox
(`build/scripts/run-desktop-acceptance.mjs:399-430,432-458,460-528,530-560`).
Les scénarios Playwright de parcours vérifient aussi l’édition/autosave et le
scroll visible (`tests/app/e2e/linux-usage-regressions.spec.js:118-132,250-275`,
`tests/app/e2e/ui-feature-regressions.spec.js:649-814`).

## Bloqueurs détaillés

| Domaine | Comportement Vue/Tauri observé | État Freya actuel | Plus petit chemin de conversion réutilisable |
| --- | --- | --- | --- |
| Chrome et identité | Le host affiche top bar, titre éditable, date, tags, pin, fermeture, surface Muya et footer optionnel (`NoteEditorHost.vue:1-50,201-239`). | Freya affiche seulement Undo, Save et une zone scrollable (`editor_view.rs:46-123`). Pas de titre, tags, pin, fermeture, compteurs ou footer. | Porter le contrat du host autour d’un `EditorController` Freya : identité/path/métadonnées hors du renderer de blocs, puis réutiliser la même logique Markdown de titre/tags (`NoteEditorHost.vue:833-885`, `noteDocument.js:22-29`). |
| Éditabilité | Muya crée un conteneur `contenteditable` (`muya/lib/index.js:597-620`) et le runtime Rust DOM rend des nœuds éditables. | `paragraph().spans_iter(...)`, `label()` et `rect()` ne sont jamais éditables; le seul accès mutable est `State<ShellState>` dans les boutons (`editor_view.rs:52-95`). | Ajouter une vraie surface Freya d’édition inline/bloc. Chaque texte doit conserver son `NodeId`; les événements texte doivent appeler `muya_core::EditorSession` puis appliquer les `ViewPatch`, au lieu de reparser ou d’écrire une façade séparée. |
| Saisie texte | `beforeinput` traduit insertion, saut de ligne, suppression, undo/redo et formats vers le moteur Rust (`realMuyaRustAdapter.js:247-301`, `inputController/index.js:101-135`). | `EditorAction::InsertText` existe mais aucun événement Freya ne l’appelle (`editor.rs:19-25,133-155`). Sa sélection initiale est créée à l’offset 0 par `muya-core` (`crates/muya-core/src/session/types.rs:66-80`); il n’y a pas de caret utilisateur. | Étendre l’adaptateur à `SessionCommand`/`ProtocolCommand`, dont la liste couvre insertion, paragraphe, suppression, paste, formats, blocs, listes, tâches, tableaux, images, composition, undo/redo (`crates/muya-core/src/protocol/types.rs:16-84`). Brancher input texte et composition Freya à cette API. |
| Sélection et caret | La sélection DOM est lue, convertie en curseur Muya, publiée au store et utilisée pour le menu de formats/citation (`runtimeEditor.vue:1393-1420`; `ui-feature-regressions.spec.js:679-698`). | `EditorDocument::set_selection` accepte une sélection logique, mais aucun hit-test texte → `NodeId`/offset UTF-16, dessin de sélection ou mise à jour de sélection n’existe (`editor.rs:124-131,220-223`; `editor_view.rs:341-368`). | Réutiliser le modèle `Selection`/offset UTF-16 de `muya-core` (`crates/muya-core/src/selection/mod.rs:5-43`) et implémenter uniquement le mapping Freya des coordonnées de texte vers `Selection`. Publier ensuite un snapshot de sélection pour les commandes, formats et extensions. |
| Raccourcis clavier | Le contrôleur intercepte Enter, Tab/Shift+Tab, navigation, suppression, composition et le runtime écoute les commandes bus Undo/Redo/SelectAll/format/recherche (`inputController/index.js:185-224`; `runtimeEditor.vue:1317-1347`). Le runner vérifie aussi Cmd/Ctrl+F (`run-desktop-acceptance.mjs:530-536`). | Aucun handler clavier dans `editor_view.rs`; Undo est uniquement un bouton souris et Redo n’est même pas rendu (`editor_view.rs:52-95`). | Créer une table courte key → `ProtocolCommand`/`SessionCommand` et la brancher aux événements Freya. Ne pas recréer les règles de document : les déléguer à `muya-core` et, pour les cas complexes, reprendre les décisions de `realMuyaRustAdapter.js:315-333,522-605`. |
| Suppression, paragraphes et blocs | Muya/Rust gère Backspace/Delete, Enter, duplicate/delete paragraph, block quote, code block, heading, listes et horizontal rule (`runtimeEditor.vue:1045-1159`; `protocol/dispatch.rs:34-46,49-122`). | Aucun de ces gestes n’est possible depuis la vue. `EditorDocument` ne sait pas les dispatcher (`editor.rs:133-163`). | Ajouter des variantes typées à l’adaptateur et appeler les commandes existantes (`Command`, `BlockCommand`, `BlockTypeCommand`, `ListCommand`, `HorizontalRule`) via `SessionCommand` (`crates/muya-core/src/session/types.rs:14-37`, `protocol/dispatch.rs:49-96`). |
| Formats inline | La sélection déclenche les formats et le moteur traite strong/emphasis/strike; le store reçoit les formats (`runtimeEditor.vue:1418-1420`, `realMuyaRustAdapter.js:530-535`). | Le rendu sait seulement afficher bold/italic/strike/code par `Span`; aucun clic ou raccourci ne modifie le document (`editor_view.rs:506-535`). | Utiliser `SessionCommand::Mark`/`MarkCommand` via le protocole (`protocol/dispatch.rs:57-60`) et rerendre les patches. Garder les styles actuels uniquement comme renderer de l’état canonique. |
| Clipboard, paste, drop et IME | Muya gère copier/couper/coller HTML/texte, drop de fichiers/images et composition (`inputController/index.js:143-183`; `realMuyaRustAdapter.js:351-420`). Le host gère aussi le drop d’une entrée de vault et l’insère au curseur (`NoteEditorHost.vue:779-831`). | Aucun handler clipboard/drop/composition. Les images sont converties en texte `[Image non rendue...]` (`editor_view.rs:458-463`). | Réutiliser le protocole `PasteMarkdown`, `InsertImage` et les commandes de composition (`protocol/types.rs:24-25,49-58,76-81`), puis porter le seul adaptateur de fichiers vers le service Tauri/vault existant. Conserver `droppedEntryMarkdown` pour les liens d’entrée. |
| Tâches | Muya rend une vraie checkbox et son clic modifie le Markdown; le runner vérifie `[x]` sur disque (`run-desktop-acceptance.mjs:550-560`). | Freya rend `☑`/`☐` comme un caractère dans le marqueur de liste (`editor_view.rs:312-329`); ce n’est pas un contrôle et ne peut pas être coché. | Rendre un contrôle Freya associé au `NodeId` du `ListItem`, dispatcher `SetTaskChecked` (`protocol/types.rs:65-68`) et publier le changement au host/autosave. |
| Images | Muya rend l’image, permet preview, toolbar, remplacement/suppression/resize et ouvre Excalidraw avec Ctrl/Cmd-clic (`runtimeEditor.vue:472-493,567-574,1383-1391`; `runtimeEditor.vue:1370-1380`). | Freya affiche une chaîne rouge avec source/alt et ne charge aucune image (`editor_view.rs:458-463`). | Réutiliser les commandes `InsertImage`/`ImageCommand` et le service d’assets; construire ensuite un composant Freya image minimal avec clic/preview. Ne pas masquer l’absence du service par un placeholder déclaré comme rendu final. |
| Tables | Les plugins `TablePicker` et `TableBarTools` sont montés; les opérations rows/columns et navigation de cellules atteignent Rust (`runtimeEditor.vue:1200-1219`, `realMuyaRustAdapter.js:422-470`). | La table est une suite de rectangles/labels statiques; aucune cellule n’est sélectionnable ni modifiable (`editor_view.rs:213-253`). | Utiliser `CreateTable`, `TableCommand` et `TableNavigationCommand` (`protocol/types.rs:45-48,70-75`) avec un état de cellule Freya. Commencer par navigation/édition de texte, puis toolbar. |
| Footnotes, liens, HTML, math, diagrammes | Muya possède plugins/outils de footnote, liens, rendu de code/diagrammes et actions de clic (`runtimeEditor.vue:1200-1219,1370-1391`; `muya/lib/index.js:245-253`). | Footnote/link/HTML/math sont aplatis en texte; les blocs front matter, footnote definition, reference definition et diagram sont marqués « non rendu » (`editor_view.rs:255-279,453-498`). Un lien n’est qu’une couleur (`editor_view.rs:453-457,506-535`). | Préserver toutes les natures de nœuds dans `muya-core`, puis ajouter des renderers Freya par type. Pour les actions, réutiliser les commandes/protocoles Rust existants; pour les extensions JS/Excalidraw, exposer un host API explicite au lieu d’inventer un comportement natif incomplet. |
| Rendu et sémantique visuelle | Le runner exige des titres structurés, `pre`, checkbox et image sémantiques, et interdit que le Markdown brut apparaisse (`run-desktop-acceptance.mjs:449-458`). | Le renderer utilise des choix de tailles/couleurs locaux et affiche des messages d’erreur dans le document pour les nœuds non pris en charge (`editor_view.rs:151-160,371-396,538-547`). Cela produit une apparence et une sémantique différentes même sans interaction. | Utiliser `Document`/`ViewPatch` comme modèle unique, ajouter les types manquants, puis calibrer fonts, largeur, marges, line-height et états actifs à partir des captures Playwright/Freya. Une capture statique ne suffit pas tant que les transitions d’état ne sont pas identiques. |
| Propagation des changements | `dispatchMuyaChange` envoie Markdown/cursor/history/TOC au store; le store met à jour la tab, `isSaved`, index et autosave (`runtimeEditorChanges.js:1-17`; `editor.js:1153-1222`). | Une mutation via `EditorDocument` reste dans la struct; la vue ne convertit pas `EditorUpdate` en état `ShellState`, ne met pas à jour l’index et ne publie aucun événement de changement (`editor.rs:57-91`; `editor_view.rs:46-123`). | Faire de `EditorController` la frontière unique : `SessionUpdate` → état Freya (Markdown, selection, dirty, patches) → service d’écriture/index. Ne pas recopier le store en cache séparé. |
| Save explicite | Le parcours réel utilise la commande `file.save`, confirme `isSaved` et vérifie le fichier sur disque (`run-desktop-acceptance.mjs:460-490`). | Le bouton Save appelle directement `fs::write` (`editor_view.rs:74-95`; `editor.rs:233-236`), sans état `isSaved`, sans `changed`, sans notification `tab-saved`, sans refresh de la bibliothèque et sans contrat Tauri `tauri_notes_write` (`Elephant/backend/tauri/src/core_commands.rs:261-279`). | Faire passer l’écriture par l’adaptateur de notes/vault de production, conserver le résultat `changed/updatedAt`, puis mettre à jour dirty/entrée visible. Garder `EditorDocument` centré sur session + sérialisation; déplacer l’effet disque dans un host de persistance. |
| Autosave et fermeture | `LISTEN_FOR_CONTENT_CHANGE` planifie l’autosave; `NoteEditorHost` a une file, un fallback `notes.write` → `fileUtils.writeFile`, un flush à la fermeture et au démontage (`editor.js:1205-1257`; `NoteEditorHost.vue:567-616,619-699,938-948`). | Aucun autosave, aucune file de sauvegarde, aucun flush de fermeture/démontage. `EditorDocument::save` peut échouer mais aucune mutation ne le déclenche (`editor.rs:233-236`). | Porter la politique de sauvegarde comme service réutilisable, avec debounce, séquencement in-flight, flush et fallback explicite. Le controller doit émettre `dirty` à chaque `SessionUpdate`. |
| Scroll et mouvement | Muya publie `scrollTop`, le store le mémorise par tab et `handleFileChange` le restaure; le host compacte la top bar au-delà de 24 px; le mode typewriter anime le curseur (`muya/lib/index.js:69-82`; `runtimeEditor.vue:354-365,1393-1412`; `editor.js:126-132,253-262`; `NoteEditorHost.vue:168-172,927-948`). Les tests vérifient le changement après défilement (`ui-feature-regressions.spec.js:774-814`). | `scrollable(true)` fournit au mieux un scroll générique (`editor_view.rs:112-120`), sans valeur persistée, événement, restauration, compactage de top bar, typewriter ou auto-scroll du caret. | Exposer `scrollTop` et les événements de scroll depuis le composant Freya, les stocker avec l’état de note, restaurer après montage, puis reproduire les animations avec la clock Freya. Ce point est indispensable à la comparaison « en mouvement ». |
| Restart | Le store sérialise buffered state dans le runtime (`bufferedState.js:17-50`, `runtimeBridge.js:789-791`) et restaure tabs, Markdown, curseur, historique et layout (`editor.js:49-101`). | `ShellState::load_from_root` initialise toujours `editor: None` (`Elephant/freya/src/app.rs:40-79`); `EditorDocument::load` relit seulement le fichier et recrée une session vierge (`editor.rs:181-187`). L’état non sauvegardé, le caret, l’historique et le scroll sont perdus. | Définir un snapshot de session versionné contenant path, Markdown, `Selection`, dirty, historique sérialisable et scroll; l’enregistrer avec le host existant et le restaurer avant le premier rendu. Vérifier ensuite le vrai redémarrage du processus, pas seulement un nouveau render Freya. |
| Erreurs et crash runtime | Le runtime Rust expose une erreur visible `role="alert"`, journalise revision/sélection/longueur et signale les erreurs de montage/patch (`RustMuyaRuntimeEditor.vue:12-17,83-105`; `runtimeEditor.vue:311-327`). Les acceptance checks refusent `.muya-rust-runtime-error` et les erreurs renderer (`linux-usage-regressions.spec.js:80-84`). | Les erreurs Undo/Save vont dans `ShellState.error` et sont affichées comme « Library error »; les nœuds absents deviennent `[Muya node unavailable]` et les types non supportés deviennent du contenu rouge dans le document (`editor_view.rs:142-145,371-396`; `app.rs:211-245`). Il n’y a pas de rôle alert, de correlation id, de phase runtime, ni de capture de patch/selection. | Introduire un état d’erreur éditeur distinct (montage, input, patch, save, restore), visible et loggé avec action/path/revision/selection. Propager les `EditError` du protocole sans les convertir en texte dans le document. |
| Validation réelle | Le Playwright local contrôle le chemin Electron via `_electron` (`tests/app/e2e/helpers.js:4,169-207`); le runner Tauri actuel parle à un serveur HTTP DOM/commande et capture peu de visuel (`build/scripts/run-desktop-acceptance.mjs:43-129`). | Les tests Freya actuels valident surtout l’arbre d’accessibilité et quelques mutations de session; ils ne prouvent pas le parcours éditeur natif complet. | Après conversion, garder un scénario commun : mêmes fixture, actions, checkpoints d’état, captures et chronologie. Utiliser `TestingRunner::move_cursor/press_cursor/write_text/scroll/poll/render` côté Freya et les actions/captures Playwright côté Tauri, puis comparer chaque état intermédiaire et chaque frame. Tant que l’éditeur Freya n’est pas interactif, aucune parité visuelle ou temporelle n’est démontrable. |

## Chemin de conversion minimal recommandé

L’ordre minimal qui évite une réécriture depuis zéro est :

1. **Faire de `muya-core` le moteur commun réellement piloté.** Remplacer
   `EditorAction` par une frontière couvrant `SessionCommand` ou
   `EditorRequest`/`ProtocolCommand`; garder les révisions, sélections, patches,
   composition et history déjà fournis par `muya-core`
   (`Elephant/crates/muya-core/src/session`, `.../protocol`, `.../view`).
2. **Convertir la surface Muya, pas seulement son rendu.** Porter d’abord le
   texte éditable, caret, sélection, Enter/Backspace, Undo/Redo et formats; les
   coordonnées Freya doivent produire des `SelectionPoint` UTF-16 et les
   changements doivent appliquer les `ViewPatch` retournés. Le renderer actuel
   peut être conservé comme base de styles seulement.
3. **Raccorder le controller au host de note.** Reproduire la frontière
   `dispatchMuyaChange` → store/dirty/index → service de save, avec titre/tags,
   assets, fermeture et erreurs. Ne pas laisser `fs::write` dans l’événement UI
   comme chemin de production.
4. **Ajouter les sémantiques interactives par priorité d’acceptance :** tâches,
   images, tables, liens/citations, clipboard/drop, footnotes et extensions.
   Chaque ajout doit utiliser les commandes Rust existantes avant d’ajouter un
   nouveau contrat.
5. **Porter scroll/restart et seulement ensuite comparer le mouvement.** La
   comparaison doit capturer les états avant/après input, pendant scroll/animation,
   après autosave, après fermeture et après redémarrage. Une capture finale ne
   prouve pas ces transitions.

## Preuve manquante avant de pouvoir lever le blocage

Il faut au minimum, sur le commit final :

- une saisie réelle dans Freya qui modifie le Markdown au caret sélectionné;
- Enter, suppression, formats, Undo/Redo et un raccourci Cmd/Ctrl vérifiés dans
  la même fixture que Tauri;
- une sélection visible et exploitable, incluant la citation et la checkbox;
- une sauvegarde automatique et explicite vérifiée sur disque, puis un flush à
  la fermeture;
- scroll, restauration de scroll/caret et redémarrage vérifiés;
- un chemin d’erreur visible et loggé pour input, patch, save et restart;
- rendu sémantique des headings, code, images, tâches et tables sans texte
  placeholder;
- captures Freya `render()`/`poll()` et captures Playwright au même timeline,
  avec diff statique **et** temporel inspecté;
- le runner réel adapté au runtime effectivement testé. Le simple passage des
  tests unitaires ou d’un arbre d’accessibilité ne suffit pas.

À ce stade, l’état honnête est donc `NOT PROVEN`, et non « paritaire ».
