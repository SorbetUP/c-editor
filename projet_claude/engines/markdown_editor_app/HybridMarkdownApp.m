#import <Cocoa/Cocoa.h>
#import <WebKit/WebKit.h>

// Import C engines
#import "../editor/editor_abi.h"
#import "../markdown/markdown.h"
#import "../cursor/cursor_manager.h"

// Global variables for the hybrid editor
WKWebView *g_hybridWebView = nil;
NSTimer *g_renderTimer = nil;
NSMutableArray *g_lineContents = nil;
NSInteger g_currentLineIndex = -1;

// C engines state
static int c_engines_initialized = 0;

// Forward declarations
NSString* generateHybridEditorHTML(NSArray *lineContents, NSInteger currentLine);
NSString* markdownToHTMLUsingCEngine(NSString *markdown);

// WKWebView bridge for JavaScript communication
@interface WebViewBridge : NSObject <WKScriptMessageHandler>
@end

@implementation WebViewBridge

- (void)userContentController:(WKUserContentController *)userContentController 
      didReceiveScriptMessage:(WKScriptMessage *)message {
    
    NSDictionary *body = message.body;
    NSString *action = body[@"action"];
    
    NSLog(@"🌐 JS Bridge: %@", action);
    
    if ([action isEqualToString:@"lineClicked"]) {
        NSInteger newLineIndex = [body[@"lineIndex"] integerValue];
        NSLog(@"👆 Line clicked: %ld", newLineIndex);
        [self handleLineClick:newLineIndex];
        
    } else if ([action isEqualToString:@"contentChanged"]) {
        NSInteger lineIndex = [body[@"lineIndex"] integerValue];
        NSString *content = body[@"content"];
        NSLog(@"📝 Content changed: line %ld = %@", lineIndex, content);
        [self handleContentChange:lineIndex content:content];
        
    } else if ([action isEqualToString:@"renderMarkdown"]) {
        NSString *markdown = body[@"markdown"];
        NSString *html = [self renderMarkdownWithCEngine:markdown];
        
        // Send HTML back to JavaScript
        NSString *js = [NSString stringWithFormat:@"window.handleRenderedHTML('%@');", 
                        [self escapeForJavaScript:html]];
        dispatch_async(dispatch_get_main_queue(), ^{
            [g_hybridWebView evaluateJavaScript:js completionHandler:nil];
        });
    }
}

- (void)handleLineClick:(NSInteger)newLineIndex {
    if (newLineIndex == g_currentLineIndex) {
        return; // Same line, no change needed
    }
    
    // Update current line index
    NSInteger previousIndex = g_currentLineIndex;
    g_currentLineIndex = newLineIndex;
    
    NSLog(@"🔄 Line switch: %ld -> %ld", previousIndex, g_currentLineIndex);
    
    // Update the webview to reflect the change
    [self updateHybridDisplay];
}

- (void)handleContentChange:(NSInteger)lineIndex content:(NSString *)content {
    // Update line content
    if (lineIndex >= 0 && lineIndex < [g_lineContents count]) {
        g_lineContents[lineIndex] = content;
        NSLog(@"💾 Updated line %ld: %@", lineIndex, content);
    }
}

- (NSString *)renderMarkdownWithCEngine:(NSString *)markdown {
    if (!c_engines_initialized) {
        return [markdown copy];
    }
    
    const char *markdown_cstr = [markdown UTF8String];
    const char *html_cstr = editor_markdown_to_html(markdown_cstr);
    
    if (!html_cstr) {
        return [markdown copy];
    }
    
    NSString *html = [NSString stringWithUTF8String:html_cstr];
    NSLog(@"🔧 C Engine: %@ -> %@", markdown, html);
    return html;
}

- (NSString *)escapeForJavaScript:(NSString *)string {
    NSString *escaped = [string stringByReplacingOccurrencesOfString:@"\\" withString:@"\\\\"];
    escaped = [escaped stringByReplacingOccurrencesOfString:@"'" withString:@"\\'"];
    escaped = [escaped stringByReplacingOccurrencesOfString:@"\n" withString:@"\\n"];
    escaped = [escaped stringByReplacingOccurrencesOfString:@"\r" withString:@"\\r"];
    return escaped;
}

