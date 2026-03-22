// ui_framework.h - Framework d'interface utilisateur pour ElephantNotes
// Système de composants UI modulaire avec barre latérale et éditeur intégré

#ifndef UI_FRAMEWORK_H
#define UI_FRAMEWORK_H

#include <Cocoa/Cocoa.h>
#include <stdbool.h>
#include "../cursor/cursor_manager.h"

#ifdef __cplusplus
extern "C" {
#endif

// Types d'icônes pour la barre latérale
// Ordre optimisé pour un workflow naturel d'utilisation
typedef enum {
    UI_ICON_HOME = 0,       // 🏠 Maison - dashboard (vue d'ensemble)
    UI_ICON_FILES = 1,      // 📁 Dossier - gestionnaire de fichiers (navigation)
    UI_ICON_EDITOR = 2,     // 📝 Crayon - éditeur de texte (création/édition)
    UI_ICON_SEARCH = 3,     // 🔍 Loupe - recherche (recherche dans le contenu)
    UI_ICON_TABLES = 4,     // 📊 Tableau - création et gestion des tableaux
    UI_ICON_TOOLS = 5,      // 🔧 Outils - maintenance et aide (utilitaires)
    UI_ICON_SETTINGS = 6,   // ⚙️ Roue crantée - paramètres (configuration)
    UI_ICON_BACK = 7        // ⬅️ Flèche - retour/navigation
} UIIconType;

// États des composants UI
typedef enum {
    UI_STATE_NORMAL = 0,
    UI_STATE_HOVER = 1,
    UI_STATE_ACTIVE = 2,
    UI_STATE_DISABLED = 3
} UIState;

// Configuration de la barre latérale
typedef struct {
    CGFloat width;              // Largeur de la barre (défaut: 60px)
    NSColor* backgroundColor;   // Couleur de fond
    NSColor* iconColor;         // Couleur des icônes
    NSColor* hoverColor;        // Couleur au survol
    NSColor* activeColor;       // Couleur active
    CGFloat iconSize;           // Taille des icônes (défaut: 24px)
    CGFloat iconSpacing;        // Espacement entre icônes (défaut: 20px)
} UISidebarConfig;

// Configuration de l'éditeur
typedef struct {
    NSColor* backgroundColor;   // Couleur de fond de l'éditeur
    NSColor* textColor;         // Couleur du texte
    NSFont* font;              // Police de l'éditeur
    CGFloat marginLeft;         // Marge gauche pour la barre latérale
    bool showLineNumbers;       // Afficher numéros de ligne
} UIEditorConfig;

// Gestionnaire d'événements pour les icônes
typedef void (*UIIconClickHandler)(UIIconType iconType, void* userData);
typedef void (*UIIconHoverHandler)(UIIconType iconType, bool isHovering, void* userData);

// Structure principale du framework UI
typedef struct {
    NSWindow* window;
    NSView* containerView;
    NSView* sidebarView;
    id editorView;  // MarkdownEditor instance
    NSScrollView* editorScrollView;
    
    UISidebarConfig sidebarConfig;
    UIEditorConfig editorConfig;
    
    UIIconClickHandler clickHandler;
    UIIconHoverHandler hoverHandler;
    void* userData;
    
    NSMutableArray* iconButtons;
    UIIconType activeIcon;
    
    bool isInitialized;
} UIFramework;

// ========== API Principal ==========

// Initialisation et nettoyage
UIFramework* ui_framework_create(NSWindow* window);
void ui_framework_destroy(UIFramework* framework);
bool ui_framework_setup_layout(UIFramework* framework);

// Configuration de la barre latérale
void ui_framework_set_sidebar_config(UIFramework* framework, const UISidebarConfig* config);
UISidebarConfig ui_framework_get_default_sidebar_config(void);
void ui_framework_add_icon(UIFramework* framework, UIIconType iconType);
void ui_framework_set_icon_active(UIFramework* framework, UIIconType iconType);
void ui_framework_set_icon_enabled(UIFramework* framework, UIIconType iconType, bool enabled);

// Configuration de l'éditeur
void ui_framework_set_editor_config(UIFramework* framework, const UIEditorConfig* config);
UIEditorConfig ui_framework_get_default_editor_config(void);
NSTextView* ui_framework_get_editor(UIFramework* framework);

// Gestionnaires d'événements
void ui_framework_set_click_handler(UIFramework* framework, UIIconClickHandler handler, void* userData);
void ui_framework_set_hover_handler(UIFramework* framework, UIIconHoverHandler handler, void* userData);

// Gestion du contenu de l'éditeur
void ui_framework_set_editor_content(UIFramework* framework, NSString* content);
NSString* ui_framework_get_editor_content(UIFramework* framework);
void ui_framework_clear_editor(UIFramework* framework);

// Gestion de la fenêtre et mise en page
void ui_framework_update_layout(UIFramework* framework);
void ui_framework_show(UIFramework* framework);
void ui_framework_hide(UIFramework* framework);

// Utilitaires pour les icônes
NSImage* ui_framework_create_icon_image(UIIconType iconType, CGFloat size, NSColor* color);
NSString* ui_framework_get_icon_name(UIIconType iconType);
NSString* ui_framework_get_icon_tooltip(UIIconType iconType);

// Thèmes et apparence
typedef struct {
    NSColor* primaryColor;      // Couleur principale
    NSColor* secondaryColor;    // Couleur secondaire
    NSColor* backgroundLight;   // Fond clair
    NSColor* backgroundDark;    // Fond sombre
    NSColor* textPrimary;       // Texte principal
    NSColor* textSecondary;     // Texte secondaire
    NSColor* accentColor;       // Couleur d'accent
} UITheme;

void ui_framework_apply_theme(UIFramework* framework, const UITheme* theme);
UITheme ui_framework_get_default_light_theme(void);
UITheme ui_framework_get_default_dark_theme(void);

// Animation et transitions
void ui_framework_animate_icon_hover(UIFramework* framework, UIIconType iconType, bool hover);
void ui_framework_animate_sidebar_width(UIFramework* framework, CGFloat targetWidth, NSTimeInterval duration);

// État et persistance
typedef struct {
    UIIconType activeIcon;
    CGFloat sidebarWidth;
    bool sidebarVisible;
    NSString* editorContent;
    NSRange editorSelection;
} UIFrameworkState;

UIFrameworkState ui_framework_save_state(UIFramework* framework);
void ui_framework_restore_state(UIFramework* framework, const UIFrameworkState* state);

#ifdef __cplusplus
}
#endif

#endif // UI_FRAMEWORK_H