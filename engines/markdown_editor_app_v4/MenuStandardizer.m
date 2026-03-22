//
// MenuStandardizer.m - Implémentation de la standardisation des menus
// Assure que tous les menus (recherche, dashboard, paramètres) ont la même taille
//

#import "MenuStandardizer.h"

@implementation MenuStandardizer

static MenuStandardizer* _sharedInstance = nil;

+ (instancetype)sharedStandardizer {
    static dispatch_once_t onceToken;
    dispatch_once(&onceToken, ^{
        _sharedInstance = [[MenuStandardizer alloc] init];
    });
    return _sharedInstance;
}

- (instancetype)init {
    self = [super init];
    if (self) {
        // Configuration standard basée sur l'UI Framework
        _sidebarWidth = 60.0;
        _headerHeight = 0.0;  // Pas de header supplémentaire, on utilise l'espace éditeur
        _contentPadding = 0.0; // Utiliser tout l'espace disponible
    }
    return self;
}

+ (NSView*)createStandardMenuContainer:(UIFramework*)framework {
    if (!framework) return nil;
    
    // Récupérer l'éditeur pour avoir accès à sa zone
    NSTextView* editor = ui_framework_get_editor(framework);
    if (!editor) return nil;
    
    // Récupérer le conteneur de l'éditeur
    NSView* editorContainer = [editor superview];
    while (editorContainer && ![editorContainer isKindOfClass:[NSScrollView class]]) {
        editorContainer = [editorContainer superview];
    }
    
    if (editorContainer) {
        editorContainer = [editorContainer superview]; // Parent du ScrollView
    }
    
    if (!editorContainer) {
        NSLog(@"⚠️ MenuStandardizer: Impossible de trouver le conteneur éditeur");
        return nil;
    }
    
    // Nettoyer le conteneur existant
    for (NSView* subview in [editorContainer subviews]) {
        [subview removeFromSuperview];
    }
    
    // Créer le conteneur de menu standard
    NSView* menuContainer = [[NSView alloc] init];
    [menuContainer setTranslatesAutoresizingMaskIntoConstraints:NO];
    [editorContainer addSubview:menuContainer];
    
    // Contraintes pour utiliser exactement l'espace de l'éditeur
    [NSLayoutConstraint activateConstraints:@[
        [menuContainer.topAnchor constraintEqualToAnchor:editorContainer.topAnchor],
        [menuContainer.leadingAnchor constraintEqualToAnchor:editorContainer.leadingAnchor],
        [menuContainer.trailingAnchor constraintEqualToAnchor:editorContainer.trailingAnchor],
        [menuContainer.bottomAnchor constraintEqualToAnchor:editorContainer.bottomAnchor]
    ]];
    
    NSLog(@"✅ MenuStandardizer: Conteneur menu standard créé");
    return menuContainer;
}

+ (void)setupStandardConstraints:(NSView*)menuView inContainer:(NSView*)container {
    if (!menuView || !container) return;
    
    [menuView setTranslatesAutoresizingMaskIntoConstraints:NO];
    
    [NSLayoutConstraint activateConstraints:@[
        [menuView.topAnchor constraintEqualToAnchor:container.topAnchor],
        [menuView.leadingAnchor constraintEqualToAnchor:container.leadingAnchor],
        [menuView.trailingAnchor constraintEqualToAnchor:container.trailingAnchor],
        [menuView.bottomAnchor constraintEqualToAnchor:container.bottomAnchor]
    ]];
}

+ (CGRect)getStandardContentFrame:(UIFramework*)framework {
    if (!framework) return CGRectZero;
    
    NSTextView* editor = ui_framework_get_editor(framework);
    if (!editor) return CGRectZero;
    
    NSView* editorContainer = [editor superview];
    if (editorContainer) {
        return [editorContainer frame];
    }
    
    return CGRectZero;
}

- (NSView*)createStandardMenuView:(UIFramework*)framework {
    return [MenuStandardizer createStandardMenuContainer:framework];
}

- (void)configureStandardLayout:(NSView*)menuView inFramework:(UIFramework*)framework {
    // Cette méthode peut être utilisée pour des configurations spécifiques supplémentaires
    [menuView setWantsLayer:YES];
    menuView.layer.backgroundColor = [[NSColor controlBackgroundColor] CGColor];
}

- (CGRect)calculateContentFrame:(UIFramework*)framework {
    return [MenuStandardizer getStandardContentFrame:framework];
}

- (CGRect)standardMenuFrame {
    // Frame calculé dynamiquement basé sur l'UI Framework
    return CGRectMake(0, 0, 800, 600); // Valeurs par défaut, sera ajusté dynamiquement
}

@end