- (void)updateHybridDisplay {
    // Generate the updated HTML and send to WebView
    NSString *completeHTML = [self generateHybridHTML];
    
    dispatch_async(dispatch_get_main_queue(), ^{
        [g_hybridWebView loadHTMLString:completeHTML baseURL:nil];
    });
}

- (NSString *)generateHybridHTML {
    return generateHybridEditorHTML(g_lineContents, g_currentLineIndex);
}

@end

// Initialize C engines
BOOL initializeCEngines(void) {
    if (c_engines_initialized) {
        return YES;
    }
    
    NSLog(@"🔧 Initializing C engines...");
    
    // Initialize editor engine
    int editor_result = editor_library_init();
    if (editor_result != 0) {
        NSLog(@"❌ Editor engine initialization failed: %d", editor_result);
        return NO;
    }
    NSLog(@"✅ Editor engine initialized");
    
    // Enable debug logging
    editor_enable_debug_logging(true);
    
    // Test markdown engine
    const char *test_html = editor_markdown_to_html("**test**");
    if (test_html) {
        NSLog(@"✅ Markdown engine test: %s", test_html);
        // Note: editor_markdown_to_html returns const char*, no need to free
    }
    
    // Cursor manager is header-only, no initialization needed
    NSLog(@"✅ Cursor manager available");
    
    c_engines_initialized = 1;
    return YES;
}

// Convert markdown to HTML using C engine
NSString* markdownToHTMLUsingCEngine(NSString *markdown) {
    if (!c_engines_initialized) {
        NSLog(@"❌ C engines not initialized");
        return [markdown copy];
    }
    
    const char *markdown_cstr = [markdown UTF8String];
    const char *html_cstr = editor_markdown_to_html(markdown_cstr);
    
    if (!html_cstr) {
        NSLog(@"❌ C markdown engine returned NULL for: %@", markdown);
        return [markdown copy];
    }
    
    NSString *html = [NSString stringWithUTF8String:html_cstr];
    // Note: const char* returned by C engine, no need to free
    
    NSLog(@"✅ C engine: \"%@\" -> \"%@\"", 
          [markdown substringToIndex:MIN(30, [markdown length])],
          [html substringToIndex:MIN(50, [html length])]);
    
    return html;
}

// Generate complete HTML for line rendering
NSString* generateLineHTML(NSString *content, BOOL isCurrentLine) {
    NSString *htmlContent;
    
    if (isCurrentLine) {
        // Show raw markdown for current line
        htmlContent = [NSString stringWithFormat:@"<span class=\"markdown-text\">%@</span>", 
                       content.length > 0 ? content : @""];
    } else {
        // Render using C engine for non-current lines
        if (content.length == 0) {
            htmlContent = @"&nbsp;";
        } else {
            htmlContent = markdownToHTMLUsingCEngine(content);
        }
    }
    
    NSString *lineClass = isCurrentLine ? @"current-line" : @"rendered-line";
    
    return [NSString stringWithFormat:@
        "<div class=\"editor-line %@\" contenteditable=\"true\" spellcheck=\"false\">"
        "%@"
        "</div>", lineClass, htmlContent];
}

