//
//  ENAppDelegate.m
//  ElephantNotes V4 - Délégué d'application simplifié
//

#import "ENAppDelegate.h"
#include "professional_file_manager.h"

@implementation ENAppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification {
    NSLog(@"🚀 [ENAppDelegate] Démarrage d'ElephantNotes V4");
    
    [self createMainMenu];
    [self createMainWindow];
    [self initializeUIFramework];
    [self setupMainController];
    [self showMainWindow];
    
    NSLog(@"✅ [ENAppDelegate] Application démarrée avec succès");
}

- (void)createMainMenu {
    NSLog(@"📋 [ENAppDelegate] Création du menu principal");
    
    NSMenu* mainMenu = [[NSMenu alloc] init];
    
    // Menu Application
    NSMenuItem* appMenuItem = [[NSMenuItem alloc] init];
    NSMenu* appMenu = [[NSMenu alloc] initWithTitle:@"ElephantNotes"];
    
    [appMenu addItemWithTitle:@"About ElephantNotes" action:nil keyEquivalent:@""];
    [appMenu addItem:[NSMenuItem separatorItem]];
    [appMenu addItemWithTitle:@"Hide ElephantNotes" action:@selector(hide:) keyEquivalent:@"h"];
    [appMenu addItemWithTitle:@"Hide Others" action:@selector(hideOtherApplications:) keyEquivalent:@"h"];
    [[appMenu itemAtIndex:[appMenu numberOfItems] - 1] setKeyEquivalentModifierMask:(NSEventModifierFlagOption | NSEventModifierFlagCommand)];
    [appMenu addItemWithTitle:@"Show All" action:@selector(unhideAllApplications:) keyEquivalent:@""];
    [appMenu addItem:[NSMenuItem separatorItem]];
    [appMenu addItemWithTitle:@"Quit ElephantNotes" action:@selector(terminate:) keyEquivalent:@"q"];
    
    [appMenuItem setSubmenu:appMenu];
    [mainMenu addItem:appMenuItem];
    
    // Menu Edit (pour les raccourcis copy/paste)
    NSMenuItem* editMenuItem = [[NSMenuItem alloc] init];
    NSMenu* editMenu = [[NSMenu alloc] initWithTitle:@"Edit"];
    
    [editMenu addItemWithTitle:@"Undo" action:@selector(undo:) keyEquivalent:@"z"];
    [editMenu addItemWithTitle:@"Redo" action:@selector(redo:) keyEquivalent:@"Z"];
    [editMenu addItem:[NSMenuItem separatorItem]];
    [editMenu addItemWithTitle:@"Cut" action:@selector(cut:) keyEquivalent:@"x"];
    [editMenu addItemWithTitle:@"Copy" action:@selector(copy:) keyEquivalent:@"c"];
    [editMenu addItemWithTitle:@"Paste" action:@selector(paste:) keyEquivalent:@"v"];
    [editMenu addItemWithTitle:@"Select All" action:@selector(selectAll:) keyEquivalent:@"a"];
    
    [editMenuItem setSubmenu:editMenu];
    [mainMenu addItem:editMenuItem];
    
    // Menu File
    NSMenuItem* fileMenuItem = [[NSMenuItem alloc] init];
    NSMenu* fileMenu = [[NSMenu alloc] initWithTitle:@"File"];
    
    [fileMenu addItemWithTitle:@"New" action:@selector(newDocument:) keyEquivalent:@"n"];
    [fileMenu addItemWithTitle:@"Open..." action:@selector(openDocument:) keyEquivalent:@"o"];
    [fileMenu addItem:[NSMenuItem separatorItem]];
    [fileMenu addItemWithTitle:@"Save" action:@selector(saveDocument:) keyEquivalent:@"s"];
    [fileMenu addItemWithTitle:@"Save As..." action:@selector(saveDocumentAs:) keyEquivalent:@"S"];
    
    [fileMenuItem setSubmenu:fileMenu];
    [mainMenu addItem:fileMenuItem];
    
    [NSApp setMainMenu:mainMenu];
    
    [appMenu release];
    [appMenuItem release];
    [editMenu release];
    [editMenuItem release];
    [fileMenu release];
    [fileMenuItem release];
    [mainMenu release];
    
    NSLog(@"✅ [ENAppDelegate] Menu principal créé");
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
    // Laisser le menu gérer tous les raccourcis standards
    // Ne plus intercepter ici, le menu s'en occupe
    [super keyDown:event];
}

@end