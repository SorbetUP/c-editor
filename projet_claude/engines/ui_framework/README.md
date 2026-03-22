# UIFramework Documentation

## Vue d'ensemble

Le UIFramework est un système d'interface utilisateur modulaire pour ElephantNotes V3, offrant une barre latérale interactive et un éditeur Markdown intégré avec rendu en temps réel.

## Architecture

### 1. MarkdownEditor
**Fichier**: `ui_framework.m` (lignes 50-344)

Éditeur de texte spécialisé héritant de NSTextView avec rendu Markdown en temps réel.

#### Fonctionnalités principales:
- **Rendu automatique**: Timer de 0.1s pour détecter les changements
- **Librairie C intégrée**: Utilise `editor_parse_markdown()` pour un parsing optimisé
- **Formatage visuel**: Headers, bold, italic avec couleurs personnalisées
- **Gestion d'erreur**: Fallback gracieux si le parsing C échoue

#### Méthodes publiques:
```objc
- (void)setRenderingEnabled:(BOOL)enabled;  // Contrôle du rendu automatique
- (void)forceRender;                        // Force un rendu immédiat
```

#### Méthodes de setup:
```objc
- (void)setupEditor;              // Configuration visuelle de base
- (void)setupMarkdownRendering;   // Initialisation du timer de rendu
- (void)initializeCLibrary;       // Initialisation de la librairie C
```

#### Méthodes de rendu:
```objc
- (void)render;                                    // Rendu principal avec cache intelligent
- (NSMutableAttributedString *)renderUsingCLibrary:;        // Intégration librairie C optimisée
- (NSMutableAttributedString *)createFormattedTextFromJSON:originalText:;  // Parser JSON → NSAttributedString
- (NSMutableAttributedString *)createSimpleFormattedText:; // Fallback simple
- (void)applyFormattedTextWithSelectionPreservation:;      // Application avec préservation curseur
- (void)handleRenderingException:;                 // Gestion d'erreur robuste
```

#### Méthodes de formatage avancées:
```objc
- (void)formatAsHeader:range:level:;               // Formatage headers (H1-H6)
- (void)formatAsBold:range:;                       // Formatage gras
- (void)formatAsItalic:range:;                     // Formatage italique
- (void)applyEnhancedStyleForType:range:text:;     // Styles étendus (nouveau)
- (void)formatAsCode:range:;                       // Code inline et blocs
- (void)formatAsLink:range:;                       // Liens hypertexte
- (void)formatAsStrikethrough:range:;              // Texte barré
- (void)formatAsBlockquote:range:;                 // Citations
- (void)formatAsList:range:;                       // Listes à puces/numérotées
```

### 2. UIIconButton
**Fichier**: `ui_framework.m` (lignes 346-400)

Boutons d'icônes pour la barre latérale avec gestion d'états visuels.

#### États supportés:
- `UI_STATE_NORMAL`: État par défaut
- `UI_STATE_HOVER`: Survol souris
- `UI_STATE_ACTIVE`: Icône activée
- `UI_STATE_DISABLED`: Désactivé

#### Propriétés:
```objc
@property UIIconType iconType;      // Type d'icône (SEARCH, HOME, etc.)
@property UIFramework* framework;   // Référence au framework parent
```

### 3. API C du Framework
**Fichier**: `ui_framework.m` (lignes 402+)

Interface C pour la gestion globale du framework.

#### Fonctions principales:

##### Création/Destruction:
```c
UIFramework* ui_framework_create(NSWindow* window);
void ui_framework_destroy(UIFramework* framework);
bool ui_framework_setup_layout(UIFramework* framework);
```

##### Gestion des icônes:
```c
void ui_framework_add_icon(UIFramework* framework, UIIconType iconType);
void ui_framework_set_icon_active(UIFramework* framework, UIIconType iconType);
void ui_framework_set_icon_enabled(UIFramework* framework, UIIconType iconType, bool enabled);
```

