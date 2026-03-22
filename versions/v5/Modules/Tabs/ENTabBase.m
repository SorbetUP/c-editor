//
//  ENTabBase.m
//  ElephantNotes V4 - Classe de base pour tous les onglets
//

#import "ENTabBase.h"

@interface ENTabBase()
@property (nonatomic, copy) NSString* tabName;
@property (nonatomic, copy) NSString* tabIcon;
@end

@implementation ENTabBase

- (instancetype)initWithName:(NSString*)name icon:(NSString*)icon {
    self = [super init];
    if (self) {
        _tabName = [name copy];
        _tabIcon = [icon copy];
    }
    return self;
}

- (NSString*)generateContent {
    // Méthode à implémenter par les sous-classes
    return @"# Contenu par défaut\n\nCette méthode doit être implémentée par la sous-classe.";
}

- (void)didBecomeActive {
    // Appelé quand l'onglet devient actif
    [self displayContent];
}

- (void)didBecomeInactive {
    // Appelé quand l'onglet devient inactif
    // Peut être utilisé pour nettoyer des timers, etc.
}

- (void)refreshContent {
    // Rafraîchit le contenu de l'onglet
    if (_uiFramework) {
        [self displayContent];
    }
}

- (void)displayContent {
    if (!_uiFramework) {
        NSLog(@"⚠️ [%@] UIFramework non défini", _tabName);
        return;
    }
    
    NSString* content = [self generateContent];
    if (content) {
        [self showInEditor:content];
    }
}

- (void)showInEditor:(NSString*)content {
    if (!_uiFramework || !content) {
        NSLog(@"⚠️ [%@] Impossible d'afficher le contenu", _tabName);
        return;
    }
    
    // Utiliser la fonction UI Framework pour afficher le contenu
    ui_framework_set_editor_content(_uiFramework, content);
    NSLog(@"📄 [%@] Contenu affiché (%lu caractères)", _tabName, (unsigned long)[content length]);
}

- (void)handleContentSave:(NSString*)content {
    // Méthode par défaut - peut être overridée par les sous-classes
    // Notifier le délégué si défini
    if (self.delegate && [self.delegate respondsToSelector:@selector(tabContentDidSave:withContent:)]) {
        [self.delegate tabContentDidSave:self withContent:content];
    }
}

- (void)dealloc {
    // Nettoyage
    [_tabName release];
    [_tabIcon release];
    [_currentVaultPath release];
    [_currentVaultName release];
    [super dealloc];
}

@end