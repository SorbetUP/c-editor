//
//  ENSidebar.h
//  ElephantNotes V4 - Module de gestion de la barre latérale
//

#import <Cocoa/Cocoa.h>
#include "../ui_framework/ui_framework.h"

@protocol ENSidebarDelegate <NSObject>
- (void)sidebar:(id)sender didClickIcon:(UIIconType)iconType;
- (void)sidebar:(id)sender didHoverIcon:(UIIconType)iconType isHovering:(bool)isHovering;
@end

@interface ENSidebar : NSObject

@property (nonatomic, assign) id<ENSidebarDelegate> delegate;
@property (nonatomic, assign) UIFramework* uiFramework;
@property (nonatomic, readonly) UIIconType activeIcon;

// Initialisation
- (instancetype)initWithUIFramework:(UIFramework*)framework;
- (void)setup;

// Gestion des icônes
- (void)setActiveIcon:(UIIconType)iconType;
- (void)enableIcon:(UIIconType)iconType;
- (void)disableIcon:(UIIconType)iconType;

// Callbacks C pour l'UI Framework
void sidebar_icon_click_handler(UIIconType iconType, void* userData);
void sidebar_icon_hover_handler(UIIconType iconType, bool isHovering, void* userData);

@end