##### Gestion du contenu:
```c
void ui_framework_set_editor_content(UIFramework* framework, NSString* content);
NSString* ui_framework_get_editor_content(UIFramework* framework);
void ui_framework_clear_editor(UIFramework* framework);
```

##### Callbacks:
```c
void ui_framework_set_click_handler(UIFramework* framework, UIIconClickHandler handler, void* userData);
void ui_framework_set_hover_handler(UIFramework* framework, UIIconHoverHandler handler, void* userData);
```

## Configuration

### Constantes personnalisables:
```objc
static const NSTimeInterval UI_MARKDOWN_UPDATE_INTERVAL = 0.1;  // Fréquence rendu
static const CGFloat UI_DEFAULT_EDITOR_FONT_SIZE = 14.0;        // Taille police
```

### Couleurs par défaut:
- **Texte éditeur**: RGB(0.878, 0.878, 0.878) - Gris clair
- **Fond éditeur**: RGB(0.118, 0.118, 0.118) - Gris foncé
- **Headers**: RGB(0.2, 0.4, 1.0) - Bleu
- **Bold**: RGB(0.0, 0.8, 0.0) - Vert
- **Italic**: RGB(1.0, 1.0, 0.0) - Jaune

### Couleurs étendues V3.0 (nouvelles):
- **Code**: RGB(1.0, 0.6, 0.0) - Orange avec police monospace
- **Liens**: RGB(0.3, 0.7, 1.0) - Bleu ciel avec soulignement
- **Strikethrough**: Couleur grisée avec barré
- **Blockquotes**: RGB(0.7, 0.7, 0.7) - Gris avec indent
- **Listes**: Couleur normale avec formatage spécial

## Intégration avec la librairie C

### Initialisation:
```c
// Appelée automatiquement dans initializeCLibrary
EditorResult result = editor_library_init();
```

### Stratégies de Parsing Multiples:

#### 1. Parsing Complet avec Structure JSON (Méthode principale):
```c
// Dans renderUsingCLibrary: - API principale recommandée
EditorResult result = editor_parse_markdown(markdown_cstr, &json_result);
```

#### 2. Parsing Simple (Fallback rapide):
```c
// Fallback si le parsing complet échoue
const char *simple_result = editor_parse_markdown_simple(markdown_cstr);
```

#### 3. Conversion HTML Directe (Debug/Prévisualisation):
```c
// Pour debug ou prévisualisation HTML
const char *html_result = editor_markdown_to_html(markdown_cstr);
```

### Cache Intelligent:
```objc
// Évite re-parser le même contenu
static NSString *lastRenderedText = nil;
static NSMutableAttributedString *cachedResult = nil;

if (lastRenderedText && [lastRenderedText isEqualToString:currentText] && cachedResult) {
    [self applyFormattedTextWithSelectionPreservation:cachedResult];
    return;
}
```

### Libération mémoire:
```c
// Toujours libérer les résultats
editor_free_string(json_result);
```

### Gestion d'erreur robuste:
```objc
// Stratégie de fallback en cascade
NSMutableAttributedString *formattedText = [self renderUsingCLibrary:currentText];
if (formattedText) {
    // Succès avec librairie C
} else {
    // Fallback vers rendu simple
    NSMutableAttributedString *fallback = [self createSimpleFormattedText:currentText];
}
```

## Extensibilité

### Ajouter un nouveau type de formatage:

1. **Étendre applyEnhancedStyleForType** (méthode V3.0 recommandée):
```objc
} else if ([type isEqualToString:@"code"]) {
    [self formatAsCode:text range:range];
}
```

2. **Implémenter la méthode de formatage**:
```objc
- (void)formatAsCode:(NSMutableAttributedString *)text range:(NSRange)range {
    [text addAttributes:@{
        NSForegroundColorAttributeName: [NSColor orangeColor],
        NSFontAttributeName: [NSFont monospacedSystemFontOfSize:UI_DEFAULT_EDITOR_FONT_SIZE]
    } range:range];
}
```

### Ajouter une nouvelle icône:

