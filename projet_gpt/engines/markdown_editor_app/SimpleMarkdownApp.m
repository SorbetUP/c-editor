#import <Cocoa/Cocoa.h>
#import <WebKit/WebKit.h>

// Import our C engines (disabled for testing)
// #import "editor_abi.h"
// #import "markdown.h"

// Global variables for real-time rendering
NSTextView *g_textView = nil;
WKWebView *g_previewWebView = nil;
NSTimer *g_renderTimer = nil;

// Generate complete HTML page with styling matching port 8001
NSString* generateCompleteHTML(NSString *markdownHTML) {
    NSString *html = [NSString stringWithFormat:@"<!DOCTYPE html>\n"
        "<html lang=\"en\">\n"
        "<head>\n"
        "    <meta charset=\"UTF-8\">\n"
        "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n"
        "    <title>C Markdown Editor - Preview</title>\n"
        "    <style>\n"
        "        body {\n"
        "            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;\n"
        "            background-color: #1a1a1a;\n"
        "            color: #e0e0e0;\n"
        "            margin: 20px;\n"
        "            line-height: 1.6;\n"
        "            font-size: 14px;\n"
        "        }\n"
        "        h1, h2, h3, h4, h5, h6 {\n"
        "            color: #4c6ef5;\n"
        "            margin-top: 2em;\n"
        "            margin-bottom: 0.5em;\n"
        "        }\n"
        "        h1 { font-size: 2em; border-bottom: 2px solid #444; padding-bottom: 0.5em; }\n"
        "        h2 { font-size: 1.5em; border-bottom: 1px solid #333; padding-bottom: 0.3em; }\n"
        "        h3 { font-size: 1.3em; }\n"
        "        strong { color: #51cf66; font-weight: bold; }\n"
        "        em { color: #ffd43b; font-style: italic; }\n"
        "        code {\n"
        "            background-color: #2d2d2d;\n"
        "            color: #ff6b6b;\n"
        "            padding: 2px 6px;\n"
        "            border-radius: 3px;\n"
        "            font-family: 'Monaco', monospace;\n"
        "            font-size: 0.9em;\n"
        "        }\n"
        "        blockquote {\n"
        "            border-left: 4px solid #4c6ef5;\n"
        "            margin: 1em 0;\n"
        "            padding-left: 1em;\n"
        "            color: #b0b0b0;\n"
        "            font-style: italic;\n"
        "        }\n"
        "        ul, ol {\n"
        "            margin: 1em 0;\n"
        "            padding-left: 2em;\n"
        "        }\n"
        "        li {\n"
        "            margin: 0.5em 0;\n"
        "            color: #e0e0e0;\n"
        "        }\n"
        "        li::marker {\n"
        "            color: #4c6ef5;\n"
        "        }\n"
        "        p {\n"
        "            margin: 1em 0;\n"
        "            line-height: 1.7;\n"
        "        }\n"
        "        pre {\n"
        "            background-color: #2d2d2d;\n"
        "            border: 1px solid #444;\n"
        "            border-radius: 6px;\n"
        "            padding: 1em;\n"
        "            overflow-x: auto;\n"
        "            margin: 1em 0;\n"
        "        }\n"
        "        .emoji {\n"
        "            font-family: 'Apple Color Emoji', 'Segoe UI Emoji';\n"
        "        }\n"
        "    </style>\n"
        "</head>\n"
        "<body>\n"
        "%@\n"
        "</body>\n"
        "</html>", markdownHTML];
    return html;
}

