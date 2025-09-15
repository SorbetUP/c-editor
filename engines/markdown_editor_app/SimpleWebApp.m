#import <Cocoa/Cocoa.h>
#import <WebKit/WebKit.h>

// Import C engines
#import "../editor/editor_abi.h"

// Global variables
WKWebView *g_webView = nil;

@interface SimpleWebDelegate : NSObject <WKNavigationDelegate, WKScriptMessageHandler>
@end

@implementation SimpleWebDelegate

- (void)webView:(WKWebView *)webView didFinishNavigation:(WKNavigation *)navigation {
    NSLog(@"✅ Web page loaded successfully");
}

- (void)webView:(WKWebView *)webView didFailNavigation:(WKNavigation *)navigation withError:(NSError *)error {
    NSLog(@"❌ Navigation failed: %@", error.localizedDescription);
}

- (void)userContentController:(WKUserContentController *)userContentController didReceiveScriptMessage:(WKScriptMessage *)message {
    if ([message.name isEqualToString:@"markdownRenderer"]) {
        NSDictionary *msgBody = message.body;
        NSString *markdown = msgBody[@"markdown"];
        
        if (markdown) {
            // Use C engine to render
            const char *html = editor_markdown_to_html([markdown UTF8String]);
            if (html) {
                NSString *htmlResult = [NSString stringWithUTF8String:html];
                
                // Send result back to JavaScript
                NSString *script = [NSString stringWithFormat:@"window.handleNativeMarkdownResult('%@');", htmlResult];
                [g_webView evaluateJavaScript:script completionHandler:nil];
            }
        }
    }
}

@end

