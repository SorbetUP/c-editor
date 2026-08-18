# Référence Excalidraw : interface, interactions et contrat d’intégration

Ce document décrit le comportement à reproduire dans une implémentation
native Rust/Freya. La référence fonctionnelle est le code officiel Excalidraw
et la référence Elephant est :

- Elephant/frontend/app/services/excalidraw.js
- Elephant/frontend/src/renderer/src/addons/builtin/excalidraw.js
- Elephant/frontend/src/renderer/src/addons/builtin/ui/ExcalidrawEditorOverlay.vue
- Elephant/frontend/app/components/editor/ExcalidrawDialog.vue

Source officielle : https://github.com/excalidraw/excalidraw

## 1. Modèle mental

Excalidraw n’est pas un simple panneau de boutons qui dessine des rectangles.
C’est un éditeur vectoriel à canvas infini composé de quatre couches :

1. document sérialisé : éléments, fichiers binaires, appState et métadonnées ;
2. moteur de scène : ordre, sélection, hit-testing, bindings et mutations ;
3. renderer : formes, texte, images, flèches, poignées et sélections ;
4. couche UI : toolbar, propriétés, bibliothèque, menus, dialogs, footer,
   zoom et états d’aide.

La couche UI doit piloter l’état du moteur. Elle ne doit jamais contenir une
deuxième représentation cachée de l’outil actif ou de la scène.

## 2. Composition de l’interface

La composition desktop est organisée autour d’un canvas plein écran et de
surfaces flottantes :

- bouton menu en haut à gauche ;
- toolbar horizontale en haut au centre ;
- contrôles d’état en haut à droite ;
- panneau de propriétés flottant à gauche ;
- bibliothèque à droite ;
- zoom et undo/redo en bas à gauche ;
- aide en bas à droite ;
- dialogs centrés avec scrim.

Le code LayerUI.tsx monte ces surfaces par zones et ne remplace pas le canvas
quand un panneau s’ouvre.

## 3. Toolbar principale

Ordre desktop :

1. verrouillage ;
2. main/pan ;
3. sélection ;
4. rectangle ;
5. losange ;
6. ellipse ;
7. flèche ;
8. ligne ;
9. dessin libre ;
10. texte ;
11. image ;
12. gomme ;
13. actions de forme.

Chaque bouton expose une icône vectorielle, un nom accessible, un raccourci,
un état actif, un tooltip et une action qui modifie appState.activeTool.

Le canvas doit lire l’outil actif au moment du pointer-down. Il ne faut pas
conserver un outil capturé lors d’un ancien render.

## 4. Gestes

### Sélection

- pointer-down : hit-test inverse des éléments ;
- élément trouvé : sélection puis déplacement ;
- aucun élément : pan si autorisé ;
- drag : mutation de x/y ou des points ;
- release : commit historique ;
- clic vide : désélection ;
- double-clic : édition ou création de texte.

### Main

Pointer-down + drag déplace la caméra sans créer d’élément. La molette et la
barre espace peuvent également contrôler le pan.

### Rectangle, ellipse et losange

Pointer-down crée immédiatement un élément. Le drag met à jour sa bounding box.
Les dimensions sont normalisées, les styles viennent du style courant et
l’élément reste sélectionné après release.

### Ligne et flèche

Les points sont relatifs au point initial. Une flèche porte endArrowhead. Le
déplacement ne doit jamais convertir une ligne en rectangle.

### Dessin libre

Un élément freedraw commence avec [0, 0]. Les points suivants sont relatifs au
point initial et sont ajoutés à chaque mouvement. Le renderer doit rester
continu si le pointeur traverse un enfant SVG.

### Texte

Le texte est une donnée d’élément éditable. Taille, famille, alignement et
style viennent du panneau de propriétés. Escape annule et la modification est
persistée.

### Image

Un sélecteur ajoute une entrée dans files et un élément image avec position et
dimensions. Le fichier doit survivre au rechargement.

### Gomme

Le hit-test cible l’élément sous le pointeur. Il est marqué supprimé ou retiré
par une opération d’historique réversible.

## 5. Panneau de propriétés

Lorsqu’un élément est sélectionné, le panneau rend selon son type :

- trait, fond, largeur, style et opacité ;
- remplissage et ordre de calque ;
- angle et poignées ;
- taille, famille, alignement et style de texte ;
- pointes de lignes et flèches.

Le panneau est flottant, scrollable, au-dessus du canvas et ne réduit pas la
zone de dessin.

## 6. Menus et dialogs

Le menu principal comprend charger, sauvegarder, exporter, export image,
recherche, aide, reset, liens Excalidraw, thème et fond.

Le dialog Elephant de nom contient un scrim, Name your drawing, une
description, un champ, Cancel, Save, Escape, Enter et le refus d’un nom vide.

## 7. Viewport

Le viewport contient zoom, pan, taille, centre et état de défilement. Les
coordonnées scène sont indépendantes des coordonnées écran. Freya doit
convertir les événements dans l’espace scène avant mutation.

## 8. Persistance Elephant

- scène : *.excalidraw ;
- preview : *.png ;
- fichiers : files et sidecars .assets ;
- note Markdown liée par image .assets ;
- MIME : application/vnd.excalidraw+json.

Le service Tauri normalise le chemin, résout la scène, charge le JSON, monte
Excalidraw, sauvegarde, produit le preview, met à jour le Markdown et copie les
sidecars lors d’un déplacement. Freya doit reprendre ce contrat.

## 9. Limites de l’ancien canvas Freya

L’ancien canvas était une approximation : pas de propriétés complètes, texte
non éditable, images absentes, pas de bibliothèque/export fiable, historique
incomplet, hit-testing partiel et événements pointer désynchronisés. Il ne doit
plus être étendu par petites corrections indépendantes.

## 10. Contrat du futur package Draw

~~~rust
pub struct DrawDocument {
    pub elements: Vec<DrawElement>,
    pub files: DrawFiles,
    pub app_state: DrawAppState,
}

pub struct DrawEditorState {
    pub document: DrawDocument,
    pub viewport: Viewport,
    pub active_tool: ToolType,
    pub selection: SelectionState,
    pub history: HistoryState,
}

pub trait DrawStorage {
    fn load_scene(&self, path: &Path) -> Result<DrawDocument, DrawError>;
    fn save_scene(&self, path: &Path, document: &DrawDocument) -> Result<(), DrawError>;
    fn export_png(&self, document: &DrawDocument, path: &Path) -> Result<(), DrawError>;
}
~~~

Freya ne fournit que l’adaptateur UI et les événements. Le domaine doit être
testable sans fenêtre.