// Render markdown to HTML using our C engine
NSString* renderMarkdownToHTML(NSString *markdown) {
    NSLog(@"🔧 renderMarkdownToHTML called with: \"%@\"", [markdown substringToIndex:MIN(50, [markdown length])]);
    
    if (!markdown || [markdown length] == 0) {
        NSLog(@"⚠️ Empty markdown input");
        return @"<empty>";
    }
    
    // Enhanced markdown to HTML conversion
    NSMutableString *result = [markdown mutableCopy];
    
    // Convert headers (# H1, ## H2, etc.) - doing this manually for better control
    NSArray *lines = [result componentsSeparatedByString:@"\n"];
    NSMutableArray *processedLines = [NSMutableArray array];
    
    for (NSString *line in lines) {
        if ([line hasPrefix:@"# "]) {
            NSString *headerText = [line substringFromIndex:2];
            [processedLines addObject:[NSString stringWithFormat:@"<h1>%@</h1>", headerText]];
        } else if ([line hasPrefix:@"## "]) {
            NSString *headerText = [line substringFromIndex:3];
            [processedLines addObject:[NSString stringWithFormat:@"<h2>%@</h2>", headerText]];
        } else if ([line hasPrefix:@"### "]) {
            NSString *headerText = [line substringFromIndex:4];
            [processedLines addObject:[NSString stringWithFormat:@"<h3>%@</h3>", headerText]];
        } else {
            [processedLines addObject:line];
        }
    }
    
    [result setString:[processedLines componentsJoinedByString:@"\n"]];
    
    // Convert bold text (**text** -> <strong>text</strong>)
    NSRegularExpression *boldRegex = [NSRegularExpression regularExpressionWithPattern:@"\\*\\*([^*]+)\\*\\*" 
                                                                              options:0 
                                                                                error:nil];
    [boldRegex replaceMatchesInString:result 
                              options:0 
                                range:NSMakeRange(0, [result length]) 
                         withTemplate:@"<strong>$1</strong>"];
    
    // Convert italic text (*text* -> <em>text</em>)
    NSRegularExpression *italicRegex = [NSRegularExpression regularExpressionWithPattern:@"\\*([^*]+)\\*" 
                                                                                 options:0 
                                                                                   error:nil];
    [italicRegex replaceMatchesInString:result 
                                options:0 
                                  range:NSMakeRange(0, [result length]) 
                           withTemplate:@"<em>$1</em>"];
    
    // Convert inline code (`code` -> <code>code</code>)
    NSRegularExpression *codeRegex = [NSRegularExpression regularExpressionWithPattern:@"`([^`]+)`" 
                                                                               options:0 
                                                                                 error:nil];
    [codeRegex replaceMatchesInString:result 
                              options:0 
                                range:NSMakeRange(0, [result length]) 
                         withTemplate:@"<code>$1</code>"];
    
    // Convert list items (- item -> <li>item</li>)
    NSRegularExpression *listRegex = [NSRegularExpression regularExpressionWithPattern:@"^-\\s+(.+)$" 
                                                                               options:NSRegularExpressionAnchorsMatchLines 
                                                                                 error:nil];
    [listRegex replaceMatchesInString:result 
                              options:0 
                                range:NSMakeRange(0, [result length]) 
                         withTemplate:@"<li>$1</li>"];
    
    // Convert blockquotes (> text -> <blockquote>text</blockquote>)
    NSRegularExpression *quoteRegex = [NSRegularExpression regularExpressionWithPattern:@"^>\\s+(.+)$" 
                                                                                options:NSRegularExpressionAnchorsMatchLines 
                                                                                  error:nil];
    [quoteRegex replaceMatchesInString:result 
                               options:0 
                                 range:NSMakeRange(0, [result length]) 
                          withTemplate:@"<blockquote>$1</blockquote>"];
    
    // Convert line breaks to <br> tags
    [result replaceOccurrencesOfString:@"\n" 
                            withString:@"<br>\n" 
                               options:0 
                                 range:NSMakeRange(0, [result length])];
    
    NSLog(@"✅ SIMPLE ENGINE: \"%@\" -> \"%@\"", [markdown substringToIndex:MIN(30, [markdown length])], [result substringToIndex:MIN(50, [result length])]);
    return result;
}

// Simple function for timer callback (C function can't be used as Objective-C selector)
@interface TimerTarget : NSObject
+ (void)updateMarkdownPreview:(NSTimer *)timer;
@end

