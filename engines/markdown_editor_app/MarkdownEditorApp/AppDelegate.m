#import "AppDelegate.h"

@implementation AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification {
    NSLog(@"🚀 App launching...");
    
    // Force app activation
    [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];
    
    // Create the main window with same dark styling as HTML version
    NSRect windowFrame = NSMakeRect(100, 100, 800, 600);
    
    self.window = [[NSWindow alloc] initWithContentRect:windowFrame
                                              styleMask:(NSWindowStyleMaskTitled |
                                                       NSWindowStyleMaskClosable |
                                                       NSWindowStyleMaskMiniaturizable |
                                                       NSWindowStyleMaskResizable)
                                                backing:NSBackingStoreBuffered
                                                  defer:NO];
    
    NSLog(@"📦 Window created");
    
    [self.window setTitle:@"C Markdown Editor - Test"];
    [self.window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]];
    
    // Create a working text editor without complex constraints
    NSScrollView *scrollView = [[NSScrollView alloc] initWithFrame:self.window.contentView.bounds];
    scrollView.hasVerticalScroller = YES;
    scrollView.hasHorizontalScroller = NO;
    scrollView.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
    scrollView.backgroundColor = [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0];
    
    NSTextView *textView = [[NSTextView alloc] init];
    textView.backgroundColor = [NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0];
    textView.textColor = [NSColor whiteColor];
    textView.insertionPointColor = [NSColor whiteColor];
    textView.font = [NSFont fontWithName:@"Monaco" size:14];
    textView.string = @"# C Markdown Editor\n\nCeci est votre **éditeur markdown** natif macOS !\n\n- Utilise les moteurs C\n- Interface native\n- Performance optimale\n\nÉcrivez votre *markdown* ici...";
    
    scrollView.documentView = textView;
    [self.window setContentView:scrollView];
    
    NSLog(@"🎛️ View controller set");
    
    // Make window visible and bring to front
    [self.window center];
    [self.window makeKeyAndOrderFront:nil];
    [self.window orderFrontRegardless];
    
    // Force app activation
    [NSApp activateIgnoringOtherApps:YES];
    
    NSLog(@"✅ Window should be visible now");
}

- (void)applicationWillTerminate:(NSNotification *)aNotification {
    // Cleanup
}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {
    return YES;
}

// Handle file opening (drag & drop or Open With)
- (BOOL)application:(NSApplication *)sender openFile:(NSString *)filename {
    NSLog(@"📁 Would open file: %@", filename);
    return YES;
}

@end