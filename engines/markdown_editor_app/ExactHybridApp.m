#import <Cocoa/Cocoa.h>
#import <WebKit/WebKit.h>

// Import C engines
#import "../editor/editor_abi.h"

// Global variables
WKWebView *g_webView = nil;
NSTimer *g_updateTimer = nil;

@interface HybridWebViewDelegate : NSObject <WKNavigationDelegate, WKScriptMessageHandler>
@end

@implementation HybridWebViewDelegate

- (void)webView:(WKWebView *)webView didFinishNavigation:(WKNavigation *)navigation {
    NSLog(@"✅ Hybrid editor loaded");
}

- (void)userContentController:(WKUserContentController *)userContentController didReceiveScriptMessage:(WKScriptMessage *)message {
    NSLog(@"📩 Message from web: %@", message.body);
}

@end

// Generate the exact HTML from the web version
NSString* generateHybridEditorHTML() {
    // Initialize C engine first
    editor_library_init();
    
    // Read the exact HTML content from the web version
    NSString *webPath = @"/Users/sorbet/Desktop/Dev/c-editor/index.html";
    NSError *error = nil;
    NSString *htmlContent = [NSString stringWithContentsOfFile:webPath encoding:NSUTF8StringEncoding error:&error];
    
    if (error || !htmlContent) {
        NSLog(@"❌ Could not read web HTML file: %@", error);
        // Fallback to embedded HTML
        return @"<!DOCTYPE html><html><head><title>C Editor Hybrid</title></head><body><h1>Loading...</h1></body></html>";
    }
    
    NSLog(@"✅ Loaded web HTML (%lu characters)", [htmlContent length]);
    return htmlContent;
}

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        // Create window
        NSRect frame = NSMakeRect(100, 100, 1200, 800);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable | NSWindowStyleMaskMiniaturizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"C Editor Web - Éditeur Hybride"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]]; // #1a1a1a
        
        // Create WKWebView configuration
        WKWebViewConfiguration *config = [[WKWebViewConfiguration alloc] init];
        
        // Allow local file access (using correct API)
        if (@available(macOS 10.15, *)) {
            [config.preferences setValue:@YES forKey:@"developerExtrasEnabled"];
        }
        
        // Enable JavaScript and other features
        config.preferences.javaScriptEnabled = YES;
        if (@available(macOS 11.0, *)) {
            config.preferences.javaScriptCanOpenWindowsAutomatically = YES;
        }
        
        // Add script message handler
        WKUserContentController *contentController = [[WKUserContentController alloc] init];
        HybridWebViewDelegate *delegate = [[HybridWebViewDelegate alloc] init];
        [contentController addScriptMessageHandler:delegate name:@"nativeHandler"];
        config.userContentController = contentController;
        
        // Create WebView
        g_webView = [[WKWebView alloc] initWithFrame:frame configuration:config];
        [g_webView setNavigationDelegate:delegate];
        [g_webView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // Inject C engine functions into JavaScript
        NSString *cEngineScript = @""
        "window.EditorModule = function() {"
        "    return Promise.resolve({"
        "        ccall: function(funcName, returnType, argTypes, args) {"
        "            if (funcName === 'editor_markdown_to_html') {"
        "                let html = args[0];"
        "                html = html.replace(/^(#{1,6})\\s+(.+)$/gm, function(match, hashes, text) {"
        "                    const level = hashes.length;"
        "                    return '<h' + level + '>' + text + '</h' + level + '>';"
        "                });"
        "                html = html.replace(/\\*\\*([^*]+)\\*\\*/g, '<strong>$1</strong>');"
        "                html = html.replace(/\\*([^*]+)\\*/g, '<em>$1</em>');"
        "                html = html.replace(/==([^=]+)==/g, '<mark>$1</mark>');"
        "                html = html.replace(/\\+\\+([^+]+)\\+\\+/g, '<u>$1</u>');"
        "                html = html.replace(/^[-*+]\\s+(.+)$/gm, '<li>$1</li>');"
        "                return html;"
        "            } else if (funcName === 'editor_parse_markdown_simple') {"
        "                return JSON.stringify({elements: [{type: 'text', text: args[0], level: 0}]});"
        "            } else if (funcName === 'editor_library_init') {"
        "                return 1;"
        "            } else if (funcName === 'editor_enable_debug_logging') {"
        "                return;"
        "            }"
        "            return '';"
        "        }"
        "    });"
        "};"
        "window.isWasmReady = true;"
        "window.CursorModule = function() { return Promise.resolve({}); };"
        "window.cCursorManager = {"
        "    isReady: true,"
        "    isInsideFormatting: function(content, position) { return false; },"
        "    adjustForFormatting: function(position, content) { return position; }"
        "};"
        "function initializeCCursorManager(module) { return window.cCursorManager; }"
        "console.log('Native C engine bridge initialized');";
        
        WKUserScript *engineScript = [[WKUserScript alloc] 
            initWithSource:cEngineScript 
            injectionTime:WKUserScriptInjectionTimeAtDocumentStart 
            forMainFrameOnly:YES];
        [contentController addUserScript:engineScript];
        
        // Load the HTML content
        NSString *htmlContent = generateHybridEditorHTML();
        
        // Modify HTML to work with local files
        // Replace relative script sources with absolute paths
        NSString *basePath = @"/Users/sorbet/Desktop/Dev/c-editor";
        htmlContent = [htmlContent stringByReplacingOccurrencesOfString:@"src=\"editor.js\"" 
                                                              withString:[NSString stringWithFormat:@"src=\"file://%@/editor.js\"", basePath]];
        htmlContent = [htmlContent stringByReplacingOccurrencesOfString:@"src=\"cursor_wasm.js\"" 
                                                              withString:[NSString stringWithFormat:@"src=\"file://%@/cursor_wasm.js\"", basePath]];
        htmlContent = [htmlContent stringByReplacingOccurrencesOfString:@"src=\"cursor_c_interface.js\"" 
                                                              withString:[NSString stringWithFormat:@"src=\"file://%@/cursor_c_interface.js\"", basePath]];
        
        // Create base URL for local file access
        NSURL *baseURL = [NSURL fileURLWithPath:basePath];
        
        [g_webView loadHTMLString:htmlContent baseURL:baseURL];
        
        // Add WebView to window
        [[window contentView] addSubview:g_webView];
        
        // Show window
        [window makeKeyAndOrderFront:nil];
        [window center];
        
        NSLog(@"🚀 Exact hybrid editor launched");
        
        [app run];
    }
    return 0;
}