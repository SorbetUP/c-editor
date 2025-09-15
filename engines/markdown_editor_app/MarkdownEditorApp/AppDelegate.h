#import <Cocoa/Cocoa.h>

@class SimpleViewController;

@interface AppDelegate : NSObject <NSApplicationDelegate>

@property (strong, nonatomic) NSWindow *window;
@property (strong, nonatomic) SimpleViewController *editorViewController;

@end