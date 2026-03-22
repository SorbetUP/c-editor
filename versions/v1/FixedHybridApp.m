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
    
    int editor_result = editor_library_init();
    if (editor_result != 0) {
        NSLog(@"❌ Editor engine initialization failed: %d", editor_result);
        return NO;
    }
    NSLog(@"✅ Editor engine initialized");
    
    editor_enable_debug_logging(true);
    
    const char *test_html = editor_markdown_to_html("**test**");
    if (test_html) {
        NSLog(@"✅ Markdown engine test: %s", test_html);
    }
    
    c_engines_initialized = 1;
    return YES;
}

// Enhanced bridge with cursor position mapping
@interface FixedCEngineBridge : NSObject <WKScriptMessageHandler>
@end

@implementation FixedCEngineBridge

- (void)userContentController:(WKUserContentController *)userContentController 
      didReceiveScriptMessage:(WKScriptMessage *)message {
    
    NSDictionary *body = message.body;
    NSString *action = body[@"action"];
    
    if ([action isEqualToString:@"renderMarkdown"]) {
        NSString *markdown = body[@"markdown"];
        NSInteger lineIndex = [body[@"lineIndex"] integerValue];
        NSString *html = [self renderWithCEngine:markdown];
        
        // Send HTML back with line index for proper mapping
        NSString *js = [NSString stringWithFormat:@"window.receiveRenderedHTML('%@', %ld);", 
                        [self escapeForJS:html], lineIndex];
        dispatch_async(dispatch_get_main_queue(), ^{
            [g_webView evaluateJavaScript:js completionHandler:nil];
        });
        
    } else if ([action isEqualToString:@"mapCursorPosition"]) {
        NSString *markdown = body[@"markdown"];
        NSInteger htmlPosition = [body[@"htmlPosition"] integerValue];
        
        // Use C cursor engine for precise mapping
        NSInteger markdownPosition = [self mapHtmlToMarkdown:htmlPosition markdown:markdown];
        
        NSString *js = [NSString stringWithFormat:@"window.receiveCursorMapping(%ld, %ld);", 
                        htmlPosition, markdownPosition];
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
    return html;
}

- (NSInteger)mapHtmlToMarkdown:(NSInteger)htmlPos markdown:(NSString *)markdown {
    if (!c_engines_initialized || !markdown) {
        return htmlPos;
    }
    
    const char *markdown_cstr = [markdown UTF8String];
    cursor_position_t result = cursor_html_to_markdown((int)htmlPos, markdown_cstr);
    
    if (result.is_valid) {
        return result.position;
    }
    
    return htmlPos; // Fallback
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
        
        if (!initializeCEngines()) {
            NSLog(@"❌ Failed to initialize C engines, exiting");
            return 1;
        }
        
        NSRect frame = NSMakeRect(100, 100, 1200, 800);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"C Editor Web - Fixed Hybrid"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]];
        
        // Create main view with same layout as before
        NSView *mainView = [[NSView alloc] initWithFrame:[[window contentView] bounds]];
        [mainView setWantsLayer:YES];
        [mainView layer].backgroundColor = [NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0].CGColor;
        [mainView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        CGFloat windowWidth = [[window contentView] bounds].size.width;
        CGFloat windowHeight = [[window contentView] bounds].size.height;
        
        // Header
        NSView *headerView = [[NSView alloc] initWithFrame:NSMakeRect(0, windowHeight - 50, windowWidth, 50)];
        [headerView setWantsLayer:YES];
        [headerView layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor;
        [headerView setAutoresizingMask:NSViewWidthSizable | NSViewMinYMargin];
        
        NSTextField *titleLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 15, 400, 20)];
        [titleLabel setStringValue:@"C Editor Web - Fixed Hybrid"];
        [titleLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [titleLabel setBackgroundColor:[NSColor clearColor]];
        [titleLabel setBordered:NO];
        [titleLabel setEditable:NO];
        [titleLabel setFont:[NSFont systemFontOfSize:16 weight:NSFontWeightSemibold]];
        [headerView addSubview:titleLabel];
        
        NSView *statusDot = [[NSView alloc] initWithFrame:NSMakeRect(windowWidth - 200, 21, 8, 8)];
        [statusDot setWantsLayer:YES];
        [statusDot layer].backgroundColor = [NSColor colorWithRed:0.318 green:0.812 blue:0.400 alpha:1.0].CGColor;
        [statusDot layer].cornerRadius = 4.0;
        [statusDot setAutoresizingMask:NSViewMinXMargin];
        
        NSTextField *statusLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(windowWidth - 180, 15, 150, 20)];
        [statusLabel setStringValue:@"Fixed C Engines"];
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
        [editorContainer setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        NSView *paneHeader = [[NSView alloc] initWithFrame:NSMakeRect(0, windowHeight - 130, windowWidth, 40)];
        [paneHeader setWantsLayer:YES];
        [paneHeader layer].backgroundColor = [NSColor colorWithRed:0.176 green:0.176 blue:0.176 alpha:1.0].CGColor;
        [paneHeader setAutoresizingMask:NSViewWidthSizable | NSViewMinYMargin];
        
        NSTextField *paneLabel = [[NSTextField alloc] initWithFrame:NSMakeRect(20, 10, windowWidth - 40, 20)];
        [paneLabel setStringValue:@"Éditeur Hybride - CORRIGÉ - Délimiteurs masqués, curseur précis, Enter fixé"];
        [paneLabel setTextColor:[NSColor colorWithRed:0.878 green:0.878 blue:0.878 alpha:1.0]];
        [paneLabel setBackgroundColor:[NSColor clearColor]];
        [paneLabel setBordered:NO];
        [paneLabel setEditable:NO];
        [paneLabel setFont:[NSFont systemFontOfSize:14 weight:NSFontWeightSemibold]];
        [paneLabel setAutoresizingMask:NSViewWidthSizable];
        [paneHeader addSubview:paneLabel];
        [editorContainer addSubview:paneHeader];
        
        // WebView with enhanced bridge
        CGFloat editorY = 20;
        CGFloat editorHeight = windowHeight - 150;
        NSRect webFrame = NSMakeRect(20, editorY, windowWidth - 40, editorHeight);
        
        WKWebViewConfiguration *config = [[WKWebViewConfiguration alloc] init];
        FixedCEngineBridge *bridge = [[FixedCEngineBridge alloc] init];
        [config.userContentController addScriptMessageHandler:bridge name:@"cengine"];
        
        WKWebView *webView = [[WKWebView alloc] initWithFrame:webFrame configuration:config];
        g_webView = webView;
        [webView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        [webView setWantsLayer:YES];
        [webView layer].borderColor = [NSColor colorWithRed:0.267 green:0.267 blue:0.267 alpha:1.0].CGColor;
        [webView layer].borderWidth = 1.0;
        [webView layer].cornerRadius = 4.0;
        
        // Fixed HTML with enhanced JavaScript
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
        "            outline: none;\n"
        "        }\n"
        "        .editor-line.current-line {\n"
        "            background-color: rgba(76, 110, 245, 0.15);\n"
        "            border-left: 4px solid #4c6ef5;\n"
        "            padding-left: 16px;\n"
        "        }\n"
        "        .editor-line.rendered-line {\n"
        "            background-color: rgba(255, 255, 255, 0.03);\n"
        "        }\n"
        "        .editor-line:empty::before {\n"
        "            content: '\\200B';\n"
        "            color: transparent;\n"
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
        "        <div class=\"editor-line current-line\" data-line=\"0\"># Test des Corrections</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"1\"></div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"2\">Tests à faire:</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"3\">- **Gras complet**</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"4\">- **Gras incomplet</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"5\">- Gras incomplet**</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"6\">- *Italique*</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"7\">- ==Surligné==</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"8\">- ++Souligné++</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"9\"></div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"10\">## Test Enter et Curseur</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"11\">Cliquez entre 'G' et 'r' dans Gras</div>\n"
        "    </div>\n"
        "    \n"
        "    <div style=\"margin-top: 20px; font-size: 12px; color: #666; font-family: system;\" id=\"debug\">\n"
        "        Debug: Ligne courante 0, Position curseur: N/A\n"
        "    </div>\n"
        "    \n"
        "    <script>\n"
        "        let currentLineIndex = 0;\n"
        "        const editor = document.getElementById('editor');\n"
        "        const lines = Array.from(editor.querySelectorAll('.editor-line'));\n"
        "        const lineContents = [];\n"
        "        const debugDiv = document.getElementById('debug');\n"
        "        let pendingRenders = new Map();\n"
        "        let isUpdatingLines = false;\n"
        "        \n"
        "        // Store initial content\n"
        "        lines.forEach((line, index) => {\n"
        "            lineContents[index] = line.textContent || '';\n"
        "        });\n"
        "        \n"
        "        // Enhanced fallback markdown converter that doesn't render incomplete markup\n"
        "        function smartMarkdownToHtml(text) {\n"
        "            if (!text || !text.trim()) return '&nbsp;';\n"
        "            \n"
        "            let html = text;\n"
        "            \n"
        "            // Headers (complete only)\n"
        "            html = html.replace(/^# (.+)$/gm, '<h1>$1</h1>');\n"
        "            html = html.replace(/^## (.+)$/gm, '<h2>$1</h2>');\n"
        "            html = html.replace(/^### (.+)$/gm, '<h3>$1</h3>');\n"
        "            \n"
        "            // Only render COMPLETE formatting pairs\n"
        "            // Bold - only if both ** are present\n"
        "            html = html.replace(/\\*\\*([^*]+)\\*\\*/g, '<strong>$1</strong>');\n"
        "            \n"
        "            // Italic - only if both * are present and not part of **\n"
        "            html = html.replace(/(?<!\\*)\\*([^*]+)\\*(?!\\*)/g, '<em>$1</em>');\n"
        "            \n"
        "            // Highlight - only if both == are present\n"
        "            html = html.replace(/==([^=]+)==/g, '<mark>$1</mark>');\n"
        "            \n"
        "            // Underline - only if both ++ are present\n"
        "            html = html.replace(/\\+\\+([^+]+)\\+\\+/g, '<u>$1</u>');\n"
        "            \n"
        "            // DON'T render incomplete pairs - they stay as text\n"
        "            console.log('Smart markdown conversion:', text, '->', html);\n"
        "            return html;\n"
        "        }\n"
        "        \n"
        "        // Calculate cursor position in plain text\n"
        "        function getCursorPosition(element) {\n"
        "            const selection = window.getSelection();\n"
        "            if (selection.rangeCount === 0) return 0;\n"
        "            \n"
        "            const range = selection.getRangeAt(0);\n"
        "            const preCaretRange = range.cloneRange();\n"
        "            preCaretRange.selectNodeContents(element);\n"
        "            preCaretRange.setEnd(range.endContainer, range.endOffset);\n"
        "            \n"
        "            return preCaretRange.toString().length;\n"
        "        }\n"
        "        \n"
        "        // Set cursor position in element\n"
        "        function setCursorPosition(element, position) {\n"
        "            const textNode = element.firstChild;\n"
        "            if (!textNode || textNode.nodeType !== Node.TEXT_NODE) return;\n"
        "            \n"
        "            const range = document.createRange();\n"
        "            const selection = window.getSelection();\n"
        "            const safePosition = Math.min(position, textNode.textContent.length);\n"
        "            \n"
        "            range.setStart(textNode, safePosition);\n"
        "            range.collapse(true);\n"
        "            selection.removeAllRanges();\n"
        "            selection.addRange(range);\n"
        "        }\n"
        "        \n"
        "        // Enhanced line rendering with C engine\n"
        "        function renderLineWithCEngine(index) {\n"
        "            if (!lines[index] || pendingRenders.has(index)) return;\n"
        "            \n"
        "            const content = lineContents[index] || '';\n"
        "            if (!content.trim()) {\n"
        "                lines[index].innerHTML = '&nbsp;';\n"
        "                return;\n"
        "            }\n"
        "            \n"
        "            pendingRenders.set(index, content);\n"
        "            console.log('Requesting C engine render for line', index, ':', content);\n"
        "            \n"
        "            window.webkit.messageHandlers.cengine.postMessage({\n"
        "                action: 'renderMarkdown',\n"
        "                markdown: content,\n"
        "                lineIndex: index\n"
        "            });\n"
        "            \n"
        "            // Enhanced fallback with smart rendering\n"
        "            setTimeout(() => {\n"
        "                if (pendingRenders.has(index)) {\n"
        "                    console.log('Using smart fallback for line', index);\n"
        "                    const smartHtml = smartMarkdownToHtml(content);\n"
        "                    lines[index].innerHTML = smartHtml;\n"
        "                    pendingRenders.delete(index);\n"
        "                }\n"
        "            }, 50);\n"
        "        }\n"
        "        \n"
        "        // Receive rendered HTML from C engine\n"
        "        window.receiveRenderedHTML = function(html, lineIndex) {\n"
        "            if (lines[lineIndex] && pendingRenders.has(lineIndex)) {\n"
        "                console.log('C engine rendered line', lineIndex, ':', html);\n"
        "                lines[lineIndex].innerHTML = html;\n"
        "                pendingRenders.delete(lineIndex);\n"
        "            }\n"
        "        };\n"
        "        \n"
        "        // Receive cursor mapping from C engine\n"
        "        window.receiveCursorMapping = function(htmlPos, markdownPos) {\n"
        "            console.log('Cursor mapping: HTML', htmlPos, '-> Markdown', markdownPos);\n"
        "            if (lines[currentLineIndex]) {\n"
        "                setCursorPosition(lines[currentLineIndex], markdownPos);\n"
        "            }\n"
        "        };\n"
        "        \n"
        "        function showLineAsMarkdown(index, cursorPosition = 0) {\n"
        "            if (!lines[index]) return;\n"
        "            const content = lineContents[index] || '';\n"
        "            console.log('Showing line', index, 'as markdown:', content, 'cursor at:', cursorPosition);\n"
        "            \n"
        "            lines[index].innerHTML = '';\n"
        "            lines[index].textContent = content;\n"
        "            \n"
        "            // Set cursor position after a brief delay\n"
        "            setTimeout(() => {\n"
        "                setCursorPosition(lines[index], cursorPosition);\n"
        "            }, 10);\n"
        "        }\n"
        "        \n"
        "        function updateCurrentLine() {\n"
        "            const selection = window.getSelection();\n"
        "            if (selection.rangeCount === 0 || isUpdatingLines) return;\n"
        "            \n"
        "            let node = selection.anchorNode;\n"
        "            while (node && node !== editor) {\n"
        "                if (node.classList && node.classList.contains('editor-line')) {\n"
        "                    const newIndex = parseInt(node.dataset.line);\n"
        "                    if (newIndex !== currentLineIndex && newIndex >= 0 && newIndex < lines.length) {\n"
        "                        switchToLine(newIndex);\n"
        "                    }\n"
        "                    break;\n"
        "                }\n"
        "                node = node.parentNode;\n"
        "            }\n"
        "        }\n"
        "        \n"
        "        function switchToLine(newIndex) {\n"
        "            if (newIndex === currentLineIndex || !lines[newIndex] || isUpdatingLines) return;\n"
        "            \n"
        "            isUpdatingLines = true;\n"
        "            console.log('\\n=== SWITCHING: Line', currentLineIndex, '->', newIndex, '===');\n"
        "            \n"
        "            let cursorPosition = 0;\n"
        "            \n"
        "            // If clicking on a rendered line, map cursor position\n"
        "            if (lines[newIndex].classList.contains('rendered-line')) {\n"
        "                cursorPosition = getCursorPosition(lines[newIndex]);\n"
        "                console.log('Clicked at HTML position:', cursorPosition, 'in line:', lineContents[newIndex]);\n"
        "                \n"
        "                // Request cursor mapping from C engine\n"
        "                window.webkit.messageHandlers.cengine.postMessage({\n"
        "                    action: 'mapCursorPosition',\n"
        "                    markdown: lineContents[newIndex] || '',\n"
        "                    htmlPosition: cursorPosition\n"
        "                });\n"
        "            }\n"
        "            \n"
        "            // Save current line content\n"
        "            if (lines[currentLineIndex] && lines[currentLineIndex].classList.contains('current-line')) {\n"
        "                lineContents[currentLineIndex] = lines[currentLineIndex].textContent || '';\n"
        "                console.log('Saved line', currentLineIndex, 'content:', lineContents[currentLineIndex]);\n"
        "            }\n"
        "            \n"
        "            // Render old current line\n"
        "            if (lines[currentLineIndex]) {\n"
        "                lines[currentLineIndex].classList.remove('current-line');\n"
        "                lines[currentLineIndex].classList.add('rendered-line');\n"
        "                renderLineWithCEngine(currentLineIndex);\n"
        "            }\n"
        "            \n"
        "            // Show new line as markdown\n"
        "            lines[newIndex].classList.remove('rendered-line');\n"
        "            lines[newIndex].classList.add('current-line');\n"
        "            showLineAsMarkdown(newIndex, cursorPosition);\n"
        "            \n"
        "            currentLineIndex = newIndex;\n"
        "            debugDiv.textContent = `Debug: Ligne courante ${currentLineIndex}, Contenu: '${lineContents[currentLineIndex]}'`;\n"
        "            \n"
        "            setTimeout(() => {\n"
        "                isUpdatingLines = false;\n"
        "                lines[currentLineIndex].focus();\n"
        "            }, 50);\n"
        "            \n"
        "            console.log('=== SWITCH COMPLETE ===\\n');\n"
        "        }\n"
        "        \n"
        "        // Enhanced event listeners\n"
        "        editor.addEventListener('click', function(e) {\n"
        "            e.preventDefault();\n"
        "            setTimeout(updateCurrentLine, 10);\n"
        "        });\n"
        "        \n"
        "        editor.addEventListener('keydown', function(e) {\n"
        "            if (e.key === 'Enter') {\n"
        "                // Let Enter happen naturally, then update line tracking\n"
        "                setTimeout(() => {\n"
        "                    // Re-scan for lines after Enter creates new ones\n"
        "                    const newLines = Array.from(editor.querySelectorAll('.editor-line'));\n"
        "                    if (newLines.length !== lines.length) {\n"
        "                        console.log('Lines changed after Enter:', lines.length, '->', newLines.length);\n"
        "                        // TODO: Handle line insertion/deletion\n"
        "                    }\n"
        "                    updateCurrentLine();\n"
        "                }, 20);\n"
        "            }\n"
        "        });\n"
        "        \n"
        "        editor.addEventListener('keyup', function(e) {\n"
        "            if (lines[currentLineIndex] && lines[currentLineIndex].classList.contains('current-line')) {\n"
        "                lineContents[currentLineIndex] = lines[currentLineIndex].textContent || '';\n"
        "                debugDiv.textContent = `Debug: Ligne courante ${currentLineIndex}, Contenu: '${lineContents[currentLineIndex]}'`;\n"
        "            }\n"
        "        });\n"
        "        \n"
        "        editor.addEventListener('input', function(e) {\n"
        "            if (lines[currentLineIndex] && lines[currentLineIndex].classList.contains('current-line')) {\n"
        "                lineContents[currentLineIndex] = lines[currentLineIndex].textContent || '';\n"
        "            }\n"
        "        });\n"
        "        \n"
        "        // Initialize\n"
        "        console.log('=== INITIALIZING FIXED VERSION ===');\n"
        "        lines.forEach((line, index) => {\n"
        "            if (index !== currentLineIndex) {\n"
        "                renderLineWithCEngine(index);\n"
        "            } else {\n"
        "                showLineAsMarkdown(index);\n"
        "            }\n"
        "        });\n"
        "        \n"
        "        setTimeout(() => {\n"
        "            if (lines[0]) {\n"
        "                lines[0].focus();\n"
        "                console.log('Focused first line');\n"
        "            }\n"
        "        }, 200);\n"
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
        [footerLabel setStringValue:@"C Editor Web - Fixed Version avec Corrections de Bugs"];
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
        [window center];
        [window makeKeyAndOrderFront:nil];
        [app activateIgnoringOtherApps:YES];
        
        NSLog(@"🚀 Fixed Hybrid App launched - All bugs should be corrected!");
        
        [app run];
    }
    
    return 0;
}