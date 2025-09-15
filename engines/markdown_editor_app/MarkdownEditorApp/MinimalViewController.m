#import <Cocoa/Cocoa.h>

@interface MinimalViewController : NSViewController
@end

@implementation MinimalViewController

- (void)viewDidLoad {
    [super viewDidLoad];
    
    NSLog(@"✅ MinimalViewController loaded");
    
    // Just create a simple colored view
    NSView *testView = [[NSView alloc] init];
    testView.wantsLayer = YES;
    testView.layer.backgroundColor = [NSColor redColor].CGColor;
    
    self.view = testView;
    
    NSLog(@"✅ Red test view created");
}

@end