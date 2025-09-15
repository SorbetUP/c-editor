#import <Cocoa/Cocoa.h>

// Global variables for simple test
NSTextView *g_textView = nil;

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        // Create basic window
        NSRect frame = NSMakeRect(100, 100, 800, 600);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"Simple Hybrid Test"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]];
        
        // Create simple text editor with proper setup
        NSRect textFrame = NSMakeRect(20, 20, 760, 560);
        NSScrollView *scrollView = [[NSScrollView alloc] initWithFrame:textFrame];
        [scrollView setHasVerticalScroller:YES];
        [scrollView setHasHorizontalScroller:NO];
        [scrollView setAutohidesScrollers:YES];
        [scrollView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        [scrollView setBorderType:NSLineBorder];
        
        // Get content size properly
        NSSize contentSize = [scrollView contentSize];
        NSTextView *textView = [[NSTextView alloc] initWithFrame:NSMakeRect(0, 0, contentSize.width, contentSize.height)];
        
        // Configure text view properly
        [textView setMinSize:NSMakeSize(0.0, contentSize.height)];
        [textView setMaxSize:NSMakeSize(FLT_MAX, FLT_MAX)];
        [textView setVerticallyResizable:YES];
        [textView setHorizontallyResizable:NO];
        [textView setAutoresizingMask:NSViewWidthSizable];
        
        // IMPORTANT: Set background and text colors - USING RED BACKGROUND FOR DEBUGGING
        [textView setBackgroundColor:[NSColor redColor]]; // RED for visibility test
        [textView setTextColor:[NSColor whiteColor]]; // WHITE text for visibility
        [textView setInsertionPointColor:[NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0]]; // Blue cursor
        [textView setFont:[NSFont fontWithName:@"Monaco" size:14]];
        [textView setRichText:NO];
        [textView setImportsGraphics:NO];
        
        // Configure text container
        [[textView textContainer] setContainerSize:NSMakeSize(contentSize.width, FLT_MAX)];
        [[textView textContainer] setWidthTracksTextView:YES];
        
        // Set initial content - SIMPLE TEST FIRST
        [textView setString:@"TEST: Can you see this text? If yes, type below this line.\n\n# Titre\n\nÉcrivez votre markdown ici...\n\nExemple:\n- **Gras**\n- *Italique*\n- ==Surligné==\n- ++Souligné++"];
        
        [scrollView setDocumentView:textView];
        g_textView = textView;
        
        [[window contentView] addSubview:scrollView];
        
        // Show window
        [window center];
        [window makeKeyAndOrderFront:nil];
        [app activateIgnoringOtherApps:YES];
        
        NSLog(@"🚀 Simple hybrid test ready - can you type and see text?");
        
        // Run app
        [app run];
    }
    
    return 0;
}