// Generate simple HTML for hybrid editor
NSString* generateSimpleHybridHTML() {
    return @"<!DOCTYPE html>"
    "<html lang=\"fr\">"
    "<head>"
    "    <meta charset=\"UTF-8\">"
    "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">"
    "    <title>C Editor - Éditeur Hybride</title>"
    "    <style>"
    "        * { margin: 0; padding: 0; box-sizing: border-box; }"
    "        body { background: #1a1a1a; color: #e0e0e0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; height: 100vh; overflow: hidden; }"
    "        .header { background: #2d2d2d; padding: 10px 20px; display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #444; }"
    "        .header h1 { font-size: 16px; font-weight: 600; }"
    "        .status { display: flex; align-items: center; gap: 8px; }"
    "        .status-indicator { width: 8px; height: 8px; border-radius: 50%; background: #51cf66; }"
    "        .main-content { display: flex; height: calc(100vh - 60px); }"
    "        .hybrid-editor { flex: 1; display: flex; flex-direction: column; }"
    "        .pane-header { background: #2d2d2d; padding: 10px 20px; border-bottom: 1px solid #444; font-weight: 600; font-size: 14px; }"
    "        .hybrid-editor-container { flex: 1; padding: 20px; }"
    "        #hybridEditor { width: 100%; height: 100%; background: #1e1e1e; color: #e0e0e0; border: 1px solid #444; border-radius: 4px; padding: 20px; font-family: 'Monaco', 'Consolas', monospace; font-size: 14px; line-height: 1.8; overflow-y: auto; outline: none; }"
    "        #hybridEditor:focus { border-color: #4c6ef5; }"
    "        .editor-line { min-height: 1.8em; padding: 4px 8px; margin: 0; border-radius: 3px; transition: background-color 0.2s ease; cursor: text; }"
    "        .editor-line.current-line { background-color: rgba(76, 110, 245, 0.1); border-left: 3px solid #4c6ef5; padding-left: 15px; }"
    "        .editor-line.rendered-line { background-color: rgba(255, 255, 255, 0.02); }"
    "        .editor-line:empty::before { content: '\\200B'; color: transparent; }"
    "        .editor-line h1, .editor-line h2, .editor-line h3 { margin: 0; color: #fff; display: inline; }"
    "        .editor-line h1 { font-size: 24px; } .editor-line h2 { font-size: 20px; } .editor-line h3 { font-size: 18px; }"
    "        .editor-line strong { color: #fff; font-weight: bold; }"
    "        .editor-line em { color: #ffd93d; font-style: italic; }"
    "        .editor-line u { text-decoration: underline; }"
    "        .editor-line mark { background: #ffd93d; color: #000; padding: 1px 2px; }"
    "    </style>"
    "</head>"
    "<body>"
    "    <div class=\"header\">"
    "        <h1>C Editor - Éditeur Hybride (Native)</h1>"
    "        <div class=\"status\">"
    "            <span class=\"status-indicator\"></span>"
    "            <span>C Engine Ready</span>"
    "        </div>"
    "    </div>"
    "    <div class=\"main-content\">"
    "        <div class=\"hybrid-editor\">"
    "            <div class=\"pane-header\">Éditeur Hybride - Ligne courante en Markdown, autres lignes rendues</div>"
    "            <div class=\"hybrid-editor-container\">"
    "                <div id=\"hybridEditor\" contenteditable=\"true\" spellcheck=\"false\">"
    "                    <div class=\"editor-line current-line\" data-line=\"0\"># Titre</div>"
    "                    <div class=\"editor-line rendered-line\" data-line=\"1\"></div>"
    "                    <div class=\"editor-line rendered-line\" data-line=\"2\">Écrivez votre markdown ici...</div>"
    "                    <div class=\"editor-line rendered-line\" data-line=\"3\"></div>"
    "                    <div class=\"editor-line rendered-line\" data-line=\"4\">Exemple:</div>"
    "                    <div class=\"editor-line rendered-line\" data-line=\"5\"><li><strong>Gras</strong></li></div>"
    "                    <div class=\"editor-line rendered-line\" data-line=\"6\"><li><em>Italique</em></li></div>"
    "                    <div class=\"editor-line rendered-line\" data-line=\"7\"><li><mark>Surligné</mark></li></div>"
    "                    <div class=\"editor-line rendered-line\" data-line=\"8\"><li><u>Souligné</u></li></div>"
    "                </div>"
    "            </div>"
    "        </div>"
    "    </div>"
    "    <script>"
    "        let currentLineIndex = 0;"
    "        let editorLines = [];"
    "        let lineContents = [];"
    "        const hybridEditor = document.getElementById('hybridEditor');"
    "        "
    "        // Initialize"
    "        function initializeEditor() {"
    "            editorLines = Array.from(hybridEditor.querySelectorAll('.editor-line'));"
    "            editorLines.forEach((line, index) => {"
    "                lineContents[index] = line.textContent || '';"
    "                line.dataset.line = index;"
    "            });"
    "        }"
    "        "
    "        // Handle native markdown rendering result"
    "        window.handleNativeMarkdownResult = function(html) {"
    "            if (currentLineIndex >= 0 && editorLines[currentLineIndex]) {"
    "                const line = editorLines[currentLineIndex];"
    "                line.innerHTML = html;"
    "                line.classList.remove('current-line');"
    "                line.classList.add('rendered-line');"
    "            }"
    "        };"
    "        "
    "        // Get current line"
    "        function getCurrentLineElement() {"
    "            const selection = window.getSelection();"
    "            if (selection.rangeCount === 0) return null;"
    "            let node = selection.anchorNode;"
    "            while (node && node !== hybridEditor) {"
    "                if (node.classList && node.classList.contains('editor-line')) {"
    "                    return node;"
    "                }"
    "                node = node.parentNode;"
    "            }"
    "            return null;"
    "        }"
    "        "
    "        // Update line states"
    "        function updateLineStates() {"
    "            const currentLine = getCurrentLineElement();"
    "            const newLineIndex = currentLine ? parseInt(currentLine.dataset.line) : -1;"
    "            if (newLineIndex === currentLineIndex) return;"
    "            "
    "            // Render previous current line"
    "            if (currentLineIndex >= 0 && editorLines[currentLineIndex]) {"
    "                const prevLine = editorLines[currentLineIndex];"
    "                lineContents[currentLineIndex] = prevLine.textContent || '';"
    "                "
    "                // Call native renderer"
    "                window.webkit.messageHandlers.markdownRenderer.postMessage({"
    "                    markdown: lineContents[currentLineIndex]"
    "                });"
    "            }"
    "            "
    "            // Switch new current line to markdown"
    "            if (newLineIndex >= 0 && newLineIndex < editorLines.length) {"
    "                currentLineIndex = newLineIndex;"
    "                const newLine = editorLines[currentLineIndex];"
    "                newLine.textContent = lineContents[currentLineIndex] || '';"
    "                newLine.classList.remove('rendered-line');"
    "                newLine.classList.add('current-line');"
    "                "
    "                // Update other lines"
    "                editorLines.forEach((line, index) => {"
    "                    if (index !== currentLineIndex) {"
    "                        line.classList.remove('current-line');"
    "                        line.classList.add('rendered-line');"
    "                    }"
    "                });"
    "            }"
    "        }"
    "        "
    "        // Event listeners"
    "        hybridEditor.addEventListener('click', () => {"
    "            setTimeout(updateLineStates, 10);"
    "        });"
    "        "
    "        hybridEditor.addEventListener('keyup', () => {"
    "            setTimeout(updateLineStates, 10);"
    "        });"
    "        "
    "        hybridEditor.addEventListener('input', () => {"
    "            if (currentLineIndex >= 0 && editorLines[currentLineIndex]) {"
    "                lineContents[currentLineIndex] = editorLines[currentLineIndex].textContent || '';"
    "            }"
    "        });"
    "        "
    "        // Initialize when loaded"
    "        document.addEventListener('DOMContentLoaded', initializeEditor);"
    "        setTimeout(initializeEditor, 100);"
    "    </script>"
    "</body>"
    "</html>";
}

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        // Initialize C engine
        editor_library_init();
        
        // Create window
        NSRect frame = NSMakeRect(100, 100, 1200, 800);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable | NSWindowStyleMaskMiniaturizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"C Editor - Éditeur Hybride"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]];
        
        // Create WebView configuration
        WKWebViewConfiguration *config = [[WKWebViewConfiguration alloc] init];
        config.preferences.javaScriptEnabled = YES;
        
        // Add script message handler for markdown rendering
        WKUserContentController *contentController = [[WKUserContentController alloc] init];
        SimpleWebDelegate *delegate = [[SimpleWebDelegate alloc] init];
        [contentController addScriptMessageHandler:delegate name:@"markdownRenderer"];
        config.userContentController = contentController;
        
        // Create WebView
        g_webView = [[WKWebView alloc] initWithFrame:frame configuration:config];
        [g_webView setNavigationDelegate:delegate];
        [g_webView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // Load the HTML content
        NSString *htmlContent = generateSimpleHybridHTML();
        [g_webView loadHTMLString:htmlContent baseURL:nil];
        
        // Add WebView to window
        [[window contentView] addSubview:g_webView];
        
        // Show window
        [window makeKeyAndOrderFront:nil];
        [window center];
        
        NSLog(@"🚀 Simple hybrid editor launched");
        
        [app run];
    }
    return 0;
}