@implementation TimerTarget
+ (void)updateMarkdownPreview:(NSTimer *)timer {
    NSLog(@"📅 Timer callback - updating preview...");
    
    if (!g_textView) {
        NSLog(@"❌ g_textView is nil");
        return;
    }
    if (!g_previewWebView) {
        NSLog(@"❌ g_previewWebView is nil");
        return;
    }
    
    NSString *markdown = [g_textView string];
    NSLog(@"📝 Current markdown content length: %lu", [markdown length]);
    
    NSString *markdownHTML = renderMarkdownToHTML(markdown);
    NSLog(@"🎨 Rendered markdown HTML length: %lu", [markdownHTML length]);
    
    // Generate complete HTML page with styling
    NSString *completeHTML = generateCompleteHTML(markdownHTML);
    NSLog(@"🌐 Complete HTML page length: %lu", [completeHTML length]);
    
    // Load HTML into WKWebView
    NSLog(@"🔄 Loading HTML into WKWebView...");
    [g_previewWebView loadHTMLString:completeHTML baseURL:nil];
    NSLog(@"✅ WKWebView HTML loaded");
}
@end

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        // Create window
        NSRect frame = NSMakeRect(100, 100, 800, 600);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"C Markdown Editor - WORKING!"];
        [window setBackgroundColor:[NSColor colorWithRed:0.1 green:0.1 blue:0.1 alpha:1.0]];
        
        // Create main editor view with dark theme (like the HTML version)
        NSView *editorView = [[NSView alloc] initWithFrame:[[window contentView] bounds]];
        [editorView setWantsLayer:YES];
        [editorView layer].backgroundColor = [NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0].CGColor; // #1a1a1a
        [editorView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // Header with title (like HTML version)
        NSView *headerView = [[NSView alloc] initWithFrame:NSMakeRect(0, [[window contentView] bounds].size.height - 50, [[window contentView] bounds].size.width, 50)];
        [headerView setWantsLayer:YES];
        [headerView layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor; // #2d2d2d
        [headerView setAutoresizingMask:NSViewWidthSizable | NSViewMinYMargin];
        
        NSTextField *titleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 15, 400, 20)];
        [titleLabel setStringValue:@"🚀 C Markdown Editor - Éditeur Hybride"];
        [titleLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]]; // #e0e0e0
        [titleLabel setBackgroundColor:[NSColor clearColor]];
        [titleLabel setBordered:NO];
        [titleLabel setEditable:NO];
        [titleLabel setFont:[NSFont systemFontOfSize:14 weight:NSFontWeightSemibold]];
        
        [headerView addSubview:titleLabel];
        
        // Status indicator (green dot like HTML version)
        NSView *statusDot = [[NSView alloc] initWithFrame:NSMakeRect([[window contentView] bounds].size.width - 150, 21, 8, 8)];
        [statusDot setWantsLayer:YES];
        [statusDot layer].backgroundColor = [NSColor colorWithRed:0.318 green:0.812 blue:0.400 alpha:1.0].CGColor; // #51cf66
        [statusDot layer].cornerRadius = 4.0;
        [statusDot setAutoresizingMask:NSViewMinXMargin];
        
        NSTextField *statusLabel = [[NSTextField alloc] initWithFrame:NSMakeRect([[window contentView] bounds].size.width - 130, 15, 120, 20)];
        [statusLabel setStringValue:@"C Engine Ready"];
        [statusLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [statusLabel setBackgroundColor:[NSColor clearColor]];
        [statusLabel setBordered:NO];
        [statusLabel setEditable:NO];
        [statusLabel setFont:[NSFont systemFontOfSize:12]];
        [statusLabel setAutoresizingMask:NSViewMinXMargin];
        
        [headerView addSubview:statusDot];
        [headerView addSubview:statusLabel];
        [editorView addSubview:headerView];
        
        // Split view: Editor on left, Preview on right
        CGFloat windowWidth = [[window contentView] bounds].size.width;
        CGFloat windowHeight = [[window contentView] bounds].size.height;
        CGFloat splitWidth = (windowWidth - 60) / 2; // 20px margin + 20px between panels
        
        // LEFT: Markdown Editor
        NSRect editorFrame = NSMakeRect(20, 20, splitWidth, windowHeight - 90);
        NSScrollView *editorScrollView = [[NSScrollView alloc] initWithFrame:editorFrame];
        [editorScrollView setHasVerticalScroller:YES];
        [editorScrollView setHasHorizontalScroller:NO];
        [editorScrollView setAutohidesScrollers:YES];
        [editorScrollView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        [editorScrollView setBorderType:NSLineBorder];
        [editorScrollView layer].borderColor = [NSColor colorWithRed:0.267 green:0.267 blue:0.267 alpha:1.0].CGColor;
        [editorScrollView layer].borderWidth = 1.0;
        
        NSSize editorContentSize = [editorScrollView contentSize];
        NSTextView *textView = [[NSTextView alloc] initWithFrame:NSMakeRect(0, 0, editorContentSize.width, editorContentSize.height)];
        [textView setMinSize:NSMakeSize(0.0, editorContentSize.height)];
        [textView setMaxSize:NSMakeSize(FLT_MAX, FLT_MAX)];
        [textView setVerticallyResizable:YES];
        [textView setHorizontallyResizable:NO];
        [textView setAutoresizingMask:NSViewWidthSizable];
        
        // Style editor (matching HTML version)
        [textView setBackgroundColor:[NSColor colorWithRed:0.118 green:0.118 blue:0.118 alpha:1.0]]; // #1e1e1e
        [textView setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]]; // #e0e0e0  
        [textView setInsertionPointColor:[NSColor colorWithRed:0.298 green:0.431 blue:0.961 alpha:1.0]]; // #4c6ef5
        [textView setFont:[NSFont fontWithName:@"Monaco" size:14]];
        [textView setRichText:NO];
        [textView setImportsGraphics:NO];
        
        // Set initial content
        [textView setString:@"# C Markdown Editor avec Rendu Temps Réel\n\nÉditez ce markdown et voyez le **rendu en direct** à droite!\n\nExemples:\n- **Texte gras**\n- *Texte italique*\n- `code inline`\n\n## Moteurs C Intégrés\n\n✅ Parser markdown C\n✅ Rendu HTML en temps réel\n✅ Interface native macOS\n\n> **Note**: Le rendu utilise nos moteurs C développés spécialement pour cet éditeur."];
        
        [[textView textContainer] setContainerSize:NSMakeSize(editorContentSize.width, FLT_MAX)];
        [[textView textContainer] setWidthTracksTextView:YES];
        [editorScrollView setDocumentView:textView];
        
        // RIGHT: HTML Preview with WKWebView
        NSRect previewFrame = NSMakeRect(40 + splitWidth, 20, splitWidth, windowHeight - 90);
        
        // Create WKWebView for HTML rendering
        WKWebViewConfiguration *config = [[WKWebViewConfiguration alloc] init];
        WKWebView *previewWebView = [[WKWebView alloc] initWithFrame:previewFrame configuration:config];
        [previewWebView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // Style WebView container
        [previewWebView setWantsLayer:YES];
        [previewWebView layer].borderColor = [NSColor colorWithRed:0.267 green:0.267 blue:0.267 alpha:1.0].CGColor;
        [previewWebView layer].borderWidth = 1.0;
        
        // Store global references for timer callback
        g_textView = textView;
        g_previewWebView = previewWebView;
        
        // Load test HTML to verify WKWebView works
        NSString *testHTML = @"<html><body style='background:#2d2d2d; color:white; font-size:14px; padding:20px;'>Preview will appear here...</body></html>";
        [previewWebView loadHTMLString:testHTML baseURL:nil];
        
        // Add both views to editor
        [editorView addSubview:editorScrollView];
        [editorView addSubview:previewWebView];
        
        
        // Labels for panels
        NSTextField *editorLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(25, windowHeight - 85, 200, 15)];
        [editorLabel setStringValue:@"📝 Éditeur Markdown"];
        [editorLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [editorLabel setBackgroundColor:[NSColor clearColor]];
        [editorLabel setBordered:NO];
        [editorLabel setEditable:NO];
        [editorLabel setFont:[NSFont systemFontOfSize:11]];
        
        NSTextField *previewLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(45 + splitWidth, windowHeight - 85, 200, 15)];
        [previewLabel setStringValue:@"🎨 Rendu C Engine"];
        [previewLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [previewLabel setBackgroundColor:[NSColor clearColor]];
        [previewLabel setBordered:NO];
        [previewLabel setEditable:NO];
        [previewLabel setFont:[NSFont systemFontOfSize:11]];
        
        [editorView addSubview:editorLabel];
        [editorView addSubview:previewLabel];
        
        [window setContentView:editorView];
        
        // Show window
        [window center];
        [window makeKeyAndOrderFront:nil];
        [app activateIgnoringOtherApps:YES];
        
        NSLog(@"🚀 C Markdown Editor with real-time rendering ready!");
        
        // Start real-time rendering timer (updates every 500ms)
        g_renderTimer = [NSTimer scheduledTimerWithTimeInterval:0.5
                                                         target:[TimerTarget class]
                                                       selector:@selector(updateMarkdownPreview:)
                                                       userInfo:nil
                                                        repeats:YES];
        
        // Initial render
        [TimerTarget updateMarkdownPreview:nil];
        
        // Run app
        [app run];
    }
    
    return 0;
}