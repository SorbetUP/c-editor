//
//  ENToolsTab.h
//  ElephantNotes V4 - Module Tools pour les outils et l'aide
//

#import "ENTabBase.h"

@interface ENToolsTab : ENTabBase

@property (nonatomic, assign) BOOL debugMode;
@property (nonatomic, strong) NSArray* recentActions;

// Méthodes d'outils
- (NSString*)generateToolsContent;
- (NSString*)generateDebugContent;
- (NSString*)generateHelpContent;
- (NSString*)generateShortcutsContent;

// Actions d'outils
- (void)exportVault;
- (void)importVault;
- (void)optimizeVault;
- (void)validateVault;

@end