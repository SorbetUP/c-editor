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

// Bridge that handles C engine communication
@interface PerfectCEngineBridge : NSObject <WKScriptMessageHandler>
@end

@implementation PerfectCEngineBridge

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
        
        [window setTitle:@"Perfect Hybrid - Exact Web Copy"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]];
        
        // Simple layout - just the editor
        NSView *mainView = [[NSView alloc] initWithFrame:[[window contentView] bounds]];
        [mainView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        CGFloat windowWidth = [[window contentView] bounds].size.width;
        CGFloat windowHeight = [[window contentView] bounds].size.height;
        
        // WebView that exactly copies the web version
        WKWebViewConfiguration *config = [[WKWebViewConfiguration alloc] init];
        PerfectCEngineBridge *bridge = [[PerfectCEngineBridge alloc] init];
        [config.userContentController addScriptMessageHandler:bridge name:@"cengine"];
        
        WKWebView *webView = [[WKWebView alloc] initWithFrame:NSMakeRect(20, 20, windowWidth-40, windowHeight-40) configuration:config];
        g_webView = webView;
        [webView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // HTML that EXACTLY copies the web version logic
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
        "    <div id=\"hybridEditor\" contenteditable=\"true\" spellcheck=\"false\">\n"
        "        <div class=\"editor-line current-line\" data-line=\"0\"># Test Parfait</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"1\"></div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"2\">Tests curseur:</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"3\">- **Gras complet**</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"4\">- **Gras incomplet</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"5\">- incomplet**</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"6\">- *italique*</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"7\">- ==surligné==</div>\n"
        "        <div class=\"editor-line rendered-line\" data-line=\"8\">- ++souligné++</div>\n"
        "    </div>\n"
        "    \n"
        "    <div style=\"position: fixed; bottom: 10px; left: 10px; font-size: 11px; color: #666;\" id=\"debugInfo\">\n"
        "        Perfect Hybrid - Ligne: 0\n"
        "    </div>\n"
        "    \n"
        "    <script>\n"
        "        // EXACT COPY of web version variables and functions\n"
        "        let currentLineIndex = 0;\n"
        "        let editorLines = [];\n"
        "        let lineContents = [];\n"
        "        const hybridEditor = document.getElementById('hybridEditor');\n"
        "        const debugInfo = document.getElementById('debugInfo');\n"
        "        let pendingRenders = new Map();\n"
        "        \n"
        "        // Initialize - EXACT COPY from web version\n"
        "        editorLines = Array.from(hybridEditor.querySelectorAll('.editor-line'));\n"
        "        editorLines.forEach((line, index) => {\n"
        "            lineContents[index] = line.textContent || '';\n"
        "            line.dataset.line = index;\n"
        "        });\n"
        "        \n"
        "        // EXACT COPY of mapHtmlPositionToMarkdown from web version\n"
        "        function mapHtmlPositionToMarkdown(htmlPosition, markdownText) {\n"
        "            // Handle empty markdown\n"
        "            if (!markdownText || markdownText.trim() === '') {\n"
        "                console.log(`🔍 Empty markdown, returning position 0`);\n"
        "                return 0;\n"
        "            }\n"
        "            \n"
        "            console.log(`🔍 DEBUG: Mapping HTML pos ${htmlPosition} in \"${markdownText}\"`);\n"
        "            \n"
        "            // Build a complete character-by-character mapping\n"
        "            const mapping = [];\n"
        "            let htmlPos = 0;\n"
        "            let i = 0;\n"
        "            \n"
        "            // Handle header prefix first\n"
        "            const headerMatch = markdownText.match(/^(#{1,6})\\s+/);\n"
        "            if (headerMatch) {\n"
        "                const prefixLength = headerMatch[0].length;\n"
        "                // Map all positions before the actual content to after the prefix\n"
        "                for (let h = 0; h <= htmlPos; h++) {\n"
        "                    mapping[h] = prefixLength;\n"
        "                }\n"
        "                i = prefixLength;\n"
        "                console.log(`🏷️ Header prefix: \"${headerMatch[0]}\" -> positions 0-${htmlPos} map to ${prefixLength}`);\n"
        "            }\n"
        "            \n"
        "            // Process character by character\n"
        "            while (i < markdownText.length) {\n"
        "                const char = markdownText[i];\n"
        "                \n"
        "                if (char === '*' && i + 1 < markdownText.length) {\n"
        "                    if (markdownText[i + 1] === '*') {\n"
        "                        // Bold: **text**\n"
        "                        const endPos = markdownText.indexOf('**', i + 2);\n"
        "                        if (endPos !== -1) {\n"
        "                            const innerText = markdownText.substring(i + 2, endPos);\n"
        "                            console.log(`⭐ Bold: \"**${innerText}**\" at MD ${i}, HTML will be ${htmlPos}-${htmlPos + innerText.length - 1}`);\n"
        "                            \n"
        "                            // Map each HTML position inside the bold text to corresponding MD position\n"
        "                            for (let j = 0; j < innerText.length; j++) {\n"
        "                                mapping[htmlPos + j] = i + 2 + j;\n"
        "                            }\n"
        "                            \n"
        "                            htmlPos += innerText.length;\n"
        "                            i = endPos + 2;\n"
        "                            continue;\n"
        "                        }\n"
        "                    } else {\n"
        "                        // Italic: *text*\n"
        "                        const endPos = markdownText.indexOf('*', i + 1);\n"
        "                        if (endPos !== -1) {\n"
        "                            const innerText = markdownText.substring(i + 1, endPos);\n"
        "                            console.log(`💫 Italic: \"*${innerText}*\" at MD ${i}, HTML will be ${htmlPos}-${htmlPos + innerText.length - 1}`);\n"
        "                            \n"
        "                            // Map each HTML position inside the italic text to corresponding MD position\n"
        "                            for (let j = 0; j < innerText.length; j++) {\n"
        "                                mapping[htmlPos + j] = i + 1 + j;\n"
        "                            }\n"
        "                            \n"
        "                            htmlPos += innerText.length;\n"
        "                            i = endPos + 1;\n"
        "                            continue;\n"
        "                        }\n"
        "                    }\n"
        "                } else if (char === '=' && i + 1 < markdownText.length && markdownText[i + 1] === '=') {\n"
        "                    // Highlight: ==text==\n"
        "                    const endPos = markdownText.indexOf('==', i + 2);\n"
        "                    if (endPos !== -1) {\n"
        "                        const innerText = markdownText.substring(i + 2, endPos);\n"
        "                        console.log(`🌟 Highlight: \"==${innerText}==\" at MD ${i}, HTML will be ${htmlPos}-${htmlPos + innerText.length - 1}`);\n"
        "                        \n"
        "                        // Map each HTML position inside the highlight text to corresponding MD position\n"
        "                        for (let j = 0; j < innerText.length; j++) {\n"
        "                            mapping[htmlPos + j] = i + 2 + j;\n"
        "                        }\n"
        "                        \n"
        "                        htmlPos += innerText.length;\n"
        "                        i = endPos + 2;\n"
        "                        continue;\n"
        "                    }\n"
        "                } else if (char === '+' && i + 1 < markdownText.length && markdownText[i + 1] === '+') {\n"
        "                    // Underline: ++text++\n"
        "                    const endPos = markdownText.indexOf('++', i + 2);\n"
        "                    if (endPos !== -1) {\n"
        "                        const innerText = markdownText.substring(i + 2, endPos);\n"
        "                        console.log(`🔸 Underline: \"++${innerText}++\" at MD ${i}, HTML will be ${htmlPos}-${htmlPos + innerText.length - 1}`);\n"
        "                        \n"
        "                        // Map each HTML position inside the underline text to corresponding MD position\n"
        "                        for (let j = 0; j < innerText.length; j++) {\n"
        "                            mapping[htmlPos + j] = i + 2 + j;\n"
        "                        }\n"
        "                        \n"
        "                        htmlPos += innerText.length;\n"
        "                        i = endPos + 2;\n"
        "                        continue;\n"
        "                    }\n"
        "                }\n"
        "                \n"
        "                // Regular character - direct 1:1 mapping\n"
        "                mapping[htmlPos] = i;\n"
        "                htmlPos++;\n"
        "                i++;\n"
        "            }\n"
        "            \n"
        "            // Return the mapped position\n"
        "            const result = mapping[htmlPosition] !== undefined ? mapping[htmlPosition] : markdownText.length;\n"
        "            \n"
        "            // Show relevant portion of mapping table for debugging\n"
        "            const showFrom = Math.max(0, htmlPosition - 3);\n"
        "            const showTo = Math.min(mapping.length, htmlPosition + 8);\n"
        "            const relevantMapping = mapping.slice(showFrom, showTo);\n"
        "            const indices = Array.from({length: showTo - showFrom}, (_, i) => showFrom + i);\n"
        "            \n"
        "            console.log(`🎯 Mapping table [${showFrom}-${showTo-1}]:`, relevantMapping);\n"
        "            console.log(`🔢 HTML indices [${showFrom}-${showTo-1}]:`, indices);\n"
        "            console.log(`📍 HTML ${htmlPosition} -> MD ${result} (${htmlPosition < mapping.length ? 'mapped' : 'beyond end'})`);\n"
        "            return result;\n"
        "        }\n"
        "        \n"
        "        // EXACT COPY of other essential functions from web version\n"
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
        "                // Get the text content up to the cursor position\n"
        "                const textBeforeCursor = preCaretRange.toString();\n"
        "                return textBeforeCursor.length;\n"
        "            } catch (error) {\n"
        "                console.warn('Error calculating plain text position:', error);\n"
        "                return 0;\n"
        "            }\n"
        "        }\n"
        "        \n"
        "        function getCurrentLineElement() {\n"
        "            const selection = window.getSelection();\n"
        "            if (selection.rangeCount === 0) return null;\n"
        "            \n"
        "            let node = selection.anchorNode;\n"
        "            \n"
        "            // Find the parent .editor-line element\n"
        "            while (node && node !== hybridEditor) {\n"
        "                if (node.classList && node.classList.contains('editor-line')) {\n"
        "                    return node;\n"
        "                }\n"
        "                node = node.parentNode;\n"
        "            }\n"
        "            \n"
        "            return null;\n"
        "        }\n"
        "        \n"
        "        // C ENGINE rendering\n"
        "        function renderLineAsHTML(lineIndex) {\n"
        "            const line = editorLines[lineIndex];\n"
        "            if (!line) return;\n"
        "            \n"
        "            // ALWAYS update content from current line text when it was in markdown mode\n"
        "            if (lineIndex === currentLineIndex) {\n"
        "                lineContents[lineIndex] = line.textContent || '';\n"
        "                console.log(`💾 Saved line ${lineIndex} content:`, lineContents[lineIndex]);\n"
        "            }\n"
        "            \n"
        "            const markdownText = lineContents[lineIndex] || '';\n"
        "            console.log(`🔄 Rendering line ${lineIndex} as HTML:`, markdownText);\n"
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
        "            // Fallback after 100ms\n"
        "            setTimeout(() => {\n"
        "                if (pendingRenders.has(lineIndex)) {\n"
        "                    console.log('Using fallback for line', lineIndex);\n"
        "                    // Simple fallback - only render COMPLETE formatting\n"
        "                    let html = markdownText;\n"
        "                    html = html.replace(/^# (.+)$/gm, '<h1>$1</h1>');\n"
        "                    html = html.replace(/^## (.+)$/gm, '<h2>$1</h2>');\n"
        "                    html = html.replace(/\\*\\*([^*]+)\\*\\*/g, '<strong>$1</strong>');\n"
        "                    html = html.replace(/(?<!\\*)\\*([^*]+)\\*(?!\\*)/g, '<em>$1</em>');\n"
        "                    html = html.replace(/==([^=]+)==/g, '<mark>$1</mark>');\n"
        "                    html = html.replace(/\\+\\+([^+]+)\\+\\+/g, '<u>$1</u>');\n"
        "                    line.innerHTML = html;\n"
        "                    pendingRenders.delete(lineIndex);\n"
        "                }\n"
        "            }, 100);\n"
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
        "            console.log(`🎯 showLineAsMarkdownWithPosition: line ${lineIndex}, targetPosition ${targetCursorPosition}`);\n"
        "            \n"
        "            // Get the stored markdown content\n"
        "            const markdownContent = lineContents[lineIndex] || '';\n"
        "            \n"
        "            // Set as plain text (markdown source)\n"
        "            line.innerHTML = '';\n"
        "            line.textContent = markdownContent;\n"
        "            \n"
        "            // Always focus the line and restore cursor position\n"
        "            requestAnimationFrame(() => {\n"
        "                try {\n"
        "                    line.focus();\n"
        "                    \n"
        "                    // Set cursor position\n"
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
        "                    console.warn('Error setting cursor position:', error);\n"
        "                    line.focus();\n"
        "                }\n"
        "            });\n"
        "            \n"
        "            console.log(`✅ Line ${lineIndex} switched to markdown: \"${markdownContent}\"`);\n"
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
        "        // EXACT COPY of updateLineStates from web version\n"
        "        function updateLineStates() {\n"
        "            const currentLine = getCurrentLineElement();\n"
        "            const newLineIndex = currentLine ? parseInt(currentLine.dataset.line) : -1;\n"
        "            \n"
        "            console.log(`🔄 updateLineStates: currentLine=`, currentLine, `newLineIndex=${newLineIndex}`);\n"
        "            \n"
        "            // If we can't find a current line, try to maintain the previous state\n"
        "            if (newLineIndex === -1) {\n"
        "                console.log(`⚠️ No current line found, maintaining currentLineIndex=${currentLineIndex}`);\n"
        "                return;\n"
        "            }\n"
        "            \n"
        "            // Validate new line index\n"
        "            if (newLineIndex >= editorLines.length) {\n"
        "                console.warn(`⚠️ Invalid newLineIndex: ${newLineIndex}, max: ${editorLines.length - 1}`);\n"
        "                return;\n"
        "            }\n"
        "            \n"
        "            if (newLineIndex === currentLineIndex) return; // No change\n"
        "            \n"
        "            console.log('Line changed from', currentLineIndex, 'to', newLineIndex);\n"
        "            \n"
        "            // Render the previously current line (if any)\n"
        "            if (currentLineIndex >= 0 && editorLines[currentLineIndex]) {\n"
        "                renderLineAsHTML(currentLineIndex);\n"
        "            }\n"
        "            \n"
        "            // Calculate cursor position in the new line BEFORE switching it to markdown\n"
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
        "                            // Make sure we have current content\n"
        "                            let markdownContent = lineContents[newLineIndex] || '';\n"
        "                            if (!markdownContent && newLine.textContent) {\n"
        "                                markdownContent = newLine.textContent;\n"
        "                                lineContents[newLineIndex] = markdownContent;\n"
        "                                console.log(`📝 Updated line content from DOM: \"${markdownContent}\"`);\n"
        "                            }\n"
        "                            \n"
        "                            targetCursorPosition = mapHtmlPositionToMarkdown(htmlPosition, markdownContent);\n"
        "                            console.log(`📍 HTML position: ${htmlPosition}, Markdown position: ${targetCursorPosition}`);\n"
        "                        }\n"
        "                    } catch (error) {\n"
        "                        console.warn('Error calculating cursor position:', error);\n"
        "                    }\n"
        "                }\n"
        "            }\n"
        "            \n"
        "            // Switch new current line to raw markdown\n"
        "            if (newLineIndex >= 0 && newLineIndex < editorLines.length) {\n"
        "                currentLineIndex = newLineIndex;\n"
        "                showLineAsMarkdownWithPosition(currentLineIndex, targetCursorPosition);\n"
        "                updateLineClasses();\n"
        "                debugInfo.textContent = `Perfect Hybrid - Ligne: ${currentLineIndex}, Contenu: ${lineContents[currentLineIndex]}`;\n"
        "            }\n"
        "        }\n"
        "        \n"
        "        // Event listeners - EXACT COPY\n"
        "        hybridEditor.addEventListener('click', () => {\n"
        "            requestAnimationFrame(updateLineStates);\n"
        "        });\n"
        "        \n"
        "        hybridEditor.addEventListener('keyup', () => {\n"
        "            requestAnimationFrame(updateLineStates);\n"
        "        });\n"
        "        \n"
        "        hybridEditor.addEventListener('focus', () => {\n"
        "            requestAnimationFrame(updateLineStates);\n"
        "        });\n"
        "        \n"
        "        hybridEditor.addEventListener('input', () => {\n"
        "            // Update line contents for current line\n"
        "            if (currentLineIndex >= 0 && editorLines[currentLineIndex]) {\n"
        "                lineContents[currentLineIndex] = editorLines[currentLineIndex].textContent || '';\n"
        "                debugInfo.textContent = `Perfect Hybrid - Ligne: ${currentLineIndex}, Contenu: ${lineContents[currentLineIndex]}`;\n"
        "            }\n"
        "        });\n"
        "        \n"
        "        // Initialize - EXACT COPY\n"
        "        // Initial render - render all lines as HTML first\n"
        "        editorLines.forEach((line, index) => {\n"
        "            renderLineAsHTML(index);\n"
        "        });\n"
        "        \n"
        "        // Then set first line as current (markdown)\n"
        "        setTimeout(() => {\n"
        "            currentLineIndex = 0;\n"
        "            showLineAsMarkdownWithPosition(0, 0);\n"
        "            updateLineClasses();\n"
        "            \n"
        "            // Focus first line\n"
        "            if (editorLines[0]) {\n"
        "                editorLines[0].focus();\n"
        "            }\n"
        "        }, 100);\n"
        "    </script>\n"
        "</body>\n"
        "</html>";
        
        [webView loadHTMLString:htmlContent baseURL:nil];
        [mainView addSubview:webView];
        [window setContentView:mainView];
        
        [window center];
        [window makeKeyAndOrderFront:nil];
        [app activateIgnoringOtherApps:YES];
        
        NSLog(@"🚀 Perfect Hybrid launched - EXACT copy of web version!");
        
        [app run];
    }
    
    return 0;
}