// Generate complete HTML page with JavaScript interaction
NSString* generateHybridEditorHTML(NSArray *lineContents, NSInteger currentLine) {
    NSMutableString *linesHTML = [[NSMutableString alloc] init];
    
    for (NSInteger i = 0; i < [lineContents count]; i++) {
        NSString *content = lineContents[i];
        BOOL isCurrent = (i == currentLine);
        NSString *lineClass = isCurrent ? @"current-line" : @"rendered-line";
        
        if (isCurrent) {
            // Show raw markdown for current line
            [linesHTML appendFormat:@"<div class=\"editor-line %@\" contenteditable=\"true\" data-line=\"%ld\" spellcheck=\"false\">%@</div>\n", 
             lineClass, i, content];
        } else {
            // Show rendered HTML for non-current lines
            NSString *renderedHTML = markdownToHTMLUsingCEngine(content);
            if (renderedHTML.length == 0) {
                renderedHTML = @"&nbsp;";
            }
            [linesHTML appendFormat:@"<div class=\"editor-line %@\" contenteditable=\"true\" data-line=\"%ld\" spellcheck=\"false\">%@</div>\n", 
             lineClass, i, renderedHTML];
        }
    }
    
    NSString *completeHTML = [NSString stringWithFormat:@"<!DOCTYPE html>\n"
        "<html lang=\"fr\">\n"
        "<head>\n"
        "    <meta charset=\"UTF-8\">\n"
        "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n"
        "    <title>C Hybrid Editor</title>\n"
        "    <style>\n"
        "        * {\n"
        "            margin: 0;\n"
        "            padding: 0;\n"
        "            box-sizing: border-box;\n"
        "        }\n"
        "\n"
        "        body {\n"
        "            background: #1a1a1a;\n"
        "            color: #e0e0e0;\n"
        "            font-family: 'Monaco', 'Consolas', monospace;\n"
        "            height: 100vh;\n"
        "            padding: 20px;\n"
        "            overflow: hidden;\n"
        "        }\n"
        "\n"
        "        .editor-line {\n"
        "            min-height: 1.8em;\n"
        "            padding: 4px 8px;\n"
        "            margin: 0;\n"
        "            border-radius: 3px;\n"
        "            transition: background-color 0.2s ease;\n"
        "            cursor: text;\n"
        "            font-size: 14px;\n"
        "            line-height: 1.8;\n"
        "            outline: none;\n"
        "            border: none;\n"
        "        }\n"
        "\n"
        "        .editor-line.current-line {\n"
        "            background-color: rgba(76, 110, 245, 0.1);\n"
        "            border-left: 3px solid #4c6ef5;\n"
        "            padding-left: 15px;\n"
        "        }\n"
        "\n"
        "        .editor-line.rendered-line {\n"
        "            background-color: rgba(255, 255, 255, 0.02);\n"
        "        }\n"
        "\n"
        "        .editor-line:empty::before {\n"
        "            content: '\\200B';\n"
        "            color: transparent;\n"
        "        }\n"
        "\n"
        "        .markdown-text {\n"
        "            color: #e0e0e0;\n"
        "        }\n"
        "\n"
        "        /* Rendered HTML styling */\n"
        "        .editor-line h1, .editor-line h2, .editor-line h3, .editor-line h4, .editor-line h5, .editor-line h6 {\n"
        "            margin: 0;\n"
        "            color: #4c6ef5;\n"
        "            display: inline;\n"
        "        }\n"
        "\n"
        "        .editor-line h1 { font-size: 24px; }\n"
        "        .editor-line h2 { font-size: 20px; }\n"
        "        .editor-line h3 { font-size: 18px; }\n"
        "\n"
        "        .editor-line strong {\n"
        "            color: #51cf66;\n"
        "            font-weight: bold;\n"
        "        }\n"
        "\n"
        "        .editor-line em {\n"
        "            color: #ffd43b;\n"
        "            font-style: italic;\n"
        "        }\n"
        "\n"
        "        .editor-line u {\n"
        "            text-decoration: underline;\n"
        "            color: #ff6b6b;\n"
        "        }\n"
        "\n"
        "        .editor-line mark {\n"
        "            background: #ffd43b;\n"
        "            color: #000;\n"
        "            padding: 1px 2px;\n"
        "        }\n"
        "\n"
        "        .editor-line code {\n"
        "            background: #2d2d2d;\n"
        "            color: #ff6b6b;\n"
        "            padding: 2px 6px;\n"
        "            border-radius: 3px;\n"
        "            font-family: 'Monaco', monospace;\n"
        "        }\n"
        "    </style>\n"
        "</head>\n"
        "<body>\n"
        "    <div id=\"hybridEditor\">\n"
        "%@\n"
        "    </div>\n"
        "    <script>\n"
        "        let currentLineIndex = %ld;\n"
        "        let editorLines = [];\n"
        "        let lineContents = [];\n"
        "\n"
        "        function initializeEditor() {\n"
        "            const editor = document.getElementById('hybridEditor');\n"
        "            editorLines = Array.from(editor.querySelectorAll('.editor-line'));\n"
        "            \n"
        "            // Initialize line contents\n"
        "            editorLines.forEach((line, index) => {\n"
        "                lineContents[index] = line.textContent || '';\n"
        "            });\n"
        "            \n"
        "            // Add event listeners\n"
        "            editor.addEventListener('click', handleClick);\n"
        "            editor.addEventListener('keyup', handleKeyUp);\n"
        "            editor.addEventListener('input', handleInput);\n"
        "            \n"
        "            // Focus current line\n"
        "            if (editorLines[currentLineIndex]) {\n"
        "                editorLines[currentLineIndex].focus();\n"
        "            }\n"
        "        }\n"
        "\n"
        "        function handleClick(e) {\n"
        "            const clickedLine = e.target.closest('.editor-line');\n"
        "            if (clickedLine) {\n"
        "                const lineIndex = parseInt(clickedLine.dataset.line);\n"
        "                if (lineIndex !== currentLineIndex) {\n"
        "                    console.log('Line clicked:', lineIndex);\n"
        "                    window.webkit.messageHandlers.hybrid.postMessage({\n"
        "                        action: 'lineClicked',\n"
        "                        lineIndex: lineIndex\n"
        "                    });\n"
        "                }\n"
        "            }\n"
        "        }\n"
        "\n"
        "        function handleKeyUp(e) {\n"
        "            const currentLine = e.target.closest('.editor-line');\n"
        "            if (currentLine && currentLine.classList.contains('current-line')) {\n"
        "                const lineIndex = parseInt(currentLine.dataset.line);\n"
        "                const content = currentLine.textContent || '';\n"
        "                \n"
        "                if (lineContents[lineIndex] !== content) {\n"
        "                    lineContents[lineIndex] = content;\n"
        "                    window.webkit.messageHandlers.hybrid.postMessage({\n"
        "                        action: 'contentChanged',\n"
        "                        lineIndex: lineIndex,\n"
        "                        content: content\n"
        "                    });\n"
        "                }\n"
        "            }\n"
        "        }\n"
        "\n"
        "        function handleInput(e) {\n"
        "            const currentLine = e.target.closest('.editor-line');\n"
        "            if (currentLine && currentLine.classList.contains('current-line')) {\n"
        "                const lineIndex = parseInt(currentLine.dataset.line);\n"
        "                const content = currentLine.textContent || '';\n"
        "                lineContents[lineIndex] = content;\n"
        "            }\n"
        "        }\n"
        "\n"
        "        // Initialize when page loads\n"
        "        document.addEventListener('DOMContentLoaded', initializeEditor);\n"
        "    </script>\n"
        "</body>\n"
        "</html>", linesHTML, currentLine];
    
    return completeHTML;
}