1. **Étendre l'enum** dans `ui_framework.h`:
```c
typedef enum {
    UI_ICON_SEARCH = 0,
    UI_ICON_FOLDER = 1,
    UI_ICON_BACK = 2,
    UI_ICON_HOME = 3,
    UI_ICON_SETTINGS = 4,
    UI_ICON_NEW_FEATURE = 5  // Nouvelle icône
} UIIconType;
```

2. **Ajouter le dessin** dans `ui_framework_create_icon_image`:
```c
case UI_ICON_NEW_FEATURE:
    ui_draw_new_feature_icon(size);
    break;
```

3. **Implémenter la fonction de dessin**:
```c
static void ui_draw_new_feature_icon(CGFloat size) {
    // Code de dessin personnalisé
}
```

## Patterns de Développement

### Séparation des responsabilités:
- **MarkdownEditor**: Rendu et édition de texte
- **UIIconButton**: Interactions barre latérale
- **API C**: Orchestration globale

### Gestion d'erreur:
- **Try-catch**: Protection contre les crashes NSTextView
- **Validation ranges**: Éviter les accès hors limites
- **Fallback**: Rendu simple si parsing C échoue

### Performance:
- **Timer intelligent**: Rendu seulement si changement de ligne
- **Librairie C**: Parser natif optimisé
- **ARC**: Gestion automatique de la mémoire

## Debugging

### Logs utiles:
```objc
NSLog(@"✅ Librairie C d'édition initialisée avec succès");
NSLog(@"❌ Échec de l'initialisation de la librairie C: %s", error);
NSLog(@"✅ Contenu éditeur mis à jour (%lu caractères)", length);
```

### Points de debug fréquents:
1. **Initialisation librairie C**: Vérifier `editor_library_init()`
2. **Parsing JSON**: Valider la structure retournée
3. **Ranges NSAttributedString**: Éviter les out-of-bounds
4. **Timer de rendu**: S'assurer qu'il n'est pas invalidé prématurément

## Maintenance

### Tests recommandés:
1. **Test de rendu**: Vérifier que les styles Markdown s'appliquent
2. **Test de performance**: Mesurer le temps de rendu sur gros documents
3. **Test d'erreur**: Simuler échecs de parsing C
4. **Test d'interaction**: Vérifier callbacks d'icônes

### Améliorations récentes V3.0:
1. **✅ Cache de rendu intelligent**: Implémenté - évite re-parser le même contenu
2. **✅ Stratégies de parsing multiples**: 3 APIs C intégrées avec fallback automatique
3. **✅ Gestion d'erreur robuste**: Exception handling et récupération gracieuse
4. **✅ Support Markdown étendu**: Code, liens, strikethrough, blockquotes, listes
5. **✅ Documentation Doxygen**: Commentaires détaillés pour maintenance

### Optimisations futures:
1. **Rendu différentiel**: Mettre à jour seulement les parties modifiées
2. **Threading**: Déplacer le parsing C sur thread séparé
3. **Configuration**: Rendre les couleurs et polices configurables
4. **Prévisualisation HTML**: Utiliser editor_markdown_to_html() pour mode préview

## Intégration ElephantNotes V3

### Utilisation dans l'application:
```objc
// Création du framework
UIFramework* framework = ui_framework_create(window);
ui_framework_setup_layout(framework);

// Configuration des callbacks
ui_framework_set_click_handler(framework, icon_click_handler, self);

// Chargement du contenu
ui_framework_set_editor_content(framework, markdownContent);
```

### Intégration avec le système de vaults V3.0:
- **Sauvegarde automatique**: Le contenu de l'éditeur est automatiquement sauvegardé
- **Navigation améliorée**: Callbacks d'icônes permettent navigation fluide entre modes
- **Rendu universel**: Compatible avec tous les fichiers .md du vault
- **Performance optimisée**: Cache intelligent évite re-parser lors de navigation
- **Récupération d'erreur**: Fallback gracieux si problème avec librairie C
- **Logs détaillés**: Monitoring complet des opérations de rendu pour debugging