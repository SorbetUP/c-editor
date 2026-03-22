//
//  ENAppDelegate.m
//  ElephantNotes V4 - Délégué d'application simplifié
//

#import "ENAppDelegate.h"
#include "../file_manager/professional_file_manager.h"

@implementation ENAppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification {
    NSLog(@"🚀 [ENAppDelegate] Démarrage d'ElephantNotes V4");
    
    [self createMainWindow];
    [self initializeUIFramework];
    [self setupMainController];
    [self showMainWindow];
    
    NSLog(@"✅ [ENAppDelegate] Application démarrée avec succès");
}

- (void)createMainWindow {
    NSLog(@"🪟 [ENAppDelegate] Création de la fenêtre principale");
    
    NSRect windowFrame = NSMakeRect(100, 100, 1200, 800);
    _mainWindow = [[ENWindow alloc] initWithContentRect:windowFrame
                                              styleMask:(NSWindowStyleMaskTitled | 
                                                       NSWindowStyleMaskClosable | 
                                                       NSWindowStyleMaskMiniaturizable | 
                                                       NSWindowStyleMaskResizable)
                                                backing:NSBackingStoreBuffered
                                                  defer:NO];
    
    [_mainWindow setTitle:@"ElephantNotes V4 - Architecture Modulaire"];
    [_mainWindow setMinSize:NSMakeSize(800, 600)];
    [_mainWindow center];
    
    NSLog(@"✅ [ENAppDelegate] Fenêtre principale créée");
}

- (void)initializeUIFramework {
    NSLog(@"🖥️ [ENAppDelegate] Initialisation du UI Framework");
    
    // Créer et configurer le UI Framework
    _uiFramework = ui_framework_create(_mainWindow);
    if (!_uiFramework) {
        NSLog(@"❌ [ENAppDelegate] Échec de création du UI Framework");
        [NSApp terminate:nil];
        return;
    }
    
    // Note: editor_init sera appelé avec un Document, pas globalement
    
    // Initialiser le gestionnaire professionnel
    FileResult fileResult = professional_init();
    if (fileResult != FILE_SUCCESS) {
        NSLog(@"❌ [ENAppDelegate] Échec d'initialisation du gestionnaire professionnel: %d", fileResult);
        [NSApp terminate:nil];
        return;
    }
    
    // Configurer le layout
    ui_framework_setup_layout(_uiFramework);
    
    NSLog(@"✅ [ENAppDelegate] UI Framework initialisé");
}

- (void)setupMainController {
    NSLog(@"🎮 [ENAppDelegate] Configuration du contrôleur principal");
    
    _mainController = [[ENMainController alloc] init];
    [_mainController setupWithUIFramework:_uiFramework];
    
    NSLog(@"✅ [ENAppDelegate] Contrôleur principal configuré");
}

- (void)showMainWindow {
    NSLog(@"👁️ [ENAppDelegate] Affichage de la fenêtre principale");
    
    [_mainWindow makeKeyAndOrderFront:nil];
    [NSApp activateIgnoringOtherApps:YES];
    
    NSLog(@"✅ [ENAppDelegate] Fenêtre principale affichée");
}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {
    return YES;
}

- (void)applicationWillTerminate:(NSNotification *)aNotification {
    NSLog(@"🔄 [ENAppDelegate] Arrêt de l'application");
    
    // Nettoyage
    if (_uiFramework) {
        ui_framework_destroy(_uiFramework);
        _uiFramework = NULL;
    }
    
    professional_cleanup();
    // Note: editor_cleanup sera appelé par document
    
    NSLog(@"✅ [ENAppDelegate] Application fermée proprement");
}

- (void)dealloc {
    [_mainWindow release];
    [_mainController release];
    [super dealloc];
}

@end

@implementation ENWindow

- (void)keyDown:(NSEvent *)event {
    // Gestion des raccourcis clavier globaux
    NSString *characters = [event characters];
    NSUInteger modifierFlags = [event modifierFlags];
    
    if (modifierFlags & NSEventModifierFlagCommand) {
        if ([characters isEqualToString:@"n"]) {
            NSLog(@"⌨️ [ENWindow] Nouveau fichier (⌘+N)");
            // TODO: Implémenter nouveau fichier
            return;
        } else if ([characters isEqualToString:@"o"]) {
            NSLog(@"⌨️ [ENWindow] Ouvrir fichier (⌘+O)");
            // TODO: Implémenter ouvrir fichier
            return;
        } else if ([characters isEqualToString:@"s"]) {
            NSLog(@"⌨️ [ENWindow] Sauvegarder (⌘+S)");
            // TODO: Implémenter sauvegarde
            return;
        }
    }
    
    // Transmettre les autres événements
    [super keyDown:event];
}

@end