// Convert HTML cursor position to Markdown using C engine
NSInteger htmlPositionToMarkdown(NSInteger htmlPos, NSString *markdownText) {
    if (!c_engines_initialized || !markdownText) {
        return htmlPos;
    }
    
    const char *markdown_cstr = [markdownText UTF8String];
    cursor_position_t result = cursor_html_to_markdown((int)htmlPos, markdown_cstr);
    
    if (result.is_valid) {
        NSLog(@"🎯 C Cursor: HTML %ld -> MD %d", htmlPos, result.position);
        return result.position;
    } else {
        NSLog(@"⚠️ C Cursor: Invalid conversion, using fallback");
        return htmlPos;
    }
}

// Convert Markdown cursor position to HTML (simplified - bidirectional conversion not available)
NSInteger markdownPositionToHTML(NSInteger mdPos, NSString *markdownText) {
    // For now, use the HTML to markdown conversion in reverse
    // This is a limitation until we implement the reverse function in C
    NSLog(@"🔄 C Cursor: MD->HTML conversion (simplified) %ld", mdPos);
    return mdPos; // Fallback to direct mapping
}

// Adjust cursor position for formatting using C engine
NSInteger adjustCursorForFormatting(NSInteger position, NSString *content) {
    if (!c_engines_initialized || !content) {
        return position;
    }
    
    const char *content_cstr = [content UTF8String];
    cursor_position_t result = cursor_adjust_for_formatting((int)position, content_cstr, true);
    
    if (result.is_valid && result.position != (int)position) {
        NSLog(@"🔧 C Cursor: Position adjusted %ld -> %d", position, result.position);
        return result.position;
    }
    
    return position;
}

// Analyze formatting at cursor position using C engine
NSString* analyzeFormattingAtPosition(NSInteger position, NSString *content) {
    if (!c_engines_initialized || !content) {
        return @"NONE";
    }
    
    const char *content_cstr = [content UTF8String];
    formatting_context_t context = cursor_analyze_formatting(content_cstr, (int)position);
    
    switch (context.type) {
        case MARKER_BOLD: return @"BOLD";
        case MARKER_ITALIC: return @"ITALIC";
        case MARKER_HIGHLIGHT: return @"HIGHLIGHT";
        case MARKER_UNDERLINE: return @"UNDERLINE";
        case MARKER_HEADER: return @"HEADER";
        default: return @"NONE";
    }
}

