# Séparation Freya : Draw et Note

## Statut des dépôts externes

Les dépôts privés sont maintenant accessibles et ont été clonés dans :

- Elephant/feature-incoming/Draw
- Elephant/feature-incoming/Note

Ils contenaient seulement un README et aucun Cargo.toml. Ils ont donc été
initialisés comme crates Rust séparables avec tests de domaine, sans prétendre
réutiliser du code qui n’existait pas encore dans ces dépôts.

## Architecture cible

~~~
Elephant/
  freya/                 shell, vault, sidebar, settings, routing
packages/
  elephant-draw/         domaine de dessin et format Excalidraw
  elephant-note/         domaine Markdown et moteur Muya
~~~

Le shell dépend des deux packages. Les packages ne dépendent ni de ShellState,
ni de VaultAdapter, ni de Freya.

## Package elephant-draw

Modules recommandés :

- document.rs : document Excalidraw sérialisable ;
- element.rs : éléments rectangle, ellipse, diamond, line, arrow, freedraw,
  text, image ;
- viewport.rs : zoom, pan, coordonnées scène ;
- tools.rs : outil actif et raccourcis ;
- selection.rs : hit-testing, sélection, déplacement, poignées ;
- history.rs : undo/redo ;
- storage.rs : .excalidraw, .json, sidecars et sécurité des chemins ;
- export.rs : JSON, PNG et éventuellement SVG ;
- lib.rs : API publique stable.

API minimale :

~~~
pub struct DrawDocument { /* elements, files, app_state */ }
pub struct DrawEditorState { /* document, viewport, selection, history */ }
pub trait DrawStorage {
    fn load_scene(&self, path: &Path) -> Result<DrawDocument, DrawError>;
    fn save_scene(&self, path: &Path, doc: &DrawDocument) -> Result<(), DrawError>;
}
~~~

Tests obligatoires :

- round-trip JSON Excalidraw réel ;
- conservation des propriétés inconnues ;
- création de toutes les primitives ;
- changement d’outil sans réutilisation du geste précédent ;
- sélection, déplacement et effacement ;
- zoom, pan, undo/redo ;
- import .excalidraw et .json ;
- fichiers image liés ;
- chemins hors vault refusés ;
- export JSON et PNG.

## Package elephant-note

Modules recommandés :

- document.rs : document Muya ;
- markdown.rs : parse et sérialisation ;
- selection.rs : curseur et sélection ;
- commands.rs : bold, italic, strike, code, listes, quote, liens et tâches ;
- history.rs : undo/redo ;
- persistence.rs : autosave, rename et conflits externes ;
- lib.rs : API publique stable.

Tests obligatoires :

- Markdown vers Muya puis retour Markdown ;
- titres, paragraphes, listes, tâches, quotes, code et liens ;
- sélection clavier ;
- bold, italic, strike et inline code ;
- undo/redo ;
- toggle task persistant ;
- tags et date ;
- renommage de titre séparé du renommage de fichier ;
- autosave ;
- conflit d’édition externe ;
- fermeture avec erreur de sauvegarde ;
- reprise après redémarrage.

## Adaptateurs Freya

Le shell monte :

~~~
DrawView::new(draw_state, draw_storage)
NoteView::new(note_state, note_persistence)
~~~

Les vues traduisent uniquement pointer, clavier, focus, scroll, toolbar,
commandes, erreurs et sauvegarde. Elles ne recréent pas la logique de document.

Adaptateurs Elephant :

- ElephantDrawStorage implémente DrawStorage ;
- ElephantNotePersistence implémente NotePersistence ;
- le shell fournit le chemin de vault, les logs et les commandes de fenêtre.

## Référence de l’intégration Tauri

Elephant/frontend/app/services/excalidraw.js résout scènes, previews et
sidecars.

Elephant/frontend/src/renderer/src/addons/builtin/excalidraw.js déclare les
commandes, l’insertion rapide, la toolbar d’image et la copie des compagnons.

Elephant/frontend/src/renderer/src/addons/builtin/ui/ExcalidrawEditorOverlay.vue
orchestre ouverture, fermeture, sauvegarde, preview, note Markdown et logs.

Elephant/frontend/app/components/editor/ExcalidrawDialog.vue définit la fenêtre,
le dialog de nom et les boutons.

Pour Note, la référence est Muya et Elephant/crates/muya-core.

## Migration

1. Figer les contrats JSON et Markdown.
2. Déplacer le domaine Drawing dans elephant-draw.
3. Déplacer le domaine Muya dans elephant-note.
4. Ajouter les tests de package sans fenêtre.
5. Ajouter les adaptateurs Freya.
6. Remplacer les routes Freya actuelles.
7. Valider les parcours réels avec Computer Use.
8. Supprimer l’ancien canvas seulement après preuve de parité.

## Critères de fin

La migration n’est terminée que lorsque les packages Draw et Note réels sont
accessibles, compilent en Rust, sont utilisés par Freya, disposent de tests
isolés, et que les parcours suivants sont prouvés dans l’application :

- ouvrir, modifier et sauvegarder une note Markdown ;
- ouvrir, modifier et sauvegarder une scène Excalidraw ;
- fermer sans perte ;
- redémarrer et recharger ;
- importer/exporter ;
- gérer les erreurs de stockage.
