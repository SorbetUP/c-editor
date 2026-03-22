#import <Cocoa/Cocoa.h>
#import <WebKit/WebKit.h>

int main(int argc, const char * argv[]) {
    @autoreleasepool {
        NSApplication *app = [NSApplication sharedApplication];
        [app setActivationPolicy:NSApplicationActivationPolicyRegular];
        
        // Create basic window
        NSRect frame = NSMakeRect(100, 100, 900, 600);
        NSWindow *window = [[NSWindow alloc] 
            initWithContentRect:frame
            styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable)
            backing:NSBackingStoreBuffered 
            defer:NO];
        
        [window setTitle:@"Debug Test - Step by Step"];
        [window setBackgroundColor:[NSColor colorWithRed:0.102 green:0.102 blue:0.102 alpha:1.0]];
        
        // Create WebView
        NSRect webFrame = NSMakeRect(20, 20, 860, 560);
        WKWebViewConfiguration *config = [[WKWebViewConfiguration alloc] init];
        WKWebView *webView = [[WKWebView alloc] initWithFrame:webFrame configuration:config];
        [webView setAutoresizingMask:NSViewWidthSizable | NSViewHeightSizable];
        
        // VERY SIMPLE test - just test rendering without interaction first
        NSString *htmlContent = @"<!DOCTYPE html>\n"
        "<html>\n"
        "<head>\n"
        "    <meta charset=\"UTF-8\">\n"
        "    <style>\n"
        "        body {\n"
        "            background: #1a1a1a;\n"
        "            color: #e0e0e0;\n"
        "            font-family: Monaco, monospace;\n"
        "            padding: 20px;\n"
        "            font-size: 14px;\n"
        "        }\n"
        "        .test-line {\n"
        "            margin: 10px 0;\n"
        "            padding: 5px;\n"
        "            border-left: 3px solid #666;\n"
        "        }\n"
        "        h1 { color: #4c6ef5; font-size: 24px; }\n"
        "        h2 { color: #4c6ef5; font-size: 20px; }\n"
        "        strong { color: #51cf66; font-weight: bold; }\n"
        "        em { color: #ffd43b; font-style: italic; }\n"
        "        u { color: #ff6b6b; text-decoration: underline; }\n"
        "        mark { background: #ffd43b; color: #000; padding: 2px; }\n"
        "    </style>\n"
        "</head>\n"
        "<body>\n"
        "    <h2>Test de Rendu - Étape par Étape</h2>\n"
        "    \n"
        "    <div class=\"test-line\">\n"
        "        <strong>Test 1 - Markdown brut:</strong><br>\n"
        "        # Titre<br>\n"
        "        **Gras**<br>\n"
        "        *Italique*<br>\n"
        "        ==Surligné==<br>\n"
        "        ++Souligné++\n"
        "    </div>\n"
        "    \n"
        "    <div class=\"test-line\">\n"
        "        <strong>Test 2 - HTML rendu:</strong><br>\n"
        "        <h1>Titre</h1>\n"
        "        <strong>Gras</strong><br>\n"
        "        <em>Italique</em><br>\n"
        "        <mark>Surligné</mark><br>\n"
        "        <u>Souligné</u>\n"
        "    </div>\n"
        "    \n"
        "    <div class=\"test-line\">\n"
        "        <strong>Test 3 - Conversion JavaScript:</strong><br>\n"
        "        <div id=\"markdown-input\">**Test** *italique* ==surligné== ++souligné++</div>\n"
        "        <div id=\"html-output\">Chargement...</div>\n"
        "    </div>\n"
        "    \n"
        "    <button onclick=\"testConversion()\">Tester la Conversion</button>\n"
        "    <button onclick=\"testLineSwitch()\">Tester Changement de Ligne</button>\n"
        "    \n"
        "    <div id=\"debug-info\" style=\"margin-top: 20px; font-size: 12px; color: #888;\"></div>\n"
        "    \n"
        "    <script>\n"
        "        function markdownToHtml(text) {\n"
        "            console.log('Converting:', text);\n"
        "            let html = text;\n"
        "            \n"
        "            // Headers\n"
        "            html = html.replace(/^# (.+)$/gm, '<h1>$1</h1>');\n"
        "            html = html.replace(/^## (.+)$/gm, '<h2>$1</h2>');\n"
        "            \n"
        "            // Bold\n"
        "            html = html.replace(/\\*\\*([^*]+)\\*\\*/g, '<strong>$1</strong>');\n"
        "            \n"
        "            // Italic\n"
        "            html = html.replace(/\\*([^*]+)\\*/g, '<em>$1</em>');\n"
        "            \n"
        "            // Highlight\n"
        "            html = html.replace(/==([^=]+)==/g, '<mark>$1</mark>');\n"
        "            \n"
        "            // Underline\n"
        "            html = html.replace(/\\+\\+([^+]+)\\+\\+/g, '<u>$1</u>');\n"
        "            \n"
        "            console.log('Result:', html);\n"
        "            return html;\n"
        "        }\n"
        "        \n"
        "        function testConversion() {\n"
        "            const input = document.getElementById('markdown-input').textContent;\n"
        "            const output = markdownToHtml(input);\n"
        "            document.getElementById('html-output').innerHTML = output;\n"
        "            document.getElementById('debug-info').innerHTML = \n"
        "                'Input: ' + input + '<br>Output: ' + output;\n"
        "        }\n"
        "        \n"
        "        function testLineSwitch() {\n"
        "            const line = document.getElementById('markdown-input');\n"
        "            const currentContent = line.textContent;\n"
        "            \n"
        "            if (line.innerHTML.includes('<')) {\n"
        "                // Currently HTML, switch to markdown\n"
        "                line.innerHTML = '';\n"
        "                line.textContent = currentContent;\n"
        "                document.getElementById('debug-info').innerHTML = 'Switched to markdown: ' + currentContent;\n"
        "            } else {\n"
        "                // Currently markdown, switch to HTML\n"
        "                const html = markdownToHtml(currentContent);\n"
        "                line.innerHTML = html;\n"
        "                document.getElementById('debug-info').innerHTML = 'Switched to HTML: ' + html;\n"
        "            }\n"
        "        }\n"
        "        \n"
        "        // Test conversion on load\n"
        "        setTimeout(testConversion, 1000);\n"
        "    </script>\n"
        "</body>\n"
        "</html>";
        
        [webView loadHTMLString:htmlContent baseURL:nil];
        [[window contentView] addSubview:webView];
        
        // Show window
        [window center];
        [window makeKeyAndOrderFront:nil];
        [app activateIgnoringOtherApps:YES];
        
        NSLog(@"🚀 Debug test ready! Test each step.");
        
        // Run app
        [app run];
    }
    
    return 0;
}