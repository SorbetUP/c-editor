//
//  main_v4.m
//  ElephantNotes V4 - Architecture Modulaire
//

#import <Cocoa/Cocoa.h>
#import "Modules/Controllers/ENAppDelegate.h"

int main(int argc, const char * argv[]) {
    NSLog(@"🚀 Démarrage d'ElephantNotes V4 - Architecture Modulaire");
    
    @autoreleasepool {
        // Créer l'application
        NSApplication* app = [NSApplication sharedApplication];
        
        // Configurer le délégué d'application
        ENAppDelegate* appDelegate = [[ENAppDelegate alloc] init];
        [app setDelegate:appDelegate];
        
        // Démarrer la boucle d'événements
        NSLog(@"🔄 Lancement de la boucle principale");
        [app run];
        
        // Nettoyage
        [appDelegate release];
    }
    
    return 0;
}