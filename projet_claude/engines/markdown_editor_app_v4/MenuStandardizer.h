//
// MenuStandardizer.h - Classe pour standardiser les dimensions des menus
// Assure que tous les menus (recherche, dashboard, paramètres) ont la même taille
//

#import <Cocoa/Cocoa.h>
#include "../ui_framework/ui_framework.h"

@interface MenuStandardizer : NSObject

// Configuration standard pour tous les menus
@property (nonatomic, readonly) CGRect standardMenuFrame;
@property (nonatomic, readonly) CGFloat sidebarWidth;
@property (nonatomic, readonly) CGFloat headerHeight;
@property (nonatomic, readonly) CGFloat contentPadding;

// Méthodes statiques pour standardisation
+ (instancetype)sharedStandardizer;
+ (NSView*)createStandardMenuContainer:(UIFramework*)framework;
+ (void)setupStandardConstraints:(NSView*)menuView inContainer:(NSView*)container;
+ (CGRect)getStandardContentFrame:(UIFramework*)framework;

// Configuration des menus
- (NSView*)createStandardMenuView:(UIFramework*)framework;
- (void)configureStandardLayout:(NSView*)menuView inFramework:(UIFramework*)framework;
- (CGRect)calculateContentFrame:(UIFramework*)framework;

@end