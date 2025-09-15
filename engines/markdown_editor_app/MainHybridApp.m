#import <Cocoa/Cocoa.h>
#import <WebKit/WebKit.h>

// Import C engines
#import "../editor/editor_abi.h"
#import "../markdown/markdown.h"
#import "../cursor/cursor_manager.h"

// Global variables
WKWebView *g_webView = nil;
static int c_engines_initialized = 0;

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
    }
    
    NSLog(@"✅ Cursor manager available");
    
    c_engines_initialized = 1;
    return YES;
}

// Bridge for JavaScript to call C engines
@interface CEngineBridge : NSObject <WKScriptMessageHandler>
@end

@implementation CEngineBridge

- (void)userContentController:(WKUserContentController *)userContentController 
      didReceiveScriptMessage:(WKScriptMessage *)message {
    
    NSDictionary *body = message.body;
    NSString *action = body[@"action"];
    
    if ([action isEqualToString:@"renderMarkdown"]) {
        NSString *markdown = body[@"markdown"];
        NSString *html = [self renderWithCEngine:markdown];
        
        // Send HTML back to JavaScript
        NSString *js = [NSString stringWithFormat:@"window.receiveRenderedHTML('%@');", 
                        [self escapeForJS:html]];
        dispatch_async(dispatch_get_main_queue(), ^{
            [g_webView evaluateJavaScript:js completionHandler:nil];
        });
    }
}

