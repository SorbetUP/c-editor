//
//  ENAppDelegate.h
//  ElephantNotes V4 - Délégué d'application simplifié
//

#import <Cocoa/Cocoa.h>
#import "ENMainController.h"

@interface ENAppDelegate : NSObject <NSApplicationDelegate>

@property (nonatomic, strong) NSWindow* mainWindow;
@property (nonatomic, strong) ENMainController* mainController;
@property (nonatomic, assign) UIFramework* uiFramework;

@end

@interface ENWindow : NSWindow
@end