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

// Bridge with enhanced debugging
@interface UltimateCEngineBridge : NSObject <WKScriptMessageHandler>
@end

@implementation UltimateCEngineBridge

- (void)userContentController:(WKUserContentController *)userContentController 
      didReceiveScriptMessage:(WKScriptMessage *)message {
    
    NSDictionary *body = message.body;
    NSString *action = body[@"action"];
    
    if ([action isEqualToString:@"renderMarkdown"]) {
        NSString *markdown = body[@"markdown"];
        NSInteger lineIndex = [body[@"lineIndex"] integerValue];
        NSString *html = [self renderWithCEngine:markdown];
        
        NSString *js = [NSString stringWithFormat:@"window.receiveRenderedHTML('%@', %ld);", 
                        [self escapeForJS:html], lineIndex];
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
    
    return [NSString stringWithUTF8String:html_cstr];
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
        
        [window setTitle:@"Ultimate Hybrid - All Bugs Fixed"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]];
        
        NSView *mainView = [[NSView alloc] initWithFrame:[[window contentView] bounds]];
        [mainView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        CGFloat windowWidth = [[window contentView] bounds].size.width;
        CGFloat windowHeight = [[window contentView] bounds].size.height;
        
        WKWebViewConfiguration *config = [[WKWebViewConfiguration alloc] init];
        UltimateCEngineBridge *bridge = [[UltimateCEngineBridge alloc] init];
        [config.userContentController addScriptMessageHandler:bridge name:@"cengine"];
        
        WKWebView *webView = [[WKWebView alloc] initWithFrame:NSMakeRect(20, 20, windowWidth-40, windowHeight-40) configuration:config];
        g_webView = webView;
        [webView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // HTML with all fixes for Enter key, incomplete markdown, line deletion
        NSString *htmlContent = @"<!DOCTYPE html>\n"
        "<html lang=\"fr\">\n"
        "<head>\n"
        "    <meta charset=\"UTF-8\">\n"
        "    <style>\n"
        "        * { margin: 0; padding: 0; box-sizing: border-box; }\n"
        "        body {\n"
        "            background: #1a1a1a;\n"
        "            color: #e0e0e0;\n"
        "            font-family: Monaco, monospace;\n"
        "            font-size: 14px;\n"
        "            line-height: 1.8;\n"
        "            padding: 20px;\n"
        "            overflow: hidden;\n"
        "        }\n"
        "        .editor-line {\n"
        "            min-height: 1.8em;\n"
        "            padding: 4px 8px;\n"
        "            margin: 0;\n"
        "            border-radius: 3px;\n"
        "            transition: background-color 0.2s ease;\n"
        "            cursor: text;\n"
        "            outline: none;\n"
        "        }\n"
        "        .editor-line.current-line {\n"
        "            background-color: rgba(76, 110, 245, 0.1);\n"
        "            border-left: 3px solid #4c6ef5;\n"
        "            padding-left: 15px;\n"
        "        }\n"
        "        .editor-line.rendered-line {\n"
        "            background-color: rgba(255, 255, 255, 0.02);\n"
        "        }\n"
        "        .editor-line:empty::before {\n"
        "            content: '\\200B';\n"
        "            color: transparent;\n"
        "        }\n"
        "        .editor-line h1, .editor-line h2, .editor-line h3, .editor-line h4, .editor-line h5, .editor-line h6 {\n"
        "            margin: 0;\n"
        "            color: #4c6ef5;\n"
        "            display: inline;\n"
        "        }\n"
        "        .editor-line h1 { font-size: 24px; }\n"
        "        .editor-line h2 { font-size: 20px; }\n"
        "        .editor-line h3 { font-size: 18px; }\n"
        "        .editor-line strong { color: #51cf66; font-weight: bold; }\n"
        "        .editor-line em { color: #ffd43b; font-style: italic; }\n"
        "        .editor-line u { text-decoration: underline; color: #ff6b6b; }\n"
        "        .editor-line mark { background: #ffd43b; color: #000; padding: 1px 2px; }\n"
        "    </style>\n"
        "</head>\n"
        "<body>\n"
        "    <h2 style=\"margin-bottom: 20px;\">🔧 Ultimate Hybrid - All Bugs Fixed</h2>\n"
        "    \n"
        "    <div id=\"hybridEditor\" contenteditable=\"true\" spellcheck=\"false\">\n"
        "        <div class=\"editor-line current-line\" data-line=\"0\"># Test Bugs Fixes</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"1\"></div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"2\">Tests à faire:</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"3\">- **Complet** (doit cacher **)</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"4\">- **Incomplet (doit rester **)</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"5\">- *italique*</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"6\">- ==surligné==</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"7\">- ++souligné++</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"8\"></div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"9\">Appuyez Enter ici pour tester ↓</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"10\">Supprimez cette ligne avec Backspace</div>\n"
        "    </div>\n"
        "    \n"
        "    <div style=\"position: fixed; bottom: 10px; left: 10px; font-size: 11px; color: #666; background: #333; padding: 5px; border-radius: 3px;\" id=\"debugInfo\">\n"
        "        Ligne: 0 | Lignes totales: 11 | Dernière action: init\n"
        "    </div>\n"
        "    \n"
        "    <script>\n"
        "        // Enhanced variables with better tracking\n"
        "        let currentLineIndex = 0;\n"
        "        let editorLines = [];\n"
        "        let lineContents = [];\n"
        "        const hybridEditor = document.getElementById('hybridEditor');\n"
        "        const debugInfo = document.getElementById('debugInfo');\n"
        "        let pendingRenders = new Map();\n"
        "        let isProcessingUpdate = false;\n"
        "        let lastAction = 'init';\n"
        "        \n"
        "        // Initialize lines\n"
        "        function refreshLines() {\n"
        "            const oldCount = editorLines.length;\n"
        "            editorLines = Array.from(hybridEditor.querySelectorAll('.editor-line'));\n"
        "            \n"
        "            // Reassign data-line attributes\n"
        "            editorLines.forEach((line, index) => {\n"
        "                line.dataset.line = index;\n"
        "                if (!lineContents[index]) {\n"
        "                    lineContents[index] = line.textContent || '';\n"
        "                }\n"
        "            });\n"
        "            \n"
        "            // Trim lineContents array to match actual lines\n"
        "            lineContents.length = editorLines.length;\n"
        "            \n"
        "            if (oldCount !== editorLines.length) {\n"
        "                console.log(`📏 Lines changed: ${oldCount} -> ${editorLines.length}`);\n"
        "                lastAction = `lines-changed-${oldCount}-to-${editorLines.length}`;\n"
        "            }\n"
        "            \n"
        "            updateDebugInfo();\n"
        "        }\n"
        "        \n"
        "        function updateDebugInfo() {\n"
        "            const currentContent = lineContents[currentLineIndex] || '';\n"
        "            debugInfo.textContent = `Ligne: ${currentLineIndex} | Lignes totales: ${editorLines.length} | Dernière action: ${lastAction} | Contenu: \"${currentContent.substring(0, 20)}${currentContent.length > 20 ? '...' : ''}\"`;\n"
        "        }\n"
        "        \n"
        "        // Mutation Observer to detect DOM changes (Enter, Delete, etc.)\n"
        "        const observer = new MutationObserver((mutations) => {\n"
        "            let needsRefresh = false;\n"
        "            \n"
        "            mutations.forEach((mutation) => {\n"
        "                if (mutation.type === 'childList') {\n"
        "                    if (mutation.addedNodes.length > 0 || mutation.removedNodes.length > 0) {\n"
        "                        needsRefresh = true;\n"
        "                        lastAction = 'dom-mutation';\n"
        "                        console.log('🔄 DOM mutation detected:', mutation);\n"
        "                    }\n"
        "                }\n"
        "            });\n"
        "            \n"
        "            if (needsRefresh && !isProcessingUpdate) {\n"
        "                setTimeout(() => {\n"
        "                    refreshLines();\n"
        "                    updateLineStates();\n"
        "                }, 10);\n"
        "            }\n"
        "        });\n"
        "        \n"
        "        // Start observing\n"
        "        observer.observe(hybridEditor, {\n"
        "            childList: true,\n"
        "            subtree: true\n"
        "        });\n"
        "        \n"
        "        // EXACT mapHtmlPositionToMarkdown from web version\n"
        "        function mapHtmlPositionToMarkdown(htmlPosition, markdownText) {\n"
        "            if (!markdownText || markdownText.trim() === '') {\n"
        "                return 0;\n"
        "            }\n"
        "            \n"
        "            console.log(`🔍 Mapping HTML pos ${htmlPosition} in \"${markdownText}\"`);\n"
        "            \n"
        "            const mapping = [];\n"
        "            let htmlPos = 0;\n"
        "            let i = 0;\n"
        "            \n"
        "            // Handle header prefix\n"
        "            const headerMatch = markdownText.match(/^(#{1,6})\\s+/);\n"
        "            if (headerMatch) {\n"
        "                const prefixLength = headerMatch[0].length;\n"
        "                for (let h = 0; h <= htmlPos; h++) {\n"
        "                    mapping[h] = prefixLength;\n"
        "                }\n"
        "                i = prefixLength;\n"
        "            }\n"
        "            \n"
        "            while (i < markdownText.length) {\n"
        "                const char = markdownText[i];\n"
        "                \n"
        "                if (char === '*' && i + 1 < markdownText.length) {\n"
        "                    if (markdownText[i + 1] === '*') {\n"
        "                        // Bold: **text**\n"
        "                        const endPos = markdownText.indexOf('**', i + 2);\n"
        "                        if (endPos !== -1) {\n"
        "                            const innerText = markdownText.substring(i + 2, endPos);\n"
        "                            for (let j = 0; j < innerText.length; j++) {\n"
        "                                mapping[htmlPos + j] = i + 2 + j;\n"
        "                            }\n"
        "                            htmlPos += innerText.length;\n"
        "                            i = endPos + 2;\n"
        "                            continue;\n"
        "                        }\n"
        "                    } else {\n"
        "                        // Italic: *text*\n"
        "                        const endPos = markdownText.indexOf('*', i + 1);\n"
        "                        if (endPos !== -1) {\n"
        "                            const innerText = markdownText.substring(i + 1, endPos);\n"
        "                            for (let j = 0; j < innerText.length; j++) {\n"
        "                                mapping[htmlPos + j] = i + 1 + j;\n"
        "                            }\n"
        "                            htmlPos += innerText.length;\n"
        "                            i = endPos + 1;\n"
        "                            continue;\n"
        "                        }\n"
        "                    }\n"
        "                } else if (char === '=' && i + 1 < markdownText.length && markdownText[i + 1] === '=') {\n"
        "                    const endPos = markdownText.indexOf('==', i + 2);\n"
        "                    if (endPos !== -1) {\n"
        "                        const innerText = markdownText.substring(i + 2, endPos);\n"
        "                        for (let j = 0; j < innerText.length; j++) {\n"
        "                            mapping[htmlPos + j] = i + 2 + j;\n"
        "                        }\n"
        "                        htmlPos += innerText.length;\n"
        "                        i = endPos + 2;\n"
        "                        continue;\n"
        "                    }\n"
        "                } else if (char === '+' && i + 1 < markdownText.length && markdownText[i + 1] === '+') {\n"
        "                    const endPos = markdownText.indexOf('++', i + 2);\n"
        "                    if (endPos !== -1) {\n"
        "                        const innerText = markdownText.substring(i + 2, endPos);\n"
        "                        for (let j = 0; j < innerText.length; j++) {\n"
        "                            mapping[htmlPos + j] = i + 2 + j;\n"
        "                        }\n"
        "                        htmlPos += innerText.length;\n"
        "                        i = endPos + 2;\n"
        "                        continue;\n"
        "                    }\n"
        "                }\n"
        "                \n"
        "                mapping[htmlPos] = i;\n"
        "                htmlPos++;\n"
        "                i++;\n"
        "            }\n"
        "            \n"
        "            const result = mapping[htmlPosition] !== undefined ? mapping[htmlPosition] : markdownText.length;\n"
        "            console.log(`📍 HTML ${htmlPosition} -> MD ${result}`);\n"
        "            return result;\n"
        "        }\n"
        "        \n"
        "        function calculatePlainTextPosition(htmlElement) {\n"
        "            try {\n"
        "                const selection = window.getSelection();\n"
        "                if (selection.rangeCount === 0) return 0;\n"
        "                \n"
        "                const range = selection.getRangeAt(0);\n"
        "                const preCaretRange = range.cloneRange();\n"
        "                preCaretRange.selectNodeContents(htmlElement);\n"
        "                preCaretRange.setEnd(range.startContainer, range.startOffset);\n"
        "                \n"
        "                return preCaretRange.toString().length;\n"
        "            } catch (error) {\n"
        "                return 0;\n"
        "            }\n"
        "        }\n"
        "        \n"
        "        function getCurrentLineElement() {\n"
        "            const selection = window.getSelection();\n"
        "            if (selection.rangeCount === 0) return null;\n"
        "            \n"
        "            let node = selection.anchorNode;\n"
        "            while (node && node !== hybridEditor) {\n"
        "                if (node.classList && node.classList.contains('editor-line')) {\n"
        "                    return node;\n"
        "                }\n"
        "                node = node.parentNode;\n"
        "            }\n"
        "            return null;\n"
        "        }\n"
        "        \n"
        "        // Enhanced rendering with STRICT incomplete markdown handling\n"
        "        function renderLineAsHTML(lineIndex) {\n"
        "            const line = editorLines[lineIndex];\n"
        "            if (!line) return;\n"
        "            \n"
        "            if (lineIndex === currentLineIndex) {\n"
        "                lineContents[lineIndex] = line.textContent || '';\n"
        "            }\n"
        "            \n"
        "            const markdownText = lineContents[lineIndex] || '';\n"
        "            \n"
        "            if (!markdownText.trim()) {\n"
        "                line.innerHTML = '&nbsp;';\n"
        "                return;\n"
        "            }\n"
        "            \n"
        "            // Request C engine rendering\n"
        "            pendingRenders.set(lineIndex, markdownText);\n"
        "            window.webkit.messageHandlers.cengine.postMessage({\n"
        "                action: 'renderMarkdown',\n"
        "                markdown: markdownText,\n"
        "                lineIndex: lineIndex\n"
        "            });\n"
        "            \n"
        "            // STRICT fallback - NEVER render incomplete markdown\n"
        "            setTimeout(() => {\n"
        "                if (pendingRenders.has(lineIndex)) {\n"
        "                    let html = markdownText;\n"
        "                    \n"
        "                    // Only render COMPLETE pairs\n"
        "                    // Headers (always complete)\n"
        "                    html = html.replace(/^# (.+)$/gm, '<h1>$1</h1>');\n"
        "                    html = html.replace(/^## (.+)$/gm, '<h2>$1</h2>');\n"
        "                    html = html.replace(/^### (.+)$/gm, '<h3>$1</h3>');\n"
        "                    \n"
        "                    // STRICT: Only render if BOTH delimiters are present\n"
        "                    // Bold: must have both **\n"
        "                    const boldMatches = html.match(/\\*\\*[^*]*\\*\\*/g);\n"
        "                    if (boldMatches) {\n"
        "                        boldMatches.forEach(match => {\n"
        "                            const inner = match.slice(2, -2);\n"
        "                            html = html.replace(match, `<strong>${inner}</strong>`);\n"
        "                        });\n"
        "                    }\n"
        "                    \n"
        "                    // Italic: must have both * (and not be part of **)\n"
        "                    const italicMatches = html.match(/(?<!\\*)\\*[^*]+\\*(?!\\*)/g);\n"
        "                    if (italicMatches) {\n"
        "                        italicMatches.forEach(match => {\n"
        "                            const inner = match.slice(1, -1);\n"
        "                            html = html.replace(match, `<em>${inner}</em>`);\n"
        "                        });\n"
        "                    }\n"
        "                    \n"
        "                    // Highlight: must have both ==\n"
        "                    const highlightMatches = html.match(/==[^=]+==/g);\n"
        "                    if (highlightMatches) {\n"
        "                        highlightMatches.forEach(match => {\n"
        "                            const inner = match.slice(2, -2);\n"
        "                            html = html.replace(match, `<mark>${inner}</mark>`);\n"
        "                        });\n"
        "                    }\n"
        "                    \n"
        "                    // Underline: must have both ++\n"
        "                    const underlineMatches = html.match(/\\+\\+[^+]+\\+\\+/g);\n"
        "                    if (underlineMatches) {\n"
        "                        underlineMatches.forEach(match => {\n"
        "                            const inner = match.slice(2, -2);\n"
        "                            html = html.replace(match, `<u>${inner}</u>`);\n"
        "                        });\n"
        "                    }\n"
        "                    \n"
        "                    // DEBUG: Show what we're rendering\n"
        "                    if (html !== markdownText) {\n"
        "                        console.log(`✅ STRICT render line ${lineIndex}: \"${markdownText}\" -> \"${html}\"`);\n"
        "                    } else {\n"
        "                        console.log(`📝 STRICT no render line ${lineIndex}: \"${markdownText}\" (incomplete)`);\n"
        "                    }\n"
        "                    \n"
        "                    line.innerHTML = html;\n"
        "                    pendingRenders.delete(lineIndex);\n"
        "                }\n"
        "            }, 50);\n"
        "        }\n"
        "        \n"
        "        // Receive rendered HTML from C engine\n"
        "        window.receiveRenderedHTML = function(html, lineIndex) {\n"
        "            if (editorLines[lineIndex] && pendingRenders.has(lineIndex)) {\n"
        "                console.log('C engine rendered line', lineIndex, ':', html);\n"
        "                editorLines[lineIndex].innerHTML = html;\n"
        "                pendingRenders.delete(lineIndex);\n"
        "            }\n"
        "        };\n"
        "        \n"
        "        function showLineAsMarkdownWithPosition(lineIndex, targetCursorPosition = 0) {\n"
        "            const line = editorLines[lineIndex];\n"
        "            if (!line) return;\n"
        "            \n"
        "            const markdownContent = lineContents[lineIndex] || '';\n"
        "            line.innerHTML = '';\n"
        "            line.textContent = markdownContent;\n"
        "            \n"
        "            requestAnimationFrame(() => {\n"
        "                try {\n"
        "                    line.focus();\n"
        "                    \n"
        "                    const textNode = line.firstChild;\n"
        "                    if (textNode && textNode.nodeType === Node.TEXT_NODE) {\n"
        "                        const range = document.createRange();\n"
        "                        const selection = window.getSelection();\n"
        "                        const safePosition = Math.min(targetCursorPosition, textNode.textContent.length);\n"
        "                        \n"
        "                        range.setStart(textNode, safePosition);\n"
        "                        range.collapse(true);\n"
        "                        selection.removeAllRanges();\n"
        "                        selection.addRange(range);\n"
        "                        \n"
        "                        console.log(`✅ Cursor positioned at ${safePosition} in line ${lineIndex}`);\n"
        "                    }\n"
        "                } catch (error) {\n"
        "                    line.focus();\n"
        "                }\n"
        "            });\n"
        "        }\n"
        "        \n"
        "        function updateLineClasses() {\n"
        "            editorLines.forEach((line, index) => {\n"
        "                line.classList.remove('current-line', 'rendered-line');\n"
        "                if (index === currentLineIndex) {\n"
        "                    line.classList.add('current-line');\n"
        "                } else {\n"
        "                    line.classList.add('rendered-line');\n"
        "                }\n"
        "            });\n"
        "        }\n"
        "        \n"
        "        function updateLineStates() {\n"
        "            if (isProcessingUpdate) return;\n"
        "            isProcessingUpdate = true;\n"
        "            \n"
        "            const currentLine = getCurrentLineElement();\n"
        "            const newLineIndex = currentLine ? parseInt(currentLine.dataset.line) : -1;\n"
        "            \n"
        "            if (newLineIndex === -1 || newLineIndex >= editorLines.length) {\n"
        "                isProcessingUpdate = false;\n"
        "                return;\n"
        "            }\n"
        "            \n"
        "            if (newLineIndex === currentLineIndex) {\n"
        "                isProcessingUpdate = false;\n"
        "                return;\n"
        "            }\n"
        "            \n"
        "            console.log(`🔄 Line changed: ${currentLineIndex} -> ${newLineIndex}`);\n"
        "            lastAction = `line-switch-${currentLineIndex}-to-${newLineIndex}`;\n"
        "            \n"
        "            // Render old line\n"
        "            if (currentLineIndex >= 0 && editorLines[currentLineIndex]) {\n"
        "                renderLineAsHTML(currentLineIndex);\n"
        "            }\n"
        "            \n"
        "            // Calculate cursor position\n"
        "            let targetCursorPosition = 0;\n"
        "            if (newLineIndex >= 0 && editorLines[newLineIndex]) {\n"
        "                const newLine = editorLines[newLineIndex];\n"
        "                const selection = window.getSelection();\n"
        "                if (selection.rangeCount > 0) {\n"
        "                    try {\n"
        "                        const range = selection.getRangeAt(0);\n"
        "                        if (newLine.contains(range.startContainer) || range.startContainer === newLine) {\n"
        "                            const htmlPosition = calculatePlainTextPosition(newLine);\n"
        "                            \n"
        "                            let markdownContent = lineContents[newLineIndex] || '';\n"
        "                            if (!markdownContent && newLine.textContent) {\n"
        "                                markdownContent = newLine.textContent;\n"
        "                                lineContents[newLineIndex] = markdownContent;\n"
        "                            }\n"
        "                            \n"
        "                            targetCursorPosition = mapHtmlPositionToMarkdown(htmlPosition, markdownContent);\n"
        "                            console.log(`📍 HTML ${htmlPosition} -> MD ${targetCursorPosition}`);\n"
        "                        }\n"
        "                    } catch (error) {\n"
        "                        console.warn('Error calculating cursor position:', error);\n"
        "                    }\n"
        "                }\n"
        "            }\n"
        "            \n"
        "            // Switch to new line\n"
        "            currentLineIndex = newLineIndex;\n"
        "            showLineAsMarkdownWithPosition(currentLineIndex, targetCursorPosition);\n"
        "            updateLineClasses();\n"
        "            updateDebugInfo();\n"
        "            \n"
        "            setTimeout(() => {\n"
        "                isProcessingUpdate = false;\n"
        "            }, 50);\n"
        "        }\n"
        "        \n"
        "        // Event listeners with better handling\n"
        "        hybridEditor.addEventListener('click', () => {\n"
        "            lastAction = 'click';\n"
        "            requestAnimationFrame(updateLineStates);\n"
        "        });\n"
        "        \n"
        "        hybridEditor.addEventListener('keyup', (e) => {\n"
        "            lastAction = `keyup-${e.key}`;\n"
        "            requestAnimationFrame(updateLineStates);\n"
        "        });\n"
        "        \n"
        "        hybridEditor.addEventListener('focus', () => {\n"
        "            lastAction = 'focus';\n"
        "            requestAnimationFrame(updateLineStates);\n"
        "        });\n"
        "        \n"
        "        hybridEditor.addEventListener('input', () => {\n"
        "            if (currentLineIndex >= 0 && editorLines[currentLineIndex]) {\n"
        "                lineContents[currentLineIndex] = editorLines[currentLineIndex].textContent || '';\n"
        "                lastAction = 'input';\n"
        "                updateDebugInfo();\n"
        "            }\n"
        "        });\n"
        "        \n"
        "        // Enhanced keydown handling for Enter and Delete\n"
        "        hybridEditor.addEventListener('keydown', (e) => {\n"
        "            if (e.key === 'Enter') {\n"
        "                lastAction = 'enter-pressed';\n"
        "                // Let Enter happen, then update tracking\n"
        "                setTimeout(() => {\n"
        "                    refreshLines();\n"
        "                    updateLineStates();\n"
        "                }, 20);\n"
        "            } else if (e.key === 'Backspace' || e.key === 'Delete') {\n"
        "                lastAction = `${e.key.toLowerCase()}-pressed`;\n"
        "                // Let deletion happen, then update tracking\n"
        "                setTimeout(() => {\n"
        "                    refreshLines();\n"
        "                    updateLineStates();\n"
        "                }, 20);\n"
        "            }\n"
        "        });\n"
        "        \n"
        "        // Initialize\n"
        "        console.log('🚀 Ultimate Hybrid initializing...');\n"
        "        refreshLines();\n"
        "        \n"
        "        // Render all non-current lines\n"
        "        editorLines.forEach((line, index) => {\n"
        "            if (index !== currentLineIndex) {\n"
        "                renderLineAsHTML(index);\n"
        "            }\n"
        "        });\n"
        "        \n"
        "        // Set first line as current\n"
        "        setTimeout(() => {\n"
        "            showLineAsMarkdownWithPosition(0, 0);\n"
        "            updateLineClasses();\n"
        "            updateDebugInfo();\n"
        "            \n"
        "            if (editorLines[0]) {\n"
        "                editorLines[0].focus();\n"
        "            }\n"
        "            \n"
        "            console.log('✅ Ultimate Hybrid initialized!');\n"
        "        }, 200);\n"
        "    </script>\n"
        "</body>\n"
        "</html>";
        
        [webView loadHTMLString:htmlContent baseURL:nil];
        [mainView addSubview:webView];
        [window setContentView:mainView];
        
        [window center];
        [window makeKeyAndOrderFront:nil];
        [app activateIgnoringOtherApps:YES];
        
        NSLog(@"🚀 Ultimate Hybrid launched - ALL bugs should be fixed!");
        
        [app run];
    }
    
    return 0;
}