// Timer callback for hybrid editor updates with C cursor management
@interface HybridTimerTarget : NSObject
+ (void)updateHybridEditor:(NSTimer *)timer;
@end

@implementation HybridTimerTarget
+ (void)updateHybridEditor:(NSTimer *)timer {
    // For now, just log that the system is running
    NSLog(@"🔄 Hybrid editor with C engines active (lines: %lu, current: %ld)", 
          [g_lineContents count], g_currentLineIndex);
    
    // Test C cursor functions if we have content
    if ([g_lineContents count] > 0 && g_currentLineIndex >= 0) {
        NSString *currentContent = g_lineContents[g_currentLineIndex];
        if (currentContent.length > 0) {
            // Test cursor conversion
            NSInteger testPos = currentContent.length / 2;
            NSInteger htmlPos = markdownPositionToHTML(testPos, currentContent);
            NSInteger backToMd = htmlPositionToMarkdown(htmlPos, currentContent);
            
            // Test formatting analysis
            NSString *formatting = analyzeFormattingAtPosition(testPos, currentContent);
            
            NSLog(@"🧪 C Cursor Test: MD %ld <-> HTML %ld <-> MD %ld, Format: %@", 
                  testPos, htmlPos, backToMd, formatting);
        }
    }
}
@end

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        // Initialize C engines first
        if (!initializeCEngines()) {
            NSLog(@"❌ Failed to initialize C engines, exiting");
            return 1;
        }
        
        // Create window matching web version layout
        NSRect frame = NSMakeRect(100, 100, 1200, 800);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"C Editor Web - Éditeur Hybride"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]];
        
        // Create main container view
        NSView *mainView = [[NSView alloc] initWithFrame:[[window contentView] bounds]];
        [mainView setWantsLayer:YES];
        [mainView layer].backgroundColor = [NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0].CGColor;
        [mainView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // Header (matching web version)
        CGFloat windowWidth = [[window contentView] bounds].size.width;
        CGFloat windowHeight = [[window contentView] bounds].size.height;
        
        NSView *headerView = [[NSView alloc] initWithFrame:NSMakeRect(0, windowHeight - 50, windowWidth, 50)];
        [headerView setWantsLayer:YES];
        [headerView layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor;
        [headerView setAutoresizingMask:NSViewWidthSizable | NSViewMinYMargin];
        
        // Header title
        NSTextField *titleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 15, 400, 20)];
        [titleLabel setStringValue:@"C Editor Web - Éditeur Hybride"];
        [titleLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [titleLabel setBackgroundColor:[NSColor clearColor]];
        [titleLabel setBordered:NO];
        [titleLabel setEditable:NO];
        [titleLabel setFont:[NSFont systemFontOfSize:16 weight:NSFontWeightSemibold]];
        [headerView addSubview:titleLabel];
        
        // Status indicator
        NSView *statusDot = [[NSView alloc] initWithFrame:NSMakeRect(windowWidth - 200, 21, 8, 8)];
        [statusDot setWantsLayer:YES];
        [statusDot layer].backgroundColor = [NSColor colorWithRed:0.318 green:0.812 blue:0.400 alpha:1.0].CGColor;
        [statusDot layer].cornerRadius = 4.0;
        [statusDot setAutoresizingMask:NSViewMinXMargin];
        
        NSTextField *statusLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(windowWidth - 180, 15, 150, 20)];
        [statusLabel setStringValue:@"WASM + C Cursor Ready"];
        [statusLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [statusLabel setBackgroundColor:[NSColor clearColor]];
        [statusLabel setBordered:NO];
        [statusLabel setEditable:NO];
        [statusLabel setFont:[NSFont systemFontOfSize:12]];
        [statusLabel setAutoresizingMask:NSViewMinXMargin];
        
        [headerView addSubview:statusDot];
        [headerView addSubview:statusLabel];
        [mainView addSubview:headerView];
        
        // Hybrid editor container (matching web version)
        NSView *editorContainer = [[NSView alloc] initWithFrame:NSMakeRect(0, 40, windowWidth, windowHeight - 90)];
        [editorContainer setWantsLayer:YES];
        [editorContainer layer].backgroundColor = [NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0].CGColor;
        [editorContainer setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // Pane header
        NSView *paneHeader = [[NSView alloc] initWithFrame:NSMakeRect(0, windowHeight - 130, windowWidth, 40)];
        [paneHeader setWantsLayer:YES];
        [paneHeader layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor;
        [paneHeader setAutoresizingMask:NSViewWidthSizable | NSViewMinYMargin];
        
        NSTextField *paneLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 10, windowWidth - 40, 20)];
        [paneLabel setStringValue:@"Éditeur Hybride - Ligne courante en Markdown, autres lignes rendues"];
        [paneLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [paneLabel setBackgroundColor:[NSColor clearColor]];
        [paneLabel setBordered:NO];
        [paneLabel setEditable:NO];
        [paneLabel setFont:[NSFont systemFontOfSize:14 weight:NSFontWeightSemibold]];
        [paneLabel setAutoresizingMask:NSViewWidthSizable];
        [paneHeader addSubview:paneLabel];
        [editorContainer addSubview:paneHeader];
        
        // Hybrid editor using WKWebView with JavaScript bridge
        CGFloat editorY = 20;
        CGFloat editorHeight = windowHeight - 150;
        NSRect hybridFrame = NSMakeRect(20, editorY, windowWidth - 40, editorHeight);
        
        // Configure WebView with message handler
        WKWebViewConfiguration *config = [[WKWebViewConfiguration alloc] init];
        WebViewBridge *bridge = [[WebViewBridge alloc] init];
        [config.userContentController addScriptMessageHandler:bridge name:@"hybrid"];
        
        WKWebView *hybridWebView = [[WKWebView alloc] initWithFrame:hybridFrame configuration:config];
        g_hybridWebView = hybridWebView;
        [hybridWebView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // Style WebView
        [hybridWebView setWantsLayer:YES];
        [hybridWebView layer].borderColor = [NSColor colorWithRed:0.267 green:0.267 blue:0.267 alpha:1.0].CGColor;
        [hybridWebView layer].borderWidth = 1.0;
        [hybridWebView layer].cornerRadius = 4.0;
        
        [editorContainer addSubview:hybridWebView];
        [mainView addSubview:editorContainer];
        
        // Footer
        NSView *footerView = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, windowWidth, 40)];
        [footerView setWantsLayer:YES];
        [footerView layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor;
        [footerView setAutoresizingMask:NSViewWidthSizable | NSViewMaxYMargin];
        
        NSTextField *footerLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(0, 10, windowWidth, 20)];
        [footerLabel setStringValue:@"C Editor Web - Powered by Native C Engines | GitHub: SorbetUP/c-editor"];
        [footerLabel setTextColor:[NSColor colorWithRed:0.533 green:0.533 blue:0.533 alpha:1.0]];
        [footerLabel setBackgroundColor:[NSColor clearColor]];
        [footerLabel setBordered:NO];
        [footerLabel setEditable:NO];
        [footerLabel setFont:[NSFont systemFontOfSize:12]];
        [footerLabel setAlignment:NSTextAlignmentCenter];
        [footerLabel setAutoresizingMask:NSViewWidthSizable];
        [footerView addSubview:footerLabel];
        [mainView addSubview:footerView];
        
        // Initialize line arrays
        g_lineContents = [[NSMutableArray alloc] initWithArray:@[
            @"# C Editor Web - Éditeur Hybride",
            @"",
            @"Écrivez votre markdown ici...",
            @"",
            @"Exemple:",
            @"- **Gras**",
            @"- *Italique*",
            @"- ==Surligné==",
            @"- ++Souligné++"
        ]];
        g_currentLineIndex = 0;
        
        // Load initial HTML
        NSString *initialHTML = generateHybridEditorHTML(g_lineContents, g_currentLineIndex);
        [hybridWebView loadHTMLString:initialHTML baseURL:nil];
        
        [window setContentView:mainView];
        
        // Show window
        [window center];
        [window makeKeyAndOrderFront:nil];
        [app activateIgnoringOtherApps:YES];
        
        NSLog(@"🚀 C Hybrid Editor launched successfully!");
        
        // Start timer for updates
        g_renderTimer = [NSTimer scheduledTimerWithTimeInterval:0.5
                                                         target:[HybridTimerTarget class]
                                                       selector:@selector(updateHybridEditor:)
                                                       userInfo:nil
                                                        repeats:YES];
        
        // Run app
        [app run];
    }
    
    return 0;
}