- (NSString *)renderWithCEngine:(NSString *)markdown {
    if (!c_engines_initialized || !markdown) {
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

- (NSString *)escapeForJS:(NSString *)string {
    NSString *escaped = [string stringByReplacingOccurrencesOfString:@"\\" withString:@"\\\\"];
    escaped = [escaped stringByReplacingOccurrencesOfString:@"'" withString:@"\\'"];
    escaped = [escaped stringByReplacingOccurrencesOfString:@"\n" withString:@"\\n"];
    escaped = [escaped stringByReplacingOccurrencesOfString:@"\r" withString:@"\\r"];
    return escaped;
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
        
        // Header
        CGFloat windowWidth = [[window contentView] bounds].size.width;
        CGFloat windowHeight = [[window contentView] bounds].size.height;
        
        NSView *headerView = [[NSView alloc] initWithFrame:NSMakeRect(0, windowHeight - 50, windowWidth, 50)];
        [headerView setWantsLayer:YES];
        [headerView layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor;
        [headerView setAutoresizingMask:NSViewWidthSizable | NSViewMinYMargin];
        
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
        [statusLabel setStringValue:@"C Engines Ready"];
        [statusLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [statusLabel setBackgroundColor:[NSColor clearColor]];
        [statusLabel setBordered:NO];
        [statusLabel setEditable:NO];
        [statusLabel setFont:[NSFont systemFontOfSize:12]];
        [statusLabel setAutoresizingMask:NSViewMinXMargin];
        
        [headerView addSubview:statusDot];
        [headerView addSubview:statusLabel];
        [mainView addSubview:headerView];
        
        // Editor container
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
        [paneLabel setStringValue:@"Éditeur Hybride - Ligne courante en Markdown, autres lignes rendues avec moteurs C"];
        [paneLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [paneLabel setBackgroundColor:[NSColor clearColor]];
        [paneLabel setBordered:NO];
        [paneLabel setEditable:NO];
        [paneLabel setFont:[NSFont systemFontOfSize:14 weight:NSFontWeightSemibold]];
        [paneLabel setAutoresizingMask:NSViewWidthSizable];
        [paneHeader addSubview:paneLabel];
        [editorContainer addSubview:paneHeader];
        
        // WebView with C engine bridge
        CGFloat editorY = 20;
        CGFloat editorHeight = windowHeight - 150;
        NSRect webFrame = NSMakeRect(20, editorY, windowWidth - 40, editorHeight);
        
        WKWebViewConfiguration *config = [[WKWebViewConfiguration alloc] init];
        CEngineBridge *bridge = [[CEngineBridge alloc] init];
        [config.userContentController addScriptMessageHandler:bridge name:@"cengine"];
        
        WKWebView *webView = [[WKWebView alloc] initWithFrame:webFrame configuration:config];
        g_webView = webView;
        [webView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        [webView setWantsLayer:YES];
        [webView layer].borderColor = [NSColor colorWithRed:0.267 green:0.267 blue:0.267 alpha:1.0].CGColor;
        [webView layer].borderWidth = 1.0;
        [webView layer].cornerRadius = 4.0;
        
        // HTML with working logic from test + C engine integration
        NSString *htmlContent = @"<!DOCTYPE html>\n"
        "<html lang=\"fr\">\n"
        "<head>\n"
        "    <meta charset=\"UTF-8\">\n"
        "    <style>\n"
        "        body {\n"
        "            background: #1a1a1a;\n"
        "            color: #e0e0e0;\n"
        "            font-family: Monaco, monospace;\n"
        "            padding: 20px;\n"
        "            font-size: 14px;\n"
        "            line-height: 1.6;\n"
        "            margin: 0;\n"
        "        }\n"
        "        .editor-line {\n"
        "            min-height: 1.8em;\n"
        "            padding: 8px 12px;\n"
        "            margin: 2px 0;\n"
        "            border-radius: 4px;\n"
        "            cursor: text;\n"
        "            transition: all 0.2s ease;\n"
        "        }\n"
        "        .editor-line.current-line {\n"
        "            background-color: rgba(76, 110, 245, 0.15);\n"
        "            border-left: 4px solid #4c6ef5;\n"
        "            padding-left: 16px;\n"
        "        }\n"
        "        .editor-line.rendered-line {\n"
        "            background-color: rgba(255, 255, 255, 0.03);\n"
        "        }\n"
        "        h1 { color: #4c6ef5; font-size: 24px; margin: 0; }\n"
        "        h2 { color: #4c6ef5; font-size: 20px; margin: 0; }\n"
        "        h3 { color: #4c6ef5; font-size: 18px; margin: 0; }\n"
        "        strong { color: #51cf66; font-weight: bold; }\n"
        "        em { color: #ffd43b; font-style: italic; }\n"
        "        u { color: #ff6b6b; text-decoration: underline; }\n"
        "        mark { background: #ffd43b; color: #000; padding: 2px 4px; border-radius: 2px; }\n"
        "    </style>\n"
        "</head>\n"
        "<body>\n"
        "    <div id=\"editor\" contenteditable=\"true\" spellcheck=\"false\">\n"
        "        <div class=\"editor-line current-line\" data-line=\"0\"># C Editor Web - Éditeur Hybride</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"1\"></div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"2\">Écrivez votre markdown ici...</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"3\"></div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"4\">Exemple:</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"5\">- **Gras**</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"6\">- *Italique*</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"7\">- ==Surligné==</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"8\">- ++Souligné++</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"9\"></div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"10\">## Moteurs C Intégrés</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"11\">- ✅ Parser markdown C</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"12\">- ✅ Rendu HTML en temps réel</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"13\">- ✅ Interface native macOS</div>\n"
        "    </div>\n"
        "    \n"
        "    <script>\n"
        "        let currentLineIndex = 0;\n"
        "        const editor = document.getElementById('editor');\n"
        "        const lines = Array.from(editor.querySelectorAll('.editor-line'));\n"
        "        const lineContents = [];\n"
        "        let pendingRenders = new Set();\n"
        "        \n"
        "        // Store initial content\n"
        "        lines.forEach((line, index) => {\n"
        "            lineContents[index] = line.textContent || '';\n"
        "        });\n"
        "        \n"
        "        // Fallback markdown converter (if C engine fails)\n"
        "        function fallbackMarkdownToHtml(text) {\n"
        "            if (!text || !text.trim()) return '&nbsp;';\n"
        "            \n"
        "            let html = text;\n"
        "            html = html.replace(/^# (.+)$/gm, '<h1>$1</h1>');\n"
        "            html = html.replace(/^## (.+)$/gm, '<h2>$1</h2>');\n"
        "            html = html.replace(/^### (.+)$/gm, '<h3>$1</h3>');\n"
        "            html = html.replace(/\\*\\*([^*]+)\\*\\*/g, '<strong>$1</strong>');\n"
        "            html = html.replace(/\\*([^*]+)\\*/g, '<em>$1</em>');\n"
        "            html = html.replace(/==([^=]+)==/g, '<mark>$1</mark>');\n"
        "            html = html.replace(/\\+\\+([^+]+)\\+\\+/g, '<u>$1</u>');\n"
        "            \n"
        "            return html;\n"
        "        }\n"
        "        \n"
        "        // Render using C engine\n"
        "        function renderLineWithCEngine(index) {\n"
        "            if (!lines[index] || pendingRenders.has(index)) return;\n"
        "            \n"
        "            const content = lineContents[index] || '';\n"
        "            if (!content.trim()) {\n"
        "                lines[index].innerHTML = '&nbsp;';\n"
        "                return;\n"
        "            }\n"
        "            \n"
        "            pendingRenders.add(index);\n"
        "            console.log('Requesting C engine render for line', index, ':', content);\n"
        "            \n"
        "            // Request C engine rendering\n"
        "            window.webkit.messageHandlers.cengine.postMessage({\n"
        "                action: 'renderMarkdown',\n"
        "                markdown: content,\n"
        "                lineIndex: index\n"
        "            });\n"
        "            \n"
        "            // Fallback timeout\n"
        "            setTimeout(() => {\n"
        "                if (pendingRenders.has(index)) {\n"
        "                    console.log('C engine timeout for line', index, ', using fallback');\n"
        "                    const fallbackHtml = fallbackMarkdownToHtml(content);\n"
        "                    lines[index].innerHTML = fallbackHtml;\n"
        "                    pendingRenders.delete(index);\n"
        "                }\n"
        "            }, 100);\n"
        "        }\n"
        "        \n"
        "        // Receive rendered HTML from C engine\n"
        "        window.receiveRenderedHTML = function(html) {\n"
        "            // Find which line this is for (simple approach)\n"
        "            for (let index of pendingRenders) {\n"
        "                if (lines[index]) {\n"
        "                    console.log('C engine rendered line', index, ':', html);\n"
        "                    lines[index].innerHTML = html;\n"
        "                    pendingRenders.delete(index);\n"
        "                    break;\n"
        "                }\n"
        "            }\n"
        "        };\n"
        "        \n"
        "        function showLineAsMarkdown(index) {\n"
        "            if (!lines[index]) return;\n"
        "            const content = lineContents[index] || '';\n"
        "            lines[index].innerHTML = '';\n"
        "            lines[index].textContent = content;\n"
        "        }\n"
        "        \n"
        "        function switchToLine(newIndex) {\n"
        "            if (newIndex === currentLineIndex || !lines[newIndex]) return;\n"
        "            \n"
        "            console.log('Switching from line', currentLineIndex, 'to line', newIndex);\n"
        "            \n"
        "            // Save current line content\n"
        "            if (lines[currentLineIndex]) {\n"
        "                lineContents[currentLineIndex] = lines[currentLineIndex].textContent || '';\n"
        "            }\n"
        "            \n"
        "            // Render old current line with C engine\n"
        "            if (lines[currentLineIndex]) {\n"
        "                lines[currentLineIndex].classList.remove('current-line');\n"
        "                lines[currentLineIndex].classList.add('rendered-line');\n"
        "                renderLineWithCEngine(currentLineIndex);\n"
        "            }\n"
        "            \n"
        "            // Show new line as markdown\n"
        "            lines[newIndex].classList.remove('rendered-line');\n"
        "            lines[newIndex].classList.add('current-line');\n"
        "            showLineAsMarkdown(newIndex);\n"
        "            \n"
        "            currentLineIndex = newIndex;\n"
        "        }\n"
        "        \n"
        "        // Event listeners\n"
        "        editor.addEventListener('click', function(e) {\n"
        "            let clickedLine = e.target;\n"
        "            if (!clickedLine.classList.contains('editor-line')) {\n"
        "                clickedLine = clickedLine.closest('.editor-line');\n"
        "            }\n"
        "            \n"
        "            if (clickedLine && clickedLine.dataset.line) {\n"
        "                const lineIndex = parseInt(clickedLine.dataset.line);\n"
        "                switchToLine(lineIndex);\n"
        "            }\n"
        "        });\n"
        "        \n"
        "        editor.addEventListener('keyup', function(e) {\n"
        "            if (lines[currentLineIndex]) {\n"
        "                lineContents[currentLineIndex] = lines[currentLineIndex].textContent || '';\n"
        "            }\n"
        "        });\n"
        "        \n"
        "        // Initialize - render all non-current lines with C engine\n"
        "        lines.forEach((line, index) => {\n"
        "            if (index !== currentLineIndex) {\n"
        "                renderLineWithCEngine(index);\n"
        "            } else {\n"
        "                showLineAsMarkdown(index);\n"
        "            }\n"
        "        });\n"
        "        \n"
        "        setTimeout(() => {\n"
        "            if (lines[0]) lines[0].focus();\n"
        "        }, 500);\n"
        "    </script>\n"
        "</body>\n"
        "</html>";
        
        [webView loadHTMLString:htmlContent baseURL:nil];
        [editorContainer addSubview:webView];
        [mainView addSubview:editorContainer];
        
        // Footer
        NSView *footerView = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, windowWidth, 40)];
        [footerView setWantsLayer:YES];
        [footerView layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor;
        [footerView setAutoresizingMask:NSViewWidthSizable | NSViewMaxYMargin];
        
        NSTextField *footerLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(0, 10, windowWidth, 20)];
        [footerLabel setStringValue:@"C Editor Web - Powered by Native C Engines"];
        [footerLabel setTextColor:[NSColor colorWithRed:0.533 green:0.533 blue:0.533 alpha:1.0]];
        [footerLabel setBackgroundColor:[NSColor clearColor]];
        [footerLabel setBordered:NO];
        [footerLabel setEditable:NO];
        [footerLabel setFont:[NSFont systemFontOfSize:12]];
        [footerLabel setAlignment:NSTextAlignmentCenter];
        [footerLabel setAutoresizingMask:NSViewWidthSizable];
        [footerView addSubview:footerLabel];
        [mainView addSubview:footerView];
        
        [window setContentView:mainView];
        
        // Show window
        [window center];
        [window makeKeyAndOrderFront:nil];
        [app activateIgnoringOtherApps:YES];
        
        NSLog(@"🚀 Main C Hybrid Editor launched!");
        
        // Run app
        [app run];
    }
    
    return 0;
}