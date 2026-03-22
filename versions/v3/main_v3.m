// main_v3.m - Point d'entrée principal pour ElephantNotes V3
#import <Cocoa/Cocoa.h>
#import "ElephantNotesV3.h"

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSLog(@"🚀 Démarrage d'ElephantNotes V3");
        
        // Créer l'application
        NSApplication* app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        // Créer et configurer l'app delegate
        ElephantNotesV3AppDelegate* appDelegate = [[ElephantNotesV3AppDelegate alloc] init];
        [app setDelegate:appDelegate];
        
        // Créer le menu principal
        NSMenu* mainMenu = [[NSMenu alloc] init];
        
        // Menu Application
        NSMenuItem* appMenuItem = [[NSMenuItem alloc] init];
        NSMenu* appMenu = [[NSMenu alloc] init];
        
        [appMenu addItemWithTitle:@"À propos d'ElephantNotes V3" 
                           action:@selector(orderFrontStandardAboutPanel:) 
                    keyEquivalent:@""];
        [appMenu addItem:[NSMenuItem separatorItem]];
        [appMenu addItemWithTitle:@"Masquer ElephantNotes V3" 
                           action:@selector(hide:) 
                    keyEquivalent:@"h"];
        [appMenu addItemWithTitle:@"Masquer les autres" 
                           action:@selector(hideOtherApplications:) 
                    keyEquivalent:@"h"];
        [[appMenu itemAtIndex:[appMenu numberOfItems] - 1] setKeyEquivalentModifierMask:NSEventModifierFlagOption | NSEventModifierFlagCommand];
        [appMenu addItemWithTitle:@"Tout afficher" 
                           action:@selector(unhideAllApplications:) 
                    keyEquivalent:@""];
        [appMenu addItem:[NSMenuItem separatorItem]];
        [appMenu addItemWithTitle:@"Quitter ElephantNotes V3" 
                           action:@selector(terminate:) 
                    keyEquivalent:@"q"];
        
        [appMenuItem setSubmenu:appMenu];
        [mainMenu addItem:appMenuItem];
        
        // Menu Fichier
        NSMenuItem* fileMenuItem = [[NSMenuItem alloc] init];
        NSMenu* fileMenu = [[NSMenu alloc] initWithTitle:@"Fichier"];
        
        [fileMenu addItemWithTitle:@"Nouvelle note" 
                            action:@selector(newDocument:) 
                     keyEquivalent:@"n"];
        [fileMenu addItemWithTitle:@"Ouvrir..." 
                            action:@selector(openDocument:) 
                     keyEquivalent:@"o"];
        [fileMenu addItem:[NSMenuItem separatorItem]];
        [fileMenu addItemWithTitle:@"Sauvegarder" 
                            action:@selector(saveDocument:) 
                     keyEquivalent:@"s"];
        [fileMenu addItemWithTitle:@"Sauvegarder sous..." 
                            action:@selector(saveDocumentAs:) 
                     keyEquivalent:@"S"];
        
        [fileMenuItem setSubmenu:fileMenu];
        [mainMenu addItem:fileMenuItem];
        
        // Menu Vaults
        NSMenuItem* vaultMenuItem = [[NSMenuItem alloc] init];
        NSMenu* vaultMenu = [[NSMenu alloc] initWithTitle:@"Vaults"];
        
        [vaultMenu addItemWithTitle:@"Gestionnaire de vaults..." 
                             action:@selector(showVaultManager:) 
                      keyEquivalent:@"v"];
        [vaultMenu addItemWithTitle:@"Nouveau vault..." 
                             action:@selector(showVaultSetup:) 
                      keyEquivalent:@"V"];
        
        [vaultMenuItem setSubmenu:vaultMenu];
        [mainMenu addItem:vaultMenuItem];
        
        // Menu Édition
        NSMenuItem* editMenuItem = [[NSMenuItem alloc] init];
        NSMenu* editMenu = [[NSMenu alloc] initWithTitle:@"Édition"];
        
        [editMenu addItemWithTitle:@"Annuler" 
                            action:@selector(undo:) 
                     keyEquivalent:@"z"];
        [editMenu addItemWithTitle:@"Rétablir" 
                            action:@selector(redo:) 
                     keyEquivalent:@"Z"];
        [editMenu addItem:[NSMenuItem separatorItem]];
        [editMenu addItemWithTitle:@"Couper" 
                            action:@selector(cut:) 
                     keyEquivalent:@"x"];
        [editMenu addItemWithTitle:@"Copier" 
                            action:@selector(copy:) 
                     keyEquivalent:@"c"];
        [editMenu addItemWithTitle:@"Coller" 
                            action:@selector(paste:) 
                     keyEquivalent:@"v"];
        [editMenu addItem:[NSMenuItem separatorItem]];
        [editMenu addItemWithTitle:@"Tout sélectionner" 
                            action:@selector(selectAll:) 
                     keyEquivalent:@"a"];
        
        [editMenuItem setSubmenu:editMenu];
        [mainMenu addItem:editMenuItem];
        
        // Menu Fenêtre
        NSMenuItem* windowMenuItem = [[NSMenuItem alloc] init];
        NSMenu* windowMenu = [[NSMenu alloc] initWithTitle:@"Fenêtre"];
        
        [windowMenu addItemWithTitle:@"Réduire" 
                              action:@selector(performMiniaturize:) 
                       keyEquivalent:@"m"];
        [windowMenu addItemWithTitle:@"Zoom" 
                              action:@selector(performZoom:) 
                       keyEquivalent:@""];
        
        [windowMenuItem setSubmenu:windowMenu];
        [mainMenu addItem:windowMenuItem];
        [app setWindowsMenu:windowMenu];
        
        [app setMainMenu:mainMenu];
        
        // Démarrer l'application
        NSLog(@"🔄 Lancement de la boucle principale");
        [app run];
    }
    
    return 0;
}